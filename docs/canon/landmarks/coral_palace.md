# The Coral Palace (coral_palace)

Base commit: 71c6aaa (content.json and voyage.rs unchanged since 45b7378). Status: draft canon, phase 2. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Category | A (port landmark) | - |
| Port | [Coral Throne](../ports/coral_throne.md) | - |
| Region | South Seas | C:1839 |
| Shipped text | "The Coral Palace - grown over centuries from living reef, rising from the lagoon like a crown." [ascii-folded] | C:12197 |
| Sea mark | none | voyage.rs:592-607 |
| Player sees it | yes, as the Landmark line on arrival | crates/portlight-sim/src/port_arrival_engine.rs:157-159 |
| Mechanic | none [flavour, no mechanic] | - |
| Also an institution | `ct_palace` | C:11240 |

## 1. Location (all [invented] unless a cite or a named data field is given)

Rises from the lagoon behind the narrow reef entrance (landmark; dock scene). The Pearl Hall (C:11230) is a chamber inside.

No coordinates exist for landmarks; the chart position is the port's own map x/y. Placing landmarks on the chart is a future phase.

## 2. Data cites

- dock_scene: "The lagoon entrance is narrow and treacherous - local pilots guide every ship through the reef. The Coral Palace rises ahead, its walls glittering with embedded shells and pearls. Warriors in war canoes escort you to the royal dock." (C:12200)
- atmosphere: "Warm lagoon water, crushed coral underfoot, and the distant beat of ceremonial drums." (C:12199)
- tavern_rumor: "The Coral King has no heir. Three princes compete for the throne, each courting foreign captains for weapons and alliances. Smart traders play all three." (C:12201)
- local_custom: "All trade begins with tribute to the Coral King. Silk or weapons preferred. Refusal is exile." (C:12198)
- named in `description` at C:11234
- named in `name` at C:11241
- named in `dock_scene` at C:12200
- named in `arrival_text` at C:13216
- regional festival: Coral Coronation (C:11976)
- regional festival: Fire Walking (C:11990)
- named lane: The Deep South Run (Pearl Shallows - Coral Throne, C:2311) - lanes are deferred to a later phase

## 3. History (all [invented] unless cited)

- T2: the palace begins to grow, centuries ago.
- The Coral Coronation renews the king's oath here (C:11976).

## 4. Appearance and palette (art guidance only; commissions nothing; the character hold stands)

Pale reef stone grown into towers, shells and pearls set in the walls. [invented]

Palette: drawn from the Coral Throne port palette ([port file section C.3](../ports/coral_throne.md)), so the landmark reads as part of its port. Values and statuses are the port file's; judged in sRGB against the envelope (luma 65-141, chroma p98 35; small-area accents up to chroma 64). No sea or water swatch.

| Role | Port swatch | sRGB | Luma / chroma | Status |
|---|---|---|---|---|
| Mass | pale reef stone (Dominant) | (132, 126, 112) #847E70 | 126 / 20 | in range |
| Detail | lagoon shallows, muted (Secondary 1) | (92, 112, 108) #5C706C | 107 / 20 | in range |
| Accent | pearl and tribute silk, small (Accent) | (148, 136, 120) #948878 | 137 / 28 | in range |

Kit: compose from the shipped will-use plates; no named landmark has art today. Any new plate is a gap for the art gap list, not a commission.

## 5. People and use (all [invented] unless cited)

The Coral King, three rival princes (rumour), pilots who guide ships in.

Extra depth (thin port): [invented] New rooms are grown, not built: divers plant coral on frames and wait a generation; each king adds one room.

## 6. Sound hook (all [invented] sound design unless quoted from a cite)

- Ceremonial drums (atmosphere); paddles of the escort.
- Fragments: none proposed yet [proposal]. No music, no voice; #45 ear hold unaffected.

## 7. Links to ports

- Owning port: [Coral Throne](../ports/coral_throne.md)
- [Pearl Shallows](../ports/pearl_shallows.md)
- [Ember Isle](../ports/ember_isle.md)
- [Typhoon Anchorage](../ports/typhoon_anchorage.md)

## 8. Flags

- none
