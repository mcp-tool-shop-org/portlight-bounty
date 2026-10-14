# The Master Shipyard (master_shipyard)

Base commit: 71c6aaa (content.json and voyage.rs unchanged since 45b7378). Status: draft canon, phase 2. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Category | A (port landmark) | - |
| Port | [Silva Bay](../ports/silva_bay.md) | - |
| Region | Mediterranean | C:381 |
| Shipped text | "The Master Shipyard - where the finest hulls in the Mediterranean take shape." [ascii-folded] | C:12027 |
| Sea mark | none | voyage.rs:592-607 |
| Player sees it | yes, as the Landmark line on arrival | crates/portlight-sim/src/port_arrival_engine.rs:157-159 |
| Mechanic | none [flavour, no mechanic] | - |
| Also an institution | `sb_shipyard` | C:7420 |

## 1. Location (all [invented] unless a cite or a named data field is given)

The waterfront of Silva Bay: three slipways and two dry docks at the head of the bay (institution text). The yard is the town (phase 1 identity line).

No coordinates exist for landmarks; the chart position is the port's own map x/y. Placing landmarks on the chart is a future phase.

## 2. Data cites

- dock_scene: "Sawdust drifts like snow across the waterfront. Half-finished hulls rise on slipways, their ribs exposed like the skeletons of great fish." (C:12030)
- atmosphere: "Fresh-cut timber, hot pitch, and the rhythmic sound of adzes shaping keels." (C:12029)
- tavern_rumor: "The master shipwright refuses to build for anyone who has lost a ship to storms. She says it's bad wood-luck." (C:12031)
- local_custom: "Touching another captain's ship without permission is grounds for a duel." (C:12028)
- named in `name` at C:7421
- regional festival: Harvest of Plenty (C:11753)
- regional festival: Night of Lanterns (C:11766)
- named lane: The Timber Run (Porto Novo - Silva Bay, C:1942) - see [../lanes/timber_run.md](../lanes/timber_run.md)

## 3. History (all [invented] unless cited)

- T6: the Brotherhood begins carving ship names in its hall, about two hundred years ago.
- T15: the yard refuses the Iron Pact war-fleet commission; it builds single hulls of every class, never fleets for a power (phase 1 ruling).

## 4. Appearance and palette (art guidance only; commissions nothing; the character hold stands)

Serrated shed ridges, slipways with half-built ribs 'like the skeletons of great fish' (dock scene), steam box chimney, sawdust underfoot. The tallest shape is a hull on the slip. [invented]

Palette: drawn from the Silva Bay port palette ([port file section C.3](../ports/silva_bay.md)), so the landmark reads as part of its port. Values and statuses are the port file's; judged in sRGB against the envelope (luma 65-141, chroma p98 35; small-area accents up to chroma 64). No sea or water swatch.

| Role | Port swatch | sRGB | Luma / chroma | Status |
|---|---|---|---|---|
| Mass | weathered tarred timber (Dominant) | (103, 86, 68) #675644 | 88 / 35 | in range |
| Detail | fieldstone and wet quay grey (Secondary 2) | (118, 116, 108) #76746C | 116 / 10 | in range |
| Accent | fresh-cut oak and sawdust (Accent) | (163, 137, 99) #A38963 | 140 / 64 | small-area accent (chroma 64) |

Kit: compose from the shipped will-use plates; no named landmark has art today. Any new plate is a gap for the art gap list, not a commission.

## 5. People and use (all [invented] unless cited)

Elena and the Brotherhood shipwrights, sawyers, caulkers. Every hull class the sim sells is built here, including the Royal Man-of-War (no art depicts it; art hold).

## 6. Sound hook (all [invented] sound design unless quoted from a cite)

- Adze bite into green timber; mallet on treenail (ambience only, never a cue); steam box hiss; pitch pot simmer.
- Fragment candidates: see the port's section I fragment ideas (../ports/silva_bay.md) [proposal; no fragment or asset exists yet]. No music, no voice; voice and music holds stand (README).

## 7. Links to ports

- Owning port: [Silva Bay](../ports/silva_bay.md)
- [Porto Novo](../ports/porto_novo.md)
- [Ironhaven](../ports/ironhaven.md)

## 8. Flags

- The NPC line naming a 'Timber Exchange' (C:7542) is the Timber Yard: DF-8 (phase 1).
