# The Navigator - player type (navigator)

Base commit: c413788 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 5. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n; `S/` is crates/portlight-sim/src/, `G/` is crates/portlight-godot/src/. Legend: [exists file:line] in the sim today | [proposal; planned slice PT-Sn] intended, not shipped | [invented] lore | [flavour, no mechanic] | DF-n see ../data-fixes.md.
Who this captain is: [../captains/navigator.md](../captains/navigator.md). This file is how the type plays.

## 1. Identity and fantasy

The route specialist of Silva Bay. Fast, frugal and storm-wise; the fantasy is the captain whose charts others copy. [invented] where not data.

The player's verb: "I go where the long haul pays" [invented]

## 2. Unique mechanics and perks

Starting values (all [exists], cited):

| Key | Value | Tag |
|---|---|---|
| Silver - ship - provisions | 450 - Coastal Sloop (sloop) - 30 | [exists C:2399-2401] |
| Trade terms | buy x1.05 - sell x1.0 - luxury bonus 0.0 - port fees x1.0 | [exists C:2402-2406] |
| Voyage terms | provision burn x0.7 - speed +1.5 - storm resist +0.15 - cargo damage x0.7 | [exists C:2408-2412] |
| Inspection terms | chance x1.0 - seizure 0.0 - fines x1.0 | [exists C:2414-2417] |
| Starting regard (data keys, not player terms) | commercial_trust 5, mediterranean 5, east_indies 5 | [exists C:2419; S/model.rs:255-270] |
| Pirate regard | none | [exists C:2419] |
| Contract bias (x2 offer weight) | proc_timber_shipyard, proc_medicines_relief, proc_weapons_garrison, short_tea_crisis, ret_cotton_return, ret_spice_restock, circ_med_africa, circ_indies_loop, circ_south_seas, rep_charter_atlantic | [exists crates/portlight-sim/src/contracts.rs:75-80] |
| captain_type branch | star-navigation event runs at speed 1.2 with the navigator's own line | [exists S/voyage.rs:295-300] |

Perks:

- Provision burn x0.7, the lowest of any type [exists C:2409]
- Speed +1.5 and storm resist +0.15 [exists C:2410-2411]
- Cargo damage x0.7 [exists C:2412]
- Unique star-navigation line (S/voyage.rs:295-300) [exists S/voyage.rs:295-300]
- Circuit, return and charter contract bias (10 templates) [exists S/contracts.rs:75-80]
- Trade-off: buy x1.05 [exists C:2403]
- No new perk planned; revisit only if PT-S0 says so [proposal; planned slice PT-S0]

## 3. How the 20 ports treat this type

Basis: [exists] means a real effect of this type's numbers (regional regard, pirate regard, fees, inspection, home or mentor); [invented] is flavour only and implies no mechanic. Fees and inspection terms apply at every port alike (C:2402-2417); no port has a per-type rule.

| Port | Bloc | How they treat this type | Basis |
|---|---|---|---|
| [Porto Novo](../ports/porto_novo.md) | [The Exchange Alliance](../factions/exchange_alliance.md) | starts with warm Mediterranean regard (5); home bloc; Silva Bay shipwrights respect a sailor | [exists C:2419] [invented] |
| [Al-Manar](../ports/al_manar.md) | [The Exchange Alliance](../factions/exchange_alliance.md) | starts with warm Mediterranean regard (5); home bloc; Silva Bay shipwrights respect a sailor | [exists C:2419] [invented] |
| [Silva Bay](../ports/silva_bay.md) | [The Exchange Alliance](../factions/exchange_alliance.md) | home port; the mentor is here; starts with warm Mediterranean regard (5); home bloc; Silva Bay shipwrights respect a sailor | [exists C:2397-2398] [exists C:2419] [invented] |
| [Corsair's Rest](../ports/corsairs_rest.md) | [The Shadow Ports](../factions/shadow_ports.md) | starts with warm Mediterranean regard (5); Shadow Ports want the navigator's charts | [exists C:2419] [invented] |
| [Ironhaven](../ports/ironhaven.md) | [The Iron Pact](../factions/iron_pact.md) | Pact pilots trade weather talk | [invented] |
| [Stormwall](../ports/stormwall.md) | [The Iron Pact](../factions/iron_pact.md) | Pact pilots trade weather talk | [invented] |
| [Thornport](../ports/thornport.md) | [The Free Ports](../factions/free_ports.md) | Free Ports are waypoints | [invented] |
| [Sun Harbor](../ports/sun_harbor.md) | [The Gold Coast Compact](../factions/gold_coast_compact.md) | Compact ports welcome a fast hull | [invented] |
| [Palm Cove](../ports/palm_cove.md) | [The Gold Coast Compact](../factions/gold_coast_compact.md) | Compact ports welcome a fast hull | [invented] |
| [Iron Point](../ports/iron_point.md) | [The Gold Coast Compact](../factions/gold_coast_compact.md) | Compact ports welcome a fast hull | [invented] |
| [Pearl Shallows](../ports/pearl_shallows.md) | [The Gold Coast Compact](../factions/gold_coast_compact.md) | Compact ports welcome a fast hull | [invented] |
| [Jade Port](../ports/jade_port.md) | [The Silk Circle](../factions/silk_circle.md) | starts with warm East Indies regard (5); Silk Circle values the long road east | [exists C:2419] [invented] |
| [Monsoon Reach](../ports/monsoon_reach.md) | [The Free Ports](../factions/free_ports.md) | starts with warm East Indies regard (5); Free Ports are waypoints | [exists C:2419] [invented] |
| [Silk Haven](../ports/silk_haven.md) | [The Silk Circle](../factions/silk_circle.md) | starts with warm East Indies regard (5); Silk Circle values the long road east | [exists C:2419] [invented] |
| [Crosswind Isle](../ports/crosswind_isle.md) | [The Free Ports](../factions/free_ports.md) | starts with warm East Indies regard (5); Free Ports are waypoints | [exists C:2419] [invented] |
| [Dragon's Gate](../ports/dragons_gate.md) | [The Silk Circle](../factions/silk_circle.md) | starts with warm East Indies regard (5); Silk Circle values the long road east | [exists C:2419] [invented] |
| [Spice Narrows](../ports/spice_narrows.md) | [The Shadow Ports](../factions/shadow_ports.md) | starts with warm East Indies regard (5); Shadow Ports want the navigator's charts | [exists C:2419] [invented] |
| [Ember Isle](../ports/ember_isle.md) | [The Coral Crown](../factions/coral_crown.md) | Coral Crown pilots test a stranger's seamanship | [invented] |
| [Typhoon Anchorage](../ports/typhoon_anchorage.md) | [The Coral Crown](../factions/coral_crown.md) | Coral Crown pilots test a stranger's seamanship | [invented] |
| [Coral Throne](../ports/coral_throne.md) | [The Coral Crown](../factions/coral_crown.md) | Coral Crown pilots test a stranger's seamanship | [invented] |

## 4. Reputation and faction effects

- Starting regard is copied from the type's data into the standing at new game (S/model.rs:255-270). Regional customs inspection rises with heat in a region (S/reputation.rs:216-227). Pirate hostility reads pirate regard (S/encounter.rs:42-62).
- Starts Mediterranean 5, East Indies 5, commercial trust 5 (C:2419)
- No pirate regard (S/encounter.rs:42-62)
- No spillover: regard with one bloc or faction does not move its rivals (DF-28). Canon claims none.

## 5. Progression

Uses existing systems: milestone and career scoring (S/campaign.rs:1245-1310), the ship ladder (C:131 ships) and contract offers (S/contracts.rs:75-80).

- Early: Short Med runs to bank days saved
- Mid: Circuits east (circuit contracts)
- Late: Galleon lanes and long charters

## 6. Look and feel (words only)

Words, not art direction. Character plates stay held; canon sets no face, age, gender or dress for the captain (../captains/README.md rules).

- Mood: calm, airy, precise - a deck that feels measured [invented; look note]
- Materials: brass dividers and sextant, rolled charts in leather tubes, a chart table of pale ash, a log line on a reel [invented; look note]
- Silhouette: tall and narrow - a raised quarterdeck, upright verticals [invented; look note]
- Palette: salt-faded indigo, sea-mist grey, pale ash; upper-mid values, cool; small accent of compass-needle verdigris [invented; look note]
- Charts pinned under glass, wind notes in the margins [invented]
- A clean, light deck built for speed [invented]
- The crew talk in bearings and days [invented]

## 7. Sound hooks

- sfx_chart_sail_depart (G/game.rs:4529): the type's signature departure
- sfx_chart_next_day (G/game.rs:4651): days gained at sea
- New sounds are ideas only: a sextant click on the star line (idea, no asset requested)

## 8. Flags

- None.
