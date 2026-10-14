# The Silk Road by Sea (silk_road_by_sea)

Base commit: ab03ace (content.json, sea_culture.rs, voyage.rs and view.rs as cited). Status: draft canon, phase 3. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Name | The Silk Road by Sea | C:2176 |
| Ends | [Jade Port](../ports/jade_port.md) - [Silk Haven](../ports/silk_haven.md) (undirected) | C:2171-2172 |
| Regions | East Indies - East Indies | port records |
| Distance | 18 | C:2173 |
| Danger | 0.07 | C:2174 |
| Minimum class | sloop | C:2175 |
| Days at minimum-class speed | 2 (distance 18 / speed 8, Python rounding) | voyage.rs:726-728; C:137 |
| Chart label | "Silk Haven (The Silk Road by Sea)  18  danger 0.07  sloop  N days" when docked at Jade Port, and the mirror from Silk Haven | crates/portlight-chart/src/view.rs:213-223, 327-334 |
| Godot | the label is drawn as the chart hover text | crates/portlight-godot/src/chart_canvas.rs:228, 444 |
| Encounter table | 4 entries keyed by the name | C:12460; sea_culture.rs:121-136 |
| Mechanic | the name selects the route encounter table (25% roll per voyage, sea_culture.rs:125); otherwise flavour | sea_culture.rs:30-36 |

## 1. Data cites

- lore: "Where porcelain meets silk. The two oldest trades in the East, connected by the shortest route." [ascii-folded] (C:2177)
- encounter (sighting): "A silk convoy glides past - three junks in formation, their sails painted with the Weavers' Guild pattern. The silk inside is worth more than your ship." [ascii-folded] (C:12463)
- encounter (weather): "Incense drifts from a passing vessel. In the Silk Waters, even the cargo smells beautiful." [ascii-folded] (C:12468)
- encounter (crew): "Your crew folds their clothes more carefully after a few days in these waters. The East Indies changes how people treat fabric." [ascii-folded] (C:12473)
- encounter (encounter): "A sampan merchant offers a bolt of silk through your porthole while you're anchored for the night. The price is suspiciously good. Your bosun advises caution." [ascii-folded] (C:12478)

## 2. Course and waters (all [invented] unless cited)

The short, sheltered lane between the kilns and the looms; incense drifts from passing hulls (C:12468).

No lane has coordinates or a drawn path; the chart joins the two port map x/y points with a straight segment (view.rs:200-210). Any drawn course is a future phase.

## 3. History (all [invented] unless cited)

- T1 (~1000 BS): the Loom Quarter's patterns are first woven; silk travels this water from then on.
- T7 (~150 BS): the Silk Circle formalises the trade.

## 4. People and traffic (all [invented] unless cited)

Silk convoys of three junks with the Weavers' Guild pattern on their sails (C:12463); sampan merchants selling silk through portholes at night (C:12478).

## 5. Sound hook (all [invented] sound design unless quoted from a cite)

- Junk battens creaking, soft silk-wrapped cargo, quiet water (atmosphere).
- Lanes are heard at sea only through the voyage text. Ambience only, never a cue; never layered on sfx_chart_sail_depart or any arrival cue. No music, no voice; the holds stand (README). Fragment candidates: none proposed [proposal].

## 6. Links

- Ports: [Jade Port](../ports/jade_port.md), [Silk Haven](../ports/silk_haven.md)
- Landmarks: [The Porcelain Quarter](../landmarks/porcelain_quarter.md), [The Loom Quarter](../landmarks/loom_quarter.md)

## 7. Flags

- 'connected by the shortest route' (C:2177): 18 is the shortest East Indies-only named lane; unnamed lanes not checked as rivals. Safest lane in the game (0.07). No flag.
