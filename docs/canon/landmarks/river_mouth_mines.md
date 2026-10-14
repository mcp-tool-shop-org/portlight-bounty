# The River Mouth Mines (river_mouth_mines)

Base commit: 71c6aaa (content.json and voyage.rs unchanged since 45b7378). Status: draft canon, phase 2. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Category | A (port landmark) | - |
| Port | [Iron Point](../ports/iron_point.md) | - |
| Region | West Africa | C:961 |
| Shipped text | "The River Mouth Mines - tunnels carved into red cliffs where the river meets the sea." [ascii-folded] | C:12097 |
| Sea mark | none | voyage.rs:592-607 |
| Player sees it | yes, as the Landmark line on arrival | crates/portlight-sim/src/port_arrival_engine.rs:157-159 |
| Mechanic | none [flavour, no mechanic] | - |

## 1. Location (all [invented] unless a cite or a named data field is given)

Tunnels in the red cliffs where the river meets the sea (landmark); ore carts run down to The River Mouth Dock (C:9030).

No coordinates exist for landmarks; the chart position is the port's own map x/y. Placing landmarks on the chart is a future phase.

## 2. Data cites

- dock_scene: "Ore carts rattle down tracks to the loading docks. The river runs red with mine runoff. Everything here - the buildings, the clothes, the skin - is stained rust." (C:12100)
- atmosphere: "Red dust, the metallic tang of iron, and the echo of picks from the mine shafts." (C:12099)
- tavern_rumor: "The miners hit something in the deep shaft last week. Not iron - something older. The foreman sealed the tunnel and doubled the guard." (C:12101)
- local_custom: "Miners spit on their hands before a deal. It means the ore is honest." (C:12098)
- regional festival: Dyer's Moon (C:11864)
- regional festival: Pearl Tide (C:11877)

## 3. History (all [invented] unless cited)

- T10: Iron Point joins the Compact for protection of its mines.
- The sealed deep shaft (rumour) is canon as sealed and unexplained.

## 4. Appearance and palette (art guidance only; commissions nothing; the character hold stands)

Dark tunnel mouths in red cliffs, ore tracks, a red-stained river. Everything wears the red dust (dock scene). [invented]

Palette: drawn from the Iron Point port palette ([port file section C.3](../ports/iron_point.md)), so the landmark reads as part of its port. Values and statuses are the port file's; judged in sRGB against the envelope (luma 65-141, chroma p98 35; small-area accents up to chroma 64). No sea or water swatch.

| Role | Port swatch | sRGB | Luma / chroma | Status |
|---|---|---|---|---|
| Mass | red cliff earth, muted (Dominant) | (107, 84, 72) #6B5448 | 88 / 35 | in range |
| Detail | ore grey (Secondary 2) | (96, 94, 92) #605E5C | 94 / 4 | in range |
| Accent | rust (Accent) | (138, 98, 75) #8A624B | 105 / 63 | small-area accent (chroma 63) |

Kit: compose from the shipped will-use plates; no named landmark has art today. Any new plate is a gap for the art gap list, not a commission.

## 5. People and use (all [invented] unless cited)

Miners, the foreman, ore-cart crews.

## 6. Sound hook (all [invented] sound design unless quoted from a cite)

- Picks echoing from shafts (atmosphere); carts on rails.
- Fragments: none proposed yet [proposal]. No music, no voice; #45 ear hold unaffected.

## 7. Links to ports

- Owning port: [Iron Point](../ports/iron_point.md)
- [Ironhaven](../ports/ironhaven.md)
- [Sun Harbor](../ports/sun_harbor.md)

## 8. Flags

- Near-duplicate: Mines vs River Mouth Dock. Resolved: the Dock is the loading quay below the Mines.
