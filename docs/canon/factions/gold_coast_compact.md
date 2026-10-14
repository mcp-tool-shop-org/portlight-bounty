# The Gold Coast Compact (gold_coast_compact)

Base commit: 4f08125 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 4. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Name | The Gold Coast Compact | C:13432 |
| Kind | trade bloc (`port_politics.blocs`) | C:13386 |
| Ports | [Sun Harbor](../ports/sun_harbor.md), [Palm Cove](../ports/palm_cove.md), [Iron Point](../ports/iron_point.md), [Pearl Shallows](../ports/pearl_shallows.md) | C:13433 |
| Regions | West Africa | port records |
| Preferred partners | The Free Ports, The Coral Crown | C:13441 |
| Rivals (list) | The Iron Pact | C:13445 |
| Hostile to (list) | - | C:13448 |
| Loyalty bonus - disloyalty penalty | 0.06 - 0.05 [flavour, no mechanic] (DF-15) | C:13449-13450 |
| Player sees | the name on the custom captain 'Trade bloc' row | crates/portlight-godot/src/game.rs:3056-3064 |
| Mechanic | none found: a custom captain's bloc is validated and stored only (custom_captain.rs:180-181, 328) | open question Q2 |

## 1. Data cites

- description: "West Africa's communal trading network. Four ports bound by kinship, shared harvests, and the principle that trade is relationship, not transaction. They distrust industrial powers and welcome all who trade honestly." [ascii-folded] (C:13439)
- trade philosophy: "Generosity first, then business. The coast remembers." (C:13440)

## 2. Relations with every other bloc (hand-maintained from `relationships`, symmetric in data)

| Bloc | Disposition | Data text | Cite |
|---|---|---|---|
| [The Exchange Alliance](exchange_alliance.md) | neutral | "Distant but respectful. Cotton and dyes flow north; manufactured goods flow south." [ascii-folded] | C:13727-13728 |
| [The Iron Pact](iron_pact.md) | rival | "The Pact wants West Africa's iron ore. The Compact wants fair prices, not industrial exploitation." [ascii-folded] | C:13762-13763 |
| [The Silk Circle](silk_circle.md) | neutral | "Cotton for silk, dyes for porcelain. Distant but mutually profitable." [ascii-folded] | C:13797-13798 |
| [The Free Ports](free_ports.md) | allied | "Shared philosophy: trade is relationship, not domination. The Compact and the Free Ports understand each other." [ascii-folded] | C:13804-13805 |
| [The Shadow Ports](shadow_ports.md) | neutral | "The coast doesn't judge. They trade with everyone who trades honestly - even pirates, sometimes." [ascii-folded] | C:13811-13812 |
| [The Coral Crown](coral_crown.md) | allied | "Kinship across the water. Pearl divers from both shores share techniques and ceremonies." [ascii-folded] | C:13818-13819 |

Spillover values sit beside each row in data and are unread by the sim (DF-15).

## 3. History (all [invented] unless cited)

- T10 (~80 BS): the Compact is sworn among four West African ports (README T10).
- T12 (~50 BS): the Compact's ties to the Free Ports and the Coral Crown are formed in the same years as the free-port charter (README T12) [invented link].

A bloc history may add detail but must not move a timeline event to another era (README).

## 4. Character and customs (all [invented] unless cited)

Kinship before contract: generosity first, then business (C:13440). The Compact reads every trade as a relationship and remembers how it was treated [invented]. It trades even with pirates, sometimes, if they trade honestly (C:13812).

No bloc has a flag, colour, seal or uniform in data. Canon proposes none; any bloc device is a future art brief (holds stand).

## 5. Sound hook

None. Blocs are heard only through their ports (each port file, section I). No cue, no music.

## 6. Links

- Ports: [Sun Harbor](../ports/sun_harbor.md), [Palm Cove](../ports/palm_cove.md), [Iron Point](../ports/iron_point.md), [Pearl Shallows](../ports/pearl_shallows.md)
- Lanes touching bloc ports: [The Cotton Crossing](../lanes/cotton_crossing.md), [The Long Crossing](../lanes/long_crossing.md), [The Deep South Run](../lanes/deep_south_run.md)
- Matrix: [relations-matrix.md](../relations-matrix.md) (per-bloc summary)

## 7. Flags

- Region versus bloc: 'the Gold Coast' is the region and 'the Compact' is the bloc (DF-10). Canon uses 'the Compact' for the bloc.
