# The Scholar - player type (scholar)

Base commit: c413788 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 5. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n; `S/` is crates/portlight-sim/src/, `G/` is crates/portlight-godot/src/. Legend: [exists file:line] in the sim today | [proposal; planned slice PT-Sn] intended, not shipped | [invented] lore | [flavour, no mechanic] | DF-n see ../data-fixes.md.
Who this captain is: [../captains/scholar.md](../captains/scholar.md). This file is how the type plays.

## 1. Identity and fantasy

A Wind Temple student of Monsoon Reach; knowledge is the cargo. [invented] where not data.

The player's verb: "I read the wind and the world" [invented]

## 2. Unique mechanics and perks

Starting values (all [exists], cited):

| Key | Value | Tag |
|---|---|---|
| Silver - ship - provisions | 400 - Coastal Sloop (sloop) - 35 | [exists C:2517-2519] |
| Trade terms | buy x1.0 - sell x1.0 - luxury bonus 0.0 - port fees x0.85 | [exists C:2520-2524] |
| Voyage terms | provision burn x0.75 - speed +1.0 - storm resist +0.1 - cargo damage x0.8 | [exists C:2526-2530] |
| Inspection terms | chance x0.7 - seizure 0.0 - fines x0.8 | [exists C:2532-2535] |
| Starting regard (data keys, not player terms) | commercial_trust 5, mediterranean 2, north_atlantic 2, west_africa 2, east_indies 8, south_seas 2 | [exists C:2537; S/model.rs:255-270] |
| Pirate regard | none | [exists C:2537] |
| Contract bias (x2 offer weight) | none today | [exists crates/portlight-sim/src/contracts.rs:75-80] |
| captain_type branch | no captain_type branch; differs by data only | [exists custom_captain.rs:432-442 (catalog only)] |

Perks:

- Speed +1.0, storm resist +0.1, provision burn x0.75 [exists C:2527-2529]
- Regard 2 in every region and 8 in the East Indies: no cold start anywhere [exists C:2537]
- Inspection x0.7, port fees x0.85 [exists C:2524, C:2533]
- Not the best provisioner: the Navigator burns x0.7 (DF-29) [exists C:2409]
- Circuit and medicine contract bias [proposal; planned slice PT-S1]
- Route insight: departure check shows lane danger or season hints one tier earlier; copy only, layout does not shift. Trade-off: no trade edge [proposal; planned slice PT-S5]

## 3. How the 20 ports treat this type

Basis: [exists] means a real effect of this type's numbers (regional regard, pirate regard, fees, inspection, home or mentor); [invented] is flavour only and implies no mechanic. Fees and inspection terms apply at every port alike (C:2520-2535); no port has a per-type rule.

| Port | Bloc | How they treat this type | Basis |
|---|---|---|---|
| [Porto Novo](../ports/porto_novo.md) | [The Exchange Alliance](../factions/exchange_alliance.md) | starts with warm Mediterranean regard (2); Alliance scholars trade notes | [exists C:2537] [invented] |
| [Al-Manar](../ports/al_manar.md) | [The Exchange Alliance](../factions/exchange_alliance.md) | starts with warm Mediterranean regard (2); Alliance scholars trade notes | [exists C:2537] [invented] |
| [Silva Bay](../ports/silva_bay.md) | [The Exchange Alliance](../factions/exchange_alliance.md) | starts with warm Mediterranean regard (2); Alliance scholars trade notes | [exists C:2537] [invented] |
| [Corsair's Rest](../ports/corsairs_rest.md) | [The Shadow Ports](../factions/shadow_ports.md) | starts with warm Mediterranean regard (2); Shadow Ports sell secrets to a student | [exists C:2537] [invented] |
| [Ironhaven](../ports/ironhaven.md) | [The Iron Pact](../factions/iron_pact.md) | starts with warm North Atlantic regard (2); Pact engineers talk shop | [exists C:2537] [invented] |
| [Stormwall](../ports/stormwall.md) | [The Iron Pact](../factions/iron_pact.md) | starts with warm North Atlantic regard (2); Pact engineers talk shop | [exists C:2537] [invented] |
| [Thornport](../ports/thornport.md) | [The Free Ports](../factions/free_ports.md) | starts with warm North Atlantic regard (2); home bloc; Brother Anand | [exists C:2537] [invented] |
| [Sun Harbor](../ports/sun_harbor.md) | [The Gold Coast Compact](../factions/gold_coast_compact.md) | starts with warm West Africa regard (2); Compact elders share lore | [exists C:2537] [invented] |
| [Palm Cove](../ports/palm_cove.md) | [The Gold Coast Compact](../factions/gold_coast_compact.md) | starts with warm West Africa regard (2); Compact elders share lore | [exists C:2537] [invented] |
| [Iron Point](../ports/iron_point.md) | [The Gold Coast Compact](../factions/gold_coast_compact.md) | starts with warm West Africa regard (2); Compact elders share lore | [exists C:2537] [invented] |
| [Pearl Shallows](../ports/pearl_shallows.md) | [The Gold Coast Compact](../factions/gold_coast_compact.md) | starts with warm West Africa regard (2); Compact elders share lore | [exists C:2537] [invented] |
| [Jade Port](../ports/jade_port.md) | [The Silk Circle](../factions/silk_circle.md) | starts with warm East Indies regard (8); Silk Circle temples respect a student | [exists C:2537] [invented] |
| [Monsoon Reach](../ports/monsoon_reach.md) | [The Free Ports](../factions/free_ports.md) | home port; the mentor is here; starts with warm East Indies regard (8); home bloc; Brother Anand | [exists C:2515-2516] [exists C:2537] [invented] |
| [Silk Haven](../ports/silk_haven.md) | [The Silk Circle](../factions/silk_circle.md) | starts with warm East Indies regard (8); Silk Circle temples respect a student | [exists C:2537] [invented] |
| [Crosswind Isle](../ports/crosswind_isle.md) | [The Free Ports](../factions/free_ports.md) | starts with warm East Indies regard (8); home bloc; Brother Anand | [exists C:2537] [invented] |
| [Dragon's Gate](../ports/dragons_gate.md) | [The Silk Circle](../factions/silk_circle.md) | starts with warm East Indies regard (8); Silk Circle temples respect a student | [exists C:2537] [invented] |
| [Spice Narrows](../ports/spice_narrows.md) | [The Shadow Ports](../factions/shadow_ports.md) | starts with warm East Indies regard (8); Shadow Ports sell secrets to a student | [exists C:2537] [invented] |
| [Ember Isle](../ports/ember_isle.md) | [The Coral Crown](../factions/coral_crown.md) | starts with warm South Seas regard (2); Coral Crown keepers test a student's manners | [exists C:2537] [invented] |
| [Typhoon Anchorage](../ports/typhoon_anchorage.md) | [The Coral Crown](../factions/coral_crown.md) | starts with warm South Seas regard (2); Coral Crown keepers test a student's manners | [exists C:2537] [invented] |
| [Coral Throne](../ports/coral_throne.md) | [The Coral Crown](../factions/coral_crown.md) | starts with warm South Seas regard (2); Coral Crown keepers test a student's manners | [exists C:2537] [invented] |

## 4. Reputation and faction effects

- Starting regard is copied from the type's data into the standing at new game (S/model.rs:255-270). Regional customs inspection rises with heat in a region (S/reputation.rs:216-227). Pirate hostility reads pirate regard (S/encounter.rs:42-62).
- Starts 2 in four regions, 8 in the East Indies, trust 5 (C:2537)
- No pirate regard
- No spillover: regard with one bloc or faction does not move its rivals (DF-28). Canon claims none.

## 5. Progression

Uses existing systems: milestone and career scoring (S/campaign.rs:1245-1310), the ship ladder (C:131 ships) and contract offers (S/contracts.rs:75-80).

- Early: Short East Indies runs
- Mid: Circuits and medicine relief (PT-S1)
- Late: Every region open; career scoring

## 6. Look and feel (words only)

Words, not art direction. Character plates stay held; canon sets no face, age, gender or dress for the captain (../captains/README.md rules).

- Mood: quiet, cluttered, curious - a cabin that has outgrown its shelves [invented; AD note]
- Materials: crated books under oilcloth, glass specimen jars, a vellum-bound log, a lamp-lit writing desk of walnut [invented; AD note]
- Silhouette: soft and stacked - bundles and rounded piles, no hard edges [invented; AD note]
- Palette: parchment buff, ink-wash grey-blue, walnut brown; mid values, neutral; small accent of a lamp-glow ochre [invented; AD note]
- Notebooks in oilcloth, a monsoon chart [invented]
- A quiet deck [invented]
- The crew learn to read clouds [invented]

## 7. Sound hooks

- sfx_chart_next_day (G/game.rs:4651): a day's notes
- sfx_ui_contracts_open (G/game.rs:6826): reading the board
- New sounds are ideas only: a page turn on a route hint (idea, no asset requested)

## 8. Flags

- DF-29: Python 'best provision efficiency' (C:2527 0.75 vs C:2409 0.7)
