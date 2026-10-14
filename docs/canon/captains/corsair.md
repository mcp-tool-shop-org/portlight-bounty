# The Corsair (corsair)

Base commit: 4f08125 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 4. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Name | The Corsair | C:2472 |
| Title | Crimson Tide Captain | C:2473 |
| Home port | [Corsair's Rest](../ports/corsairs_rest.md) | C:2475 |
| Home region (data) | Mediterranean | C:2474 |
| Mentor | No One, The Tide's Voice, at [Corsair's Rest](../ports/corsairs_rest.md) | C:2476; C:7770-7772 |
| Starting silver | 500 | C:2477 |
| Starting ship | Coastal Sloop (sloop) | C:2478; C:133 |
| Starting provisions | 30 | C:2479 |
| Trade terms | buy x1.05 - sell x0.95 - luxury bonus 0.15 - port fees x0.5 | C:2481-2484 |
| Voyage terms | provision burn x0.9 - speed +0.3 - storm resist +0.0 - cargo damage x0.85 | C:2487-2490 |
| Inspection terms | chance x1.5 - seizure 0.08 - fines x1.5 | C:2493-2495 |
| Starting regard (data keys, not player terms) | customs_heat 12, mediterranean -5 | C:2497 |
| Pirate regard | The Crimson Tide 25 | C:2506 |
| Player sees | "The Corsair  --  Crimson Tide Captain  --  Corsair's Rest  --  sloop  --  500 silver" on the new game captain button | crates/portlight-godot/src/logic.rs:1355-1366; game.rs:2967-2974 |

## 1. The archetype (cited)

A captain of the Crimson Tide out of Corsair's Rest, answering to No One, the Tide's Voice (C:2473-2476; C:7770-7772). The only start with pirate regard (crimson_tide 25), enough for trade terms with the Tide (encounter.rs:59-64).

## 2. Background (all [invented])

[invented] Pays tribute and is owed favours in return; welcome at the Rest, watched everywhere the Alliance posts its law.

Archetypes are roles, not fixed people: the player names the captain (custom_captain.rs, new game Name row, game.rs:2960-2965). Canon gives no gender, age, face or dress; the character hold stands.

## 3. Links

- Home port: [Corsair's Rest](../ports/corsairs_rest.md) (bloc [The Shadow Ports](../factions/shadow_ports.md))
- Faction: [The Crimson Tide](../factions/crimson_tide.md)

## 4. Flags

- None.
