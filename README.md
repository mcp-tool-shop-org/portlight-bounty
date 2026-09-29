# portlight-bounty

Rust port of [Portlight](https://github.com/mcp-tool-shop-org/portlight), a trade-first maritime strategy game. Stage 1 is the rules and simulation, checked against the Python game at commit `9b02494`. Stage 2 is the first playable dimetric chart: Mediterranean, four ports, sail from Porto Novo to Al-Manar, dock, and trade.

The simulation crate does not depend on a UI. The Godot project is a view over `Session`. The map of what is ported, and what is not, is [docs/PORTING-PLAN.md](docs/PORTING-PLAN.md).

## Layout

- `crates/portlight-sim` — goods, ports, prices, trade, voyages, victory paths, and `Session` (the turn-by-turn API). Save v12, contracts, encounters, skills and career, infrastructure, credit and insurance, fleet, injuries, weapons and loot, narrative, culture and consequences, the area 7a ship, crew, and contract commands, and hunting and bounty are merged in the sim and are not yet offered by the Godot view.
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

Script commands, in `script.rs` order: `new`, `buy`, `sell`, `depart`, `advance`, `arrival_narrative`, `evaluate_consequences`, `accept_contract`, `complete_contract`, `buy_infrastructure`, `take_credit`, `buy_insurance`, `deposit`, `withdraw`, `repay_credit`, `hire`, `provision`, `work`, `duel`, `resolve_duel`, `encounter`, `naval`, `board`, `fight`, `capture`, `train`, `recruit`, `skill`, `remember`, `agency`, `spare`, `take_all`, `gear`, `buy_ship`, `upgrade`, `form_convoy`, `repair_fleet`, `repair`, `rename_ship`, `dock_current_ship`, `board_fleet_ship`, `sell_fleet_ship`, `fire`, `abandon_contract`, `transfer`, `maintain`, `hunt`, `bounty`, `wanted`.

`godot` 0.5.5 needs Rust 1.94 or newer. The sim pin in `rust-toolchain.toml` is 1.98.1, and the sim CI job excludes `portlight-godot`. The Godot CI job installs stable, then sets `RUSTUP_TOOLCHAIN=stable` for the build. That variable overrides `rust-toolchain.toml`. The job still downloads Godot 4.7.2, runs `cargo test -p portlight-godot`, and runs the headless import and `--smoke`. Build the extension locally with `cargo build -p portlight-godot`.

## Run the chart

Godot 4.7.2, official build. The extension looks for `target/debug/libportlight_godot.so` (and the release and other-platform names in `godot/portlight.gdextension`).

```
cargo build -p portlight-godot
godot --path godot
```

New game starts merchant Ada at Porto Novo, seed 1. The chart draws that port's `sail_lanes`, including warning and blocked lanes. Sail, Next day, and the market buttons call `Session`. They do not compute prices, days, or whether a lane is legal. While docked, Market, Hire sailor, Provisions +5, and Work sit on one row. Work calls `Session::work` (3 to 5 silver; markets, provisions, wages, and reputation do not tick). A failure shows `SimError` text. A pending duel offers a stance fight (`thrust`, `slash`, `parry`) or auto-resolve. Next day still calls `Session::advance`, and the sim checks the stances. The outcome's standing change is shown and not applied. A sale logs the receipt and any contract summaries. `Session::board` is the contract board, and deck melee is `Session::resolve_boarding`. The view does not call `board`, `encounter_choice`, `naval_round`, or `resolve_boarding`. The lane list sits in a scroll pane beside the chart so the Sail buttons stay inside the 1280×720 window.

Chart water and the port marker are the approved landing plates. The generator still writes the catalog and any future placeholder:

```
cargo run -p portlight-chart --bin gen_placeholders -- godot/assets
```

That command does not rewrite `godot/assets/landing/`. That directory is the whole v0.3.0 bundle: 70 PNGs (sloop, cutter, brigantine, and galleon, nine plates each, 30 harbour tiles, and chart water plus the port marker) and `MANIFEST.json`. `cargo test -p portlight-chart` checks the manifest hash, every entry sha256, that the committed PNG set matches the manifest, and that every plate `.import` follows that entry's import fields (asset-spec Rev 4 R11: lossless, Fix Alpha Border on, premultiply off, mipmaps on for chart water only). Harbour ids, including the −48 px sea datum, are listed in `godot/assets/catalog/locked-ids.csv`. That folder is `.gdignore`d so Godot does not import the catalog as a translation. The first playable does not open a harbour scene. `godot/scenes/harbour_seam.tscn` is a separate seam plate.

`godot/project.godot` sets `rendering/viewport/hdr_2d=false` (the Godot default, written explicitly). 2D blending stays sRGB. Sprite `.import` files follow Rev 4 R11: lossless, Fix Alpha Border on, premultiplied alpha off, VRAM off. Mipmaps are on for `chart_water_a`–`c` only. The canvas filter is Nearest (`default_texture_filter=0`), so a plate drawn at 1:1 keeps its pixels.

Headless Godot cannot produce a frame. Its dummy renderer has no viewport texture. Asking for one logs `Parameter "t" is null` and `viewport image was empty`, and that used to exit 0. The headless smoke checks the session only and does not read the viewport:

```
godot --headless --path godot --import --quit
godot --headless --path godot -- --smoke
```

Screenshots need a real GL context. On Linux that is llvmpipe under Xvfb. Setting `PORTLIGHT_SHOT` captures the window. The process exits non-zero if that image is empty or mostly one flat colour (the collapsed chart was a 28 px strip over the clear colour). CI greps the Godot log for those renderer errors.

```
xvfb-run -a -s "-screen 0 1280x720x24" \
  env PORTLIGHT_SMOKE=1 PORTLIGHT_SHOT=/tmp/portlight-first-playable.png \
  godot --display-driver x11 --rendering-driver opengl3 --path godot -- --smoke
```

The art-director frame (docked sloop at Porto Novo, Swift Cutter under sail at f7, `ship_cutter_wake`) writes `/tmp/chart-cutter-f7.png`. `--art-docs` or `PORTLIGHT_ART_DOCS` writes `docs/screenshots/chart-cutter-f7.png`. `PORTLIGHT_SHOT` always wins.

Encounter captures use the same `/tmp/<file>` default. They do not follow `--art-docs`. A docs frame is only an explicit path, for example `PORTLIGHT_SHOT=docs/screenshots/encounter-galleon.png`. `--encounter-screen` writes `encounter-approach.png`, `encounter-naval.png`, `encounter-boarding.png`, `encounter-fight.png`, and `encounter-outcome.png` under `/tmp`, or under `PORTLIGHT_ENCOUNTER_DIR` when that is set. Headless `--encounter-screen` does not read the viewport. `--encounter-galleon` writes `encounter-galleon.png` with `royal_man_of_war` in the player plate slot (caption `man_of_war`) and that template's hull and crew on the card. The encounter frame check follows the capture mode, not the filename, so `PORTLIGHT_SHOT=/tmp/galleon1.png` is still an encounter frame. The harbour seam plate:

```
xvfb-run -a -s "-screen 0 1280x720x24" \
  env PORTLIGHT_SHOT=docs/screenshots/chart-cutter-f7.png \
  godot --display-driver x11 --rendering-driver opengl3 --path godot -- --art

xvfb-run -a -s "-screen 0 1280x720x24" \
  env PORTLIGHT_SEAM_DIR=/tmp \
  godot --display-driver x11 --rendering-driver opengl3 --path godot \
  res://scenes/harbour_seam.tscn
```

The seam scene validates the layout before it writes. An illegal layout (pilings on a pier cell, or a pier on a quay cell) prints the fault, exits non-zero, and does not save a PNG. A blank or mostly flat frame fails the same check as `PORTLIGHT_SHOT` and also exits non-zero.

## Parity

`cargo test` compares every script in `parity/scripts/` to `parity/golden/`. The file list is:

- `abandon_contract.txt`
- `board_fleet_ship.txt`
- `boarding.txt`
- `bounty_board.txt`
- `bounty_claim.txt`
- `bounty_hunter_voyage.txt`
- `bounty_max.txt`
- `bounty_not_defeated.txt`
- `bounty_not_hunting.txt`
- `bounty_unknown.txt`
- `buy_broker.txt`
- `buy_insurance.txt`
- `buy_warehouse.txt`
- `cargo_loss.txt`
- `consequences.txt`
- `contraband_sell.txt`
- `contract_accept.txt`
- `contract_arrival_rng.txt`
- `contract_complete.txt`
- `contract_expire.txt`
- `crew_minimum.txt`
- `dock_current_ship.txt`
- `dock_work.txt`
- `dry_dock_named.txt`
- `duel_block.txt`
- `duel_draw.txt`
- `duel_invalid.txt`
- `duel_loss.txt`
- `duel_none.txt`
- `duel_resolve.txt`
- `duel_short.txt`
- `duel_win.txt`
- `encounter_flee.txt`
- `encounter_negotiate.txt`
- `event_ceremony.txt`
- `event_foreign.txt`
- `event_musician.txt`
- `event_whale.txt`
- `fire_crew.txt`
- `fleet_convoy.txt`
- `fleet_form.txt`
- `fleet_transfer.txt`
- `g9.txt`
- `gear_armor.txt`
- `hire_and_sail.txt`
- `hire_broke.txt`
- `hire_full.txt`
- `hire_navigator.txt`
- `hire_role.txt`
- `hire_sea.txt`
- `hull_day20.txt`
- `hunt_port_fail.txt`
- `hunt_port_success.txt`
- `hunt_sea_fail.txt`
- `hunt_sea_morale.txt`
- `hunt_sea_success.txt`
- `injury_heal.txt`
- `inspection_rep.txt`
- `maintain.txt`
- `maintain_blacksmith.txt`
- `milestone_reached.txt`
- `narrative_beats.txt`
- `naval_combat.txt`
- `new_game.txt`
- `port_arrival.txt`
- `port_days.txt`
- `provision_sea.txt`
- `provision_silva.txt`
- `provision_zero.txt`
- `recruit_companion.txt`
- `rename_ship.txt`
- `repair_fleet.txt`
- `repair_ship.txt`
- `sea_captain_agency.txt`
- `sea_culture.txt`
- `sell_fleet_ship.txt`
- `skill_spend.txt`
- `smuggler.txt`
- `take_credit.txt`
- `trade_in_port.txt`
- `train_crew.txt`
- `upgrade_naval.txt`
- `victory_provenance.txt`
- `victory_spare.txt`
- `victory_takeall.txt`
- `voyage.txt`
- `work_at_sea.txt`

To regenerate the goldens from the Python checkout:

```
PYTHONPATH=/path/to/portlight/src python3 tools/extract_content.py
PYTHONPATH=/path/to/portlight/src python3 tools/parity/check.py --write-golden
```

The RNG is CPython's MT19937, so the same seed and script produce the same state for the systems that are ported. Full `GameSession` parity is still open; the plan lists the gaps.
