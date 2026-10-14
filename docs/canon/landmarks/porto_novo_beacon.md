# The beacon of Porto Novo (porto_novo_beacon)

Base commit: 71c6aaa (content.json and voyage.rs unchanged since 45b7378). Status: draft canon, phase 2. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Category | B (sea mark) | - |
| Port | [Porto Novo](../ports/porto_novo.md) | - |
| Region | Mediterranean | C:210 |
| Shipped text | "The beacon of Porto Novo breaks through the haze. You're on course." | crates/portlight-sim/src/voyage.rs:594 |
| Player sees it | yes, as a random voyage event on any lane (DF-19) | voyage.rs:592-607 |
| Mechanic | none [flavour, no mechanic] | - |

## 1. Location (all [invented] unless a cite or a named data field is given)

[invented] A lamp on the roof of The Harbor Master's Tower, the stone tower at the harbour mouth (C:6958). The tower's brass telescope shares the roof. The beacon is not in content.json; it exists only in the voyage lighthouse line (crates/portlight-sim/src/voyage.rs:594).

No coordinates exist for landmarks; the chart position is the port's own map x/y. Placing landmarks on the chart is a future phase.

## 2. Data cites

- named in `name` at C:6959
- voyage lighthouse line: crates/portlight-sim/src/voyage.rs:594
- regional festival: Harvest of Plenty (C:11753)
- regional festival: Night of Lanterns (C:11766)
- named lane: The Grain Road (Porto Novo - Al-Manar, C:1933) - lanes are deferred to a later phase
- named lane: The Timber Run (Porto Novo - Silva Bay, C:1942) - lanes are deferred to a later phase
- named lane: The Shadow Lane (Porto Novo - Corsair's Rest, C:1960) - lanes are deferred to a later phase
- named lane: The Cotton Crossing (Porto Novo - Sun Harbor, C:2032) - lanes are deferred to a later phase

## 3. History (all [invented] unless cited)

- T18: the Harbor Master's office puts a fire basket, later a lamp, on the tower roof so grain barges can find the harbour mouth before dawn [invented].

## 4. Appearance and palette (art guidance only; commissions nothing; the character hold stands)

A small lamp housing on a square stone tower; by day just a dark iron cage beside the telescope, by night a warm point of light. Small-area accent only. [invented]

Palette: drawn from the Porto Novo port palette ([port file section C.3](../ports/porto_novo.md)), so the landmark reads as part of its port. Values and statuses are the port file's; judged in sRGB against the envelope (luma 65-141, chroma p98 35; small-area accents up to chroma 64). No sea or water swatch.

| Role | Port swatch | sRGB | Luma / chroma | Status |
|---|---|---|---|---|
| Mass | harbour slate (Secondary 2) | (92, 96, 98) #5C6062 | 95 / 6 | in range |
| Detail | limewashed stone in shade (Dominant) | (120, 114, 104) #787268 | 115 / 16 | in range |
| Accent | Exchange brass (Accent) | (148, 126, 84) #947E54 | 128 / 64 | small-area accent (chroma 64) |

Kit: compose from the shipped will-use plates; no named landmark has art today. Any new plate is a gap for the art gap list, not a commission.

## 5. People and use (all [invented] unless cited)

Kept by the Harbor Master's clerks (Vasco's office, C:6958 area). Lit at dusk, trimmed at midnight, doused at dawn when the Exchange reads its rates [invented].

## 6. Sound hook (all [invented] sound design unless quoted from a cite)

- None at the beacon itself; from sea it is silent. Hook only: oil hiss and a lamp door closing [invented].
- Fragments: none proposed yet [proposal]. No music, no voice; #45 ear hold unaffected.

## 7. Links to ports

- Owning port: [Porto Novo](../ports/porto_novo.md)
- [Al-Manar](../ports/al_manar.md)
- [Silva Bay](../ports/silva_bay.md)
- [Corsair's Rest](../ports/corsairs_rest.md)
- [Sun Harbor](../ports/sun_harbor.md)

## 8. Flags

- Data gap: no beacon or lighthouse in Porto Novo data. Canon places it on the Harbor Master's Tower. See DF-18.
- The lighthouse event can fire on any lane, so a captain in the South Seas can 'see' Porto Novo's beacon (voyage.rs:592-607). Canon does not explain this away; it is flagged as DF-19.
