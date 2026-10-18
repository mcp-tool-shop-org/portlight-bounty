# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
