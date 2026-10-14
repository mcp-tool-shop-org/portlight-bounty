# The Distillery Row (distillery_row)

Base commit: 71c6aaa (content.json and voyage.rs unchanged since 45b7378). Status: draft canon, phase 2. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Category | A (port landmark) | - |
| Port | [Palm Cove](../ports/palm_cove.md) | - |
| Region | West Africa | C:882 |
| Shipped text | "The Distillery Row - seven rum houses, each with its own secret recipe and fierce pride." [ascii-folded] | C:12087 |
| Sea mark | none | voyage.rs:592-607 |
| Player sees it | yes, as the Landmark line on arrival | crates/portlight-sim/src/port_arrival_engine.rs:157-159 |
| Mechanic | none [flavour, no mechanic] | - |

## 1. Location (all [invented] unless a cite or a named data field is given)

Seven rum houses along the hillside above the cove (landmark); barrels roll down ramps to the boats (dock scene). The Third Distillery (C:8806) is one of the seven.

No coordinates exist for landmarks; the chart position is the port's own map x/y. Placing landmarks on the chart is a future phase.

## 2. Data cites

- dock_scene: "Small boats bob in the sheltered cove. Rum barrels roll down wooden ramps from the hilltop distilleries. Children dive for coins tossed by arriving crews." (C:12090)
- atmosphere: "Sweet molasses, fermenting sugar cane, and the lazy hum of bees." (C:12089)
- tavern_rumor: "Old Cassius at the third distillery claims his rum can cure fever, heal wounds, and make you invisible to customs inspectors. Two of those are probably true." (C:12091)
- local_custom: "A cup of rum for every stranger. Refusing is an insult that lasts generations." (C:12088)
- regional festival: Dyer's Moon (C:11864)
- regional festival: Pearl Tide (C:11877)

## 3. History (all [invented] unless cited)

- Each house claims to be oldest; canon names none [invented].

## 4. Appearance and palette (art guidance only; commissions nothing; the character hold stands)

A terrace of seven thatched stillhouses stepping up the hill, copper stills glinting through open sides, barrel ramps to the water. [invented]

Palette: drawn from the Palm Cove port palette ([port file section C.3](../ports/palm_cove.md)), so the landmark reads as part of its port. Values and statuses are the port file's; judged in sRGB against the envelope (luma 65-141, chroma p98 35; small-area accents up to chroma 64). No sea or water swatch.

| Role | Port swatch | sRGB | Luma / chroma | Status |
|---|---|---|---|---|
| Mass | palm-thatch brown (Dominant) | (113, 99, 78) #71634E | 100 / 35 | in range |
| Detail | cask oak (Secondary 2) | (96, 78, 61) #604E3D | 81 / 35 | in range |
| Accent | amber rum (Accent) | (140, 111, 77) #8C6F4D | 115 / 63 | small-area accent (chroma 63) |

Kit: compose from the shipped will-use plates; no named landmark has art today. Any new plate is a gap for the art gap list, not a commission.

## 5. People and use (all [invented] unless cited)

Distillers, Old Cassius at the third distillery (rumour), boat crews, children diving for coins.

## 6. Sound hook (all [invented] sound design unless quoted from a cite)

- Barrels rumbling down wooden ramps; copper still ticks; bees.
- Fragments: none proposed yet [proposal]. No music, no voice; #45 ear hold unaffected.

## 7. Links to ports

- Owning port: [Palm Cove](../ports/palm_cove.md)
- [Sun Harbor](../ports/sun_harbor.md)

## 8. Flags

- Near-duplicate: Distillery Row vs The Third Distillery. Consistent in data: the Third Distillery is one house on the Row.
