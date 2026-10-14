# The Free Port Bell (free_port_bell)

Base commit: 71c6aaa (content.json and voyage.rs unchanged since 45b7378). Status: draft canon, phase 2. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Category | A (port landmark) | - |
| Port | [Crosswind Isle](../ports/crosswind_isle.md) | - |
| Region | East Indies | C:1385 |
| Shipped text | "The Free Port Bell - rung once for every ship that enters, never for one that leaves." [ascii-folded] | C:12147 |
| Sea mark | none | voyage.rs:592-607 |
| Player sees it | yes, as the Landmark line on arrival | crates/portlight-sim/src/port_arrival_engine.rs:157-159 |
| Mechanic | none [flavour, no mechanic] | - |

## 1. Location (all [invented] unless a cite or a named data field is given)

At the harbour entrance; rung once for every ship that enters, never for one that leaves (landmark).

No coordinates exist for landmarks; the chart position is the port's own map x/y. Placing landmarks on the chart is a future phase.

## 2. Data cites

- dock_scene: "Ships from every region jostle for berth. The Free Port Bell tolls as you enter. Money-changers set up tables on the dock before your anchor hits bottom." (C:12150)
- atmosphere: "A dozen languages, cooking smoke from every cuisine, and the constant negotiation of trade." (C:12149)
- tavern_rumor: "Someone tried to claim the isle for an eastern dynasty last year. By morning, every captain in harbor had their guns trained on his ship. He left." (C:12151)
- local_custom: "All flags fly here. No nation claims the isle and all are welcome. That's the law." (C:12148)
- named in `dock_scene` at C:12150
- regional festival: Silk Weavers' Fair (C:11920)
- regional festival: Monsoon Prayer (C:11934)
- named lane: The Long Crossing (Sun Harbor - Crosswind Isle, C:2104) - lanes are deferred to a later phase
- named lane: The Porcelain Lane (Crosswind Isle - Jade Port, C:2131) - lanes are deferred to a later phase

## 3. History (all [invented] unless cited)

- T22: cast and hung at the free-port charter [invented].

## 4. Appearance and palette (art guidance only; commissions nothing; the character hold stands)

A bronze bell in an open timber frame on the harbour rocks. The only bronze on the isle. [invented]

Palette: drawn from the Crosswind Isle port palette ([port file section C.3](../ports/crosswind_isle.md)), so the landmark reads as part of its port. Values and statuses are the port file's; judged in sRGB against the envelope (luma 65-141, chroma p98 35; small-area accents up to chroma 64). No sea or water swatch.

| Role | Port swatch | sRGB | Luma / chroma | Status |
|---|---|---|---|---|
| Mass | weathered mixed timber (Dominant) | (133, 123, 111) #857B6F | 124 / 22 | in range |
| Detail | wind-scoured stone (Secondary 1) | (114, 112, 106) #72706A | 112 / 8 | in range |
| Accent | bell bronze (Accent) | (145, 120, 81) #917851 | 122 / 64 | small-area accent (chroma 64) |

Kit: compose from the shipped will-use plates; no named landmark has art today. Any new plate is a gap for the art gap list, not a commission.

## 5. People and use (all [invented] unless cited)

A bell-keeper paid by the Free Port Council [invented]; every arriving crew hears it.

## 6. Sound hook (all [invented] sound design unless quoted from a cite)

- One toll per arrival heard across the harbour (dock scene; port ambience only, never layered on or substituted for sfx_chart_dock_arrive; pitched, so never a cue identity, bell rule).
- Fragment candidates: see the port's section I fragment ideas (../ports/crosswind_isle.md) [proposal; no fragment or asset exists yet]. No music, no voice; voice and music holds stand (README).

## 7. Links to ports

- Owning port: [Crosswind Isle](../ports/crosswind_isle.md)
- [Sun Harbor](../ports/sun_harbor.md)
- [Jade Port](../ports/jade_port.md)
- [Dragon's Gate](../ports/dragons_gate.md)

## 8. Flags

- Near-duplicate: the Free Port Bell vs The Bell and Board (C:10156). Resolved: two bells. The Free Port Bell tolls for arrivals; the Bell and Board's own small bell rings for votes and last call.
