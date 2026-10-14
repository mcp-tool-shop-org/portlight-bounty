# The Iron Wolves (iron_wolves)

Base commit: 4f08125 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 4. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Name | The Iron Wolves | C:2906 |
| Kind | pirate faction (`factions`) | C:2877 |
| Territory | North Atlantic | C:2907 |
| Smuggler attitude | business | C:2910 |
| Captains | [Sergeant Kruze](../captains/pirates/sergeant_kruze.md), [Gnaw](../captains/pirates/gnaw.md) | C:2957, C:2964 |
| Home | none; the Wolves raid from unnamed fjords north of Stormwall (README) [invented] | - |
| Player sees | the name in the encounter panel and duel prompt; the custom captain 'Pirate faction' row | crates/portlight-godot/src/logic.rs:1089-1091, 2203-2212; game.rs:3065-3073 |

## 1. What the faction does in the sim

- Region pick: a pirate encounter prefers captains whose faction holds the destination region (voyage.rs:340-349; encounter.rs:96-105). With no faction in a region, the first faction in the list is used (encounter.rs:91-94).
- Attitude: a smuggler captain gets +5 toward this faction when the attitude is friendly, cooperative or respectful (encounter.rs:50-58). 'business' does not qualify.
- Warning at sea: low regard in the faction's territory can bring a crossing-the-bow warning (consequences.rs:59-66).
- A captain's home region falls back to the faction's first territory region (memory.rs:379-384).
- Relations: only `hostile` pairs are read, for journal beats (content.rs:1156-1170; narrative.rs:655, 672, 736). Spillover and vendetta trigger text are unread (DF-28).

## 2. Data cites

- Deserters: No One "was a garrison officer who deserted - like the Iron Wolves, but earlier" [ascii-folded] (C:7781); Gunnar "deserted from the same garrison as the Iron Wolves - but years earlier ... They left because the discipline bored them." [ascii-folded] (C:7971)
- Commander Vogt sees "Iron Wolves raiding supply lines" [ascii-folded] (C:8201).
- Dragon's Gate grudge: "Fifteen Wolf sailors are still chained to the seabed." (C:13641)
- Coral Throne: Hana bought weapons from an Iron Wolves captain for the Crown's armoury (C:11335).
- The Privateer archetype starts with the Wolves' regard at -5 (C:2466) - see [The Privateer](../captains/privateer.md).

## 3. Relations with every other faction (hand-maintained from `faction_relationships`, C:13866)

| Faction | Disposition | Data text | Vendetta trigger (unread) | Cite |
|---|---|---|---|---|
| [The Crimson Tide](crimson_tide.md) | rival | "Former allies who split over territory. Both are organized and militaristic, but the Wolves left when the Tide's leader demanded tribute from their operations. Now they compete for the same weapons trade in overlapping waters." [ascii-folded] | "Selling black powder to the Tide that was meant for the Wolves." | C:13886-13889 |
| [The Monsoon Syndicate](monsoon_syndicate.md) | hostile | "The Wolves tried to muscle into the Eastern weapons trade. Raj the Quiet fed their fleet positions to a navy patrol. Three Wolf ships burned. The Wolves have never forgiven - or forgotten. Syndicate informants are killed on sight." [ascii-folded] | "Sharing intelligence with the Syndicate about Wolf positions." | C:13902-13905 |
| [The Deep Reef Brotherhood](deep_reef_brotherhood.md) | hostile | "Mutual contempt. The Brotherhood sees the Wolves as soulless mercenaries who fight without honor. The Wolves see the Brotherhood as romantic fools who'll die for principles instead of profit. They attack each other whenever their paths cross." [ascii-folded] | "Fencing stolen cargo from Brotherhood raids through Wolf contacts." | C:13910-13913 |

Pirate factions have no relationship rows with the trade blocs. Their bloc ties are through ports only (Shadow Ports, C:13493).

## 4. History (all [invented] unless cited)

- T33 (~30 BS): the Wolves desert the garrison and sail with the Crimson Tide (README T33).
- T34 (~22 BS): they split from the Tide when its leader demands tribute (README T34).
- T35 (~10 BS): three Wolf ships burn after Raj the Quiet sells their positions (README T35).
- T36 (~8 BS): Wolves try to run weapons through Dragon's Gate (README T36).

## 5. Character and customs (all [invented] unless cited)

Organised and militaristic (C:13887); deserters who kept the drill and dropped the oath [invented]. They trade weapons and black powder in waters they share with the Tide (C:13887, C:13889) and sell as far as the Coral Throne (C:11335).

No faction has a flag, colour or device in data beyond quoted sighting text (for example 'Crimson pennants', C:12819). Canon adds none; the character hold stands.

## 6. Sound hook

None proposed. Factions are met through voyage and encounter text only; no cue, music or voice (holds stand).

## 7. Links

- Ports: [Stormwall](../ports/stormwall.md), [Ironhaven](../ports/ironhaven.md)
- Captains: [Sergeant Kruze](../captains/pirates/sergeant_kruze.md), [Gnaw](../captains/pirates/gnaw.md)

## 8. Flags

- Home port is canon only (DF-17).
- An Iron Wolves captain trades at Coral Throne (C:11335), far outside the North Atlantic territory (C:2907-2909). Resolved in lore: Wolves sell anywhere and raid only at home. No data change.
