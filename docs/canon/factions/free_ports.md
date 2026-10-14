# The Free Ports (free_ports)

Base commit: 4f08125 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 4. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Name | The Free Ports | C:13476 |
| Kind | trade bloc (`port_politics.blocs`) | C:13386 |
| Ports | [Crosswind Isle](../ports/crosswind_isle.md), [Monsoon Reach](../ports/monsoon_reach.md), [Thornport](../ports/thornport.md) | C:13477 |
| Regions | East Indies, North Atlantic | port records |
| Preferred partners | The Gold Coast Compact | C:13484 |
| Rivals (list) | - | C:13487 |
| Hostile to (list) | - | C:13488 |
| Loyalty bonus - disloyalty penalty | 0.03 - 0.0 [flavour, no mechanic] (DF-15) | C:13489-13490 |
| Player sees | the name on the custom captain 'Trade bloc' row | crates/portlight-godot/src/game.rs:3056-3064 |
| Mechanic | none found: a custom captain's bloc is validated and stored only (custom_captain.rs:180-181, 328) | open question Q2 |

## 1. Data cites

- description: "The neutrals. Three ports that refuse to join any bloc, trade with everyone, and profit from every conflict. They anger the blocs by refusing to take sides - but everyone needs a neutral ground." [ascii-folded] (C:13482)
- trade philosophy: "All flags fly here. No nation claims us." (C:13483)

## 2. Relations with every other bloc (hand-maintained from `relationships`, symmetric in data)

| Bloc | Disposition | Data text | Cite |
|---|---|---|---|
| [The Exchange Alliance](exchange_alliance.md) | neutral | "The Alliance resents their neutrality but needs their trade routes." [ascii-folded] | C:13741-13742 |
| [The Iron Pact](iron_pact.md) | neutral | "Thornport trades tea and tobacco with Ironhaven. Practical, not political." [ascii-folded] | C:13776-13777 |
| [The Gold Coast Compact](gold_coast_compact.md) | allied | "Shared philosophy: trade is relationship, not domination. The Compact and the Free Ports understand each other." [ascii-folded] | C:13804-13805 |
| [The Silk Circle](silk_circle.md) | neutral | "The Circle tolerates neutrals. Crosswind Isle is useful as a buffer zone." [ascii-folded] | C:13825-13826 |
| [The Shadow Ports](shadow_ports.md) | neutral | "The Free Ports don't judge - but they don't protect smugglers either." [ascii-folded] | C:13846-13847 |
| [The Coral Crown](coral_crown.md) | neutral | "Distant. Monsoon Reach trades with the Crown occasionally, but no formal ties." [ascii-folded] | C:13853-13854 |

Spillover values sit beside each row in data and are unread by the sim (DF-15).

## 3. History (all [invented] unless cited)

- T12 (~50 BS): the Captain's Table charters Crosswind Isle as a free port; the Free Ports form (README T12).
- T22 (~50 BS): Crosswind Isle hangs the Free Port Bell at the charter (README T22).
- T14 (~25 BS): the Monsoon Shortcut joins Al-Manar to Monsoon Reach (README T14).

A bloc history may add detail but must not move a timeline event to another era (README).

## 4. Character and customs (all [invented] unless cited)

All flags fly here (C:13483). The Free Ports profit from every conflict by refusing to take a side, and every bloc needs a neutral ground (C:13482).

No bloc has a flag, colour, seal or uniform in data. Canon proposes none; any bloc device is a future art brief (holds stand).

## 5. Sound hook

None. Blocs are heard only through their ports (each port file, section I). No cue, no music.

## 6. Links

- Ports: [Crosswind Isle](../ports/crosswind_isle.md), [Monsoon Reach](../ports/monsoon_reach.md), [Thornport](../ports/thornport.md)
- Lanes touching bloc ports: [The Tea and Tobacco Road](../lanes/tea_and_tobacco_road.md), [The Long Crossing](../lanes/long_crossing.md), [The Porcelain Lane](../lanes/porcelain_lane.md), [Typhoon Alley](../lanes/typhoon_alley.md), [The Monsoon Shortcut](../lanes/monsoon_shortcut.md)
- Matrix: [relations-matrix.md](../relations-matrix.md) (per-bloc summary)

## 7. Flags

- The data calls them 'three ports that refuse to join any bloc' (C:13482) while listing them as a bloc. Resolved in lore: the Free Ports are a bloc of non-alignment, a compact to stay out of compacts. No data change.
