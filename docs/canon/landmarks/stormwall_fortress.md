# The Fortress (stormwall_fortress)

Base commit: 71c6aaa (content.json and voyage.rs unchanged since 45b7378). Status: draft canon, phase 2. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Category | A (port landmark) | - |
| Port | [Stormwall](../ports/stormwall.md) | - |
| Region | North Atlantic | C:648 |
| Shipped text | "The Fortress - grey stone walls rising from the sea, older than anyone remembers." [ascii-folded] | C:12057 |
| Sea mark | none | voyage.rs:592-607 |
| Player sees it | yes, as the Landmark line on arrival | crates/portlight-sim/src/port_arrival_engine.rs:157-159 |
| Mechanic | none [flavour, no mechanic] | - |

## 1. Location (all [invented] unless a cite or a named data field is given)

The fortress is the town: grey walls rising from the sea; the harbour quays are built into its walls (The Fortress Gate, C:8122).

No coordinates exist for landmarks; the chart position is the port's own map x/y. Placing landmarks on the chart is a future phase.

## 2. Data cites

- dock_scene: "Navy vessels line the inner harbor. Soldiers in grey coats patrol the docks, checking manifests against sealed orders. Everything is orderly. Everything is watched." (C:12060)
- atmosphere: "Salt wind, military drums from the garrison, and the creak of heavy gates." (C:12059)
- tavern_rumor: "The garrison commander has doubled the night watch. Nobody says why, but the soldiers look north when they think no one's watching." (C:12061)
- local_custom: "All cargo is inspected. Complaining about it marks you as a southerner." (C:12058)
- named in `power_structure` at C:8118
- named in `name` at C:8123
- named in `description` at C:8126
- named in `political_leaning` at C:8128
- named in `description` at C:8146
- named in `description` at C:8156
- regional festival: The Iron Days (C:11808)
- regional festival: Midwinter Market (C:11822)
- named lane: The Iron Strait (Ironhaven - Stormwall, C:1978) - see [../lanes/iron_strait.md](../lanes/iron_strait.md)
- named lane: The Tea and Tobacco Road (Stormwall - Thornport, C:1996) - see [../lanes/tea_and_tobacco_road.md](../lanes/tea_and_tobacco_road.md)

## 3. History (all [invented] unless cited)

- T3: raised by the old Northern Crown.
- T8: the Crown dissolves; the garrison holds the straits alone.

## 4. Appearance and palette (art guidance only; commissions nothing; the character hold stands)

Sheer grey granite walls from the waterline, one sea gate, a watchtower. No colour except lamp-lit windows at night. [invented]

Palette: drawn from the Stormwall port palette ([port file section C.3](../ports/stormwall.md)), so the landmark reads as part of its port. Values and statuses are the port file's; judged in sRGB against the envelope (luma 65-141, chroma p98 35; small-area accents up to chroma 64). No sea or water swatch.

| Role | Port swatch | sRGB | Luma / chroma | Status |
|---|---|---|---|---|
| Mass | weathered fortress granite (Dominant) | (112, 112, 110) #70706E | 112 / 2 | in range |
| Detail | sea-wet stone (Secondary 1) | (86, 90, 92) #565A5C | 89 / 6 | in range |
| Accent | lamp-lit window (Accent) | (140, 120, 86) #8C7856 | 122 / 54 | small-area accent (chroma 54) |

Kit: compose from the shipped will-use plates; no named landmark has art today. Any new plate is a gap for the art gap list, not a commission.

## 5. People and use (all [invented] unless cited)

Garrison in grey coats, inspectors (The Inspection Hall, C:8172), the commander. Night watch doubled (rumour).

## 6. Sound hook (all [invented] sound design unless quoted from a cite)

- Heavy gate hinges, salt wind, patrol boots (atmosphere); garrison drums distant only, music NOTE ONLY.
- Fragment candidates: see the port's section I fragment ideas (../ports/stormwall.md) [proposal; no fragment or asset exists yet]. No music, no voice; voice and music holds stand (README).

## 7. Links to ports

- Owning port: [Stormwall](../ports/stormwall.md)
- [Ironhaven](../ports/ironhaven.md)
- [Thornport](../ports/thornport.md)

## 8. Flags

- Near-duplicate: 'The Fortress' (landmark) and 'The Fortress Gate' (harbour). Resolved in lore: the Gate is the sea gate and quays inside the Fortress walls; one structure.
- Not the Storm Wall of Typhoon Anchorage: different places, different seas.
