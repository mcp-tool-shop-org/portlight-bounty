# The Smuggler's Run (smugglers_run)

Base commit: ab03ace (content.json, sea_culture.rs, voyage.rs and view.rs as cited). Status: draft canon, phase 3. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Name | The Smuggler's Run | C:2293 |
| Ends | [Corsair's Rest](../ports/corsairs_rest.md) - [Spice Narrows](../ports/spice_narrows.md) (undirected) | C:2288-2289 |
| Regions | Mediterranean - East Indies | port records |
| Distance | 80 | C:2290 |
| Danger | 0.25 | C:2291 |
| Minimum class | galleon | C:2292 |
| Days at minimum-class speed | 20 (distance 80 / speed 4, Python rounding) | voyage.rs:726-728; C:182 |
| Chart label | "Spice Narrows (The Smuggler's Run)  80  danger 0.25  galleon  N days" when docked at Corsair's Rest, and the mirror from Spice Narrows | crates/portlight-chart/src/view.rs:213-223, 327-334 |
| Godot | the label is drawn as the chart hover text | crates/portlight-godot/src/chart_canvas.rs:228, 444 |
| Encounter table | 6 entries keyed by the name | C:12580; sea_culture.rs:121-136 |
| Mechanic | the name selects the route encounter table (25% roll per voyage, sea_culture.rs:125); otherwise flavour | sea_culture.rs:30-36 |

## 1. Data cites

- lore: "Only the desperate or the bold sail this direct. Pirate kings once controlled both ends." [ascii-folded] (C:2294)
- encounter (encounter): "Darkness. You run without lights because on the Smuggler's Run, visibility is the enemy. Your crew communicates in whispers." [ascii-folded] (C:12583)
- encounter (sighting): "A Crimson Tide ship appears on the horizon - then another. Then a third. They're not pursuing you. They're watching. The Run belongs to them." [ascii-folded] (C:12588)
- encounter (encounter): "Your lookout spots a trail of floating debris - crates, barrels, a torn sail. Someone dumped cargo in a hurry. Navy patrol, probably. Your crew moves your contraband deeper into the hold." [ascii-folded] (C:12593)
- encounter (crew): "Eighty leagues of open water between two black market ports. The longest, most dangerous route in the Known World. Your crew knows the stakes. Nobody complains. Nobody jokes." [ascii-folded] (C:12598)
- encounter (lore): "An old pirate shanty drifts from below deck - someone's singing about the Run. The lyrics are about a captain who made the crossing so many times, the navy thought he was a ghost. He stopped correcting them." [ascii-folded] (C:12603)
- encounter (encounter): "The Syndicate's signal - a lantern flash from an unseen ship - briefly illuminates the darkness. Three short, one long. It means 'patrol ahead, alter course.' Your navigator adjusts without being told." [ascii-folded] (C:12608)

## 2. Course and waters (all [invented] unless cited)

The longest and most dangerous lane in data (80, 0.25), long-haul open ocean (DF-21), sailed without lights (C:12583).

No lane has coordinates or a drawn path; the chart joins the two port map x/y points with a straight segment (view.rs:200-210). Any drawn course is a future phase.

## 3. History (all [invented] unless cited)

- T28 (~100 BS): pirate kings hold both ends of the Run, 'once controlled both ends' (C:2294); their line ends before T13.
- T13 (~40 BS): the Crimson Tide (Corsair's Rest) and the Monsoon Syndicate (Spice Narrows) settle in; today the Tide watches the Run (C:12588) and the Syndicate signals on it (C:12608). Two crews now share what one king once held; this resolves 'once' in lore.

## 4. People and traffic (all [invented] unless cited)

Crimson Tide watchers, Syndicate signal lanterns ('three short, one long', C:12608), crews in whispers.

## 5. Sound hook (all [invented] sound design unless quoted from a cite)

- Muffled oars; hull in dark swell; a shuttered lantern's slide (atmosphere). Voice NOTE ONLY: whispered orders [invented], on hold. The shanty (C:12603) is music NOTE ONLY.
- Lanes are heard at sea only through the voyage text. Ambience only, never a cue; never layered on sfx_chart_sail_depart or any arrival cue. No music, no voice; the holds stand (README). Fragment candidates: none proposed yet [proposal; no fragment or asset exists yet].

## 6. Links

- Ports: [Corsair's Rest](../ports/corsairs_rest.md), [Spice Narrows](../ports/spice_narrows.md)
- Landmarks: [The Cliff Tavern](../landmarks/cliff_tavern.md), [The Hanging Market](../landmarks/hanging_market.md)

## 7. Flags

- 'Eighty leagues' (C:12598) matches distance 80 and the claim 'longest, most dangerous' holds in data. No flag.
- Shanty (C:12603) on music hold.
