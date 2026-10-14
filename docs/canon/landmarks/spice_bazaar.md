# The Spice Bazaar (spice_bazaar)

Base commit: 71c6aaa (content.json and voyage.rs unchanged since 45b7378). Status: draft canon, phase 2. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Category | A (port landmark) | - |
| Port | [Al-Manar](../ports/al_manar.md) | - |
| Region | Mediterranean | C:291 |
| Shipped text | "The Spice Bazaar - a covered market where a hundred aromas compete for your attention." [ascii-folded] | C:12017 |
| Sea mark | none | voyage.rs:592-607 |
| Player sees it | yes, as the Landmark line on arrival | crates/portlight-sim/src/port_arrival_engine.rs:157-159 |
| Mechanic | none [flavour, no mechanic] | - |
| Also an institution | `am_bazaar` | C:7196 |

## 1. Location (all [invented] unless a cite or a named data field is given)

A covered market three city blocks long behind the harbour, reached by the marble steps from the quay (dock scene; institution text).

No coordinates exist for landmarks; the chart position is the port's own map x/y. Placing landmarks on the chart is a future phase.

## 2. Data cites

- dock_scene: "Feluccas with triangular sails crowd the harbor. Porters carry brass-bound chests of spice up marble steps worn smooth by centuries of trade." (C:12020)
- atmosphere: "Cinnamon, cardamom, and the murmur of a hundred negotiations conducted in whispers." (C:12019)
- tavern_rumor: "A merchant from the east claims to know a spice island no chart has ever shown. He's been buying a lot of rum and saying little else." (C:12021)
- local_custom: "Bargaining begins with tea. Refusing the first cup insults the seller." (C:12018)
- named in `name` at C:7197
- named in `rivalry_reason` at C:13535
- regional festival: Harvest of Plenty (C:11753)
- regional festival: Night of Lanterns (C:11766)
- named lane: The Grain Road (Porto Novo - Al-Manar, C:1933) - lanes are deferred to a later phase
- named lane: The Monsoon Shortcut (Al-Manar - Monsoon Reach, C:2284) - lanes are deferred to a later phase

## 3. History (all [invented] unless cited)

- T4: the bazaar is roofed over, joining open stalls into one covered market (existing timeline).
- The Night of Lanterns is held along its aisles and on the harbour (festival C:11766).

## 4. Appearance and palette (art guidance only; commissions nothing; the character hold stands)

A long low roof line of plaster domes and vaults, brass lamps and herb bundles hung from the ceiling inside; marble steps worn smooth. From sea: a pale band with small dark arcade openings. [invented]

Palette: drawn from the Al-Manar port palette ([port file section C.3](../ports/al_manar.md)), so the landmark reads as part of its port. Values and statuses are the port file's; judged in sRGB against the envelope (luma 65-141, chroma p98 35; small-area accents up to chroma 64). No sea or water swatch.

| Role | Port swatch | sRGB | Luma / chroma | Status |
|---|---|---|---|---|
| Mass | sun-baked plaster (Dominant) | (143, 129, 108) #8F816C | 130 / 35 | in range |
| Detail | shade under awnings (Secondary 1) | (88, 80, 70) #585046 | 81 / 18 | in range |
| Accent | saffron dust (Accent) | (152, 129, 88) #988158 | 131 / 64 | small-area accent (chroma 64) |

Kit: compose from the shipped will-use plates; no named landmark has art today. Any new plate is a gap for the art gap list, not a commission.

## 5. People and use (all [invented] unless cited)

Spice factors, porters with brass-bound chests (dock scene), the Khoury family's agents. Negotiations are murmured, never shouted [cite: atmosphere].

## 6. Sound hook (all [invented] sound design unless quoted from a cite)

- Murmured bargaining under a vaulted roof; brass lamp chains; spice scoops on scale pans.
- Fragments: none proposed yet [proposal]. No music, no voice; #45 ear hold unaffected.

## 7. Links to ports

- Owning port: [Al-Manar](../ports/al_manar.md)
- [Porto Novo](../ports/porto_novo.md)
- [Monsoon Reach](../ports/monsoon_reach.md)

## 8. Flags

- none
