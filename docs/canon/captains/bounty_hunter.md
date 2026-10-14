# The Bounty Hunter (bounty_hunter)

Base commit: 9acf9e2 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 4. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Name | The Bounty Hunter | C:2626 |
| Title | Licensed Hunter | C:2627 |
| Home port | [Crosswind Isle](../ports/crosswind_isle.md) | C:2629 |
| Home region (data) | North Atlantic | C:2628 |
| Mentor | Dock Master Tao, Dock Master, at [Crosswind Isle](../ports/crosswind_isle.md) | C:2630; C:10220-10222 |
| Starting silver | 425 | C:2631 |
| Starting ship | Swift Cutter (cutter) | C:2632; C:148 |
| Starting provisions | 30 | C:2633 |
| Trade terms | buy x1.05 - sell x0.98 - luxury bonus 0.0 - port fees x0.7 | C:2635-2638 |
| Voyage terms | provision burn x0.9 - speed +0.5 - storm resist +0.05 - cargo damage x1.0 | C:2641-2644 |
| Inspection terms | chance x0.6 - seizure 0.0 - fines x0.5 | C:2647-2649 |
| Starting regard (data keys, not player terms) | commercial_trust 3, mediterranean 3, north_atlantic 8 | C:2651 |
| Pirate regard | none | C:2659 |
| Player sees | "The Bounty Hunter  --  Licensed Hunter  --  Crosswind Isle  --  cutter  --  425 silver" on the new game captain button | crates/portlight-godot/src/logic.rs:1355-1366; game.rs:2918-2925 |

## 1. The archetype (cited)

A licensed hunter working out of Crosswind Isle under Dock Master Tao (C:2627-2630). Shares the title 'Licensed Hunter' with the Privateer (DF-26) and a home-region mismatch (DF-3).

## 2. Background (all [invented])

[invented] Takes bounties posted at the neutral Bell and Board and answers to no bloc.

Archetypes are roles, not fixed people: the player names the captain (custom_captain.rs, new game Name row, game.rs:2911-2916). Canon gives no gender, age, face or dress; the character hold stands.

## 3. Links

- Home port: [Crosswind Isle](../ports/crosswind_isle.md) (bloc [The Free Ports](../factions/free_ports.md))

## 4. Flags

- Home region North Atlantic against home port Crosswind Isle (East Indies) (DF-3).
- Same title as The Privateer (DF-26).
