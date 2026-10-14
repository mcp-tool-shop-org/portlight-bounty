# The Deep South Run (deep_south_run)

Base commit: ab03ace (content.json, sea_culture.rs, voyage.rs and view.rs as cited). Status: draft canon, phase 3. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Name | The Deep South Run | C:2311 |
| Ends | [Pearl Shallows](../ports/pearl_shallows.md) - [Coral Throne](../ports/coral_throne.md) (undirected) | C:2306-2307 |
| Regions | West Africa - South Seas | port records |
| Distance | 68 | C:2308 |
| Danger | 0.2 | C:2309 |
| Minimum class | galleon | C:2310 |
| Days at minimum-class speed | 17 (distance 68 / speed 4, Python rounding) | voyage.rs:726-728; C:182 |
| Chart label | "Coral Throne (The Deep South Run)  68  danger 0.20  galleon  N days" when docked at Pearl Shallows, and the mirror from Coral Throne | crates/portlight-chart/src/view.rs:213-223, 327-334 |
| Godot | the label is drawn as the chart hover text | crates/portlight-godot/src/chart_canvas.rs:228, 444 |
| Encounter table | 5 entries keyed by the name | C:12645; sea_culture.rs:121-136 |
| Mechanic | the name selects the route encounter table (25% roll per voyage, sea_culture.rs:125); otherwise flavour | sea_culture.rs:30-36 |

## 1. Data cites

- lore: "Pearl to pearl. The divers of the shallows and the divers of the reef - same craft, different kingdoms." [ascii-folded] (C:2312)
- encounter (lore): "Pearl to pearl. The divers of the shallows and the divers of the reef - same craft, different kingdoms. Your cargo carries the weight of both traditions." [ascii-folded] (C:12648)
- encounter (weather): "Turquoise water so clear you can see the reef twenty feet below. Beautiful and deadly. One wrong turn and the coral claims your hull." [ascii-folded] (C:12653)
- encounter (sighting): "A war canoe from Coral Throne appears. Warriors in painted armor watch you pass. One raises a spear - not in threat but in salute. You've been recognized." [ascii-folded] (C:12658)
- encounter (lore): "Your crew spots something glinting in the shallows - a pearl, dropped or thrown. Nobody dives for it. On the Deep South Run, taking what the reef didn't offer is bad luck." [ascii-folded] (C:12663)
- encounter (encounter): "Ceremonial drums carry across the water from an island you can't see. The Coral Kingdom is celebrating something. Your crew listens. The rhythm is infectious." [ascii-folded] (C:12668)

## 2. Course and waters (all [invented] unless cited)

Clear turquoise reef water from Pearl Shallows to Coral Throne; one wrong turn and the coral claims the hull (C:12653).

No lane has coordinates or a drawn path; the chart joins the two port map x/y points with a straight segment (view.rs:200-210). Any drawn course is a future phase.

## 3. History (all [invented] unless cited)

- T2 (~700 BS): the Coral Palace begins to grow at the southern end.
- 'Pearl to pearl' (C:2312): diving is one craft in two kingdoms. Taking an unoffered pearl is bad luck (C:12663).

## 4. People and traffic (all [invented] unless cited)

Coral Throne war canoes that salute known ships (C:12658); divers of both kingdoms.

## 5. Sound hook (all [invented] sound design unless quoted from a cite)

- Reef surf, paddles, distant ceremonial drums (C:12668; music NOTE ONLY, as for the Coral Palace).
- Lanes are heard at sea only through the voyage text. Ambience only, never a cue; never layered on sfx_chart_sail_depart or any arrival cue. No music, no voice; the holds stand (README). Fragment candidates: none proposed yet [proposal; no fragment or asset exists yet].

## 6. Links

- Ports: [Pearl Shallows](../ports/pearl_shallows.md), [Coral Throne](../ports/coral_throne.md)
- Landmarks: [The Blessing Pool](../landmarks/blessing_pool.md), [The Coral Palace](../landmarks/coral_palace.md)

## 7. Flags

- The Deep Reef Brotherhood lives at sea between Pearl Shallows and Ember Isle (README, invented); this lane is the Brotherhood's water too [invented]. No data flag.
