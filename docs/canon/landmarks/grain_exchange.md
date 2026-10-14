# The Grain Exchange (grain_exchange)

Base commit: 71c6aaa (content.json and voyage.rs unchanged since 45b7378). Status: draft canon, phase 2. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Category | A (port landmark) | - |
| Port | [Porto Novo](../ports/porto_novo.md) | - |
| Region | Mediterranean | C:210 |
| Shipped text | "The Grain Exchange - a columned hall where prices are read aloud at dawn." [ascii-folded] | C:12007 |
| Sea mark | none | voyage.rs:592-607 |
| Player sees it | yes, as the Landmark line on arrival | crates/portlight-sim/src/port_arrival_engine.rs:157-159 |
| Mechanic | none [flavour, no mechanic] | - |
| Also an institution | `pn_exchange` | C:6968 |

## 1. Location (all [invented] unless a cite or a named data field is given)

On the waterfront, the largest building in Porto Novo (institution text). Its steps face the stone quays where the grain barges berth; the Broker's Desk is a back office inside it (C:7018 area).

No coordinates exist for landmarks; the chart position is the port's own map x/y. Placing landmarks on the chart is a future phase.

## 2. Data cites

- dock_scene: "Grain barges jostle for position along the stone quays. Clerks with wax tablets tally every bushel while porters sweat under heavy sacks." (C:12010)
- atmosphere: "Flour dust, sea salt, and the shouts of dockhands loading grain barges." (C:12009)
- tavern_rumor: "They say the Grain Exchange is built on the ruins of an older temple. Some nights the clerks hear chanting from beneath the floor." (C:12011)
- local_custom: "Captains ring the harbor bell on first arrival. It's bad luck not to." (C:12008)
- named in `name` at C:6969
- named in `tavern_rumor` at C:12011
- named in `rivalry_reason` at C:13535
- regional festival: Harvest of Plenty (C:11753)
- regional festival: Night of Lanterns (C:11766)
- named lane: The Grain Road (Porto Novo - Al-Manar, C:1933) - lanes are deferred to a later phase
- named lane: The Timber Run (Porto Novo - Silva Bay, C:1942) - lanes are deferred to a later phase
- named lane: The Shadow Lane (Porto Novo - Corsair's Rest, C:1960) - lanes are deferred to a later phase
- named lane: The Cotton Crossing (Porto Novo - Sun Harbor, C:2032) - lanes are deferred to a later phase

## 3. History (all [invented] unless cited)

- T4: built on the ruins of an older temple (the tavern rumour says so; canon treats the temple as real and the chanting as unproven).
- T5: the Exchange Alliance is signed in the hall; the Exchange becomes the Alliance's working floor.
- The dawn reading of rates from the steps is the oldest unbroken custom in the Middle Sea [invented].

## 4. Appearance and palette (art guidance only; commissions nothing; the character hold stands)

Columned hall of limewashed stone, wide steps to the quay, a bronze bell frame over the pediment for the great bell (see Porto Novo bells below). Grain-sack hemp awnings along the colonnade. From the water it reads as a long low pale block with a dark colonnade band. [invented]

Palette: drawn from the Porto Novo port palette ([port file section C.3](../ports/porto_novo.md)), so the landmark reads as part of its port. Values and statuses are the port file's; judged in sRGB against the envelope (luma 65-141, chroma p98 35; small-area accents up to chroma 64). No sea or water swatch.

| Role | Port swatch | sRGB | Luma / chroma | Status |
|---|---|---|---|---|
| Mass | limewashed stone in shade (Dominant) | (120, 114, 104) #787268 | 115 / 16 | in range |
| Detail | grain-sack hemp (Secondary 1) | (126, 112, 91) #7E705B | 113 / 35 | in range |
| Accent | Exchange brass (Accent) | (148, 126, 84) #947E54 | 128 / 64 | small-area accent (chroma 64) |

Kit: compose from the shipped will-use plates; no named landmark has art today. Any new plate is a gap for the art gap list, not a commission.

## 5. People and use (all [invented] unless cited)

Clerks with wax tablets (dock scene), porters, factors, the Exchange Guild. Marta sets prices here (power structure, C:6954 area). Dawn: rates read aloud from the steps; noon: trading floor; dusk: tallies sealed [invented rhythm].

## 6. Sound hook (all [invented] sound design unless quoted from a cite)

- A single voice reading rates at dawn over harbour murmur; wax-tablet scratches; sacks thudding on stone.
- Fragments: none proposed yet [proposal]. No music, no voice; #45 ear hold unaffected.

## 7. Links to ports

- Owning port: [Porto Novo](../ports/porto_novo.md)
- [Al-Manar](../ports/al_manar.md)
- [Silva Bay](../ports/silva_bay.md)
- [Corsair's Rest](../ports/corsairs_rest.md)
- [Sun Harbor](../ports/sun_harbor.md)

## 8. Flags

- Porto Novo bells, resolved in lore: there are two bells. (1) The salvaged bell over the door of The Harbor Bell tavern is the 'harbor bell' captains ring on first arrival (custom C:12008; tavern C:6998); the tavern is named for it. (2) The Exchange's great bell hangs over this hall and is rung only for the Harvest of Plenty (C:11753). No third bell exists.
