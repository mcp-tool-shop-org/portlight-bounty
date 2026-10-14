# The Exchange Alliance (exchange_alliance)

Base commit: 9acf9e2 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 4. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Name | The Exchange Alliance | C:13389 |
| Kind | trade bloc (`port_politics.blocs`) | C:13386 |
| Ports | [Porto Novo](../ports/porto_novo.md), [Al-Manar](../ports/al_manar.md), [Silva Bay](../ports/silva_bay.md) | C:13390 |
| Regions | Mediterranean | port records |
| Preferred partners | The Iron Pact | C:13397 |
| Rivals (list) | The Silk Circle | C:13400 |
| Hostile to (list) | The Shadow Ports | C:13403 |
| Loyalty bonus - disloyalty penalty | 0.08 - 0.1 [flavour, no mechanic] (DF-15) | C:13406-13407 |
| Player sees | the name on the custom captain 'Trade bloc' row | crates/portlight-godot/src/game.rs:3007-3015 |
| Mechanic | none found: a custom captain's bloc is validated and stored only (custom_captain.rs:180-181, 328) | open question Q2 |

## 1. Data cites

- description: "The Mediterranean old guard. Three ports bound by centuries of contract law, shared weights and measures, and a mutual commitment to fair dealing. They despise smugglers and distrust the East." [ascii-folded] (C:13395)
- trade philosophy: "Contracts are sacred. Prices are posted. Cheats are remembered." (C:13396)

## 2. Relations with every other bloc (hand-maintained from `relationships`, symmetric in data)

| Bloc | Disposition | Data text | Cite |
|---|---|---|---|
| [The Iron Pact](iron_pact.md) | allied | "United by shared commitment to order, law, and recorded trade. The Alliance supplies grain; the Pact supplies iron." [ascii-folded] | C:13720-13721 |
| [The Gold Coast Compact](gold_coast_compact.md) | neutral | "Distant but respectful. Cotton and dyes flow north; manufactured goods flow south." [ascii-folded] | C:13727-13728 |
| [The Silk Circle](silk_circle.md) | rival | "The old rivalry. Both claim to be the cradle of civilization. Both want to set the terms of East-West trade." [ascii-folded] | C:13734-13735 |
| [The Free Ports](free_ports.md) | neutral | "The Alliance resents their neutrality but needs their trade routes." [ascii-folded] | C:13741-13742 |
| [The Shadow Ports](shadow_ports.md) | hostile | "Smugglers undermine everything the Alliance stands for. Captains who trade at Corsair's Rest lose standing fast." [ascii-folded] | C:13748-13749 |
| [The Coral Crown](coral_crown.md) | neutral | "Too distant to care much. The Crown's pearls are welcome; their politics aren't." [ascii-folded] | C:13755-13756 |

Spillover values sit beside each row in data and are unread by the sim (DF-15).

## 3. History (all [invented] unless cited)

- T5 (~300 BS): the Alliance is signed at Porto Novo, which keeps the headquarters (README T5).
- T13 (~40 BS): Porto Novo's strict code drives smugglers to Corsair's Rest; the Alliance's hostility to the Shadow Ports dates from here (README T13).
- T15 (~12 BS): Silva Bay refuses the Iron Pact's war-fleet commission and stays in the Alliance on its own terms (README T15; C:7456).

A bloc history may add detail but must not move a timeline event to another era (README).

## 4. Character and customs (all [invented] unless cited)

The Alliance thinks in ledgers: posted prices, shared weights, remembered cheats (C:13396). It is old money and old law; it treats smugglers as a moral failing rather than a trade risk [invented]. Silva Bay is the loose stone: a member because it is convenient, not because it is loyal (C:7565).

No bloc has a flag, colour, seal or uniform in data. Canon proposes none; any bloc device is a future art brief (holds stand).

## 5. Sound hook

None. Blocs are heard only through their ports (each port file, section I). No cue, no music.

## 6. Links

- Ports: [Porto Novo](../ports/porto_novo.md), [Al-Manar](../ports/al_manar.md), [Silva Bay](../ports/silva_bay.md)
- Lanes touching bloc ports: [The Grain Road](../lanes/grain_road.md), [The Timber Run](../lanes/timber_run.md), [The Shadow Lane](../lanes/shadow_lane.md), [The Cotton Crossing](../lanes/cotton_crossing.md), [The Monsoon Shortcut](../lanes/monsoon_shortcut.md)
- Matrix: [relations-matrix.md](../relations-matrix.md) (per-bloc summary)

## 7. Flags

- The Alliance lists no partner but the Iron Pact; Gold Coast, Free Ports and Coral Crown are neutral in the table. Consistent. No flag.
