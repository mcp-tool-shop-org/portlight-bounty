# The Silk Circle (silk_circle)

Base commit: 9acf9e2 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 4. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Name | The Silk Circle | C:13454 |
| Kind | trade bloc (`port_politics.blocs`) | C:13386 |
| Ports | [Jade Port](../ports/jade_port.md), [Silk Haven](../ports/silk_haven.md), [Dragon's Gate](../ports/dragons_gate.md) | C:13455 |
| Regions | East Indies | port records |
| Preferred partners | The Coral Crown | C:13462 |
| Rivals (list) | The Exchange Alliance | C:13465 |
| Hostile to (list) | The Iron Pact | C:13468 |
| Loyalty bonus - disloyalty penalty | 0.1 - 0.08 [flavour, no mechanic] (DF-15) | C:13471-13472 |
| Player sees | the name on the custom captain 'Trade bloc' row | crates/portlight-godot/src/game.rs:3007-3015 |
| Mechanic | none found: a custom captain's bloc is validated and stored only (custom_captain.rs:180-181, 328) | open question Q2 |

## 1. Data cites

- description: "The Eastern dynasties. Ancient, hierarchical, and fiercely protective of their luxury goods. They control porcelain, silk, and the finest tea. They view Western merchants as barbarians - useful barbarians, but barbarians." [ascii-folded] (C:13460)
- trade philosophy: "Quality over quantity. Hierarchy governs all." (C:13461)

## 2. Relations with every other bloc (hand-maintained from `relationships`, symmetric in data)

| Bloc | Disposition | Data text | Cite |
|---|---|---|---|
| [The Exchange Alliance](exchange_alliance.md) | rival | "The old rivalry. Both claim to be the cradle of civilization. Both want to set the terms of East-West trade." [ascii-folded] | C:13734-13735 |
| [The Iron Pact](iron_pact.md) | hostile | "The Silk Circle bans weapons imports. The Pact's entire economy is weapons. Fundamental conflict." [ascii-folded] | C:13769-13770 |
| [The Gold Coast Compact](gold_coast_compact.md) | neutral | "Cotton for silk, dyes for porcelain. Distant but mutually profitable." [ascii-folded] | C:13797-13798 |
| [The Free Ports](free_ports.md) | neutral | "The Circle tolerates neutrals. Crosswind Isle is useful as a buffer zone." [ascii-folded] | C:13825-13826 |
| [The Shadow Ports](shadow_ports.md) | rival | "The Spice Lords at Spice Narrows are a thorn. They undercut Silk Circle prices with stolen goods." [ascii-folded] | C:13832-13833 |
| [The Coral Crown](coral_crown.md) | allied | "Ancient cultural ties. The Crown's pearls and medicines complement the Circle's silk and porcelain." [ascii-folded] | C:13839-13840 |

Spillover values sit beside each row in data and are unread by the sim (DF-15).

## 3. History (all [invented] unless cited)

- T1 (~1000 BS): Silk Haven's patterns are first woven (README T1).
- T7 (~150 BS): the Circle is formalised; Dragon's Gate raises its towers and chain (README T7).
- T21 (~15 BS): Dragon's Gate raises its chains and does not lower them again (README T21).
- T36 (~8 BS): Iron Wolves try to run weapons through the Gate; fifteen are chained to the seabed (README T36).

A bloc history may add detail but must not move a timeline event to another era (README).

## 4. Character and customs (all [invented] unless cited)

Hierarchy governs all; quality over quantity (C:13461). The Circle sees Western merchants as useful barbarians (C:13460) and bans weapons imports, which is the root of its feud with the Iron Pact (C:13770).

No bloc has a flag, colour, seal or uniform in data. Canon proposes none; any bloc device is a future art brief (holds stand).

## 5. Sound hook

None. Blocs are heard only through their ports (each port file, section I). No cue, no music.

## 6. Links

- Ports: [Jade Port](../ports/jade_port.md), [Silk Haven](../ports/silk_haven.md), [Dragon's Gate](../ports/dragons_gate.md)
- Lanes touching bloc ports: [The Porcelain Lane](../lanes/porcelain_lane.md), [The Silk Road by Sea](../lanes/silk_road_by_sea.md), [The Northern Passage](../lanes/northern_passage.md)
- Matrix: [relations-matrix.md](../relations-matrix.md) (per-bloc summary)

## 7. Flags

- Silk Circle lists The Iron Pact in `hostile_to`; the Pact does not list it back (DF-7).
- The table says Silk Circle and Shadow Ports are rivals (C:13832), but neither bloc lists the other in `rivals` (DF-27).
