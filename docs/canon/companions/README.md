# Companion canon index

Base commit: 515a488 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at 515a488. Python cites read portlight@9b02494 path:line. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Companions (named, hireable) | 10 | C:5542 |
| Companion roles | 5 | C:5480 |
| Morale triggers | 13, all 13 fire (DF-35 resolved) | C:5704; session.rs:2074-2076 |
| Personality modifiers | 4 | C:5836 |
| Ports with a companion | 9 of 20 | companions[*].home_port_id |
| Party size | at most 2 | model.rs:431,439 |
| Morale at hire; departure | 70; 10 or less | companion.rs:102, 157-176 |
| Rust record shape | id, name, role, home port, region, description, personality, hire cost, required standing, greeting, hire dialog, loyalty line, warning line, departure line | content.rs:338-356 |
| Python reference | same 10 records, same strings (compared field by field, no differences) | portlight@9b02494 src/portlight/content/companions.py:100-266 |

Named companions are the authored party members. Officers are generated crew records and have their own index: [officers/README.md](../officers/README.md). Port NPCs, mentors and pirate captains stay in their own files.

## 1. What the player sees today

| What | Reaches the player? | Cite |
|---|---|---|
| One row per companion for hire at the docked port: name, role, silver cost, required regional regard and its region | Yes. The shipped row reads 'Name  Role  N silver  standing N in Region' | logic.rs:2150-2181 (row at 2176) |
| 'No companions for hire at this port.' | Yes, at a port with none or with all hired or departed | logic.rs:1865; game.rs:5932 |
| A confirm line before hiring: 'Recruit Name for N silver.' | Yes, every recruit asks first | logic.rs:1923; game.rs:5652-5667 |
| A notice after hiring: 'Recruited Iron Marta.' | Yes, with the catalog display name (DF-36 resolved) | logic.rs:1948-1953; game.rs:5748 |
| 'Party n/2.' and one row per member: 'Name  Role  morale N' | Yes | logic.rs:2129-2149 |
| 'Name approves. (+N morale)' or 'Name disapproves. (-N morale)' after a duel victory | Yes, built in code, not authored | companion.rs:147-151; recap.rs:1085-1093 |
| 'Name leaves. <departure line>' after a victory | Yes: the only authored companion text a player can read | recap.rs:1085-1093; logic.rs:940-949 |
| Description, greeting, hire dialog, loyalty line, warning line | Loyalty line (morale >= 80) and warning line (morale <= 29) reach the Crew desk (DF-35 resolved); description, greeting, and hire dialog are data only | content.rs:338-356; logic.rs; game.rs |
| Role description and the seven effect numbers | Yes: role description, party summary, and cohesion reach the Crew desk, and all seven effect numbers are live in the sim (DF-34 resolved) | companion.rs:193-280; logic.rs; game.rs |
| A dismiss control | Yes: Session::dismiss_companion and the Crew desk Dismiss button (DF-34 resolved) | session.rs; logic.rs; game.rs |
| A departed companion | Gone for the save: cannot be hired again and is hidden from the desk | companion.rs:66-70; logic.rs:2161 |
| Character art | None: the character hold stands | crew_screen.rs:3-6 |

## 2. Index (10 companions)

<!-- hand-maintained:begin -->
| Companion | Role | Personality | Home port | Region | Hire cost | Required regard | Cite | File |
|---|---|---|---|---|---|---|---|---|
| Iron Marta | Marine | reckless | [Ironhaven](../ports/ironhaven.md) | North Atlantic | 80 | 5 | C:5544 | [iron_marta.md](iron_marta.md) |
| Red Tomas | Marine | pragmatic | [Corsair's Rest](../ports/corsairs_rest.md) | Mediterranean | 60 | 0 | C:5560 | [red_tomas.md](red_tomas.md) |
| Star Lila | Navigator | gentle | [Crosswind Isle](../ports/crosswind_isle.md) | East Indies | 100 | 10 | C:5576 | [star_lila.md](star_lila.md) |
| Old Henry | Navigator | pragmatic | [Thornport](../ports/thornport.md) | North Atlantic | 70 | 5 | C:5592 | [old_henry.md](old_henry.md) |
| Dr. Amara Osei | Surgeon | gentle | [Sun Harbor](../ports/sun_harbor.md) | West Africa | 120 | 10 | C:5608 | [dr_amara.md](dr_amara.md) |
| Patches | Surgeon | reckless | [Typhoon Anchorage](../ports/typhoon_anchorage.md) | South Seas | 50 | 0 | C:5624 | [patches.md](patches.md) |
| Shadow Kai | Smuggler | pragmatic | [Spice Narrows](../ports/spice_narrows.md) | East Indies | 90 | 5 | C:5640 | [shadow_kai.md](shadow_kai.md) |
| Rosa the Fence | Smuggler | pragmatic | [Corsair's Rest](../ports/corsairs_rest.md) | Mediterranean | 70 | 0 | C:5656 | [rosa_the_fence.md](rosa_the_fence.md) |
| Coin Mariam | Quartermaster | lawful | [Al-Manar](../ports/al_manar.md) | Mediterranean | 100 | 10 | C:5672 | [coin_mariam.md](coin_mariam.md) |
| Dockhand Pete | Quartermaster | reckless | [Palm Cove](../ports/palm_cove.md) | West Africa | 40 | 0 | C:5688 | [dockhand_pete.md](dockhand_pete.md) |
<!-- hand-maintained:end -->

The cite is the record's id line; every field line is in the entity file. Rows are in data order.

## 3. Roles (5)

<!-- hand-maintained:begin -->
| Role | Authored effect numbers (non-zero) | Companions | Cite | File |
|---|---|---|---|---|
| Marine | combat damage bonus 1, interception chance 0.25 | Iron Marta, Red Tomas | C:5482 | [roles/marine.md](roles/marine.md) |
| Navigator | speed bonus 0.5, danger reduction 0.03 | Star Lila, Old Henry | C:5494 | [roles/navigator.md](roles/navigator.md) |
| Surgeon | heal rate bonus 1.0 | Dr. Amara Osei, Patches | C:5506 | [roles/surgeon.md](roles/surgeon.md) |
| Smuggler | inspection evasion 0.3 | Shadow Kai, Rosa the Fence | C:5518 | [roles/smuggler.md](roles/smuggler.md) |
| Quartermaster | trade bonus 0.05 | Coin Mariam, Dockhand Pete | C:5530 | [roles/quartermaster.md](roles/quartermaster.md) |
<!-- hand-maintained:end -->

All seven effect fields are summed by `party_combat_bonus` and nothing calls it (companion.rs:193-209; the only call is a test, companion.rs:327). See DF-34.

## 4. Hire flow

1. The captain docks at a port. The crew desk lists the companions whose home port is that port and who are neither in the party nor departed (logic.rs:2150-2181).
2. The captain picks one. The desk asks 'Recruit Name for N silver.' and waits for a confirm (game.rs:5652-5667).
3. The sim checks, in order: right port, not already aboard, not departed, party under 2, silver, regional regard (companion.rs:51-93). Any failure is a plain sentence ('Party is full (max 2 companions).') shown as the notice (game.rs:5739-5754).
4. On success the cost is taken and the companion starts at morale 70 (session.rs:3359-3373; companion.rs:95-108). The notice reads 'Recruited <Name>.' (DF-36 resolved).
5. Role effects, party summary, cohesion, and morale lines take effect immediately (DF-34, DF-35 resolved); greeting and hire dialog remain data only.

## 5. Morale and departure

- Morale runs 0 to 100 and clamps (companion.rs:142). Below 30 a companion's effects scale down by morale/30 (companion.rs:185-191; DF-34 resolved). At 10 or less they leave and cannot return (companion.rs:157-176).
- Every trigger adds the role delta and the personality delta (even when the role base delta is zero; DF-35 resolved); a total of zero does nothing (companion.rs:130-141).
- All 13 triggers fire in Rust across combat, trade, voyage, and arrival hooks (DF-35 resolved).

Triggers (hand-maintained from C:5704-5835; deltas by role in data order: Marine, Navigator, Surgeon, Smuggler, Quartermaster):

<!-- hand-maintained:begin -->
| Trigger | Deltas | Fired by | Cite |
|---|---|---|---|
| combat_won | +5, +1, -1, +0, +0 | Session (DF-35 resolved) | C:5706 |
| combat_lost | -3, -2, -2, -1, -2 | Session (DF-35 resolved) | C:5716 |
| crew_killed | -2, -1, -5, +0, -1 | Session (DF-35 resolved) | C:5726 |
| profitable_trade | +0, +0, +0, +2, +5 | Session (DF-35 resolved) | C:5736 |
| losing_trade | +0, +0, +0, -1, -3 | Session (DF-35 resolved) | C:5746 |
| contraband_trade | -2, -1, -3, +5, -1 | Session (DF-35 resolved) | C:5756 |
| spared_enemy | +2, +1, +5, +0, +0 | session.rs:2074-2076 | C:5766 |
| took_all | +1, -1, -3, +3, +2 | session.rs:2074-2076 | C:5776 |
| fled_combat | -5, +2, +2, +1, +0 | Session (DF-35 resolved) | C:5786 |
| injury_sustained | -1, -2, -1, -2, -2 | Session (DF-35 resolved) | C:5796 |
| long_voyage | -1, +2, -1, -1, -1 | Session (DF-35 resolved) | C:5806 |
| port_arrival | +1, +1, +1, +1, +2 | Session (DF-35 resolved) | C:5816 |
| nemesis_defeated | +8, +3, +2, +3, +3 | Session (DF-35 resolved) | C:5826 |
<!-- hand-maintained:end -->

Personality modifiers (added to the role delta, only where the role delta is not zero):

<!-- hand-maintained:begin -->
| Personality | Deltas | Cite |
|---|---|---|
| lawful | contraband_trade -3, spared_enemy +2, took_all -2 | C:5838 |
| pragmatic | none | C:5846 |
| reckless | combat_won +2, fled_combat -3, long_voyage -2 | C:5850 |
| gentle | crew_killed -3, spared_enemy +3, took_all -3, combat_won -1 | C:5858 |
<!-- hand-maintained:end -->

What the two live triggers do to each companion today [inferred from companion.rs:124-176; no test was run]:

<!-- hand-maintained:begin -->
| Companion | Spared enemy | Took all | Take-all victories in a row from 70 to departure |
|---|---|---|---|
| Iron Marta | +2 | +1 | never |
| Red Tomas | +2 | +1 | never |
| Star Lila | +4 | -4 | 15 |
| Old Henry | +1 | -1 | 60 |
| Dr. Amara Osei | +8 | -6 | 10 |
| Patches | +5 | -3 | 20 |
| Shadow Kai | +0 | +3 | never |
| Rosa the Fence | +0 | +3 | never |
| Coin Mariam | +0 | +0 | never |
| Dockhand Pete | +0 | +2 | never |
<!-- hand-maintained:end -->

The departure path is real for navigators and surgeons only; marines, smugglers and quartermasters gain or hold morale on both live triggers. The test at session.rs:5396-5409 seats a surgeon at morale 0 and checks the departure line.

## 6. Canon conventions

- Names: 'companion' is a named, hireable party member; 'officer' is a generated crew record; a crew 'role' is a hired job. The same four role names (Marine, Navigator, Surgeon, Quartermaster) exist as companion roles and as crew roles; they are separate systems with separate records (resolved in lore, ../data-fixes.md).
- Player words: canon never uses Trust, Standing or Heat as player terms (Option A). The shipped offer row prints 'standing'; it is quoted only in cite columns. Canon says 'regional regard' for the required number.
- Character hold: companions are quoted as the data describes them. A pronoun or a dress detail appears only where a cited data line already says it.
- Real-world words in data are quoted and not extended: 'Javanese' (Star Lila, C:5581).
- Iron Marta's garrison is [invented] canon: Stormwall's, placed at T38 in ../README.md; her file says so and it is in the resolved-in-lore list.

## 7. Links

- Ports: [Al-Manar](../ports/al_manar.md), [Corsair's Rest](../ports/corsairs_rest.md), [Ironhaven](../ports/ironhaven.md), [Thornport](../ports/thornport.md), [Sun Harbor](../ports/sun_harbor.md), [Palm Cove](../ports/palm_cove.md), [Crosswind Isle](../ports/crosswind_isle.md), [Spice Narrows](../ports/spice_narrows.md), [Typhoon Anchorage](../ports/typhoon_anchorage.md)
- Crew system: [officers/README.md](../officers/README.md); captains: [captains/README.md](../captains/README.md); factions: [factions/README.md](../factions/README.md)
- Open data issues: [data-fixes.md](../data-fixes.md) (DF-34 to DF-37)

## F. Flags

- DF-34 (resolved): companion role effects, cohesion, party summary, and dismiss are now wired in sim and Godot. DF-35 (resolved): all 13 morale triggers and personality deltas now fire, and loyalty/warning lines are surfaced on the Crew desk. DF-36 (resolved): the recruit notice now prints the catalog display name. DF-37 (resolved): same-name collisions with port NPCs disambiguated (Dr. Amara Osei).

## G. Gaps in the data (computed from content.json at the base commit)

Ports in `ports[*].id` with no companion (no record has that port as `home_port_id`). Facts only; canon does not say these ports have or lack anyone else.

<!-- hand-maintained:begin -->
| Port | Region | Bloc |
|---|---|---|
| [Porto Novo](../ports/porto_novo.md) | Mediterranean | [The Exchange Alliance](../factions/exchange_alliance.md) |
| [Silva Bay](../ports/silva_bay.md) | Mediterranean | [The Exchange Alliance](../factions/exchange_alliance.md) |
| [Stormwall](../ports/stormwall.md) | North Atlantic | [The Iron Pact](../factions/iron_pact.md) |
| [Iron Point](../ports/iron_point.md) | West Africa | [The Gold Coast Compact](../factions/gold_coast_compact.md) |
| [Pearl Shallows](../ports/pearl_shallows.md) | West Africa | [The Gold Coast Compact](../factions/gold_coast_compact.md) |
| [Jade Port](../ports/jade_port.md) | East Indies | [The Silk Circle](../factions/silk_circle.md) |
| [Monsoon Reach](../ports/monsoon_reach.md) | East Indies | [The Free Ports](../factions/free_ports.md) |
| [Silk Haven](../ports/silk_haven.md) | East Indies | [The Silk Circle](../factions/silk_circle.md) |
| [Dragon's Gate](../ports/dragons_gate.md) | East Indies | [The Silk Circle](../factions/silk_circle.md) |
| [Ember Isle](../ports/ember_isle.md) | South Seas | [The Coral Crown](../factions/coral_crown.md) |
| [Coral Throne](../ports/coral_throne.md) | South Seas | [The Coral Crown](../factions/coral_crown.md) |
<!-- hand-maintained:end -->

11 of 20 ports have no companion. Companions by region (home ports, computed):

<!-- hand-maintained:begin -->
| Region | Companions | Cite |
|---|---|---|
| Mediterranean | Red Tomas, Rosa the Fence, Coin Mariam | C:5542 |
| North Atlantic | Iron Marta, Old Henry | C:5542 |
| West Africa | Dr. Amara Osei, Dockhand Pete | C:5542 |
| East Indies | Star Lila, Shadow Kai | C:5542 |
| South Seas | Patches | C:5542 |
<!-- hand-maintained:end -->

Corsair's Rest has two companions; every other home port has one. Each region listed has at least one companion; a region with none would show 'none' above.

## H. Invented filler (not in the game)

None written. This section is empty by default. If design review asks for it, it may hold at most one line per gap port, each tagged [invented], saying what a captain might plausibly meet there; no stats, prices or hire cost, no name that already exists in data, no file per filler, and never linked from a port file as fact.
