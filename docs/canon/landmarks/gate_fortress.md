# The Gate Fortress (gate_fortress)

Base commit: 71c6aaa (content.json and voyage.rs unchanged since 45b7378). Status: draft canon, phase 2. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Category | A (port landmark) | - |
| Port | [Dragon's Gate](../ports/dragons_gate.md) | - |
| Region | East Indies | C:1487 |
| Shipped text | "The Gate Fortress - twin stone towers flanking the strait, chains ready to close the passage." [ascii-folded] | C:12157 |
| Sea mark | none | voyage.rs:592-607 |
| Player sees it | yes, as the Landmark line on arrival | crates/portlight-sim/src/port_arrival_engine.rs:157-159 |
| Mechanic | none [flavour, no mechanic] | - |

## 1. Location (all [invented] unless a cite or a named data field is given)

Twin stone towers flanking the strait, chains between (landmark). The Chain Harbor (C:10360) is the water between the towers; The Gate Command (C:10380) is the fortress's highest tower.

No coordinates exist for landmarks; the chart position is the port's own map x/y. Placing landmarks on the chart is a future phase.

## 2. Data cites

- dock_scene: "Soldiers in lacquered armor inspect every ship that passes the twin towers. Tea merchants wait patiently on the inner docks, knowing inspection takes time." (C:12160)
- atmosphere: "Military precision, jasmine tea, and the distant clank of the harbor chains." (C:12159)
- tavern_rumor: "The fortress commander has not lowered the chains in fifteen years. The last captain who tried to run them is still chained to the seabed as a warning." (C:12161)
- local_custom: "Weapons in cargo are declared or confiscated. There is no third option." (C:12158)
- regional festival: Silk Weavers' Fair (C:11920)
- regional festival: Monsoon Prayer (C:11934)

## 3. History (all [invented] unless cited)

- T7: the towers and chain are raised.
- T13: the Iron Wolves try to run weapons through; fifteen are still chained to the seabed (grudge).
- T21: the chains are last lowered fifteen years before Day 1 (rumour) [invented date].

## 4. Appearance and palette (art guidance only; commissions nothing; the character hold stands)

Two square dark towers, a sagging chain between, lacquer-red banners as the only colour. [invented]

Palette: drawn from the Dragon's Gate port palette ([port file section C.3](../ports/dragons_gate.md)), so the landmark reads as part of its port. Values and statuses are the port file's; judged in sRGB against the envelope (luma 65-141, chroma p98 35; small-area accents up to chroma 64). No sea or water swatch.

| Role | Port swatch | sRGB | Luma / chroma | Status |
|---|---|---|---|---|
| Mass | dark fortress stone (Dominant) | (106, 104, 102) #6A6866 | 104 / 4 | in range |
| Detail | chain iron (Secondary 1) | (84, 86, 88) #545658 | 86 / 4 | in range |
| Accent | lacquer red, small (Accent) | (132, 84, 72) #845448 | 93 / 60 | small-area accent (chroma 60) |

Kit: compose from the shipped will-use plates; no named landmark has art today. Any new plate is a gap for the art gap list, not a commission.

## 5. People and use (all [invented] unless cited)

Gate Wardens, inspectors, the commander (Zhang Wei).

Extra depth (thin port): [invented] Closing the chain takes sixty seconds (C:10360 area) and forty winch hands; the drill is run every new moon so the town hears it.

## 6. Sound hook (all [invented] sound design unless quoted from a cite)

- Chain clank; the gate gong [invented].
- Fragments: none proposed yet [proposal]. No music, no voice; #45 ear hold unaffected.

## 7. Links to ports

- Owning port: [Dragon's Gate](../ports/dragons_gate.md)
- [Jade Port](../ports/jade_port.md)
- [Silk Haven](../ports/silk_haven.md)
- [Crosswind Isle](../ports/crosswind_isle.md)
- [Spice Narrows](../ports/spice_narrows.md)

## 8. Flags

- Near-duplicate: Gate Fortress vs Chain Harbor vs Gate Command. Resolved: one fortress; harbour and command tower are parts of it.
- Chain Harbor (Dragon's Gate) vs Chained Harbor (Typhoon Anchorage, C:11024): different. The Gate's chains bar a strait; the Anchorage's chains moor ships.
