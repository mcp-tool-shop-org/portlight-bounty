#!/usr/bin/env python3
"""Check `list_save_slots` against the hand-edited fixture folder.

`parity/fixtures/save_slots/saves` holds 25 JSON files. Rust's
`list_save_slots` and Python's `list_save_slots` disagree on 8 of them.
Three of those eight make Python raise out of the JSON `try`, so the whole
listing dies; Rust skips the file and keeps going. The other five still
produce a row on the Python side. `parity/golden/save_slots.json` records
both results. This script checks the Python half. The Rust test checks the
Rust half and does not change the skip.

    PYTHONPATH=/path/to/portlight/src python3 tools/parity/save_slots.py --check
"""

from __future__ import annotations

import json
import shutil
import sys
import tempfile
from pathlib import Path

from portlight.app.session import list_save_slots


ROOT = Path(__file__).resolve().parents[2]
FIXTURE = ROOT / "parity" / "fixtures" / "save_slots"
GOLDEN = ROOT / "parity" / "golden" / "save_slots.json"


def copy_saves(dest: Path, names: list[str]) -> None:
    saves = dest / "saves"
    saves.mkdir(parents=True)
    for name in names:
        shutil.copy(FIXTURE / "saves" / name, saves / name)


def listing(base: Path):
    try:
        return list_save_slots(base), None
    except Exception as exc:  # noqa: BLE001 — the fixture's crashes are the point
        return None, exc


def main() -> int:
    parser_check = "--check" in sys.argv
    if not parser_check:
        print("pass --check; this script does not rewrite the golden", file=sys.stderr)
        return 2
    golden = json.loads(GOLDEN.read_text(encoding="utf-8"))
    names = sorted(path.name for path in (FIXTURE / "saves").glob("*.json"))
    if len(names) != 25:
        print(f"fixture has {len(names)} json files, expected 25", file=sys.stderr)
        return 1
    diffs = golden["differences"]
    if len(diffs) != 8:
        print(f"golden records {len(diffs)} differences, expected 8", file=sys.stderr)
        return 1
    crashes = [row for row in diffs if row["python"] == "crash"]
    if len(crashes) != 3:
        print(f"golden records {len(crashes)} crashes, expected 3", file=sys.stderr)
        return 1
    crash_names = {row["file"] for row in crashes}
    if not crash_names <= set(names):
        print(f"crash files missing from the fixture: {crash_names - set(names)}", file=sys.stderr)
        return 1

    kept = [name for name in names if name not in crash_names]
    tmp = Path(tempfile.mkdtemp(prefix="portlight-slots-"))
    try:
        copy_saves(tmp, kept)
        rows, exc = listing(tmp)
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    if exc is not None:
        print(f"Python raised with the crash files removed: {exc}", file=sys.stderr)
        return 1
    if rows != golden["python"]:
        print("Python listing drifted from parity/golden/save_slots.json", file=sys.stderr)
        return 1

    for row in diffs:
        alone = Path(tempfile.mkdtemp(prefix="portlight-slot-"))
        try:
            if row["python"] == "crash":
                # A good file beside the poison file must not be returned.
                copy_saves(alone, ["alpha.json", row["file"]])
                rows, exc = listing(alone)
                if exc is None:
                    print(f"{row['file']} no longer crashes the listing: {rows}", file=sys.stderr)
                    return 1
                if type(exc).__name__ != row["exception"]:
                    print(
                        f"{row['file']} raised {type(exc).__name__}, expected {row['exception']}",
                        file=sys.stderr,
                    )
                    return 1
            else:
                copy_saves(alone, [row["file"]])
                rows, exc = listing(alone)
                if exc is not None or rows != [row["python"]]:
                    print(f"{row['file']} Python row drifted: {rows} {exc}", file=sys.stderr)
                    return 1
        finally:
            shutil.rmtree(alone, ignore_errors=True)

    full, exc = listing(FIXTURE)
    if exc is None:
        print(f"full fixture listed {len(full)} rows; Python should have raised", file=sys.stderr)
        return 1
    # captain_bad_day.json is the first poison file in sorted order.
    if type(exc).__name__ != "ValueError":
        print(f"full fixture raised {type(exc).__name__}, expected ValueError", file=sys.stderr)
        return 1
    print(
        f"{GOLDEN} matches Python "
        f"({len(names)} files, {len(diffs)} differences, {len(crashes)} crash the listing)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
