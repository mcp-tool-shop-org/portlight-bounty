# The Long Crossing (long_crossing)

Base commit: ab03ace (content.json, sea_culture.rs, voyage.rs and view.rs as cited). Status: draft canon, phase 3. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Name | The Long Crossing | C:2104 |
| Ends | [Sun Harbor](../ports/sun_harbor.md) - [Crosswind Isle](../ports/crosswind_isle.md) (undirected) | C:2099-2100 |
| Regions | West Africa - East Indies | port records |
| Distance | 64 | C:2101 |
| Danger | 0.18 | C:2102 |
| Minimum class | galleon | C:2103 |
| Days at minimum-class speed | 16 (distance 64 / speed 4, Python rounding) | voyage.rs:726-728; C:182 |
| Chart label | "Crosswind Isle (The Long Crossing)  64  danger 0.18  galleon  N days" when docked at Sun Harbor, and the mirror from Crosswind Isle | crates/portlight-chart/src/view.rs:213-223, 327-334 |
| Godot | the label is drawn as the chart hover text | crates/portlight-godot/src/chart_canvas.rs:228, 444 |
| Encounter table | 7 entries keyed by the name | C:12395; sea_culture.rs:121-136 |
| Mechanic | the name selects the route encounter table (25% roll per voyage, sea_culture.rs:125); otherwise flavour | sea_culture.rs:30-36 |

## 1. Data cites

- lore: "Weeks of open water. The route that separates traders from merchants. Many set out. Fewer arrive." [ascii-folded] (C:2105)
- encounter (weather): "Open water. No land in any direction. The sky meets the sea in a circle that hasn't changed in three days. The crew goes quiet." [ascii-folded] (C:12398)
- encounter (crew): "Day twelve of the crossing. Your navigator takes star readings twice a night. The slightest error here means weeks of drift." [ascii-folded] (C:12403)
- encounter (lore): "A piece of driftwood with carved symbols floats past - a prayer marker from an island nobody's charted. Someone was here before you." [ascii-folded] (C:12408)
- encounter (crew): "The crew's conversation turns to home. Where they came from. Where they'll go when they're done. The Long Crossing forces these thoughts." [ascii-folded] (C:12413)
- encounter (encounter): "Your water casks are half-empty. The navigator says two more days. The cook starts rationing without being asked. Everyone knows what's at stake." [ascii-folded] (C:12418)
- encounter (weather): "A flying fish lands on deck. The crew takes it as a sign - land is close. Your navigator confirms: the current has shifted. The East Indies are near." [ascii-folded] (C:12423)
- encounter (sighting): "You spot another ship on the same heading - impossibly far away, a speck of sail. Someone else is making the crossing. You'll never know who." [ascii-folded] (C:12428)

## 2. Course and waters (all [invented] unless cited)

Open ocean from the Gold Coast to Crosswind Isle, with no land in any direction (C:12398). Navigators take star readings twice a night (C:12403); the current shift and flying fish tell them the East Indies are near (C:12423).

No lane has coordinates or a drawn path; the chart joins the two port map x/y points with a straight segment (view.rs:200-210). Any drawn course is a future phase.

## 3. History (all [invented] unless cited)

- T26 (~100 BS): the first recorded crossing; driftwood prayer markers from uncharted islands show others were there first (C:12408).
- T12 (~50 BS): Crosswind Isle's charter as a free port makes the crossing worth the risk.

## 4. People and traffic (all [invented] unless cited)

Galleon crews only (min class galleon); rationing cooks, star-reading navigators, far sails never identified (C:12428).

## 5. Sound hook (all [invented] sound design unless quoted from a cite)

- Long ocean swell under the keel; water casks sloshing in the hold; a lone gust through the shrouds (atmosphere). No sand or hourglass sound (next_day lock).
- Lanes are heard at sea only through the voyage text. Ambience only, never a cue; never layered on sfx_chart_sail_depart or any arrival cue. No music, no voice; the holds stand (README). Fragment candidates: none proposed yet [proposal; no fragment or asset exists yet].

## 6. Links

- Ports: [Sun Harbor](../ports/sun_harbor.md), [Crosswind Isle](../ports/crosswind_isle.md)
- Landmarks: [The Cotton Steps](../landmarks/cotton_steps.md), [The Free Port Bell](../landmarks/free_port_bell.md)

## 7. Flags

- Resolved by DF-24: lore now says 'Weeks of open water' (C:2105); 16 days at galleon speed fits.
