# Faction canon index

Base commit: 9acf9e2. Status: draft canon, phase 4. Hand-maintained from content.json `port_politics.blocs` (C:13386), `port_politics.relationships` (C:13716), `factions` (C:2877) and `faction_relationships` (C:13866); there is no generator.

Two kinds of faction exist in data, and canon keeps them apart:

- Trade blocs (7): political groupings of ports. The player sees a bloc name only on the custom captain 'Trade bloc' row (crates/portlight-godot/src/game.rs:3007-3015). No live mechanic reads a bloc today (DF-15; custom_captain.rs:180-181, 328).
- Pirate factions (4): crews with territory regions and captains. The player meets them in pirate encounters and the duel prompt (crates/portlight-godot/src/logic.rs:1086-1094, 2203-2212) and on the custom captain 'Pirate faction' row (game.rs:3016-3024).

## Trade blocs

| Bloc | Ports | Regions | Partners | Rivals (list) | Hostile (list) | Cite | File |
|---|---|---|---|---|---|---|---|
| The Exchange Alliance | [Porto Novo](../ports/porto_novo.md), [Al-Manar](../ports/al_manar.md), [Silva Bay](../ports/silva_bay.md) | Mediterranean | The Iron Pact | The Silk Circle | The Shadow Ports | C:13388 | [exchange_alliance.md](exchange_alliance.md) |
| The Iron Pact | [Ironhaven](../ports/ironhaven.md), [Stormwall](../ports/stormwall.md) | North Atlantic | The Exchange Alliance | The Gold Coast Compact | The Shadow Ports | C:13410 | [iron_pact.md](iron_pact.md) |
| The Gold Coast Compact | [Sun Harbor](../ports/sun_harbor.md), [Palm Cove](../ports/palm_cove.md), [Iron Point](../ports/iron_point.md), [Pearl Shallows](../ports/pearl_shallows.md) | West Africa | The Free Ports, The Coral Crown | The Iron Pact | - | C:13431 | [gold_coast_compact.md](gold_coast_compact.md) |
| The Silk Circle | [Jade Port](../ports/jade_port.md), [Silk Haven](../ports/silk_haven.md), [Dragon's Gate](../ports/dragons_gate.md) | East Indies | The Coral Crown | The Exchange Alliance | The Iron Pact | C:13453 | [silk_circle.md](silk_circle.md) |
| The Free Ports | [Crosswind Isle](../ports/crosswind_isle.md), [Monsoon Reach](../ports/monsoon_reach.md), [Thornport](../ports/thornport.md) | East Indies, North Atlantic | The Gold Coast Compact | - | - | C:13475 | [free_ports.md](free_ports.md) |
| The Shadow Ports | [Corsair's Rest](../ports/corsairs_rest.md), [Spice Narrows](../ports/spice_narrows.md) | East Indies, Mediterranean | - | - | The Exchange Alliance, The Iron Pact | C:13493 | [shadow_ports.md](shadow_ports.md) |
| The Coral Crown | [Ember Isle](../ports/ember_isle.md), [Typhoon Anchorage](../ports/typhoon_anchorage.md), [Coral Throne](../ports/coral_throne.md) | South Seas | The Gold Coast Compact, The Silk Circle | - | - | C:13511 | [coral_crown.md](coral_crown.md) |

Bloc to bloc dispositions (symmetric table) are in each bloc file, section 2, and in [relations-matrix.md](../relations-matrix.md). Per-bloc lists disagree with the table in two places (DF-7, DF-27).

## Pirate factions

| Faction | Territory | Smuggler attitude | Captains | Home (canon) | Cite | File |
|---|---|---|---|---|---|---|
| The Crimson Tide | Mediterranean, North Atlantic | neutral | Scarlet Ana, The Butcher | Corsair's Rest [invented] | C:2879 | [crimson_tide.md](crimson_tide.md) |
| The Monsoon Syndicate | East Indies | friendly | Raj the Quiet, Typhoon Mei | Spice Narrows [invented] | C:2888 | [monsoon_syndicate.md](monsoon_syndicate.md) |
| The Deep Reef Brotherhood | South Seas, West Africa | respectful | Old Coral, The Diver | none, at sea [invented] | C:2896 | [deep_reef_brotherhood.md](deep_reef_brotherhood.md) |
| The Iron Wolves | North Atlantic | business | Sergeant Kruze, Gnaw | none, northern fjords [invented] | C:2905 | [iron_wolves.md](iron_wolves.md) |

## Faction to faction (hand-maintained from `faction_relationships`)

| | The Crimson Tide | The Monsoon Syndicate | The Deep Reef Brotherhood | The Iron Wolves |
|---|---|---|---|---|
| The Crimson Tide | . | rival | hostile | rival |
| The Monsoon Syndicate | rival | . | neutral | hostile |
| The Deep Reef Brotherhood | hostile | neutral | . | hostile |
| The Iron Wolves | rival | hostile | hostile | . |

Only `hostile` is read by the sim (content.rs:1156; journal beats, narrative.rs:655, 672, 736). Spillover and vendetta triggers are unread (DF-28).

## Rules

- Bloc and faction ids are data ids; file names are canon-only where they differ (gold_coast_compact for `gold_coast`, deep_reef_brotherhood for `deep_reef`).
- Pirate factions are not blocs and have no bloc relationship rows. Their link to the blocs is the Shadow Ports (C:13493) and port text.
- 'Trade bloc' and 'Pirate faction' are the shipped player labels (game.rs:3011, 3020); canon prose uses them.
- No bloc or faction has a flag, colour, seal or device in data. Canon invents none; a device would be an art brief and the holds stand.
- Phase 4 data issues are DF-26 to DF-28 in [data-fixes.md](../data-fixes.md).
