# The Tea and Tobacco Road (tea_and_tobacco_road)

Base commit: ab03ace (content.json, sea_culture.rs, voyage.rs and view.rs as cited). Status: draft canon, phase 3. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Name | The Tea and Tobacco Road | C:1996 |
| Ends | [Stormwall](../ports/stormwall.md) - [Thornport](../ports/thornport.md) (undirected) | C:1991-1992 |
| Regions | North Atlantic - North Atlantic | port records |
| Distance | 18 | C:1993 |
| Danger | 0.1 | C:1994 |
| Minimum class | sloop | C:1995 |
| Days at minimum-class speed | 2 (distance 18 / speed 8, Python rounding) | voyage.rs:726-728; C:137 |
| Chart label | "Thornport (The Tea and Tobacco Road)  18  danger 0.10  sloop  N days" when docked at Stormwall, and the mirror from Thornport | crates/portlight-chart/src/view.rs:213-223, 327-334 |
| Godot | the label is drawn as the chart hover text | crates/portlight-godot/src/chart_canvas.rs:228, 444 |
| Encounter table | 4 entries keyed by the name | C:12340; sea_culture.rs:121-136 |
| Mechanic | the name selects the route encounter table (25% roll per voyage, sea_culture.rs:125); otherwise flavour | sea_culture.rs:30-36 |

## 1. Data cites

- lore: "Thornport's warmth heading to Stormwall's cold. The northern comfort trade." [ascii-folded] (C:1997)
- encounter (sighting): "A Thornport whaler - converted to trading - passes with tea crates stacked on deck. The crew smokes pipes and waves cheerfully." [ascii-folded] (C:12343)
- encounter (sighting): "The Whale Arch is visible from here - a bleached jawbone spanning the harbor entrance. Even at this distance, you can see the names carved in it." [ascii-folded] (C:12348)
- encounter (crew): "Your cook brews tea with leaves bought at Thornport. The crew gathers. For a moment, the northern sea feels almost warm." [ascii-folded] (C:12353)
- encounter (weather): "Fog rolls in. A horn sounds from somewhere ahead - Thornport's harbor signal. The old whalers' way of saying 'you're close.'" [ascii-folded] (C:12358)

## 2. Course and waters (all [invented] unless cited)

A fog-prone northern coastal run. The Whale Arch is visible from the lane (C:12348) and a fog horn tells crews they are near Thornport (C:12358).

No lane has coordinates or a drawn path; the chart joins the two port map x/y points with a straight segment (view.rs:200-210). Any drawn course is a future phase.

## 3. History (all [invented] unless cited)

- T9 (~90 BS): the whaling era ends and whalers convert to trading (C:12343).
- T14 (~25 BS): Stormwall's two-week tea tax on Thornport (README T14) is remembered on this road.

## 4. People and traffic (all [invented] unless cited)

Converted whalers with tea crates on deck, crews smoking pipes (C:12343).

## 5. Sound hook (all [invented] sound design unless quoted from a cite)

- Fog-damp quiet, Thornport's harbour horn far off (C:12358; ambience only, never a cue), a kettle on the cook's stove.
- Lanes are heard at sea only through the voyage text. Ambience only, never a cue; never layered on sfx_chart_sail_depart or any arrival cue. No music, no voice; the holds stand (README). Fragment candidates: none proposed [proposal].

## 6. Links

- Ports: [Stormwall](../ports/stormwall.md), [Thornport](../ports/thornport.md)
- Landmarks: [The Whale Arch](../landmarks/whale_arch.md), [The Fortress](../landmarks/stormwall_fortress.md)

## 7. Flags

- The Thornport fog horn (C:12358) is a third Thornport sound beside the port bed; no conflict with phase 1 or 2 sound notes. No data flag.
