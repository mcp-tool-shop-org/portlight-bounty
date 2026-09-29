#!/usr/bin/env python3
"""Negative cases for the build_manifest verifier gates.

The studio bundle is not required. Acceptance tests are pytest cases that
call main() on a temporary studio fixture. A passing real run still emits
MANIFEST.json only when `problems` stays empty.
"""
from __future__ import annotations

import csv
import hashlib
import importlib.util
import io
import json
import os
import re
import shutil
import struct
import subprocess
import sys
import zlib

try:
    import pytest
except ImportError:
    # A --user install is not visible to this process on GitHub's runner.
    _pytest_target = "/tmp/portlight-pytest"
    subprocess.check_call(
        [sys.executable, "-m", "pip", "install", "--target", _pytest_target, "pytest==9.1.1"]
    )
    sys.path.insert(0, _pytest_target)
    import pytest

HERE = os.path.dirname(os.path.abspath(__file__))
SPEC = importlib.util.spec_from_file_location(
    "build_manifest_pb002", os.path.join(HERE, "build_manifest_pb002.py")
)
mod = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(mod)

QUAY = list(mod.EXP_QUAY_FLAG)
failures = []


def check(name, ok, detail):
    if ok:
        print(f"ok  {name}: {detail}")
    else:
        print(f"NOT CLOSED  {name}: {detail}")
        failures.append(name)


def joined(problems):
    return "; ".join(problems) if problems else "(no problem)"


def quay_object(inventory_ids=None, rows=None):
    ids = QUAY if inventory_ids is None else inventory_ids
    body_rows = rows if rows is not None else [
        {"id": aid, "pass": True, "exit": 0} for aid in QUAY
    ]
    return {
        "inventory": [{"id": aid, "sha256": "abc"} for aid in ids],
        "rows": body_rows,
    }


def quay_row_gate(qrows, problems):
    for aid in QUAY:
        row = qrows.get(aid)
        if not row or row.get("pass") is not True or row.get("exit") != 0:
            problems.append(f"{aid}: Verifier quay_flag row missing or not pass/exit 0")


def test_gate_helpers():
    failures.clear()
    # Happy path: exact inventory, passing rows, no stub, no FAIL line.
    happy = quay_object()
    problems = []
    inv = mod.bind_inventory(happy, QUAY, "quay_flag", problems)
    rows = mod.index_rows(happy, QUAY, "quay_flag", problems)
    quay_row_gate(rows, problems)
    mod.require_no_fail_results(
        "Result: **3/3 PASS**\n✅ Builder may land\n",
        "retrieval-quay_flag_painterly.md",
        problems,
    )
    check(
        "happy quay object",
        not problems and not mod.json_has_stub_key(happy) and set(inv) == set(QUAY),
        joined(problems) if problems else "no problems",
    )

    # G2 quay: empty inventory.
    problems = []
    mod.bind_inventory({"inventory": [], "rows": happy["rows"]}, QUAY, "quay_flag", problems)
    check("quay empty inventory", any("inventory ids" in p for p in problems), joined(problems))

    # G2 quay: partial inventory.
    problems = []
    mod.bind_inventory(quay_object(QUAY[:2]), QUAY, "quay_flag", problems)
    check("quay partial inventory", any("inventory ids" in p for p in problems), joined(problems))

    # G2 quay: extra inventory id.
    problems = []
    mod.bind_inventory(quay_object(QUAY + ["quay_flag_d"]), QUAY, "quay_flag", problems)
    check("quay extra inventory id", any("inventory ids" in p for p in problems), joined(problems))

    problems = []
    dup_inv = quay_object()
    dup_inv["inventory"].append({"id": "quay_flag_a", "sha256": "other"})
    mod.bind_inventory(dup_inv, QUAY, "quay_flag", problems)
    check(
        "quay duplicate inventory id",
        any("duplicate inventory id quay_flag_a" in p for p in problems),
        joined(problems),
    )

    # G2 harbour: empty, partial, and extra against the real expected set.
    problems = []
    mod.bind_inventory({"inventory": []}, mod.EXP_HARB, "harbour", problems)
    check("harbour empty inventory", any("inventory ids" in p for p in problems), joined(problems))

    problems = []
    mod.bind_inventory(
        {"inventory": [{"id": mod.EXP_HARB[0], "sha256": "abc"}]},
        mod.EXP_HARB,
        "harbour",
        problems,
    )
    check("harbour partial inventory", any("inventory ids" in p for p in problems), joined(problems))

    problems = []
    extra = [{"id": aid, "sha256": "abc"} for aid in mod.EXP_HARB]
    extra.append({"id": "not_a_plate", "sha256": "abc"})
    mod.bind_inventory({"inventory": extra}, mod.EXP_HARB, "harbour", problems)
    check("harbour extra inventory id", any("inventory ids" in p for p in problems), joined(problems))

    problems = []
    full = [{"id": aid, "sha256": "abc"} for aid in mod.EXP_HARB]
    got = mod.bind_inventory({"inventory": full}, mod.EXP_HARB, "harbour", problems)
    check(
        "harbour exact inventory",
        not problems and set(got) == set(mod.EXP_HARB),
        joined(problems) if problems else f"{len(got)} ids",
    )

    # G1: PASS summary plus a FAIL result line.
    doc = (
        "Result: **3/3 PASS**\n"
        "Verdict: **FAIL**\n"
        "Result: **2/3 PASS**\n"
        "✅ Builder may land\n"
    )
    problems = []
    mod.require_no_fail_results(doc, "retrieval-quay_flag_painterly.md", problems)
    check("PASS doc with a FAIL result line", bool(problems), joined(problems))

    summary = "**Result: 36 PASS, 0 FAIL**\n| **PASS** |\n| notes | 0 FAIL |\n**ANDON v0.1.1: 30 PASS, 0 FAIL.**\n"
    check(
        "zero-FAIL summary is not a FAIL result",
        mod.fail_result_lines(summary) == [],
        "no FAIL result line",
    )

    # Real verdicts: a plain FAIL token, a Verdict:/Result: line, or a FAIL cell
    # in any column of a table row. A header is not required.
    verdicts = (
        "FAIL",
        "**FAIL**",
        "Verdict: FAIL",
        "Result: **FAIL**",
        "| id | Verdict |\n| --- | --- |\n| quay_flag_a | FAIL |",
        "| id | Result |\n| --- | --- |\n| quay_flag_a | **FAIL** |",
    )
    for sample in verdicts:
        problems = []
        mod.require_no_fail_results(sample + "\n", "retrieval.md", problems)
        check(f"negative: FAIL verdict {sample.splitlines()[0]!r}", bool(problems), joined(problems))
    prose_fail = (
        "Neither is a FAIL of the delivered images.\n"
        "Addendum V.1 keeps that FAIL…\n"
    )
    check(
        "prose FAIL mentions are not verdicts",
        mod.fail_result_lines(prose_fail) == [],
        "no FAIL verdict",
    )
    check(
        "headerless FAIL (exit 1) is a verdict",
        mod.fail_result_lines("| water_a..c | FAIL (exit 1) |\n") == [
            "| water_a..c | FAIL (exit 1) |",
        ],
        "no Verdict header",
    )
    check(
        "marked headerless exit 1 on a documented id is exempt",
        mod.fail_result_lines(
            "Documented stock v0.1.1 canvas-size failure.\n"
            "| chart_water_a | FAIL (exit 1) |\n"
            "| port_marker | FAIL (exit 1) |\n"
        ) == [],
        "documented ids",
    )
    check(
        "exit 2 on chart_water_a is not exempt",
        mod.fail_result_lines(
            "Documented stock v0.1.1 canvas-size failure.\n"
            "| chart_water_a | FAIL (exit 2) |\n"
        ) == ["| chart_water_a | FAIL (exit 2) |"],
        "exit 2",
    )
    check(
        "undocumented id with exit 1 is not exempt",
        mod.fail_result_lines(
            "Documented stock v0.1.1 canvas-size failure.\n"
            "| chart_new | FAIL (exit 1) |\n"
        ) == ["| chart_new | FAIL (exit 1) |"],
        "chart_new",
    )
    check(
        "newline does not join a stock mark",
        mod._marks_stock_v011_canvas("stock\nv0.1.1 canvas-size") is False
        and mod._marks_stock_v011_canvas("Documented stock v0.1.1 canvas-size failure.") is True,
        "one line",
    )
    unmarked_exit = (
        "| id | Result |\n| --- | --- |\n"
        "| water_a..c | FAIL (exit 1) |\n"
        "| chart_port_marker | FAIL (exit 1) |\n"
    )
    check(
        "unmarked FAIL (exit 1) is a verdict",
        mod.fail_result_lines(unmarked_exit) == [
            "| water_a..c | FAIL (exit 1) |",
            "| chart_port_marker | FAIL (exit 1) |",
        ],
        "port_marker must not pass by accident",
    )
    check(
        "doc-marked stock canvas FAIL (exit 1) is exempt",
        mod.fail_result_lines(
            "Documented stock v0.1.1 canvas-size failure.\n" + unmarked_exit
        ) == [],
        "stock v0.1.1",
    )

    # Duplicate row id, fail then pass. Last row would pass; the duplicate must not.
    dup_rows = [
        {"id": "quay_flag_a", "pass": False, "exit": 1},
        {"id": "quay_flag_a", "pass": True, "exit": 0},
        {"id": "quay_flag_b", "pass": True, "exit": 0},
        {"id": "quay_flag_c", "pass": True, "exit": 0},
    ]
    problems = []
    mapped = mod.index_rows({"rows": dup_rows}, QUAY, "quay_flag", problems)
    check(
        "duplicate row id fail-then-pass",
        any("duplicate row id quay_flag_a" in p for p in problems) and mapped["quay_flag_a"]["pass"] is True,
        joined(problems),
    )

    # Unexpected extra row.
    problems = []
    mod.index_rows(
        {"rows": happy["rows"] + [{"id": "quay_flag_d", "pass": False, "exit": 1}]},
        QUAY,
        "quay_flag",
        problems,
    )
    check("unexpected extra row", any("unexpected row id quay_flag_d" in p for p in problems), joined(problems))

    # B1: a failing row, and a non-zero exit, still fail.
    for name, row in (
        ("row pass false", {"id": "quay_flag_b", "pass": False, "exit": 0}),
        ("row exit 1", {"id": "quay_flag_b", "pass": True, "exit": 1}),
    ):
        rows = [{"id": aid, "pass": True, "exit": 0} for aid in QUAY]
        rows[1] = row
        problems = []
        indexed = mod.index_rows({"rows": rows}, QUAY, "quay_flag", problems)
        quay_row_gate(indexed, problems)
        check(f"B1 {name}", any("quay_flag_b" in p and "not pass/exit 0" in p for p in problems), joined(problems))

    # B2: _STUB at any case, including nested. A value that merely says _STUB is not a key.
    for key in ("_STUB", "_stub", "_Stub"):
        check(f"B2 key {key}", mod.json_has_stub_key({key: True}), "matched")
        check(
            f"B2 nested {key}",
            mod.json_has_stub_key({"rows": [{"id": "quay_flag_a", "meta": {key: 1}}]}),
            "matched",
        )
    problems = []
    mod.reject_stub_keys({"stub": True}, "results.json", problems)
    check("stub key is rejected", bool(problems), joined(problems))
    check("_STUB value is not a key", not mod.json_has_stub_key({"note": "_STUB"}), "ignored")
    check("stub value is not a key", not mod.json_has_stub_key({"note": "stub"}), "ignored")
    check(
        "negative: key containing _STUB",
        mod.json_has_stub_key({"harbour_STUB_marker": True}),
        "matched",
    )
    check(
        "negative: nested harbour stub key",
        mod.json_has_stub_key({"rows": [{"id": "water_a", "meta": {"pre_stub_note": 1}}]}),
        "matched",
    )
    problems = []
    mod.reject_stub_keys(
        {"inventory": [], "harbour_STUB_marker": True},
        "harbour_andon_results.json",
        problems,
    )
    check("negative: harbour stub key is a gate problem", bool(problems), joined(problems))

    raw = b'{"_STUB": true, "inventory": []}\n'
    digest = mod.sha256_bytes(raw)
    doc_real = "Result: **3/3 PASS**\nplate notes\n"
    check("B2 doc missing results sha", digest not in doc_real, digest[:12])
    check("B2 doc containing results sha", digest in (doc_real + digest), "bound")

    # One buffer is parsed and hashed. Re-serializing the object is a different byte string.
    parsed_problems = []
    parsed = mod.parse_json_object(raw, "quay_flag_painterly_results.json", parsed_problems)
    check(
        "results bytes parsed and hashed once",
        parsed == {"_STUB": True, "inventory": []}
        and digest == hashlib.sha256(raw).hexdigest()
        and not parsed_problems,
        digest[:12],
    )

    # A non-object or broken results file is a problem, not an exception.
    for raw_bad, needle in (
        (b"[]", "not a JSON object"),
        (b"1", "not a JSON object"),
        (b'"x"', "not a JSON object"),
        (b"null", "not a JSON object"),
        (b"{", "not valid JSON"),
    ):
        problems = []
        try:
            got = mod.parse_json_object(raw_bad, "results.json", problems)
        except Exception as exc:  # noqa: BLE001 - the gate must not traceback
            got = exc
        check(
            f"clean problem for {raw_bad!r}",
            got is None and any(needle in p for p in problems),
            joined(problems),
        )

    # Real ships rows: pass true, v011_exit 1, no exit key.
    stock = ship_stock_v011_row("ship_sloop_f0")
    check("stock v011_exit row passes", mod.ship_row_failure(stock) is None, repr(stock))
    check(
        "missing exit passes when pass is true",
        mod.ship_row_failure({"id": "ship_sloop_f0", "pass": True}) is None,
        "no exit key",
    )
    check(
        "exit must be 0 when present",
        mod.ship_row_failure({"id": "ship_sloop_f0", "pass": True, "v011_exit": 1, "exit": 1}) == "nonzero exit",
        "nonzero exit",
    )
    check(
        "pass false refuses the row",
        mod.ship_row_failure({**stock, "pass": False}) == "not pass",
        "not pass",
    )
    check(
        "verdict FAIL refuses the row",
        mod.ship_row_failure({**stock, "verdict": "FAIL"}) == "verdict FAIL",
        "verdict FAIL",
    )
    check(
        "false gate refuses the row",
        mod.ship_row_failure({**stock, "gate": False}) == "false gate",
        "false gate",
    )
    check(
        "nested false gate refuses the row",
        mod.ship_row_failure({**stock, "gate": {"closed": False}}) == "false gate",
        "nested gate",
    )
    check(
        "false gates dict refuses the row",
        mod.ship_row_failure({**stock, "gates": {"G1": False}}) == "false gate",
        "gates G1",
    )
    for label, gate in (
        ("string false", "false"),
        ("string FALSE", "FALSE"),
        ("zero", 0),
        ("null", None),
    ):
        check(
            f"gate value {label} refuses the row",
            mod.ship_row_failure({**stock, "gates": {"G1": gate}}) == "false gate",
            label,
        )
    check(
        "verdict fail any case refuses the row",
        mod.ship_row_failure({**stock, "verdict": "fail"}) == "verdict FAIL",
        "fail",
    )
    mixed_pass = {
        **stock,
        "fails": [],
        "Fails": [],
        "gates": {"G1": True},
        "Gates": {"G1": True},
        "verdict": "PASS",
        "Verdict": "pass",
    }
    check("passing case variants still pass", mod.ship_row_failure(mixed_pass) is None, "variants")
    check(
        "Fails list refuses even when fails is empty",
        mod.ship_row_failure({**stock, "fails": [], "Fails": ["canvas"]}) == "non-empty fails",
        "Fails",
    )
    check(
        "Gates false refuses even when gates are true",
        mod.ship_row_failure({**stock, "gates": {"G1": True}, "Gates": {"G1": False}}) == "false gate",
        "Gates",
    )
    check(
        "Verdict fail refuses even when verdict is PASS",
        mod.ship_row_failure({**stock, "verdict": "PASS", "Verdict": "fail"}) == "verdict FAIL",
        "Verdict",
    )
    real_gates = {
        **stock,
        "gates": {"G1": True, "G2": True},
        "info": {"G5_would_pass": False},
    }
    check(
        "info.G5_would_pass false does not refuse the row",
        mod.ship_row_failure(real_gates) is None,
        repr(real_gates),
    )
    check(
        "non-empty fails list refuses the row",
        mod.ship_row_failure({**stock, "fails": ["canvas"]}) == "non-empty fails",
        "fails",
    )
    check(
        "non-empty fails string refuses the row",
        mod.ship_row_failure({**stock, "fails": "canvas"}) == "non-empty fails",
        "fails string",
    )
    check(
        "non-empty fails dict refuses the row",
        mod.ship_row_failure({**stock, "fails": {"reason": "canvas"}}) == "non-empty fails",
        "fails dict",
    )
    for empty in (None, [], "", {}):
        check(
            f"empty fails {empty!r} passes",
            mod.ship_row_failure({**stock, "fails": empty}) is None,
            repr(empty),
        )
    benign = {**stock, "verdict": "PASS", "gate": True, "fails": [], "stubborn_note": "keep"}
    check("benign ships fields still pass", mod.ship_row_failure(benign) is None, repr(benign))
    check("stubborn_note is not a stub key", not mod.json_has_stub_key({"stubborn_note": "keep"}), "token")
    check("stub token still matches", mod.json_has_stub_key({"pre_stub_note": 1}), "token")
    for n in (0, 2, 12):
        cell = (
            "| id | Result |\n"
            "| --- | --- |\n"
            f"| plate | FAIL (exit {n}) |\n"
        )
        check(
            f"FAIL (exit {n}) is a verdict",
            mod.fail_result_lines(cell) == [f"| plate | FAIL (exit {n}) |"],
            f"exit {n}",
        )
    spread = (
        "Documented stock\n"
        "v0.1.1\n"
        "canvas-size failure.\n"
        "| id | Result |\n"
        "| --- | --- |\n"
        "| chart_new | FAIL (exit 1) |\n"
    )
    check(
        "spread stock terms do not exempt a chart row",
        mod.fail_result_lines(spread) == ["| chart_new | FAIL (exit 1) |"],
        "terms must share a line",
    )
    bold = "| quay_flag_b | **FAIL** |\n"
    check(
        "bold FAIL row is a verdict without a header",
        mod.fail_result_lines(bold) == ["| quay_flag_b | **FAIL** |"],
        "main **FAIL** rule",
    )
    assert not failures, failures

# ---------------------------------------------------------------- main() fixture
COLUMNS = [
    "id", "status", "phase", "view", "canvas", "anchor_px", "hull_len_px",
    "mesh_source", "rot_z_f0..f7", "hull", "footprint", "layer",
    "layer_offset_px", "texture_origin", "y_sort_origin", "andon_kind",
]


def tiny_png():
    """2x2 PNG. png_size only needs a valid signature and IHDR."""
    ihdr = struct.pack(">IIBBBBB", 2, 2, 8, 2, 0, 0, 0)

    def chunk(tag, data):
        crc = zlib.crc32(tag + data) & 0xFFFFFFFF
        return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", crc)

    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", ihdr) + chunk(b"IEND", b"")


def ensure_parent(path):
    os.makedirs(os.path.dirname(path), exist_ok=True)


def write_bytes(path, data):
    ensure_parent(path)
    with open(path, "wb") as fh:
        fh.write(data)


def write_text(path, text):
    ensure_parent(path)
    with open(path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(text)


def write_json(path, obj):
    raw = (json.dumps(obj, indent=2) + "\n").encode("utf-8")
    write_bytes(path, raw)
    return raw


def plate_row(**over):
    item = {column: "" for column in COLUMNS}
    item.update(over)
    return item


def ship_ids():
    return [f"ship_{cls}_{frame}" for cls in mod.CLASSES for frame in mod.FRAMES]


def ship_stock_v011_row(aid):
    """Real ships row: pass true, v011_exit 1, every gate true, info.G5_would_pass false."""
    return {
        "id": aid,
        "pass": True,
        "v011_exit": 1,
        "gates": {"G1": True, "G2": True, "G3": True, "G4": True, "G5": True},
        "info": {"G5_would_pass": False},
    }


def harbour_source_dir(aid):
    if aid.startswith("water_"):
        return "ground"
    if aid == "pier_pilings_1x1":
        return "props"
    return "quay"


def harbour_landed(aid):
    if aid.startswith("water_"):
        return f"ground/{aid}.png"
    if aid == "pier_pilings_1x1":
        return f"props/{aid}/beauty.png"
    return f"structures/{aid}/beauty.png"


def build_fixture(studio):
    """A complete studio small enough for main() and shaped like the real gates."""
    png = tiny_png()
    png_sha = hashlib.sha256(png).hexdigest()
    mesh = b"plate-mesh"
    prefix = hashlib.sha256(mesh).hexdigest()[:12]
    plates = os.path.join(studio, "pb-002-plates")
    outbox = os.path.join(studio, "outbox-PB-002")
    verifier = os.path.join(studio, "grok-bot-verifier", "PB-002")
    landing = os.path.join(plates, "landing")
    rows = []

    mesh_paths = {}
    for cls in mod.CLASSES:
        mesh_path = os.path.join(studio, "meshes", f"ship_{cls}.glb")
        write_bytes(mesh_path, mesh)
        mesh_paths[cls] = mesh_path
    for aid in ship_ids():
        base = aid.rsplit("_", 1)[0]
        cls = base.split("_", 1)[1]
        write_bytes(os.path.join(plates, "ships", base, f"{aid}.png"), png)
        write_bytes(os.path.join(landing, "chart", "ships", base, f"{aid}.png"), png)
        item = plate_row(
            id=aid, status="LOCKED", phase="P1", view="ships", canvas="2x2",
            anchor_px="1,1", hull_len_px="40", mesh_source=mesh_paths[cls], hull="hull",
        )
        item["rot_z_f0..f7"] = "0"
        rows.append(item)
    write_json(
        os.path.join(verifier, "ships_results.json"),
        {
            "inventory": [{"id": aid, "sha256": png_sha} for aid in ship_ids()],
            "rows": [ship_stock_v011_row(aid) for aid in ship_ids()],
        },
    )
    write_text(
        os.path.join(outbox, "retrieval-ships.md"),
        "**Result: 36 PASS, 0 FAIL**\n"
        "Neither is a FAIL of the delivered images.\n"
        + "".join(f"| {aid} | **PASS** |\n" for aid in ship_ids()),
    )

    for aid in mod.EXP_HARB:
        src = os.path.join(plates, harbour_source_dir(aid), f"{aid}.png")
        write_bytes(src, png)
        write_bytes(os.path.join(landing, harbour_landed(aid)), png)
        write_json(os.path.splitext(src)[0] + ".andon.json", {"andon_version": "0.1.1", "pass": True, "kind": "ground"})
        write_json(os.path.splitext(src)[0] + ".render.json", {"blender": "5.2", "engine": "eevee"})
        rows.append(plate_row(
            id=aid, status="LOCKED", phase="P0", view="harbour", canvas="2x2",
            anchor_px="0,0", footprint="1x1", layer="ground", layer_offset_px="0,0",
            texture_origin="-", y_sort_origin="-", andon_kind="ground",
        ))
    write_json(
        os.path.join(verifier, "harbour_andon_results.json"),
        {
            "inventory": [{"id": aid, "sha256": png_sha} for aid in mod.EXP_HARB],
            "rows": [{"id": aid, "pass": True} for aid in mod.EXP_HARB],
        },
    )
    write_text(
        os.path.join(outbox, "retrieval-harbour.md"),
        "**ANDON v0.1.1: 30 PASS, 0 FAIL.**\n"
        + "".join(f"| {aid} | **PASS** (exit 0) |\n" for aid in mod.EXP_HARB),
    )

    chart_obj = {}
    for aid in mod.EXP_CHART:
        src = os.path.join(plates, "chart", f"{aid}.png")
        write_bytes(src, png)
        write_bytes(os.path.join(landing, "chart", f"{aid}.png"), png)
        write_json(
            os.path.splitext(src)[0] + ".andon.json",
            {"andon_gate_of_record": {"pass": True, "exit_code": 0}},
        )
        write_json(os.path.splitext(src)[0] + ".render.json", {"blender": "5.2", "post": {"resize": "none"}})
        chart_obj[aid] = {"sha256": png_sha, "sim": {"pass": True}}
        rows.append(plate_row(
            id=aid, status="LOCKED", phase="P0", view="chart", canvas="2x2",
            anchor_px="1,1", footprint="1x1", layer="chart", layer_offset_px="0,0",
            texture_origin="-", y_sort_origin="0", andon_kind="chart",
        ))
    write_json(os.path.join(verifier, "chart", "chart_retrieval_results.json"), chart_obj)
    write_text(
        os.path.join(outbox, "retrieval-chart.md"),
        "**Chart profile (SIMULATED): 4 PASS, 0 FAIL.**\n"
        "Documented stock v0.1.1 canvas-size failure.\n"
        "| id | Result |\n"
        "| --- | --- |\n"
        "| water_a..c | FAIL (exit 1) |\n"
        "| chart_port_marker | FAIL (exit 1) |\n"
        "✅ Builder may land\n",
    )

    quay_ids = list(mod.EXP_QUAY_FLAG)
    for aid in quay_ids:
        src = os.path.join(plates, "ground", f"{aid}.png")
        write_bytes(src, png)
        write_bytes(os.path.join(landing, "ground", f"{aid}.png"), png)
        write_json(
            os.path.splitext(src)[0] + ".andon.json",
            {"andon_version": "0.1.1", "pass": True, "kind": "ground"},
        )
        write_json(
            os.path.splitext(src)[0] + ".render.json",
            {
                "source_square": {"sha256": "abc"},
                "base_square": {"sha256": "def"},
                "brushwork": {"seed": 1500, "graph_sha256": "ghi"},
            },
        )
        rows.append(plate_row(
            id=aid, status="LOCKED", phase="P0", view="harbour", canvas="2x2",
            anchor_px="0,0", footprint="1x1", layer="ground", layer_offset_px="0,0",
            texture_origin="-", y_sort_origin="-", andon_kind="ground",
        ))
    quay_raw = write_json(
        os.path.join(verifier, "quay_flag_painterly_results.json"),
        {
            "inventory": [{"id": aid, "sha256": png_sha} for aid in quay_ids],
            "rows": [{"id": aid, "pass": True, "exit": 0} for aid in quay_ids],
        },
    )
    write_text(
        os.path.join(outbox, "retrieval-quay_flag_painterly.md"),
        "Result: **3/3 PASS**\n"
        + hashlib.sha256(quay_raw).hexdigest()
        + "\nAddendum V.1 keeps that FAIL…\n"
        "✅ Builder may land\n",
    )

    spec_rows = "\n".join(f"| `ship_{cls}` | hull | `{prefix}` |" for cls in mod.CLASSES)
    write_text(
        os.path.join(outbox, "asset-spec.md"),
        "Seat: Game Designer · Rev 4\n\n"
        "| R11 | Mipmaps ON for `chart_water_a..c` only; Fix Alpha Border ON everywhere |\n\n"
        + spec_rows
        + "\n",
    )
    write_text(
        os.path.join(outbox, "art-gate.md"),
        "## Addendum D (05:15 ET)\n"
        "**Result: PASS. All 36 ids are will-use.**\n\n"
        "## Addendum E (05:20 ET)\n"
        "**Result: PASS. All 30 are will-use.**\n\n"
        "AMENDED 05:25 ET (Addendum E.1).** The seams are still **Pass**.\n\n"
        "## Addendum F (06:00 ET)\n"
        "**Chart set: PASS. All 4 are will-use**\n\n"
        "## Addendum U.4 placement\n"
        "## Addendum V.2 painterly v5p PASS\n",
    )
    csv_path = os.path.join(outbox, "asset-list.csv")
    ensure_parent(csv_path)
    with open(csv_path, "w", encoding="utf-8", newline="\n") as fh:
        writer = csv.DictWriter(fh, fieldnames=COLUMNS, lineterminator="\n")
        writer.writeheader()
        writer.writerows(rows)


def run_main(argv):
    buf = io.StringIO()
    old = sys.stdout
    sys.stdout = buf
    status = 0
    try:
        try:
            mod.main(argv)
        except SystemExit as exc:
            code = exc.code
            if isinstance(code, str):
                buf.write(code + "\n")
                status = 1
            elif code not in (None, 0):
                status = code
    finally:
        sys.stdout = old
    return status, buf.getvalue()


def main_argv(studio, *extra):
    return ["--studio", studio, "--version", "9.9.9", *extra]



ROOT = os.path.abspath(os.path.join(HERE, ".."))
RETRIEVAL_DOCS = (
    "retrieval-ships.md",
    "retrieval-harbour.md",
    "retrieval-chart.md",
    "retrieval-quay_flag_painterly.md",
)
RESULT_FILES = {
    "quay": os.path.join("grok-bot-verifier", "PB-002", "quay_flag_painterly_results.json"),
    "harbour": os.path.join("grok-bot-verifier", "PB-002", "harbour_andon_results.json"),
    "ships": os.path.join("grok-bot-verifier", "PB-002", "ships_results.json"),
    "chart": os.path.join("grok-bot-verifier", "PB-002", "chart", "chart_retrieval_results.json"),
}


@pytest.fixture(scope="module")
def studio(tmp_path_factory):
    path = tmp_path_factory.mktemp("acceptance") / "studio"
    build_fixture(str(path))
    return str(path)


def clone_studio(studio, dest):
    shutil.copytree(studio, dest)
    csv_path = os.path.join(dest, "outbox-PB-002", "asset-list.csv")
    text = open(csv_path, encoding="utf-8").read().replace(studio, dest)
    write_text(csv_path, text)
    return dest


def insert_middle(path, line):
    """Put a verdict line after the first line, not only at the end of the doc."""
    text = open(path, encoding="utf-8").read()
    lines = text.splitlines(keepends=True)
    extra = line if line.endswith("\n") else line + "\n"
    lines.insert(1, extra)
    write_text(path, "".join(lines))


def mutate_results(copy, which, mutate):
    rel = RESULT_FILES[which]
    path = os.path.join(copy, rel)
    old = open(path, "rb").read()
    body = json.loads(old)
    mutate(body)
    raw = write_json(path, body)
    if which == "quay":
        doc = os.path.join(copy, "outbox-PB-002", "retrieval-quay_flag_painterly.md")
        text = open(doc, encoding="utf-8").read()
        old_sha = hashlib.sha256(old).hexdigest()
        new_sha = hashlib.sha256(raw).hexdigest()
        assert old_sha in text
        write_text(doc, text.replace(old_sha, new_sha))
    return raw


@pytest.mark.parametrize("doc", RETRIEVAL_DOCS)
@pytest.mark.parametrize(
    "verdict",
    [
        "FAIL",
        "**FAIL**",
        "Verdict: FAIL",
        "Result: **FAIL**",
        "| id | Verdict |\n| --- | --- |\n| plate | FAIL |",
        "| id | Result |\n| --- | --- |\n| plate | **FAIL** |",
    ],
)
def test_fail_verdict_anywhere_in_retrieval_doc(studio, tmp_path, doc, verdict):
    copy = clone_studio(studio, str(tmp_path / "copy"))
    insert_middle(os.path.join(copy, "outbox-PB-002", doc), verdict)
    status, out = run_main(main_argv(copy))
    assert status != 0
    assert "FAIL result" in out


@pytest.mark.parametrize("doc", RETRIEVAL_DOCS)
def test_bold_fail_row_refuses_each_retrieval_doc(studio, tmp_path, doc):
    """Main refuses a quay_flag_b row whose cell is bold **FAIL** in every retrieval doc."""
    copy = clone_studio(studio, str(tmp_path / "copy"))
    insert_middle(os.path.join(copy, "outbox-PB-002", doc), "| quay_flag_b | **FAIL** |")
    status, out = run_main(main_argv(copy))
    assert status == 1
    assert "FAIL result" in out
    assert "**FAIL**" in out


@pytest.mark.parametrize("which", ["ships", "chart"])
def test_stub_key_rejected_in_ships_and_chart(studio, tmp_path, which):
    copy = clone_studio(studio, str(tmp_path / "copy"))
    mutate_results(copy, which, lambda body: body.__setitem__("stub", True))
    status, out = run_main(main_argv(copy))
    label = os.path.basename(RESULT_FILES[which])
    assert status == 1
    assert f"{label} has a stub key" in out


@pytest.mark.parametrize(
    "which,nest",
    [
        ("ships", lambda body: body["rows"][0].__setitem__("meta", {"pre_stub_note": 1})),
        ("chart", lambda body: body[next(iter(body))].__setitem__("meta", {"_STUB": True})),
    ],
)
def test_nested_stub_key_rejected_in_ships_and_chart(studio, tmp_path, which, nest):
    copy = clone_studio(studio, str(tmp_path / "copy"))
    mutate_results(copy, which, nest)
    status, out = run_main(main_argv(copy))
    label = os.path.basename(RESULT_FILES[which])
    assert status == 1
    assert f"{label} has a stub key" in out


def test_stubborn_note_is_not_a_stub_key(studio, tmp_path):
    assert mod.json_has_stub_key({"stubborn_note": "keep"}) is False
    assert mod.json_has_stub_key({"rows": [{"meta": {"stubborn_note": 1}}]}) is False
    copy = clone_studio(studio, str(tmp_path / "copy"))
    mutate_results(copy, "ships", lambda body: body.__setitem__("stubborn_note", "keep"))
    status, out = run_main(main_argv(copy))
    assert status == 0, out


@pytest.mark.parametrize("which", ["quay", "harbour"])
@pytest.mark.parametrize("key", ["_STUB", "_stub", "stub"])
def test_stub_key_rejected_in_quay_and_harbour(studio, tmp_path, which, key):
    copy = clone_studio(studio, str(tmp_path / "copy"))
    mutate_results(copy, which, lambda body: body.__setitem__(key, True))
    status, out = run_main(main_argv(copy))
    label = os.path.basename(RESULT_FILES[which])
    assert status != 0
    assert f"{label} has a stub key" in out


@pytest.mark.parametrize("which", ["quay", "harbour"])
def test_nested_stub_key_rejected_in_quay_and_harbour(studio, tmp_path, which):
    copy = clone_studio(studio, str(tmp_path / "copy"))

    def nest(body):
        body["rows"][0]["meta"] = {"stub": True}

    mutate_results(copy, which, nest)
    status, out = run_main(main_argv(copy))
    label = os.path.basename(RESULT_FILES[which])
    assert status != 0
    assert f"{label} has a stub key" in out


def test_ships_empty_inventory_fails(studio, tmp_path):
    copy = clone_studio(studio, str(tmp_path / "copy"))
    mutate_results(copy, "ships", lambda body: body.__setitem__("inventory", []))
    status, out = run_main(main_argv(copy))
    assert status != 0
    assert "ships inventory ids" in out


def test_ships_missing_row_fails(studio, tmp_path):
    copy = clone_studio(studio, str(tmp_path / "copy"))
    mutate_results(copy, "ships", lambda body: body["rows"].pop(0))
    status, out = run_main(main_argv(copy))
    assert status != 0
    assert "Verifier ships row" in out


def test_ships_failing_row_fails(studio, tmp_path):
    copy = clone_studio(studio, str(tmp_path / "copy"))

    def fail_row(body):
        body["rows"][0]["pass"] = False
        body["rows"][0]["exit"] = 0

    mutate_results(copy, "ships", fail_row)
    status, out = run_main(main_argv(copy))
    assert status != 0
    assert "not pass" in out


def test_ships_missing_exit_passes_when_pass_true(studio, tmp_path):
    copy = clone_studio(studio, str(tmp_path / "copy"))

    def drop_exit(body):
        row = body["rows"][0]
        row.pop("exit", None)
        row["pass"] = True

    mutate_results(copy, "ships", drop_exit)
    status, out = run_main(main_argv(copy))
    assert status == 0, out


def test_ships_nonzero_exit_fails(studio, tmp_path):
    copy = clone_studio(studio, str(tmp_path / "copy"))

    def bad_exit(body):
        row = body["rows"][0]
        row.pop("v011_exit", None)
        row.pop("expected", None)
        row["pass"] = True
        row["verdict"] = "PASS"
        row["exit"] = 1

    mutate_results(copy, "ships", bad_exit)
    status, out = run_main(main_argv(copy))
    assert status != 0
    assert "nonzero exit" in out


def test_ships_stock_v011_exit_passes(studio, tmp_path):
    """A real ships_results.json row has v011_exit 1 and no exit key."""
    copy = clone_studio(studio, str(tmp_path / "copy"))

    def stock(body):
        body["rows"][0] = ship_stock_v011_row(body["rows"][0]["id"])

    mutate_results(copy, "ships", stock)
    status, out = run_main(main_argv(copy))
    assert status == 0, out
    assert "PROBLEMS:" not in out


def test_ships_verdict_fail_fails(studio, tmp_path):
    copy = clone_studio(studio, str(tmp_path / "copy"))
    mutate_results(copy, "ships", lambda body: body["rows"][0].__setitem__("verdict", "FAIL"))
    status, out = run_main(main_argv(copy))
    assert status == 1
    assert "verdict FAIL" in out


def test_ships_false_gate_fails(studio, tmp_path):
    copy = clone_studio(studio, str(tmp_path / "copy"))
    mutate_results(copy, "ships", lambda body: body["rows"][0].__setitem__("gate", False))
    status, out = run_main(main_argv(copy))
    assert status == 1
    assert "false gate" in out


def test_ships_false_gates_dict_fails(studio, tmp_path):
    copy = clone_studio(studio, str(tmp_path / "copy"))
    mutate_results(copy, "ships", lambda body: body["rows"][0].__setitem__("gates", {"G1": False}))
    status, out = run_main(main_argv(copy))
    assert status == 1
    assert "false gate" in out


@pytest.mark.parametrize("gate", ["false", "FALSE", 0, None])
def test_ships_falsey_gate_values_fail(studio, tmp_path, gate):
    copy = clone_studio(studio, str(tmp_path / "copy"))
    mutate_results(copy, "ships", lambda body: body["rows"][0].__setitem__("gates", {"G1": gate}))
    status, out = run_main(main_argv(copy))
    assert status == 1
    assert "false gate" in out


def test_ships_verdict_fail_any_case(studio, tmp_path):
    copy = clone_studio(studio, str(tmp_path / "copy"))
    mutate_results(copy, "ships", lambda body: body["rows"][0].__setitem__("verdict", "Fail"))
    status, out = run_main(main_argv(copy))
    assert status == 1
    assert "verdict FAIL" in out


@pytest.mark.parametrize(
    "mutate,needle",
    [
        (lambda row: row.update({"fails": [], "Fails": ["canvas"]}), "non-empty fails"),
        (lambda row: row.update({"gates": {"G1": True}, "Gates": {"G1": "false"}}), "false gate"),
        (lambda row: row.update({"verdict": "PASS", "Verdict": "fail"}), "verdict FAIL"),
    ],
)
def test_ships_case_variant_keys_refuse_if_any_fails(studio, tmp_path, mutate, needle):
    copy = clone_studio(studio, str(tmp_path / "copy"))
    mutate_results(copy, "ships", lambda body: mutate(body["rows"][0]))
    status, out = run_main(main_argv(copy))
    assert status == 1
    assert needle in out


def test_real_ships_row_shape_passes(studio, tmp_path):
    """The 36 real rows keep pass true, v011_exit 1, all gates true, and info.G5_would_pass false."""
    copy = clone_studio(studio, str(tmp_path / "copy"))

    def real(body):
        body["rows"] = [ship_stock_v011_row(row["id"]) for row in body["rows"]]

    mutate_results(copy, "ships", real)
    status, out = run_main(main_argv(copy))
    assert status == 0, out


def test_ships_info_g5_would_pass_false_still_passes(studio, tmp_path):
    """Real rows nest G5_would_pass under info. A false there is not a gate."""
    copy = clone_studio(studio, str(tmp_path / "copy"))

    def real(body):
        row = body["rows"][0]
        row["pass"] = True
        row["v011_exit"] = 1
        row.pop("exit", None)
        row["gates"] = {"G1": True, "G2": True}
        row["info"] = {"G5_would_pass": False}

    mutate_results(copy, "ships", real)
    status, out = run_main(main_argv(copy))
    assert status == 0, out


def test_ships_nonempty_fails_list_fails(studio, tmp_path):
    copy = clone_studio(studio, str(tmp_path / "copy"))
    mutate_results(copy, "ships", lambda body: body["rows"][0].__setitem__("fails", ["canvas"]))
    status, out = run_main(main_argv(copy))
    assert status == 1
    assert "non-empty fails" in out


@pytest.mark.parametrize("fails", ["canvas", {"reason": "canvas"}])
def test_ships_nonlist_fails_fails(studio, tmp_path, fails):
    """A non-empty fails string or dict refuses the row, same as a non-empty list."""
    copy = clone_studio(studio, str(tmp_path / "copy"))
    mutate_results(copy, "ships", lambda body: body["rows"][0].__setitem__("fails", fails))
    status, out = run_main(main_argv(copy))
    assert status == 1
    assert "non-empty fails" in out


@pytest.mark.parametrize("fails", [None, [], "", {}])
def test_ships_empty_fails_passes(studio, tmp_path, fails):
    copy = clone_studio(studio, str(tmp_path / "copy"))
    mutate_results(copy, "ships", lambda body: body["rows"][0].__setitem__("fails", fails))
    status, out = run_main(main_argv(copy))
    assert status == 0, out


def test_ships_exit_n_verdict_in_retrieval_doc_fails(studio, tmp_path):
    copy = clone_studio(studio, str(tmp_path / "copy"))
    insert_middle(
        os.path.join(copy, "outbox-PB-002", "retrieval-ships.md"),
        "| id | Result |\n| --- | --- |\n| quay_flag_b | FAIL (exit 2) |",
    )
    status, out = run_main(main_argv(copy))
    assert status == 1
    assert "FAIL (exit 2)" in out


def test_chart_water_exit_2_is_not_stock_exempt(studio, tmp_path):
    copy = clone_studio(studio, str(tmp_path / "copy"))
    insert_middle(
        os.path.join(copy, "outbox-PB-002", "retrieval-chart.md"),
        "| chart_water_a | FAIL (exit 2) |",
    )
    status, out = run_main(main_argv(copy))
    assert status == 1
    assert "| chart_water_a | FAIL (exit 2) |" in out


def test_undocumented_exit_1_is_not_stock_exempt(studio, tmp_path):
    copy = clone_studio(studio, str(tmp_path / "copy"))
    insert_middle(
        os.path.join(copy, "outbox-PB-002", "retrieval-chart.md"),
        "| chart_new | FAIL (exit 1) |",
    )
    status, out = run_main(main_argv(copy))
    assert status == 1
    assert "| chart_new | FAIL (exit 1) |" in out


def test_headerless_fail_exit_row_is_recognised(studio, tmp_path):
    copy = clone_studio(studio, str(tmp_path / "copy"))
    insert_middle(
        os.path.join(copy, "outbox-PB-002", "retrieval-ships.md"),
        "| chart_new | FAIL (exit 1) |",
    )
    status, out = run_main(main_argv(copy))
    assert status == 1
    assert "FAIL (exit 1)" in out


def test_spread_stock_terms_do_not_exempt_new_chart_row(studio, tmp_path):
    """A new chart FAIL (exit 1) row fails when the stock terms are not on one line."""
    copy = clone_studio(studio, str(tmp_path / "copy"))
    write_text(
        os.path.join(copy, "outbox-PB-002", "retrieval-chart.md"),
        "**Chart profile (SIMULATED): 4 PASS, 0 FAIL.**\n"
        "Documented stock\n"
        "v0.1.1\n"
        "canvas-size failure.\n"
        "| id | Result |\n"
        "| --- | --- |\n"
        "| chart_new | FAIL (exit 1) |\n"
        "| water_a..c | FAIL (exit 1) |\n"
        "| chart_port_marker | FAIL (exit 1) |\n"
        "✅ Builder may land\n",
    )
    status, out = run_main(main_argv(copy))
    assert status == 1
    assert "retrieval-chart.md has a FAIL result line: | chart_new | FAIL (exit 1) |" in out


PROSE_FAIL_LINES = (
    "Neither is a FAIL of the delivered images.",
    "Addendum V.1 keeps that FAIL…",
)


def test_prose_fail_mentions_are_not_verdicts():
    text = "\n".join(PROSE_FAIL_LINES) + "\n"
    assert mod.fail_result_lines(text) == []
    assert mod.fail_result_lines("| chart_water_a | FAIL (exit 1) |\n") == [
        "| chart_water_a | FAIL (exit 1) |",
    ]
    assert mod.fail_result_lines(
        "Documented stock v0.1.1 canvas-size failure.\n"
        "| chart_water_a | FAIL (exit 1) |\n"
    ) == []
    assert mod.fail_result_lines(
        "Documented stock v0.1.1 canvas-size failure.\n"
        "| chart_water_a | FAIL (exit 2) |\n"
    ) == ["| chart_water_a | FAIL (exit 2) |"]
    assert mod.fail_result_lines(
        "Documented stock v0.1.1 canvas-size failure.\n"
        "| chart_new | FAIL (exit 1) |\n"
    ) == ["| chart_new | FAIL (exit 1) |"]
    unmarked = (
        "| id | Result |\n"
        "| --- | --- |\n"
        "| water_a..c | FAIL (exit 1) |\n"
        "| chart_port_marker | FAIL (exit 1) |\n"
    )
    assert mod.fail_result_lines(unmarked) == [
        "| water_a..c | FAIL (exit 1) |",
        "| chart_port_marker | FAIL (exit 1) |",
    ]
    marked = "Documented stock v0.1.1 canvas-size failure.\n" + unmarked
    assert mod.fail_result_lines(marked) == []
    bare_fail_in_marked_doc = (
        "Documented stock v0.1.1 canvas-size failure.\n"
        "| id | Result |\n"
        "| --- | --- |\n"
        "| quay_flag_a | FAIL |\n"
    )
    assert mod.fail_result_lines(bare_fail_in_marked_doc) == ["| quay_flag_a | FAIL |"]
    row_mark = (
        "| id | Result | note |\n"
        "| --- | --- | --- |\n"
        "| chart_port_marker | FAIL (exit 1) | documented stock v0.1.1 canvas-size failure |\n"
    )
    assert mod.fail_result_lines(row_mark) == []


def test_clean_fixture_main_exits_0_and_check_matches(studio):
    status, out = run_main(main_argv(studio))
    assert status == 0, out
    assert "PROBLEMS:" not in out
    assert "73 entries" in out
    status, out = run_main(main_argv(studio, "--check"))
    assert status == 0, out
    assert "MATCHES" in out
    assert "problems=0" in out


def _cmp_committed_source():
    text = open(os.path.join(ROOT, ".github", "workflows", "ci.yml"), encoding="utf-8").read()
    match = re.search(r"^([ \t]*)cmp_committed\(\) \{.*?\n\1\}", text, re.M | re.S)
    assert match, "CI compare function cmp_committed() was not found"
    return match.group(0)


def _run_compare(root, name, fresh, committed_bytes, fresh_bytes):
    shot = root / "docs" / "screenshots"
    shot.mkdir(parents=True)
    (shot / name).write_bytes(committed_bytes)
    fresh.write_bytes(fresh_bytes)
    script = _cmp_committed_source() + f'\ncmp_committed "{name}" "{fresh}"\n'
    return subprocess.run(
        ["bash", "-c", script],
        cwd=root,
        capture_output=True,
        text=True,
    )


def test_ci_compare_match_prints_one_success_line(tmp_path):
    payload = b"plate-bytes"
    fresh = tmp_path / "fresh.png"
    proc = _run_compare(tmp_path / "repo", "plate.png", fresh, payload, payload)
    digest = hashlib.sha256(payload).hexdigest()
    assert proc.returncode == 0, proc.stderr
    assert proc.stdout == f"compared docs/screenshots/plate.png sha256={digest}\n"
    assert proc.stdout.count("\n") == 1


def test_ci_compare_mismatch_exits_nonzero(tmp_path):
    fresh = tmp_path / "fresh.png"
    proc = _run_compare(tmp_path / "repo", "plate.png", fresh, b"committed", b"fresh")
    assert proc.returncode != 0


def test_committed_manifest_entry_bytes_match():
    """Each committed MANIFEST entry sha is the bytes under godot/assets/landing.

    This does not pin a release version and does not rebuild the bundle.
    """
    landing = os.path.join(ROOT, "godot", "assets", "landing")
    manifest = json.loads(open(os.path.join(landing, "MANIFEST.json"), encoding="utf-8").read())
    entries = manifest["entries"]
    assert entries
    for entry in entries:
        path = os.path.join(landing, entry["path"])
        assert os.path.isfile(path), entry["path"]
        digest = hashlib.sha256(open(path, "rb").read()).hexdigest()
        assert digest == entry["sha256"], entry["id"]


if __name__ == "__main__":
    raise SystemExit(pytest.main([__file__, "-q", "--tb=short"]))
