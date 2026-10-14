# Typhoon Alley (typhoon_alley)

Base commit: ab03ace (content.json, sea_culture.rs, voyage.rs and view.rs as cited). Status: draft canon, phase 3. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Name | Typhoon Alley | C:2230 |
| Ends | [Monsoon Reach](../ports/monsoon_reach.md) - [Typhoon Anchorage](../ports/typhoon_anchorage.md) (undirected) | C:2225-2226 |
| Regions | East Indies - South Seas | port records |
| Distance | 52 | C:2227 |
| Danger | 0.2 | C:2228 |
| Minimum class | galleon | C:2229 |
| Days at minimum-class speed | 13 (distance 52 / speed 4, Python rounding) | voyage.rs:726-728; C:182 |
| Chart label | "Typhoon Anchorage (Typhoon Alley)  52  danger 0.20  galleon  N days" when docked at Monsoon Reach, and the mirror from Typhoon Anchorage | crates/portlight-chart/src/view.rs:213-223, 327-334 |
| Godot | the label is drawn as the chart hover text | crates/portlight-godot/src/chart_canvas.rs:228, 444 |
| Encounter table | 6 entries keyed by the name | C:12485; sea_culture.rs:121-136 |
| Mechanic | the name selects the route encounter table (25% roll per voyage, sea_culture.rs:125); otherwise flavour | sea_culture.rs:30-36 |

## 1. Data cites

- lore: "Named for obvious reasons. The monsoon winds funnel through here like a gauntlet. Timing is survival." [ascii-folded] (C:2231)
- encounter (weather): "The pressure drops. Your ears pop. The sky turns a color you've never seen - yellow-grey-green. Typhoon Alley earns its name." [ascii-folded] (C:12488)
- encounter (lore): "A wrecked hull drifts past - no mast, no crew, barnacles already growing. Someone's journey ended here. Your crew says nothing." [ascii-folded] (C:12493)
- encounter (sighting): "A storm rider's outrigger appears from nowhere, surfs the leading wave, and vanishes behind a squall. You're in their territory." [ascii-folded] (C:12498)
- encounter (crew): "Your ship groans. The hull flexes in ways the shipwright intended - if she's from Monsoon Reach or Typhoon Anchorage. If she's Mediterranean... you pray." [ascii-folded] (C:12503)
- encounter (lore): "Lightning illuminates something on the horizon - an island not on your charts. When dawn comes, it's gone. The crew doesn't discuss it." [ascii-folded] (C:12508)
- encounter (weather): "Rain so heavy it stings the skin. The crew works by touch. The compass spins. Your navigator switches to dead reckoning and doesn't sleep for two days." [ascii-folded] (C:12513)

## 2. Course and waters (all [invented] unless cited)

The monsoon funnel from Monsoon Reach south to Typhoon Anchorage (C:2231). Pressure drops, the sky turns yellow-grey-green (C:12488), compasses spin in rain (C:12513).

No lane has coordinates or a drawn path; the chart joins the two port map x/y points with a straight segment (view.rs:200-210). Any drawn course is a future phase.

## 3. History (all [invented] unless cited)

- T20 (~70 BS): Typhoon Anchorage lashes its first wrecks into the Storm Wall; many of them died in this alley [invented].
- The phantom island seen by lightning (C:12508) is kept unexplained.

## 4. People and traffic (all [invented] unless cited)

Storm riders in outriggers who surf the leading wave (C:12498); galleon crews only.

## 5. Sound hook (all [invented] sound design unless quoted from a cite)

- Wind howl, hull flex groan (C:12503), rain sheets (atmosphere). No pagoda bells or gongs (phase 2 ruling).
- Lanes are heard at sea only through the voyage text. Ambience only, never a cue; never layered on sfx_chart_sail_depart or any arrival cue. No music, no voice; the holds stand (README). Fragment candidates: none proposed [proposal].

## 6. Links

- Ports: [Monsoon Reach](../ports/monsoon_reach.md), [Typhoon Anchorage](../ports/typhoon_anchorage.md)
- Landmarks: [The Wind Temple](../landmarks/wind_temple.md), [The Storm Wall](../landmarks/storm_wall.md)

## 7. Flags

- None.
