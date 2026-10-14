# The Crimson Tide (crimson_tide)

Base commit: 9acf9e2 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 4. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Name | The Crimson Tide | C:2880 |
| Kind | pirate faction (`factions`) | C:2877 |
| Territory | Mediterranean, North Atlantic | C:2881 |
| Smuggler attitude | neutral | C:2885 |
| Captains | [Scarlet Ana](../captains/pirates/scarlet_ana.md), [The Butcher](../captains/pirates/the_butcher.md) | C:2915, C:2922 |
| Home | Corsair's Rest (README, Pirate home ports) [invented; DF-17] | - |
| Player sees | the name in the encounter panel and duel prompt; the custom captain 'Pirate faction' row | crates/portlight-godot/src/logic.rs:1089-1091, 2203-2212; game.rs:3016-3024 |

## 1. What the faction does in the sim

- Region pick: a pirate encounter prefers captains whose faction holds the destination region (voyage.rs:340-349; encounter.rs:96-105). With no faction in a region, the first faction in the list is used (encounter.rs:97-99).
- Attitude: a smuggler captain gets +5 toward this faction when the attitude is friendly, cooperative or respectful (encounter.rs:50-58). 'neutral' does not qualify.
- Warning at sea: low regard in the faction's territory can bring a crossing-the-bow warning (consequences.rs:59-66).
- A captain's home region falls back to the faction's first territory region (memory.rs:379-384).
- Relations: only `hostile` pairs are read, for journal beats (content.rs:1156-1170; narrative.rs:655, 672, 736). Spillover and vendetta trigger text are unread (DF-28).

## 2. Data cites

- Corsair's Rest power structure: "The Crimson Tide protects the cove through No One's authority, collecting tribute and settling disputes." [ascii-folded] (C:7650)
- No One, governor title 'The Tide's Voice' (C:7649, C:7770-7772): "the person who makes sure the Crimson Tide's interests are respected, tribute is collected, and disputes are settled before they become violence" (C:7776)
- Smuggler's Run encounter: "A Crimson Tide ship appears on the horizon - then another. Then a third. ... The Run belongs to them." [ascii-folded] (C:12588)
- Dimitri Andros refused to build the Tide a flagship (C:7084); the Smith made Scarlet Ana's sword (C:8037); Master Ink painted her a scroll that hangs in Corsair's Rest (C:10041).
- The player archetype The Corsair is titled 'Crimson Tide Captain' and starts with the Tide's regard at 25 (C:2473, C:2506) - see [The Corsair](../captains/corsair.md).

## 3. Relations with every other faction (hand-maintained from `faction_relationships`, C:13866)

| Faction | Disposition | Data text | Vendetta trigger (unread) | Cite |
|---|---|---|---|---|
| [The Monsoon Syndicate](monsoon_syndicate.md) | rival | "Business rivals who respect each other's territory - barely. The Tide wants to control the Mediterranean-to-East route. The Syndicate wants to control all Eastern trade. They cooperate when it suits them and undercut when it doesn't." [ascii-folded] | "Running opium for the Syndicate through Tide waters." | C:13870-13873 |
| [The Deep Reef Brotherhood](deep_reef_brotherhood.md) | hostile | "Fundamental incompatibility. The Tide runs a protection racket - pay or suffer. The Brotherhood answers to no one and attacks anyone who claims authority over the sea. They've been killing each other's captains for decades." [ascii-folded] | "Supplying weapons to the Tide that get used against Brotherhood ships." | C:13878-13881 |
| [The Iron Wolves](iron_wolves.md) | rival | "Former allies who split over territory. Both are organized and militaristic, but the Wolves left when the Tide's leader demanded tribute from their operations. Now they compete for the same weapons trade in overlapping waters." [ascii-folded] | "Selling black powder to the Tide that was meant for the Wolves." | C:13886-13889 |

Pirate factions have no relationship rows with the trade blocs. Their bloc ties are through ports only (Shadow Ports, C:13493).

## 4. History (all [invented] unless cited)

- T28 (~100 BS): one line of pirate kings holds both ends of the Smuggler's Run; the Tide claims descent from it [invented].
- T30 (~45 BS): the Tide and the Deep Reef Brotherhood begin killing each other's captains (README T30).
- T31 (~45 BS): No One deserts the garrison and comes to the Tide (README T31).
- T13 (~40 BS): the Tide settles at Corsair's Rest (README T13).
- T33-T34 (~30 to ~22 BS): the Iron Wolves sail with the Tide, then split over tribute (README T33, T34).

## 5. Character and customs (all [invented] unless cited)

A protection racket with manners: pay or suffer (C:13879), and the Rest exists because the Tide permits it (C:7680). Its captains are businesslike on the water - Scarlet Ana passes at distance on business elsewhere (C:12819). [invented: the Tide counts tribute as rent, never as theft.]

No faction has a flag, colour or device in data beyond quoted sighting text (for example 'Crimson pennants', C:12819). Canon adds none; the character hold stands.

## 6. Sound hook

None proposed. Factions are met through voyage and encounter text only; no cue, music or voice (holds stand).

## 7. Links

- Ports: [Corsair's Rest](../ports/corsairs_rest.md)
- Lanes: [The Shadow Lane](../lanes/shadow_lane.md), [The Smuggler's Run](../lanes/smugglers_run.md)
- Captains: [Scarlet Ana](../captains/pirates/scarlet_ana.md), [The Butcher](../captains/pirates/the_butcher.md)

## 8. Flags

- Home port is canon only (DF-17).
