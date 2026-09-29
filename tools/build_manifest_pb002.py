#!/usr/bin/env python3
"""build_manifest_pb002.py - derive MANIFEST.json for the portlight-bounty PB-002 landing bundle.

    python3 tools/build_manifest_pb002.py --out-dir landing-next --version 0.3.0 [--check] [--studio DIR]
    (--out-dir is relative to pb-002-plates/ unless absolute; default "landing")

Never hand-edit MANIFEST.json; rerun this. Everything is derived from files + outbox docs:

  ships (36, P1)     ids/anchor/LOA/mesh      <- asset-list.csv; canvas <- PNG; sha256 computed
                     gates: art-gate.md Addendum D, retrieval-ships.md 36 PASS/0 FAIL; renderer <- ships/<class>/sidecar.json
                     sha vs source and vs grok-bot-verifier/PB-002/ships_results.json
                     ships inventory must equal the 36 ids (empty fails). A missing row refuses the build.
                     A row passes when `pass` is true. `exit`, if present, must be 0. A missing
                     `exit` is fine when `pass` is true. `v011_exit` is not a pass criterion.
                     A verdict of FAIL, a false `gate` value, or a non-empty `fails` list refuses
                     the row. Real rows look like {"pass": true, "v011_exit": 1} with no `exit`
                     key and are accepted as they are.
  harbour (30, P0)   ids/canvas/anchor/footprint/layer/offsets/andon_kind <- asset-list.csv
                     gates: Addendum E + E.1, retrieval-harbour.md 30 PASS/0 FAIL (v0.1.1); renderer <- <id>.render.json
                     sha vs source and vs harbour_andon_results.json; <id>.andon.json must be v0.1.1 pass
  chart (4, P0/P1)   ids/canvas/anchor/footprint/layer/offsets/andon_kind <- asset-list.csv
                     gates: Addendum F (PASS), retrieval-chart.md 4 PASS/0 FAIL + "Builder may land";
                     renderer <- <id>.render.json; andon gate of record <- <id>.andon.json
                     sha vs source and vs grok-bot-verifier/PB-002/chart/chart_retrieval_results.json
  quay_flag (3, P0)  ids quay_flag_a..c: painterly v5p quay stone (procedural v5 base + Salt Road img2img brushwork),
                     art-gate Addendum V.2 PASS; placement per Addendum U.4 (asset-list rows unchanged)
                     canvas/anchor/footprint/layer/offsets/andon_kind <- asset-list.csv; source ground/<id>.png;
                     <id>.andon.json must be v0.1.1 pass (kind ground); renderer <- <id>.render.json
                     (must carry base_square, source_square and brushwork{seed, graph_sha256});
                     MANDATORY Verifier gate (B1): grok-bot-verifier/PB-002/quay_flag_painterly_results.json
                     (3 rows pass/exit 0, inventory sha == plate) + outbox-PB-002/retrieval-quay_flag_painterly.md
                     ("Result: **3/3 PASS**", ends "✅ Builder may land")
                     B2: every gate input (ships, harbour, chart, quay_flag results JSON) refuses a key
                     whose tokens include stub (any case, any depth), including _STUB, _stub, and stub.
                     A key such as stubborn_note is not a stub marker.
                     The quay retrieval doc must contain its results sha256.
                     The results file is read once: those bytes are parsed and hashed.
                     Inventory ids must equal the expected plate set (quay_flag, harbour, and ships).
                     A real FAIL verdict refuses the build: any line containing **FAIL**, a line
                     whose verdict token is exactly FAIL (a plain line, or Verdict:/Result:), or a
                     Verdict/Result cell that is FAIL or FAIL (exit N) for any N. A counted summary
                     such as 0 FAIL does not contain **FAIL**. Prose that only mentions FAIL does not.
                     A verdict-column FAIL (exit N) is exempt only when that same row, or one single
                     line of the doc, marks it as the documented stock v0.1.1 canvas-size failure.
                     Terms spread across lines do not mark it. Bold **FAIL** is never exempt.
                     A duplicate row id or an unexpected row id also refuses the build.
                     A results file that is not a JSON object is a problem, not a traceback.
  import settings    per plate, from the plate's .import file if one exists (chart), else derived from
                     asset-spec Rev 4 R11 (recorded as source). A sidecar/.import that contradicts R11 aborts.

Layout (asset-spec Rev 4 §4 Output and §7g): chart/ships/ship_<class>/<id>.png, ground/water_*.png,
structures/<quay|pier id>/beauty.png, props/pier_pilings_1x1/beauty.png, chart/<id>.png.
Sidecars and .import files are NOT copied into the bundle (Coordinator ruling); their values are carried here.

Emitted paths: entry `path` is relative to the bundle root; provenance paths are relative to the studio
workspace root. The script refuses to write if any emitted string starts with '/' or contains '/workspace'.
csv "-" = not applicable by spec -> null + raw csv string kept; empty/unparseable -> null + reported.
--check: recompute and compare byte-for-byte against <out-dir>/MANIFEST.json; exit 1 on difference/problems.
"""
from __future__ import annotations
import argparse, csv, hashlib, json, os, re, struct, sys

DEFAULT_STUDIO = "/workspace/studio"  # read-side only; never emitted
HARB_SRC_DIRS = ["ground", "quay", "pier", "props"]
CLASSES = ["sloop", "cutter", "brigantine", "galleon"]
FRAMES = [f"f{k}" for k in range(8)] + ["wake"]
EXP_HARB = (["water_a", "water_b", "water_c"] + [f"quay_{i:04b}" for i in range(1, 16)]
            + ["pier_UL_DR", "pier_UR_DL", "pier_UL", "pier_UR", "pier_DR", "pier_DL",
               "pier_UL_UR_DR", "pier_UR_DR_DL", "pier_DR_DL_UL", "pier_DL_UL_UR", "pier_UL_UR_DR_DL",
               "pier_pilings_1x1"])
EXP_CHART = ["chart_water_a", "chart_water_b", "chart_water_c", "chart_port_marker"]
EXP_QUAY_FLAG = ["quay_flag_a", "quay_flag_b", "quay_flag_c"]
R11_MIP_ON = {"chart_water_a", "chart_water_b", "chart_water_c"}
R11_SRC = "asset-spec Rev 4 R11"


def bind_studio(studio):
    """Point every gate input at `studio`. The default root is DEFAULT_STUDIO."""
    global STUDIO, PLATES, OUTBOX, VERIFIER, CSV, SPEC, GATE
    global RETR_SHIPS, RETR_HARB, RETR_CHART, RETR_QUAY_FLAG
    global VJ_SHIPS, VJ_HARB, VJ_CHART, SHIP_SRC, CHART_SRC
    global QUAY_FLAG_SRC, VJ_QUAY_FLAG
    STUDIO = studio
    PLATES = os.path.join(STUDIO, "pb-002-plates")
    OUTBOX = os.path.join(STUDIO, "outbox-PB-002")
    VERIFIER = os.path.join(STUDIO, "grok-bot-verifier", "PB-002")
    CSV = os.path.join(OUTBOX, "asset-list.csv")
    SPEC = os.path.join(OUTBOX, "asset-spec.md")
    GATE = os.path.join(OUTBOX, "art-gate.md")
    RETR_SHIPS = os.path.join(OUTBOX, "retrieval-ships.md")
    RETR_HARB = os.path.join(OUTBOX, "retrieval-harbour.md")
    RETR_CHART = os.path.join(OUTBOX, "retrieval-chart.md")
    RETR_QUAY_FLAG = os.path.join(OUTBOX, "retrieval-quay_flag_painterly.md")
    VJ_SHIPS = os.path.join(VERIFIER, "ships_results.json")
    VJ_HARB = os.path.join(VERIFIER, "harbour_andon_results.json")
    VJ_CHART = os.path.join(VERIFIER, "chart", "chart_retrieval_results.json")
    SHIP_SRC = os.path.join(PLATES, "ships")
    CHART_SRC = os.path.join(PLATES, "chart")
    QUAY_FLAG_SRC = os.path.join(PLATES, "ground")
    VJ_QUAY_FLAG = os.path.join(VERIFIER, "quay_flag_painterly_results.json")


bind_studio(DEFAULT_STUDIO)


# ---------------------------------------------------------------- helpers
def sha256(p):
    h = hashlib.sha256()
    with open(p, "rb") as fh:
        for b in iter(lambda: fh.read(1 << 20), b""):
            h.update(b)
    return h.hexdigest()


def png_size(p):
    with open(p, "rb") as fh:
        head = fh.read(24)
    if head[:8] != b"\x89PNG\r\n\x1a\n" or head[12:16] != b"IHDR":
        raise SystemExit(f"not a PNG: {p}")
    return list(struct.unpack(">II", head[16:24]))


def srel(p):
    """studio-relative path for emission (never absolute)."""
    if p is None:
        return None
    ap = os.path.abspath(p)
    if not ap.startswith(STUDIO + os.sep):
        raise SystemExit(f"path outside studio cannot be emitted relatively: {p}")
    return os.path.relpath(ap, STUDIO)


def parse_xy(s):
    m = re.fullmatch(r"(-?\d+)\s*,\s*(-?\d+)", (s or "").strip())
    return [int(m.group(1)), int(m.group(2))] if m else None


def parse_int(s):
    m = re.match(r"(-?\d+)(\s|$|\()", (s or "").strip())
    return int(m.group(1)) if m else None


def parse_wh(s):
    m = re.match(r"(\d+)x(\d+)(\s|$)", (s or "").strip())
    return [int(m.group(1)), int(m.group(2))] if m else None


def csv_field(raw, aid, name, parser, nulls, na):
    raw = (raw or "").strip()
    if raw.startswith("-") and not re.match(r"-\d", raw):
        na.append(f"{aid}.{name} (csv {raw!r})")
        return None, raw
    v = parser(raw)
    if v is None:
        nulls.append(f"{aid}.{name} (csv {raw!r})")
    return v, raw


def read_import(p):
    """parse a Godot .import [params] section into a dict of raw strings."""
    out, sec = {}, None
    for line in open(p, encoding="utf-8"):
        line = line.strip()
        if line.startswith("["):
            sec = line.strip("[]")
        elif "=" in line and sec == "params":
            k, v = line.split("=", 1)
            out[k.strip()] = v.strip()
    return out


def load_json(p):
    return json.load(open(p, encoding="utf-8")) if os.path.isfile(p) else None


def read_file_bytes(p):
    """Read a file once. None when it is missing."""
    if not os.path.isfile(p):
        return None
    with open(p, "rb") as fh:
        return fh.read()


def sha256_bytes(raw):
    return hashlib.sha256(raw).hexdigest()


def parse_json_object(raw, label, problems):
    """Parse bytes already read. A missing file is the caller's message.
    Invalid JSON or a non-object top level is a problem, not a traceback.
    """
    if raw is None:
        return None
    try:
        obj = json.loads(raw)
    except json.JSONDecodeError as exc:
        problems.append(f"{label} is not valid JSON ({exc.msg})")
        return None
    if not isinstance(obj, dict):
        problems.append(f"{label} is not a JSON object")
        return None
    return obj


def json_has_key(obj, key):
    """True if any string key contains `key`, ignoring case, at any depth."""
    if isinstance(obj, dict):
        needle = key.lower()
        return any(isinstance(k, str) and needle in k.lower() for k in obj) or any(
            json_has_key(v, key) for v in obj.values()
        )
    if isinstance(obj, list):
        return any(json_has_key(v, key) for v in obj)
    return False


def _key_has_stub_token(key):
    """True when `stub` is its own token. stubborn_note does not match."""
    return any(part.lower() == "stub" for part in re.findall(r"[A-Za-z0-9]+", key))


def json_has_stub_key(obj):
    """True if any key has a stub token, ignoring case, at any depth."""
    if isinstance(obj, dict):
        for key, value in obj.items():
            if isinstance(key, str) and _key_has_stub_token(key):
                return True
            if json_has_stub_key(value):
                return True
        return False
    if isinstance(obj, list):
        return any(json_has_stub_key(value) for value in obj)
    return False


def reject_stub_keys(obj, label, problems):
    """Quay and harbour results, and every other gate input, refuse a stub key."""
    if obj is not None and json_has_stub_key(obj):
        problems.append(f"{label} has a stub key")


def _bare_cell(text):
    return re.sub(r"[*_`]", "", text).strip()


def _table_cells(line):
    return [cell.strip() for cell in line.strip().strip("|").split("|")]


def _is_table_row(line):
    stripped = line.strip()
    return stripped.startswith("|") and stripped.endswith("|") and stripped.count("|") >= 2


def _is_separator_row(line):
    if not _is_table_row(line):
        return False
    cells = _table_cells(line)
    return bool(cells) and all(re.fullmatch(r":?-{3,}:?", cell.replace(" ", "")) for cell in cells)


def _is_verdict_header(cell):
    return _bare_cell(cell).lower() in {"verdict", "result"}


def _marks_stock_v011_canvas(text):
    """True when one line marks the documented stock v0.1.1 canvas-size failure.

    Every term has to sit on that same line. A newline means the terms were
    spread, so the mark does not count.
    """
    if not text or "\n" in text or "\r" in text:
        return False
    if re.search(r"(?i)stock\s+v0\.1\.1\s+canvas(?:[-\s](?:size|scale)|scale)", text):
        return True
    return (
        re.search(r"(?i)v0\.1\.1", text) is not None
        and re.search(r"(?i)canvas(?:-size|\s+size|\s+scale)", text) is not None
        and re.search(r"(?i)stock|expected|documented", text) is not None
    )


def _verdict_cell_kind(cell):
    """'fail' or 'exit' when a verdict cell is a FAIL, else None.

    FAIL (exit N) is a verdict for any N. It must not pass only because it is
    not the bare token FAIL.
    """
    bare = _bare_cell(cell)
    if bare == "FAIL":
        return "fail"
    if re.fullmatch(r"FAIL\s*\(exit\s*\d+\)", bare):
        return "exit"
    return None


def _stock_canvas_exempt(line, doc, kind):
    """A verdict-column FAIL (exit N) is exempt only from a single-line mark.

    The mark is this row, or one other line of the doc. Terms spread across
    lines do not exempt the row. A bare FAIL is exempt only when its own row
    carries the mark. Bold **FAIL** never reaches this helper.
    """
    if _marks_stock_v011_canvas(line):
        return True
    if kind != "exit":
        return False
    return any(_marks_stock_v011_canvas(ln) for ln in doc.splitlines())


def _line_verdict_token_is_fail(line):
    """A plain FAIL / **FAIL** line, or Verdict:/Result: whose token is exactly that."""
    if re.fullmatch(r"\**\s*FAIL\s*\**", line):
        return True
    labelled = re.match(r"(?i)^(?:\**\s*)?(?:verdict|result)\s*:\s*(.*?)\s*$", line)
    if not labelled:
        return False
    token = _bare_cell(labelled.group(1)).rstrip(".")
    return token == "FAIL"


def _verdict_columns(header):
    return [i for i, cell in enumerate(header) if _is_verdict_header(cell)]


def fail_result_lines(text):
    """Lines that report a real FAIL verdict.

    A counted summary such as '0 FAIL' is not one: it does not contain **FAIL**.
    Neither is prose that merely mentions FAIL. Any line containing **FAIL**
    is a verdict and is not stock-exempt. A Verdict or Result cell counts when
    it is FAIL or FAIL (exit N) for any N. An exit-N cell is exempt only when
    that row, or one single line of the doc, marks the documented stock v0.1.1
    canvas-size failure.
    """
    raw_lines = text.splitlines()
    header_for = {}
    current = None
    for index, raw in enumerate(raw_lines):
        line = raw.strip()
        if _is_separator_row(line) and index > 0 and _is_table_row(raw_lines[index - 1]):
            current = _table_cells(raw_lines[index - 1])
            continue
        if current is not None and _is_table_row(line) and not _is_separator_row(line):
            header_for[index] = current
        elif line and not _is_table_row(line):
            current = None
    found = []
    for index, raw in enumerate(raw_lines):
        line = raw.strip()
        if not line:
            continue
        # Main refuses any line that contains the bold token **FAIL**.
        # **Result: 36 PASS, 0 FAIL** does not contain that token.
        if "**FAIL**" in line:
            found.append(line)
            continue
        if _line_verdict_token_is_fail(line):
            found.append(line)
            continue
        header = header_for.get(index)
        if not header:
            continue
        cells = _table_cells(line)
        kinds = [
            kind
            for column in _verdict_columns(header)
            if column < len(cells)
            for kind in (_verdict_cell_kind(cells[column]),)
            if kind
        ]
        if kinds and not all(_stock_canvas_exempt(line, text, kind) for kind in kinds):
            found.append(line)
    return found


def require_no_fail_results(text, label, problems):
    lines = fail_result_lines(text)
    if lines:
        problems.append(f"{label} has a FAIL result line: {lines[0]}")


def bind_inventory(obj, expected, label, problems):
    """id -> sha256. The id set must equal `expected`. Duplicate ids are an error."""
    if obj is None:
        return {}
    inv = obj.get("inventory")
    if not isinstance(inv, list):
        problems.append(f"{label} inventory is not a list")
        return {}
    mapping = {}
    for row in inv:
        if not isinstance(row, dict) or "id" not in row:
            problems.append(f"{label} inventory entry missing id")
            continue
        aid = row["id"]
        if aid in mapping:
            problems.append(f"{label} duplicate inventory id {aid}")
        mapping[aid] = row.get("sha256")
    if set(mapping) != set(expected):
        problems.append(
            f"{label} inventory ids {sorted(set(mapping))} != {sorted(expected)}"
        )
    return mapping


def index_rows(obj, expected, label, problems):
    """id -> row. Duplicate ids and ids outside `expected` are errors.
    The last row is kept so a later pass check still sees a row; the duplicate
    is already a problem, so fail-then-pass does not land.
    """
    if obj is None:
        return {}
    rows = obj.get("rows", [])
    if not isinstance(rows, list):
        problems.append(f"{label} rows is not a list")
        return {}
    expected_set = set(expected)
    mapping = {}
    for row in rows:
        if not isinstance(row, dict) or "id" not in row:
            problems.append(f"{label} row missing id")
            continue
        aid = row["id"]
        if aid in mapping:
            problems.append(f"{label} duplicate row id {aid}")
        if aid not in expected_set:
            problems.append(f"{label} unexpected row id {aid}")
        mapping[aid] = row
    return mapping


def _row_field(row, name):
    """(present, value) for a key equal to `name`, ignoring case."""
    for key, value in row.items():
        if isinstance(key, str) and key.lower() == name:
            return True, value
    return False, None


def _text_verdict_fail(value):
    if not isinstance(value, str):
        return False
    if "**FAIL**" in value:
        return True
    bare = _bare_cell(value).rstrip(".")
    return bare == "FAIL" or re.fullmatch(r"FAIL\s*\(exit\s*\d+\)", bare) is not None


def _contains_false(value):
    if value is False:
        return True
    if isinstance(value, dict):
        return any(_contains_false(item) for item in value.values())
    if isinstance(value, list):
        return any(_contains_false(item) for item in value)
    return False


def ship_row_failure(row):
    """Why a ships verifier row refuses the build, or None when it may land.

    A row passes when `pass` is true. `exit`, if present, must be 0. A missing
    `exit` is fine when `pass` is true. `v011_exit` is not a pass criterion.
    `verdict` FAIL, a false `gate` value, or a non-empty `fails` list refuses
    the row. Real rows are {"pass": true, "v011_exit": 1, ...} with no exit key.
    """
    if not isinstance(row, dict):
        return "missing"
    if row.get("pass") is not True:
        return "not pass"
    if "exit" in row and row.get("exit") != 0:
        return "nonzero exit"
    present, verdict = _row_field(row, "verdict")
    if present and _text_verdict_fail(verdict):
        return "verdict FAIL"
    present, gate = _row_field(row, "gate")
    if present and _contains_false(gate):
        return "false gate"
    present, fails = _row_field(row, "fails")
    if present and isinstance(fails, list) and fails:
        return "non-empty fails"
    return None


def walk_strings(o):
    if isinstance(o, str):
        yield o
    elif isinstance(o, dict):
        for k, v in o.items():
            yield from walk_strings(k)
            yield from walk_strings(v)
    elif isinstance(o, list):
        for v in o:
            yield from walk_strings(v)


# ---------------------------------------------------------------- import settings
def import_settings(aid, src_png, problems):
    """.import file if present (must agree with R11), else derived from R11."""
    want_mip = aid in R11_MIP_ON
    imp_p = src_png + ".import"
    if os.path.isfile(imp_p):
        prm = read_import(imp_p)
        got = {"mipmaps_generate": prm.get("mipmaps/generate"), "fix_alpha_border": prm.get("process/fix_alpha_border"),
               "premult_alpha": prm.get("process/premult_alpha"), "compress_mode": prm.get("compress/mode"),
               "size_limit": prm.get("process/size_limit")}
        exp = {"mipmaps_generate": "true" if want_mip else "false", "fix_alpha_border": "true",
               "premult_alpha": "false", "compress_mode": "0"}
        for k, v in exp.items():
            if got[k] != v:
                problems.append(f"STOP-R11: {aid} .import {k}={got[k]!r} contradicts {R11_SRC} (expects {v})")
        return {"mipmaps": got["mipmaps_generate"] == "true", "fix_alpha_border": got["fix_alpha_border"] == "true",
                "premult_alpha": got["premult_alpha"] == "true",
                "compress": "Lossless" if got["compress_mode"] == "0" else f"mode {got['compress_mode']}",
                "size_limit": int(got["size_limit"]) if got["size_limit"] and got["size_limit"].isdigit() else None,
                "source": srel(imp_p) + f" (agrees with {R11_SRC})"}
    return {"mipmaps": want_mip, "fix_alpha_border": True, "premult_alpha": False, "compress": "Lossless",
            "size_limit": None, "source": R11_SRC}


def cross_check_sidecar_mip(aid, val, where, problems):
    if val is None:
        return
    if bool(val) != (aid in R11_MIP_ON):
        problems.append(f"STOP-R11: {aid} {where} mipmaps={val!r} contradicts {R11_SRC}")


# ---------------------------------------------------------------- main
def main(argv=None):
    ap = argparse.ArgumentParser()
    ap.add_argument("--version", default="0.1.0")
    ap.add_argument("--out-dir", default="landing", help="bundle dir (relative to pb-002-plates/ unless absolute)")
    ap.add_argument("--groups", default="ships,harbour,chart,quay_flag")
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--studio", default=DEFAULT_STUDIO, help="studio root holding plates, outbox, and verifier inputs")
    a = ap.parse_args(argv)
    bind_studio(a.studio)
    OUT = a.out_dir if os.path.isabs(a.out_dir) else os.path.join(PLATES, a.out_dir)
    groups = set(a.groups.split(","))
    problems, nulls, na = [], [], []
    gate = open(GATE, encoding="utf-8").read()
    spec = open(SPEC, encoding="utf-8").read()
    rows_all = list(csv.DictReader(open(CSV, encoding="utf-8")))

    # R11 text must be the Rev 4 wording the import rule encodes
    r11 = re.search(r"^\| R11 \|(.*)$", spec, re.M)
    if not re.search(r"Seat: Game Designer · Rev 4", spec):
        problems.append("asset-spec.md is not Rev 4")
    if not r11 or "Mipmaps ON for `chart_water_a..c` only" not in r11.group(1) or "Fix Alpha Border ON everywhere" not in r11.group(1):
        problems.append("asset-spec R11 wording not as encoded (chart_water_a..c mipmaps only; fix alpha border everywhere)")

    entries, counts = [], {}
    qgate = {"addendum": "V.2"}

    # ================= ships =================
    if "ships" in groups:
        retr = open(RETR_SHIPS, encoding="utf-8").read()
        require_no_fail_results(retr, "retrieval-ships.md", problems)
        m = re.search(r"\*\*Result:\s*(\d+)\s+PASS,\s*(\d+)\s+FAIL\*\*", retr)
        n_rows = len(re.findall(r"\|\s*\*\*PASS\*\*\s*\|", retr))
        if not m or m.groups() != ("36", "0") or n_rows != 36:
            problems.append(f"retrieval-ships.md not 36 PASS/0 FAIL (summary={m.groups() if m else None}, rows={n_rows})")
        if not re.search(r"## Addendum D .*?\*\*Result: PASS\. All 36 ids are will-use", gate, re.S):
            problems.append("art-gate.md Addendum D PASS not found")
        expected = [f"ship_{c}_{f}" for c in CLASSES for f in FRAMES]
        vraw = read_file_bytes(VJ_SHIPS)
        vj = parse_json_object(vraw, "ships_results.json", problems)
        if vraw is None:
            problems.append("verifier ships_results.json missing")
        # Empty inventory is a set mismatch. Rows are read and must pass.
        vinv = bind_inventory(vj, expected, "ships", problems) if vj is not None else {}
        vrows = index_rows(vj, expected, "ships", problems) if vj is not None else {}
        reject_stub_keys(vj, "ships_results.json", problems)
        spec_mesh_prefix = dict(re.findall(r"\|\s*`(ship_[a-z_]+)`\s*\|.*?\|\s*`([0-9a-f]{12})`\s*\|", spec))
        if len(spec_mesh_prefix) != 4:
            problems.append(f"asset-spec §7a mesh sha prefixes not found (got {spec_mesh_prefix})")
        rows = [r for r in rows_all if r["id"].startswith("ship_") and r["status"].startswith("LOCKED")]
        if sorted(r["id"] for r in rows) != sorted(expected):
            problems.append("csv LOCKED ship ids != expected 36")
        mesh_cache, side_cache, ship_e = {}, {}, []
        for r in rows:
            aid = r["id"]
            cls = re.fullmatch(r"ship_([a-z_]+)_(f[0-7]|wake)", aid).group(1)
            base = f"ship_{cls}"
            rel = f"chart/ships/{base}/{aid}.png"
            p, src = os.path.join(OUT, rel), os.path.join(SHIP_SRC, base, f"{aid}.png")
            if not os.path.isfile(src):
                problems.append(f"{aid}: source missing"); continue
            if not os.path.isfile(p):
                problems.append(f"missing landed file {rel}"); continue
            w, h = png_size(p); digest = sha256(p)
            if sha256(src) != digest: problems.append(f"{aid}: landed bytes differ from source")
            if vj is not None and vinv.get(aid) != digest: problems.append(f"{aid}: sha256 differs from Verifier inventory")
            ship_row = vrows.get(aid)
            ship_failure = ship_row_failure(ship_row) if vj is not None else None
            if ship_failure:
                problems.append(f"{aid}: Verifier ships row {ship_failure}")
            if r["canvas"].strip() != f"{w}x{h}": problems.append(f"{aid}: canvas {w}x{h} != csv {r['canvas']}")
            anchor, _ = csv_field(r["anchor_px"], aid, "anchor", parse_xy, nulls, na)
            loa, _ = csv_field(r["hull_len_px"], aid, "LOA", parse_int, nulls, na)
            mesh_file = re.split(r"\s+\(", r["mesh_source"].strip())[0] or None
            if mesh_file and mesh_file not in mesh_cache:
                mesh_cache[mesh_file] = sha256(mesh_file) if os.path.isfile(mesh_file) else None
            msha = mesh_cache.get(mesh_file)
            if msha is None: problems.append(f"{aid}: mesh file missing {mesh_file}")
            elif base in spec_mesh_prefix and not msha.startswith(spec_mesh_prefix[base]):
                problems.append(f"{aid}: mesh sha {msha[:12]} != asset-spec §7a {spec_mesh_prefix[base]}")
            if base not in side_cache:
                side_cache[base] = load_json(os.path.join(SHIP_SRC, base, "sidecar.json")) or {}
            sd = side_cache[base]
            rot = parse_int(r["rot_z_f0..f7"])
            ship_e.append({
                "id": aid, "group": "ships", "class": base, "frame": aid.rsplit("_", 1)[1], "path": rel,
                "canvas": [w, h], "canvas_str": f"{w}x{h}", "anchor": anchor,
                "LOA": {"px": loa, "definition": "hull length = length overall incl. bowsprit and boom, at f3/f7 (asset-spec §7c; ship-footprints Rev 2)"} if loa is not None else None,
                "rot_z": rot, "view": r["view"], "phase": r["phase"], "status": r["status"],
                "sha256": digest, "bytes": os.path.getsize(p),
                "import": import_settings(aid, src, problems),
                "provenance": {
                    "mesh": {"hull": r["hull"] or None, "file": srel(mesh_file), "sha256": msha,
                             "note": "wake: rig only, disc (no mesh in frame)" if aid.endswith("_wake") else None},
                    "renderer": {"name": "Blender", "version": sd.get("blender"), "engine": sd.get("engine"),
                                 "samples": sd.get("samples"), "summary": "Blender 5.2 EEVEE"},
                    "gate": {"art_director": "art-gate.md Addendum D (05:15 ET): PASS, 36 will-use",
                             "verifier": "retrieval-ships.md: 36 PASS / 0 FAIL",
                             "andon_profile": "andon-chart-profile.md Rev 1 (ship) + art-gate Addenda B/C"},
                    "render_sidecar": srel(os.path.join(SHIP_SRC, base, "sidecar.json")),
                    "source_path": srel(src)}})
        order = {e: i for i, e in enumerate(expected)}
        ship_e.sort(key=lambda e: order[e["id"]])
        entries += ship_e; counts["ships"] = len(ship_e)

    # ================= harbour =================
    if "harbour" in groups:
        rh = open(RETR_HARB, encoding="utf-8").read()
        require_no_fail_results(rh, "retrieval-harbour.md", problems)
        mh = re.search(r"\*\*ANDON v0\.1\.1:\s*(\d+)\s+PASS,\s*(\d+)\s+FAIL\.\*\*", rh)
        n_rows = len(re.findall(r"\|\s*\*\*PASS\*\* \(exit 0\)\s*\|", rh))
        if not mh or mh.groups() != ("30", "0") or n_rows != 30:
            problems.append(f"retrieval-harbour.md not 30 PASS/0 FAIL (summary={mh.groups() if mh else None}, rows={n_rows})")
        if not re.search(r"## Addendum E .*?\*\*Result: PASS\. All 30 are will-use", gate, re.S):
            problems.append("art-gate.md Addendum E PASS not found")
        if not re.search(r"AMENDED 05:25 ET \(Addendum E\.1\)\.\*\*.*?still \*\*Pass\*\*", gate, re.S):
            problems.append("art-gate.md Addendum E.1 Pass not found")
        hraw = read_file_bytes(VJ_HARB)
        hj = parse_json_object(hraw, "harbour_andon_results.json", problems)
        if hraw is None:
            problems.append("verifier harbour_andon_results.json missing")
        hinv = bind_inventory(hj, EXP_HARB, "harbour", problems) if hj is not None else {}
        hrows = index_rows(hj, EXP_HARB, "harbour", problems) if hj is not None else {}
        reject_stub_keys(hj, "harbour_andon_results.json", problems)
        rows = [r for r in rows_all if r["phase"] == "P0" and r["view"] == "harbour" and r["status"].startswith("LOCKED") and r["id"] not in EXP_QUAY_FLAG]
        if sorted(r["id"] for r in rows) != sorted(EXP_HARB):
            problems.append("csv harbour P0 ids != expected 30")
        harb_e = []
        for r in rows:
            aid = r["id"]
            if aid.startswith("water_"): rel = f"ground/{aid}.png"
            elif aid == "pier_pilings_1x1": rel = f"props/{aid}/beauty.png"
            else: rel = f"structures/{aid}/beauty.png"
            hits = [os.path.join(PLATES, d, aid + ".png") for d in HARB_SRC_DIRS if os.path.isfile(os.path.join(PLATES, d, aid + ".png"))]
            if len(hits) != 1: problems.append(f"{aid}: source missing/ambiguous"); continue
            src = hits[0]; p = os.path.join(OUT, rel)
            if not os.path.isfile(p): problems.append(f"missing landed file {rel}"); continue
            w, h = png_size(p); digest = sha256(p)
            if sha256(src) != digest: problems.append(f"{aid}: landed bytes differ from source")
            if hj is not None and hinv.get(aid) != digest: problems.append(f"{aid}: sha256 differs from Verifier harbour inventory")
            if hj is not None and (not isinstance(hrows.get(aid), dict) or hrows[aid].get("pass") is not True):
                problems.append(f"{aid}: Verifier harbour row missing or not pass")
            if r["canvas"].strip() != f"{w}x{h}": problems.append(f"{aid}: canvas {w}x{h} != csv {r['canvas']}")
            anchor, _ = csv_field(r["anchor_px"], aid, "anchor", parse_xy, nulls, na)
            fp, fp_raw = csv_field(r["footprint"], aid, "footprint", parse_wh, nulls, na)
            lo, _ = csv_field(r["layer_offset_px"], aid, "layer_offset_px", parse_xy, nulls, na)
            to, to_raw = csv_field(r["texture_origin"], aid, "texture_origin", parse_xy, nulls, na)
            ys, ys_raw = csv_field(r["y_sort_origin"], aid, "y_sort_origin", parse_int, nulls, na)
            base = os.path.splitext(src)[0]
            andon = load_json(base + ".andon.json")
            if not andon or andon.get("andon_version") != "0.1.1" or andon.get("pass") is not True:
                problems.append(f"{aid}: source .andon.json missing / not v0.1.1 pass")
            rj = load_json(base + ".render.json") or {}
            if not rj: problems.append(f"{aid}: .render.json missing")
            cross_check_sidecar_mip(aid, rj.get("mipmaps"), ".render.json", problems)
            harb_e.append({
                "id": aid, "group": "harbour",
                "subgroup": "water" if aid.startswith("water_") else ("pilings" if aid == "pier_pilings_1x1" else aid.split("_")[0]),
                "path": rel, "canvas": [w, h], "canvas_str": f"{w}x{h}", "anchor": anchor, "LOA": None,
                "footprint": fp, "layer": r["layer"].strip() or None, "layer_offset_px": lo,
                "texture_origin": to, "texture_origin_csv": to_raw if to is None else None,
                "y_sort_origin": ys, "y_sort_origin_csv": ys_raw if ys is None else None,
                "andon_kind": r["andon_kind"].strip() or None,
                "view": r["view"], "phase": r["phase"], "status": r["status"],
                "sha256": digest, "bytes": os.path.getsize(p),
                "import": import_settings(aid, src, problems),
                "provenance": {
                    "mesh": None,
                    "renderer": {"name": "Blender", "version": rj.get("blender"), "engine": rj.get("engine"),
                                 "samples": rj.get("samples"), "scene_kind": rj.get("kind"),
                                 "camera_euler_deg": rj.get("camera_euler_deg"), "ortho_scale": rj.get("ortho_scale"),
                                 "px_per_bu": rj.get("px_per_bu"), "sun_euler_deg": rj.get("sun_euler_deg"),
                                 "summary": "Blender 5.2 EEVEE"},
                    "andon": {"version": andon.get("andon_version") if andon else None,
                              "kind": andon.get("kind") if andon else None, "pass": andon.get("pass") if andon else None},
                    "gate": {"art_director": "art-gate.md Addendum E (05:20 ET): PASS, 30 will-use; Addendum E.1 (05:25 ET): seams Pass under Godot sRGB, hdr_2d=false",
                             "verifier": "retrieval-harbour.md: 30 PASS / 0 FAIL on iso_andon v0.1.1"},
                    "render_sidecar": srel(base + ".render.json"),
                    "source_path": srel(src)}})
        horder = {e: i for i, e in enumerate(EXP_HARB)}
        harb_e.sort(key=lambda e: horder[e["id"]])
        entries += harb_e; counts["harbour"] = len(harb_e)

    # ================= chart =================
    if "chart" in groups:
        rc = open(RETR_CHART, encoding="utf-8").read()
        require_no_fail_results(rc, "retrieval-chart.md", problems)
        mc = re.search(r"\*\*Chart profile \(SIMULATED\):\s*(\d+)\s+PASS,\s*(\d+)\s+FAIL\.\*\*", rc)
        if not mc or mc.groups() != ("4", "0"):
            problems.append(f"retrieval-chart.md not 4 PASS/0 FAIL (got {mc.groups() if mc else None})")
        if not re.search(r"✅ Builder may land\s*$", rc):
            problems.append("retrieval-chart.md does not end with '✅ Builder may land'")
        gate_f = re.search(r"## Addendum F \(([^)]*)\).*?\*\*Chart set: PASS\. All 4 are will-use\*\*", gate, re.S)
        if not gate_f:
            problems.append("art-gate.md Addendum F chart PASS not found")
        craw = read_file_bytes(VJ_CHART)
        cj = parse_json_object(craw, "chart_retrieval_results.json", problems)
        if craw is None:
            problems.append("verifier chart_retrieval_results.json missing")
        reject_stub_keys(cj, "chart_retrieval_results.json", problems)
        rows = [r for r in rows_all if r["view"] == "chart" and r["id"] in EXP_CHART and r["status"].startswith("LOCKED")]
        if sorted(r["id"] for r in rows) != sorted(EXP_CHART):
            problems.append(f"csv chart ids != expected 4: {[r['id'] for r in rows]}")
        chart_e = []
        for r in rows:
            aid = r["id"]
            rel = f"chart/{aid}.png"
            src = os.path.join(CHART_SRC, aid + ".png"); p = os.path.join(OUT, rel)
            if not os.path.isfile(src): problems.append(f"{aid}: source missing"); continue
            if not os.path.isfile(p): problems.append(f"missing landed file {rel}"); continue
            w, h = png_size(p); digest = sha256(p)
            if sha256(src) != digest: problems.append(f"{aid}: landed bytes differ from source")
            if cj is not None:
                cv = cj.get(aid) or {}
                if cv.get("sha256") != digest: problems.append(f"{aid}: sha256 differs from Verifier chart results")
                if not (cv.get("sim") or {}).get("pass"): problems.append(f"{aid}: Verifier chart sim not pass")
            if r["canvas"].strip() != f"{w}x{h}": problems.append(f"{aid}: canvas {w}x{h} != csv {r['canvas']}")
            anchor, _ = csv_field(r["anchor_px"], aid, "anchor", parse_xy, nulls, na)
            fp, fp_raw = csv_field(r["footprint"], aid, "footprint", parse_wh, nulls, na)
            lo, _ = csv_field(r["layer_offset_px"], aid, "layer_offset_px", parse_xy, nulls, na)
            to, to_raw = csv_field(r["texture_origin"], aid, "texture_origin", parse_xy, nulls, na)
            ys, ys_raw = csv_field(r["y_sort_origin"], aid, "y_sort_origin", parse_int, nulls, na)
            base = os.path.splitext(src)[0]
            andon = load_json(base + ".andon.json") or {}
            gor = andon.get("andon_gate_of_record") or {}
            if gor.get("pass") is not True or gor.get("exit_code") != 0:
                problems.append(f"{aid}: .andon.json gate of record not pass")
            cross_check_sidecar_mip(aid, andon.get("import_mipmaps_generate", gor.get("import_mipmaps_generate")), ".andon.json", problems)
            rj = load_json(base + ".render.json") or {}
            if not rj: problems.append(f"{aid}: .render.json missing")
            gi = rj.get("godot_import", "")
            if gi:
                s_on, s_off = re.search(r"mipmaps on", gi, re.I), re.search(r"mipmaps off", gi, re.I)
                cross_check_sidecar_mip(aid, True if s_on and not s_off else (False if s_off else None), ".render.json godot_import", problems)
            if (rj.get("post") or {}).get("resize") != "none":
                problems.append(f"{aid}: render.json post.resize is not 'none'")
            chart_e.append({
                "id": aid, "group": "chart", "subgroup": "water" if aid.startswith("chart_water_") else "marker",
                "path": rel, "canvas": [w, h], "canvas_str": f"{w}x{h}", "anchor": anchor, "LOA": None,
                "footprint": fp, "footprint_csv": fp_raw if fp_raw != (f"{fp[0]}x{fp[1]}" if fp else None) else None,
                "layer": r["layer"].strip() or None, "layer_offset_px": lo,
                "texture_origin": to, "texture_origin_csv": to_raw if to is None else None,
                "y_sort_origin": ys, "y_sort_origin_csv": ys_raw if (ys is None or not re.fullmatch(r"-?\d+", ys_raw)) else None,
                "andon_kind": r["andon_kind"].strip() or None,
                "view": r["view"], "phase": r["phase"], "status": r["status"],
                "sha256": digest, "bytes": os.path.getsize(p),
                "import": import_settings(aid, src, problems),
                "provenance": {
                    "mesh": None,
                    "renderer": {"name": "Blender", "version": rj.get("blender"), "engine": rj.get("engine"),
                                 "samples": rj.get("samples"), "scene_kind": rj.get("kind"),
                                 "camera_euler_deg": rj.get("camera_euler_deg"), "ortho_scale": rj.get("ortho_scale"),
                                 "px_per_bu": rj.get("px_per_bu"), "sun_euler_deg": rj.get("sun_euler_deg"),
                                 "post": rj.get("post"), "summary": "Blender 5.2 EEVEE"},
                    "andon": {"profile": gor.get("profile"), "kind": gor.get("kind"), "pass": gor.get("pass"),
                              "exit_code": gor.get("exit_code"),
                              "stock_v011": "fails on canvas scale as expected (retrieval-chart.md)"},
                    "gate": {"art_director": f"art-gate.md Addendum F ({gate_f.group(1) if gate_f else '?'}): chart set PASS, 4 will-use (marker mipmaps OFF)",
                             "verifier": "retrieval-chart.md: 4 PASS / 0 FAIL (chart profile Rev 1, simulated), '✅ Builder may land'"},
                    "render_sidecar": srel(base + ".render.json"),
                    "import_sidecar": srel(src + ".import") if os.path.isfile(src + ".import") else None,
                    "source_path": srel(src)}})
        corder = {e: i for i, e in enumerate(EXP_CHART)}
        chart_e.sort(key=lambda e: corder[e["id"]])
        entries += chart_e; counts["chart"] = len(chart_e)

    # ================= quay_flag (painterly v5p quay stone ground, Addendum V.2) =================
    if "quay_flag" in groups:
        gate_v2 = re.search(r"## Addendum V\.2 [^\n]*painterly v5p[^\n]*PASS[^\n]*\n", gate)
        if not gate_v2:
            problems.append("art-gate.md Addendum V.2 (painterly v5p) PASS not found")
        if not re.search(r"## Addendum U\.4 [^\n]*placement", gate):
            problems.append("art-gate.md Addendum U.4 (placement) not found")
        # Cited gate. Default: Addendum V.2 (unchanged behaviour). If the quay_flag <id>.render.json sidecars all carry the
        # same "manifest_gate" {"addendum", "art_director", "summary"}, that addendum must be a PASS heading in art-gate.md,
        # and its strings are cited instead (W.3 for the W.2 retone).
        qgate = {"addendum": "V.2",
                 "art_director": "art-gate.md Addendum V.2 (2026-09-29): PASS, painterly v5p quay stone a/b/c; placement Addendum U.4",
                 "summary": "procedural v5 square -> Salt Road img2img brushwork (seed 1500) [b/c: a's border re-pasted, v5p] -> 2:1 projection + diamond_clip --feather 8"}
        sgates = [(load_json(os.path.join(QUAY_FLAG_SRC, q + ".render.json")) or {}).get("manifest_gate") for q in EXP_QUAY_FLAG]
        if any(sgates):
            if not all(isinstance(g, dict) and g == sgates[0] and all(isinstance(g.get(k), str) and g.get(k) for k in qgate) for g in sgates):
                problems.append("quay_flag render.json manifest_gate missing, malformed or not identical on all ids")
            else:
                if not re.search(r"## Addendum " + re.escape(sgates[0]["addendum"]) + r" [^\n]*PASS[^\n]*\n", gate):
                    problems.append(f"art-gate.md Addendum {sgates[0]['addendum']} PASS not found (render.json manifest_gate)")
                qgate = {k: sgates[0][k] for k in qgate}
        qraw = read_file_bytes(VJ_QUAY_FLAG)
        qj = parse_json_object(qraw, "quay_flag_painterly_results.json", problems)
        if qraw is None:
            problems.append("verifier quay_flag_painterly_results.json missing")
        reject_stub_keys(qj, "quay_flag_painterly_results.json", problems)
        qinv = bind_inventory(qj, EXP_QUAY_FLAG, "quay_flag", problems) if qj is not None else {}
        qrows = index_rows(qj, EXP_QUAY_FLAG, "quay_flag", problems) if qj is not None else {}
        for q in EXP_QUAY_FLAG:
            qr = qrows.get(q)
            if not qr or qr.get("pass") is not True or qr.get("exit") != 0:
                problems.append(f"{q}: Verifier quay_flag row missing or not pass/exit 0")
        rq = open(RETR_QUAY_FLAG, encoding="utf-8").read() if os.path.isfile(RETR_QUAY_FLAG) else ""
        require_no_fail_results(rq, "retrieval-quay_flag_painterly.md", problems)
        if qraw is not None and sha256_bytes(qraw) not in rq:
            problems.append("retrieval-quay_flag_painterly.md does not contain the sha256 of quay_flag_painterly_results.json")
        if not re.search(r"Result: \*\*3/3 PASS\*\*", rq):
            problems.append("retrieval-quay_flag_painterly.md missing or not 'Result: **3/3 PASS**'")
        if not re.search(r"✅ Builder may land\s*$", rq):
            problems.append("retrieval-quay_flag_painterly.md does not end with '✅ Builder may land'")
        rows = [r for r in rows_all if r["id"] in EXP_QUAY_FLAG and r["status"].startswith("LOCKED")]
        if sorted(r["id"] for r in rows) != sorted(EXP_QUAY_FLAG):
            problems.append(f"csv quay_flag ids != expected 3: {[r['id'] for r in rows]}")
        qf_e = []
        for r in rows:
            aid = r["id"]
            rel = f"ground/{aid}.png"
            src = os.path.join(QUAY_FLAG_SRC, aid + ".png"); p = os.path.join(OUT, rel)
            if not os.path.isfile(src): problems.append(f"{aid}: source missing"); continue
            if not os.path.isfile(p): problems.append(f"missing landed file {rel}"); continue
            w, h = png_size(p); digest = sha256(p)
            if sha256(src) != digest: problems.append(f"{aid}: landed bytes differ from source")
            if qj is not None and qinv.get(aid) != digest: problems.append(f"{aid}: sha256 differs from Verifier quay_flag inventory")
            if r["canvas"].strip() != f"{w}x{h}": problems.append(f"{aid}: canvas {w}x{h} != csv {r['canvas']}")
            anchor, _ = csv_field(r["anchor_px"], aid, "anchor", parse_xy, nulls, na)
            fp, fp_raw = csv_field(r["footprint"], aid, "footprint", parse_wh, nulls, na)
            lo, _ = csv_field(r["layer_offset_px"], aid, "layer_offset_px", parse_xy, nulls, na)
            to, to_raw = csv_field(r["texture_origin"], aid, "texture_origin", parse_xy, nulls, na)
            ys, ys_raw = csv_field(r["y_sort_origin"], aid, "y_sort_origin", parse_int, nulls, na)
            base = os.path.splitext(src)[0]
            andon = load_json(base + ".andon.json")
            if not andon or andon.get("andon_version") != "0.1.1" or andon.get("pass") is not True or andon.get("kind") != "ground":
                problems.append(f"{aid}: source .andon.json missing / not v0.1.1 ground pass")
            rj = load_json(base + ".render.json") or {}
            if not rj: problems.append(f"{aid}: .render.json missing")
            if rj.get("source_square", {}).get("sha256") is None: problems.append(f"{aid}: .render.json has no source_square sha256")
            bw = rj.get("brushwork") or {}
            if (rj.get("base_square") or {}).get("sha256") is None: problems.append(f"{aid}: .render.json has no base_square sha256")
            if bw.get("seed") is None or not bw.get("graph_sha256"): problems.append(f"{aid}: .render.json brushwork seed/graph_sha256 missing")
            cross_check_sidecar_mip(aid, rj.get("mipmaps"), ".render.json", problems)
            qf_e.append({
                "id": aid, "group": "quay_flag", "subgroup": "flag",
                "path": rel, "canvas": [w, h], "canvas_str": f"{w}x{h}", "anchor": anchor, "LOA": None,
                "footprint": fp, "layer": r["layer"].strip() or None, "layer_offset_px": lo,
                "texture_origin": to, "texture_origin_csv": to_raw if to is None else None,
                "y_sort_origin": ys, "y_sort_origin_csv": ys_raw if ys is None else None,
                "andon_kind": r["andon_kind"].strip() or None,
                "view": r["view"], "phase": r["phase"], "status": r["status"],
                "sha256": digest, "bytes": os.path.getsize(p),
                "import": import_settings(aid, src, problems),
                "provenance": {
                    "mesh": None,
                    "renderer": {"name": "procedural base + img2img brushwork", "version": None, "engine": rj.get("engine"), "samples": None,
                                 "supersample": rj.get("supersample"), "scene_kind": rj.get("kind"),
                                 "camera_euler_deg": rj.get("camera_euler_deg"), "ortho_scale": rj.get("ortho_scale"),
                                 "px_per_bu": rj.get("px_per_bu"), "sun_euler_deg": rj.get("sun_euler_deg"),
                                 "generator": (rj.get("generator") or {}).get("script"), "variant_rng": (rj.get("generator") or {}).get("variant_rng"),
                                 "base_square_sha256": (rj.get("base_square") or {}).get("sha256"),
                                 "source_square_sha256": (rj.get("source_square") or {}).get("sha256"),
                                 "brushwork": {k: bw.get(k) for k in ("model", "lora", "lora_strength", "seed", "denoise", "method", "graph", "graph_sha256")},
                                 **({"retone": {k: (rj.get("retone") or {}).get(k) for k in ("lut_sha256", "script", "script_sha256", "params", "params_sha256", "parent_diamond_sha256")}} if rj.get("retone") else {}),
                                 "summary": qgate["summary"]},
                    "andon": {"version": andon.get("andon_version") if andon else None,
                              "kind": andon.get("kind") if andon else None, "pass": andon.get("pass") if andon else None},
                    "gate": {"art_director": qgate["art_director"],
                             "verifier": ("grok-bot-verifier/PB-002/quay_flag_painterly_results.json; outbox-PB-002/retrieval-quay_flag_painterly.md 3/3 PASS" if qj else None)},
                    "render_sidecar": srel(base + ".render.json"),
                    "source_path": srel(src)}})
        qorder = {e: i for i, e in enumerate(EXP_QUAY_FLAG)}
        qf_e.sort(key=lambda e: qorder[e["id"]])
        entries += qf_e; counts["quay_flag"] = len(qf_e)

    # ================= extra-file audit =================
    allowed = {os.path.join(OUT, e["path"]) for e in entries} | {os.path.join(OUT, "MANIFEST.json")}
    for dp, _, fns in os.walk(OUT):
        for fn in fns:
            fp = os.path.join(dp, fn)
            if fp not in allowed:
                problems.append(f"extra file in bundle: {os.path.relpath(fp, OUT)}")
    if len({e["id"] for e in entries}) != len(entries):
        problems.append("duplicate ids")

    imp_summary = {}
    for e in entries:
        k = f"mipmaps={'on' if e['import']['mipmaps'] else 'off'} fix_alpha_border={'on' if e['import']['fix_alpha_border'] else 'off'} source={'import file' if e['import']['source'] != R11_SRC else R11_SRC}"
        imp_summary[k] = imp_summary.get(k, 0) + 1

    man = {
        "manifest": "portlight-bounty PB-002 landing bundle",
        "consumer": "portlight-bounty",
        "task": "PB-002",
        "version": a.version,
        "generator": "pb-002-plates/tools/build_manifest_pb002.py",
        "path_convention": {"entry.path": "relative to the bundle root (this MANIFEST's directory)",
                            "provenance.*": "relative to the studio workspace root"},
        "sources": {"asset_list": "outbox-PB-002/asset-list.csv", "asset_spec": "outbox-PB-002/asset-spec.md (Rev 4)",
                    "art_gate": "outbox-PB-002/art-gate.md Addenda D, E, E.1, F" + (", U.4, V.2" + ("" if qgate["addendum"] == "V.2" else ", " + qgate["addendum"]) if "quay_flag" in groups else ""),
                    "verifier": [f"outbox-PB-002/retrieval-{g}.md" for g in ("ships", "harbour", "chart", "quay_flag_painterly") if g.split("_painterly")[0] in groups]},
        "layout": {"ships": "chart/ships/ship_<class>/ship_<class>_{f0..f7,wake}.png (asset-spec §7g)",
                   "harbour": "ground/water_*.png, structures/<quay|pier id>/beauty.png, props/pier_pilings_1x1/beauty.png (asset-spec §4 Output)",
                   "chart": "chart/<id>.png (asset-spec §4 Output)",
                   **({"quay_flag": "ground/quay_flag_{a,b,c}.png (art-gate Addendum " + qgate["addendum"] + ")"} if "quay_flag" in groups else {})},
        "import_policy": {"rule": "asset-spec Rev 4 R11: Lossless, fix_alpha_border on, premult off, VRAM off; mipmaps ON for chart_water_a..c only",
                          "summary": imp_summary},
        "px_per_bu": {"chart": 90.51, "harbour": 181.019, **({"quay_flag": 181.019} if "quay_flag" in groups else {})},
        "counts": counts,
        "count": len(entries),
        "entries": entries,
    }
    text = json.dumps(man, indent=2) + "\n"
    bad = [s for s in walk_strings(man) if s.startswith("/") or "/workspace" in s]
    if bad:
        problems.append(f"absolute/workspace strings in output ({len(bad)}): {bad[:5]}")

    if problems:
        print("PROBLEMS:"); [print("  -", x) for x in problems]
    if nulls:
        print("NULL fields (missing in csv/spec):"); [print("  -", x) for x in nulls]
    if na:
        print(f"n/a by spec (csv '-'): {len(na)} fields"); [print("  -", x) for x in na]
    print("import settings:", json.dumps(imp_summary))
    out = os.path.join(OUT, "MANIFEST.json")
    if a.check:
        cur = open(out, encoding="utf-8").read() if os.path.isfile(out) else ""
        same = cur == text
        print(f"check [{os.path.relpath(OUT, PLATES)}]: MANIFEST.json {'MATCHES' if same else 'DIFFERS FROM'} regenerated output; "
              f"entries={len(entries)} {counts}, problems={len(problems)}")
        sys.exit(0 if same and not problems else 1)
    if problems:
        sys.exit("refusing to write MANIFEST.json")
    with open(out, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(text)
    print(f"wrote {os.path.relpath(out, PLATES)}: version {a.version}, {len(entries)} entries {counts}, nulls={len(nulls)}, n/a={len(na)}")


if __name__ == "__main__":
    main()
