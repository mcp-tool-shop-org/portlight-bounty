# The Timber Run (timber_run)

Base commit: ab03ace (content.json, sea_culture.rs, voyage.rs and view.rs as cited). Status: draft canon, phase 3. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Name | The Timber Run | C:1942 |
| Ends | [Porto Novo](../ports/porto_novo.md) - [Silva Bay](../ports/silva_bay.md) (undirected) | C:1937-1938 |
| Regions | Mediterranean - Mediterranean | port records |
| Distance | 16 | C:1939 |
| Danger | 0.08 | C:1940 |
| Minimum class | sloop | C:1941 |
| Days at minimum-class speed | 2 (distance 16 / speed 8, Python rounding) | voyage.rs:726-728; C:137 |
| Chart label | "Silva Bay (The Timber Run)  16  danger 0.08  sloop  N days" when docked at Porto Novo, and the mirror from Silva Bay | crates/portlight-chart/src/view.rs:213-223, 327-334 |
| Godot | the label is drawn as the chart hover text | crates/portlight-godot/src/chart_canvas.rs:228, 444 |
| Encounter table | 4 entries keyed by the name | C:12260; sea_culture.rs:121-136 |
| Mechanic | the name selects the route encounter table (25% roll per voyage, sea_culture.rs:125); otherwise flavour | sea_culture.rs:30-36 |

## 1. Data cites

- lore: "Short and safe - the shipwrights' lifeline. Silva Bay's timber built Porto Novo's fleet." [ascii-folded] (C:1943)
- encounter (sighting): "A timber barge from Silva Bay passes, logs lashed together and riding the current. The smell of fresh-cut oak carries across the water." [ascii-folded] (C:12263)
- encounter (weather): "Sawdust floats on the surface - you're near Silva Bay. The entire harbor sheds wood dust like a forest sheds leaves." [ascii-folded] (C:12268)
- encounter (crew): "Your ship's carpenter examines the hull planking and nods. 'Silva Bay timber. Best in the Mediterranean. This wood will outlast all of us.'" [ascii-folded] (C:12273)
- encounter (sighting): "A fishing boat crosses your path. The fisherman shouts: 'Elena's building something big! Three masts! I saw the keel!'" [ascii-folded] (C:12278)

## 2. Course and waters (all [invented] unless cited)

The shortest named lane: a sheltered coastal hop from Porto Novo to the forested bay. Sawdust on the surface tells a crew they are close (C:12268).

No lane has coordinates or a drawn path; the chart joins the two port map x/y points with a straight segment (view.rs:200-210). Any drawn course is a future phase.

## 3. History (all [invented] unless cited)

- T6 (~200 BS): with Silva Bay's shipyard tradition, the run becomes the yard's lifeline; 'Silva Bay's timber built Porto Novo's fleet' (C:1943).
- Elena's new three-master keel (C:12278) is current gossip, not history.

## 4. People and traffic (all [invented] unless cited)

Timber barges with logs lashed together (C:12263), fishing boats, the yard's own carpenters going to and fro.

## 5. Sound hook (all [invented] sound design unless quoted from a cite)

- Log rafts knocking softly, water through lashed timber, gulls (atmosphere; ambience only, never a cue).
- Lanes are heard at sea only through the voyage text. Ambience only, never a cue; never layered on sfx_chart_sail_depart or any arrival cue. No music, no voice; the holds stand (README). Fragment candidates: none proposed yet [proposal; no fragment or asset exists yet].

## 6. Links

- Ports: [Porto Novo](../ports/porto_novo.md), [Silva Bay](../ports/silva_bay.md)
- Landmarks: [The Master Shipyard](../landmarks/master_shipyard.md), [The Grain Exchange](../landmarks/grain_exchange.md)

## 7. Flags

- 'Short and safe' (C:1943) holds: 16 distance, 0.08 danger; one other lane is safer (DF-23 context only, no contradiction here).
