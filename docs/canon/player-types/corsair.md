# The Corsair - player type (corsair)

Base commit: c413788 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 5. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n; `S/` is crates/portlight-sim/src/, `G/` is crates/portlight-godot/src/. Legend: [exists file:line] in the sim today | [proposal; planned slice PT-Sn] intended, not shipped | [invented] lore | [flavour, no mechanic] | DF-n see ../data-fixes.md.
Who this captain is: [../captains/corsair.md](../captains/corsair.md). This file is how the type plays.

## 1. Identity and fantasy

Also called 'Pirate'. A Crimson Tide captain out of Corsair's Rest; the cove is home, the lawful world is not. [invented] where not data.

The player's verb: "I fly the crimson pennant" [invented]

## 2. Unique mechanics and perks

Starting values (all [exists], cited):

| Key | Value | Tag |
|---|---|---|
| Silver - ship - provisions | 500 - Coastal Sloop (sloop) - 30 | [exists C:2477-2479] |
| Trade terms | buy x1.05 - sell x0.95 - luxury bonus 0.15 - port fees x0.5 | [exists C:2480-2484] |
| Voyage terms | provision burn x0.9 - speed +0.3 - storm resist +0.0 - cargo damage x0.85 | [exists C:2486-2490] |
| Inspection terms | chance x1.5 - seizure 0.08 - fines x1.5 | [exists C:2492-2495] |
| Starting regard (data keys, not player terms) | customs_heat 12, mediterranean -5 | [exists C:2497; S/model.rs:255-270] |
| Pirate regard | crimson_tide 25 | [exists C:2497] |
| Contract bias (x2 offer weight) | smug_opium_run, smug_powder_delivery, smug_fence_the_take, smug_faction_supply | [exists crates/portlight-sim/src/contracts.rs:75-80] |
| captain_type branch | no captain_type branch; differs by data only (smuggling contract bias is data) | [exists S/contracts.rs:75-80] |

Perks:

- Crimson Tide regard 25: the Tide is friendly in encounters [exists C:2497; S/encounter.rs:42-62]
- Port fees x0.5, the lowest of any type [exists C:2484]
- Luxury sell bonus 0.15 [exists C:2483]
- Smuggling contract bias (4 templates) [exists S/contracts.rs:75-80]
- Pirate-port welcome: a reduced port fee at Crimson Tide aligned ports. Trade-off: the existing inspection x1.5 and seizure 0.08 stay [proposal; planned slice PT-S4]
- Trade-off today: inspection x1.5, seizure 0.08, fines x1.5, heat 12, Mediterranean -5 [exists C:2493-2495, C:2497]

## 3. How the 20 ports treat this type

Basis: [exists] means a real effect of this type's numbers (regional regard, pirate regard, fees, inspection, home or mentor); [invented] is flavour only and implies no mechanic. Fees and inspection terms apply at every port alike (C:2480-2495); no port has a per-type rule.

| Port | Bloc | How they treat this type | Basis |
|---|---|---|---|
| [Porto Novo](../ports/porto_novo.md) | [The Exchange Alliance](../factions/exchange_alliance.md) | starts with cool Mediterranean regard (-5); the Alliance shuts its doors in lore; the numbers are the cool Med regard and high inspection | [exists C:2497] [invented] |
| [Al-Manar](../ports/al_manar.md) | [The Exchange Alliance](../factions/exchange_alliance.md) | starts with cool Mediterranean regard (-5); the Alliance shuts its doors in lore; the numbers are the cool Med regard and high inspection | [exists C:2497] [invented] |
| [Silva Bay](../ports/silva_bay.md) | [The Exchange Alliance](../factions/exchange_alliance.md) | starts with cool Mediterranean regard (-5); the Alliance shuts its doors in lore; the numbers are the cool Med regard and high inspection | [exists C:2497] [invented] |
| [Corsair's Rest](../ports/corsairs_rest.md) | [The Shadow Ports](../factions/shadow_ports.md) | home port; the mentor is here; starts with cool Mediterranean regard (-5); Shadow Ports are home water; Crimson Tide aligned: target of the PT-S4 fee cut | [exists C:2475-2476] [exists C:2497] [invented] [proposal; planned slice PT-S4] |
| [Ironhaven](../ports/ironhaven.md) | [The Iron Pact](../factions/iron_pact.md) | Pact customs search a crimson hull | [invented] |
| [Stormwall](../ports/stormwall.md) | [The Iron Pact](../factions/iron_pact.md) | Pact customs search a crimson hull | [invented] |
| [Thornport](../ports/thornport.md) | [The Free Ports](../factions/free_ports.md) | Free Ports sell to anyone | [invented] |
| [Sun Harbor](../ports/sun_harbor.md) | [The Gold Coast Compact](../factions/gold_coast_compact.md) | the Compact trades cautiously | [invented] |
| [Palm Cove](../ports/palm_cove.md) | [The Gold Coast Compact](../factions/gold_coast_compact.md) | the Compact trades cautiously | [invented] |
| [Iron Point](../ports/iron_point.md) | [The Gold Coast Compact](../factions/gold_coast_compact.md) | the Compact trades cautiously | [invented] |
| [Pearl Shallows](../ports/pearl_shallows.md) | [The Gold Coast Compact](../factions/gold_coast_compact.md) | the Compact trades cautiously | [invented] |
| [Jade Port](../ports/jade_port.md) | [The Silk Circle](../factions/silk_circle.md) | Silk Circle inspectors are thorough | [invented] |
| [Monsoon Reach](../ports/monsoon_reach.md) | [The Free Ports](../factions/free_ports.md) | Free Ports sell to anyone | [invented] |
| [Silk Haven](../ports/silk_haven.md) | [The Silk Circle](../factions/silk_circle.md) | Silk Circle inspectors are thorough | [invented] |
| [Crosswind Isle](../ports/crosswind_isle.md) | [The Free Ports](../factions/free_ports.md) | Free Ports sell to anyone | [invented] |
| [Dragon's Gate](../ports/dragons_gate.md) | [The Silk Circle](../factions/silk_circle.md) | Silk Circle inspectors are thorough | [invented] |
| [Spice Narrows](../ports/spice_narrows.md) | [The Shadow Ports](../factions/shadow_ports.md) | Shadow Ports are home water; Crimson Tide aligned: target of the PT-S4 fee cut | [invented] [proposal; planned slice PT-S4] |
| [Ember Isle](../ports/ember_isle.md) | [The Coral Crown](../factions/coral_crown.md) | Coral Crown courts trade, warily | [invented] |
| [Typhoon Anchorage](../ports/typhoon_anchorage.md) | [The Coral Crown](../factions/coral_crown.md) | Coral Crown courts trade, warily | [invented] |
| [Coral Throne](../ports/coral_throne.md) | [The Coral Crown](../factions/coral_crown.md) | Coral Crown courts trade, warily | [invented] |

## 4. Reputation and faction effects

- Starting regard is copied from the type's data into the standing at new game (S/model.rs:255-270). Regional customs inspection rises with heat in a region (S/reputation.rs:216-227). Pirate hostility reads pirate regard (S/encounter.rs:42-62).
- Starts Mediterranean -5, heat 12 everywhere, Crimson Tide 25 (C:2497)
- Heat 12 is under the first inspection step at 15 (S/reputation.rs:216-227)
- No Deep Reef start regard [exists C:2497]
- No spillover: regard with one bloc or faction does not move its rivals (DF-28). Canon claims none.

## 5. Progression

Uses existing systems: milestone and career scoring (S/campaign.rs:1245-1310), the ship ladder (C:131 ships) and contract offers (S/contracts.rs:75-80).

- Early: Smuggling runs out of Corsair's Rest
- Mid: Faction supply and powder jobs
- Late: Shadow-path career

## 6. Look and feel (words only)

Words, not art direction. Character plates stay held; canon sets no face, age, gender or dress for the captain (../captains/README.md rules).

- Mood: loud, patched, unruly - a deck that has been through things [invented; look note]
- Materials: patched sailcloth, mismatched salvaged hardware, scarred rail timber, a grapnel and chain left where they fell [invented; look note]
- Silhouette: jagged and irregular - ragged edges, uneven lines, clutter breaking the outline [invented; look note]
- Palette: weathered rust, smoke-stained ochre, blood-brown timber; mid-low values, hot; small accent of a sun-bleached crimson rag [invented; look note]
- A lean deck, the pennant always ready [invented]
- Cabin: trophies, a chart of coves [invented]
- The crew talk loud in port and quiet at sea [invented]

## 7. Sound hooks

- sfx_ui_port_open (G/game.rs:3137): arrival at a friendly cove
- sfx_ui_trade_coin (G/game.rs:4749): fencing a prize
- New sounds are ideas only: a pennant snap at departure (idea, no asset requested)

## 8. Flags

- None.
