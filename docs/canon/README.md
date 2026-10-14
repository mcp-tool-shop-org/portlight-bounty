# Portlight canon

Base commit: 143e284 (phase 1: ports); 71c6aaa (phase 2: landmarks); ab03ace (phase 3: lanes). Status: draft (docs/canon/). Written 2026-10-14. Inputs: content.json and the sim at the base commit, plus design research notes (not in this repo).

## Purpose

The canon is the single written reference for what Portlight's ports are like: people, look, governance, relations and history. It keeps new copy, art briefs, sound and content consistent with what the game already does. It adds no mechanics.

## The rule: canon never contradicts the sim or shipped copy. Flag, don't change.

1. Precedence, highest first:
   1. sim behaviour;
   2. content.json;
   3. shipped player-facing copy;
   4. canon prose.

   Canon loses every tie.
2. Every grounded fact carries a cite. `C:n` means crates/portlight-sim/data/content.json line n at the base commit; source files are cited by path:line. Anything else is marked [invented].
3. When lore and the game disagree, canon resolves it in lore where it can (and says so in the port file's section J). Data contradictions go to data-fixes.md with file:line and a proposed fix. Canon never edits the game.
4. Data with no mechanic behind it (bloc loyalty numbers, sacred goods, timber grades) is written as [flavour, no mechanic].
5. Copy rules:
   - ASCII, with ' - ' as the separator;
   - quotes from content.json are folded to ASCII;
   - catalog names in prose, ids only in cites;
   - Option A: no Trust, Standing or Heat as player terms;
   - 'silver' is the only currency word;
   - house style: proper names keep their data spelling (Sun Harbor); the common noun is 'harbour' (DF-9);
   - 'the Gold Coast' is the region and 'the Compact' is the bloc (DF-10).
6. Holds: canon commissions no art, sound or spend. The character hold stands. `man_of_war` art is held (art hold), although the ship exists in lore and in the sim.

## Halfway-world rule

No canon rule names a fixed list of ports, blocs, goods or regions. The port set is content.json `ports[*].id`. Section 0, A and F tables and relations-matrix.md are hand-maintained from content.json (there is no generator). A new port appears in the tables when they are updated by hand; its lore cells start empty.

## How to add a port

1. Add the port to the sim first (content.json ports, routes, port_politics, culture, institutions), through a sim PR with its parity note. Canon never introduces a port.
2. Create ports/<port_id>.md from any existing port file. Keep sections 0, A-J.
3. Update the hand-maintained tables and the matrix from content.json.
4. Write sections B-E and G-I from cited data. Mark every invention. Place the port's history on the shared timeline below; add a timeline row only if no existing one fits.
5. Add contradictions to data-fixes.md. Never fix the data in the canon PR.
6. Review: design review (canon), art review (section C), analogy review (section D), sound review (section I).

## Port file sections

0 at a glance (hand-maintained) | A trade goods (hand-maintained plus prose) | B people | C visual identity (visual pattern) | D analogies (analogy pattern; at most three plus one contrast) | E governance | F relations with every other port (hand-maintained) | G world details (history, population, language, currency, religion, climate, pirates; invented and marked) | H extra depth (thin ports only) | I sound and ambience | J flags.

## Shared visual blocks (section C refers here)

- Camera: dimetric 2:1, plate camera euler (60, 0, 45), tile 256x128, sun euler (73.4, 0, -54.9), Blender 5.2 EEVEE. sx = 128(X+Y), sy = 64 + 64(X-Y). Any new plate matches this camera or is re-gated.
- Envelope: judge colour in sRGB with `hdr_2d = false`. Luma p2 65, median 102, p98 141; chroma median 11, p98 35.
  - A swatch outside this range is labelled 'mood, not values'.
  - Accents up to chroma 64 are allowed in small areas only.
  - World palettes are references, not UI tokens. UI stays INK / GOLD / CREAM / MUTED / DUE.
  - Sea and water colour are deferred to the water renders; port palettes carry no sea swatch.
  - Within a region, port dominants sit at least 10 luma apart, so ports read apart even inside the muted envelope.
- Do: compose from the 14 will-use plates; keep `hdr_2d = false`; send any new plate through ANDON and the art gate with exact 2x (Addendum M.1).
- Don't:
  - use the forbidden paths (`assets/iso/`, `_rejected/`, `assets/rooms/`, `assets/floors/`, `assets/tilesets/`, `stage/room_stage.gd`);
  - use `man_of_war` art (held);
  - remake a shipped plate unless its ANDON is red;
  - put painterly props in the iso harbour;
  - draw character figures or portraits;
  - resample chart water;
  - spend or open a harbour set without the operator.
- Signage: ASCII only, at most one cited motto on the landmark, and no world text that duplicates UI copy.

## Shared timeline (all [invented] unless a cite is named; years before the game's Day 1, 'BS')

| ID | When | Event | Grounding |
|---|---|---|---|
| T1 | ~1000 BS | Silk Haven's Loom Quarter patterns are first woven | 'patterns a thousand years old' (Silk Haven landmark, C:12136 area) |
| T2 | ~700 BS | The Coral Palace begins to grow; the Ember Keepers settle the volcano | 'grown over centuries' (C:12196 area) |
| T3 | ~500 BS | The old Northern Crown raises the Stormwall Fortress | 'older than anyone remembers' (C:12056 area) |
| T4 | ~400 BS | Al-Manar's bazaar is roofed; Porto Novo builds its Grain Exchange on temple ruins | Al-Manar 'Ancient port' (C:293) |
| T5 | ~300 BS | The Exchange Alliance is signed at Porto Novo, which keeps the headquarters | 'centuries of contract law' (C:13388); HQ grudge (C:13541 area) |
| T6 | ~200 BS | Silva Bay begins carving ship names in the Brotherhood Hall | 'two hundred years' (C:7414 area) |
| T7 | ~150 BS | The Silk Circle is formalised; Dragon's Gate raises its towers and chain | [invented] |
| T8 | ~120 BS | The Northern Crown dissolves; Stormwall's garrison holds the straits alone | resolves Stormwall's 'not the crown' (C:13570 area) |
| T9 | ~90 BS | Thornport's whaling era; the Whale Arch is raised | 'decades of salt' (C:12066 area) |
| T10 | ~80 BS | The Gold Coast Compact is sworn among four West African ports | [invented date] |
| T11 | ~60 BS | The Iron Pact is founded (Ironhaven foundries supply Stormwall); Ember Isle refuses its medicine exclusivity | bloc text (C:13410) |
| T12 | ~50 BS | The Captain's Table charters Crosswind Isle as a free port; the Free Ports and the Coral Crown form | [invented date] |
| T13 | ~40 BS | Porto Novo's strict code drives smugglers to Corsair's Rest; the Spice Lords take the Narrows; the Crimson Tide and Monsoon Syndicate settle in | rivalry texts (C:13671, C:13680 areas) |
| T14 | ~25 BS | The Monsoon Shortcut links Al-Manar to Monsoon Reach; Stormwall's two-week tea tax on Thornport | route lore; Thornport grudge |
| T15 | ~12 BS | Silva Bay refuses the Iron Pact's war-fleet commission; Ironhaven weapons are found at Corsair's Rest | grudges (C:13550, C:13559 areas) |
| T16 | ~1 BS | Typhoon Anchorage divers poach Pearl Shallows' reef 'three seasons ago'; Ember Peak grumbles more often | grudge and tension texts |
| T17 | Day 1 | The player's voyage begins | the sim |

Rows T18 on were added with the landmark canon (phase 2). Ids are stable, so read them by date, not by number:

| Id | When | Event | Grounding |
|---|---|---|---|
| T18 | ~180 BS | Porto Novo puts a beacon on the Harbor Master's Tower roof | voyage.rs:594 names the beacon; its place is [invented] (DF-18) |
| T19 | ~45 BS | Bones begins carving lost whalers' names on the Whale Arch; the Whale Bone copies them into its beams | C:8362, C:8499 (the names); date [invented] |
| T20 | ~70 BS | Typhoon Anchorage lashes its first wrecks into the Storm Wall | 'built from the hulls of ships that didn't make it' (C:12187); date [invented] |
| T21 | ~15 BS | Dragon's Gate raises its chains and does not lower them again | 'not lowered the chains in fifteen years' (tavern rumour, C:12161) |
| T22 | ~50 BS | Crosswind Isle hangs the Free Port Bell at the charter (with T12) | [invented] |
| T23 | ~35 BS | The Hanging Market reaches its present span across the Narrows | [invented] |

Rows T24 on were added with the lane canon (phase 3). Read them by date, not by number:

| Id | When | Event | Grounding |
|---|---|---|---|
| T24 | ~600 BS | Grain ships first sail the Grain Road | 'since before memory' (C:1934); date [invented] |
| T25 | ~250 BS | Grain south, cotton north: the Cotton Crossing becomes a regular trade | 'Been this way for centuries' (C:12388); date [invented] |
| T26 | ~100 BS | The first recorded Long Crossing | driftwood prayer markers (C:12408); event and date [invented] |
| T27 | ~30 BS | Old Farouk sails the Monsoon Shortcut seven times; navigators copy his charts (before T14) | C:12568; date [invented] |
| T28 | ~100 BS | Pirate kings hold both ends of the Smuggler's Run; their line ends before T13 | 'once controlled both ends' (C:2294); date [invented] |
| T29 | ~55 BS | Ironhaven iron first goes east by the Northern Passage (after T11) | 'Iron west, porcelain east' (C:2303); date [invented] |

Every port file's section G cites these IDs. A port history may add detail but must not move an event to another era.

## Pirate home ports ([invented]; data gives factions regions only)

- The Crimson Tide (territory Mediterranean, North Atlantic, C:2879): home Corsair's Rest. The cove's governor is 'The Tide's Voice' and Scarlet Ana is sighted off the Rest.
- The Monsoon Syndicate (territory East Indies, C:2888): home Spice Narrows. 'Raj the Quiet's Shadow' sits in the Narrows' institutions and Typhoon Mei is sighted there.
- The Iron Wolves (territory North Atlantic, C:2905): home none. The Wolves keep no harbour. They raid from unnamed fjords north of Stormwall; Sergeant Kruze is sighted off Ironhaven.
- The Deep Reef Brotherhood (territory South Seas, West Africa, C:2896): home none. The Brotherhood lives at sea on reef anchorages between Pearl Shallows and Ember Isle, and keeps no port.

## World conventions ([invented] unless cited)

- Currency: silver everywhere. The sim prices everything in silver; canon never names another money in copy.
- Languages: one trade speech per region (Middle Sea, Northern, Gold Coast, Silk Waters, Reef Kingdoms), all mutually workable at a quay. No invented words appear in player copy.
- Religion: regional and quiet; no faith has a mechanic. Port faiths are in each section G.
- Population: rough figures for art and writing scale only, never shown as numbers in copy.

## Landmarks (phase 2)

- One file per shipped port landmark (20, from `culture.ports[].landmark`, C:12007-12197) plus one sea mark found only in code (the Porto Novo beacon, voyage.rs:594): see landmarks/README.md.
- Each port file links its landmark in section C.1. The five sea-mark ports (Porto Novo, Ironhaven, Monsoon Reach, Ember Isle, Thornport) also link it in section I.
- Landmark palettes reuse their port's swatches, so a landmark always reads as part of its port.
- The 15 named lanes have their own files from phase 3 (lanes/README.md).
- Not in data, future phase: no reefs, rocks, wrecks (other than the Storm Wall), lighthouses (other than the code-only beacon) or forts exist as data objects. No landmark has coordinates, art, an icon or a sound.
- Phase 2 data issues are DF-18 to DF-21 in data-fixes.md.

## Lanes (phase 3)

- One file per route with a `lore_name` (15 of 43, C:1928-2312). The name is shown on the chart lane label (crates/portlight-chart/src/view.rs:213-223) and keys a route encounter table (C:12210-12669; sea_culture.rs:121-136).
- Each endpoint port links its lanes in section F (Lane lore lines); landmark files link the lanes that pass them.
- Lanes have no drawn course, art or sound. Phase 3 data issues are DF-22 to DF-25 in data-fixes.md.

## Files

- README.md (this file)
- ports/<port_id>.md, one per content.json port
- relations-matrix.md
- data-fixes.md (proposals for the game, not canon; DF-n items)
- landmarks/README.md and landmarks/<landmark_id>.md (21 files)
- lanes/README.md and lanes/<lane_id>.md (15 files)

Deepened thin ports (inventory 'Undefined'): Dragon's Gate, Spice Narrows, Silk Haven, Coral Throne, Ember Isle.
