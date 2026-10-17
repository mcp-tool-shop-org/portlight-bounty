# The Privateer (privateer)

Base commit: 4f08125 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 4. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Name | The Privateer | C:2432 |
| Title | Letter of Marque | C:2433 |
| Home port | [Stormwall](../ports/stormwall.md) | C:2435 |
| Home region (data) | North Atlantic | C:2434 |
| Mentor | Commander Elias Vogt, Garrison Commander, at [Stormwall](../ports/stormwall.md) | C:2436; C:8194-8196 |
| Starting silver | 475 | C:2437 |
| Starting ship | Swift Cutter (cutter) | C:2438; C:148 |
| Starting provisions | 30 | C:2439 |
| Trade terms | buy x0.95 - sell x1.02 - luxury bonus 0.1 - port fees x0.6 | C:2441-2444 |
| Voyage terms | provision burn x0.9 - speed +0.5 - storm resist +0.05 - cargo damage x0.9 | C:2447-2450 |
| Inspection terms | chance x0.3 - seizure 0.0 - fines x0.5 | C:2453-2455 |
| Starting regard (data keys, not player terms) | commercial_trust 8, mediterranean 3, north_atlantic 8 | C:2457 |
| Pirate regard | The Iron Wolves -5 | C:2466 |
| Player sees | "The Privateer  --  Letter of Marque  --  Stormwall  --  cutter  --  475 silver" on the new game captain button | crates/portlight-godot/src/logic.rs:1355-1366; game.rs:2967-2974 |

## 1. The archetype (cited)

A Stormwall-licensed hunter under Commander Vogt (C:2436; C:8194-8196); starts in a Swift Cutter and already disliked by the Iron Wolves (-5, C:2466).

## 2. Background (all [invented])

[invented] Holds a garrison letter against pirates; the Wolves know the letter and the ship.

Archetypes are roles, not fixed people: the player names the captain (custom_captain.rs, new game Name row, game.rs:2960-2965). Canon gives no gender, age, face or dress; the character hold stands.

## 3. Links

- How this type plays: [player-types/privateer.md](../player-types/privateer.md)
- Home port: [Stormwall](../ports/stormwall.md) (bloc [The Iron Pact](../factions/iron_pact.md))
- Faction: [The Iron Wolves](../factions/iron_wolves.md)

## 4. Flags

- Title 'Letter of Marque' (C:2433); no longer shared with The Bounty Hunter (DF-26, resolved).
- Starts at -5 with the Iron Wolves; any later change clamps to 0-100 (session.rs:3701, 3705). Open question Q4.
