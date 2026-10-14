# The Cotton Steps (cotton_steps)

Base commit: 71c6aaa (content.json and voyage.rs unchanged since 45b7378). Status: draft canon, phase 2. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Category | A (port landmark) | - |
| Port | [Sun Harbor](../ports/sun_harbor.md) | - |
| Region | West Africa | C:806 |
| Shipped text | "The Cotton Steps - wide stone stairs where cotton bales are displayed and auctioned." [ascii-folded] | C:12077 |
| Sea mark | none | voyage.rs:592-607 |
| Player sees it | yes, as the Landmark line on arrival | crates/portlight-sim/src/port_arrival_engine.rs:157-159 |
| Mechanic | none [flavour, no mechanic] | - |
| Also an institution | `sh_steps` | C:8572 |

## 1. Location (all [invented] unless a cite or a named data field is given)

Wide stone stairs from the waterfront where bales are displayed and auctioned (landmark).

No coordinates exist for landmarks; the chart position is the port's own map x/y. Placing landmarks on the chart is a future phase.

## 2. Data cites

- dock_scene: "Cotton bales stack three stories high along the waterfront. Women in indigo cloth oversee the weighing, their voices carrying the count in song." (C:12080)
- atmosphere: "Warm earth, indigo dye, and the rhythmic singing of dockworkers loading bales." (C:12079)
- tavern_rumor: "A ship arrived last month carrying nothing but empty crates - paid in full. The harbor master looked the other way. Nobody asks about empty crates here." (C:12081)
- local_custom: "The first trade of the day is blessed by an elder. Smart captains arrive early." (C:12078)
- named in `power_structure` at C:8558
- named in `name` at C:8573
- named in `description` at C:8706
- named in `agenda` at C:8707
- regional festival: Dyer's Moon (C:11864)
- regional festival: Pearl Tide (C:11877)
- named lane: The Cotton Crossing (Porto Novo - Sun Harbor, C:2032) - see [../lanes/cotton_crossing.md](../lanes/cotton_crossing.md)
- named lane: The Long Crossing (Sun Harbor - Crosswind Isle, C:2104) - see [../lanes/long_crossing.md](../lanes/long_crossing.md)

## 3. History (all [invented] unless cited)

- T10: the Gold Coast Compact is sworn on the steps [invented].
- Dyer's Moon stalls fill the lower steps (festival C:11864).

## 4. Appearance and palette (art guidance only; commissions nothing; the character hold stands)

Broad pale stone stairs stacked with cotton bales three storeys high (dock scene); indigo cloth of the weighers as the only strong colour. [invented]

Palette: drawn from the Sun Harbor port palette ([port file section C.3](../ports/sun_harbor.md)), so the landmark reads as part of its port. Values and statuses are the port file's; judged in sRGB against the envelope (luma 65-141, chroma p98 35; small-area accents up to chroma 64). No sea or water swatch.

| Role | Port swatch | sRGB | Luma / chroma | Status |
|---|---|---|---|---|
| Mass | sunlit stone steps (Dominant) | (137, 124, 102) #897C66 | 125 / 35 | in range |
| Detail | cotton bale (Secondary 1) | (134, 128, 116) #868074 | 128 / 18 | in range |
| Accent | indigo cloth, muted (Secondary 2) | (80, 86, 104) #505668 | 86 / 24 | in range |

Kit: compose from the shipped will-use plates; no named landmark has art today. Any new plate is a gap for the art gap list, not a commission.

## 5. People and use (all [invented] unless cited)

Women in indigo who oversee the weighing and sing the count (dock scene); bidders; porters.

## 6. Sound hook (all [invented] sound design unless quoted from a cite)

- Bale thumps on stone steps; scale beams creaking (atmosphere).
- Music NOTE ONLY: Kofi's counting song, "rises into song when he's counting bales" (C:8706); music stays on hold.
- Fragment candidates: see the port's section I fragment ideas (../ports/sun_harbor.md) [proposal; no fragment or asset exists yet]. No music, no voice; voice and music holds stand (README).

## 7. Links to ports

- Owning port: [Sun Harbor](../ports/sun_harbor.md)
- [Porto Novo](../ports/porto_novo.md)
- [Palm Cove](../ports/palm_cove.md)
- [Crosswind Isle](../ports/crosswind_isle.md)

## 8. Flags

- none
