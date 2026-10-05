# Portlight Bounty — Comprehensive Handoff for Grok Bots

## Executive Summary

This is the Rust port of **Portlight**, a trade-first maritime strategy game. The Python source of truth lives at `mcp-tool-shop-org/portlight`, commit `9b02494cca9cd8f58c4531c41f75be02c9720e43`. Stage 1 (rules + simulation core) is shipped on `main`. Stage 2 (Godot 4 dimetric front end) is merged as #2 at `110469ff1aaa82cf1cf2c98454f878caf3f72d28`. `main` is `1b00b8fd8a87d0ec7ac8dff0cbdfacc4dde2da1b`. **Your job is to port what is still open** so the Godot front end can drive it through the `Session` API. Systems that are merged in the sim are not yet offered by the Godot view.

The sim crate (`crates/portlight-sim`) must remain UI-free. No terminal, no ratatui, no Godot bindings, no graphics dependencies. The CLI crate (`portlight-cli`) and the Godot crate (`portlight-godot`) consume the sim. All new rules code goes in `portlight-sim`.

---

## Architecture — Non-Negotiable Constraints

1. **UI-Free Sim Crate**: `crates/portlight-sim` has zero UI dependencies. It exposes a turn-by-turn `Session` API. The Godot front end on `main` (merged #2) drives `Session` methods.
2. **Deterministic Core**: Every run must reproduce from `(content_json, seed, action_script)`. The RNG is CPython's MT19937 (`pyrand.rs`). Do not introduce non-determinism (wall-clock timestamps, random UUIDs, ambient randomness).
3. **Python Parity**: Where a Python system exists and is within scope, the Rust behavior must match the Python engine for the same seed/script. Use `tools/parity/check.py` with `--write-golden` after changes. New systems must include golden scripts.
4. **Content JSON**: Static catalogs are extracted from the Python checkout by `tools/extract_content.py` into `crates/portlight-sim/data/content.json`, which is embedded with `include_str!`. If you add new content modules (contracts, skills, weapons, etc.), extend `extract_content.py` first, regenerate the JSON, and add matching `Deserialize` structs in `content.rs`.
5. **Python 3 Rounding**: Prices and voyage formulas use CPython's `float.__round__` (banker's rounding). Use `util::py_round`. Do not use Rust's `f64::round` directly.
6. **Error Handling**: All public sim methods return `Result<T, SimError>`. Error messages match the Python sentences where possible.
7. **Session API Surface**: New systems that a player interacts with must expose methods on `Session`. Read-only state goes through `world()`. The system writes its own records inside those methods. `books_mut()` stays a test seam. Godot, scripts, and bots do not finish a victory path by poking it.

---

## Current State (Main Branch)

### Done (locked by parity tests)
- Content catalogs: 18 goods, 20 ports, 43 routes, 5 ship templates, 9 captain archetypes, seasons, factions, pirate captains
- CPython MT19937 RNG (`pyrand.rs`) — checked against CPython 3.12.3
- Python 3 rounding (`util.rs`)
- `new_game`, buy/sell, FIFO cargo, receipt ids, trade reputation
- Market tick, in-port days, heat decay, provisions, wages
- Voyage: depart, advance_day, arrive, event table, inspection reputation
- Sail-picker lanes with raw speed estimates
- Duel system (stance + auto-resolve)
- Four victory paths (`campaign.rs`). Contracts are merged in the sim, so a played game can complete a path. The docked Contracts screen (#33) offers accept / complete / abandon through `Session`.
- Public `Session` API on `main`: `new`, `buy`, `sell`, `depart`, `advance`, `hire_crew`, `provision`, `work`, `duel`, `resolve_pending_duel`, `sail_lanes`, `victory`, `books`/`books_mut`, `save`/`load`, `board`, `accept_contract`, `complete_contract`, `abandon_contract`, `encounter_choice`, `naval_round`, `resolve_boarding`, `buy_ship`, `install_upgrade`, `rename_ship`, `dock_current_ship`, `board_fleet_ship`, `sell_fleet_ship`, `fire_crew`, `repair`, `buy_infrastructure`, `take_credit`, `buy_insurance`, `train_crew`, `recruit_companion`, `spend_skill_point`, `hunt`, `bounty_board`, `accept_bounty`, `hunt_bounty`, `claim_bounty`. The Godot view calls `Session::new`, `buy`, `sell`, `depart`, `advance`, `hire_crew`, `provision`, `work`, `duel`, and `resolve_pending_duel`, and it reads `world`, `books`, and `victory`. The chart reads `sail_lanes`. The docked Contracts screen also calls `available_contracts` / `board`, `accept_contract`, `complete_contract`, and `abandon_contract` (#33). Other methods past `books_mut` remain merged in the sim; see README for which desks are live.
- `get_service_modifier` is ported. `provision`, single-ship `repair`, and dry dock apply it.
- Upgrade bonuses are applied in `ship.rs`. The Godot view does not offer upgrades.
- Bounty-hunter sea event is ported. `bounty_hunter_voyage.txt` sets wanted level 3.

### Merged on `main`

| Area | PR | Merge commit |
|---|---|---|
| Save v12 | #4 | `9dce614d0c5e85964cbd796f7482db438ba15569` |
| Contracts | #3 | `1adb966351c7936fb635afc0fb69b2e82f5dc7f2` |
| Encounters | #5 | `f970259a023c97dd11925c8bc4bf82300bfcb3db` |
| Skills and career | #6 | `711ed6b84a8b4ec783eedd1bc1742baa87ec4e7f` |
| Infrastructure, credit, and insurance | #7 | `33e2e2b93883fa94a4fec986aa8ade376f6bdfb2` |
| Fleet, injuries, weapons, and loot | #8 | `b2e4aba76d4c22cb1713004d8a71df257be87800` |
| Narrative, culture, and consequences | #9 | `33ff36254882a3fb58db30f017d3f6d0fd77155e` |
| Session commands 7a (`rename_ship`, `dock_current_ship`, `board_fleet_ship`, `sell_fleet_ship`, `fire_crew`, `abandon_contract`, single-ship `repair`) | #10 | `f6dc82e46fc622783ce495267dab141e10f33fdf` |
| Godot stage 2 (chart plus harbour seam) | #2 | `110469ff1aaa82cf1cf2c98454f878caf3f72d28` |
| Hunting and bounty 7b (`hunt`, bounty board, accept, hunt, and claim, voyage bounty hunter) | #11 | `1b00b8fd8a87d0ec7ac8dff0cbdfacc4dde2da1b` |

Stage 2 is the chart view. The other rows are merged in the sim and are not yet offered by the Godot view.

### Locked. Do not "fix" these
Python and the stage-1 goldens agree. A later UI calls `sail_lanes` and `Session` rather than correcting the numbers.

- Sail-picker days use raw `ship.speed`. Crew, morale, season, and captain modifiers stay off that estimate.
- The picker lists lanes that `depart` then blocks. A rank gap of 1 is a warning. A gap of 2 or more is `BLOCKED`.
- Victory path id `commercial_empire` is not milestone family `commercial_finance`.
- `Session::advance` passes day `0` into `tick_markets`. Seasonal stock drains stay off. Python's `tick_markets` defaults `current_day` to `0` on that path too.
- `standing_delta` is returned on a duel outcome and is not written onto reputation.
- The navigator's storm resistance and the quartermaster's sell bonus exist in Python and are never called. The +0.5 sea-day speed and the 10% wage discount are already applied. Leave the uncalled pair uncalled.

### In the Godot view
- The encounter screen offers negotiate, flee, naval combat, and boarding through `Session`. The chart still shows the voyage-event stance duel.
- Chart ship-class plates and the water-variant fix are done. `ship_draw` draws each class from the MANIFEST. Cutter, brigantine, and galleon plates are drawn for those classes, and `man_of_war` uses the galleon plates and keeps the class name.

### Not started
- Remaining Godot desks beyond the live chart overlays (see README): narrative depth and any sim systems still without a desk. Contracts are offered: docked Contracts screen (#33) over `Session` board (accept / complete / abandon). New game / save / load, shipyard, crew, harbour, and hunt are also on the chart; do not treat them as "not started".
- `engine/custom_captain.py` and the invariant tests from `stress/`

### Deprioritized
- `engine/underworld.py` and `engine/merchant.py`. Deprioritized, because the Python session never calls them.
- `gunner_damage_mult`, `marine_boarding_bonus`, and `surgeon_death_reduction` are defined in `ship_stats.py` but never called by the Python game; not applied, matching Python.
- `get_fee_modifier` is called only by the Python reputation view, not by gameplay; not applied, matching Python.

### Skipped unless Mike asks
- `printandplay/**` and `balance/**`

---

## Module Porting Map: Python → Rust

The Python source is ~46k lines under `src/portlight/` at commit `9b02494`. Below is every module from the old hit list, mapped to its target Rust file, with complexity notes. Status says whether it is merged.

### Engine Systems (Rules)

| Python Module | Rust Target | Scope | Status | Complexity |
|---|---|---|---|---|
| `engine/contracts.py` | `contracts.rs` + `session.rs` | Contract generation, acceptance, completion, expiry, contract board RNG (`seed + 7919`) | Sim #3 `1adb966351c7936fb635afc0fb69b2e82f5dc7f2`; Godot Contracts desk #33 | High |
| `engine/combat.py` | `combat.rs` | Personal fight after boarding: shoot, throw, dodge, styles. Does not replace `duel.rs` | Merged in the sim, not yet offered by the Godot view. #5 `f970259a023c97dd11925c8bc4bf82300bfcb3db` | High |
| `engine/naval.py` | `naval.rs` | Naval rounds, cannon fire, maneuvering, sinking, prize capture. A failed flee enters here | Merged in the sim, not yet offered by the Godot view. #5 `f970259a023c97dd11925c8bc4bf82300bfcb3db` | High |
| `engine/encounter.py` | `encounter.rs` | Approach is negotiate / flee / fight. Failed flee is `resolve_flee`, then naval. Not the stance duel | Merged in the sim. The Godot encounter screen calls `Session`. #5 `f970259a023c97dd11925c8bc4bf82300bfcb3db`. | High |
| `engine/hunting.py` | `hunting.rs` | Bounty hunting mechanics, patrol behavior, wanted level escalation | Merged in the sim, not yet offered by the Godot view. #11 `1b00b8fd8a87d0ec7ac8dff0cbdfacc4dde2da1b` | Medium |
| `engine/loot.py` | `loot.rs` | Loot tables, prize goods generation, contraband handling | Merged in the sim, not yet offered by the Godot view. #8 `b2e4aba76d4c22cb1713004d8a71df257be87800` | Medium |
| `engine/infrastructure.py` | `infrastructure.rs` | Brokers, warehouses, shipyards, dry docks, port facilities purchase/rent | Merged in the sim, not yet offered by the Godot view. #7 `33e2e2b93883fa94a4fec986aa8ade376f6bdfb2` | High |
| `engine/save.py` | `save.rs` | Port the Python JSON save. Version 12, with its migration chain. Do not invent a binary format | Merged in the sim, not yet offered by the Godot view. #4 `9dce614d0c5e85964cbd796f7482db438ba15569` | High |
| `engine/narrative.py` | `narrative.rs` | Narrative beat generation, story hooks, quest chains | Merged in the sim, not yet offered by the Godot view. #9 `33ff36254882a3fb58db30f017d3f6d0fd77155e` | Medium |
| `engine/consequences.py` | `consequences.rs` | Consequence evaluation, delayed effects, reputation cascades | Merged in the sim, not yet offered by the Godot view. #9 `33ff36254882a3fb58db30f017d3f6d0fd77155e` | Medium |
| `engine/captain_memory.py` | `memory.rs` | Captain memory/ history tracking for narrative callbacks | Merged in the sim, not yet offered by the Godot view. #9 `33ff36254882a3fb58db30f017d3f6d0fd77155e` | Medium |
| `engine/culture_engine.py` | `culture.rs` | Port culture depth, local customs, cultural affinity | Merged in the sim, not yet offered by the Godot view. #9 `33ff36254882a3fb58db30f017d3f6d0fd77155e` | Medium |
| `engine/sea_culture_engine.py` | `sea_culture.rs` | Sea culture enrichment (draws from same RNG after `advance_day`) | Merged in the sim, not yet offered by the Godot view. #9 `33ff36254882a3fb58db30f017d3f6d0fd77155e` | Medium |
| `engine/port_arrival_engine.py` | `port_arrival_engine.rs` | Arrival prose generation, port welcome/turnaway logic | Merged in the sim, not yet offered by the Godot view. #9 `33ff36254882a3fb58db30f017d3f6d0fd77155e` | Medium |
| `engine/companion_engine.py` | `companion.rs` | Companion recruitment, loyalty, companion abilities | Merged in the sim, not yet offered by the Godot view. #6 `711ed6b84a8b4ec783eedd1bc1742baa87ec4e7f` | Medium |
| `engine/skill_engine.py` | `skills.rs` | Skill trees, captain skills, skill checks | Merged in the sim, not yet offered by the Godot view. #6 `711ed6b84a8b4ec783eedd1bc1742baa87ec4e7f` | Medium |
| `engine/training.py` | `training.rs` | Crew training, experience gain, promotion | Merged in the sim, not yet offered by the Godot view. #6 `711ed6b84a8b4ec783eedd1bc1742baa87ec4e7f` | Low |
| `engine/injuries.py` | `injuries.rs` | Injury system, recovery, surgeon effects, permanent wounds | Merged in the sim, not yet offered by the Godot view. #8 `b2e4aba76d4c22cb1713004d8a71df257be87800` | Medium |
| `engine/weapon_quality.py` | `weapon_quality.rs` | Weapon quality tiers, durability, breakage | Merged in the sim, not yet offered by the Godot view. #8 `b2e4aba76d4c22cb1713004d8a71df257be87800` | Low |
| `engine/weapon_provenance.py` | `weapon_provenance.rs` | Weapon origin tracking, provenance bonuses | Merged in the sim, not yet offered by the Godot view. #8 `b2e4aba76d4c22cb1713004d8a71df257be87800` | Low |
| `engine/fleet.py` | `fleet.rs` | Convoy mechanics, fleet formation, multi-ship travel | Merged in the sim, not yet offered by the Godot view. #8 `b2e4aba76d4c22cb1713004d8a71df257be87800`. Area 7a is #10 `f6dc82e46fc622783ce495267dab141e10f33fdf`. | High |
| `engine/bounty.py` | `bounty.rs` | Bounty board, bounty claiming, target tracking | Merged in the sim, not yet offered by the Godot view. #11 `1b00b8fd8a87d0ec7ac8dff0cbdfacc4dde2da1b` | Medium |
| `engine/underworld.py` | `underworld.rs` | Criminal networks, smuggling rings, black market access | Deprioritized, because the Python session never calls them. No `underworld.rs`. `encounter.rs` has its own hostility function. | Medium |
| `engine/merchant.py` | `merchant.rs` | Merchant NPCs, trade partners, bulk deals | Deprioritized, because the Python session never calls them. `merchant.rs` is the markup helper. `Session` does not call that module. `Session::buy_gear` still buys one stocked item at a merchant markup. | Medium |
| `engine/custom_captain.py` | `custom_captain.rs` | Custom captain creation, point-buy system | Not started | Medium |
| `engine/campaign.py` (milestones) | `campaign.rs` | `evaluate_milestones`, career profiles beyond family table | Merged in the sim, not yet offered by the Godot view. #6 `711ed6b84a8b4ec783eedd1bc1742baa87ec4e7f` | High |

### Content Catalogs (Data → JSON → Rust)

| Python Module | Content JSON Section | Rust Struct Target | Notes |
|---|---|---|---|
| `content/contracts.py` | `contracts` | `ContractDef` in `content.rs` | In `content.json` on main |
| `content/infrastructure.py` | `infrastructure` | `InfrastructureDef` | In `content.json` on main |
| `content/armor.py` | `armor` | `ArmorDef` | In `content.json` on main |
| `content/melee_weapons.py` | `melee_weapons` | `MeleeWeaponDef` | In `content.json` on main |
| `content/ranged_weapons.py` | `ranged_weapons` | `RangedWeaponDef` | In `content.json` on main |
| `content/fighting_styles.py` | `fighting_styles` | `FightingStyleDef` | In `content.json` on main |
| `content/skills.py` | `skills` | `SkillDef` | In `content.json` on main |
| `content/injuries.py` | `injuries` | `InjuryDef` | In `content.json` on main |
| `content/loot_tables.py` | `loot_tables` | `LootTableDef` | In `content.json` on main |
| `content/upgrades.py` | `upgrades` | `UpgradeDef` | In `content.json` on main |
| `content/merchants.py` | `merchants` | `MerchantDef` | In `content.json` on main |
| `content/officer_names.py` | `officer_names` | `OfficerNameDef` | In `content.json` on main |
| `content/companions.py` | `companions` | `CompanionDef` | In `content.json` on main |
| `content/culture.py` | `culture` | `CultureDef` | In `content.json` on main |
| `content/sea_culture.py` | `sea_culture` | `SeaCultureDef` | In `content.json` on main |
| `content/port_politics.py` | `port_politics` | `PortPoliticsDef` | In `content.json` on main |
| `content/port_institutions.py` | `port_institutions` | `PortInstitutionDef` | In `content.json` on main |
| `content/port_institutions_east.py` | `port_institutions_east` | `PortInstitutionDef` | In `content.json` on main |
| `content/cross_port_networks.py` | `cross_port_networks` | `CrossPortNetworkDef` | In `content.json` on main |
| `content/campaign.py` | `campaign` | `CampaignDef`, `MilestoneDef` | In `content.json` on main |

### App/UI Layer (Reference Only — Do Not Port into Sim)

These live in Python but must **not** be ported into `portlight-sim`. The Godot front end on `main` (merged #2) replaces them. They are listed only for understanding what the sim must support.

- `app/cli.py` — Replaced by Godot UI
- `app/tui/**` — Replaced by Godot UI
- `app/views.py` — Replaced by Godot UI
- `app/formatting.py` — Replaced by Godot UI
- `app/combat_views.py` — Replaced by Godot UI
- `printandplay/**` — Skipped unless Mike asks

---

## Content Extraction Workflow

When you add a new content catalog, you must:

1. **Extend `tools/extract_content.py`** to import the Python module and serialize it into the JSON dict.
2. **Run** `PYTHONPATH=/path/to/portlight/src python3 tools/extract_content.py` to regenerate `content.json`.
3. **Add matching structs** in `crates/portlight-sim/src/content.rs` with `#[derive(Debug, Clone, Deserialize)]`.
4. **Add accessor methods** on `Content` (e.g., `pub fn contract(&self, id: &str) -> Option<&ContractDef>`).
5. **Add the field** to the `Content` struct so it deserializes.
6. **Commit `content.json`** — it is a generated artifact but must be checked in so CI can build without Python.

Example pattern from existing code:
```rust
// In content.rs
#[derive(Debug, Clone, Deserialize)]
pub struct ContractDef {
    pub id: String,
    pub title: String,
    // ... etc
}

// Add to Content struct:
pub contracts: Vec<ContractDef>,

// Add accessor:
pub fn contract(&self, id: &str) -> Option<&ContractDef> {
    self.contracts.iter().find(|c| c.id == id)
}
```

---

## Session API — How New Systems Surface

`Session` in `session.rs` is the public API. Every player-facing system must expose itself here. Follow these patterns:

- **Action methods**: `pub fn buy(...)`, `pub fn sell(...)`, `pub fn depart(...)`, `pub fn advance(...)` — these mutate world state and return `Result<T, SimError>`.
- **Query methods**: `pub fn sail_lanes(...)`, `pub fn victory(...)` — these read world state and return plain data.
- **Books access**: `pub fn books(&self) -> &HouseBooks` is what victory evaluation reads. `books_mut()` is for tests. A player action updates the books inside its own `Session` method.
- **World access**: `pub fn world(&self) -> &World` — the UI reads raw state here.

When you add a new system (e.g., contracts), add methods like:
```rust
impl Session {
    pub fn available_contracts(&self) -> Vec<&Contract> { ... }
    pub fn accept_contract(&mut self, contract_id: &str) -> Result<(), SimError> { ... }
    pub fn advance_contracts(&mut self) { ... } // tick expiry, etc.
}
```

**Important**: `Session::advance` is the main turn driver. It currently calls:
1. Heat decay
2. `tick_markets` with day `0` (seasonal drains stay off)
3. Provisions / wages
4. Voyage advance or arrival
5. Reprice all ports

Python's `GameSession.advance` already has an order. If your system is called from that method, add the call in that same position. Do not invent a separate lifecycle. Systems Python calls from `advance` today, in order, include `tick_reputation`, `tick_contracts`, infrastructure upkeep, then the in-port or at-sea split. At sea, `enrich_voyage_day` draws the session RNG after `advance_day`. Wiring that draw changes every sea-day golden. Re-run the existing oracle scripts in the same PR.

---

## Model Expansion

`model.rs` on `main` contains `World`, `Captain`, `Ship`, `Voyage`, `Standing`, `Port`, `Route`, `MarketSlot`, `CargoItem`, `Incident`, `PendingDuel`, `DeferredFee`, `Officer`, and the structs named below.

These structs are on `main`:
- `Contract` (active, completed, expired)
- `InfrastructureRecord` (warehouses owned, brokers hired, etc.)
- `PirateEncounterRecord` (for save/load and history)
- `Companion`
- `Skill`
- `Injury`
- `Weapon` / `Armor` (equipped gear)
- `FleetShip` (for convoys)
- `MilestoneRecord`
- `NarrativeBeat`
- `CaptainMemory`

All model structs must derive `Clone` and `Debug` at minimum. If they appear in `Snapshot` (for parity tests), they need `Serialize`. If they are constructed from content JSON, they need `Deserialize` or manual construction from `content.rs` defs.

---

## RNG Discipline

The `Session` holds one `PyRandom` (`rng`). This is the **only** source of randomness.

- **Contract board**: Save the session RNG, draw the refresh from `Random(seed + 7919)`, then restore the session RNG. That draw must not advance the session stream. There is no second long-lived generator.
- **Sea events**: Already ported in `voyage.rs`. Use the session RNG.
- **Combat**: If you port `engine/combat.py`, use the session RNG for hit rolls, damage, etc.
- **Loot**: Use the session RNG for table rolls.
- **Save**: Python does not store the MT19937 state. `load` reseeds with `Random(seed + day)`. Port that. Do not serialize the state vector.

**Never** call `rand::thread_rng()` or `std::random`. Never. The parity harness will catch you.

---

## Testing Requirements

Every new system must include:

1. **Unit tests** in the source file under `mod tests` (see `voyage.rs`, `campaign.rs` for patterns).
2. **Golden scripts** in `parity/scripts/` if the system produces observable state changes. Add the script to `tests/parity.rs` so `golden_scripts_match_python` runs it.
3. **Golden JSON** generated with `python3 tools/parity/check.py --write-golden` against the Python checkout.
4. **Victory fixtures** if the system affects victory paths.

Run this on a branch cut from `main`. The Godot crate is on `main`. The 1.98.1 pin can build it, and the sim CI job excludes `portlight-godot`. The Godot CI job uses stable.

```bash
cargo test --locked --workspace
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

If you touch `extract_content.py` or add new content, also run the Python oracle:
```bash
PYTHONPATH=/path/to/portlight/src python3 tools/parity/check.py --require-oracle
PYTHONPATH=/path/to/portlight/src python3 tools/parity/victory_cases.py --check
```

---

## PR #2 Context (Merged)

Branch: `origin/cursor/rust-port-stage2-dimetric-e4ee`. Merged as #2 at `110469ff1aaa82cf1cf2c98454f878caf3f72d28`.

This adds:
- `crates/portlight-chart/` — Dimetric rendering library (harbour, chart, landing, seam)
- `crates/portlight-godot/` — GDExtension binding the sim to Godot 4.7.2
- `godot/` — Godot project with asset catalog, landing plates (PNG), MANIFEST.json

Branch from current `main`. The sim under that commit is `1b00b8fd8a87d0ec7ac8dff0cbdfacc4dde2da1b`. Do not branch from PR #2, and do not edit `portlight-chart`, `portlight-godot`, or `godot/`. PR #2 does not modify `crates/portlight-sim/`. UI wiring is a later PR. New `Session` methods do not appear in the Godot view by themselves.

Flee is not part of the stance duel in `duel.rs`. That duel stays the voyage-event fight. Approach, flee, naval combat, and boarding are the encounter machine below.

---

## Working in parallel

The seven areas below are merged on `main`. Each one was its own PR. Do not share a branch. Checking out `5decf672b6918a11db016123fe49642ee6ba0348` skips this handoff. Read this file on `main` at `1b00b8fd8a87d0ec7ac8dff0cbdfacc4dde2da1b`.

New rules live in your new `.rs` file. Touch `session.rs`, `model.rs`, `content.rs`, `content.json`, and `tools/extract_content.py` only to add your types and your `Session` methods. Do not reformat those files.

Inside your own area, do the work in this order:

1. Content catalog into JSON, then the `Deserialize` structs.
2. Runtime structs.
3. Rules in the new file.
4. `Session` methods.
5. Unit tests, golden scripts, and the oracle.

`save.rs` ports the Python JSON save of the state that exists on `main` now: world, ledger, trade sequence, and books. Later structs get added by the area that introduces them. Do not block on Areas 1–6, and do not invent a format that includes structs you are not porting.

---

## Specific Deliverables by System Area

### Area 1: Contracts & Contract Economy
Sim #3 `1adb966351c7936fb635afc0fb69b2e82f5dc7f2`; Godot Contracts desk #33.
- [x] `ContractDef` in `content.rs`, `Contract` in `model.rs`
- [x] `contracts.rs` with generation, acceptance, completion, expiry
- [x] Contract board RNG: `PyRandom::from_seed(world.seed + 7919)`
- [x] `Session::available_contracts()`, `Session::accept_contract()`, `Session::complete_contract()`
- [x] `Session::complete_contract` writes the completed contract onto the books. Callers do not use `books_mut()`
- [x] New golden scripts: `contract_accept.txt`, `contract_complete.txt`, `contract_expire.txt`

### Area 2: Interactive Encounter & Combat
Merged in the sim. The Godot encounter screen calls `Session`. #5 `f970259a023c97dd11925c8bc4bf82300bfcb3db`.
- [x] `MeleeWeaponDef`, `RangedWeaponDef`, `ArmorDef`, `FightingStyleDef` in `content.rs`
- [x] `Weapon`, `Armor`, `FightingStyle` in `model.rs`
- [x] `duel.rs` stays as it is. That is the voyage-event stance fight (`thrust` / `slash` / `parry`) and the five-stance auto-resolve.
- [x] `encounter.rs` — approach is negotiate, flee, or fight. `resolve_flee` uses `attempt_flee`. Success leaves. Failure takes the broadside and opens naval combat. It does not enter `duel.rs`.
- [x] `naval.rs` — cannon fire, hull damage, sinking check, prize capture.
- [x] `combat.rs` — the personal fight after boarding: shoot, throw, dodge, styles, stamina. This is `engine/combat.py`, not a second copy of `duel.rs`.
- [x] The print-and-play `2d6` flee rule is not the engine.
- [x] `Session::encounter_choice(choice)`, `Session::naval_round()`, `Session::resolve_boarding()`. `Session::board` is the contract board.
- [x] Encounter history on `Captain` (`duels_won`, `duels_lost`, `PirateEncounterRecord`)
- [x] New golden scripts: `encounter_negotiate.txt`, `encounter_flee.txt`, `naval_combat.txt`, `boarding.txt`

### Area 3: Skills, Companions & Career
Merged in the sim, not yet offered by the Godot view. #6 `711ed6b84a8b4ec783eedd1bc1742baa87ec4e7f`.
- [x] `SkillDef`, `CompanionDef`, `MerchantDef` in `content.rs`
- [x] `Skill`, `Companion`, `CaptainMemory` in `model.rs`
- [x] `skills.rs` — skill tree lookup, skill check rolls
- [x] `companion.rs` — recruitment, loyalty decay, companion abilities
- [x] `training.rs` — crew XP, promotion thresholds
- [x] `campaign.rs` — `evaluate_milestones`, career profiles (`PROFILE_MILESTONE_FAMILIES`)
- [x] `Session::train_crew()`, `Session::recruit_companion()`, `Session::spend_skill_point()`
- [x] Keep `commercial_finance` as the milestone family id and `commercial_empire` as the path id

### Area 4: Infrastructure, Credit & Insurance
Merged in the sim, not yet offered by the Godot view. #7 `33e2e2b93883fa94a4fec986aa8ade376f6bdfb2`.
- [x] `InfrastructureDef` in `content.rs`
- [x] `InfrastructureRecord` in `model.rs`
- [x] `infrastructure.rs` — broker fees, warehouse rent, dry dock repair, insurance policies
- [x] `cross_port_networks.rs` — network effects across ports
- [x] `Session::buy_infrastructure()`, `Session::take_credit()`, `Session::buy_insurance()`
- [x] Victory evaluators read infrastructure from `HouseBooks`

### Area 5: Fleet, Injuries & Weapons
Merged in the sim, not yet offered by the Godot view. #8 `b2e4aba76d4c22cb1713004d8a71df257be87800`. Area 7a is #10 `f6dc82e46fc622783ce495267dab141e10f33fdf`.
- [x] `UpgradeDef`, `InjuryDef`, `LootTableDef` in `content.rs`
- [x] `FleetShip`, `Injury`, `WeaponQuality` in `model.rs`
- [x] `fleet.rs` — convoy formation, fleet speed calculation, shared cargo
- [x] `injuries.rs` — injury table rolls, surgeon recovery bonus, permanent effects
- [x] `weapon_quality.rs` — durability loss on use, breakage check
- [x] `weapon_provenance.rs` — origin bonuses, provenance chains
- [x] `Session::form_convoy()`, `Session::repair_fleet()`

### Area 6: Narrative, Culture & Consequences
Merged in the sim, not yet offered by the Godot view. #9 `33ff36254882a3fb58db30f017d3f6d0fd77155e`.
- [x] `CultureDef`, `SeaCultureDef`, `PortPoliticsDef` in `content.rs`
- [x] `NarrativeBeat`, `Consequence`, `SeaCultureState` in `model.rs`
- [x] `narrative.rs` — beat generation from state triggers
- [x] `consequences.rs` — history-gated effects applied on the day they fire. Python does not keep a delayed queue, and v12 has no queue key.
- [x] `culture.rs` — port cultural affinity effects on prices/reputation
- [x] `sea_culture.rs` — `enrich_voyage_day`. Python calls it after `advance_day` and it draws the session RNG. Putting that call into `Session::advance` is this area's job, in that position, with the existing sea-day goldens regenerated from the oracle in the same PR.
- [x] `port_arrival_engine.rs` — arrival prose, port welcome logic
- [x] `Session::arrival_narrative()`, `Session::evaluate_consequences()`

### Area 7: Save/Load & Persistence
Merged in the sim, not yet offered by the Godot view. #4 `9dce614d0c5e85964cbd796f7482db438ba15569`.
- [x] Port `engine/save.py`. The file is JSON, current version 12, with the v1–v12 migration chain. Do not design a new format.
- [x] Serialize the state that exists on `main`: `World`, ledger, `trade_seq`, and books. Do not serialize the MT19937 state. `Session::load` reseeds with `Random(seed + day)`, matching Python.
- [x] Later areas add their own structs to this format in their own PRs.
- [x] Round-trip test: save, load, same snapshot the Python loader would rebuild for that slot.
- [x] `Session::save()` and `Session::load()` live on the sim. The CLI does not grow a second format.

### Area 7b: Hunting and bounty
Merged in the sim, not yet offered by the Godot view. #11 `1b00b8fd8a87d0ec7ac8dff0cbdfacc4dde2da1b`.
- [x] `Session::hunt()`, `Session::bounty_board()`, `Session::accept_bounty()`, `Session::hunt_bounty()`, `Session::claim_bounty()`
- [x] Voyage bounty hunter at wanted level 3 (`bounty_hunter_voyage.txt`)
- [x] Golden scripts: `hunt_port_success.txt`, `hunt_port_fail.txt`, `hunt_sea_success.txt`, `hunt_sea_fail.txt`, `hunt_sea_morale.txt`, `bounty_board.txt`, `bounty_claim.txt`, `bounty_max.txt`, `bounty_not_defeated.txt`, `bounty_not_hunting.txt`, `bounty_unknown.txt`, `bounty_hunter_voyage.txt`

---

## Content JSON Expansion Checklist

As you add catalogs, tick them off:
- [x] `contracts`
- [x] `infrastructure`
- [x] `armor`
- [x] `melee_weapons`
- [x] `ranged_weapons`
- [x] `fighting_styles`
- [x] `skills`
- [x] `injuries`
- [x] `loot_tables`
- [x] `upgrades`
- [x] `merchants`
- [x] `officer_names`
- [x] `companions`
- [x] `culture`
- [x] `sea_culture`
- [x] `port_politics`
- [x] `port_institutions`
- [x] `port_institutions_east`
- [x] `cross_port_networks`
- [x] `campaign`

---

## Files You Will Touch

**Existing files to modify:**
- `crates/portlight-sim/src/content.rs` — add new `Deserialize` structs
- `crates/portlight-sim/src/model.rs` — add new runtime structs
- `crates/portlight-sim/src/session.rs` — expose new systems on `Session`
- `crates/portlight-sim/src/lib.rs` — re-export new public modules
- `crates/portlight-sim/src/campaign.rs` — add milestone evaluation
- `tools/extract_content.py` — serialize new catalogs

**On `main`:**
- `crates/portlight-sim/src/contracts.rs`
- `crates/portlight-sim/src/combat.rs`
- `crates/portlight-sim/src/naval.rs`
- `crates/portlight-sim/src/encounter.rs`
- `crates/portlight-sim/src/infrastructure.rs`
- `crates/portlight-sim/src/save.rs`
- `crates/portlight-sim/src/narrative.rs`
- `crates/portlight-sim/src/consequences.rs`
- `crates/portlight-sim/src/culture.rs`
- `crates/portlight-sim/src/sea_culture.rs`
- `crates/portlight-sim/src/companion.rs`
- `crates/portlight-sim/src/skills.rs`
- `crates/portlight-sim/src/training.rs`
- `crates/portlight-sim/src/injuries.rs`
- `crates/portlight-sim/src/weapon_quality.rs`
- `crates/portlight-sim/src/weapon_provenance.rs`
- `crates/portlight-sim/src/fleet.rs`
- `crates/portlight-sim/src/hunting.rs`
- `crates/portlight-sim/src/loot.rs`
- `crates/portlight-sim/src/merchant.rs` (markup helper only; deprioritized)
- `crates/portlight-sim/src/bounty.rs`
- `crates/portlight-sim/src/memory.rs`
- `crates/portlight-sim/src/port_arrival_engine.rs`

**Not on `main`:**
- `crates/portlight-sim/src/underworld.rs` (deprioritized, because the Python session never calls it)
- `crates/portlight-sim/src/custom_captain.rs` (not started)

**Test files to modify/create:**
- `crates/portlight-sim/tests/parity.rs` — add new golden script names
- `parity/scripts/*.txt` — new action scripts
- `parity/golden/*.json` — generated from Python oracle

---

## Communication

When you finish a slice:
1. Run `cargo test --locked --workspace`
2. Run `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings`
3. If you changed content, regenerate goldens with the Python oracle
4. Update this handoff: tick off completed items in the checklist
5. Open a PR against `main`. One area per PR. PR #2 is merged at `110469ff1aaa82cf1cf2c98454f878caf3f72d28`.

---

## Source of Truth Access

The Python game is public at `mcp-tool-shop-org/portlight`. To check out the exact commit:
```bash
git clone https://github.com/mcp-tool-shop-org/portlight.git
cd portlight
git checkout 9b02494cca9cd8f58c4531c41f75be02c9720e43
```

The `src/portlight/` directory contains all Python modules referenced above. Read them. Understand the data structures and the control flow. Port faithfully.

---

*Handoff generated on 2026-09-28. Read it from `main` at `docs/HANDOFF-GROK-BOTS.md`. `main` is `1b00b8fd8a87d0ec7ac8dff0cbdfacc4dde2da1b`. Stage 1 is `5decf672b6918a11db016123fe49642ee6ba0348`. PR #2 is merged at `110469ff1aaa82cf1cf2c98454f878caf3f72d28`.*
