# The Hanging Market (hanging_market)

Base commit: 71c6aaa (content.json and voyage.rs unchanged since 45b7378). Status: draft canon, phase 2. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Category | A (port landmark) | - |
| Port | [Spice Narrows](../ports/spice_narrows.md) | - |
| Region | East Indies | C:1566 |
| Shipped text | "The Hanging Market - stalls suspended on ropes between cliff faces above the water." [ascii-folded] | C:12167 |
| Sea mark | none | voyage.rs:592-607 |
| Player sees it | yes, as the Landmark line on arrival | crates/portlight-sim/src/port_arrival_engine.rs:157-159 |
| Mechanic | none [flavour, no mechanic] | - |

## 1. Location (all [invented] unless a cite or a named data field is given)

Stalls suspended on ropes between cliff faces above the hidden anchorage (landmark). The Dark Room (C:10604), the Weigher's cave, opens off the cliff behind it.

No coordinates exist for landmarks; the chart position is the port's own map x/y. Placing landmarks on the chart is a future phase.

## 2. Data cites

- dock_scene: "The anchorage is hidden between volcanic cliffs. Rope ladders drop from the Hanging Market above. Commerce here is vertical - everything climbs." (C:12170)
- atmosphere: "Concentrated spice - overwhelming, intoxicating. The air itself burns your eyes." (C:12169)
- tavern_rumor: "The Narrows have a second market, deeper in the cliffs, where things other than spice change hands. Finding the entrance is the first test of membership." (C:12171)
- local_custom: "Prices are whispered, never spoken aloud. The walls have ears and the water carries sound." (C:12168)
- named in `dock_scene` at C:12170
- regional festival: Silk Weavers' Fair (C:11920)
- regional festival: Monsoon Prayer (C:11934)
- named lane: The Volcanic Passage (Spice Narrows - Ember Isle, C:2239) - lanes are deferred to a later phase
- named lane: The Smuggler's Run (Corsair's Rest - Spice Narrows, C:2293) - lanes are deferred to a later phase

## 3. History (all [invented] unless cited)

- T13: the Spice Lords take the Narrows and string the first stalls.
- T23: the market reaches its present span [invented].

## 4. Appearance and palette (art guidance only; commissions nothing; the character hold stands)

Plank platforms on rope webs between wet black cliffs, rope ladders to the water; spice red only on sacks. [invented]

Palette: drawn from the Spice Narrows port palette ([port file section C.3](../ports/spice_narrows.md)), so the landmark reads as part of its port. Values and statuses are the port file's; judged in sRGB against the envelope (luma 65-141, chroma p98 35; small-area accents up to chroma 64). No sea or water swatch.

| Role | Port swatch | sRGB | Luma / chroma | Status |
|---|---|---|---|---|
| Mass | wet cliff rock (Dominant) | (84, 82, 74) #54524A | 82 / 10 | in range |
| Detail | rope and plank (Secondary 1) | (102, 86, 68) #665644 | 88 / 34 | in range |
| Accent | spice red, small (Accent) | (134, 93, 71) #865D47 | 100 / 63 | small-area accent (chroma 63) |

Kit: compose from the shipped will-use plates; no named landmark has art today. Any new plate is a gap for the art gap list, not a commission.

## 5. People and use (all [invented] unless cited)

Spice factors, rope crews, the Weigher below. Prices are whispered (custom).

Extra depth (thin port): [invented] Every stall rope is owned by a different family; cutting a rope is the Narrows' only capital crime.

## 6. Sound hook (all [invented] sound design unless quoted from a cite)

- Rope creak, whispered prices carried by the water (custom C:12168).
- Fragments: none proposed yet [proposal]. No music, no voice; #45 ear hold unaffected.

## 7. Links to ports

- Owning port: [Spice Narrows](../ports/spice_narrows.md)
- [Corsair's Rest](../ports/corsairs_rest.md)
- [Ember Isle](../ports/ember_isle.md)
- [Dragon's Gate](../ports/dragons_gate.md)

## 8. Flags

- Near-duplicate: Hanging Market vs The Dark Room. Resolved: the market trades in the open above; the Dark Room is where the Weigher sets prices. The rumoured 'second market deeper in the cliffs' is neither, and stays a rumour.
