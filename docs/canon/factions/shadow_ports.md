# The Shadow Ports (shadow_ports)

Base commit: 4f08125 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 4. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Name | The Shadow Ports | C:13494 |
| Kind | trade bloc (`port_politics.blocs`) | C:13386 |
| Ports | [Corsair's Rest](../ports/corsairs_rest.md), [Spice Narrows](../ports/spice_narrows.md) | C:13495 |
| Regions | East Indies, Mediterranean | port records |
| Preferred partners | - | C:13501 |
| Rivals (list) | - | C:13502 |
| Hostile to (list) | The Exchange Alliance, The Iron Pact | C:13503 |
| Loyalty bonus - disloyalty penalty | 0.12 - 0.0 [flavour, no mechanic] (DF-15) | C:13507-13508 |
| Player sees | the name on the custom captain 'Trade bloc' row | crates/portlight-godot/src/game.rs:3056-3064 |
| Mechanic | none found: a custom captain's bloc is validated and stored only (custom_captain.rs:180-181, 328) | open question Q2 |

## 1. Data cites

- description: "The black market. Two ports outside all legitimate trade agreements. They answer to pirate factions, not merchant guilds. The legitimate blocs despise them; the smart captains use them." [ascii-folded] (C:13499)
- trade philosophy: "No names, no questions, no manifests." (C:13500)

## 2. Relations with every other bloc (hand-maintained from `relationships`, symmetric in data)

| Bloc | Disposition | Data text | Cite |
|---|---|---|---|
| [The Exchange Alliance](exchange_alliance.md) | hostile | "Smugglers undermine everything the Alliance stands for. Captains who trade at Corsair's Rest lose standing fast." [ascii-folded] | C:13748-13749 |
| [The Iron Pact](iron_pact.md) | hostile | "Pirates arm themselves with stolen Pact weapons. The garrison takes this personally." [ascii-folded] | C:13783-13784 |
| [The Gold Coast Compact](gold_coast_compact.md) | neutral | "The coast doesn't judge. They trade with everyone who trades honestly - even pirates, sometimes." [ascii-folded] | C:13811-13812 |
| [The Silk Circle](silk_circle.md) | rival | "The Spice Lords at Spice Narrows are a thorn. They undercut Silk Circle prices with stolen goods." [ascii-folded] | C:13832-13833 |
| [The Free Ports](free_ports.md) | neutral | "The Free Ports don't judge - but they don't protect smugglers either." [ascii-folded] | C:13846-13847 |
| [The Coral Crown](coral_crown.md) | neutral | "The Crown doesn't care about your morals, only your tribute. Shadow Ports are welcome if they pay." [ascii-folded] | C:13860-13861 |

Spillover values sit beside each row in data and are unread by the sim (DF-15).

## 3. History (all [invented] unless cited)

- T13 (~40 BS): smugglers driven out of Porto Novo settle at Corsair's Rest; the Spice Lords take the Narrows; the Crimson Tide and the Monsoon Syndicate settle in (README T13).
- T28 (~100 BS): before them, one line of pirate kings held both ends of the Smuggler's Run (README T28).

A bloc history may add detail but must not move a timeline event to another era (README).

## 4. Character and customs (all [invented] unless cited)

No names, no questions, no manifests (C:13500). The two ports answer to pirate factions, not merchant guilds (C:13499): Corsair's Rest to the Crimson Tide, Spice Narrows to the Monsoon Syndicate (C:7650, C:10590).

No bloc has a flag, colour, seal or uniform in data. Canon proposes none; any bloc device is a future art brief (holds stand).

## 5. Sound hook

None. Blocs are heard only through their ports (each port file, section I). No cue, no music.

## 6. Links

- Ports: [Corsair's Rest](../ports/corsairs_rest.md), [Spice Narrows](../ports/spice_narrows.md)
- Lanes touching bloc ports: [The Shadow Lane](../lanes/shadow_lane.md), [The Volcanic Passage](../lanes/volcanic_passage.md), [The Smuggler's Run](../lanes/smugglers_run.md)
- Matrix: [relations-matrix.md](../relations-matrix.md) (per-bloc summary)

## 7. Flags

- loyalty_bonus 0.12 is the highest of any bloc; it has no live effect (DF-15). [flavour, no mechanic]
- The Silk Circle rivalry is in the table only (DF-27).
