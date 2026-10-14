# The Deep Reef Brotherhood (deep_reef_brotherhood)

Base commit: 9acf9e2 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 4. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Name | The Deep Reef Brotherhood | C:2897 |
| Kind | pirate faction (`factions`) | C:2877 |
| Territory | South Seas, West Africa | C:2898 |
| Smuggler attitude | respectful | C:2902 |
| Captains | [Old Coral](../captains/pirates/old_coral.md), [The Diver](../captains/pirates/the_diver.md) | C:2943, C:2950 |
| Home | none; the Brotherhood lives at sea on reef anchorages between Pearl Shallows and Ember Isle (README) [invented] | - |
| Player sees | the name in the encounter panel and duel prompt; the custom captain 'Pirate faction' row | crates/portlight-godot/src/logic.rs:1089-1091, 2203-2212; game.rs:3016-3024 |

## 1. What the faction does in the sim

- Region pick: a pirate encounter prefers captains whose faction holds the destination region (voyage.rs:340-349; encounter.rs:96-105). With no faction in a region, the first faction in the list is used (encounter.rs:97-99).
- Attitude: a smuggler captain gets +5 toward this faction when the attitude is friendly, cooperative or respectful (encounter.rs:50-58). 'respectful' qualifies.
- Warning at sea: low regard in the faction's territory can bring a crossing-the-bow warning (consequences.rs:59-66).
- A captain's home region falls back to the faction's first territory region (memory.rs:379-384).
- Relations: only `hostile` pairs are read, for journal beats (content.rs:1156-1170; narrative.rs:655, 672, 736). Spillover and vendetta trigger text are unread (DF-28).

## 2. Data cites

- Faction relations only: the Brotherhood "answers to no one and attacks anyone who claims authority over the sea" [ascii-folded] (C:13879); it sails the Southern reefs "on honor and courage" (C:13895).
- No port text names the Deep Reef Brotherhood. The Deep Reef exchange (C:11034) is unrelated (README, resolved in lore).

## 3. Relations with every other faction (hand-maintained from `faction_relationships`, C:13866)

| Faction | Disposition | Data text | Vendetta trigger (unread) | Cite |
|---|---|---|---|---|
| [The Crimson Tide](crimson_tide.md) | hostile | "Fundamental incompatibility. The Tide runs a protection racket - pay or suffer. The Brotherhood answers to no one and attacks anyone who claims authority over the sea. They've been killing each other's captains for decades." [ascii-folded] | "Supplying weapons to the Tide that get used against Brotherhood ships." | C:13878-13881 |
| [The Monsoon Syndicate](monsoon_syndicate.md) | neutral | "Different worlds, different rules. The Syndicate operates through information and politics in the East. The Brotherhood sails the Southern reefs on honor and courage. They rarely cross paths, and when they do, they nod and move on." [ascii-folded] | "None - they don't care about each other." | C:13894-13897 |
| [The Iron Wolves](iron_wolves.md) | hostile | "Mutual contempt. The Brotherhood sees the Wolves as soulless mercenaries who fight without honor. The Wolves see the Brotherhood as romantic fools who'll die for principles instead of profit. They attack each other whenever their paths cross." [ascii-folded] | "Fencing stolen cargo from Brotherhood raids through Wolf contacts." | C:13910-13913 |

Pirate factions have no relationship rows with the trade blocs. Their bloc ties are through ports only (Shadow Ports, C:13493).

## 4. History (all [invented] unless cited)

- T30 (~45 BS): the feud with the Crimson Tide begins (README T30).

## 5. Character and customs (all [invented] unless cited)

The romantic faction: honour over profit, and contempt for anyone who claims the sea (C:13879, C:13911). The Wolves call them fools who will die for principles (C:13911). [invented: the Brotherhood keeps no ledgers and no port, and settles shares on deck.]

No faction has a flag, colour or device in data beyond quoted sighting text (for example 'Crimson pennants', C:12819). Canon adds none; the character hold stands.

## 6. Sound hook

None proposed. Factions are met through voyage and encounter text only; no cue, music or voice (holds stand).

## 7. Links

- Ports: [Pearl Shallows](../ports/pearl_shallows.md), [Ember Isle](../ports/ember_isle.md)
- Captains: [Old Coral](../captains/pirates/old_coral.md), [The Diver](../captains/pirates/the_diver.md)

## 8. Flags

- Home port is canon only (DF-17).
- Territory includes West Africa (C:2898-2901), but no Gold Coast text mentions the Brotherhood. Canon places its anchorages off the Compact's reefs; no flag.
