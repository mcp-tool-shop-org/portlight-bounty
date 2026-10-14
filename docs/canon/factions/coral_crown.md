# The Coral Crown (coral_crown)

Base commit: 9acf9e2 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 4. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Name | The Coral Crown | C:13512 |
| Kind | trade bloc (`port_politics.blocs`) | C:13386 |
| Ports | [Ember Isle](../ports/ember_isle.md), [Typhoon Anchorage](../ports/typhoon_anchorage.md), [Coral Throne](../ports/coral_throne.md) | C:13513 |
| Regions | South Seas | port records |
| Preferred partners | The Gold Coast Compact, The Silk Circle | C:13520 |
| Rivals (list) | - | C:13524 |
| Hostile to (list) | - | C:13525 |
| Loyalty bonus - disloyalty penalty | 0.08 - 0.06 [flavour, no mechanic] (DF-15) | C:13526-13527 |
| Player sees | the name on the custom captain 'Trade bloc' row | crates/portlight-godot/src/game.rs:3007-3015 |
| Mechanic | none found: a custom captain's bloc is validated and stored only (custom_captain.rs:180-181, 328) | open question Q2 |

## 1. Data cites

- description: "The South Seas kingdom. Three island ports under the Coral King's authority. Trade here is tribute - you give what the king demands and receive what he allows. Disrespect means exile." [ascii-folded] (C:13518)
- trade philosophy: "Tribute earns trade. The reef remembers." (C:13519)

## 2. Relations with every other bloc (hand-maintained from `relationships`, symmetric in data)

| Bloc | Disposition | Data text | Cite |
|---|---|---|---|
| [The Exchange Alliance](exchange_alliance.md) | neutral | "Too distant to care much. The Crown's pearls are welcome; their politics aren't." [ascii-folded] | C:13755-13756 |
| [The Iron Pact](iron_pact.md) | neutral | "The Crown wants weapons. The Pact wants pearls. A transaction, not an alliance." [ascii-folded] | C:13790-13791 |
| [The Gold Coast Compact](gold_coast_compact.md) | allied | "Kinship across the water. Pearl divers from both shores share techniques and ceremonies." [ascii-folded] | C:13818-13819 |
| [The Silk Circle](silk_circle.md) | allied | "Ancient cultural ties. The Crown's pearls and medicines complement the Circle's silk and porcelain." [ascii-folded] | C:13839-13840 |
| [The Free Ports](free_ports.md) | neutral | "Distant. Monsoon Reach trades with the Crown occasionally, but no formal ties." [ascii-folded] | C:13853-13854 |
| [The Shadow Ports](shadow_ports.md) | neutral | "The Crown doesn't care about your morals, only your tribute. Shadow Ports are welcome if they pay." [ascii-folded] | C:13860-13861 |

Spillover values sit beside each row in data and are unread by the sim (DF-15).

## 3. History (all [invented] unless cited)

- T2 (~700 BS): the Coral Palace begins to grow; the Ember Keepers settle the volcano (README T2).
- T12 (~50 BS): the Coral Crown forms (README T12).
- T37 (~40 BS): the present Coral King is crowned as a boy (README T37).
- T20 (~70 BS): Typhoon Anchorage lashes its first wrecks into the Storm Wall (README T20).

A bloc history may add detail but must not move a timeline event to another era (README).

## 4. Character and customs (all [invented] unless cited)

Tribute earns trade (C:13519). The Crown is a kingdom of three islands under the Coral King (C:13518); he has no heir, and three princes court foreign captains (C:12201).

No bloc has a flag, colour, seal or uniform in data. Canon proposes none; any bloc device is a future art brief (holds stand).

## 5. Sound hook

None. Blocs are heard only through their ports (each port file, section I). No cue, no music.

## 6. Links

- Ports: [Ember Isle](../ports/ember_isle.md), [Typhoon Anchorage](../ports/typhoon_anchorage.md), [Coral Throne](../ports/coral_throne.md)
- Lanes touching bloc ports: [Typhoon Alley](../lanes/typhoon_alley.md), [The Volcanic Passage](../lanes/volcanic_passage.md), [The Deep South Run](../lanes/deep_south_run.md)
- Matrix: [relations-matrix.md](../relations-matrix.md) (per-bloc summary)

## 7. Flags

- The bloc text puts three ports 'under the Coral King's authority' (C:13518), but Ember Isle is governed through the Keeper Elder (C:10823-10824) and Typhoon Anchorage by its Storm Chief (C:11019-11020). Resolved in lore: the Crown is an overlordship of tribute; each island keeps its own ruler, and the King's rule is absolute only within the lagoon (C:11288). No data change.
