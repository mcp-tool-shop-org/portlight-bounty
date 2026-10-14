# The Great Foundry (great_foundry)

Base commit: 71c6aaa (content.json and voyage.rs unchanged since 45b7378). Status: draft canon, phase 2. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Category | A (port landmark) | - |
| Port | [Ironhaven](../ports/ironhaven.md) | - |
| Region | North Atlantic | C:559 |
| Shipped text | "The Great Foundry - its chimney visible twenty leagues at sea, glowing red at night." [ascii-folded] | C:12047 |
| Sea mark | "Ironhaven's Great Foundry chimney glows red on the horizon - a landmark for miles." | crates/portlight-sim/src/voyage.rs:595 |
| Player sees it | yes, as the Landmark line on arrival | crates/portlight-sim/src/port_arrival_engine.rs:157-159 |
| Mechanic | none [flavour, no mechanic] | - |
| Also an institution | `ih_foundry` | C:7898 |

## 1. Location (all [invented] unless a cite or a named data field is given)

Above the harbour; its chimney is visible twenty leagues at sea, glowing red at night (landmark).

No coordinates exist for landmarks; the chart position is the port's own map x/y. Placing landmarks on the chart is a future phase.

## 2. Data cites

- dock_scene: "Iron-hulled barges sit low in the water. Cranes swing overhead, loading ingots onto merchant vessels. Sparks drift from the foundry like orange snowflakes." (C:12050)
- atmosphere: "Coal smoke, ringing hammers, and the deep bass hum of bellows forcing air into furnaces." (C:12049)
- tavern_rumor: "The foundry master has been buying every scrap of timber that comes in. Some say he's building something in the deep sheds - something big." (C:12051)
- local_custom: "Greeting the harbor master with a nod, not words. The north wastes neither." (C:12048)
- named in `name` at C:7899
- named in `text` at C:12323
- voyage lighthouse line: crates/portlight-sim/src/voyage.rs:595
- regional festival: The Iron Days (C:11808)
- regional festival: Midwinter Market (C:11822)
- named lane: The Iron Strait (Ironhaven - Stormwall, C:1978) - lanes are deferred to a later phase
- named lane: The Northern Passage (Ironhaven - Jade Port, C:2302) - lanes are deferred to a later phase

## 3. History (all [invented] unless cited)

- T11: the Iron Pact is founded on the Foundry's output.
- The deep sheds where the foundry master builds 'something big' (rumour) are canon as unexplained [invented: no answer given].

## 4. Appearance and palette (art guidance only; commissions nothing; the character hold stands)

One tall brick chimney over long low sheds, cranes on the quay. By night only the chimney mouth glows. The glow is a small-area accent; the mass is soot-dark brick. [invented]

Palette: drawn from the Ironhaven port palette ([port file section C.3](../ports/ironhaven.md)), so the landmark reads as part of its port. Values and statuses are the port file's; judged in sRGB against the envelope (luma 65-141, chroma p98 35; small-area accents up to chroma 64). No sea or water swatch.

| Role | Port swatch | sRGB | Luma / chroma | Status |
|---|---|---|---|---|
| Mass | soot-stained brick, cool (Dominant) | (92, 82, 84) #5C5254 | 84 / 10 | in range |
| Detail | cold iron grey (Secondary 1) | (98, 100, 104) #626468 | 100 / 6 | in range |
| Accent | furnace glow (Accent) | (120, 82, 66) #785242 | 89 / 54 | small-area accent (chroma 54) |

Kit: compose from the shipped will-use plates; no named landmark has art today. Any new plate is a gap for the art gap list, not a commission.

## 5. People and use (all [invented] unless cited)

Smiths, furnace crews, crane gangs (dock scene). The Iron Days festival is held here (C:11808).

## 6. Sound hook (all [invented] sound design unless quoted from a cite)

- Ringing hammers and the deep bellows hum (atmosphere); crane chains.
- Fragment candidates: see the port's section I fragment ideas (../ports/ironhaven.md) [proposal; no fragment or asset exists yet]. No music, no voice; voice and music holds stand (README).

## 7. Links to ports

- Owning port: [Ironhaven](../ports/ironhaven.md)
- [Stormwall](../ports/stormwall.md)
- [Silva Bay](../ports/silva_bay.md)
- [Jade Port](../ports/jade_port.md)

## 8. Flags

- none
