#!/usr/bin/env python3
"""build_hf_art_dataset.py - derive Hugging Face dataset metadata and stage bundle for portlight-bounty landing art.

    python tools/build_hf_art_dataset.py [--manifest PATH] [--out PATH] [--card PATH] [--stage DIR]

Reads godot/assets/landing/MANIFEST.json (manifest 0.3.0, 78 entries) with
the Python standard library only and writes hf/portlight-bounty-landing-art/metadata.jsonl.

File-name scheme (`file_name`):
  `<group>/<basename>`, where `<basename>` is `os.path.basename(entry["path"])` when that
  basename is `f"{id}.png"` (the 46 files in `ships`, `chart`, `quay_flag`, and the three
  `harbour` water tiles `water_a..c`), and `f"{id}.png"` when `entry["path"]` sits in a
  per-asset subdirectory as `<dir>/<id>/beauty.png` (the 27 `harbour` quay/pier/pilings
  plates and the 5 `harbour_props` plates), so every row resolves to `<group>/<id>.png`
  and all 78 `file_name` values are unique.

Caption template (`text`, derived only from manifest fields):
  - When `class` and `frame` are present (`ships`, with `rot_z` non-null on `f0..f7`):
      `"{class} ({group}, {view} view, frame {frame}, heading rot_z {rot_z} deg)"`
  - When `class` and `frame` are present and `rot_z` is null (`ships` `wake` frames):
      `"{class} ({group}, {view} view, frame {frame})"`
  - When `class` and `frame` are null (`harbour`, `chart`, `quay_flag`, `harbour_props`):
      `"{id} ({group}/{subgroup}, {view} view)"`
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DEFAULT_MANIFEST = ROOT / "godot" / "assets" / "landing" / "MANIFEST.json"
DEFAULT_OUT = ROOT / "hf" / "portlight-bounty-landing-art" / "metadata.jsonl"
DEFAULT_CARD = ROOT / "hf" / "portlight-bounty-landing-art" / "README.md"


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as fh:
        for chunk in iter(lambda: fh.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


def derive_file_name(entry: dict) -> str:
    group = entry["group"]
    aid = entry["id"]
    rel_path = entry["path"]
    base = os.path.basename(rel_path)
    if base == "beauty.png":
        base = f"{aid}.png"
    return f"{group}/{base}"


def derive_text(entry: dict) -> str:
    group = entry["group"]
    view = entry["view"]
    cls = entry.get("class")
    frame = entry.get("frame")
    rot_z = entry.get("rot_z")
    if cls is not None and frame is not None:
        if rot_z is not None:
            return f"{cls} ({group}, {view} view, frame {frame}, heading rot_z {rot_z} deg)"
        return f"{cls} ({group}, {view} view, frame {frame})"
    aid = entry["id"]
    subgroup = entry.get("subgroup")
    group_label = f"{group}/{subgroup}" if subgroup else group
    return f"{aid} ({group_label}, {view} view)"


def build_rows(manifest_path: Path) -> tuple[list[dict], list[tuple[Path, str, str]]]:
    manifest_path = Path(manifest_path)
    bundle_dir = manifest_path.parent
    raw = json.loads(manifest_path.read_text(encoding="utf-8"))
    entries = raw.get("entries", [])
    if not entries:
        raise ValueError(f"no entries found in {manifest_path}")

    rows: list[dict] = []
    copy_plan: list[tuple[Path, str, str]] = []
    seen_names: set[str] = set()

    for entry in entries:
        aid = entry["id"]
        src_png = bundle_dir / entry["path"]
        if not src_png.is_file():
            raise FileNotFoundError(f"{aid}: missing PNG at {src_png}")

        digest = sha256_file(src_png)
        expected_sha = entry.get("sha256")
        if expected_sha and digest != expected_sha:
            raise ValueError(
                f"{aid}: sha256 mismatch for {src_png}: computed {digest}, manifest has {expected_sha}"
            )

        file_name = derive_file_name(entry)
        if file_name in seen_names:
            raise ValueError(f"{aid}: duplicate file_name {file_name!r}")
        seen_names.add(file_name)

        row = {
            "file_name": file_name,
            "id": aid,
            "group": entry["group"],
            "class": entry.get("class"),
            "frame": entry.get("frame"),
            "view": entry["view"],
            "canvas": entry["canvas"],
            "anchor": entry["anchor"],
            "sha256": digest,
            "text": derive_text(entry),
            "provenance": entry["provenance"],
        }
        rows.append(row)
        copy_plan.append((src_png, file_name, digest))

    return rows, copy_plan


def write_jsonl(rows: list[dict], out_path: Path) -> None:
    out_path = Path(out_path)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    payload = "\n".join(json.dumps(r, ensure_ascii=False) for r in rows) + "\n"
    out_path.write_text(payload, encoding="utf-8", newline="\n")


def stage_bundle(
    stage_dir: Path,
    rows: list[dict],
    copy_plan: list[tuple[Path, str, str]],
    card_path: Path,
) -> int:
    stage_dir = Path(stage_dir)
    if stage_dir.exists():
        if not stage_dir.is_dir():
            raise ValueError(f"--stage path exists and is not a directory: {stage_dir}")
        if any(stage_dir.iterdir()):
            raise ValueError(f"--stage refuses non-empty directory: {stage_dir}")
    else:
        stage_dir.mkdir(parents=True, exist_ok=True)

    card_path = Path(card_path)
    if not card_path.is_file():
        raise FileNotFoundError(f"dataset card not found at {card_path}")

    verified = 0
    for src_png, rel_dest, expected_sha in copy_plan:
        dest_png = stage_dir / rel_dest
        dest_png.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(src_png, dest_png)
        copied_sha = sha256_file(dest_png)
        if copied_sha != expected_sha:
            raise ValueError(
                f"staged sha256 mismatch for {dest_png}: got {copied_sha}, expected {expected_sha}"
            )
        verified += 1

    write_jsonl(rows, stage_dir / "metadata.jsonl")
    shutil.copy2(card_path, stage_dir / "README.md")
    return verified


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--manifest",
        type=Path,
        default=DEFAULT_MANIFEST,
        help="Path to godot/assets/landing/MANIFEST.json",
    )
    ap.add_argument(
        "--out",
        type=Path,
        default=DEFAULT_OUT,
        help="Output path for metadata.jsonl",
    )
    ap.add_argument(
        "--card",
        type=Path,
        default=DEFAULT_CARD,
        help="Path to hf/portlight-bounty-landing-art/README.md (copied when --stage is used)",
    )
    ap.add_argument(
        "--stage",
        type=Path,
        default=None,
        help="Stage the 78 PNGs, metadata.jsonl, and README.md into an empty directory and verify sha256",
    )
    args = ap.parse_args(argv)

    try:
        rows, copy_plan = build_rows(args.manifest)
        write_jsonl(rows, args.out)
        msg = f"wrote {len(rows)} rows to {args.out}"
        if args.stage is not None:
            verified = stage_bundle(args.stage, rows, copy_plan, args.card)
            msg += f"; staged {verified} PNGs + metadata.jsonl + README.md in {args.stage} (all {verified} sha256 matched)"
        print(msg)
        return 0
    except Exception as exc:  # noqa: BLE001
        print(f"ERROR: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
