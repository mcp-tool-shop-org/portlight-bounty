# The Wind Temple (wind_temple)

Base commit: 71c6aaa (content.json and voyage.rs unchanged since 45b7378). Status: draft canon, phase 2. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Category | A (port landmark) | - |
| Port | [Monsoon Reach](../ports/monsoon_reach.md) | - |
| Region | East Indies | C:1206 |
| Shipped text | "The Wind Temple - a pagoda on the headland where monks track the monsoon." [ascii-folded] | C:12127 |
| Sea mark | "The Wind Temple pagoda catches the sunset. Monsoon Reach is near." | crates/portlight-sim/src/voyage.rs:596 |
| Player sees it | yes, as the Landmark line on arrival | crates/portlight-sim/src/port_arrival_engine.rs:157-159 |
| Mechanic | none [flavour, no mechanic] | - |
| Also an institution | `mr_temple` | C:9666 |

## 1. Location (all [invented] unless a cite or a named data field is given)

A pagoda on the headland over the curved harbour (landmark; The Headland Harbor C:9656). The Temple Hall (C:9696) is its lower hall.

No coordinates exist for landmarks; the chart position is the port's own map x/y. Placing landmarks on the chart is a future phase.

## 2. Data cites

- dock_scene: "The harbor curves around a headland crowned by the Wind Temple. Ships here are built with deeper keels and stronger masts - the monsoon demands it." (C:12130)
- atmosphere: "Salt spray, incense from the Wind Temple, and the creak of monsoon-tested rigging." (C:12129)
- tavern_rumor: "The monks say this monsoon season will be the worst in a generation. The shipwrights are quietly raising their prices." (C:12131)
- local_custom: "Captains consult the monks before sailing. Their wind forecasts are uncannily accurate." (C:12128)
- named in `power_structure` at C:9652
- named in `description` at C:9660
- named in `name` at C:9667
- named in `description` at C:9700
- named in `description` at C:9734
- named in `description` at C:9756
- named in `description` at C:9822
- named in `rumor` at C:9849
- named in `atmosphere` at C:12129
- named in `dock_scene` at C:12130
- voyage lighthouse line: crates/portlight-sim/src/voyage.rs:596
- regional festival: Silk Weavers' Fair (C:11920)
- regional festival: Monsoon Prayer (C:11934)
- named lane: Typhoon Alley (Monsoon Reach - Typhoon Anchorage, C:2230) - lanes are deferred to a later phase
- named lane: The Monsoon Shortcut (Al-Manar - Monsoon Reach, C:2284) - lanes are deferred to a later phase

## 3. History (all [invented] unless cited)

- T14: the Monsoon Shortcut opens; the monks' forecasts make it sailable.
- Instruments passed down for generations (institution text).

## 4. Appearance and palette (art guidance only; commissions nothing; the character hold stands)

A tiered pagoda on a green headland, wind vanes on every tier, catches the sunset (voyage.rs:596). [invented]

Palette: drawn from the Monsoon Reach port palette ([port file section C.3](../ports/monsoon_reach.md)), so the landmark reads as part of its port. Values and statuses are the port file's; judged in sRGB against the envelope (luma 65-141, chroma p98 35; small-area accents up to chroma 64). No sea or water swatch.

| Role | Port swatch | sRGB | Luma / chroma | Status |
|---|---|---|---|---|
| Mass | teak and lacquer, muted (Dominant) | (111, 91, 77) #6F5B4D | 94 / 34 | in range |
| Detail | jungle green, muted (Secondary 2) | (76, 92, 74) #4C5C4A | 87 / 18 | in range |
| Accent | temple ochre (Accent) | (144, 117, 80) #907550 | 120 / 64 | small-area accent (chroma 64) |

Kit: compose from the shipped will-use plates; no named landmark has art today. Any new plate is a gap for the art gap list, not a commission.

## 5. People and use (all [invented] unless cited)

Monks tracking the monsoon, the five-member council, shipwrights who read their forecasts.

## 6. Sound hook (all [invented] sound design unless quoted from a cite)

- Pagoda bells with each council decision (institution text); wind in vanes.
- Fragments: none proposed yet [proposal]. No music, no voice; #45 ear hold unaffected.

## 7. Links to ports

- Owning port: [Monsoon Reach](../ports/monsoon_reach.md)
- [Al-Manar](../ports/al_manar.md)
- [Typhoon Anchorage](../ports/typhoon_anchorage.md)

## 8. Flags

- none
