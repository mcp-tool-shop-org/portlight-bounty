# The Porcelain Quarter (porcelain_quarter)

Base commit: 71c6aaa (content.json and voyage.rs unchanged since 45b7378). Status: draft canon, phase 2. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Category | A (port landmark) | - |
| Port | [Jade Port](../ports/jade_port.md) | - |
| Region | East Indies | C:1122 |
| Shipped text | "The Porcelain Quarter - a district of workshops where master potters fire kilns day and night." [ascii-folded] | C:12117 |
| Sea mark | none | voyage.rs:592-607 |
| Player sees it | yes, as the Landmark line on arrival | crates/portlight-sim/src/port_arrival_engine.rs:157-159 |
| Mechanic | none [flavour, no mechanic] | - |
| Also an institution | `jp_quarter` | C:9432 |

## 1. Location (all [invented] unless a cite or a named data field is given)

A district of kiln workshops; Chen's kiln is at its heart (institution text).

No coordinates exist for landmarks; the chart position is the port's own map x/y. Placing landmarks on the chart is a future phase.

## 2. Data cites

- dock_scene: "Bamboo cranes lift crates padded with straw from the hold. Each piece of porcelain is inspected by guild markers before it leaves the dock." (C:12120)
- atmosphere: "Kiln heat, wet clay, and the delicate tap-tap-tap of artisans painting glaze patterns." (C:12119)
- tavern_rumor: "A master potter smashed his entire year's work because he found a single flaw. The guild praised him. That kind of standard is why their porcelain costs what it does." (C:12121)
- local_custom: "Porcelain is presented on silk, never bare-handed. Fingerprints on porcelain void a sale." (C:12118)
- named in `name` at C:9433
- regional festival: Silk Weavers' Fair (C:11920)
- regional festival: Monsoon Prayer (C:11934)
- named lane: The Porcelain Lane (Crosswind Isle - Jade Port, C:2131) - see [../lanes/porcelain_lane.md](../lanes/porcelain_lane.md)
- named lane: The Silk Road by Sea (Jade Port - Silk Haven, C:2176) - see [../lanes/silk_road_by_sea.md](../lanes/silk_road_by_sea.md)
- named lane: The Northern Passage (Ironhaven - Jade Port, C:2302) - see [../lanes/northern_passage.md](../lanes/northern_passage.md)

## 3. History (all [invented] unless cited)

- The guild's flaw rule (rumour) is older than the Silk Circle [invented].

## 4. Appearance and palette (art guidance only; commissions nothing; the character hold stands)

Brick kiln domes and grey tiled roofs, heat shimmer, stacked straw-packed crates. [invented]

Palette: drawn from the Jade Port port palette ([port file section C.3](../ports/jade_port.md)), so the landmark reads as part of its port. Values and statuses are the port file's; judged in sRGB against the envelope (luma 65-141, chroma p98 35; small-area accents up to chroma 64). No sea or water swatch.

| Role | Port swatch | sRGB | Luma / chroma | Status |
|---|---|---|---|---|
| Mass | kiln-fired brick (Dominant) | (131, 111, 99) #836F63 | 114 / 32 | in range |
| Detail | grey roof tile (Secondary 2) | (94, 96, 98) #5E6062 | 96 / 4 | in range |
| Accent | white porcelain (Accent) | (142, 140, 134) #8E8C86 | 140 / 8 | in range |

Kit: compose from the shipped will-use plates; no named landmark has art today. Any new plate is a gap for the art gap list, not a commission.

## 5. People and use (all [invented] unless cited)

Master potters, guild markers inspecting every piece (dock scene).

## 6. Sound hook (all [invented] sound design unless quoted from a cite)

- Kiln roar under the tap of glaze brushes (atmosphere).
- Fragment candidates: see the port's section I fragment ideas (../ports/jade_port.md) [proposal; no fragment or asset exists yet]. No music, no voice; voice and music holds stand (README).

## 7. Links to ports

- Owning port: [Jade Port](../ports/jade_port.md)
- [Silk Haven](../ports/silk_haven.md)
- [Crosswind Isle](../ports/crosswind_isle.md)
- [Ironhaven](../ports/ironhaven.md)

## 8. Flags

- Jade Port's 'The Inspection Hall' (C:9462) shares its name with Stormwall's (C:8172): see DF-20.
