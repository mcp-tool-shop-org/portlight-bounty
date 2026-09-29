#!/usr/bin/env python3
"""Compare the Python oracle and the Rust port on the same action scripts.

When the Python game is importable (PYTHONPATH points at its src/), this
regenerates each snapshot and diffs it against `portlight script`. It also
checks the Rust binary against the committed golden files so CI can pass
without the private game checkout.

The live comparison (oracle, GameSession, and save round-trips) includes the
contract board, captain memories, active bounties, deferred fees, and the
ledger. Those keys are stripped before the golden comparison. Goldens stay
the narrow shape. `parity/expected_divergences.json` names the JSON paths
that may differ. CI prints that list. Any other difference fails.

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
        shared = min(len(a), len(b))
        for i in range(shared):
            errors.extend(close(a[i], b[i], f"{path}[{i}]"))
        if len(a) > len(b):
            for i in range(shared, len(a)):
                errors.append(f"{path}[{i}]: {a[i]!r} != <missing>")
        elif len(b) > len(a):
            for i in range(shared, len(b)):
                errors.append(f"{path}[{i}]: <missing> != {b[i]!r}")
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
        for key in ("script", "check", "summary", "rust", "python", "paths"):
            if key not in entry:
                raise SystemExit(f"{DIVERGENCES} entry missing {key}: {entry}")
        if not entry["paths"]:
            raise SystemExit(f"{DIVERGENCES} entry has no paths: {entry['script']}")
        for spec in entry["paths"]:
            if not isinstance(spec, dict) or not {"path", "python", "rust"} <= set(spec):
                raise SystemExit(
                    f"{DIVERGENCES} path must pin python and rust values: {entry['script']} {spec}"
                )
    return entries


def print_divergence_list(entries: list[dict]) -> None:
    print(f"expected divergences ({len(entries)}):")
    if not entries:
        print("  (none)")
        return
    for entry in entries:
        print(f"  {entry['script']} [{entry['check']}]: {entry['summary']}")
        bits = [
            f"{spec['path']} python={spec['python']!r} rust={spec['rust']!r}"
            for spec in entry["paths"]
        ]
        print(f"    paths {'; '.join(bits)}")
        print(f"    rust {entry['rust']}")
        print(f"    python {entry['python']}")
        if entry.get("oracle"):
            print(f"    oracle {entry['oracle']}")


def listed(entries: list[dict], name: str, check: str) -> list[dict]:
    return [entry for entry in entries if entry["script"] == name and entry["check"] == check]


# Prefixes added by roundtrip_errors. Stripped only when they start the line.
# A ` $` inside a compared value is not a prefix.
ROUNDTRIP_PREFIXES = (
    "rust-save/python-load ",
    "python-save/rust-load ",
    "rust-save/python-save ",
)


def error_path(line: str) -> str:
    """The JSON path of a close() line.

    A round-trip line starts with one of ROUNDTRIP_PREFIXES and then `$...`.
    The path ends at the first `: `. A ` $` later in the value is left alone.
    """
    text = line
    for prefix in ROUNDTRIP_PREFIXES:
        if text.startswith(prefix):
            text = text[len(prefix) :]
            break
    if text.startswith("$"):
        return text.split(": ", 1)[0]
    return text


def path_matches(path: str, pattern: str) -> bool:
    """Exact leaf match, plus trailing `[n]` indexes and nothing else.

    `$.board.offers[1].id` matches that path. `$.board.offers[1].tags[2]`
    matches a pattern of the same path, or that path plus further indexes.
    `$.board.offers` does not match `$.board.offers[0].id`, and `$.log` does
    not match `$.log[0].command`.
    """
    import re

    if path == pattern:
        return True
    if not path.startswith(pattern):
        return False
    rest = path[len(pattern) :]
    return re.fullmatch(r"(?:\[\d+\])+", rest) is not None


def partition(errors: list[str], patterns: list[str]) -> tuple[list[str], list[str], list[str]]:
    allowed = [line for line in errors if any(path_matches(error_path(line), pat) for pat in patterns)]
    unexpected = [line for line in errors if line not in allowed]
    missing = [
        pat
        for pat in patterns
        if not any(path_matches(error_path(line), pat) for line in allowed)
    ]
    return allowed, unexpected, missing


_MISSING = object()


def lookup(value, path: str):
    """Value at a `$...` path, or `_MISSING` when an index or key is absent."""
    if path == "$":
        return value
    if not path.startswith("$"):
        raise ValueError(path)
    cur = value
    i = 1
    while i < len(path):
        if path[i] == ".":
            i += 1
            end = i
            while end < len(path) and path[end] not in ".[":
                end += 1
            key = path[i:end]
            if not isinstance(cur, dict) or key not in cur:
                return _MISSING
            cur = cur[key]
            i = end
        elif path[i] == "[":
            end = path.find("]", i)
            if end < 0:
                raise ValueError(path)
            idx = int(path[i + 1 : end])
            if not isinstance(cur, list) or idx < 0 or idx >= len(cur):
                return _MISSING
            cur = cur[idx]
            i = end + 1
        else:
            raise ValueError(path)
    return cur


def is_missing_pin(expected) -> bool:
    return (
        isinstance(expected, dict)
        and list(expected) == ["$missing"]
        and expected["$missing"] is True
    )


def values_equal(actual, expected) -> bool:
    if actual is _MISSING:
        return is_missing_pin(expected)
    if is_missing_pin(expected):
        return False
    return not close(actual, expected, "$")


def show_value(actual) -> str:
    if actual is _MISSING:
        return "<missing>"
    return repr(actual)


def container_patterns(left, right, patterns: list[str]) -> list[str]:
    """Patterns whose value is a list or object. Those hide every field under them."""
    bad = []
    for pattern in patterns:
        for snap in (left, right):
            found = lookup(snap, pattern)
            if found is not _MISSING and isinstance(found, (dict, list)):
                bad.append(pattern)
                break
    return bad


def drop_meta(snap: dict) -> dict:
    """Golden files may carry `_source` / `_note`. Those are not snapshot fields."""
    return {key: value for key, value in snap.items() if not key.startswith("_")}


def _stabilize(new, old):
    """Reorder objects to the existing golden's key order when the keys match."""
    if isinstance(new, dict) and isinstance(old, dict):
        out = {}
        for key in old:
            if key in new:
                out[key] = _stabilize(new[key], old[key])
        for key, value in new.items():
            if key not in out:
                out[key] = value
        return out
    if isinstance(new, list) and isinstance(old, list):
        return [
            _stabilize(item, old[i]) if i < len(old) else item for i, item in enumerate(new)
        ]
    return new


def golden_text(oracle_snap: dict, existing_text: str | None) -> str:
    """Bytes `--write-golden` would write.

    An unchanged snapshot keeps the committed text, including `_source` and
    `_note`. A changed snapshot still carries those keys.
    """
    existing = json.loads(existing_text) if existing_text is not None else None
    body = {key: value for key, value in narrow(oracle_snap).items() if not key.startswith("_")}
    if existing_text is not None and not close(drop_meta(existing), body, "$"):
        return existing_text if existing_text.endswith("\n") else existing_text + "\n"
    meta = {}
    if existing:
        meta = {key: value for key, value in existing.items() if key.startswith("_")}
    ordered = _stabilize(body, drop_meta(existing) if existing else {})
    written = dict(meta)
    written.update(ordered)
    ensure_ascii = existing_text is None or existing_text.isascii()
    return json.dumps(written, indent=2, ensure_ascii=ensure_ascii) + "\n"


def report(
    name: str,
    check: str,
    errors: list[str],
    entries: list[dict],
    failed: list[int],
    left=None,
    right=None,
) -> None:
    known = listed(entries, name, check)
    specs: list[dict] = []
    for entry in known:
        specs.extend(entry["paths"])
    patterns = [spec["path"] for spec in specs]
    problems: list[str] = []
    if left is not None and right is not None:
        for pattern in container_patterns(left, right, patterns):
            problems.append(f"allowlist path names a list or object: {pattern}")
    allowed, unexpected, missing = partition(errors, patterns)
    if unexpected:
        label = f"{check} mismatch"
        if allowed:
            label += f" ({len(unexpected)} outside the allowlist, {len(allowed)} allowed)"
        else:
            label += f" ({len(unexpected)} differences)"
        detail = "\n".join(f"    {line}" for line in unexpected[:30])
        problems.append(f"{label}\n{detail}" if detail else label)
    if known and (not allowed or missing) and not any(line.startswith("allowlist path") for line in problems):
        # A container pattern is rejected above; do not also call it "missing".
        real_missing = [
            pat
            for pat in missing
            if left is None
            or right is None
            or pat not in container_patterns(left, right, [pat])
        ]
        if not allowed or real_missing:
            text = f"expected {check} divergence did not reproduce"
            if real_missing:
                text += f"\n    paths that did not differ: {', '.join(real_missing)}"
            problems.append(text)
    if left is not None and right is not None:
        for spec in specs:
            if spec["path"] in container_patterns(left, right, [spec["path"]]):
                continue
            py = lookup(left, spec["path"])
            rs = lookup(right, spec["path"])
            if not values_equal(py, spec["python"]) or not values_equal(rs, spec["rust"]):
                problems.append(
                    f"{spec['path']} is not the pinned pair\n"
                    f"    python {show_value(py)} pinned {spec['python']!r}\n"
                    f"    rust {show_value(rs)} pinned {spec['rust']!r}"
                )
    if problems:
        failed[0] += 1
        print(f"{name}: {check} mismatch")
        for problem in problems:
            for line in problem.splitlines():
                print(f"    {line}" if not line.startswith("    ") else line)
        return
    if known:
        print(f"{name}: expected {check} divergence ({len(allowed)} allowlisted differences)")
        for line in allowed[:12]:
            print("   ", line)
        for entry in known:
            print(f"    rust {entry['rust']}")
            print(f"    python {entry['python']}")
            if entry.get("oracle"):
                print(f"    oracle {entry['oracle']}")
        return
    print(f"{name}: {check} matches")


def roundtrip_errors(name: str, script_path: str) -> list[str]:
    """Compare loaded snapshots, not the pre-save snapshot.

    `Session::load` and `GameSession.load` recalculate port prices with the
    captain's modifiers. A fresh game has not done that yet, so the loaded
    snapshot can differ from the live one even when both loaders agree.
    """
    errors: list[str] = []
    with tempfile.TemporaryDirectory(prefix=f"pl-rt-{name.replace('.txt', '')}-") as base:
        rust_dir = os.path.join(base, "from-rust")
        py_dir = os.path.join(base, "from-python")
        os.makedirs(rust_dir)
        os.makedirs(py_dir)
        run_rust_saved(script_path, rust_dir, "slot")
        py_of_rust = run_python_load(rust_dir, "slot")
        rust_reloaded = run_rust_load(rust_dir, "slot")
        errors.extend(
            f"{ROUNDTRIP_PREFIXES[0]}{line}"
            for line in close(without_log(rust_reloaded), without_log(py_of_rust), "$")
        )
        run_oracle_saved(script_path, py_dir, "slot")
        rust_of_py = run_rust_load(py_dir, "slot")
        py_reloaded = run_python_load(py_dir, "slot")
        errors.extend(
            f"{ROUNDTRIP_PREFIXES[1]}{line}"
            for line in close(without_log(py_reloaded), without_log(rust_of_py), "$")
        )
        errors.extend(
            f"{ROUNDTRIP_PREFIXES[2]}{line}"
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
    counts = {
        "oracle": 0,
        "gamesession_run": 0,
        "gamesession_skipped": 0,
        "roundtrip_run": 0,
        "roundtrip_skipped": 0,
    }
    if args.require_oracle:
        try:
            import portlight  # noqa: F401
        except ImportError as exc:
            raise SystemExit(f"Python game not importable (--require-oracle): {exc}") from exc
    for name in scripts():
        script_path = os.path.join(SCRIPTS, name)
        golden_path = os.path.join(GOLDEN, name.replace(".txt", ".json"))
        with tempfile.TemporaryDirectory(prefix=f"pl-script-{name.replace('.txt', '')}-") as save_root:
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
                    counts["oracle"] += 1
                    report(name, "oracle", close(oracle, rust, "$"), entries, failed, oracle, rust)
                    session = run_session(script_path, os.path.join(save_root, "session"))
                    if session is None:
                        counts["gamesession_skipped"] += 1
                        if listed(entries, name, "gamesession"):
                            failed[0] += 1
                            print(f"{name}: expected gamesession divergence but the runner skipped the script")
                        else:
                            print(f"{name}: gamesession skipped")
                    else:
                        counts["gamesession_run"] += 1
                        session_view = without_log(session)
                        rust_view = without_log(rust)
                        report(
                            name,
                            "gamesession",
                            close(session_view, rust_view, "$"),
                            entries,
                            failed,
                            session_view,
                            rust_view,
                        )
                    if has_session(rust) and not args.skip_roundtrip:
                        counts["roundtrip_run"] += 1
                        report(name, "roundtrip", roundtrip_errors(name, script_path), entries, failed)
                    elif not has_session(rust):
                        counts["roundtrip_skipped"] += 1
                        if listed(entries, name, "roundtrip"):
                            failed[0] += 1
                            print(f"{name}: expected roundtrip divergence but there is no session")
                        else:
                            print(f"{name}: roundtrip skipped (no session)")
                    elif args.skip_roundtrip:
                        counts["roundtrip_skipped"] += 1
                    if args.write_golden:
                        existing_text = None
                        if os.path.exists(golden_path):
                            with open(golden_path, encoding="utf-8") as fh:
                                existing_text = fh.read()
                        rendered = golden_text(oracle, existing_text)
                        if rendered != existing_text:
                            with open(golden_path, "w", encoding="utf-8") as fh:
                                fh.write(rendered)
                        print(f"  wrote {golden_path}")
        if os.path.exists(golden_path):
            golden = load_json(golden_path)
            python_golden = golden.get("_source") == "python-oracle"
            errors = close(drop_meta(golden), narrow(rust), "$")
            if python_golden:
                # The file is the Python snapshot. The oracle allowlist names
                # the fields where Rust is known to differ; anything else fails.
                print(f"{name}: golden is the Python oracle snapshot")
                py_view = drop_meta(golden)
                rust_view = narrow(rust)
                report(name, "oracle", errors, entries, failed, py_view, rust_view)
            elif errors:
                failed[0] += 1
                print(f"{name}: golden != rust ({len(errors)} differences)")
                for line in errors[:20]:
                    print("   ", line)
            else:
                print(f"{name}: golden matches rust")
        elif args.golden_only:
            failed[0] += 1
            print(f"{name}: missing golden {golden_path}")
    print(
        "counts: oracle {oracle}, gamesession run {gamesession_run} skipped {gamesession_skipped}, "
        "roundtrip run {roundtrip_run} skipped {roundtrip_skipped}".format(**counts)
    )
    print_divergence_list(entries)
    return 1 if failed[0] else 0


if __name__ == "__main__":
    raise SystemExit(main())
