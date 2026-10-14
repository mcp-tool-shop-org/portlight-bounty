# The Shadow Lane (shadow_lane)

Base commit: ab03ace (content.json, sea_culture.rs, voyage.rs and view.rs as cited). Status: draft canon, phase 3. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Name | The Shadow Lane | C:1960 |
| Ends | [Porto Novo](../ports/porto_novo.md) - [Corsair's Rest](../ports/corsairs_rest.md) (undirected) | C:1955-1956 |
| Regions | Mediterranean - Mediterranean | port records |
| Distance | 18 | C:1957 |
| Danger | 0.18 | C:1958 |
| Minimum class | sloop | C:1959 |
| Days at minimum-class speed | 2 (distance 18 / speed 8, Python rounding) | voyage.rs:726-728; C:137 |
| Chart label | "Corsair's Rest (The Shadow Lane)  18  danger 0.18  sloop  N days" when docked at Porto Novo, and the mirror from Corsair's Rest | crates/portlight-chart/src/view.rs:213-223, 327-334 |
| Godot | the label is drawn as the chart hover text | crates/portlight-godot/src/chart_canvas.rs:228, 444 |
| Encounter table | 5 entries keyed by the name | C:12285; sea_culture.rs:121-136 |
| Mechanic | the name selects the route encounter table (25% roll per voyage, sea_culture.rs:125); otherwise flavour | sea_culture.rs:30-36 |

## 1. Data cites

- lore: "Every merchant denies using it. Every merchant uses it. The line between trade and smuggling runs through these waters." [ascii-folded] (C:1961)
- encounter (sighting): "A ship passes with no flag flying. The crew watches you. You watch them. Nobody waves. This is the Shadow Lane." [ascii-folded] (C:12288)
- encounter (encounter): "Your lookout spots a rowboat tucked into the cliff shadow. Empty. Or is it? On the Shadow Lane, the difference matters." [ascii-folded] (C:12293)
- encounter (weather): "The coast here is riddled with coves - each one a potential anchorage, each one a potential ambush. Your helmsman hugs the deeper water." [ascii-folded] (C:12298)
- encounter (lore): "A crewman mutters: 'My cousin sailed this lane for years. Made a fortune. Then one day he didn't come back. Nobody asked questions. You don't, on the Shadow Lane.'" [ascii-folded] (C:12303)
- encounter (sighting): "You catch a whiff of torch smoke from a cliff face. Corsair's Rest is close - you can't see it, but you can smell it." [ascii-folded] (C:12308)

## 2. Course and waters (all [invented] unless cited)

A short coastal lane under cliffs riddled with coves; helmsmen hug the deeper water away from them (C:12298). The cliff smoke of Corsair's Rest is smelled before it is seen (C:12308).

No lane has coordinates or a drawn path; the chart joins the two port map x/y points with a straight segment (view.rs:200-210). Any drawn course is a future phase.

## 3. History (all [invented] unless cited)

- T13 (~40 BS): Porto Novo's strict code drives smugglers to Corsair's Rest and the lane gets its name [invented date; the rivalry is cited in README T13].
- Every merchant denies using it; every merchant uses it (C:1961).

## 4. People and traffic (all [invented] unless cited)

Flagless ships that do not wave (C:12288); rowboats in cliff shadow (C:12293). Nobody asks questions (C:12303).

## 5. Sound hook (all [invented] sound design unless quoted from a cite)

- Surf in sea caves, oars shipped quietly, a far torch hiss (atmosphere).
- Lanes are heard at sea only through the voyage text. Ambience only, never a cue; never layered on sfx_chart_sail_depart or any arrival cue. No music, no voice; the holds stand (README). Fragment candidates: none proposed [proposal].

## 6. Links

- Ports: [Porto Novo](../ports/porto_novo.md), [Corsair's Rest](../ports/corsairs_rest.md)
- Landmarks: [The Cliff Tavern](../landmarks/cliff_tavern.md), [The Grain Exchange](../landmarks/grain_exchange.md)

## 7. Flags

- Danger 0.18 is the highest of the Mediterranean-only lanes; consistent with the lore. No flag.
