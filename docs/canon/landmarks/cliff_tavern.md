# The Cliff Tavern (cliff_tavern)

Base commit: 71c6aaa (content.json and voyage.rs unchanged since 45b7378). Status: draft canon, phase 2. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Category | A (port landmark) | - |
| Port | [Corsair's Rest](../ports/corsairs_rest.md) | - |
| Region | Mediterranean | C:459 |
| Shipped text | "The Cliff Tavern - carved into the rock face, visible only from the sea." [ascii-folded] | C:12037 |
| Sea mark | none | voyage.rs:592-607 |
| Player sees it | yes, as the Landmark line on arrival | crates/portlight-sim/src/port_arrival_engine.rs:157-159 |
| Mechanic | none [flavour, no mechanic] | - |

## 1. Location (all [invented] unless a cite or a named data field is given)

Carved into the rock face above the hidden cove, visible only from the sea (landmark). Mama Lucia's Kitchen (C:7684) is its back room, cut deeper into the rock (phase 1 ruling).

No coordinates exist for landmarks; the chart position is the port's own map x/y. Placing landmarks on the chart is a future phase.

## 2. Data cites

- dock_scene: "Ships anchor in the hidden cove with no flags flying. A boy in a rowboat approaches to collect the docking fee - payable in silver, no questions." (C:12040)
- atmosphere: "Torch smoke, cheap rum, and the clink of coins changing hands under the table." (C:12039)
- tavern_rumor: "There's a passage through the cliffs that leads to a second harbor. Only the oldest pirates know the way, and they aren't talking." (C:12041)
- local_custom: "No names. Every captain here is 'friend' until proven otherwise." (C:12038)
- regional festival: Harvest of Plenty (C:11753)
- regional festival: Night of Lanterns (C:11766)
- named lane: The Shadow Lane (Porto Novo - Corsair's Rest, C:1960) - lanes are deferred to a later phase
- named lane: The Smuggler's Run (Corsair's Rest - Spice Narrows, C:2293) - lanes are deferred to a later phase

## 3. History (all [invented] unless cited)

- T13: when Porto Novo's strict code drives smugglers south, the old sea cave becomes a tavern.
- The rumoured passage through the cliffs to a second harbour starts behind the Kitchen's fire pit [invented; the rumour is cited].

## 4. Appearance and palette (art guidance only; commissions nothing; the character hold stands)

No building: a row of shuttered openings and a lantern-lit terrace cut into dark cliff, only seen from seaward. Smoke from the Kitchen's smoke hole is the giveaway. [invented]

Palette: drawn from the Corsair's Rest port palette ([port file section C.3](../ports/corsairs_rest.md)), so the landmark reads as part of its port. Values and statuses are the port file's; judged in sRGB against the envelope (luma 65-141, chroma p98 35; small-area accents up to chroma 64). No sea or water swatch.

| Role | Port swatch | sRGB | Luma / chroma | Status |
|---|---|---|---|---|
| Mass | shadowed cliff rock (Dominant) | (77, 71, 65) #4D4741 | 72 / 12 | in range |
| Detail | tarred rope and dark hulls (Secondary 1) | (71, 65, 59) #47413B | 66 / 12 | in range |
| Accent | lantern amber (Accent) | (140, 113, 76) #8C714C | 116 / 64 | small-area accent (chroma 64) |

Kit: compose from the shipped will-use plates; no named landmark has art today. Any new plate is a gap for the art gap list, not a commission.

## 5. People and use (all [invented] unless cited)

Crews of no flag, Mama Lucia at the fire pit, brokers from the Cliff Caves (C:7704 area). Nobody uses names.

## 6. Sound hook (all [invented] sound design unless quoted from a cite)

- Low talk in a stone room; a fire pit; coins changing hands under the table (atmosphere).
- Fragments: none proposed yet [proposal]. No music, no voice; #45 ear hold unaffected.

## 7. Links to ports

- Owning port: [Corsair's Rest](../ports/corsairs_rest.md)
- [Porto Novo](../ports/porto_novo.md)
- [Spice Narrows](../ports/spice_narrows.md)

## 8. Flags

- Player-visible near-duplicate: the arrival line says 'The Cliff Tavern', the institution is 'Mama Lucia's Kitchen'. Resolved in lore: one place; the Cliff Tavern is the whole cave, the Kitchen is its back room and Lucia's name for it.
