# portlight-bounty

Rust port of [Portlight](https://github.com/mcp-tool-shop-org/portlight), a trade-first maritime strategy game. Stage 1 is the rules and simulation, checked against the Python game at commit `9b02494`. Stage 2 is the first playable dimetric chart: Mediterranean, four ports, sail from Porto Novo to Al-Manar, dock, and trade.

The simulation crate does not depend on a UI. The Godot project is a view over `Session`. The map of what is ported, and what is not, is [docs/PORTING-PLAN.md](docs/PORTING-PLAN.md).

## Layout

- `crates/portlight-sim` — goods, ports, prices, trade, voyages, victory paths, and `Session` (the turn-by-turn API). Save v12, contracts, encounters, skills and career, fleet, injuries, weapons and loot, narrative, culture and consequences, the area 7a ship, crew, and contract commands, and hunting and bounty are merged in the sim and are not yet offered by the Godot view. The Harbour screen offers warehouse, broker, license, credit, and insurance.
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

Script commands, in `script.rs` order: `new`, `buy`, `sell`, `depart`, `advance`, `arrival_narrative`, `evaluate_consequences`, `accept_contract`, `complete_contract`, `buy_infrastructure`, `take_credit`, `buy_insurance`, `deposit`, `withdraw`, `repay_credit`, `save`, `load`, `hire`, `provision`, `work`, `duel`, `resolve_duel`, `encounter`, `naval`, `board`, `fight`, `capture`, `train`, `recruit`, `skill`, `remember`, `agency`, `spare`, `take_all`, `gear`, `buy_ship`, `upgrade`, `form_convoy`, `repair_fleet`, `repair`, `rename_ship`, `dock_current_ship`, `board_fleet_ship`, `sell_fleet_ship`, `fire`, `abandon_contract`, `transfer`, `maintain`, `hunt`, `bounty`, `wanted`.

`godot` 0.5.5 needs Rust 1.94 or newer. The sim pin in `rust-toolchain.toml` is 1.98.1, and the sim CI job excludes `portlight-godot`. The Godot CI job installs stable, then sets `RUSTUP_TOOLCHAIN=stable` for the build. That variable overrides `rust-toolchain.toml`. The job still downloads Godot 4.7.2, runs clippy and `cargo test -p portlight-godot`, runs the headless import and `--smoke`, and compares fresh Xvfb frames to `docs/screenshots`. Headless Godot cannot draw, so that comparison is not a headless capture. Build the extension locally with `cargo build -p portlight-godot`.

## Run the chart

Godot 4.7.2, official build. The extension looks for `target/debug/libportlight_godot.so` (and the release and other-platform names in `godot/portlight.gdextension`).

```
cargo build -p portlight-godot
godot --path godot
```

The title screen offers New game and Load game. New game lists the nine catalog captains and a custom captain (name, title, the four point pools, home region and port, trade bloc, pirate faction, mentor, and backstory). Catalog games and custom games both call `Session` with seed 1. Merchant named Ada is still the first chart: pick The Merchant and leave the name as Ada. Save sits with New game and Next day and writes a version-12 slot, then shows the slot line. Load lists those slots with the captain and the day. A loaded custom captain keeps the name, the type, the silver, and the day. Version 12 does not store the custom template, so prices and voyage modifiers after load are the merchant's, the same as Python. The chart draws the current port's `sail_lanes`, including warning and blocked lanes. Sail, Next day, and the market buttons call `Session`. They do not compute prices, days, or whether a lane is legal. While docked, Market, Contracts, Hire (one sailor), Stores +5, Work, Shipyard, Harbour, and Hunt sit on one row at 10 px, still 31 px tall, so the lanes below do not move. The row is hidden at sea. At sea, Hunt sits with Next day, Save, and Journal. Work calls `Session::work` (3 to 5 silver; markets, provisions, wages, and reputation do not tick). A failure shows `SimError` text. A pending duel offers a stance fight (`thrust`, `slash`, `parry`) or auto-resolve. Next day still calls `Session::advance`, and the sim checks the stances. The outcome's standing change is shown and not applied. A sale logs the receipt and any contract summaries. `Session::board` is the contract board, and deck melee is `Session::resolve_boarding`. The lane list sits in a scroll pane beside the chart so the Sail buttons stay inside the 1280×720 window. The chart panel does not call `board`, `encounter_choice`, `naval_round`, or `resolve_boarding`. The encounter screen does call `encounter_choice`, `naval_round`, `resolve_boarding`, `fight`, `spare`, and `capture` through `Session`.

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

`--encounter-screen` runs the boarding script and then the bounty script, and does not capture unless a directory is requested. Set `PORTLIGHT_ENCOUNTER_DIR` to write the five frames there, or `--art-docs` / `PORTLIGHT_ART_DOCS` to write them under `docs/screenshots`. Headless skips only the implicit `/tmp` default. `PORTLIGHT_SHOT`, `PORTLIGHT_ENCOUNTER_DIR`, `--art-docs`, and `PORTLIGHT_ART_DOCS` still try to save, and the empty viewport exits 1.

`--newgame-screen` starts a custom captain, saves a version-12 slot, lists it, and loads it. The reload check is the name, the type, the silver, and the day. It does not capture. The last line is `portlight newgame smoke ok`. `PORTLIGHT_SHOT` or `--art-docs` / `PORTLIGHT_ART_DOCS` writes `newgame-title.png`, `newgame-captains.png`, `newgame-custom.png`, `newgame-load.png`, and `newgame-save.png` (1280×800). `PORTLIGHT_SHOT` names the directory when it is a folder, or the parent of a `.png` path. `--art-docs` writes `docs/screenshots`. An explicit capture under headless exits 1 and prints `portlight newgame smoke FAILED`.

`--contracts-screen` opens the docked contract board, accepts one offer, tries to complete it before delivery, abandons it after confirm, and closes back to the chart. The last line is `portlight contracts smoke ok`. It does not capture unless `PORTLIGHT_SHOT` or `--art-docs` / `PORTLIGHT_ART_DOCS` is set. Those write `contracts-board.png`, `contracts-active.png`, and `contracts-empty.png` (1280×720) in the same directory rule as the new-game frames. An explicit capture under headless exits 1 and prints `portlight contracts smoke FAILED`. Contracts and Crew share the docked port row, so `chart-1280.png` and `newgame-save.png` include both.

`--shipyard-screen` opens the docked yard at Porto Novo. It repairs (a full hull reports the sim's perfect-condition sentence), renames the flagship and restores the old name, buys the Swift Cutter, installs Iron Strapping, docks, boards the cutter again, and sells the parked sloop. The day does not move. The last line is `portlight shipyard smoke ok`. `PORTLIGHT_SHOT` or `--art-docs` / `PORTLIGHT_ART_DOCS` writes `shipyard-flagship.png`, `shipyard-yard.png`, and `shipyard-fleet.png` (1280×720). The directory rule matches new game. An explicit capture under headless exits 1 and prints `portlight shipyard smoke FAILED`.

`--journal-screen` opens the read-only journal on a new voyage, checks the chronicle, the four victory paths, milestones, and memories, closes it, then leaves port and opens it again at sea. Opening does not advance the day. The last line is `portlight journal smoke ok`. `PORTLIGHT_SHOT` or `--art-docs` / `PORTLIGHT_ART_DOCS` writes `journal-chronicle.png`, `journal-victory.png`, and `journal-memories.png` (1280×720) into that directory. An explicit capture under headless exits 1 and prints `portlight journal smoke FAILED`. Journal sits with New game, Next day, and Save. It is not on the docked port row.

`--crew-screen` opens the docked crew desk, hires one sailor, fires that sailor, buys one day of provisions, and trains La Destreza and Apprentice blacksmithing only after the confirm line says the calendar will advance. Cancel leaves the day alone. Close returns to the chart. Chart Hire (one sailor) and Stores +5 still call `Session`. The last line is `portlight crew smoke ok`. `PORTLIGHT_SHOT` or `--art-docs` / `PORTLIGHT_ART_DOCS` writes `crew-roster.png`, `crew-provisions.png`, `crew-training.png`, and `crew-companions.png` (1280×720) into the shot directory. The provisions frame shows the effective per-day price. The training frame is the La Destreza confirm, and the skill button reads Learn. An explicit capture under headless exits 1 and prints `portlight crew smoke FAILED`. Contracts, Shipyard, Harbour, Crew, and Hunt share the docked port row, so `chart-1280.png` and `newgame-save.png` include all five on one line.

`--harbour-screen` opens the docked counting-house at Porto Novo, leases a depot, deposits grain, opens a merchant line and draws on it, buys basic hull insurance, and rejects a license that still needs a broker. It does not advance the day for those calls. It then advances until Next day has logged a `Turn.notes` line (interest, a credit payment, a default, or a seizure). Without a capture it also sails, and the Harbour button leaves with the port row. The last line is `portlight harbour smoke ok`. `PORTLIGHT_SHOT` or `--art-docs` / `PORTLIGHT_ART_DOCS` writes `harbour-warehouse.png`, `harbour-broker.png`, and `harbour-finance.png` (1280×720). `PORTLIGHT_SHOT` names the directory when it is a folder, or the parent of a `.png` path. An explicit capture under headless exits 1 and prints `portlight harbour smoke FAILED`.

`--hunt-screen` opens Hunt at Porto Novo, forages one day, refreshes the bounty board, accepts one target, and closes. It then sails, forages at sea without opening a fight, hunts the accepted target into the existing encounter screen, and expects claim to refuse until that captain is defeated. The last line is `portlight hunt smoke ok`. It does not capture unless `PORTLIGHT_SHOT` or `--art-docs` / `PORTLIGHT_ART_DOCS` is set. Those write `hunt-forage.png`, `hunt-board.png`, and `hunt-active.png` (1280×720) into the shot directory. An explicit capture under headless exits 1 and prints `portlight hunt smoke FAILED`. Harbour, Crew, and Hunt share the docked port row with the other pier buttons. The runner `bounty` verb is not part of this screen.

`--encounter-galleon` writes `/tmp/encounter-galleon.png` unless `PORTLIGHT_SHOT` or `--art-docs` names another path, with `royal_man_of_war` in the player plate slot (caption `Man-of-war`) and that template's hull and crew on the card. The encounter frame check follows the capture mode, not the filename, so `PORTLIGHT_SHOT=/tmp/galleon1.png` is still an encounter frame.

User flags: `--smoke`, `--art`, `--art-docs`, `--encounter-screen`, `--encounter-galleon`, `--newgame-screen`, `--contracts-screen`, `--shipyard-screen`, `--journal-screen`, `--harbour-screen`, `--crew-screen`, `--hunt-screen`, `--encounter`, `--duel`, `--resolve`, `--work`. Environment: `PORTLIGHT_SMOKE`, `PORTLIGHT_SHOT`, `PORTLIGHT_ART_DOCS`, `PORTLIGHT_ENCOUNTER_DIR`, `PORTLIGHT_SEAM_DIR`, `PORTLIGHT_SEAM_ILLEGAL` (`1` is pilings on a pier cell, `2` is a pier on a quay cell). The harbour seam plate:

```
xvfb-run -a -s "-screen 0 1280x720x24" \
  env PORTLIGHT_SHOT=docs/screenshots/chart-cutter-f7.png \
  godot --display-driver x11 --rendering-driver opengl3 --path godot -- --art

xvfb-run -a -s "-screen 0 1280x720x24" \
  env PORTLIGHT_SEAM_DIR=/tmp \
  godot --display-driver x11 --rendering-driver opengl3 --path godot \
  res://scenes/harbour_seam.tscn
```

The seam scene validates the layout before it writes. An illegal layout (pilings on a pier cell, or a pier on a quay cell) prints the fault, exits non-zero, and does not save a PNG. A blank or mostly flat frame fails the same check as `PORTLIGHT_SHOT` and also exits non-zero. The same run writes the quay close-ups `quay-paving-z100.png` and `quay-paving-z072.png`. The zoom-1 crop is view pixels `(340, 160, 360, 340)`. Zoom 0.72 uses that same world window, `(424, 216, 259, 245)`. CI byte-compares both against `docs/screenshots`, with the full seam frames and the vertex crop. Behind the join quay, a kerb of `quay_1111` blocks (row 1, cols -7..-1, and col 0, rows -8..0) fronts a flat deck of 63 cells (cols -7..-1, rows -8..0) that have no raised block. They draw the same locked `quay_flag_*` plates as a block top (art-gate Addendum X). The deck runs past the widest capture view, so no flat back edge meets open water on camera (Addendum X.1); the layout check refuses a flat cell whose open back edge would show. The camera still frames only the quay, pier and pilings. The flat deck close-ups are `flat-deck-z100.png` at view pixels `(224, 72, 432, 352)` and `flat-deck-z072.png` at the same world window, `(340, 153, 311, 253)`; CI byte-compares both.

## Parity

`cargo test` compares every script in `parity/scripts/` to `parity/golden/`. The file list is:

- `abandon_contract.txt`
- `board_fleet_ship.txt`
- `boarding.txt`
- `bounty_board.txt`
- `bounty_claim.txt`
- `bounty_claim_sail.txt`
- `bounty_hunter_voyage.txt`
- `bounty_max.txt`
- `bounty_not_defeated.txt`
- `bounty_not_hunting.txt`
- `bounty_unknown.txt`
- `broker_board.txt`
- `buy_broker.txt`
- `buy_insurance.txt`
- `buy_warehouse.txt`
- `captain_bounty_hunter_s1.txt`
- `captain_bounty_hunter_s42.txt`
- `captain_corsair_s1.txt`
- `captain_corsair_s42.txt`
- `captain_dockhand_s1.txt`
- `captain_dockhand_s42.txt`
- `captain_merchant_prince_s1.txt`
- `captain_merchant_prince_s42.txt`
- `captain_merchant_s1.txt`
- `captain_merchant_s42.txt`
- `captain_navigator_s1.txt`
- `captain_navigator_s42.txt`
- `captain_privateer_s1.txt`
- `captain_privateer_s42.txt`
- `captain_scholar_s1.txt`
- `captain_scholar_s42.txt`
- `captain_smuggler_s1.txt`
- `captain_smuggler_s42.txt`
- `capture_decline.txt`
- `capture_fleet_full.txt`
- `capture_prize.txt`
- `capture_spare_prefix.txt`
- `capture_too_few.txt`
- `capture_too_many.txt`
- `cargo_loss.txt`
- `consequences.txt`
- `contraband_sell.txt`
- `contract_accept.txt`
- `contract_arrival_rng.txt`
- `contract_complete.txt`
- `contract_expire.txt`
- `credit_default.txt`
- `crew_minimum.txt`
- `custom_captain.txt`
- `custom_mentor_port.txt`
- `custom_points_total.txt`
- `custom_port_region.txt`
- `custom_reputation_max.txt`
- `custom_reputation_negative.txt`
- `custom_sailing_max.txt`
- `custom_sailing_negative.txt`
- `custom_shadow_max.txt`
- `custom_shadow_negative.txt`
- `custom_trade_max.txt`
- `custom_trade_negative.txt`
- `custom_unknown_bloc.txt`
- `custom_unknown_faction.txt`
- `custom_unknown_mentor.txt`
- `custom_unknown_port.txt`
- `custom_unknown_region.txt`
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
- `insurance_claim.txt`
- `license_repay.txt`
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
- `save_reload.txt`
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
- `warehouse_deposit.txt`
- `warehouse_seizure.txt`
- `work_at_sea.txt`

To regenerate the goldens from the Python checkout:

```
PYTHONPATH=/path/to/portlight/src python3 tools/extract_content.py
PYTHONPATH=/path/to/portlight/src python3 tools/parity/check.py --write-golden
```

The RNG is CPython's MT19937, so the same seed and script produce the same state for the systems that are ported. Full `GameSession` parity is still open; the plan lists the gaps.
