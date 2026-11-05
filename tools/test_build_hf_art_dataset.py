#!/usr/bin/env python3
"""Tests for tools/build_hf_art_dataset.py."""
from __future__ import annotations

import hashlib
import importlib.util
import json
import os
import subprocess
import sys
from pathlib import Path

try:
    import pytest
except ImportError:
    _pytest_target = "/tmp/portlight-pytest"
    subprocess.check_call(
        [sys.executable, "-m", "pip", "install", "--target", _pytest_target, "pytest==9.1.1"]
    )
    sys.path.insert(0, _pytest_target)
    import pytest

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
SPEC = importlib.util.spec_from_file_location(
    "build_hf_art_dataset", os.path.join(HERE, "build_hf_art_dataset.py")
)
mod = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(mod)


def test_build_rows_covers_all_78_unique_and_sha_matches():
    rows, copy_plan = mod.build_rows(mod.DEFAULT_MANIFEST)
    assert len(rows) == 78
    assert len(copy_plan) == 78

    file_names = [r["file_name"] for r in rows]
    assert len(set(file_names)) == 78

    ids = [r["id"] for r in rows]
    assert len(set(ids)) == 78

    required_keys = {
        "file_name",
        "id",
        "group",
        "class",
        "frame",
        "view",
        "canvas",
        "anchor",
        "sha256",
        "text",
        "provenance",
    }
    manifest = json.loads(Path(mod.DEFAULT_MANIFEST).read_text(encoding="utf-8"))
    manifest_by_id = {e["id"]: e for e in manifest["entries"]}

    for row, (src_png, rel_dest, digest) in zip(rows, copy_plan):
        assert set(row.keys()) == required_keys
        assert row["file_name"] == rel_dest
        assert row["file_name"] == f"{row['group']}/{row['id']}.png"
        assert os.path.isfile(src_png)
        actual_sha = hashlib.sha256(Path(src_png).read_bytes()).hexdigest()
        assert actual_sha == digest
        assert row["sha256"] == actual_sha
        assert row["sha256"] == manifest_by_id[row["id"]]["sha256"]
        assert isinstance(row["text"], str) and row["text"]
        assert row["provenance"] == manifest_by_id[row["id"]]["provenance"]


def test_committed_metadata_jsonl_matches_generator(tmp_path):
    out_file = tmp_path / "metadata.jsonl"
    rc = mod.main(["--out", str(out_file)])
    assert rc == 0
    committed = Path(mod.DEFAULT_OUT).read_text(encoding="utf-8")
    generated = out_file.read_text(encoding="utf-8")
    assert committed == generated
    lines = [json.loads(line) for line in committed.splitlines() if line.strip()]
    assert len(lines) == 78


def test_stage_refuses_non_empty_dir(tmp_path):
    out_file = tmp_path / "metadata.jsonl"
    stage_dir = tmp_path / "non_empty_stage"
    stage_dir.mkdir()
    (stage_dir / "sentinel.txt").write_text("occupied\n", encoding="utf-8")

    rc = mod.main(["--out", str(out_file), "--stage", str(stage_dir)])
    assert rc == 1
    assert not (stage_dir / "metadata.jsonl").exists()


def test_stage_copies_and_verifies_all_78_files(tmp_path):
    out_file = tmp_path / "metadata.jsonl"
    stage_dir = tmp_path / "staged_bundle"

    rc = mod.main(["--out", str(out_file), "--stage", str(stage_dir)])
    assert rc == 0

    staged_meta = stage_dir / "metadata.jsonl"
    staged_card = stage_dir / "README.md"
    assert staged_meta.is_file()
    assert staged_card.is_file()
    assert staged_card.read_text(encoding="utf-8") == Path(mod.DEFAULT_CARD).read_text(encoding="utf-8")

    rows = [json.loads(line) for line in staged_meta.read_text(encoding="utf-8").splitlines() if line.strip()]
    assert len(rows) == 78
    assert len({r["file_name"] for r in rows}) == 78

    for row in rows:
        staged_png = stage_dir / row["file_name"]
        assert staged_png.is_file(), row["file_name"]
        digest = hashlib.sha256(staged_png.read_bytes()).hexdigest()
        assert digest == row["sha256"], row["id"]


if __name__ == "__main__":
    raise SystemExit(pytest.main([__file__, "-q", "--tb=short"]))
