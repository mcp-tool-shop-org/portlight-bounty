# The Grain Road (grain_road)

Base commit: ab03ace (content.json, sea_culture.rs, voyage.rs and view.rs as cited). Status: draft canon, phase 3. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Name | The Grain Road | C:1933 |
| Ends | [Porto Novo](../ports/porto_novo.md) - [Al-Manar](../ports/al_manar.md) (undirected) | C:1928-1929 |
| Regions | Mediterranean - Mediterranean | port records |
| Distance | 24 | C:1930 |
| Danger | 0.12 | C:1931 |
| Minimum class | sloop | C:1932 |
| Days at minimum-class speed | 3 (distance 24 / speed 8, Python rounding) | voyage.rs:726-728; C:137 |
| Chart label | "Al-Manar (The Grain Road)  24  danger 0.12  sloop  N days" when docked at Porto Novo, and the mirror from Al-Manar | crates/portlight-chart/src/view.rs:213-223, 327-334 |
| Godot | the label is drawn as the chart hover text | crates/portlight-godot/src/chart_canvas.rs:228, 444 |
| Encounter table | 9 entries keyed by the name | C:12210; sea_culture.rs:121-136 |
| Mechanic | the name selects the route encounter table (25% roll per voyage, sea_culture.rs:125); otherwise flavour | sea_culture.rs:30-36 |

## 1. Data cites

- lore: "The oldest trade route in the Mediterranean. Grain ships have sailed this lane since before memory." [ascii-folded] (C:1934)
- encounter (sighting): "A grain convoy from Porto Novo passes - six barges in formation, hulls riding low. The Exchange Guild flag snaps in the breeze." [ascii-folded] (C:12213)
- encounter (sighting): "A felucca from Al-Manar crosses your wake, its deck stacked with brass-bound spice chests. The crew waves." [ascii-folded] (C:12218)
- encounter (sighting): "You pass the marker buoy where the old coast road meets the sea. Grain ships have used this lane since before the Exchange was built." [ascii-folded] (C:12223)
- encounter (lore): "Your bosun points at the coastline. 'See that cliff? They say the first Grain Exchange was carved into it - before Porto Novo even had a harbor.'" [ascii-folded] (C:12228)
- encounter (lore): "An old sailor on deck murmurs a prayer as you pass a rocky outcrop. 'Every grain ship salutes the Stone. It's bad luck not to.' You can't see why the rock matters. He won't explain." [ascii-folded] (C:12233)
- encounter (weather): "The Mediterranean breeze carries the smell of baking bread from the coastal villages. The crew breathes deep." [ascii-folded] (C:12238)
- encounter (weather): "Calm water, clear sky. The Grain Road earns its reputation as the gentlest route in the Known World." [ascii-folded] (C:12243)
- encounter (crew): "The crew is relaxed - they know this route. Someone starts humming a Porto Novo dock song. Others join." [ascii-folded] (C:12248)
- encounter (crew): "Your navigator corrects course by a fraction. 'The current shifts here every spring,' he says. 'Old knowledge.'" [ascii-folded] (C:12253)

## 2. Course and waters (all [invented] unless cited)

A coast-hugging run along the Middle Sea shore, never long out of sight of land. Marker buoys mark where the old coast road meets the sea (C:12223). Calm water and a spring current shift the navigators know by heart (C:12243, C:12253).

No lane has coordinates or a drawn path; the chart joins the two port map x/y points with a straight segment (view.rs:200-210). Any drawn course is a future phase.

## 3. History (all [invented] unless cited)

- T24 (~600 BS): grain ships first sail the coast between the two ports; 'since before memory' (C:1934). Placed before T4 so the Grain Road is older than both the Exchange hall and the roofed bazaar.
- The bosun's tale of a first Grain Exchange carved into a cliff (C:12228) is read as an older open-air counting place used before the columned hall rose on the temple ruins (T4; ../landmarks/grain_exchange.md). Resolved in lore.
- The Stone that every grain ship salutes (C:12233) is a rocky outcrop with no stated reason; canon keeps it unexplained [invented: it stays a mystery by design].

## 4. People and traffic (all [invented] unless cited)

Exchange Guild grain convoys and Al-Manar feluccas with brass-bound spice chests (C:12213, C:12218). Crews sing Porto Novo dock songs on this run (C:12248).

## 5. Sound hook (all [invented] sound design unless quoted from a cite)

- Lapping calm water, rigging at rest, a distant convoy's hulls creaking (atmosphere). The dock song (C:12248) is music NOTE ONLY, on hold.
- Lanes are heard at sea only through the voyage text. Ambience only, never a cue; never layered on sfx_chart_sail_depart or any arrival cue. No music, no voice; the holds stand (README). Fragment candidates: none proposed yet [proposal; no fragment or asset exists yet].

## 6. Links

- Ports: [Porto Novo](../ports/porto_novo.md), [Al-Manar](../ports/al_manar.md)
- Landmarks: [The Grain Exchange](../landmarks/grain_exchange.md), [The Spice Bazaar](../landmarks/spice_bazaar.md), [The beacon of Porto Novo](../landmarks/porto_novo_beacon.md)

## 7. Flags

- 'the gentlest route in the Known World' (C:12243) is not the lowest danger in data: DF-23.
- Cliff-carved first Exchange (C:12228) vs temple ruins (C:12011): resolved in lore above.
