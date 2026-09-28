# portlight-bounty

Rust port of [Portlight](https://github.com/mcp-tool-shop-org/portlight), a trade-first maritime strategy game. Stage 1 is the rules and simulation, checked against the Python game at commit `9b02494`. Stage 2 is the first playable dimetric chart: Mediterranean, four ports, sail from Porto Novo to Al-Manar, dock, and trade.

The simulation crate does not depend on a UI. The Godot project is a view over `Session`. The map of what is ported, and what is not, is [docs/PORTING-PLAN.md](docs/PORTING-PLAN.md).

## Layout

- `crates/portlight-sim` — goods, ports, prices, trade, voyages, victory paths, and `Session` (the turn-by-turn API)
- `crates/portlight-cli` — `portlight` binary
- `crates/portlight-chart` — dimetric projection, ship facing, and the chart view-model (no Godot)
- `crates/portlight-godot` — Godot 4.7 gdext extension
- `godot/` — Godot 4.7.2 project, placeholder chart water, and the approved landing bundle
- `parity/` — action scripts and golden snapshots from the Python engine
- `tools/parity/` — oracle and diff harness

## Run the sim

```
cargo run -p portlight-cli -- new --captain merchant --name Ada --seed 42
cargo run -p portlight-cli -- script parity/scripts/voyage.txt
cargo test --locked --workspace --exclude portlight-godot
```

Script commands: `new`, `buy`, `sell`, `depart`, `advance`, `hire`, `provision`, `work`, `duel`, `resolve_duel`.

`godot` 0.5.5 needs Rust 1.94 or newer. The sim pin in `rust-toolchain.toml` stays 1.83.0, and the sim CI job excludes `portlight-godot`. The Godot CI job installs stable, then sets `RUSTUP_TOOLCHAIN=stable` for the build. That variable overrides `rust-toolchain.toml`; without it Cargo 1.83 parses `godot-bindings` and the job fails before Godot starts. The job still downloads Godot 4.7.2, runs `cargo test -p portlight-godot`, and runs the headless import and `--smoke`. Build the extension locally with `cargo +stable build -p portlight-godot`.

## Run the chart

Godot 4.7.2, official build. The extension looks for `target/debug/libportlight_godot.so` (and the release and other-platform names in `godot/portlight.gdextension`).

```
cargo +stable build -p portlight-godot
godot --path godot
```

New game starts merchant Ada at Porto Novo, seed 1. The chart draws that port's `sail_lanes`, including warning and blocked lanes. Sail, Next day, and the market buttons call `Session`. They do not compute prices, days, or whether a lane is legal. While docked, Market, Hire sailor, Provisions +5, and Work sit on one row. Work calls `Session::work` (3 to 5 silver; markets, provisions, wages, and reputation do not tick). A failure shows `SimError` text. A pending duel offers a stance fight (`thrust`, `slash`, `parry`) or auto-resolve. Next day still calls `Session::advance`, and the sim checks the stances. The outcome's standing change is shown and not applied. Negotiate, flee, naval rounds, and boarding are not offered. The lane list sits in a scroll pane beside the chart so the Sail buttons stay inside the 1280×720 window.

Chart water and the port marker are the approved landing plates. The generator still writes the catalog and any future placeholder:

```
cargo run -p portlight-chart --bin gen_placeholders -- godot/assets
```

That command does not rewrite `godot/assets/landing/`. That directory is the whole v0.3.0 bundle: 70 PNGs (sloop, cutter, brigantine, and galleon, nine plates each, 30 harbour tiles, and chart water plus the port marker) and `MANIFEST.json`. `cargo test -p portlight-chart` checks the manifest hash, every entry sha256, that the committed PNG set matches the manifest, and that every plate `.import` follows that entry's import fields (asset-spec Rev 4 R11: lossless, Fix Alpha Border on, premultiply off, mipmaps on for chart water only). Harbour ids, including the −48 px sea datum, are listed in `godot/assets/catalog/locked-ids.csv`. That folder is `.gdignore`d so Godot does not import the catalog as a translation. The first playable does not open a harbour scene. `godot/scenes/harbour_seam.tscn` is a separate seam plate.

`godot/project.godot` sets `rendering/viewport/hdr_2d=false` (the Godot default, written explicitly). 2D blending stays sRGB. Sprite `.import` files follow Rev 4 R11: lossless, Fix Alpha Border on, premultiplied alpha off, VRAM off. Mipmaps are on for `chart_water_a`–`c` only. The canvas filter is Nearest (`default_texture_filter=0`), so a plate drawn at 1:1 keeps its pixels.

Headless smoke (session only; the dummy renderer does not return a viewport image):

```
godot --headless --path godot --import --quit
godot --headless --path godot -- --smoke
```

A PNG needs a GL context. On Linux, llvmpipe under Xvfb works. That smoke exits non-zero if the 1280×720 frame is missing or mostly one flat colour (the collapsed chart was a 28 px strip over the clear colour):

```
xvfb-run -a -s "-screen 0 1280x720x24" \
  env PORTLIGHT_SMOKE=1 PORTLIGHT_REQUIRE_FRAME=1 \
  PORTLIGHT_SHOT=/tmp/portlight-first-playable.png \
  godot --display-driver x11 --rendering-driver opengl3 --path godot -- --smoke
```

The art-director frame (docked sloop, sailing sloop at f7, wake) and the harbour seam plate:

```
xvfb-run -a -s "-screen 0 1280x720x24" \
  env PORTLIGHT_SHOT=/tmp/portlight-art-sloop.png \
  godot --display-driver x11 --rendering-driver opengl3 --path godot -- --art

xvfb-run -a -s "-screen 0 1280x720x24" \
  env PORTLIGHT_SHOT=/tmp/portlight-harbour-seam.png \
  godot --display-driver x11 --rendering-driver opengl3 --path godot \
  res://scenes/harbour_seam.tscn
```

## Parity

`cargo test` compares those scripts to `parity/golden/`. To regenerate the goldens from the Python checkout:

```
PYTHONPATH=/path/to/portlight/src python3 tools/extract_content.py
PYTHONPATH=/path/to/portlight/src python3 tools/parity/check.py --write-golden
```

The RNG is CPython's MT19937, so the same seed and script produce the same state for the systems that are ported. Full `GameSession` parity is still open; the plan lists the gaps.
