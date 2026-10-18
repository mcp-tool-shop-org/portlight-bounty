# Skill trainer canon index

Base commit: af3d105 (merged main, which includes phase 6A; content.json, the sim and the godot UI as cited are unchanged since 515a488). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at af3d105. Python cites read portlight@9b02494 path:line. Legend: [exists] code or data fact with a cite | [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

Every table row below that carries a cite is [exists] (a code or data fact); rows tagged otherwise say so.

| Key | Value | Cite |
|---|---|---|
| Skills in the data | 1 (Blacksmithing) | C:5367 |
| Skill levels | 3 (Apprentice, Journeyman, Master) | C:5374-5388 |
| Skill trainers (named, authored) | 5, all for Blacksmithing | C:5397 |
| Ports with a trainer | 5 of 20 | skills.trainers[*].port_id |
| Teaching limit | each trainer teaches up to a level (1, 2 or 3) | C:5403, C:5412, C:5421, C:5430, C:5439 |
| Rust record shape | id, name, skill, port, teaching limit, description, dialog | content.rs:296-304 |
| Python reference | same 5 records, same strings (compared field by field, no differences) | portlight@9b02494 src/portlight/content/skills.py:101-148 |

The five trainers are the authored teachers of the only skill. Style masters (fighting styles) are a separate index: [style-masters/README.md](../style-masters/README.md).

## 1. What the player sees today

Every table row below that carries a cite is [exists] (a code or data fact); rows tagged otherwise say so.

| What | Reaches the player? | Cite |
|---|---|---|
| A skill line at the docked port: 'Blacksmithing  <level>. <trainer names> (teaches to N).' | Yes (built in code from the trainer names and the highest teaching limit) | logic.rs:2063-2128 (line at 2112-2115) |
| 'Next: <level>  N silver  N days.' or 'Already at maximum.' | Yes | logic.rs:2098-2111 |
| 'No Blacksmithing trainer at this port.' | Yes, at a port with none (uses the skill's catalog name; DF-40 resolved) | logic.rs:2150-2162 |
| A confirm line: 'Learn <level> Blacksmithing for N silver. The calendar will advance N days.' | Yes | logic.rs:1896-1898; game.rs:5627-5650 |
| A notice after learning: 'Trained Blacksmithing. The calendar advanced N days.' | Yes, with the catalog name (DF-40 resolved) | logic.rs:1954-1963; game.rs:5733-5754 |
| Trainer description and dialog | Yes: `CrewSkillLine.trainer_description` and `trainer_dialog` on the Crew desk (DF-39 resolved) | content.rs:296-304; logic.rs; game.rs |
| Skill description and the level descriptions | Level description reaches the desk (`CrewSkillLine.level_description`, DF-39 resolved) | logic.rs; game.rs |
| The level effects (maintenance discount, slower wear, field repair, cheaper upgrades) | Yes: see section 3 (DF-39 resolved) | session.rs; skills.rs; game.rs |
| Character art | None: the character hold stands | crew_screen.rs:3-6 |

## 2. The skill and its levels

Skill: Blacksmithing, "The art of metalwork - maintaining, repairing, and improving weapons and armor." (C:5369). Its training_port_feature is shipyard (C:5370); `skills::can_learn_skill` checks both `skill.training_port_feature` against the docked port's features and `catalog.trainers_at` (skills.rs; DF-39 resolved). All 5 trainer ports have the shipyard feature, and all 5 shipyard ports have a trainer (computed from the features lines).

<!-- hand-maintained:begin -->
| Level | Name | Silver | Days | Effect text (data) | Cite |
|---|---|---|---|---|---|
| 1 | Apprentice | 50 | 3 | "Basic maintenance. Reduce maintenance costs by 25%. Degradation slowed by 20%." | C:5374-5378 |
| 2 | Journeyman | 150 | 5 | "Field repairs at sea. Reduce maintenance costs by 50%. Degradation slowed by 40%. Can restore worn weapons to standard at sea." | C:5381-5385 |
| 3 | Master | 400 | 8 | "Master craftsman. Maintenance costs halved. Degradation slowed by 60%. Smith upgrades cost 25% less. Field repairs restore to standard." | C:5388-5392 |
<!-- hand-maintained:end -->

<!-- hand-maintained:begin -->
| Level | Maintenance discount | Wear slowed by | Upgrade discount | Field repair | Cite |
|---|---|---|---|---|---|
| 1 | 0.25 | 0.2 | 0.0 | no | C:5454-5459 |
| 2 | 0.5 | 0.4 | 0.0 | yes (to standard) | C:5462-5467 |
| 3 | 0.5 | 0.6 | 0.25 | yes (to standard) | C:5470-5475 |
<!-- hand-maintained:end -->

All five trainers' teaching limits are authored; the total silver for all three levels is 600 and the total time 16 days (C:5374, C:5381, C:5388).

## 3. Which effects are live

- [exists] Wear slowed: live in the sim after a personal fight, if a melee weapon or armour is equipped (session.rs:1913, 1928-1960; skills.rs:117-119). Gear can be bought and equipped on the Godot Crew desk `Armoury` section (`Action::CrewBuyGear` / `crew.gear.<id>`, DF-46 resolved).
- [exists] Maintenance discount: live inside `Session::maintain_weapon`, reachable from the Godot Crew desk (`Action::CrewMaintainGear` / `crew.maintain.<id>`) and the script runner (`maintain`, session.rs; script.rs; DF-39 resolved).
- [exists] Upgrade discount and field repair: `Session::upgrade_weapon` (at shipyard ports, applying `skills::apply_upgrade_discount`) is reachable from the Godot Crew desk (`Action::CrewUpgradeGear` / `crew.upgrade.<id>`) and the script runner (`smith`); `Session::field_repair_weapon` is wired on `Session` and the script runner (`field_repair`, DF-39 resolved).

## 4. Index (5 trainers)

<!-- hand-maintained:begin -->
| Trainer | Port | Region | Bloc | Teaches up to | Cite | File |
|---|---|---|---|---|---|---|
| Forge Master Erik | [Ironhaven](../ports/ironhaven.md) | North Atlantic | [The Iron Pact](../factions/iron_pact.md) | 3 (Master) | C:5400 | [forge_master_erik.md](forge_master_erik.md) |
| Old Vasquez | [Porto Novo](../ports/porto_novo.md) | Mediterranean | [The Exchange Alliance](../factions/exchange_alliance.md) | 2 (Journeyman) | C:5409 | [old_vasquez.md](old_vasquez.md) |
| The Silva Bay Smithy | [Silva Bay](../ports/silva_bay.md) | Mediterranean | [The Exchange Alliance](../factions/exchange_alliance.md) | 3 (Master) | C:5418 | [silva_bay_smiths.md](silva_bay_smiths.md) |
| Rajan the Tinker | [Monsoon Reach](../ports/monsoon_reach.md) | East Indies | [The Free Ports](../factions/free_ports.md) | 2 (Journeyman) | C:5427 | [monsoon_tinker.md](monsoon_tinker.md) |
| Kai the Shaper | [Typhoon Anchorage](../ports/typhoon_anchorage.md) | South Seas | [The Coral Crown](../factions/coral_crown.md) | 1 (Apprentice) | C:5436 | [typhoon_smith.md](typhoon_smith.md) |
<!-- hand-maintained:end -->

The cite is the name line. Rows are in data order. Each port has one trainer, so the desk's '(teaches to N)' equals that trainer's limit.

## 5. How training works

- [exists] The call chain: the Learn button asks to confirm, then calls spend_skill_point, which calls can_learn_skill and learn_skill, and advances the calendar one day at a time (game.rs:5627-5650, 5716-5735; session.rs:3380-3405; skills.rs:32-100).
- [exists] can_learn_skill refuses, in order: an unknown skill; the maximum level reached; a port missing `skill.training_port_feature` ('No shipyard at this port.'); no trainer at the port; a teaching limit below the next level ('<trainer> can only teach up to level N. Find a more skilled trainer.'); too little silver (skills.rs:32-95; DF-39 resolved). With several trainers at a port the best limit counts (skills.rs:57-72); no port has two today.
- [exists] The Godot desk greys the button by the same limit: it is enabled only when the next level exists and is at or below the highest limit at the port (logic.rs:2097).
- [exists] Python is the same: the same checks, the same costs, and the trainer's dialog is printed for each trainer at the port (portlight@9b02494 src/portlight/engine/skill_engine.py:30-60; src/portlight/app/cli.py:2733-2780).

## 6. Canon conventions

- Names are the catalog names ('The Silva Bay Smithy' is a group; use the catalog name). Ids appear only in cite tables and file names.
- Quote every data line with its C:n; mark every addition [invented]; put contradictions in data-fixes.md and never fix the data in a canon PR.
- Three words read alike and are different things: the level called Master (C:5388), the title of the merchant Master Chen at Jade Port (C:6004-6006), and the 'master' of a fighting style (style masters index). Resolved in lore (data-fixes).

## F. Flags

- DF-39 (resolved): trainer description/dialog and level description reach the Godot Crew desk; `skill.training_port_feature` is checked in `can_learn_skill`; `Session::maintain_weapon`, `Session::upgrade_weapon`, and `Session::field_repair_weapon` wire the Blacksmithing effects. DF-40 (resolved): the skill notice and empty-trainer line now use the catalog name ('Blacksmithing').
- Only one skill exists in the data; canon does not invent a second.

## G. Gaps in the data (computed from content.json at the base commit)

Ports in `ports[*].id` with no skill trainer (no record has that port as `port_id`). Facts only; canon does not say these ports have or lack anyone else.

<!-- hand-maintained:begin -->
| Port | Region | Bloc |
|---|---|---|
| [Al-Manar](../ports/al_manar.md) | Mediterranean | [The Exchange Alliance](../factions/exchange_alliance.md) |
| [Corsair's Rest](../ports/corsairs_rest.md) | Mediterranean | [The Shadow Ports](../factions/shadow_ports.md) |
| [Stormwall](../ports/stormwall.md) | North Atlantic | [The Iron Pact](../factions/iron_pact.md) |
| [Thornport](../ports/thornport.md) | North Atlantic | [The Free Ports](../factions/free_ports.md) |
| [Sun Harbor](../ports/sun_harbor.md) | West Africa | [The Gold Coast Compact](../factions/gold_coast_compact.md) |
| [Palm Cove](../ports/palm_cove.md) | West Africa | [The Gold Coast Compact](../factions/gold_coast_compact.md) |
| [Iron Point](../ports/iron_point.md) | West Africa | [The Gold Coast Compact](../factions/gold_coast_compact.md) |
| [Pearl Shallows](../ports/pearl_shallows.md) | West Africa | [The Gold Coast Compact](../factions/gold_coast_compact.md) |
| [Jade Port](../ports/jade_port.md) | East Indies | [The Silk Circle](../factions/silk_circle.md) |
| [Silk Haven](../ports/silk_haven.md) | East Indies | [The Silk Circle](../factions/silk_circle.md) |
| [Crosswind Isle](../ports/crosswind_isle.md) | East Indies | [The Free Ports](../factions/free_ports.md) |
| [Dragon's Gate](../ports/dragons_gate.md) | East Indies | [The Silk Circle](../factions/silk_circle.md) |
| [Spice Narrows](../ports/spice_narrows.md) | East Indies | [The Shadow Ports](../factions/shadow_ports.md) |
| [Ember Isle](../ports/ember_isle.md) | South Seas | [The Coral Crown](../factions/coral_crown.md) |
| [Coral Throne](../ports/coral_throne.md) | South Seas | [The Coral Crown](../factions/coral_crown.md) |
<!-- hand-maintained:end -->

15 of 20 ports have no skill trainer. Trainers by region (home ports, computed):

<!-- hand-maintained:begin -->
| Region | Ports in the region | Trainers | Cite |
|---|---|---|---|
| Mediterranean | 4 | Old Vasquez, The Silva Bay Smithy | skills.trainers[*].port_id |
| North Atlantic | 3 | Forge Master Erik | skills.trainers[*].port_id |
| West Africa | 4 | none | skills.trainers[*].port_id |
| East Indies | 6 | Rajan the Tinker | skills.trainers[*].port_id |
| South Seas | 3 | Kai the Shaper | skills.trainers[*].port_id |
<!-- hand-maintained:end -->

West Africa has no trainer at all, so a captain cannot learn the skill in that region; the other four regions have one to two. A region with none shows 'none' above.

## H. Invented filler (not in the game)

None written. This section is empty by default. If design review asks for it, it may hold at most one line per gap port, each tagged [invented], saying what a captain might plausibly meet there; no stats, prices or fees, no name that already exists in data, no file per filler, and never linked from a port file as fact.
