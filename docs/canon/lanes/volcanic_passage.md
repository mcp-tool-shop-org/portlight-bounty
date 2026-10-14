# The Volcanic Passage (volcanic_passage)

Base commit: ab03ace (content.json, sea_culture.rs, voyage.rs and view.rs as cited). Status: draft canon, phase 3. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Name | The Volcanic Passage | C:2239 |
| Ends | [Spice Narrows](../ports/spice_narrows.md) - [Ember Isle](../ports/ember_isle.md) (undirected) | C:2234-2235 |
| Regions | East Indies - South Seas | port records |
| Distance | 48 | C:2236 |
| Danger | 0.18 | C:2237 |
| Minimum class | galleon | C:2238 |
| Days at minimum-class speed | 12 (distance 48 / speed 4, Python rounding) | voyage.rs:726-728; C:182 |
| Chart label | "Ember Isle (The Volcanic Passage)  48  danger 0.18  galleon  N days" when docked at Spice Narrows, and the mirror from Ember Isle | crates/portlight-chart/src/view.rs:213-223, 327-334 |
| Godot | the label is drawn as the chart hover text | crates/portlight-godot/src/chart_canvas.rs:228, 444 |
| Encounter table | 5 entries keyed by the name | C:12520; sea_culture.rs:121-136 |
| Mechanic | the name selects the route encounter table (25% roll per voyage, sea_culture.rs:125); otherwise flavour | sea_culture.rs:30-36 |

## 1. Data cites

- lore: "Warm currents and sulfur air. The locals say the sea boils near the islands on bad days." [ascii-folded] (C:2240)
- encounter (weather): "The water warms. Not from the sun - from below. Volcanic vents heat the current here. The hull temperature rises. Your crew shifts nervously." [ascii-folded] (C:12523)
- encounter (weather): "Sulfur. The air thickens with it. Your eyes water. Welcome to the Volcanic Passage - the sea itself is on fire underneath." [ascii-folded] (C:12528)
- encounter (encounter): "A column of steam rises from the water ahead - an underwater vent. Your navigator routes around it. The crew watches the thermometer climb." [ascii-folded] (C:12533)
- encounter (sighting): "An Ember Isle medicine boat crosses your path, obsidian-hulled and silent. The herbalists wave. Their cargo smells of eucalyptus and volcanic earth." [ascii-folded] (C:12538)
- encounter (crew): "Your cook heats water for tea using the sea itself - he drops a bucket into the volcanic current and it comes up warm. 'Free fuel,' he says." [ascii-folded] (C:12543)

## 2. Course and waters (all [invented] unless cited)

Warm currents and sulfur air between the Narrows and Ember Isle; vents heat the water and throw up steam columns (C:12523, C:12533).

No lane has coordinates or a drawn path; the chart joins the two port map x/y points with a straight segment (view.rs:200-210). Any drawn course is a future phase.

## 3. History (all [invented] unless cited)

- T2 (~700 BS): the Ember Keepers settle the volcano; their medicine boats use this water.
- T16 (~1 BS): Ember Peak grumbles more often; the steam columns are more frequent [invented link].

## 4. People and traffic (all [invented] unless cited)

Obsidian-hulled Ember Isle medicine boats (C:12538); cooks heating tea water from the sea (C:12543).

## 5. Sound hook (all [invented] sound design unless quoted from a cite)

- Vent hiss, bubbling water, a far geological rumble (atmosphere; Ember Isle is the only geological bed, phase 1).
- Lanes are heard at sea only through the voyage text. Ambience only, never a cue; never layered on sfx_chart_sail_depart or any arrival cue. No music, no voice; the holds stand (README). Fragment candidates: none proposed [proposal].

## 6. Links

- Ports: [Spice Narrows](../ports/spice_narrows.md), [Ember Isle](../ports/ember_isle.md)
- Landmarks: [Ember Peak](../landmarks/ember_peak.md), [The Hanging Market](../landmarks/hanging_market.md)

## 7. Flags

- 'obsidian-hulled' (C:12538) is a physical impossibility for a hull; read as obsidian-inlaid or black-lacquered [invented]. Resolved in lore.
