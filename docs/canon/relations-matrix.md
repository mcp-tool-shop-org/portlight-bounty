# Cross-port relations matrix

Base commit: 143e284. Generated from content.json (port_politics.relationships, routes, ports[*].market, cross_port_networks), so no port list is hand-typed. Regenerate whenever content.json changes.

Cell format: `{disposition}{lane}{flow}{net}`.
- disposition: S same bloc | A allied | N neutral | R rival | H hostile (bloc to bloc; symmetric in data);
- lane: ` - L{distance}{class initial}` when a direct route exists (undirected);
- flow: ` - x{n}`, the number of goods row exports (affinity > 1) that column imports (affinity < 1); directional by nature, so x(row, col) need not equal x(col, row);
- net: ` - *` when a cross_port_networks link joins the two ports (symmetric).

Column keys: PN = Porto Novo, AM = Al-Manar, SB = Silva Bay, CR = Corsair's Rest, I = Ironhaven, S = Stormwall, T = Thornport, SH = Sun Harbor, PC = Palm Cove, IP = Iron Point, PS = Pearl Shallows, JP = Jade Port, MR = Monsoon Reach, SH = Silk Haven, CI = Crosswind Isle, DG = Dragon's Gate, SN = Spice Narrows, EI = Ember Isle, TA = Typhoon Anchorage, CT = Coral Throne

| from \ to | PN | AM | SB | CR | I | S | T | SH | PC | IP | PS | JP | MR | SH | CI | DG | SN | EI | TA | CT |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Porto Novo | . | S - L24s - x2 - * | S - L16s - x2 - * | H - L18s - x1 | A - L36c - x2 | A - x1 | N | N - L40c - x1 - * | N - x1 | N - x2 | N - x2 | R - x2 | N - x2 | R - x2 - * | N - * | R - x1 - * | H - x2 | N - x2 | N - x2 | N - x1 |
| Al-Manar | S - L24s - * | . | S - L20s | H - x1 | A | A | N - x2 | N - L48b - x1 | N | N - x1 | N - x1 | R | N - L72g - x1 - * | R - x1 | N | R - x1 | H | N | N - x1 | N |
| Silva Bay | S - L16s - x2 - * | S - L20s - x2 | . | H - L14s | A - L32b - x1 | A - x1 - * | N | N | N - L44b - x3 | N - x2 | N | R - x2 | N - x2 | R - x2 | N - x1 | R - x2 | H - x1 | N - x2 | N - x1 | N - x1 |
| Corsair's Rest | H - L18s - x1 | H - x2 | H - L14s - x2 | . | H - x2 | H - L40b - x3 | N | N - x1 | N - x1 | N - x3 | N - x1 | R - x2 | N - x2 | R - x2 | N - x1 | R - x3 | S - L80g - x4 - * | N - x3 | N - x2 | N - x2 |
| Ironhaven | A - L36c - x1 | A - x2 | A - L32b - x1 | H | . | S - L20s - x1 - * | N - L22s | R | R - x2 | R - x2 - * | R | H - L76g - x2 | N - x1 | H - x2 | N - x1 | H - x2 - * | H - x2 | N - x2 | N - x1 | N - x2 |
| Stormwall | A | A - x1 | A - x2 - * | H - L40b - x2 | S - L20s - x1 - * | . | N - L18s - x1 | R | R - x1 - * | R - x3 | R - x2 | H - x1 | N - x2 | H - x1 | N | H - x1 | H - x2 | N - x2 - * | N - x2 | N - x2 |
| Thornport | N - x2 | N - x2 | N | N | N - L22s - x2 | N - L18s - x2 | . | A | A - x2 | A - x2 | A | N - x1 | S - x2 | N - x1 | S - x1 | N - x2 - * | N - x1 | N - x2 | N - x1 | N - x2 |
| Sun Harbor | N - L40c - x3 - * | N - L48b - x2 | N - x1 | N | R - x1 | R - x1 | A | . | S - L20s - x2 - * | S - L18s - x1 | S - L24s | N - x3 - * | A - x2 - * | N - x4 | A - L64g - x1 | N - x3 | N - x1 | A - x2 | A - x1 | A - x1 |
| Palm Cove | N - x1 | N - x1 | N - L44b - x2 | N | R - x3 | R - x2 - * | A | S - L20s - x1 - * | . | S - L22s - x1 | S - L20s - x1 | N - x2 | A - x2 | N - x2 | A | N - x3 | N - x1 | A - x2 | A - x1 | A |
| Iron Point | N | N - x1 | N | N | R - * | R | A | S - L18s | S - L22s - x1 | . | S | N - x1 - * | A | N - x1 | A - L60b | N - x1 | N | A | A | A |
| Pearl Shallows | N - x2 | N | N - x1 | N | R - x1 | R | A | S - L24s | S - L20s | S | . | N - x1 | A - x1 | N - x2 | A - L56b | N - x1 | N | A - x1 | A | A - L68g |
| Jade Port | R - x1 | R - x3 | R - x1 | R - x1 | H - L76g - x1 | H - x1 | N - x3 | N - x3 - * | N - x1 | N - x1 - * | N - x1 | . | N - L20s - x1 | S - L18s - x1 - * | N - L28s | S - L22s | R - x1 | A | A - x2 | A - x3 |
| Monsoon Reach | N | N - L72g - x1 - * | N - x1 | N - x1 | N - x1 | N - x1 | S - x2 | A - x2 - * | A - x1 | A | A - x1 | N - L20s | . | N - L22s - x1 | S - L24s | N | N - L26b - x1 | N | N - L52g - x1 | N - x1 |
| Silk Haven | R - x1 - * | R - x2 | R - x1 | R - x1 | H - x1 | H - x1 | N - x2 | N - x2 | N - x1 | N - x1 | N - x1 | S - L18s - * | N - L22s - x1 | . | N - L32b | S | R - L20b - x1 | A | A - x2 | A - x2 |
| Crosswind Isle | N - * | N | N | N | N | N | S | A - L64g | A | A - L60b | A - L56b | N - L28s | S - L24s | N - L32b | . | N - L30b | N | N - L56g | N | N |
| Dragon's Gate | R - x1 - * | R - x2 | R | R | H - * | H | N - x1 - * | N - x1 | N | N - x1 | N | S - L22s | N - x1 | S | N - L30b | . | R - L28b | A | A - x1 | A - x2 |
| Spice Narrows | H | H | H | S - L80g - x1 - * | H | H | N - x1 | N - x1 | N | N | N | R | N - L26b | R - L20b - x1 | N | R - L28b | . | N - L48g | N | N |
| Ember Isle | N - x1 | N - x1 | N | N - x1 | N | N - * | N - x1 | A | A - x1 | A - x1 | A - x1 | A - x1 | N - x1 | A - x2 | N - L56g | A - x2 | N - L48g | . | S - L24s - x1 - * | S - L28s - * |
| Typhoon Anchorage | N - x1 | N | N | N | N - x1 | N | N - x1 | A - x1 | A - x1 | A - x1 | A | A | N - L52g - x1 | A - x1 | N | A | N | S - L24s - x1 - * | . | S - L22b |
| Coral Throne | N | N - x1 | N - x1 | N | N - x2 | N - x2 | N | A - x1 | A | A - x1 | A - L68g - x1 | A - x1 | N - x1 | A - x1 | N | A - x2 | N - x1 | S - L28s - x1 - * | S - L22b - x1 | . |

Symmetry check (generated): disposition, lane and network parts are identical in (row, col) and (col, row) for every pair. Flow parts differ by design.

## Asymmetries in data (flagged, not changed)

- Corsair's Rest names Porto Novo as local rival (C:13671); Porto Novo names Al-Manar (C:13532). Canon reads local_rival_port as directional: a grievance, not a mutual feud. See DF-6.
- Silk Haven names Jade Port as local rival (C:13624); Jade Port names none (C:13615). Canon reads local_rival_port as directional: a grievance, not a mutual feud. See DF-6.
- Spice Narrows names Jade Port as local rival (C:13680); Jade Port names none (C:13615). Canon reads local_rival_port as directional: a grievance, not a mutual feud. See DF-6.
- The Silk Circle lists The Iron Pact in `hostile_to` (C:13453) but not the reverse (C:13410); the relationships table says `hostile` (C:13767).

## Per-bloc summary (generated)

- The Exchange Alliance: Porto Novo, Al-Manar, Silva Bay. Partners The Iron Pact; rivals The Silk Circle; hostile to The Shadow Ports (C:13388).
- The Iron Pact: Ironhaven, Stormwall. Partners The Exchange Alliance; rivals The Gold Coast Compact; hostile to The Shadow Ports (C:13410).
- The Gold Coast Compact: Sun Harbor, Palm Cove, Iron Point, Pearl Shallows. Partners The Free Ports, The Coral Crown; rivals The Iron Pact; hostile to - (C:13431).
- The Silk Circle: Jade Port, Silk Haven, Dragon's Gate. Partners The Coral Crown; rivals The Exchange Alliance; hostile to The Iron Pact (C:13453).
- The Free Ports: Crosswind Isle, Monsoon Reach, Thornport. Partners The Gold Coast Compact; rivals -; hostile to - (C:13475).
- The Shadow Ports: Corsair's Rest, Spice Narrows. Partners -; rivals -; hostile to The Exchange Alliance, The Iron Pact (C:13493).
- The Coral Crown: Ember Isle, Typhoon Anchorage, Coral Throne. Partners The Gold Coast Compact, The Silk Circle; rivals -; hostile to - (C:13511).

## Hand notes

| Row port | Column port | Note | Grounding |
|---|---|---|---|
| Porto Novo | Silva Bay | The Timber Run: Silva Bay timber built Porto Novo's fleet | C:1942 |
| Porto Novo | Al-Manar | Headquarters against harbour: the oldest genteel feud in the Middle Sea | C:13541 |
| Corsair's Rest | Porto Novo | The Rest exists because of Porto Novo's law (T13) | C:13671 |
| Silva Bay | Stormwall | The war-fleet refusal (T15) | C:13550 |
| Ironhaven | Iron Point | Refined against raw ore; the mutual rivalry | C:13559 |
| Pearl Shallows | Typhoon Anchorage | Reef poaching three seasons ago (T16); mutual | C:13606 |
| Silk Haven | Jade Port | Thread against clay; one-sided in data | C:13624 |
| Spice Narrows | Jade Port | Undercutting in the dark; one-sided in data | C:13680 |
| Dragon's Gate | Crosswind Isle | [invented] The Gate inspects what the Isle lets through; each calls the other's harbour 'a leak' | [invented] |
| Ember Isle | Ironhaven | Refused the Iron Pact's medicine exclusivity (T11) | C:13689 |
