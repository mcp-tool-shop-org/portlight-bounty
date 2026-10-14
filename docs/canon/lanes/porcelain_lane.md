# The Porcelain Lane (porcelain_lane)

Base commit: ab03ace (content.json, sea_culture.rs, voyage.rs and view.rs as cited). Status: draft canon, phase 3. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Name | The Porcelain Lane | C:2131 |
| Ends | [Crosswind Isle](../ports/crosswind_isle.md) - [Jade Port](../ports/jade_port.md) (undirected) | C:2126-2127 |
| Regions | East Indies - East Indies | port records |
| Distance | 28 | C:2128 |
| Danger | 0.1 | C:2129 |
| Minimum class | sloop | C:2130 |
| Days at minimum-class speed | 4 (distance 28 / speed 8, Python rounding) | voyage.rs:726-728; C:137 |
| Chart label | "Jade Port (The Porcelain Lane)  28  danger 0.10  sloop  N days" when docked at Crosswind Isle, and the mirror from Jade Port | crates/portlight-chart/src/view.rs:213-223, 327-334 |
| Godot | the label is drawn as the chart hover text | crates/portlight-godot/src/chart_canvas.rs:228, 444 |
| Encounter table | 4 entries keyed by the name | C:12435; sea_culture.rs:121-136 |
| Mechanic | the name selects the route encounter table (25% roll per voyage, sea_culture.rs:125); otherwise flavour | sea_culture.rs:30-36 |

## 1. Data cites

- lore: "From the free port to the kiln masters. Every piece of porcelain in the west passed through here." [ascii-folded] (C:2132)
- encounter (sighting): "A junk with the Kiln Masters' guild mark on its sail passes - its hold padded with straw. Every piece of porcelain in the west passed through this lane." [ascii-folded] (C:12438)
- encounter (crew): "Your crew handles the cargo more carefully here. In the East Indies, they've learned that porcelain commands respect - and that broken porcelain means broken trust." [ascii-folded] (C:12443)
- encounter (weather): "The water turns jade-green. Islands appear like scattered emeralds. You're in the Porcelain Lane - where the oldest trade in the East flows." [ascii-folded] (C:12448)
- encounter (encounter): "A fishing sampan pulls alongside. The fisherman offers you a fresh catch in exchange for news. In the East Indies, information is currency." [ascii-folded] (C:12453)

## 2. Course and waters (all [invented] unless cited)

An island-strewn eastern lane in jade-green water (C:12448), from the free port to the kiln town.

No lane has coordinates or a drawn path; the chart joins the two port map x/y points with a straight segment (view.rs:200-210). Any drawn course is a future phase.

## 3. History (all [invented] unless cited)

- T7 (~150 BS): the Silk Circle formalises; the kiln masters' junks begin regular runs to the free port.
- 'Every piece of porcelain in the west passed through here' (C:2132, C:12438): canon reads 'the west' as all western buyers, since Crosswind Isle is the east's door to them.

## 4. People and traffic (all [invented] unless cited)

Junks with the Kiln Masters' guild mark and straw-padded holds (C:12438); fishing sampans trading catch for news (C:12453).

## 5. Sound hook (all [invented] sound design unless quoted from a cite)

- Straw-packed crates shifting; light chop slapping among island rocks; a sampan's scull (atmosphere).
- Lanes are heard at sea only through the voyage text. Ambience only, never a cue; never layered on sfx_chart_sail_depart or any arrival cue. No music, no voice; the holds stand (README). Fragment candidates: none proposed yet [proposal; no fragment or asset exists yet].

## 6. Links

- Ports: [Crosswind Isle](../ports/crosswind_isle.md), [Jade Port](../ports/jade_port.md)
- Landmarks: [The Free Port Bell](../landmarks/free_port_bell.md), [The Porcelain Quarter](../landmarks/porcelain_quarter.md)

## 7. Flags

- 'where the oldest trade in the East flows' (C:12448) vs The Silk Road by Sea, 'the two oldest trades in the East' (C:2177). Compatible: porcelain is one of the two. No flag.
