# The Dockhand (dockhand)

Base commit: 4f08125 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 4. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Name | The Dockhand | C:2588 |
| Title | Self-Made Captain | C:2589 |
| Home port | [Crosswind Isle](../ports/crosswind_isle.md) | C:2591 |
| Home region (data) | East Indies | C:2590 |
| Mentor | Dock Master Tao, Dock Master, at [Crosswind Isle](../ports/crosswind_isle.md) | C:2592; C:10220-10222 |
| Starting silver | 300 | C:2593 |
| Starting ship | Coastal Sloop (sloop) | C:2594; C:133 |
| Starting provisions | 30 | C:2595 |
| Trade terms | buy x1.0 - sell x1.0 - luxury bonus 0.0 - port fees x1.0 | C:2597-2600 |
| Voyage terms | provision burn x0.85 - speed +0.0 - storm resist +0.0 - cargo damage x0.9 | C:2603-2606 |
| Inspection terms | chance x1.0 - seizure 0.0 - fines x1.0 | C:2609-2611 |
| Starting regard (data keys, not player terms) | none | C:2613 |
| Pirate regard | none | C:2621 |
| Player sees | "The Dockhand  --  Self-Made Captain  --  Crosswind Isle  --  sloop  --  300 silver" on the new game captain button | crates/portlight-godot/src/logic.rs:1355-1366; game.rs:2967-2974 |

## 1. The archetype (cited)

A self-made captain off the Crosswind Isle docks under Dock Master Tao (C:2589-2592; C:10220-10222). The poorest start (300 silver) and no starting regard anywhere.

## 2. Background (all [invented])

[invented] Loaded every flag's cargo on the free port's quay and finally bought passage on a hull of their own.

Archetypes are roles, not fixed people: the player names the captain (custom_captain.rs, new game Name row, game.rs:2960-2965). Canon gives no gender, age, face or dress; the character hold stands.

## 3. Links

- How this type plays: [player-types/dockhand.md](../player-types/dockhand.md)
- Home port: [Crosswind Isle](../ports/crosswind_isle.md) (bloc [The Free Ports](../factions/free_ports.md))

## 4. Flags

- Same mentor as The Bounty Hunter (Dock Master Tao, C:2592, C:2630). Two archetypes, one teacher; no flag.
