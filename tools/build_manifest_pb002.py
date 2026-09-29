#!/usr/bin/env python3
"""build_manifest_pb002.py - derive MANIFEST.json for the portlight-bounty PB-002 landing bundle.

    python3 tools/build_manifest_pb002.py --out-dir landing-next --version 0.3.0 [--check]
    (--out-dir is relative to pb-002-plates/ unless absolute; default "landing")

Never hand-edit MANIFEST.json; rerun this. Everything is derived from files + outbox docs:

  ships (36, P1)     ids/anchor/LOA/mesh      <- asset-list.csv; canvas <- PNG; sha256 computed
                     gates: art-gate.md Addendum D, retrieval-ships.md 36 PASS/0 FAIL; renderer <- ships/<class>/sidecar.json
                     sha vs source and vs grok-bot-verifier/PB-002/ships_results.json
  harbour (30, P0)   ids/canvas/anchor/footprint/layer/offsets/andon_kind <- asset-list.csv
                     gates: Addendum E + E.1, retrieval-harbour.md 30 PASS/0 FAIL (v0.1.1); renderer <- <id>.render.json
                     sha vs source and vs harbour_andon_results.json; <id>.andon.json must be v0.1.1 pass
  chart (4, P0/P1)   ids/canvas/anchor/footprint/layer/offsets/andon_kind <- asset-list.csv
                     gates: Addendum F (PASS), retrieval-chart.md 4 PASS/0 FAIL + "Builder may land";
                     renderer <- <id>.render.json; andon gate of record <- <id>.andon.json
                     sha vs source and vs grok-bot-verifier/PB-002/chart/chart_retrieval_results.json
  quay_flag (3, P0)  ids quay_flag_a..c (procedural quay stone, art-gate Addendum U.3 PASS)
                     canvas/anchor/footprint/layer/offsets/andon_kind <- asset-list.csv; source ground/<id>.png;
                     <id>.andon.json must be v0.1.1 pass (kind ground); renderer <- <id>.render.json (procedural, no Blender);
                     Verifier inventory grok-bot-verifier/PB-002/quay_flag_results.json checked when present
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

STUDIO = "/workspace/studio"          # read-side only; never emitted
PLATES = os.path.join(STUDIO, "pb-002-plates")
OUTBOX = os.path.join(STUDIO, "outbox-PB-002")
VERIFIER = os.path.join(STUDIO, "grok-bot-verifier", "PB-002")
CSV = os.path.join(OUTBOX, "asset-list.csv")
SPEC = os.path.join(OUTBOX, "asset-spec.md")
GATE = os.path.join(OUTBOX, "art-gate.md")
RETR_SHIPS = os.path.join(OUTBOX, "retrieval-ships.md")
RETR_HARB = os.path.join(OUTBOX, "retrieval-harbour.md")
RETR_CHART = os.path.join(OUTBOX, "retrieval-chart.md")
VJ_SHIPS = os.path.join(VERIFIER, "ships_results.json")
VJ_HARB = os.path.join(VERIFIER, "harbour_andon_results.json")
VJ_CHART = os.path.join(VERIFIER, "chart", "chart_retrieval_results.json")
SHIP_SRC = os.path.join(PLATES, "ships")
HARB_SRC_DIRS = ["ground", "quay", "pier", "props"]
CHART_SRC = os.path.join(PLATES, "chart")
CLASSES = ["sloop", "cutter", "brigantine", "galleon"]
FRAMES = [f"f{k}" for k in range(8)] + ["wake"]
EXP_HARB = (["water_a", "water_b", "water_c"] + [f"quay_{i:04b}" for i in range(1, 16)]
            + ["pier_UL_DR", "pier_UR_DL", "pier_UL", "pier_UR", "pier_DR", "pier_DL",
               "pier_UL_UR_DR", "pier_UR_DR_DL", "pier_DR_DL_UL", "pier_DL_UL_UR", "pier_UL_UR_DR_DL",
               "pier_pilings_1x1"])
EXP_CHART = ["chart_water_a", "chart_water_b", "chart_water_c", "chart_port_marker"]
EXP_QUAY_FLAG = ["quay_flag_a", "quay_flag_b", "quay_flag_c"]
QUAY_FLAG_SRC = os.path.join(PLATES, "ground")
VJ_QUAY_FLAG = os.path.join(VERIFIER, "quay_flag_results.json")
R11_MIP_ON = {"chart_water_a", "chart_water_b", "chart_water_c"}
R11_SRC = "asset-spec Rev 4 R11"


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
def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--version", default="0.1.0")
    ap.add_argument("--out-dir", default="landing", help="bundle dir (relative to pb-002-plates/ unless absolute)")
    ap.add_argument("--groups", default="ships,harbour,chart,quay_flag")
    ap.add_argument("--check", action="store_true")
    a = ap.parse_args()
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

    # ================= ships =================
    if "ships" in groups:
        retr = open(RETR_SHIPS, encoding="utf-8").read()
        m = re.search(r"\*\*Result:\s*(\d+)\s+PASS,\s*(\d+)\s+FAIL\*\*", retr)
        n_rows = len(re.findall(r"\|\s*\*\*PASS\*\*\s*\|", retr))
        if not m or m.groups() != ("36", "0") or n_rows != 36:
            problems.append(f"retrieval-ships.md not 36 PASS/0 FAIL (summary={m.groups() if m else None}, rows={n_rows})")
        if not re.search(r"## Addendum D .*?\*\*Result: PASS\. All 36 ids are will-use", gate, re.S):
            problems.append("art-gate.md Addendum D PASS not found")
        vj = load_json(VJ_SHIPS)
        vinv = {r["id"]: r["sha256"] for r in vj["inventory"]} if vj else {}
        if not vj:
            problems.append("verifier ships_results.json missing")
        spec_mesh_prefix = dict(re.findall(r"\|\s*`(ship_[a-z_]+)`\s*\|.*?\|\s*`([0-9a-f]{12})`\s*\|", spec))
        if len(spec_mesh_prefix) != 4:
            problems.append(f"asset-spec §7a mesh sha prefixes not found (got {spec_mesh_prefix})")
        rows = [r for r in rows_all if r["id"].startswith("ship_") and r["status"].startswith("LOCKED")]
        expected = [f"ship_{c}_{f}" for c in CLASSES for f in FRAMES]
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
            if vinv and vinv.get(aid) != digest: problems.append(f"{aid}: sha256 differs from Verifier inventory")
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
        mh = re.search(r"\*\*ANDON v0\.1\.1:\s*(\d+)\s+PASS,\s*(\d+)\s+FAIL\.\*\*", rh)
        n_rows = len(re.findall(r"\|\s*\*\*PASS\*\* \(exit 0\)\s*\|", rh))
        if not mh or mh.groups() != ("30", "0") or n_rows != 30:
            problems.append(f"retrieval-harbour.md not 30 PASS/0 FAIL (summary={mh.groups() if mh else None}, rows={n_rows})")
        if not re.search(r"## Addendum E .*?\*\*Result: PASS\. All 30 are will-use", gate, re.S):
            problems.append("art-gate.md Addendum E PASS not found")
        if not re.search(r"AMENDED 05:25 ET \(Addendum E\.1\)\.\*\*.*?still \*\*Pass\*\*", gate, re.S):
            problems.append("art-gate.md Addendum E.1 Pass not found")
        hj = load_json(VJ_HARB)
        hinv = {r["id"]: r["sha256"] for r in hj["inventory"]} if hj else {}
        hpass = {r["id"]: r.get("pass") for r in hj.get("rows", [])} if hj else {}
        if not hj: problems.append("verifier harbour_andon_results.json missing")
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
            if hinv and hinv.get(aid) != digest: problems.append(f"{aid}: sha256 differs from Verifier harbour inventory")
            if hpass and hpass.get(aid) is not True: problems.append(f"{aid}: Verifier harbour row not pass")
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
        mc = re.search(r"\*\*Chart profile \(SIMULATED\):\s*(\d+)\s+PASS,\s*(\d+)\s+FAIL\.\*\*", rc)
        if not mc or mc.groups() != ("4", "0"):
            problems.append(f"retrieval-chart.md not 4 PASS/0 FAIL (got {mc.groups() if mc else None})")
        if not re.search(r"✅ Builder may land\s*$", rc):
            problems.append("retrieval-chart.md does not end with '✅ Builder may land'")
        gate_f = re.search(r"## Addendum F \(([^)]*)\).*?\*\*Chart set: PASS\. All 4 are will-use\*\*", gate, re.S)
        if not gate_f:
            problems.append("art-gate.md Addendum F chart PASS not found")
        cj = load_json(VJ_CHART)
        if not cj: problems.append("verifier chart_retrieval_results.json missing")
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
            if cj:
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

    # ================= quay_flag (procedural quay stone ground, Addendum U.3) =================
    if "quay_flag" in groups:
        gate_u3 = re.search(r"## Addendum U\.3 [^\n]*\n.*?Verdict: PASS for the procedural quay stone", gate, re.S)
        if not gate_u3:
            problems.append("art-gate.md Addendum U.3 PASS not found")
        qj = load_json(VJ_QUAY_FLAG)
        qinv = {r["id"]: r["sha256"] for r in qj["inventory"]} if qj else {}
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
            if qinv and qinv.get(aid) != digest: problems.append(f"{aid}: sha256 differs from Verifier quay_flag inventory")
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
                    "renderer": {"name": "procedural", "version": None, "engine": rj.get("engine"), "samples": None,
                                 "supersample": rj.get("supersample"), "scene_kind": rj.get("kind"),
                                 "camera_euler_deg": rj.get("camera_euler_deg"), "ortho_scale": rj.get("ortho_scale"),
                                 "px_per_bu": rj.get("px_per_bu"), "sun_euler_deg": rj.get("sun_euler_deg"),
                                 "generator": (rj.get("generator") or {}).get("script"), "variant_rng": (rj.get("generator") or {}).get("variant_rng"),
                                 "source_square_sha256": (rj.get("source_square") or {}).get("sha256"),
                                 "summary": "procedural numpy/PIL, 2:1 projection + diamond_clip --feather 8"},
                    "andon": {"version": andon.get("andon_version") if andon else None,
                              "kind": andon.get("kind") if andon else None, "pass": andon.get("pass") if andon else None},
                    "gate": {"art_director": "art-gate.md Addendum U.3 (2026-09-29): PASS, procedural quay stone variants a/b/c",
                             "verifier": ("grok-bot-verifier/PB-002/quay_flag_results.json" if qj else None)},
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
                    "art_gate": "outbox-PB-002/art-gate.md Addenda D, E, E.1, F" + (", U.3" if "quay_flag" in groups else ""),
                    "verifier": [f"outbox-PB-002/retrieval-{g}.md" for g in ("ships", "harbour", "chart") if g in groups]},
        "layout": {"ships": "chart/ships/ship_<class>/ship_<class>_{f0..f7,wake}.png (asset-spec §7g)",
                   "harbour": "ground/water_*.png, structures/<quay|pier id>/beauty.png, props/pier_pilings_1x1/beauty.png (asset-spec §4 Output)",
                   "chart": "chart/<id>.png (asset-spec §4 Output)",
                   **({"quay_flag": "ground/quay_flag_{a,b,c}.png (art-gate Addendum U.3)"} if "quay_flag" in groups else {})},
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
