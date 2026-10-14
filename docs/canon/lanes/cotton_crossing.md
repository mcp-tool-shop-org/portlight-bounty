# The Cotton Crossing (cotton_crossing)

Base commit: ab03ace (content.json, sea_culture.rs, voyage.rs and view.rs as cited). Status: draft canon, phase 3. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Name | The Cotton Crossing | C:2032 |
| Ends | [Porto Novo](../ports/porto_novo.md) - [Sun Harbor](../ports/sun_harbor.md) (undirected) | C:2027-2028 |
| Regions | Mediterranean - West Africa | port records |
| Distance | 40 | C:2029 |
| Danger | 0.12 | C:2030 |
| Minimum class | cutter | C:2031 |
| Days at minimum-class speed | 4 (distance 40 / speed 9, Python rounding) | voyage.rs:726-728; C:152 |
| Chart label | "Sun Harbor (The Cotton Crossing)  40  danger 0.12  cutter  N days" when docked at Porto Novo, and the mirror from Sun Harbor | crates/portlight-chart/src/view.rs:213-223, 327-334 |
| Godot | the label is drawn as the chart hover text | crates/portlight-godot/src/chart_canvas.rs:228, 444 |
| Encounter table | 5 entries keyed by the name | C:12365; sea_culture.rs:121-136 |
| Mechanic | the name selects the route encounter table (25% roll per voyage, sea_culture.rs:125); otherwise flavour | sea_culture.rs:30-36 |

## 1. Data cites

- lore: "Mediterranean grain south, Gold Coast cotton north. The route that clothed an empire." [ascii-folded] (C:2033)
- encounter (weather): "The water changes color as you leave the Mediterranean - warmer, greener. The Gold Coast is ahead. The air carries the faintest scent of earth." [ascii-folded] (C:12368)
- encounter (sighting): "A cotton trader's vessel passes, bales stacked so high the deck is barely visible. Women in indigo cloth stand at the rail, singing the count." [ascii-folded] (C:12373)
- encounter (crew): "Your crew's mood shifts as the coast changes. The northern rigidity fades. Someone laughs. Someone else starts a story. The Gold Coast does this to people." [ascii-folded] (C:12378)
- encounter (encounter): "A pod of dolphins escorts you for an hour. Your bosun says it's a sign of welcome from the coast. The crew believes him." [ascii-folded] (C:12383)
- encounter (sighting): "You pass a merchant vessel flying the Exchange Alliance flag, heading north. 'Cotton north, grain south,' your navigator says. 'Been this way for centuries.'" [ascii-folded] (C:12388)

## 2. Course and waters (all [invented] unless cited)

The first lane to leave the Middle Sea southward; the water turns warmer and greener on the way to the Gold Coast (C:12368).

No lane has coordinates or a drawn path; the chart joins the two port map x/y points with a straight segment (view.rs:200-210). Any drawn course is a future phase.

## 3. History (all [invented] unless cited)

- T25 (~250 BS): grain south, cotton north becomes a regular exchange; 'Been this way for centuries' (C:12388), 'the route that clothed an empire' (C:2033).
- T10 (~80 BS): the Gold Coast Compact forms at the southern end.

## 4. People and traffic (all [invented] unless cited)

Cotton traders with bales stacked high and women in indigo singing the count (C:12373); Exchange Alliance merchants heading north (C:12388); dolphins (C:12383).

## 5. Sound hook (all [invented] sound design unless quoted from a cite)

- Open-water swell, bale lashings creaking, dolphins blowing (atmosphere). The count sung (C:12373) is voice NOTE ONLY, on hold (as for the Cotton Steps).
- Lanes are heard at sea only through the voyage text. Ambience only, never a cue; never layered on sfx_chart_sail_depart or any arrival cue. No music, no voice; the holds stand (README). Fragment candidates: none proposed yet [proposal; no fragment or asset exists yet].

## 6. Links

- Ports: [Porto Novo](../ports/porto_novo.md), [Sun Harbor](../ports/sun_harbor.md)
- Landmarks: [The Cotton Steps](../landmarks/cotton_steps.md), [The Grain Exchange](../landmarks/grain_exchange.md)

## 7. Flags

- 'the route that clothed an empire' (C:2033): no empire exists in data. Read as a figure of speech for the Exchange Alliance trade; resolved in lore.
