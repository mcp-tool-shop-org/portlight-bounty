# Portlight Bounty — Comprehensive Handoff for Grok Bots

## Executive Summary

This is the Rust port of **Portlight**, a trade-first maritime strategy game. The Python source of truth lives at `mcp-tool-shop-org/portlight`, commit `9b02494cca9cd8f58c4531c41f75be02c9720e43`. Stage 1 (rules + simulation core) is shipped on `main`. Stage 2 (Godot 4 dimetric front end) is in draft PR #2. **Your job is to continue porting the remaining ~30,000 lines of Python systems into the Rust sim crate** so the Godot front end can eventually drive them through the `Session` API.

The sim crate (`crates/portlight-sim`) must remain UI-free. No terminal, no ratatui, no Godot bindings, no graphics dependencies. The CLI crate (`portlight-cli`) and the Godot crate (`portlight-godot`) consume the sim. All new rules code goes in `portlight-sim`.

---

## Architecture — Non-Negotiable Constraints

1. **UI-Free Sim Crate**: `crates/portlight-sim` has zero UI dependencies. It exposes a turn-by-turn `Session` API. The Godot front end (PR #2) drives `Session` methods.
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
- Four victory paths (`campaign.rs`) — display-only until contracts are ported
- Public `Session` API: `new`, `buy`, `sell`, `depart`, `advance`, `hire_crew`, `provision`, `work`, `duel`, `resolve_pending_duel`, `sail_lanes`, `victory`, `books`/`books_mut`

### Locked. Do not "fix" these
Python and the stage-1 goldens agree. A later UI calls `sail_lanes` and `Session` rather than correcting the numbers.

- Sail-picker days use raw `ship.speed`. Crew, morale, season, and captain modifiers stay off that estimate.
- The picker lists lanes that `depart` then blocks. A rank gap of 1 is a warning. A gap of 2 or more is `BLOCKED`.
- Victory path id `commercial_empire` is not milestone family `commercial_finance`.
- `Session::advance` passes day `0` into `tick_markets`. Seasonal stock drains stay off. Python's `tick_markets` defaults `current_day` to `0` on that path too.
- `standing_delta` is returned on a duel outcome and is not written onto reputation.
- The navigator's storm resistance and the quartermaster's sell bonus exist in Python and are never called. The +0.5 sea-day speed and the 10% wage discount are already applied. Leave the uncalled pair uncalled.

### Partial (code present, not wired to session or not fully tested)
- Upgrade-aware ship stats (functions assume stock ship)
- Crew roles: only wages, casualty weights, hiring are done; gunner/marine/surgeon effects belong to combat
- Bounty-hunter sea event (code exists for wanted level 3, no golden covers it)
- Fee/service modifiers on provisions only; other services not wired

### Not Started (Your Work)
Everything below is unported and lives in the Python checkout. This is your hit list.

---

## Module Porting Map: Python → Rust

The Python source is ~46k lines under `src/portlight/` at commit `9b02494`. Below is every unported module, mapped to its target Rust file, with complexity notes.

### Engine Systems (Rules)

| Python Module | Rust Target | Scope | Complexity |
|---|---|---|---|
| `engine/contracts.py` | `contracts.rs` + `session.rs` | Contract generation, acceptance, completion, expiry, contract board RNG (`seed + 7919`) | High |
| `engine/combat.py` | `combat.rs` | Personal fight after boarding: shoot, throw, dodge, styles. Does not replace `duel.rs` | High |
| `engine/naval.py` | `naval.rs` | Naval rounds, cannon fire, maneuvering, sinking, prize capture. A failed flee enters here | High |
| `engine/encounter.py` | `encounter.rs` | Approach is negotiate / flee / fight. Failed flee is `resolve_flee`, then naval. Not the stance duel | High |
| `engine/hunting.py` | `hunting.rs` | Bounty hunting mechanics, patrol behavior, wanted level escalation | Medium |
| `engine/loot.py` | `loot.rs` | Loot tables, prize goods generation, contraband handling | Medium |
| `engine/infrastructure.py` | `infrastructure.rs` | Brokers, warehouses, shipyards, dry docks, port facilities purchase/rent | High |
| `engine/save.py` | `save.rs` | Port the Python JSON save. Version 12, with its migration chain. Do not invent a binary format | High |
| `engine/narrative.py` | `narrative.rs` | Narrative beat generation, story hooks, quest chains | Medium |
| `engine/consequences.py` | `consequences.rs` | Consequence evaluation, delayed effects, reputation cascades | Medium |
| `engine/captain_memory.py` | `memory.rs` | Captain memory/ history tracking for narrative callbacks | Medium |
| `engine/culture_engine.py` | `culture.rs` | Port culture depth, local customs, cultural affinity | Medium |
| `engine/sea_culture_engine.py` | `sea_culture.rs` | Sea culture enrichment (draws from same RNG after `advance_day`) | Medium |
| `engine/port_arrival_engine.py` | `arrival.rs` | Arrival prose generation, port welcome/turnaway logic | Medium |
| `engine/companion_engine.py` | `companion.rs` | Companion recruitment, loyalty, companion abilities | Medium |
| `engine/skill_engine.py` | `skills.rs` | Skill trees, captain skills, skill checks | Medium |
| `engine/training.py` | `training.rs` | Crew training, experience gain, promotion | Low |
| `engine/injuries.py` | `injuries.rs` | Injury system, recovery, surgeon effects, permanent wounds | Medium |
| `engine/weapon_quality.py` | `weapon_quality.rs` | Weapon quality tiers, durability, breakage | Low |
| `engine/weapon_provenance.py` | `weapon_provenance.rs` | Weapon origin tracking, provenance bonuses | Low |
| `engine/fleet.py` | `fleet.rs` | Convoy mechanics, fleet formation, multi-ship travel | High |
| `engine/bounty.py` | `bounty.rs` | Bounty board, bounty claiming, target tracking | Medium |
| `engine/underworld.py` | `underworld.rs` | Criminal networks, smuggling rings, black market access | Medium |
| `engine/merchant.py` | `merchant.rs` | Merchant NPCs, trade partners, bulk deals | Medium |
| `engine/custom_captain.py` | `custom_captain.rs` | Custom captain creation, point-buy system | Medium |
| `engine/campaign.py` (milestones) | `campaign.rs` | `evaluate_milestones`, career profiles beyond family table | High |

### Content Catalogs (Data → JSON → Rust)

| Python Module | Content JSON Section | Rust Struct Target | Notes |
|---|---|---|---|
| `content/contracts.py` | `contracts` | `ContractDef` in `content.rs` | Add to `extract_content.py` |
| `content/infrastructure.py` | `infrastructure` | `InfrastructureDef` | Add to `extract_content.py` |
| `content/armor.py` | `armor` | `ArmorDef` | Add to `extract_content.py` |
| `content/melee_weapons.py` | `melee_weapons` | `MeleeWeaponDef` | Add to `extract_content.py` |
| `content/ranged_weapons.py` | `ranged_weapons` | `RangedWeaponDef` | Add to `extract_content.py` |
| `content/fighting_styles.py` | `fighting_styles` | `FightingStyleDef` | Add to `extract_content.py` |
| `content/skills.py` | `skills` | `SkillDef` | Add to `extract_content.py` |
| `content/injuries.py` | `injuries` | `InjuryDef` | Add to `extract_content.py` |
| `content/loot_tables.py` | `loot_tables` | `LootTableDef` | Add to `extract_content.py` |
| `content/upgrades.py` | `upgrades` | `UpgradeDef` | Add to `extract_content.py` |
| `content/merchants.py` | `merchants` | `MerchantDef` | Add to `extract_content.py` |
| `content/officer_names.py` | `officer_names` | `OfficerNameDef` | Add to `extract_content.py` |
| `content/companions.py` | `companions` | `CompanionDef` | Add to `extract_content.py` |
| `content/culture.py` | `culture` | `CultureDef` | Add to `extract_content.py` |
| `content/sea_culture.py` | `sea_culture` | `SeaCultureDef` | Add to `extract_content.py` |
| `content/port_politics.py` | `port_politics` | `PortPoliticsDef` | Add to `extract_content.py` |
| `content/port_institutions.py` | `port_institutions` | `PortInstitutionDef` | Add to `extract_content.py` |
| `content/port_institutions_east.py` | `port_institutions_east` | `PortInstitutionDef` | Add to `extract_content.py` |
| `content/cross_port_networks.py` | `cross_port_networks` | `CrossPortNetworkDef` | Add to `extract_content.py` |
| `content/campaign.py` | `campaign` | `CampaignDef`, `MilestoneDef` | Add to `extract_content.py` |

### App/UI Layer (Reference Only — Do Not Port into Sim)

These live in Python but must **not** be ported into `portlight-sim`. The Godot front end (PR #2) replaces them. They are listed only for understanding what the sim must support.

- `app/cli.py` — Replaced by Godot UI
- `app/tui/**` — Replaced by Godot UI
- `app/views.py` — Replaced by Godot UI
- `app/formatting.py` — Replaced by Godot UI
- `app/combat_views.py` — Replaced by Godot UI
- `printandplay/**` — Reference material, not ported

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

`model.rs` currently contains `World`, `Captain`, `Ship`, `Voyage`, `Standing`, `Port`, `Route`, `MarketSlot`, `CargoItem`, `Incident`, `PendingDuel`, `DeferredFee`, `Officer`.

You will need to add structs for:
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

Run this on a branch cut from `main`. That workspace is the sim and the CLI. The Godot crate is not on `main`, and the 1.83 pin cannot build it.

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

## PR #2 Context (Draft — Do Not Merge Yet)

Branch: `origin/cursor/rust-port-stage2-dimetric-e4ee`

This adds:
- `crates/portlight-chart/` — Dimetric rendering library (harbour, chart, landing, seam)
- `crates/portlight-godot/` — GDExtension binding the sim to Godot 4.7.2
- `godot/` — Godot project with asset catalog, landing plates (PNG), MANIFEST.json

Branch from current `main`. The sim under that commit is `5decf67`. Do not branch from PR #2, and do not edit `portlight-chart`, `portlight-godot`, or `godot/`. PR #2 does not modify `crates/portlight-sim/`. UI wiring is a later PR. New `Session` methods do not appear in the Godot view by themselves.

Flee is not part of the stance duel in `duel.rs`. That duel stays the voyage-event fight. Approach, flee, naval combat, and boarding are the encounter machine below.

---

## Working in parallel

The seven areas below are seven branches. Each one is cut from current `main`, and each one is its own PR. Do not share a branch. Checking out `5decf67` skips this handoff.

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
- `ContractDef` in `content.rs`, `Contract` in `model.rs`
- `contracts.rs` with generation, acceptance, completion, expiry
- Contract board RNG: `PyRandom::from_seed(world.seed + 7919)`
- `Session::available_contracts()`, `Session::accept_contract()`, `Session::complete_contract()`
- `Session::complete_contract` writes the completed contract onto the books. Callers do not use `books_mut()`
- New golden scripts: `contract_accept.txt`, `contract_complete.txt`, `contract_expire.txt`

### Area 2: Interactive Encounter & Combat
- `MeleeWeaponDef`, `RangedWeaponDef`, `ArmorDef`, `FightingStyleDef` in `content.rs`
- `Weapon`, `Armor`, `FightingStyle` in `model.rs`
- `duel.rs` stays as it is. That is the voyage-event stance fight (`thrust` / `slash` / `parry`) and the five-stance auto-resolve.
- `encounter.rs` — approach is negotiate, flee, or fight. `resolve_flee` uses `attempt_flee`. Success leaves. Failure takes the broadside and opens naval combat. It does not enter `duel.rs`.
- `naval.rs` — cannon fire, hull damage, sinking check, prize capture.
- `combat.rs` — the personal fight after boarding: shoot, throw, dodge, styles, stamina. This is `engine/combat.py`, not a second copy of `duel.rs`.
- The print-and-play `2d6` flee rule is not the engine.
- `Session::encounter_choice(choice)`, `Session::naval_round()`, `Session::board()`
- Encounter history on `Captain` (`duels_won`, `duels_lost`, `PirateEncounterRecord`)
- New golden scripts: `encounter_negotiate.txt`, `encounter_flee.txt`, `naval_combat.txt`, `boarding.txt`

### Area 3: Skills, Companions & Career
- `SkillDef`, `CompanionDef`, `MerchantDef` in `content.rs`
- `Skill`, `Companion`, `CaptainMemory` in `model.rs`
- `skills.rs` — skill tree lookup, skill check rolls
- `companion.rs` — recruitment, loyalty decay, companion abilities
- `training.rs` — crew XP, promotion thresholds
- `campaign.rs` — `evaluate_milestones`, career profiles (`PROFILE_MILESTONE_FAMILIES`)
- `Session::train_crew()`, `Session::recruit_companion()`, `Session::spend_skill_point()`
- Keep `commercial_finance` as the milestone family id and `commercial_empire` as the path id

### Area 4: Infrastructure, Credit & Insurance
- `InfrastructureDef` in `content.rs`
- `InfrastructureRecord` in `model.rs`
- `infrastructure.rs` — broker fees, warehouse rent, dry dock repair, insurance policies
- `cross_port_networks.rs` — network effects across ports
- `Session::buy_infrastructure()`, `Session::take_credit()`, `Session::buy_insurance()`
- Victory evaluators read infrastructure from `HouseBooks`

### Area 5: Fleet, Injuries & Weapons
- `UpgradeDef`, `InjuryDef`, `LootTableDef` in `content.rs`
- `FleetShip`, `Injury`, `WeaponQuality` in `model.rs`
- `fleet.rs` — convoy formation, fleet speed calculation, shared cargo
- `injuries.rs` — injury table rolls, surgeon recovery bonus, permanent effects
- `weapon_quality.rs` — durability loss on use, breakage check
- `weapon_provenance.rs` — origin bonuses, provenance chains
- `Session::form_convoy()`, `Session::repair_fleet()`

### Area 6: Narrative, Culture & Consequences
- `CultureDef`, `SeaCultureDef`, `PortPoliticsDef` in `content.rs`
- `NarrativeBeat`, `Consequence`, `SeaCultureState` in `model.rs`
- `narrative.rs` — beat generation from state triggers
- `consequences.rs` — delayed effect evaluation (e.g., "in 3 days, heat +10")
- `culture.rs` — port cultural affinity effects on prices/reputation
- `sea_culture.rs` — `enrich_voyage_day`. Python calls it after `advance_day` and it draws the session RNG. Putting that call into `Session::advance` is this area's job, in that position, with the existing sea-day goldens regenerated from the oracle in the same PR.
- `port_arrival_engine.rs` — arrival prose, port welcome logic
- `Session::arrival_narrative()`, `Session::evaluate_consequences()`

### Area 7: Save/Load & Persistence
- [x] Port `engine/save.py`. The file is JSON, current version 12, with the v1–v12 migration chain. Do not design a new format.
- [x] Serialize the state that exists on `main`: `World`, ledger, `trade_seq`, and books. Do not serialize the MT19937 state. `Session::load` reseeds with `Random(seed + day)`, matching Python.
- [x] Later areas add their own structs to this format in their own PRs.
- [x] Round-trip test: save, load, same snapshot the Python loader would rebuild for that slot.
- [x] `Session::save()` and `Session::load()` live on the sim. The CLI does not grow a second format.

---

## Content JSON Expansion Checklist

As you add catalogs, tick them off:
- [ ] `contracts`
- [ ] `infrastructure`
- [ ] `armor`
- [ ] `melee_weapons`
- [ ] `ranged_weapons`
- [ ] `fighting_styles`
- [ ] `skills`
- [ ] `injuries`
- [ ] `loot_tables`
- [ ] `upgrades`
- [ ] `merchants`
- [ ] `officer_names`
- [ ] `companions`
- [ ] `culture`
- [ ] `sea_culture`
- [ ] `port_politics`
- [ ] `port_institutions`
- [ ] `port_institutions_east`
- [ ] `cross_port_networks`
- [ ] `campaign`

---

## Files You Will Touch

**Existing files to modify:**
- `crates/portlight-sim/src/content.rs` — add new `Deserialize` structs
- `crates/portlight-sim/src/model.rs` — add new runtime structs
- `crates/portlight-sim/src/session.rs` — expose new systems on `Session`
- `crates/portlight-sim/src/lib.rs` — re-export new public modules
- `crates/portlight-sim/src/campaign.rs` — add milestone evaluation
- `tools/extract_content.py` — serialize new catalogs

**New files to create (examples):**
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
- `crates/portlight-sim/src/merchant.rs`
- `crates/portlight-sim/src/underworld.rs`
- `crates/portlight-sim/src/bounty.rs`
- `crates/portlight-sim/src/memory.rs`
- `crates/portlight-sim/src/arrival.rs`
- `crates/portlight-sim/src/custom_captain.rs`

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
5. Open a PR against `main`. One area per PR. Leave PR #2 a draft.

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

*Handoff generated on 2026-09-28. Read it from `main` at `docs/HANDOFF-GROK-BOTS.md`. The stage 1 sim underneath is `5decf67`. PR #2 branch is `cursor/rust-port-stage2-dimetric-e4ee` at `0f2eedf`.*
