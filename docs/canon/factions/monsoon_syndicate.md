# The Monsoon Syndicate (monsoon_syndicate)

Base commit: 9acf9e2 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 4. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Name | The Monsoon Syndicate | C:2889 |
| Kind | pirate faction (`factions`) | C:2877 |
| Territory | East Indies | C:2890 |
| Smuggler attitude | friendly | C:2893 |
| Captains | [Raj the Quiet](../captains/pirates/raj_the_quiet.md), [Typhoon Mei](../captains/pirates/typhoon_mei.md) | C:2929, C:2936 |
| Home | Spice Narrows (README, Pirate home ports) [invented; DF-17] | - |
| Player sees | the name in the encounter panel and duel prompt; the custom captain 'Pirate faction' row | crates/portlight-godot/src/logic.rs:1089-1091, 2203-2212; game.rs:3016-3024 |

## 1. What the faction does in the sim

- Region pick: a pirate encounter prefers captains whose faction holds the destination region (voyage.rs:340-349; encounter.rs:96-105). With no faction in a region, the first faction in the list is used (encounter.rs:97-99).
- Attitude: a smuggler captain gets +5 toward this faction when the attitude is friendly, cooperative or respectful (encounter.rs:50-58). 'friendly' qualifies.
- Warning at sea: low regard in the faction's territory can bring a crossing-the-bow warning (consequences.rs:59-66).
- A captain's home region falls back to the faction's first territory region (memory.rs:379-384).
- Relations: only `hostile` pairs are read, for journal beats (content.rs:1156-1170; narrative.rs:655, 672, 736). Spillover and vendetta trigger text are unread (DF-28).

## 2. Data cites

- Spice Narrows power structure: "Spice Narrows is the Monsoon Syndicate's heart" [ascii-folded] (C:10590); Spice Lord Kiran is the Syndicate's governor there (C:10689, C:10694).
- Raj the Quiet's Shadow: "Not Raj the Quiet himself - Raj operates from the sea." [ascii-folded] (C:10733, C:10738); Raj is Kiran's intelligence chief (C:10703).
- Opium contract flavour: "The Monsoon Syndicate controls every pipe of opium east of Crosswind Isle." (C:3653)
- Wu at Dragon's Gate may still be a Syndicate informant (C:10553).
- The shadow pipeline: the Ghost Broker posts Syndicate contracts at Spice Narrows (C:11660).

## 3. Relations with every other faction (hand-maintained from `faction_relationships`, C:13866)

| Faction | Disposition | Data text | Vendetta trigger (unread) | Cite |
|---|---|---|---|---|
| [The Crimson Tide](crimson_tide.md) | rival | "Business rivals who respect each other's territory - barely. The Tide wants to control the Mediterranean-to-East route. The Syndicate wants to control all Eastern trade. They cooperate when it suits them and undercut when it doesn't." [ascii-folded] | "Running opium for the Syndicate through Tide waters." | C:13870-13873 |
| [The Deep Reef Brotherhood](deep_reef_brotherhood.md) | neutral | "Different worlds, different rules. The Syndicate operates through information and politics in the East. The Brotherhood sails the Southern reefs on honor and courage. They rarely cross paths, and when they do, they nod and move on." [ascii-folded] | "None - they don't care about each other." | C:13894-13897 |
| [The Iron Wolves](iron_wolves.md) | hostile | "The Wolves tried to muscle into the Eastern weapons trade. Raj the Quiet fed their fleet positions to a navy patrol. Three Wolf ships burned. The Wolves have never forgiven - or forgotten. Syndicate informants are killed on sight." [ascii-folded] | "Sharing intelligence with the Syndicate about Wolf positions." | C:13902-13905 |

Pirate factions have no relationship rows with the trade blocs. Their bloc ties are through ports only (Shadow Ports, C:13493).

## 4. History (all [invented] unless cited)

- T13 (~40 BS): the Syndicate settles in at Spice Narrows with the Spice Lords (README T13).
- T35 (~10 BS): Raj the Quiet feeds Wolf fleet positions to a navy patrol; three Wolf ships burn (README T35).

## 5. Character and customs (all [invented] unless cited)

Information is the cargo. The Syndicate works through informants, notes and politics (C:13895, C:10703) rather than broadsides, and it is the one faction friendly to smugglers in data (C:2893). [invented: a Syndicate promise is written, never spoken.]

No faction has a flag, colour or device in data beyond quoted sighting text (for example 'Crimson pennants', C:12819). Canon adds none; the character hold stands.

## 6. Sound hook

None proposed. Factions are met through voyage and encounter text only; no cue, music or voice (holds stand).

## 7. Links

- Ports: [Spice Narrows](../ports/spice_narrows.md)
- Lanes: [The Volcanic Passage](../lanes/volcanic_passage.md), [The Smuggler's Run](../lanes/smugglers_run.md)
- Captains: [Raj the Quiet](../captains/pirates/raj_the_quiet.md), [Typhoon Mei](../captains/pirates/typhoon_mei.md)

## 8. Flags

- Home port is canon only (DF-17).
- Raj the Quiet is both a sea captain (C:2929) and Kiran's intelligence chief (C:10703). Resolved in lore: Raj sails; his Shadow keeps his desk at the Narrows (C:10738). No data change.
