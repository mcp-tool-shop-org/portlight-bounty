#!/usr/bin/env python3
"""Generate the Rust-tuned pins in parity/expected_divergences.json.

One-off tool, not run in CI. It sweeps every parity script through the same
checks as `check.py` (live oracle, wide oracle, GameSession, save round trip),
using only the untagged entries as the allowlist. Whatever still differs is
the effect of the tuned overlay. Each such difference becomes one exact leaf
pin that names both values, tagged `"category": "rust-tuned"`.

Nothing is chosen by hand: the closed list of scripts, the entries and the
paths all come from the sweep. The tool stops, writing nothing, when a
difference cannot be pinned within the rules (a list or object path, or an
untagged pin whose value moved).

With `--label-goldens` it also adds the `_source` and `_note` lines to the
unlabelled goldens whose narrow snapshot now diverges. The snapshot bytes of a
golden are never touched. It prints one line per (script, check): the path count.

Usage (after `cargo build -p portlight-cli`, with the Python game on PYTHONPATH):

    PYTHONPATH=vendor/portlight/src python3 tools/parity/pin_tuned.py
    PYTHONPATH=vendor/portlight/src python3 tools/parity/pin_tuned.py --label-goldens
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
import tempfile
from concurrent.futures import ThreadPoolExecutor

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import check  # noqa: E402

CHECK_ORDER = ("oracle", "oracle_wide", "gamesession", "roundtrip")
PYTHON_SOURCE = "the Python game at the pinned commit (captain templates unchanged)"
RUST_SOURCE = "crates/portlight-sim/data/tuned_overlay.json applied in content()"
NOTE_MARK = " Rust-tuned scripts."
GOLDEN_NOTE = (
    "Python oracle snapshot. Rust differs on the pinned paths in "
    "expected_divergences.json (Rust-tuned)."
)
PARITY_NOTE = (
    " Rust-tuned scripts. Since the balance fork, the Rust game applies "
    "crates/portlight-sim/data/tuned_overlay.json to captain pricing and burn for "
    "bounty hunter, dockhand, privateer, scholar and smuggler, and appends "
    "privateer, merchant prince and dockhand to the captain_bias of 13 contract "
    "templates. The Python game does not. The entries with category rust-tuned "
    "differ from the Python oracle only where those values reach the snapshot, "
    "and each such path is pinned with its Python and Rust value. Check "
    "oracle_wide pins a difference under a wide-only key such as the contract "
    "board. content.json is still the unmodified extract. The oracle job also "
    "runs a build with the python-base feature, which drops the overlay and the "
    "rust-tuned entries, so every script matches as before the fork. Every other "
    "script, and every untagged entry, is unchanged."
)


def natural(path: str):
    return [int(tok) if tok.isdigit() else tok for tok in re.split(r"(\d+)", path)]


def sweep_one(name: str):
    """(name, {check: (errors, left, right)}) for one script, tuned allowlist off."""
    script_path = os.path.join(check.SCRIPTS, name)
    out = {}
    with tempfile.TemporaryDirectory(prefix=f"pl-pin-{name.replace('.txt', '')}-") as save_root:
        rust = check.run_rust(script_path, os.path.join(save_root, "rust"))
        oracle = check.run_oracle(script_path, os.path.join(save_root, "oracle"))
        narrow_errors, wide_errors = check.split_wide(check.close(oracle, rust, "$"))
        out["oracle"] = (narrow_errors, oracle, rust)
        out["oracle_wide"] = (wide_errors, oracle, rust)
        session = check.run_session(script_path, os.path.join(save_root, "session"))
        if session is not None:
            session_view = check.without_log(session)
            rust_view = check.without_log(rust)
            out["gamesession"] = (check.close(session_view, rust_view, "$"), session_view, rust_view)
        if check.has_session(rust):
            errors, python_view, rust_view = check.roundtrip_errors(name, script_path)
            out["roundtrip"] = (errors, python_view, rust_view)
    return name, out


def pin_value(value):
    if value is check._MISSING:
        return {"$missing": True}
    return value


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--label-goldens", action="store_true")
    parser.add_argument("--jobs", type=int, default=4)
    parser.add_argument("--no-write", action="store_true")
    args = parser.parse_args()
    if not os.path.exists(check.rust_bin()):
        raise SystemExit(f"missing {check.rust_bin()}; run cargo build -p portlight-cli")
    try:
        import portlight  # noqa: F401
    except ImportError as exc:
        raise SystemExit(f"Python game not importable: {exc}") from exc

    with open(check.DIVERGENCES, encoding="utf-8") as fh:
        data = json.load(fh)
    untagged = [entry for entry in data["entries"] if not check.is_tuned(entry)]
    names = check.scripts()
    with ThreadPoolExecutor(max_workers=args.jobs) as pool:
        results = dict(pool.map(sweep_one, names))

    problems: list[str] = []
    tagged: list[dict] = []
    sweep_rows: list[tuple[str, str, int]] = []
    for name in names:
        for kind in CHECK_ORDER:
            if kind not in results[name]:
                continue
            errors, left, right = results[name][kind]
            known = check.listed(untagged, name, kind)
            patterns = [spec["path"] for entry in known for spec in entry["paths"]]
            allowed, unexpected, missing = check.partition(errors, patterns)
            if known and missing:
                problems.append(f"{name} [{kind}] an untagged pin stopped differing: {missing}")
            for entry in known:
                for spec in entry["paths"]:
                    py = check.lookup(left, spec["path"])
                    rs = check.lookup(right, spec["path"])
                    if not check.values_equal(py, spec["python"]) or not check.values_equal(
                        rs, spec["rust"]
                    ):
                        problems.append(f"{name} [{kind}] an untagged pin moved: {spec['path']}")
            if not unexpected:
                continue
            leaves: dict[str, None] = {}
            for line in unexpected:
                leaves[check.error_path(line)] = None
            specs = []
            for path in sorted(leaves, key=natural):
                if check.pattern_error(path):
                    problems.append(f"{name} [{kind}] wildcard path {path}")
                    continue
                py = check.lookup(left, path)
                rs = check.lookup(right, path)
                if any(isinstance(v, (dict, list)) for v in (py, rs)):
                    problems.append(f"{name} [{kind}] difference at a list or object: {path}")
                    continue
                if py is check._MISSING and rs is check._MISSING:
                    problems.append(f"{name} [{kind}] difference with no value on either side: {path}")
                    continue
                specs.append({"path": path, "python": pin_value(py), "rust": pin_value(rs)})
            if not specs:
                continue
            tagged.append(
                {
                    "script": name,
                    "check": kind,
                    "category": check.TUNED_CATEGORY,
                    "summary": (
                        f"Rust-tuned: the overlay's captain pricing, burn and contract bias reach "
                        f"{len(specs)} {kind} paths here. Rust has the tuned value, Python the "
                        f"extract's."
                    ),
                    "rust": RUST_SOURCE,
                    "python": PYTHON_SOURCE,
                    "paths": specs,
                }
            )
            sweep_rows.append((name, kind, len(specs)))
    if problems:
        print("cannot pin within the rules; nothing written:")
        for line in problems:
            print("  ", line)
        return 1

    print("script\tcheck\tpaths")
    for name, kind, count in sweep_rows:
        print(f"{name}\t{kind}\t{count}")
    scripts = sorted({row[0] for row in sweep_rows})
    print(f"closed list: {len(scripts)} scripts, {len(tagged)} tagged entries, "
          f"{sum(row[2] for row in sweep_rows)} paths")

    if args.no_write:
        return 0
    note = data["note"]
    if NOTE_MARK in note:
        note = note[: note.index(NOTE_MARK)]
    data["note"] = note + PARITY_NOTE
    data["entries"] = untagged + tagged
    with open(check.DIVERGENCES, "w", encoding="utf-8") as fh:
        fh.write(json.dumps(data, indent=2, ensure_ascii=False) + "\n")

    if args.label_goldens:
        wanted = {row[0] for row in sweep_rows if row[1] == "oracle"}
        for name in sorted(wanted):
            path = os.path.join(check.GOLDEN, name.replace(".txt", ".json"))
            with open(path, encoding="utf-8") as fh:
                text = fh.read()
            if json.loads(text).get("_source") == "python-oracle":
                continue
            if not text.startswith("{\n"):
                print(f"cannot label {path}: unexpected layout")
                return 1
            label = (
                f'  "_source": "python-oracle",\n  "_note": {json.dumps(GOLDEN_NOTE)},\n'
            )
            with open(path, "w", encoding="utf-8") as fh:
                fh.write("{\n" + label + text[2:])
            print(f"labelled {os.path.basename(path)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
