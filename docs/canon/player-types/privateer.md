# The Privateer - player type (privateer)

Base commit: c413788 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 5. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n; `S/` is crates/portlight-sim/src/, `G/` is crates/portlight-godot/src/. Legend: [exists file:line] in the sim today | [proposal; planned slice PT-Sn] intended, not shipped | [invented] lore | [flavour, no mechanic] | DF-n see ../data-fixes.md.
Who this captain is: [../captains/privateer.md](../captains/privateer.md). This file is how the type plays.

## 1. Identity and fantasy

The garrison-licensed hunter of Stormwall. Law on the water with a cutter's guns; the fantasy is the letter of marque. [invented] where not data.

The player's verb: "I hunt under licence" [invented]

## 2. Unique mechanics and perks

Starting values (all [exists], cited):

| Key | Value | Tag |
|---|---|---|
| Silver - ship - provisions | 475 - Swift Cutter (cutter) - 30 | [exists C:2437-2439] |
| Trade terms | buy x0.95 - sell x1.02 - luxury bonus 0.1 - port fees x0.6 | [exists C:2440-2444] |
| Voyage terms | provision burn x0.9 - speed +0.5 - storm resist +0.05 - cargo damage x0.9 | [exists C:2446-2450] |
| Inspection terms | chance x0.3 - seizure 0.0 - fines x0.5 | [exists C:2452-2455] |
| Starting regard (data keys, not player terms) | commercial_trust 8, mediterranean 3, north_atlantic 8 | [exists C:2457; S/model.rs:255-270] |
| Pirate regard | iron_wolves -5 | [exists C:2457] |
| Contract bias (x2 offer weight) | none today | [exists crates/portlight-sim/src/contracts.rs:75-80] |
| captain_type branch | no captain_type branch; differs by data only | [exists custom_captain.rs:432-442 (catalog only)] |

Perks:

- Starts in a Swift Cutter (2 cannons, cargo 50) [exists C:2438; C:131]
- Inspection x0.3, the lowest of any type; fines x0.5 [exists C:2453-2455]
- Port fees x0.6 everywhere [exists C:2444]
- Iron Wolves regard -5 from day 1 [exists C:2457]
- Contract bias on weapons, garrison and Atlantic charter work [proposal; planned slice PT-S1]
- Letter of marque: wins against Iron Wolves or Crimson Tide ships pay a small North Atlantic regard reward and no heat; copy states it. Trade-off: reward only in the North Atlantic, and the Wolves stay hostile [proposal; planned slice PT-S3]
- Title is 'Letter of Marque' (DF-26, resolved) [exists C:2433]

## 3. How the 20 ports treat this type

Basis: [exists] means a real effect of this type's numbers (regional regard, pirate regard, fees, inspection, home or mentor); [invented] is flavour only and implies no mechanic. Fees and inspection terms apply at every port alike (C:2440-2455); no port has a per-type rule.

| Port | Bloc | How they treat this type | Basis |
|---|---|---|---|
| [Porto Novo](../ports/porto_novo.md) | [The Exchange Alliance](../factions/exchange_alliance.md) | starts with warm Mediterranean regard (3); Alliance ports see a Pact officer | [exists C:2457] [invented] |
| [Al-Manar](../ports/al_manar.md) | [The Exchange Alliance](../factions/exchange_alliance.md) | starts with warm Mediterranean regard (3); Alliance ports see a Pact officer | [exists C:2457] [invented] |
| [Silva Bay](../ports/silva_bay.md) | [The Exchange Alliance](../factions/exchange_alliance.md) | starts with warm Mediterranean regard (3); Alliance ports see a Pact officer | [exists C:2457] [invented] |
| [Corsair's Rest](../ports/corsairs_rest.md) | [The Shadow Ports](../factions/shadow_ports.md) | starts with warm Mediterranean regard (3); Shadow Ports read the licence as a threat | [exists C:2457] [invented] |
| [Ironhaven](../ports/ironhaven.md) | [The Iron Pact](../factions/iron_pact.md) | starts with warm North Atlantic regard (8); home bloc; the garrison and Commander Vogt | [exists C:2457] [invented] |
| [Stormwall](../ports/stormwall.md) | [The Iron Pact](../factions/iron_pact.md) | home port; the mentor is here; starts with warm North Atlantic regard (8); home bloc; the garrison and Commander Vogt | [exists C:2435-2436] [exists C:2457] [invented] |
| [Thornport](../ports/thornport.md) | [The Free Ports](../factions/free_ports.md) | starts with warm North Atlantic regard (8); Free Ports watch a lawman warily | [exists C:2457] [invented] |
| [Sun Harbor](../ports/sun_harbor.md) | [The Gold Coast Compact](../factions/gold_coast_compact.md) | the Compact tolerates a licensed hunter | [invented] |
| [Palm Cove](../ports/palm_cove.md) | [The Gold Coast Compact](../factions/gold_coast_compact.md) | the Compact tolerates a licensed hunter | [invented] |
| [Iron Point](../ports/iron_point.md) | [The Gold Coast Compact](../factions/gold_coast_compact.md) | the Compact tolerates a licensed hunter | [invented] |
| [Pearl Shallows](../ports/pearl_shallows.md) | [The Gold Coast Compact](../factions/gold_coast_compact.md) | the Compact tolerates a licensed hunter | [invented] |
| [Jade Port](../ports/jade_port.md) | [The Silk Circle](../factions/silk_circle.md) | Silk Circle customs respect the commission | [invented] |
| [Monsoon Reach](../ports/monsoon_reach.md) | [The Free Ports](../factions/free_ports.md) | Free Ports watch a lawman warily | [invented] |
| [Silk Haven](../ports/silk_haven.md) | [The Silk Circle](../factions/silk_circle.md) | Silk Circle customs respect the commission | [invented] |
| [Crosswind Isle](../ports/crosswind_isle.md) | [The Free Ports](../factions/free_ports.md) | Free Ports watch a lawman warily | [invented] |
| [Dragon's Gate](../ports/dragons_gate.md) | [The Silk Circle](../factions/silk_circle.md) | Silk Circle customs respect the commission | [invented] |
| [Spice Narrows](../ports/spice_narrows.md) | [The Shadow Ports](../factions/shadow_ports.md) | Shadow Ports read the licence as a threat | [invented] |
| [Ember Isle](../ports/ember_isle.md) | [The Coral Crown](../factions/coral_crown.md) | Coral Crown cares for tribute, not licences | [invented] |
| [Typhoon Anchorage](../ports/typhoon_anchorage.md) | [The Coral Crown](../factions/coral_crown.md) | Coral Crown cares for tribute, not licences | [invented] |
| [Coral Throne](../ports/coral_throne.md) | [The Coral Crown](../factions/coral_crown.md) | Coral Crown cares for tribute, not licences | [invented] |

## 4. Reputation and faction effects

- Starting regard is copied from the type's data into the standing at new game (S/model.rs:255-270). Regional customs inspection rises with heat in a region (S/reputation.rs:216-227). Pirate hostility reads pirate regard (S/encounter.rs:42-62).
- Starts North Atlantic 8, Mediterranean 3, commercial trust 8 (C:2457)
- Iron Wolves start at -5, so they are cooler in encounters (S/encounter.rs:42-62)
- PT-S3 would add North Atlantic regard for Wolf and Tide wins [proposal; planned slice PT-S3]
- No spillover: regard with one bloc or faction does not move its rivals (DF-28). Canon claims none.

## 5. Progression

Uses existing systems: milestone and career scoring (S/campaign.rs:1245-1310), the ship ladder (C:131 ships) and contract offers (S/contracts.rs:75-80).

- Early: Patrol the North Atlantic from Stormwall
- Mid: Garrison contracts and Wolf fights (PT-S1, PT-S3 when shipped)
- Late: Bigger hulls on the ship ladder; career scoring

## 6. Look and feel (words only)

Words, not art direction. Character plates stay held; canon sets no face, age, gender or dress for the captain (../captains/README.md rules).

- Mood: disciplined, lawful, squared away - drilled not ragged [invented; look note]
- Materials: blacked iron fittings, polished leather cartridge boxes, a sealed letter-of-marque case, coiled boarding lines in neat flakes [invented; look note]
- Silhouette: square and upright - squared yards, straight lines [invented; look note]
- Palette: slate blue, pewter, scrubbed deal planking; mid values, cool-neutral; small accent of a faded charter-seal red [invented; look note]
- A disciplined deck; guns run out clean [invented]
- Cabin: the commission in a sealed case [invented]
- Flies the Pact colours openly [invented]
- The crew talk like a watch roster [invented]
- Closest pair is The Privateer and The Bounty Hunter: the Privateer is square, scrubbed and lighter; the Bounty Hunter is sharp, oiled and darker [invented; look note]

## 7. Sound hooks

- sfx_chart_sail_depart (G/game.rs:4529): putting out on patrol
- sfx_ui_contract_accept (G/game.rs:6861): taking a garrison job
- New sounds are ideas only: a bosun's pipe on a won fight (idea, no asset requested)

## 8. Flags

- DF-26 retitle pending (PT-S3)
- DF-32: the -5 Iron Wolves start is clamped to 0 on the first Iron Wolves duel win (C:2466)
