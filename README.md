# portlight-bounty

Rust port of [Portlight](https://github.com/mcp-tool-shop-org/portlight), a trade-first maritime strategy game. Stage 1 is the rules and simulation, checked against the Python game at commit `9b02494`. Stage 2 is the first playable dimetric chart: Mediterranean, four ports, sail from Porto Novo to Al-Manar, dock, and trade.

The simulation crate does not depend on a UI. The Godot project is a view over `Session`. The map of what is ported, and what is not, is [docs/PORTING-PLAN.md](docs/PORTING-PLAN.md).

## Layout

- `crates/portlight-sim` — goods, ports, prices, trade, voyages, victory paths, and `Session` (the turn-by-turn API)
- `crates/portlight-cli` — `portlight` binary
- `crates/portlight-chart` — dimetric projection, ship facing, and the chart view-model (no Godot)
- `crates/portlight-godot` — Godot 4.7 gdext extension
- `godot/` — Godot 4.7.2 project and placeholder tiles
- `parity/` — action scripts and golden snapshots from the Python engine
- `tools/parity/` — oracle and diff harness

## Run the sim

```
cargo run -p portlight-cli -- new --captain merchant --name Ada --seed 42
cargo run -p portlight-cli -- script parity/scripts/voyage.txt
cargo test --locked --workspace
```

Script commands: `new`, `buy`, `sell`, `depart`, `advance`, `hire`, `provision`, `work`, `duel`, `resolve_duel`.

`godot` 0.5.5 needs a recent stable Rust (1.94 or newer). CI uses `dtolnay/rust-toolchain@stable`.

## Run the chart

Godot 4.7.2, official build. The extension looks for `target/debug/libportlight_godot.so` (and the release and other-platform names in `godot/portlight.gdextension`).

```
cargo build -p portlight-godot
godot --path godot
```

New game starts merchant Ada at Porto Novo, seed 1. The chart draws that port's `sail_lanes`, including warning and blocked lanes. Sail, Next day, and the market buttons call `Session`. They do not compute prices, days, or whether a lane is legal.

Placeholder tiles are generated, not drawn by hand:

```
cargo run -p portlight-chart --bin gen_placeholders -- godot/assets
```

Replace a file later by keeping its asset id in `godot/assets/catalog/asset-list.csv`. That folder is `.gdignore`d so Godot does not import the catalog as a translation. Sea, water, quay, and pier sit at the −48 px datum.

Headless smoke (session only; the dummy renderer does not return a viewport image):

```
godot --headless --path godot --import --quit
godot --headless --path godot -- --smoke
```

A PNG needs a GL context. On Linux, llvmpipe under Xvfb works:

```
xvfb-run -a -s "-screen 0 1280x720x24" \
  env PORTLIGHT_SMOKE=1 PORTLIGHT_SHOT=/tmp/portlight-first-playable.png \
  godot --display-driver x11 --rendering-driver opengl3 --path godot -- --smoke
```

## Parity

`cargo test` compares those scripts to `parity/golden/`. To regenerate the goldens from the Python checkout:

```
PYTHONPATH=/path/to/portlight/src python3 tools/extract_content.py
PYTHONPATH=/path/to/portlight/src python3 tools/parity/check.py --write-golden
```

The RNG is CPython's MT19937, so the same seed and script produce the same state for the systems that are ported. Full `GameSession` parity is still open; the plan lists the gaps.
