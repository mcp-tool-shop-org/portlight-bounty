# The Dockhand - player type (dockhand)

Base commit: c413788 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 5. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n; `S/` is crates/portlight-sim/src/, `G/` is crates/portlight-godot/src/. Legend: [exists file:line] in the sim today | [proposal; planned slice PT-Sn] intended, not shipped | [invented] lore | [flavour, no mechanic] | DF-n see ../data-fixes.md.
Who this captain is: [../captains/dockhand.md](../captains/dockhand.md). This file is how the type plays.

## 1. Identity and fantasy

Self-made captain from the Crosswind Isle docks; nobody owes you anything. [invented] where not data.

The player's verb: "I earn every plank" [invented]

## 2. Unique mechanics and perks

Starting values (all [exists], cited):

| Key | Value | Tag |
|---|---|---|
| Silver - ship - provisions | 300 - Coastal Sloop (sloop) - 30 | [exists C:2593-2595] |
| Trade terms | buy x1.0 - sell x1.0 - luxury bonus 0.0 - port fees x1.0 | [exists C:2596-2600] |
| Voyage terms | provision burn x0.85 - speed +0.0 - storm resist +0.0 - cargo damage x0.9 | [exists C:2602-2606] |
| Inspection terms | chance x1.0 - seizure 0.0 - fines x1.0 | [exists C:2608-2611] |
| Starting regard (data keys, not player terms) | none | [exists C:2613; S/model.rs:255-270] |
| Pirate regard | none | [exists C:2613] |
| Contract bias (x2 offer weight) | none today | [exists crates/portlight-sim/src/contracts.rs:75-80] |
| captain_type branch | no captain_type branch; differs by data only | [exists custom_captain.rs:432-442 (catalog only)] |

Perks:

- Silver 300, the least of any type [exists C:2593]
- No regard, no heat, no pirate regard: neutral everywhere [exists C:2613]
- Provision burn x0.85, cargo damage x0.9 [exists C:2603, C:2606]
- Procurement and shortage contract bias [proposal; planned slice PT-S1]
- Self-made growth: one scaling perk (crew wage or repair cost cut after N voyages). Must reach the PT-S0 band by day 60 and not exceed it later [proposal; planned slice PT-S6]

## 3. How the 20 ports treat this type

Basis: [exists] means a real effect of this type's numbers (regional regard, pirate regard, fees, inspection, home or mentor); [invented] is flavour only and implies no mechanic. Fees and inspection terms apply at every port alike (C:2596-2611); no port has a per-type rule.

| Port | Bloc | How they treat this type | Basis |
|---|---|---|---|
| [Porto Novo](../ports/porto_novo.md) | [The Exchange Alliance](../factions/exchange_alliance.md) | Alliance clerks see no one | [invented] |
| [Al-Manar](../ports/al_manar.md) | [The Exchange Alliance](../factions/exchange_alliance.md) | Alliance clerks see no one | [invented] |
| [Silva Bay](../ports/silva_bay.md) | [The Exchange Alliance](../factions/exchange_alliance.md) | Alliance clerks see no one | [invented] |
| [Corsair's Rest](../ports/corsairs_rest.md) | [The Shadow Ports](../factions/shadow_ports.md) | Shadow Ports recruit the unattached | [invented] |
| [Ironhaven](../ports/ironhaven.md) | [The Iron Pact](../factions/iron_pact.md) | Pact docks hire hands | [invented] |
| [Stormwall](../ports/stormwall.md) | [The Iron Pact](../factions/iron_pact.md) | Pact docks hire hands | [invented] |
| [Thornport](../ports/thornport.md) | [The Free Ports](../factions/free_ports.md) | home bloc; Dock Master Tao | [invented] |
| [Sun Harbor](../ports/sun_harbor.md) | [The Gold Coast Compact](../factions/gold_coast_compact.md) | Compact docks are kind to workers | [invented] |
| [Palm Cove](../ports/palm_cove.md) | [The Gold Coast Compact](../factions/gold_coast_compact.md) | Compact docks are kind to workers | [invented] |
| [Iron Point](../ports/iron_point.md) | [The Gold Coast Compact](../factions/gold_coast_compact.md) | Compact docks are kind to workers | [invented] |
| [Pearl Shallows](../ports/pearl_shallows.md) | [The Gold Coast Compact](../factions/gold_coast_compact.md) | Compact docks are kind to workers | [invented] |
| [Jade Port](../ports/jade_port.md) | [The Silk Circle](../factions/silk_circle.md) | Silk Circle guilds ask for papers | [invented] |
| [Monsoon Reach](../ports/monsoon_reach.md) | [The Free Ports](../factions/free_ports.md) | home bloc; Dock Master Tao | [invented] |
| [Silk Haven](../ports/silk_haven.md) | [The Silk Circle](../factions/silk_circle.md) | Silk Circle guilds ask for papers | [invented] |
| [Crosswind Isle](../ports/crosswind_isle.md) | [The Free Ports](../factions/free_ports.md) | home port; the mentor is here; home bloc; Dock Master Tao | [exists C:2591-2592] [invented] |
| [Dragon's Gate](../ports/dragons_gate.md) | [The Silk Circle](../factions/silk_circle.md) | Silk Circle guilds ask for papers | [invented] |
| [Spice Narrows](../ports/spice_narrows.md) | [The Shadow Ports](../factions/shadow_ports.md) | Shadow Ports recruit the unattached | [invented] |
| [Ember Isle](../ports/ember_isle.md) | [The Coral Crown](../factions/coral_crown.md) | Coral Crown courts ignore the unknown | [invented] |
| [Typhoon Anchorage](../ports/typhoon_anchorage.md) | [The Coral Crown](../factions/coral_crown.md) | Coral Crown courts ignore the unknown | [invented] |
| [Coral Throne](../ports/coral_throne.md) | [The Coral Crown](../factions/coral_crown.md) | Coral Crown courts ignore the unknown | [invented] |

## 4. Reputation and faction effects

- Starting regard is copied from the type's data into the standing at new game (S/model.rs:255-270). Regional customs inspection rises with heat in a region (S/reputation.rs:216-227). Pirate hostility reads pirate regard (S/encounter.rs:42-62).
- All zero (C:2613); every faction neutral (S/encounter.rs:42-62)
- No spillover: regard with one bloc or faction does not move its rivals (DF-28). Canon claims none.

## 5. Progression

Uses existing systems: milestone and career scoring (S/campaign.rs:1245-1310), the ship ladder (C:131 ships) and contract offers (S/contracts.rs:75-80).

- Early: Cheap procurement runs
- Mid: PT-S6 perk kicks in
- Late: Climb the ship ladder

## 6. Look and feel (words only)

Words, not art direction. Character plates stay held; canon sets no face, age, gender or dress for the captain (../captains/README.md rules).

- Mood: plain, hard-working, unfussy - everything chosen because it lasts [invented; AD note]
- Materials: rough-sawn pine, iron hooks and cargo slings, a mallet and caulking iron, frayed working rope [invented; AD note]
- Silhouette: compact and blocky - short and sturdy, working gear hanging off the rail [invented; AD note]
- Palette: dust grey, bleached pine, mud-tan canvas; flat mid values, neutral-warm, the lowest contrast of the nine; small accent of a chalked tally mark [invented; AD note]
- A patched deck [invented]
- Cabin: a hammock and a cash tin [invented]
- The crew are old dock friends [invented]

## 7. Sound hooks

- sfx_ui_newgame_start (G/game.rs:2111): a humble start
- sfx_chart_sail_depart (G/game.rs:4529): first voyage
- New sounds are ideas only: a hammer tap on a repair (idea, no asset requested)

## 8. Flags

- Shares home and mentor with the Bounty Hunter (inventory finding 4)
