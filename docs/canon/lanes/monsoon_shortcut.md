# The Monsoon Shortcut (monsoon_shortcut)

Base commit: ab03ace (content.json, sea_culture.rs, voyage.rs and view.rs as cited). Status: draft canon, phase 3. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Name | The Monsoon Shortcut | C:2284 |
| Ends | [Al-Manar](../ports/al_manar.md) - [Monsoon Reach](../ports/monsoon_reach.md) (undirected) | C:2279-2280 |
| Regions | Mediterranean - East Indies | port records |
| Distance | 72 | C:2281 |
| Danger | 0.22 | C:2282 |
| Minimum class | galleon | C:2283 |
| Days at minimum-class speed | 18 (distance 72 / speed 4, Python rounding) | voyage.rs:726-728; C:182 |
| Chart label | "Monsoon Reach (The Monsoon Shortcut)  72  danger 0.22  galleon  N days" when docked at Al-Manar, and the mirror from Monsoon Reach | crates/portlight-chart/src/view.rs:213-223, 327-334 |
| Godot | the label is drawn as the chart hover text | crates/portlight-godot/src/chart_canvas.rs:228, 444 |
| Encounter table | 5 entries keyed by the name | C:12550; sea_culture.rs:121-136 |
| Mechanic | the name selects the route encounter table (25% roll per voyage, sea_culture.rs:125); otherwise flavour | sea_culture.rs:30-36 |

## 1. Data cites

- lore: "Timing is everything. Catch the monsoon right and you fly. Catch it wrong and you drown." [ascii-folded] (C:2285)
- encounter (weather): "You catch the monsoon wind and FLY. The ship heels over, sails taut, crew bracing. This is why they call it the Shortcut - if you time it right, nothing's faster." [ascii-folded] (C:12553)
- encounter (weather): "The wind dies. Completely. You're becalmed in open water between the Mediterranean and the East Indies. The crew stares at limp sails." [ascii-folded] (C:12558)
- encounter (lore): "A merchant vessel that attempted the Shortcut last season drifts past. Abandoned. The log is gone. Whatever happened, nobody recorded it." [ascii-folded] (C:12563)
- encounter (lore): "Your navigator pulls out the monsoon charts - handwritten, passed down from Farouk in Al-Manar's tea house. 'He sailed this seven times,' your navigator says. 'Trust the charts.'" [ascii-folded] (C:12568)
- encounter (encounter): "Wind shifts. The monsoon is changing direction - a day early. Your navigator adjusts instantly. The margin between 'Shortcut' and 'death sentence' is measured in hours." [ascii-folded] (C:12573)

## 2. Course and waters (all [invented] unless cited)

Long-haul open ocean from Al-Manar to Monsoon Reach (DF-21). Fast with the monsoon behind, deadly when becalmed (C:12553, C:12558).

No lane has coordinates or a drawn path; the chart joins the two port map x/y points with a straight segment (view.rs:200-210). Any drawn course is a future phase.

## 3. History (all [invented] unless cited)

- T14 (~25 BS): the Shortcut links Al-Manar to Monsoon Reach (README).
- T27 (~30 BS): Old Farouk sails it seven times and his hand-drawn monsoon charts are copied by navigators (C:12568; Farouk's tea house C:7182).

## 4. People and traffic (all [invented] unless cited)

Galleon crews with copied charts; abandoned hulls of last season's failures (C:12563).

## 5. Sound hook (all [invented] sound design unless quoted from a cite)

- Taut canvas and heeling hull at speed; spray over the bow; rigging creaking under monsoon load (atmosphere).
- Lanes are heard at sea only through the voyage text. Ambience only, never a cue; never layered on sfx_chart_sail_depart or any arrival cue. No music, no voice; the holds stand (README). Fragment candidates: none proposed yet [proposal; no fragment or asset exists yet].

## 6. Links

- Ports: [Al-Manar](../ports/al_manar.md), [Monsoon Reach](../ports/monsoon_reach.md)
- Landmarks: [The Spice Bazaar](../landmarks/spice_bazaar.md), [The Wind Temple](../landmarks/wind_temple.md)

## 7. Flags

- The navigator's line 'between the Mediterranean and the East Indies' (C:12558) agrees with the DF-21 long-haul reading. No new flag.
