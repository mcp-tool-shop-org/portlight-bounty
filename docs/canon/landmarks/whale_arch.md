# The Whale Arch (whale_arch)

Base commit: 71c6aaa (content.json and voyage.rs unchanged since 45b7378). Status: draft canon, phase 2. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Category | A (port landmark) | - |
| Port | [Thornport](../ports/thornport.md) | - |
| Region | North Atlantic | C:727 |
| Shipped text | "The Whale Arch - a jawbone arch spanning the harbor entrance, bleached white by decades of salt." [ascii-folded] | C:12067 |
| Sea mark | "The Whale Arch of Thornport stands white against the grey sky." | crates/portlight-sim/src/voyage.rs:598 |
| Player sees it | yes, as the Landmark line on arrival | crates/portlight-sim/src/port_arrival_engine.rs:157-159 |
| Mechanic | none [flavour, no mechanic] | - |

## 1. Location (all [invented] unless a cite or a named data field is given)

A jawbone arch spanning the harbour entrance (landmark). The Whale Bone tavern is built under it (C:8358).

No coordinates exist for landmarks; the chart position is the port's own map x/y. Placing landmarks on the chart is a future phase.

## 2. Data cites

- dock_scene: "Whaling boats line the pier, their harpoons cleaned and ready. Tea merchants squat beside wooden crates, brewing samples for prospective buyers." (C:12070)
- atmosphere: "Wood smoke, drying fish, and the bitter-sweet scent of tobacco curing in the sheds." (C:12069)
- tavern_rumor: "The whales are swimming further north each year. The old captains say the sea is warming. The young ones say the old ones talk too much." (C:12071)
- local_custom: "Tea is offered before any negotiation. Tobacco is shared after a deal closes." (C:12068)
- named in `description` at C:8332
- named in `description` at C:8362
- named in `description` at C:8472
- named in `rumor` at C:8499
- named in `text` at C:12348
- voyage lighthouse line: crates/portlight-sim/src/voyage.rs:598
- regional festival: The Iron Days (C:11808)
- regional festival: Midwinter Market (C:11822)
- named lane: The Tea and Tobacco Road (Stormwall - Thornport, C:1996) - see [../lanes/tea_and_tobacco_road.md](../lanes/tea_and_tobacco_road.md)

## 3. History (all [invented] unless cited)

- T9: raised in the whaling era.
- T19: Bones begins carving the names of lost whalers on the Arch; the tavern copies each name into its ceiling beams [invented date].

## 4. Appearance and palette (art guidance only; commissions nothing; the character hold stands)

Two bleached jawbones meeting over the harbour mouth, names cut into the bone, white against grey sky (voyage.rs:598). [invented]

Palette: drawn from the Thornport port palette ([port file section C.3](../ports/thornport.md)), so the landmark reads as part of its port. Values and statuses are the port file's; judged in sRGB against the envelope (luma 65-141, chroma p98 35; small-area accents up to chroma 64). No sea or water swatch.

| Role | Port swatch | sRGB | Luma / chroma | Status |
|---|---|---|---|---|
| Mass | bleached bone (Accent) | (144, 140, 126) #908C7E | 140 / 18 | in range |
| Detail | fog-bank grey (Secondary 2) | (110, 116, 118) #6E7476 | 115 / 8 | in range |
| Accent | peat-smoked timber (Secondary 1) | (88, 78, 66) #584E42 | 79 / 22 | in range |

Kit: compose from the shipped will-use plates; no named landmark has art today. Any new plate is a gap for the art gap list, not a commission.

## 5. People and use (all [invented] unless cited)

Whalers, Bones the carver, mourners. Tea merchants brew samples on the pier beneath (dock scene).

## 6. Sound hook (all [invented] sound design unless quoted from a cite)

- Wind through the arch; a knife on bone; harpoon lines creaking at the pier.
- Fragment candidates: see the port's section I fragment ideas (../ports/thornport.md) [proposal; no fragment or asset exists yet]. No music, no voice; voice and music holds stand (README).

## 7. Links to ports

- Owning port: [Thornport](../ports/thornport.md)
- [Stormwall](../ports/stormwall.md)
- [Ironhaven](../ports/ironhaven.md)

## 8. Flags

- Where the 47 names are carved (C:8362, C:8499, C:12348). Resolved in lore: the Arch holds the originals, carved by Bones; the Whale Bone's beams hold copies, so the names can be read indoors in winter. Both data lines stand.
