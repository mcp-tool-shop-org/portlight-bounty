# The Iron Pact (iron_pact)

Base commit: 4f08125 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 4. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Name | The Iron Pact | C:13411 |
| Kind | trade bloc (`port_politics.blocs`) | C:13386 |
| Ports | [Ironhaven](../ports/ironhaven.md), [Stormwall](../ports/stormwall.md) | C:13412 |
| Regions | North Atlantic | port records |
| Preferred partners | The Exchange Alliance | C:13418 |
| Rivals (list) | The Gold Coast Compact | C:13421 |
| Hostile to (list) | The Shadow Ports | C:13424 |
| Loyalty bonus - disloyalty penalty | 0.06 - 0.12 [flavour, no mechanic] (DF-15) | C:13427-13428 |
| Player sees | the name on the custom captain 'Trade bloc' row | crates/portlight-godot/src/game.rs:3056-3064 |
| Mechanic | none found: a custom captain's bloc is validated and stored only (custom_captain.rs:180-181, 328) | open question Q2 |

## 1. Data cites

- description: "Military-industrial alliance. Ironhaven's foundries supply Stormwall's garrison. They control the North Atlantic's weapons trade and view anyone selling weapons to pirates as a traitor." [ascii-folded] (C:13416)
- trade philosophy: "Strength through industry. Trade serves the fortress." (C:13417)

## 2. Relations with every other bloc (hand-maintained from `relationships`, symmetric in data)

| Bloc | Disposition | Data text | Cite |
|---|---|---|---|
| [The Exchange Alliance](exchange_alliance.md) | allied | "United by shared commitment to order, law, and recorded trade. The Alliance supplies grain; the Pact supplies iron." [ascii-folded] | C:13720-13721 |
| [The Gold Coast Compact](gold_coast_compact.md) | rival | "The Pact wants West Africa's iron ore. The Compact wants fair prices, not industrial exploitation." [ascii-folded] | C:13762-13763 |
| [The Silk Circle](silk_circle.md) | hostile | "The Silk Circle bans weapons imports. The Pact's entire economy is weapons. Fundamental conflict." [ascii-folded] | C:13769-13770 |
| [The Free Ports](free_ports.md) | neutral | "Thornport trades tea and tobacco with Ironhaven. Practical, not political." [ascii-folded] | C:13776-13777 |
| [The Shadow Ports](shadow_ports.md) | hostile | "Pirates arm themselves with stolen Pact weapons. The garrison takes this personally." [ascii-folded] | C:13783-13784 |
| [The Coral Crown](coral_crown.md) | neutral | "The Crown wants weapons. The Pact wants pearls. A transaction, not an alliance." [ascii-folded] | C:13790-13791 |

Spillover values sit beside each row in data and are unread by the sim (DF-15).

## 3. History (all [invented] unless cited)

- T3 and T8 (~500 and ~120 BS): the old Northern Crown raises the Stormwall Fortress and dissolves; the garrison holds the straits alone (README T3, T8).
- T11 (~60 BS): the Pact is founded; Ironhaven foundries supply Stormwall (README T11).
- T29 (~55 BS): Ironhaven iron first goes east by the Northern Passage (README T29).
- T31 to T33 (~45 to ~30 BS): deserters leave the garrison; the last and largest group become the Iron Wolves (README T31-T33).

A bloc history may add detail but must not move a timeline event to another era (README).

## 4. Character and customs (all [invented] unless cited)

A fortress economy: trade serves the garrison (C:13417). The Pact's two quarrels are inside it - Henrik's will to sell iron to anyone against Astrid's will to restrict weapons (C:7885) - and north of it, where the Iron Wolves raid supply lines (C:8201).

No bloc has a flag, colour, seal or uniform in data. Canon proposes none; any bloc device is a future art brief (holds stand).

## 5. Sound hook

None. Blocs are heard only through their ports (each port file, section I). No cue, no music.

## 6. Links

- Ports: [Ironhaven](../ports/ironhaven.md), [Stormwall](../ports/stormwall.md)
- Lanes touching bloc ports: [The Iron Strait](../lanes/iron_strait.md), [The Tea and Tobacco Road](../lanes/tea_and_tobacco_road.md), [The Northern Passage](../lanes/northern_passage.md)
- Matrix: [relations-matrix.md](../relations-matrix.md) (per-bloc summary)

## 7. Flags

- Iron Pact `hostile_to` omits The Silk Circle although the table says hostile (DF-7).
- Ironhaven embargoes the weapons it exports (DF-5).
