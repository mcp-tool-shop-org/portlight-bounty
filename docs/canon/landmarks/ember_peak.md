# Ember Peak (ember_peak)

Base commit: 71c6aaa (content.json and voyage.rs unchanged since 45b7378). Status: draft canon, phase 2. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Category | A (port landmark) | - |
| Port | [Ember Isle](../ports/ember_isle.md) | - |
| Region | South Seas | C:1666 |
| Shipped text | "Ember Peak - the smoldering volcano visible from fifty leagues, its glow a navigator's beacon." [ascii-folded] | C:12177 |
| Sea mark | "Ember Peak's volcanic glow marks the horizon. The South Seas await." | crates/portlight-sim/src/voyage.rs:597 |
| Player sees it | yes, as the Landmark line on arrival | crates/portlight-sim/src/port_arrival_engine.rs:157-159 |
| Mechanic | none [flavour, no mechanic] | - |

## 1. Location (all [invented] unless a cite or a named data field is given)

The smouldering volcano at the island's heart, visible from fifty leagues (landmark).

No coordinates exist for landmarks; the chart position is the port's own map x/y. Placing landmarks on the chart is a future phase.

## 2. Data cites

- dock_scene: "Black sand beaches and obsidian-sharp rocks frame the harbor. Steam rises from hot springs near the dock. Herbalists sort medicinal plants in the warm volcanic soil." (C:12180)
- atmosphere: "Sulfur, tropical flowers, and the low rumble of a volcano that never fully sleeps." (C:12179)
- tavern_rumor: "The volcano speaks more often now. The islanders aren't worried - they say it always grumbles before a good harvest of medicine plants." (C:12181)
- local_custom: "Visitors place a stone at the volcano's base. It's said to appease the mountain." (C:12178)
- voyage lighthouse line: crates/portlight-sim/src/voyage.rs:597
- regional festival: Coral Coronation (C:11976)
- regional festival: Fire Walking (C:11990)
- named lane: The Volcanic Passage (Spice Narrows - Ember Isle, C:2239) - see [../lanes/volcanic_passage.md](../lanes/volcanic_passage.md)

## 3. History (all [invented] unless cited)

- T2: the Ember Keepers settle the volcano.
- T16: the peak grumbles more often.

## 4. Appearance and palette (art guidance only; commissions nothing; the character hold stands)

A dark cone with a faint glow at the crater, steam from hot springs at its foot. The glow is a small-area accent only. [invented]

Palette: drawn from the Ember Isle port palette ([port file section C.3](../ports/ember_isle.md)), so the landmark reads as part of its port. Values and statuses are the port file's; judged in sRGB against the envelope (luma 65-141, chroma p98 35; small-area accents up to chroma 64). No sea or water swatch.

| Role | Port swatch | sRGB | Luma / chroma | Status |
|---|---|---|---|---|
| Mass | obsidian beach, lifted (Dominant) | (72, 70, 72) #484648 | 71 / 2 | in range |
| Detail | volcanic soil (Secondary 1) | (98, 82, 72) #625248 | 85 / 26 | in range |
| Accent | ember glow, small (Accent) | (138, 100, 74) #8A644A | 106 / 64 | small-area accent (chroma 64) |

Kit: compose from the shipped will-use plates; no named landmark has art today. Any new plate is a gap for the art gap list, not a commission.

## 5. People and use (all [invented] unless cited)

Ember Keepers, herbalists harvesting at the hot springs; Fire Walking festival (C:11990).

Extra depth (thin port): [invented] The Keepers read the peak's colour each dusk; a red glow means a good medicine harvest, a white plume means the hot springs close.

## 6. Sound hook (all [invented] sound design unless quoted from a cite)

- Low volcanic rumble (atmosphere); steam vents.
- Fragment candidates: see the port's section I fragment ideas (../ports/ember_isle.md) [proposal; no fragment or asset exists yet]. No music, no voice; voice and music holds stand (README).

## 7. Links to ports

- Owning port: [Ember Isle](../ports/ember_isle.md)
- [Spice Narrows](../ports/spice_narrows.md)
- [Coral Throne](../ports/coral_throne.md)
- [Typhoon Anchorage](../ports/typhoon_anchorage.md)

## 8. Flags

- none
