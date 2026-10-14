# The Loom Quarter (loom_quarter)

Base commit: 71c6aaa (content.json and voyage.rs unchanged since 45b7378). Status: draft canon, phase 2. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Category | A (port landmark) | - |
| Port | [Silk Haven](../ports/silk_haven.md) | - |
| Region | East Indies | C:1298 |
| Shipped text | "The Loom Quarter - where silk is woven in patterns a thousand years old." [ascii-folded] | C:12137 |
| Sea mark | none | voyage.rs:592-607 |
| Player sees it | yes, as the Landmark line on arrival | crates/portlight-sim/src/port_arrival_engine.rs:157-159 |
| Mechanic | none [flavour, no mechanic] | - |
| Also an institution | `slh_loom` | C:9902 |

## 1. Location (all [invented] unless a cite or a named data field is given)

A district where a hundred looms never stop (institution text); berths in The Arranged Harbor (C:9892) are set for its view.

No coordinates exist for landmarks; the chart position is the port's own map x/y. Placing landmarks on the chart is a future phase.

## 2. Data cites

- dock_scene: "Sampans glide between junks with crimson sails. Merchants in silk robes examine bolts of fabric with magnifying glasses, judging thread count by touch." (C:12140)
- atmosphere: "The rhythmic clack of a hundred looms, the rustle of silk, and green tea steam." (C:12139)
- tavern_rumor: "The eldest weaver has created a pattern that changes color in different light. Three merchants have bid their entire fortunes for a single bolt." (C:12141)
- local_custom: "Silk is presented folded, never rolled. Rolling insults the weaver's art." (C:12138)
- named in `description` at C:9896
- named in `name` at C:9903
- named in `description` at C:9970
- named in `rumor` at C:9975
- named in `agenda` at C:9993
- named in `port_grudge` at C:13630
- regional festival: Silk Weavers' Fair (C:11920)
- regional festival: Monsoon Prayer (C:11934)
- named lane: The Silk Road by Sea (Jade Port - Silk Haven, C:2176) - lanes are deferred to a later phase

## 3. History (all [invented] unless cited)

- T1: the patterns are first woven, about a thousand years ago.
- The Silk Weavers' Fair hangs bolts along its waterfront (C:11920).

## 4. Appearance and palette (art guidance only; commissions nothing; the character hold stands)

Dark lacquered loom halls with open fronts, raw silk on bamboo frames; one thread of dyed colour per street. [invented]

Palette: drawn from the Silk Haven port palette ([port file section C.3](../ports/silk_haven.md)), so the landmark reads as part of its port. Values and statuses are the port file's; judged in sRGB against the envelope (luma 65-141, chroma p98 35; small-area accents up to chroma 64). No sea or water swatch.

| Role | Port swatch | sRGB | Luma / chroma | Status |
|---|---|---|---|---|
| Mass | dark lacquered timber (Dominant) | (85, 69, 61) #55453D | 72 / 24 | in range |
| Detail | raw silk, muted (Secondary 1) | (136, 128, 112) #888070 | 129 / 24 | in range |
| Accent | dyed silk thread, small (Accent) | (138, 96, 90) #8A605A | 104 / 48 | small-area accent (chroma 48) |

Kit: compose from the shipped will-use plates; no named landmark has art today. Any new plate is a gap for the art gap list, not a commission.

## 5. People and use (all [invented] unless cited)

Weavers, the eldest weaver (rumour), merchants judging thread count by touch.

Extra depth (thin port): [invented] Each street of the Quarter weaves one pattern family; apprentices spend seven years on one street before they may cross to the next. The Quarter closes for one day a year, when every loom is restrung.

## 6. Sound hook (all [invented] sound design unless quoted from a cite)

- A hundred looms clacking (atmosphere); silk rustle.
- Fragment candidates: see the port's section I fragment ideas (../ports/silk_haven.md) [proposal; no fragment or asset exists yet]. No music, no voice; voice and music holds stand (README).

## 7. Links to ports

- Owning port: [Silk Haven](../ports/silk_haven.md)
- [Jade Port](../ports/jade_port.md)
- [Dragon's Gate](../ports/dragons_gate.md)

## 8. Flags

- none
