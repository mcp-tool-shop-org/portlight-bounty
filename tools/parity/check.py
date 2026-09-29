#!/usr/bin/env python3
"""Compare the Python oracle and the Rust port on the same action scripts.

When the Python game is importable (PYTHONPATH points at its src/), this
regenerates each snapshot and diffs it against `portlight script`. It also
checks the Rust binary against the committed golden files so CI can pass
without the private game checkout.

The live comparison (oracle, GameSession, and save round-trips) includes the
contract board, captain memories, active bounties, deferred fees, and the
ledger. Those keys are stripped before the golden comparison. Goldens stay
the narrow shape. `parity/expected_divergences.json` names scripts whose
live check is a known Rust-versus-Python divergence. CI prints that list
and does not fail those scripts.

Usage (from the repo root, after `cargo build -p portlight-cli`):

    PYTHONPATH=/path/to/portlight/src python3 tools/parity/check.py
    python3 tools/parity/check.py --golden-only
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
SCRIPTS = os.path.join(ROOT, "parity", "scripts")
GOLDEN = os.path.join(ROOT, "parity", "golden")
ORACLE = os.path.join(ROOT, "tools", "parity", "oracle.py")
SESSION = os.path.join(ROOT, "tools", "parity", "session_runner.py")
SAVE_SLOT = os.path.join(ROOT, "tools", "parity", "save_slot.py")
DIVERGENCES = os.path.join(ROOT, "parity", "expected_divergences.json")

# Present in the live snapshot when PORTLIGHT_WIDE_SNAPSHOT=1. Omitted from
# parity/golden so the committed files keep the narrow shape.
WIDE_KEYS = ("board", "captain_memories", "active_bounties", "deferred_fees", "ledger")


def rust_bin() -> str:
    override = os.environ.get("PORTLIGHT_BIN")
    if override:
        return override
    return os.path.join(ROOT, "target", "debug", "portlight")


def load_json(path: str) -> dict:
    with open(path, encoding="utf-8") as fh:
        return json.load(fh)


def python_env(save_root: str | None = None) -> dict:
    env = os.environ.copy()
    # Trade-intelligence consequences draw rng.choice(list(set_of_ports)).
    # CPython 3.12 set order follows PYTHONHASHSEED. The Rust side matches seed 0.
    env["PYTHONHASHSEED"] = "0"
    if save_root:
        env["PORTLIGHT_SAVE_ROOT"] = save_root
    return env


def rust_env(save_root: str | None = None) -> dict:
    env = os.environ.copy()
    env["PORTLIGHT_WIDE_SNAPSHOT"] = "1"
    if save_root:
        env["PORTLIGHT_SAVE_ROOT"] = save_root
    return env


def run_json(argv: list[str], env: dict, label: str) -> dict:
    proc = subprocess.run(argv, check=False, capture_output=True, text=True, env=env)
    if not proc.stdout.strip():
        raise SystemExit(f"{label} produced no output:\n{proc.stderr}")
    try:
        return json.loads(proc.stdout)
    except json.JSONDecodeError as exc:
        raise SystemExit(f"{label} did not print JSON:\n{proc.stdout[:500]}\n{proc.stderr}") from exc


def run_rust(script_path: str, save_root: str) -> dict:
    return run_json(
        [rust_bin(), "script", script_path],
        rust_env(save_root),
        f"rust {os.path.basename(script_path)}",
    )


def run_oracle(script_path: str, save_root: str) -> dict:
    return run_json(
        [sys.executable, ORACLE, script_path],
        python_env(save_root),
        f"oracle {os.path.basename(script_path)}",
    )


def run_session(script_path: str, save_root: str) -> dict | None:
    """None means the script uses a verb GameSession does not implement."""
    proc = subprocess.run(
        [sys.executable, SESSION, script_path],
        check=False,
        capture_output=True,
        text=True,
        env=python_env(save_root),
    )
    if proc.returncode == 2:
        return None
    if proc.returncode != 0 or not proc.stdout.strip():
        raise SystemExit(
            f"session runner failed for {script_path} ({proc.returncode}):\n{proc.stderr}"
        )
    return json.loads(proc.stdout)


def run_rust_saved(script_path: str, directory: str, slot: str) -> dict:
    return run_json(
        [rust_bin(), "script", script_path, "--save-dir", directory, "--slot", slot],
        rust_env(directory),
        f"rust save {os.path.basename(script_path)}",
    )


def run_oracle_saved(script_path: str, directory: str, slot: str) -> dict:
    return run_json(
        [sys.executable, ORACLE, script_path, "--save-dir", directory, "--slot", slot],
        python_env(directory),
        f"oracle save {os.path.basename(script_path)}",
    )


def run_rust_load(directory: str, slot: str) -> dict:
    return run_json(
        [rust_bin(), "load", directory, slot],
        rust_env(),
        f"rust load {slot}",
    )


def run_python_load(directory: str, slot: str) -> dict:
    return run_json(
        [sys.executable, SAVE_SLOT, directory, slot],
        python_env(),
        f"python load {slot}",
    )


def close(a, b, path: str) -> list[str]:
    if isinstance(a, bool) or isinstance(b, bool):
        if a != b:
            return [f"{path}: {a!r} != {b!r}"]
        return []
    if isinstance(a, (int, float)) and isinstance(b, (int, float)):
        af, bf = float(a), float(b)
        diff = abs(af - bf)
        if diff <= 1e-6 or diff <= 1e-9 * max(abs(af), abs(bf)):
            return []
        return [f"{path}: {a!r} != {b!r}"]
    if isinstance(a, str) and isinstance(b, str):
        if a != b:
            return [f"{path}: {a!r} != {b!r}"]
        return []
    if a is None and b is None:
        return []
    if isinstance(a, list) and isinstance(b, list):
        errors = []
        if len(a) != len(b):
            return [f"{path}: length {len(a)} != {len(b)}"]
        for i, (x, y) in enumerate(zip(a, b)):
            errors.extend(close(x, y, f"{path}[{i}]"))
        return errors
    if isinstance(a, dict) and isinstance(b, dict):
        errors = []
        if set(a) != set(b):
            return [f"{path}: keys {sorted(set(a) ^ set(b))}"]
        for key in a:
            errors.extend(close(a[key], b[key], f"{path}.{key}"))
        return errors
    if a != b:
        return [f"{path}: {a!r} != {b!r}"]
    return []


def narrow(snap: dict) -> dict:
    return {key: value for key, value in snap.items() if key not in WIDE_KEYS}


def without_log(snap: dict) -> dict:
    return {key: value for key, value in snap.items() if key != "log"}


def has_session(snap: dict) -> bool:
    captain = snap.get("captain") or {}
    return bool(captain.get("name"))


def scripts() -> list[str]:
    names = sorted(fn for fn in os.listdir(SCRIPTS) if fn.endswith(".txt"))
    return names


def load_divergences() -> list[dict]:
    if not os.path.exists(DIVERGENCES):
        return []
    data = load_json(DIVERGENCES)
    entries = data.get("entries", [])
    for entry in entries:
        for key in ("script", "check", "summary", "rust", "python"):
            if key not in entry:
                raise SystemExit(f"{DIVERGENCES} entry missing {key}: {entry}")
    return entries


def print_divergence_list(entries: list[dict]) -> None:
    print(f"expected divergences ({len(entries)}):")
    if not entries:
        print("  (none)")
        return
    for entry in entries:
        print(f"  {entry['script']} [{entry['check']}]: {entry['summary']}")
        print(f"    rust {entry['rust']}")
        print(f"    python {entry['python']}")
        if entry.get("oracle"):
            print(f"    oracle {entry['oracle']}")


def listed(entries: list[dict], name: str, check: str) -> list[dict]:
    return [entry for entry in entries if entry["script"] == name and entry["check"] == check]


def report(name: str, check: str, errors: list[str], entries: list[dict], failed: list[int]) -> None:
    known = listed(entries, name, check)
    if errors and known:
        print(f"{name}: expected {check} divergence ({len(errors)} differences)")
        for line in errors[:12]:
            print("   ", line)
        for entry in known:
            print(f"    rust {entry['rust']}")
            print(f"    python {entry['python']}")
            if entry.get("oracle"):
                print(f"    oracle {entry['oracle']}")
        return
    if errors:
        failed[0] += 1
        print(f"{name}: {check} mismatch ({len(errors)} differences)")
        for line in errors[:30]:
            print("   ", line)
        return
    if known:
        failed[0] += 1
        print(f"{name}: expected {check} divergence did not reproduce")
        return
    print(f"{name}: {check} matches")


def roundtrip_errors(name: str, script_path: str) -> list[str]:
    """Compare loaded snapshots, not the pre-save snapshot.

    `Session::load` and `GameSession.load` recalculate port prices with the
    captain's modifiers. A fresh game has not done that yet, so the loaded
    snapshot can differ from the live one even when both loaders agree.
    """
    base = tempfile.mkdtemp(prefix=f"pl-rt-{name.replace('.txt', '')}-")
    errors: list[str] = []
    rust_dir = os.path.join(base, "from-rust")
    py_dir = os.path.join(base, "from-python")
    os.makedirs(rust_dir)
    os.makedirs(py_dir)
    run_rust_saved(script_path, rust_dir, "slot")
    py_of_rust = run_python_load(rust_dir, "slot")
    rust_reloaded = run_rust_load(rust_dir, "slot")
    errors.extend(
        f"rust-save/python-load {line}"
        for line in close(without_log(rust_reloaded), without_log(py_of_rust), "$")
    )
    run_oracle_saved(script_path, py_dir, "slot")
    rust_of_py = run_rust_load(py_dir, "slot")
    py_reloaded = run_python_load(py_dir, "slot")
    errors.extend(
        f"python-save/rust-load {line}"
        for line in close(without_log(py_reloaded), without_log(rust_of_py), "$")
    )
    errors.extend(
        f"rust-save/python-save {line}"
        for line in close(without_log(rust_reloaded), without_log(rust_of_py), "$")
    )
    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--golden-only", action="store_true")
    parser.add_argument("--write-golden", action="store_true", help="overwrite golden files from the oracle")
    parser.add_argument(
        "--require-oracle",
        action="store_true",
        help="fail if the Python game cannot be imported (no golden-only fallback)",
    )
    parser.add_argument(
        "--skip-roundtrip",
        action="store_true",
        help="skip save/load round-trips (debug only; CI does not pass this)",
    )
    args = parser.parse_args()
    if not os.path.exists(rust_bin()):
        raise SystemExit(f"missing {rust_bin()}; run cargo build -p portlight-cli")
    os.makedirs(GOLDEN, exist_ok=True)
    entries = load_divergences()
    print_divergence_list(entries)
    known_scripts = set(scripts())
    for entry in entries:
        if entry["script"] not in known_scripts:
            raise SystemExit(f"expected divergence names missing script {entry['script']}")
    failed = [0]
    if args.require_oracle:
        try:
            import portlight  # noqa: F401
        except ImportError as exc:
            raise SystemExit(f"Python game not importable (--require-oracle): {exc}") from exc
    for name in scripts():
        script_path = os.path.join(SCRIPTS, name)
        golden_path = os.path.join(GOLDEN, name.replace(".txt", ".json"))
        save_root = tempfile.mkdtemp(prefix=f"pl-script-{name.replace('.txt', '')}-")
        rust = run_rust(script_path, os.path.join(save_root, "rust"))
        if not args.golden_only:
            try:
                import portlight  # noqa: F401
            except ImportError as exc:
                if args.require_oracle:
                    raise SystemExit(
                        f"Python game not importable (--require-oracle): {exc}"
                    ) from exc
                print(f"{name}: Python game not importable; comparing Rust to golden only")
                args.golden_only = True
            else:
                oracle = run_oracle(script_path, os.path.join(save_root, "oracle"))
                report(name, "oracle", close(oracle, rust, "$"), entries, failed)
                session = run_session(script_path, os.path.join(save_root, "session"))
                if session is None:
                    if listed(entries, name, "gamesession"):
                        failed[0] += 1
                        print(f"{name}: expected gamesession divergence but the runner skipped the script")
                    else:
                        print(f"{name}: gamesession skipped")
                else:
                    report(
                        name,
                        "gamesession",
                        close(without_log(session), without_log(rust), "$"),
                        entries,
                        failed,
                    )
                if has_session(rust) and not args.skip_roundtrip:
                    report(name, "roundtrip", roundtrip_errors(name, script_path), entries, failed)
                elif not has_session(rust):
                    if listed(entries, name, "roundtrip"):
                        failed[0] += 1
                        print(f"{name}: expected roundtrip divergence but there is no session")
                    else:
                        print(f"{name}: roundtrip skipped (no session)")
                if args.write_golden:
                    with open(golden_path, "w", encoding="utf-8") as fh:
                        json.dump(narrow(oracle), fh, indent=2)
                        fh.write("\n")
                    print(f"  wrote {golden_path}")
        if os.path.exists(golden_path):
            golden = load_json(golden_path)
            errors = close(golden, narrow(rust), "$")
            if errors:
                failed[0] += 1
                print(f"{name}: golden != rust ({len(errors)} differences)")
                for line in errors[:20]:
                    print("   ", line)
            else:
                print(f"{name}: golden matches rust")
        elif args.golden_only:
            failed[0] += 1
            print(f"{name}: missing golden {golden_path}")
    print_divergence_list(entries)
    return 1 if failed[0] else 0


if __name__ == "__main__":
    raise SystemExit(main())
