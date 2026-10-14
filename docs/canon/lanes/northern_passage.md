# The Northern Passage (northern_passage)

Base commit: ab03ace (content.json, sea_culture.rs, voyage.rs and view.rs as cited). Status: draft canon, phase 3. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Name | The Northern Passage | C:2302 |
| Ends | [Ironhaven](../ports/ironhaven.md) - [Jade Port](../ports/jade_port.md) (undirected) | C:2297-2298 |
| Regions | North Atlantic - East Indies | port records |
| Distance | 76 | C:2299 |
| Danger | 0.22 | C:2300 |
| Minimum class | galleon | C:2301 |
| Days at minimum-class speed | 19 (distance 76 / speed 4, Python rounding) | voyage.rs:726-728; C:182 |
| Chart label | "Jade Port (The Northern Passage)  76  danger 0.22  galleon  N days" when docked at Ironhaven, and the mirror from Jade Port | crates/portlight-chart/src/view.rs:213-223, 327-334 |
| Godot | the label is drawn as the chart hover text | crates/portlight-godot/src/chart_canvas.rs:228, 444 |
| Encounter table | 5 entries keyed by the name | C:12615; sea_culture.rs:121-136 |
| Mechanic | the name selects the route encounter table (25% roll per voyage, sea_culture.rs:125); otherwise flavour | sea_culture.rs:30-36 |

## 1. Data cites

- lore: "Iron west, porcelain east. The longest profitable route in the world." [ascii-folded] (C:2303)
- encounter (lore): "The longest profitable route in the world. Iron west, porcelain east. Your hold tells the story of two civilizations that need each other." [ascii-folded] (C:12618)
- encounter (weather): "Ice. Not dangerous - floating crystals from the northern edge of the passage. Beautiful and foreign. Your crew collects pieces as souvenirs." [ascii-folded] (C:12623)
- encounter (sighting): "A whale surfaces alongside - massive, slow, unconcerned. It travels the same passage, following the same currents, and has done so longer than any ship." [ascii-folded] (C:12628)
- encounter (crew): "Week three. The provisions are holding, the crew is tired, and the passage stretches on. Your navigator says halfway. He's been saying halfway for two days." [ascii-folded] (C:12633)
- encounter (weather): "You spot Ironhaven's foundry glow on the western horizon - or is it Jade Port's kilns on the eastern? On the Northern Passage, both ends look the same from the middle." [ascii-folded] (C:12638)

## 2. Course and waters (all [invented] unless cited)

Long-haul open ocean along the northern ice edge from Ironhaven to Jade Port (DF-21); floating ice crystals (C:12623), whales on the same currents (C:12628).

No lane has coordinates or a drawn path; the chart joins the two port map x/y points with a straight segment (view.rs:200-210). Any drawn course is a future phase.

## 3. History (all [invented] unless cited)

- T29 (~55 BS): after the Iron Pact forms (T11), Ironhaven iron first goes east by this passage; 'Iron west, porcelain east' (C:2303).

## 4. People and traffic (all [invented] unless cited)

Galleon crews three weeks out (C:12633); both ends glow alike from the middle (C:12638).

## 5. Sound hook (all [invented] sound design unless quoted from a cite)

- Ice tinkle against the hull, whale breath, long swell (atmosphere).
- Lanes are heard at sea only through the voyage text. Ambience only, never a cue; never layered on sfx_chart_sail_depart or any arrival cue. No music, no voice; the holds stand (README). Fragment candidates: none proposed [proposal].

## 6. Links

- Ports: [Ironhaven](../ports/ironhaven.md), [Jade Port](../ports/jade_port.md)
- Landmarks: [The Great Foundry](../landmarks/great_foundry.md), [The Porcelain Quarter](../landmarks/porcelain_quarter.md)

## 7. Flags

- 'The longest profitable route in the world' (C:2303, C:12618): The Smuggler's Run is longer (80 vs 76). DF-25.
