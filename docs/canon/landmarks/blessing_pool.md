# The Blessing Pool (blessing_pool)

Base commit: 71c6aaa (content.json and voyage.rs unchanged since 45b7378). Status: draft canon, phase 2. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Category | A (port landmark) | - |
| Port | [Pearl Shallows](../ports/pearl_shallows.md) | - |
| Region | West Africa | C:1043 |
| Shipped text | "The Blessing Pool - a tidal pool where divers pray before descending." [ascii-folded] | C:12107 |
| Sea mark | none | voyage.rs:592-607 |
| Player sees it | yes, as the Landmark line on arrival | crates/portlight-sim/src/port_arrival_engine.rs:157-159 |
| Mechanic | none [flavour, no mechanic] | - |
| Also an institution | `ps_pool` | C:9246 |

## 1. Location (all [invented] unless a cite or a named data field is given)

A tidal pool by the shore where divers pray before descending (landmark); the Grading Stone (C:9236) and the Pool's Edge (C:9266) sit beside it.

No coordinates exist for landmarks; the chart position is the port's own map x/y. Placing landmarks on the chart is a future phase.

## 2. Data cites

- dock_scene: "Canoes glide in from the reef. Divers emerge glistening, pouches of pearls tied to their wrists. An elder sits under a baobab tree, examining each pearl by sunlight." (C:12110)
- atmosphere: "Warm shallow water, coral sand, and the silence of divers preparing their breath." (C:12109)
- tavern_rumor: "The oldest diver says there's a pearl the size of a fist somewhere in the deep reef. Three divers have gone looking. Two came back." (C:12111)
- local_custom: "Pearls are never haggled over. The diver names the price. You pay or you leave." (C:12108)
- named in `power_structure` at C:9222
- named in `description` at C:9240
- named in `name` at C:9247
- named in `description` at C:9270
- named in `description` at C:9294
- named in `greeting_friendly` at C:9318
- named in `description` at C:9378
- regional festival: Dyer's Moon (C:11864)
- regional festival: Pearl Tide (C:11877)
- named lane: The Deep South Run (Pearl Shallows - Coral Throne, C:2311) - lanes are deferred to a later phase

## 3. History (all [invented] unless cited)

- The pool has always been sacred; no founding is told [invented].
- T16: the poaching of the reef by Typhoon Anchorage divers is sworn against at the pool.

## 4. Appearance and palette (art guidance only; commissions nothing; the character hold stands)

A ring of pale rock around still water, a baobab beside it (dock scene), drying racks behind. [invented]

Palette: drawn from the Pearl Shallows port palette ([port file section C.3](../ports/pearl_shallows.md)), so the landmark reads as part of its port. Values and statuses are the port file's; judged in sRGB against the envelope (luma 65-141, chroma p98 35; small-area accents up to chroma 64). No sea or water swatch.

| Role | Port swatch | sRGB | Luma / chroma | Status |
|---|---|---|---|---|
| Mass | sand and shell (Dominant) | (146, 138, 120) #928A78 | 138 / 26 | in range |
| Detail | shallow water stone (Secondary 1) | (98, 112, 108) #62706C | 109 / 14 | in range |
| Accent | pearl sheen (Accent) | (144, 140, 132) #908C84 | 140 / 12 | in range |

Kit: compose from the shipped will-use plates; no named landmark has art today. Any new plate is a gap for the art gap list, not a commission.

## 5. People and use (all [invented] unless cited)

Elder Ama governs here (institution text); divers; the elder who examines each pearl.

## 6. Sound hook (all [invented] sound design unless quoted from a cite)

- Near silence; breath held; canoe paddles (atmosphere).
- Fragment candidates: see the port's section I fragment ideas (../ports/pearl_shallows.md) [proposal; no fragment or asset exists yet]. No music, no voice; voice and music holds stand (README).

## 7. Links to ports

- Owning port: [Pearl Shallows](../ports/pearl_shallows.md)
- [Coral Throne](../ports/coral_throne.md)
- [Typhoon Anchorage](../ports/typhoon_anchorage.md)

## 8. Flags

- none
