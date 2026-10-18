# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.1] - 2026-10-18

### Changed

- Hardened CI workflows (`ci.yml`, `pages.yml`, `codeql.yml`): pinned all runner images to `ubuntu-24.04` and set explicit top-level `permissions: contents: read`.
- Synchronized canon status tables across `docs/canon/` for `DF-34` through `DF-48` and updated the handbook for first-time headless Godot asset import (`godot --headless --path godot --import --quit`) and manifest `0.3.0`.
- Clarified `LICENSE` with the canonical `MIT License` header at line 1 ahead of the deck-prop provenance note.

### Fixed

- Scrubbed internal studio and local playtest artifact paths from public documentation (`docs/DESIGN-FIXES.md`, `docs/PORTING-PLAN.md`, `docs/UI-UX-AUDIT.md`) and removed non-shipped dev artifacts (`.claude/scheduled_tasks.lock`, `docs/Chart Audit.md`, `tools/balance_345.txt`, `tools/probe_all_types.txt`).
- Guarded simulation economy, bank, fleet, crew, and provision mutations against negative inputs and integer overflow (`buy`, `sell`, `bank_deposit`, `bank_withdraw`, `bank_letter`, `deposit_to_warehouse`, `withdraw_from_warehouse`, `repay_credit`, `hire_crew`, `fire_crew`, `provision`, `transfer_cargo`, `transfer_crew`).
- Fixed `portlight script -` so reading from stdin executes the script instead of exiting early, and rejected unknown flags or missing values in `portlight new` with exit code 2.
- Added `equip_style` and `dismiss` (alias for `dismiss_companion`) to the CLI script runner and `--help` output.
- Fixed Godot route-pirate boarding duel resolution so `take_all` and `spare` log honest route-pirate messages without claiming unapplied underworld standing deltas, and exposed `state.pending_duel` in the headless playtest snapshot.
- Fixed recruit notice formatting on the Godot Crew desk so companion `greeting` lines render without doubled quotes.
- Removed unshipped `well_1x1` from `portlight-chart` seam allowances so the runtime allowlist matches the 20 assets in `godot/assets/landing/MANIFEST.json`.

## [0.3.0] - 2026-10-18

### Added

- Armoury merchant shop on the Crew desk (`SECTION_GEAR`): displays carried melee, ranged, and armour gear alongside the docked port merchant's name, title, personality, markup, description, greeting, and regional stock (`DF-46`).
- Style master and skill trainer port presence on the Crew desk (`SECTION_TRAINING`): renders resident master and trainer descriptions and dialog, gates fighting-style learning on `training_port_ids` or `masters_at_port`, and gates skill learning on `trainers_at` or `training_port_feature` (`DF-39`).
- Gear maintenance and blacksmith quality upgrades (`Session::upgrade_weapon`, `Session::field_repair_weapon`, Crew desk `Maintain` and `Upgrade` buttons, and script runner `smith` and `field_repair` verbs) (`DF-39`).
- Companion role effects wired across voyage speed, route danger, customs inspection evasion, market buy/sell pricing, injury healing rate (including at-sea surgeon treatment), and personal combat damage/interception, plus party cohesion, summary, role descriptions, and companion dismissal (`Session::dismiss_companion`, `dismiss_companion` script verb) on the Crew desk (`DF-34`).
- Full companion morale trigger coverage (`combat_won`, `combat_lost`, `crew_killed`, `profitable_trade`, `losing_trade`, `contraband_trade`, `spared_enemy`, `took_all`, `fled_combat`, `injury_sustained`, `long_voyage`, `port_arrival`, `nemesis_defeated`), personality modifier application when a role's base delta is zero, and companion `loyalty_line` / `warning_line` display on the Crew desk (`DF-35`).
- Multi-language README translations (`README.ja.md`, `README.zh.md`, `README.es.md`, `README.fr.md`, `README.hi.md`, `README.it.md`, `README.pt-BR.md`).

### Changed

- Canon lore and catalog alignment (`DF-1`–`DF-8`, `DF-12`–`DF-16`, `DF-18`, `DF-20`, `DF-27`, `DF-28`, `DF-37`): Dimitri Andros's sea sighting is filed under `porto_novo`, Porto Novo's power structure and Harbor Master's Tower beacon are aligned with canon, Silva Bay's repair and war-fleet prose matches the simulation, cross-port network NPC names and the Rosa/Ingrid sea-widows description use canonical names and regions, same-name NPCs/merchants/companions are disambiguated, The Bounty Hunter's `home_region` is `East Indies` (`crosswind_isle`), non-shipyard institutions use accurate `institution_type` values, Ironhaven's `trade_embargo` omits `weapons`, trade-bloc `rivals` and `hostile_to` lists are symmetric, and Jade Port's `jp_quality` is `The Quality Hall`.
- Bounty balance and standing fixes (`DF-30`, `DF-31`, `DF-32`): Gnaw's bounty reward is raised from 200 to 250 silver, Scarlet Ana's bounty region is set to `Mediterranean` (matching her Corsair's Rest sighting), and `record_duel_standing` preserves existing negative underworld standing down to `-100` so The Privateer's `-5` Iron Wolves grudge moves to `-3` on a take-all win instead of clamping to `0`.

## [0.2.0] - 2026-10-17

### Added

- Counting-house silver accounts at seven ports (`bank_deposit`, `bank_withdraw`, `bank_letter`) on the Harbour desk and in the script runner, with version-13 saves when an account is open.
- Sinking consequences and recovery: hull-zero wreck penalty (30% of carried silver, ~75% of each cargo good), tow to the nearer voyage endpoint with a per-class patched hull floor, cargo insurance payouts on a wreck, low-hull departure warnings, and a hunt hull floor.
- Combat and contract recap cards: per-row causes, fight cost lines (hull and crew lost), boarding meter on the Naval card, duel and sinking rows, and contract completion/expiry/breach recaps.
- Closed bounty loop on the Hunt screen (accept, hunt, defeat, and claim) and the Bounty Hunter archetype's +25% claim reward via `tuned_overlay.json`.
- Fighting-style equipping (`Session::equip_style` and the Crew screen `Equip` control) and active-injury body-part checks on style training and equipping.
- Rust-tuned captain pricing and burn overlay (`crates/portlight-sim/data/tuned_overlay.json`), pinned in `parity/rust_tuned_pins.json` with a `python-base` feature to compile the overlay out, plus the deterministic day-60 type-balance bot (`balance_bot`) and contraband reachability probe (`contraband_probe`).
- First-voyage chart hints and docked desk consistency (Escape closes the active desk, undock clears open desks, and only one desk opens at a time).
- Canon documentation (Phases 1–6) under `docs/canon/`: 20 ports, 21 landmarks, 15 sea lanes, 7 trade blocs, 4 pirate factions, 9 player archetypes, 8 pirate captains, 9 player-type guides, 10 companions, 5 companion roles, 5 regional officer pools, 10 style masters, 5 skill trainers, and 14 merchants.

### Changed

- Contract-sale feedback shows partial-sale progress, `'incl. bonus'` notices, and live progress on the contract strip; abandoning a contract names the progress it loses.
- Day's report opens before a same-tick sea encounter when advancing the voyage.
- Shortage offer reasons, crew training/skill/recruit notices, and four catalog copy overrides (`COPY_OVERRIDES` in `tools/extract_content.py`) use display names and clean ASCII prose.

### Fixed

- Buying ammunition without a matching firearm or mechanical launcher is refused before silver is taken, and upgrading a launcher preserves existing ammunition for that slot.
- Loading a save restores route `lore_name` values from the content catalog.
- The pictures are under the MIT license with the rest of the repository. The barrel, bollard, cart, crate, and torch are built by `build_prop()` in ai-rpg-stage, commit `55ac9b2`. The v0.1.0 preface said the record did not name who made them.

## [0.1.0] - 2026-10-09

First GitHub release of the Rust port and the Godot chart.

### Added

- The simulation crate, checked against the Python game at `9b02494`.
- A command-line runner: `new`, `script`, and `load`.
- The Godot 4.7 chart: new game, save and load, sail, market, contracts, crew, shipyard, harbour, hunt, journal, the day's report, and sea fights.
- The landing bundle at manifest 0.4.3. The v0.1.0 tag said the pictures were outside the MIT grant. That sentence was wrong. See 0.2.0.
- A landing page and a handbook.
- Patch coverage fails under 90%. The repository-wide percentage aims at 90% and stays informational. The Godot crate is outside that report.

### Changed

- `portlight --help` lists the script commands the runner actually accepts, and the exit codes it uses.
