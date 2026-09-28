#!/usr/bin/env python3
"""Compare the Python oracle and the Rust port on the same action scripts.

When the Python game is importable (PYTHONPATH points at its src/), this
regenerates each snapshot and diffs it against `portlight script`. It also
checks the Rust binary against the committed golden files so CI can pass
without the private game checkout.

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

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
SCRIPTS = os.path.join(ROOT, "parity", "scripts")
GOLDEN = os.path.join(ROOT, "parity", "golden")
ORACLE = os.path.join(ROOT, "tools", "parity", "oracle.py")


def rust_bin() -> str:
    override = os.environ.get("PORTLIGHT_BIN")
    if override:
        return override
    return os.path.join(ROOT, "target", "debug", "portlight")


def load_json(path: str) -> dict:
    with open(path, encoding="utf-8") as fh:
        return json.load(fh)


def run_rust(script_path: str) -> dict:
    proc = subprocess.run(
        [rust_bin(), "script", script_path],
        check=False,
        capture_output=True,
        text=True,
    )
    if not proc.stdout.strip():
        raise SystemExit(f"rust produced no output for {script_path}: {proc.stderr}")
    return json.loads(proc.stdout)


def run_oracle(script_path: str) -> dict:
    env = os.environ.copy()
    proc = subprocess.run(
        [sys.executable, ORACLE, script_path],
        check=False,
        capture_output=True,
        text=True,
        env=env,
    )
    if proc.returncode != 0 or not proc.stdout.strip():
        raise SystemExit(f"oracle failed for {script_path}:\n{proc.stderr}")
    return json.loads(proc.stdout)


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


def scripts() -> list[str]:
    names = sorted(fn for fn in os.listdir(SCRIPTS) if fn.endswith(".txt"))
    return names


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--golden-only", action="store_true")
    parser.add_argument("--write-golden", action="store_true", help="overwrite golden files from the oracle")
    args = parser.parse_args()
    if not os.path.exists(rust_bin()):
        raise SystemExit(f"missing {rust_bin()}; run cargo build -p portlight-cli")
    os.makedirs(GOLDEN, exist_ok=True)
    failed = 0
    for name in scripts():
        script_path = os.path.join(SCRIPTS, name)
        golden_path = os.path.join(GOLDEN, name.replace(".txt", ".json"))
        rust = run_rust(script_path)
        if not args.golden_only:
            try:
                import portlight  # noqa: F401
            except ImportError:
                print(f"{name}: Python game not importable; comparing Rust to golden only")
                args.golden_only = True
            else:
                oracle = run_oracle(script_path)
                errors = close(oracle, rust, "$")
                if errors:
                    failed += 1
                    print(f"{name}: oracle != rust ({len(errors)} differences)")
                    for line in errors[:30]:
                        print("  ", line)
                else:
                    print(f"{name}: oracle matches rust")
                if args.write_golden:
                    with open(golden_path, "w", encoding="utf-8") as fh:
                        json.dump(oracle, fh, indent=2)
                        fh.write("\n")
                    print(f"  wrote {golden_path}")
        if os.path.exists(golden_path):
            golden = load_json(golden_path)
            errors = close(golden, rust, "$")
            if errors:
                failed += 1
                print(f"{name}: golden != rust ({len(errors)} differences)")
                for line in errors[:20]:
                    print("  ", line)
            else:
                print(f"{name}: golden matches rust")
        elif args.golden_only:
            failed += 1
            print(f"{name}: missing golden {golden_path}")
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
