---
pretty_name: Portlight Bounty Landing Art Dataset
license: cc-by-4.0
language:
- en
task_categories:
- image-to-text
- text-to-image
tags:
- game-assets
- pixel-scale
- isometric
- dimetric
- sprites
- ships
- harbour
- blender
size_categories:
- n<1K
configs:
- config_name: default
  data_files:
  - split: train
    path:
    - "ships/*.png"
    - "harbour/*.png"
    - "chart/*.png"
    - "quay_flag/*.png"
    - "harbour_props/*.png"
    - metadata.jsonl
---

# Portlight Bounty Landing Art Dataset (`mcp-tool-shop/portlight-bounty-landing-art`)

The complete landing art bundle (**78 RGBA `.png` sprites and tiles**, manifest `0.3.0`) from [Portlight Bounty](https://github.com/mcp-tool-shop-org/portlight-bounty), paired with per-file build provenance and derived captions in `metadata.jsonl`.

The viewer configuration in the YAML front matter follows the Hugging Face `ImageFolder` + `metadata.jsonl` specification (`https://huggingface.co/docs/datasets/image_dataset#imagefolder` and `https://huggingface.co/docs/hub/datasets-data-files-configuration`).

## What the set is

All 78 entries are defined in `godot/assets/landing/MANIFEST.json:1-5466` (`"version": "0.3.0"`, `"count": 78` at `godot/assets/landing/MANIFEST.json:4,50`). Every tile and sprite uses a **60° / 0° / 45° dimetric orthographic camera** (`"camera_euler_deg": [60.0, 0.0, 45.0]`, `"ortho_scale": 1.41421`, e.g. `godot/assets/landing/MANIFEST.json:2118-2124,4510-4516,4850-4856,5137-5143`), with pixel density `90.51 px/BU` in the chart view and `181.019 px/BU` in the harbour view (`godot/assets/landing/MANIFEST.json:37-42`).

| Group | Count | View | Canvas sizes (`px`) | Contents | `MANIFEST.json` lines |
| --- | ---: | --- | --- | --- | --- |
| `ships` | 36 | `chart` | `64×64` (`ship_sloop`), `72×80` (`ship_cutter`), `96×96` (`ship_brigantine`), `112×112` (`ship_galleon`) | 4 hull classes × (8 headings `f0..f7` + `wake` frame) | `godot/assets/landing/MANIFEST.json:44,52-2067` |
| `harbour` | 30 | `harbour` | `256×128` (`water_a..c`), `256×256` (`quay_0001..1111`, `pier_*`, `pier_pilings_1x1`) | 3 water ground tiles, 15 bitmask quay structure plates, 11 pier structure plates, 1 pier pilings plate | `godot/assets/landing/MANIFEST.json:45,2068-4458` |
| `chart` | 4 | `chart` | `128×64` (`chart_water_a..c`), `128×128` (`chart_port_marker`) | 3 chart sea tiles and 1 port marker sprite | `godot/assets/landing/MANIFEST.json:46,4459-4798` |
| `quay_flag` | 3 | `harbour` | `256×128` (`quay_flag_a..c`) | 3 painterly flagstone ground tiles | `godot/assets/landing/MANIFEST.json:47,4799-5086` |
| `harbour_props` | 5 | `harbour` | `256×256` (`bollard_1x1`, `torch_1x1`, `barrel_1x1`, `crate_1x1`, `cart_1x1`) | 5 harbour deck/quay prop sprites | `godot/assets/landing/MANIFEST.json:48,5087-5464` |

## How each group was made (provenance)

Every row in `metadata.jsonl` carries the entry's full `provenance` object from `godot/assets/landing/MANIFEST.json`:

- **`ships` (36 sprites, `godot/assets/landing/MANIFEST.json:52-2067`):** Rendered in **Blender 5.2.2 LTS (`BLENDER_EEVEE`, 64 samples)** (`godot/assets/landing/MANIFEST.json:92-98`) from the normalised `.glb` hull meshes (`sloop__01-pristine__sails-open.glb`, `cutter__01-pristine__sails-open.glb`, `brigantine__01-pristine__sails-open.glb`, `galleon__01-pristine__sails-open.glb`; `godot/assets/landing/MANIFEST.json:86-91,588-593,1090-1095,1592-1597`). Those `portlight-ships` meshes were made from **Nano Banana 2** (`Gemini 3.1 Flash Image`) plates and **Tripo v3.1** (`v3.1-20260211`) image-to-3D conversion through the studio's paid Comfy plan; see the companion dataset [`mcp-tool-shop/portlight-ships-anatomy`](https://huggingface.co/datasets/mcp-tool-shop/portlight-ships-anatomy). Each class renders 8 compass headings (`f0..f7`, `rot_z` `90°`, `45°`, `0°`, `-45°`, `-90°`, `-135°`, `180°`, `135°`) plus a rig-only disc `wake` frame (`"note": "wake: rig only, disc (no mesh in frame)"`, e.g. `godot/assets/landing/MANIFEST.json:524-530,2046-2051`).
- **`harbour` (30 plates, `godot/assets/landing/MANIFEST.json:2068-4458`) and `chart` (4 plates, `godot/assets/landing/MANIFEST.json:4459-4798`):** Direct **Blender 5.2.2 LTS (`BLENDER_EEVEE`, 64 samples)** scene renders (`scene_kind` values `water`, `quay`, `pier`, `pilings` in `harbour` at `181.019 px/BU`, and `chart_water`, `chart_marker` in `chart` at `90.51 px/BU`; `godot/assets/landing/MANIFEST.json:2112-2131,2346-2365,3546-3565,4426-4445,4504-4527,4759-4782`).
- **`quay_flag` (3 tiles, `godot/assets/landing/MANIFEST.json:4799-5086`):** Built from a deterministic procedural `v5` square base (`numpy/PIL`), painted via local ComfyUI img2img brushwork (`qwen_image_fp8_e4m3fn.safetensors` / Qwen-Image-2512 weight with the studio's Salt Road LoRA `mcp-tool-shop__saltroad-style-lora__saltroad_style_v2_lowlr_000001500.safetensors` at strength `0.75`, seed `1500`, denoise `0.15` on `quay_flag_a` and `0.2` on `quay_flag_b`/`c` with deterministic border paste), projected 2:1 with `diamond_clip --feather 8`, and finished with a fixed pointwise tonal map (`plates/retone_w2/retone_w2.py`, LUT `559ddbe3`, no generation) (`godot/assets/landing/MANIFEST.json:4843-4881,4939-4977,5035-5073`; `site/src/content/docs/handbook/art.md:16`).
- **`harbour_props` (5 sprites, `godot/assets/landing/MANIFEST.json:5087-5464`):** Built from Blender primitives by `build_prop()` (`assets/dimetric/camera/dimetric_60_45.py`) in [`mcp-tool-shop-org/ai-rpg-stage`](https://github.com/mcp-tool-shop-org/ai-rpg-stage) at commit `55ac9b2` (MIT) and rendered in **Blender 5.2.0 LTS (`BLENDER_EEVEE`)** (`godot/assets/landing/MANIFEST.json:5131-5150,5209-5224,5283-5298,5357-5376,5435-5450`; `site/src/content/docs/handbook/art.md:14`).

## File layout and verification

In the staged dataset repository, files are organised under `<group>/<basename>`, where the 32 plates stored as `<dir>/<id>/beauty.png` in the game bundle (`harbour` structures/pilings and `harbour_props`) use `<id>.png` as `<basename>` so every entry has a unique `<group>/<id>.png` path:

```text
README.md
metadata.jsonl
ships/ship_<class>_<f0..f7|wake>.png     (36 files)
harbour/water_<a..c>.png                 (3 files)
harbour/quay_<0001..1111>.png            (15 files)
harbour/pier_<variant>.png               (11 files)
harbour/pier_pilings_1x1.png             (1 file)
chart/chart_water_<a..c>.png             (3 files)
chart/chart_port_marker.png              (1 file)
quay_flag/quay_flag_<a..c>.png           (3 files)
harbour_props/<prop_id>.png              (5 files)
```

Each line in `metadata.jsonl` records:
- `file_name`: `<group>/<id>.png`
- `id`, `group`, `class`, `frame`, `view`, `canvas`, `anchor`
- `sha256`: SHA-256 hex digest of the PNG bytes, verified against `godot/assets/landing/MANIFEST.json`
- `text`: deterministic caption derived strictly from manifest fields:
  - `ships` (`f0..f7`): `"{class} ({group}, {view} view, frame {frame}, heading rot_z {rot_z} deg)"`
  - `ships` (`wake`): `"{class} ({group}, {view} view, frame {frame})"`
  - `harbour`, `chart`, `quay_flag`, `harbour_props`: `"{id} ({group}/{subgroup}, {view} view)"`
- `provenance`: the entry's full `provenance` JSON object from `MANIFEST.json`

To regenerate `metadata.jsonl` and stage a verified copy of all 78 PNGs + `metadata.jsonl` + `README.md` into an empty directory:

```bash
python tools/build_hf_art_dataset.py --stage <empty-dir>
```

## Licence

The pictures are made by mcp-tool-shop and licensed under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/legalcode). In short, and the license's legal code governs: anyone may share and adapt them for any purpose, including commercially, with credit to `"mcp-tool-shop"`. Everything else in the repository is MIT. Pictures published in v0.3.2 and earlier were released under MIT, and those pictures, including identical copies in later versions, stay available under MIT too. The license grants no trademark rights in the mcp-tool-shop name or logo (`site/src/content/docs/handbook/art.md:8`, `LICENSE:28-40`).
