# The Iron Strait (iron_strait)

Base commit: ab03ace (content.json, sea_culture.rs, voyage.rs and view.rs as cited). Status: draft canon, phase 3. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Name | The Iron Strait | C:1978 |
| Ends | [Ironhaven](../ports/ironhaven.md) - [Stormwall](../ports/stormwall.md) (undirected) | C:1973-1974 |
| Regions | North Atlantic - North Atlantic | port records |
| Distance | 20 | C:1975 |
| Danger | 0.09 | C:1976 |
| Minimum class | sloop | C:1977 |
| Days at minimum-class speed | 2 (distance 20 / speed 8, Python rounding) | voyage.rs:726-728; C:137 |
| Chart label | "Stormwall (The Iron Strait)  20  danger 0.09  sloop  N days" when docked at Ironhaven, and the mirror from Stormwall | crates/portlight-chart/src/view.rs:213-223, 327-334 |
| Godot | the label is drawn as the chart hover text | crates/portlight-godot/src/chart_canvas.rs:228, 444 |
| Encounter table | 4 entries keyed by the name | C:12315; sea_culture.rs:121-136 |
| Mechanic | the name selects the route encounter table (25% roll per voyage, sea_culture.rs:125); otherwise flavour | sea_culture.rs:30-36 |

## 1. Data cites

- lore: "Iron from the foundries, soldiers from the fortress. This lane never sleeps." [ascii-folded] (C:1979)
- encounter (sighting): "A Stormwall patrol ship passes - grey hull, no flag of welcome. The crew watches you with professional interest. They're always watching." [ascii-folded] (C:12318)
- encounter (sighting): "Iron barges from the foundry ride low in the water, sparks still drifting from the cargo. The Great Foundry's chimney glows on the horizon." [ascii-folded] (C:12323)
- encounter (weather): "The water here is dark - iron runoff from the foundries. Your navigator says it's deeper than it looks. The iron goes all the way down." [ascii-folded] (C:12328)
- encounter (lore): "An old hand on deck says: 'I served at Stormwall once. The strait's the only thing between the north and... whatever's out there.' He doesn't finish." [ascii-folded] (C:12333)

## 2. Course and waters (all [invented] unless cited)

A narrow northern strait between the foundry town and the fortress; dark, deep water stained by foundry runoff (C:12328). The Great Foundry's chimney glow marks the Ironhaven end (C:12323).

No lane has coordinates or a drawn path; the chart joins the two port map x/y points with a straight segment (view.rs:200-210). Any drawn course is a future phase.

## 3. History (all [invented] unless cited)

- T3 (~500 BS): the Stormwall Fortress is raised and the strait becomes a guarded passage.
- T11 (~60 BS): the Iron Pact puts foundry iron and fortress soldiers on one lane; 'This lane never sleeps' (C:1979).

## 4. People and traffic (all [invented] unless cited)

Grey Stormwall patrol ships (C:12318), low-riding iron barges (C:12323), soldiers rotating to the fortress.

## 5. Sound hook (all [invented] sound design unless quoted from a cite)

- Heavy hulls in chop, a patrol's oars in time, far forge thuds carried on wind (atmosphere).
- Lanes are heard at sea only through the voyage text. Ambience only, never a cue; never layered on sfx_chart_sail_depart or any arrival cue. No music, no voice; the holds stand (README). Fragment candidates: none proposed [proposal].

## 6. Links

- Ports: [Ironhaven](../ports/ironhaven.md), [Stormwall](../ports/stormwall.md)
- Landmarks: [The Great Foundry](../landmarks/great_foundry.md), [The Fortress](../landmarks/stormwall_fortress.md)

## 7. Flags

- None.
