# Sun Harbor (sun_harbor)

Base commit: 143e284 (content.json unchanged since 45b7378). Status: draft canon, phase 1. Written 2026-10-14.
Cites `C:n` are crates/portlight-sim/data/content.json line n at 143e284. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] (design-notes pattern) | DF-n see data-fixes.md | T-n see README shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Region | West Africa ("The Gold Coast") | C:803, C:11838 |
| Bloc | The Gold Coast Compact (Sun Harbor, Palm Cove, Iron Point, Pearl Shallows) | C:13579, C:13431 |
| Features | none | C:803-879 |
| Warehouse tiers | depot, regional | C:6541 |
| Port fee / provisions / repair / crew | 5 / 2 / 4 per hull point / 3 | C:803-879 |
| Map | x 14, y 22 | C:879 |
| Lanes | 6 | section F |
| Player description | "Golden coast port where cotton bales stack higher than the warehouses." | C:805 |

## A. Specialties and trade goods

<!-- hand-maintained:begin -->
| Good | Category | Base | Affinity | Stock target | Restock | Role |
|---|---|---|---|---|---|---|
| Cotton | commodity | 15 | 1.5 | 30 | 3.0 | exporter |
| Dyes | commodity | 22 | 1.4 | 18 | 2.0 | exporter |
| Pelts | commodity | 8 | 1.4 | 12 | 2.0 | exporter |
| Iron Ore | commodity | 25 | 1.3 | 22 | 2.0 | exporter |
| Rum | provision | 20 | 0.8 | 12 | 1.0 | importer |
| Porcelain | luxury | 60 | 0.55 | 5 | 0.5 | importer |
| Silk | luxury | 70 | 0.5 | 5 | 0.5 | importer |
| Spice | luxury | 55 | 0.5 | 8 | 0.8 | importer |
<!-- hand-maintained:end -->

Market cite: C:803-879. Price rule: base x (target / stock) / affinity (crates/portlight-sim/src/economy.rs:37); affinity above 1 is cheap here.
Not traded here: Grain, Timber, Tea, Pearls, Tobacco, Weapons, Medicines, Opium, Black Powder, Stolen Cargo.
Region goods [flavour, no mechanic]: sacred Pearls; prized Dyes, Cotton, Rum; forbidden none (C:11838 area).
Festival: Dyer's Moon, every 60 days for 3, market effects Dyes x2.0, Cotton x1.6 (C:11863).
Festival: Pearl Tide, every 90 days for 4, market effects Pearls x2.2, Rum x1.3 (C:11876).
Contract templates touching West Africa: `proc_grain_feed` (procurement), `proc_medicines_relief` (procurement), `ret_cotton_return` (return_freight), `ret_spice_restock` (return_freight), `ret_dyes_textile` (return_freight), `circ_med_africa` (circuit), `circ_indies_loop` (circuit), `smug_powder_delivery` (smuggling). Source port Sun Harbor: none.

## B. People

- Cultural group: The Weighers: "The women who count and weigh every bale. Their word on quality is final." (C:12076 area).
- Local custom: "The first trade of the day is blessed by an elder. Smart captains arrive early." (C:12076 area).
- Regional ethos: "The land gives freely to those who give back. Trade is a relationship, not a transaction. The elders remember every ship that entered the harbor." Greeting: "You return! The coast remembers you, Captain." Proverb: "The market remembers who gave, not who took." (C:11838 area).
- Tavern rumour (in-world, unreliable): "A ship arrived last month carrying nothing but empty crates - paid in full. The harbor master looked the other way. Nobody asks about empty crates here." (C:12076 area).

| Name | Title | Temperament | Institution | Cite |
|---|---|---|---|---|
| Adama Keita | Harbor Elder | welcoming | harbor_master | C:8634 |
| Ama Mensah | Chief Weigher | authoritative | exchange | C:8656 |
| Grandmother Binta | Port Elder | wise | governor | C:8678 |
| Kofi Asante | Cotton Master | proud | shipyard | C:8700 |
| Yusuf Diallo | Provisions Master | generous | tavern | C:8722 |
| Inspector Amara | Trade Inspector | fair | customs | C:8744 |
| Fatou Ndiaye | Contract Broker | rising | broker | C:8766 |
| Adaeze | Arms Dealer (merchant) | charming | outfitter | C:5960 |
| Kolo the Fist | style master | - | - | C:6205 |

- Dress [invented]: loose cotton in undyed and indigo-dyed cloth. The character hold stands: no likenesses or portraits.
- Speech [cite]: built from the greeting lines in each NPC record above (greeting_neutral, friendly, hostile).
- Temperament of the town: read from the power structure and tension in section E.
- Network (merchant, professional): Marta Soares - Ama Mensah, linking Porto Novo (C:11412). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Network (merchant, allied): Fatou Ndiaye - Factor Wu Jian, linking Jade Port (C:11424). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Network (tavern, allied): Mama Joy - Yusuf Diallo, linking Palm Cove (C:11544). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Network (broker, allied): Kamala Nair - Fatou Ndiaye, linking Monsoon Reach (C:11616). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Sea sighting filed under this port: Ama Mensah (C:12858).
- Hire and training canon (links only): [Dr. Amara](../companions/dr_amara.md) (companion, Surgeon); [Kolo the Fist](../style-masters/kolo_the_fist.md) (style master, Dambe); [Adaeze](../merchants/adaeze_arms.md) (merchant, Arms Dealer).

## C. Visual identity (guidance only; commissions nothing; the character hold stands)

Pattern: the shared visual-identity headings 1-7 (design notes). Shared camera, envelope and do/don't blocks are in README.md and not repeated here.

1. Architecture. Identity line [invented]: The Cotton Steps are the town: Sun Harbor is a marketplace on stairs. Landmark [cite]: "The Cotton Steps - wide stone stairs where cotton bales are displayed and auctioned." (C:12076 area). Cited buildings: The Welcome Dock (harbor_master, C:8562); The Cotton Steps (exchange, C:8572); The Acacia Throne (governor, C:8582); The Cotton House (shipyard, C:8592); Yusuf's Table (tavern, C:8602); The Honest Scale (customs, C:8612); The Contract Mat (broker, C:8622). Kit use [proposal]: compose from the shipped will-use plates; identity carriers are ground choice and building frequency, never new ids. Gaps follow the art gap list (design notes). Landmark canon: ../landmarks/cotton_steps.md.
2. Materials: local and abundant (exports): Cotton, Iron Ore, Dyes, Pelts; scarce or luxury (imports): Silk, Spice, Rum, Porcelain (section A). Atmosphere [cite]: "Warm earth, indigo dye, and the rhythmic singing of dockworkers loading bales." (C:12076 area). Climate: section G.
3. Palette (references, not UI tokens; UI stays INK / GOLD / CREAM / MUTED / DUE). Judged in sRGB against the plate envelope (luma 65-141, chroma p98 35):

| Role | Family | sRGB | Luma / chroma | Status | Tie |
|---|---|---|---|---|---|
| Dominant | sunlit stone steps | (137, 124, 102) #897C66 | 125 / 35 | in range | the Cotton Steps |
| Secondary 1 | cotton bale | (134, 128, 116) #868074 | 128 / 18 | in range | bales stacked high |
| Secondary 2 | indigo cloth, muted | (80, 86, 104) #505668 | 86 / 24 | in range | dyes export |
| Accent | ochre dust | (145, 119, 82) #917752 | 122 / 63 | small-area accent (chroma 63) | red earth and dye |

4. Dress: see section B. 5. Signage: ASCII only, no UI-duplicating words; motifs [invented]: the weigher's scale, a cotton boll. 6. Silhouette: chart scale is the shared `chart_port_marker`; harbour-scale skyline [invented]: broad stone stairs rising from the quay with warehouses either side, bales stacked higher than the roofs. 7. Do and don't: README shared list.

## D. Historical analogies (feed [invented] prose; never a cite)

- **Medieval Kano-style cloth markets, as a market function.** Fits: a port where cotton and dyes are weighed and auctioned in public (Cotton 1.5, Dyes 1.4). Suggests: elders bless the first trade; weighers are trusted officials. Limit: borrow the market function only (analogy rule 1). Avoid: costume or ethnic coding of any kind.

## E. Governance

- Title: Port Elder (C:8556).
- Power structure: "Sun Harbor is governed by Grandmother Binta from under the acacia tree - consensus leadership through wisdom, not decree. Ama's counting staff is the Compact's commercial authority. Kofi runs the Cotton Steps. Adama welcomes everyone. Yusuf feeds everyone. The port runs on relationships and memory rather than paperwork and hierarchy." [ascii-folded] (C:8558).
- Internal tension: "The tension is between tradition and ambition - but unlike Porto Novo, here they're not enemies. Fatou's ambition (Silk Circle contracts, equal partnership status) serves the Compact's interests. The question is pace: Binta wants to move slowly, Fatou wants to move now. Ama is preparing Fatou for the counting staff, which means Ama is preparing to step down. The transition hasn't been discussed openly. Nobody on the Gold Coast discusses succession openly - it happens when the elders decide it's time. The deeper tension: the Iron Pact's failed bid to buy Iron Point left scars. The Compact blocked it, but the industrial pressure hasn't gone away." (C:8559).
- Bloc stance: "Sun Harbor is the Compact's voice. The Weighers speak for all four ports at trade councils." (C:13579 area).
- Grudge: "An Iron Pact mining consortium tried to buy Iron Point. The Compact blocked it. Relations have been cool since." (C:13579 area).
- Bloc philosophy: "Generosity first, then business. The coast remembers." (C:13431 area). Loyalty and spillover numbers are [flavour, no mechanic] (DF-15).
- Faction canon: bloc [The Gold Coast Compact](../factions/gold_coast_compact.md).

## F. Relations with every other port

Hand-maintained from content.json. Disposition is bloc-to-bloc (symmetric in data); lanes are undirected; flows are directional by market affinity; networks are symmetric.

<!-- hand-maintained:begin -->
| Port | Region | Bloc | Disposition | Lane (distance, min class, danger) | Flows out (they import) | Flows in (we import) | Network |
|---|---|---|---|---|---|---|---|
| Porto Novo | Mediterranean | The Exchange Alliance | neutral (C:13725) | 40, cutter, 0.12 (C:2027) | Cotton, Dyes, Pelts | Rum | C:11412 |
| Al-Manar | Mediterranean | The Exchange Alliance | neutral (C:13725) | 48, brigantine, 0.15 (C:2036) | Iron Ore, Pelts | Spice | - |
| Silva Bay | Mediterranean | The Exchange Alliance | neutral (C:13725) | - | Cotton | - | - |
| Corsair's Rest | Mediterranean | The Shadow Ports | neutral (C:13809) | - | - | Rum | - |
| Ironhaven | North Atlantic | The Iron Pact | rival (C:13760) | - | Cotton | - | - |
| Stormwall | North Atlantic | The Iron Pact | rival (C:13760) | - | Pelts | - | - |
| Thornport | North Atlantic | The Free Ports | allied (C:13802) | - | - | - | - |
| Palm Cove | West Africa | The Gold Coast Compact | same bloc | 20, sloop, 0.1 (C:2054) | Iron Ore, Pelts | Rum | C:11544 |
| Iron Point | West Africa | The Gold Coast Compact | same bloc | 18, sloop, 0.09 (C:2063) | Pelts | - | - |
| Pearl Shallows | West Africa | The Gold Coast Compact | same bloc | 24, sloop, 0.1 (C:2081) | - | - | - |
| Jade Port | East Indies | The Silk Circle | neutral (C:13795) | - | Cotton, Iron Ore, Pelts | Porcelain, Silk, Spice | C:11424 |
| Monsoon Reach | East Indies | The Free Ports | allied (C:13802) | - | Cotton, Pelts | Silk, Spice | C:11616 |
| Silk Haven | East Indies | The Silk Circle | neutral (C:13795) | - | Cotton, Dyes, Iron Ore, Pelts | Porcelain, Silk | - |
| Crosswind Isle | East Indies | The Free Ports | allied (C:13802) | 64, galleon, 0.18 (C:2099) | Pelts | - | - |
| Dragon's Gate | East Indies | The Silk Circle | neutral (C:13795) | - | Cotton, Iron Ore, Pelts | Porcelain | - |
| Spice Narrows | East Indies | The Shadow Ports | neutral (C:13809) | - | Pelts | Spice | - |
| Ember Isle | South Seas | The Coral Crown | allied (C:13816) | - | Cotton, Pelts | - | - |
| Typhoon Anchorage | South Seas | The Coral Crown | allied (C:13816) | - | Pelts | Spice | - |
| Coral Throne | South Seas | The Coral Crown | allied (C:13816) | - | Pelts | Rum | - |
<!-- hand-maintained:end -->

- Lane lore: "The Cotton Crossing": "Mediterranean grain south, Gold Coast cotton north. The route that clothed an empire." (C:2027). Lane canon: [The Cotton Crossing](../lanes/cotton_crossing.md).
- Lane lore: "The Long Crossing": "Weeks of open water. The route that separates traders from merchants. Many set out. Fewer arrive." (C:2099). Lane canon: [The Long Crossing](../lanes/long_crossing.md).

## G. World details (undefined in data; all [invented] unless cited)

- History (shared timeline in README.md): T10: Sun Harbor hosts the oath of the Gold Coast Compact [invented]. T15: The Compact blocks an Iron Pact consortium's bid for Iron Point (grudge text).
- Population [invented]: about 15,000.
- Language [invented]: Gold Coast trade speech; weighers' counting words.
- Currency: silver is the only money in the sim (every price, fee and wage is in silver). Local coin names, if any, are [invented] flavour and never replace 'silver' in copy.
- Religion [invented]: ancestor rites led by elders; the first trade of the day is blessed (custom).
- Climate [invented port detail]: hot, humid, a wet season of heavy rain; bright sun on the steps. Regional weather [cite]: "Warm air, heavy with moisture. The coast shimmers in the heat." (C:11838 area, weather_flavor).
- Pirates in these waters [cite]: The Deep Reef Brotherhood (C:2896).

## I. Sound and ambience (all [invented] sound design unless quoted from a cite)

- **Soundscape:** "the rhythmic singing of dockworkers loading bales" (C:12079); bales thumping; scales creaking.
- **Signature:** cotton bale onto a stack (soft, heavy); beam scale settling; morning blessing is a quiet beat.
- **Music/work songs (NOTE ONLY):** canon's strongest work song - the Weighers "carrying the count in song" (C:8576). Flag for MUSIC when the hold lifts; never as SFX (no voice).
- **Vs neighbours:** the singing port; soft-fibre thuds vs Iron Point's metal.
- **Fragments [proposal; no fragment or asset exists yet]:** `src.cotton_bale`, `src.beam_scale`, `place.bale_quay`.
- Cited hooks: atmosphere "Warm earth, indigo dye, and the rhythmic singing of dockworkers loading bales." (C:12079); dock scene "Cotton bales stack three stories high along the waterfront. Women in indigo cloth oversee the weighing, their voices carrying the count in song." (C:12080).
- Invented bed idea: auction calls on the steps, the elder's blessing at dawn. No assets are requested.

## J. Flags for this port

- DF-4: Institutions of type `shipyard` at ports without the shipyard feature
- DF-10: 'Gold Coast' names both a region and a bloc
