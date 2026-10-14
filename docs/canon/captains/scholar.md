# The Scholar (scholar)

Base commit: 4f08125 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 4. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Name | The Scholar | C:2512 |
| Title | Wind Temple Student | C:2513 |
| Home port | [Monsoon Reach](../ports/monsoon_reach.md) | C:2515 |
| Home region (data) | East Indies | C:2514 |
| Mentor | Brother Anand, Wind Reader, at [Monsoon Reach](../ports/monsoon_reach.md) | C:2516; C:9750-9752 |
| Starting silver | 400 | C:2517 |
| Starting ship | Coastal Sloop (sloop) | C:2518; C:133 |
| Starting provisions | 35 | C:2519 |
| Trade terms | buy x1.0 - sell x1.0 - luxury bonus 0.0 - port fees x0.85 | C:2521-2524 |
| Voyage terms | provision burn x0.75 - speed +1.0 - storm resist +0.1 - cargo damage x0.8 | C:2527-2530 |
| Inspection terms | chance x0.7 - seizure 0.0 - fines x0.8 | C:2533-2535 |
| Starting regard (data keys, not player terms) | commercial_trust 5, mediterranean 2, north_atlantic 2, west_africa 2, east_indies 8, south_seas 2 | C:2537 |
| Pirate regard | none | C:2545 |
| Player sees | "The Scholar  --  Wind Temple Student  --  Monsoon Reach  --  sloop  --  400 silver" on the new game captain button | crates/portlight-godot/src/logic.rs:1355-1366; game.rs:2967-2974 |

## 1. The archetype (cited)

A student of the Wind Temple at Monsoon Reach under Brother Anand, the Wind Reader (C:2513-2516; C:9750-9752). Even regard in every region, strongest in the East Indies.

## 2. Background (all [invented])

[invented] Left the Temple to test its wind readings on real lanes before taking vows.

Archetypes are roles, not fixed people: the player names the captain (custom_captain.rs, new game Name row, game.rs:2960-2965). Canon gives no gender, age, face or dress; the character hold stands.

## 3. Links

- Home port: [Monsoon Reach](../ports/monsoon_reach.md) (bloc [The Free Ports](../factions/free_ports.md))

## 4. Flags

- None.
