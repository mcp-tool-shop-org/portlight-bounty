#!/usr/bin/env python3
"""Negative cases for the build_manifest verifier gates.

The studio bundle is not required. These fixtures call the same helpers
`build_manifest_pb002.py` uses. A passing real run still emits MANIFEST.json
only when `problems` stays empty; none of these cases do.
"""
from __future__ import annotations

import hashlib
import importlib.util
import os
import sys

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
    not problems and not mod.json_has_key(happy, "_STUB") and set(inv) == set(QUAY),
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
    "Recheck: quay_flag_b **FAIL** still open\n"
    "Result: **2/3 PASS**\n"
    "✅ Builder may land\n"
)
problems = []
mod.require_no_fail_results(doc, "retrieval-quay_flag_painterly.md", problems)
check("PASS doc with a FAIL result line", bool(problems), joined(problems))

summary = "**Result: 36 PASS, 0 FAIL**\n| **PASS** |\n**ANDON v0.1.1: 30 PASS, 0 FAIL.**\n"
check(
    "zero-FAIL summary is not a FAIL result",
    mod.fail_result_lines(summary) == [],
    "no FAIL result line",
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
    check(f"B2 key {key}", mod.json_has_key({key: True}, "_STUB"), "matched")
    check(
        f"B2 nested {key}",
        mod.json_has_key({"rows": [{"id": "quay_flag_a", "meta": {key: 1}}]}, "_STUB"),
        "matched",
    )
check("STUB without underscore is not _STUB", not mod.json_has_key({"STUB": 1}, "_STUB"), "ignored")
check("_STUB value is not a key", not mod.json_has_key({"note": "_STUB"}, "_STUB"), "ignored")

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

if failures:
    print(f"{len(failures)} case(s) did not fail closed")
    sys.exit(1)
print("all gate fixtures failed closed; studio inputs were not used")
