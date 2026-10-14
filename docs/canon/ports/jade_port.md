# Jade Port (jade_port)

Base commit: 143e284 (content.json unchanged since 45b7378). Status: draft canon, phase 1. Seat: Game Designer, 2026-10-14.
Cites `C:n` are crates/portlight-sim/data/content.json line n at 143e284. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] (AD or Analogist pattern) | DF-n see canon-data-fixes.md | T-n see README shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Region | East Indies ("The Silk Waters") | C:1119, C:11892 |
| Bloc | The Silk Circle (Jade Port, Silk Haven, Dragon's Gate) | C:13615, C:13453 |
| Features | none | C:1119-1203 |
| Warehouse tiers | depot, regional, commercial | C:6552 |
| Port fee / provisions / repair / crew | 10 / 3 / 3 per hull point / 7 | C:1119-1203 |
| Map | x 34, y 10 | C:1203 |
| Lanes | 5 | section F |
| Player description | "Porcelain workshops line the waterfront. Iron and grain are worth their weight in gold here." | C:1121 |

## A. Specialties and trade goods

<!-- generated:begin -->
| Good | Category | Base | Affinity | Stock target | Restock | Role |
|---|---|---|---|---|---|---|
| Porcelain | luxury | 60 | 1.6 | 30 | 3.0 | exporter |
| Silk | luxury | 70 | 1.3 | 22 | 2.5 | exporter |
| Tea | luxury | 40 | 1.3 | 18 | 2.0 | exporter |
| Spice | luxury | 55 | 1.1 | 18 | 1.5 | exporter |
| Pelts | commodity | 8 | 0.7 | 6 | 0.8 | importer |
| Rum | provision | 20 | 0.6 | 8 | 0.8 | importer |
| Cotton | commodity | 15 | 0.6 | 8 | 0.8 | importer |
| Grain | commodity | 12 | 0.5 | 8 | 0.8 | importer |
| Iron Ore | commodity | 25 | 0.5 | 5 | 0.5 | importer |
<!-- generated:end -->

Market cite: C:1119-1203. Price rule: base x (target / stock) / affinity (crates/portlight-sim/src/economy.rs:37); affinity above 1 is cheap here.
Not traded here: Timber, Dyes, Pearls, Tobacco, Weapons, Medicines, Opium, Black Powder, Stolen Cargo.
Region goods [flavour, no mechanic]: sacred Porcelain, Silk; prized Spice, Tea; forbidden Weapons (C:11892 area).
Festival: Silk Weavers' Fair, every 75 days for 4, market effects Silk x2.0, Porcelain x1.5, Dyes x1.4 (C:11919).
Festival: Monsoon Prayer, every 90 days for 3, market effects Spice x1.8, Medicines x1.6, Tea x1.4 (C:11933).
Contract templates touching East Indies: `proc_grain_feed` (procurement), `proc_timber_shipyard` (procurement), `proc_iron_forge` (procurement), `proc_medicines_relief` (procurement), `lux_spice_monopolist` (luxury_discreet), `ret_cotton_return` (return_freight), `ret_spice_restock` (return_freight), `circ_indies_loop` (circuit), `smug_opium_run` (smuggling), `smug_powder_delivery` (smuggling). Source port Jade Port: none.

## B. People

- Cultural group: The Kiln Masters: "Porcelain artisans whose guild marks are more valued than the clay itself." (C:12116 area).
- Local custom: "Porcelain is presented on silk, never bare-handed. Fingerprints on porcelain void a sale." (C:12116 area).
- Regional ethos: "Patience is mastery. The artisan who perfects one bolt of silk in a lifetime is honored above the merchant who trades a thousand. Everything here was old when your civilization was young." Greeting: "You honor us, Captain. Tea will be prepared while we discuss terms." Proverb: "A merchant who rushes drinks salt water." (C:11892 area).
- Tavern rumour (in-world, unreliable): "A master potter smashed his entire year's work because he found a single flaw. The guild praised him. That kind of standard is why their porcelain costs what it does." (C:12116 area).

| Name | Title | Temperament | Institution | Cite |
|---|---|---|---|---|
| Harbor Director Li Wei | Harbor Director | formal | harbor_master | C:9494 |
| Kiln Master Chen Bai | Guild Patriarch | austere | exchange | C:9516 |
| Lady Mei Ling | Circle Magistrate | cultivated | governor | C:9538 |
| Tea Master Huang | Tea Pavilion Keeper | observant | tavern | C:9560 |
| Inspector Zhao Min | Quality Inspector | uncompromising | customs | C:9582 |
| Factor Wu Jian | Trade Factor | smooth | broker | C:9604 |
| Apprentice Lin Yue | Kiln Apprentice | intense | apothecary | C:9626 |
| Master Chen | Master Smith (merchant) | quiet | outfitter | C:6003 |
| Pak Reza | style master | - | - | C:6221 |

- Dress [invented]: layered cotton and silk for factors; kiln workers in plain smocks. The character hold stands: no likenesses or portraits.
- Speech [cite]: built from the greeting lines in each NPC record above (greeting_neutral, friendly, hostile).
- Temperament of the town: read from the power structure and tension in section E.
- Network (merchant, allied): Fatou Ndiaye - Factor Wu Jian, linking Sun Harbor (C:11424). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Network (merchant, allied): Yaa Acheampong - Factor Wu Jian, linking Iron Point (C:11436). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Network (merchant, rival): Merchant Feng - Kiln Master Chen, linking Silk Haven (C:11508). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Network (inspector, professional): Inspector Zhao Min - Inspector Yuki, linking Silk Haven (C:11676). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Sea sighting filed under this port: Factor Wu Jian (C:12882).
- Sea sighting filed under this port: Apprentice Lin Yue (C:12906).

## C. Visual identity (guidance only; commissions nothing; the character hold stands)

Pattern: canon-port-silva_bay-visual.md headings 1-7. Shared camera, envelope and do/don't blocks are in README.md and not repeated here.

1. Architecture. Identity line [invented]: The kilns are the town: Jade Port is the Porcelain Quarter with a harbour. Landmark [cite]: "The Porcelain Quarter - a district of workshops where master potters fire kilns day and night." (C:12116 area). Cited buildings: The Ceremonial Dock (harbor_master, C:9422); The Porcelain Quarter (exchange, C:9432); The Magistrate's Hall (governor, C:9442); The Jade Pavilion (tavern, C:9452); The Inspection Hall (customs, C:9462); The Trade Office (broker, C:9472); The Apprentice's Kiln (apothecary, C:9482). Kit use [proposal]: compose from the shipped will-use plates; identity carriers are ground choice and building frequency, never new ids. Gaps follow the PB-001 gap list.
2. Materials: local and abundant (exports): Porcelain, Silk, Tea, Spice; scarce or luxury (imports): Grain, Iron Ore, Rum, Pelts, Cotton (section A). Atmosphere [cite]: "Kiln heat, wet clay, and the delicate tap-tap-tap of artisans painting glaze patterns." (C:12116 area). Climate: section G.
3. Palette (references, not UI tokens; UI stays INK / GOLD / CREAM / MUTED / DUE). Judged in sRGB against the plate envelope (luma 65-141, chroma p98 35):

| Role | Family | sRGB | Luma / chroma | Status | Tie |
|---|---|---|---|---|---|
| Dominant | kiln-fired brick | (120, 100, 88) #786458 | 103 / 32 | in range | the Porcelain Quarter |
| Secondary 1 | celadon glaze, muted | (104, 118, 106) #68766A | 114 / 14 | in range | porcelain |
| Secondary 2 | grey roof tile | (94, 96, 98) #5E6062 | 96 / 4 | in range | workshop roofs |
| Accent | white porcelain | (150, 148, 142) #96948E | 148 / 8 | mood, not values | porcelain on silk |

4. Dress: see section B. 5. Signage: ASCII only, no UI-duplicating words; motifs [invented]: a bowl on a silk square. 6. Silhouette: chart scale is the shared `chart_port_marker`; harbour-scale skyline [invented]: kiln chimneys over a dense quarter of tiled roofs. 7. Do and don't: README shared list.

## D. Historical analogies (feed [invented] prose; never a cite)

- **Jingdezhen's kiln economy, as a craft-guild function.** Fits: a porcelain town where kiln masters set quality (Porcelain 1.6). Suggests: silk under porcelain, inspection before sale, guild bans lasting generations. Limit: borrow guild quality control only; avoid naming dynasties. Avoid: dragon-and-lantern exoticism.

## E. Governance

- Title: Circle Magistrate (C:9416).
- Power structure: "Jade Port is governed by the intersection of craft and court: Chen controls the porcelain (the product), Mei controls the politics (the relationships), and Huang holds the secrets (the leverage). Li Wei enforces protocol. Zhao protects authenticity. Wu handles commerce. Lin represents the uncertain future - a prodigy who may surpass her master, inheriting a guild that must evolve to survive but whose identity is rooted in not changing." [ascii-folded] (C:9418).
- Internal tension: "Three layers of tension: (1) Chen's resistance to change vs. the Circle's need to adapt to Western trade pressure. (2) Mei's letter interception - she's protecting Chen from compromising the Circle, but she's also manipulating him, and Huang knows. (3) Lin's secret experiments - she's innovating behind Chen's back because she believes the tradition must grow. If Chen discovers her new glazes, he'll either embrace them or destroy them. She doesn't know which, and neither does anyone else. The deepest thread: Wu is negotiating with Iron Point's Yaa for raw ore, which would bypass Ironhaven entirely. If it goes through, it connects the Gold Coast's independence movement with the Silk Circle's quality obsession - an alliance that could reshape the entire game's trade network." (C:9419).
- Bloc stance: "Jade Port is the Circle's elder. The Kiln Masters have shaped Eastern trade policy for a thousand years." (C:13615 area).
- Grudge: "A Western merchant once dropped a porcelain bowl. The Kiln Masters banned his entire family from trade for three generations." (C:13615 area).
- Bloc philosophy: "Quality over quantity. Hierarchy governs all." (C:13453 area). Loyalty and spillover numbers are [flavour, no mechanic] (DF-15).

## F. Relations with every other port

Generated. Disposition is bloc-to-bloc (symmetric in data); lanes are undirected; flows are directional by market affinity; networks are symmetric.

<!-- generated:begin -->
| Port | Region | Bloc | Disposition | Lane (distance, min class, danger) | Flows out (they import) | Flows in (we import) | Network |
|---|---|---|---|---|---|---|---|
| Porto Novo | Mediterranean | The Exchange Alliance | rival (C:13732) | - | Porcelain | Grain, Rum | - |
| Al-Manar | Mediterranean | The Exchange Alliance | rival (C:13732) | - | Porcelain, Silk, Tea | - | - |
| Silva Bay | Mediterranean | The Exchange Alliance | rival (C:13732) | - | Silk | Iron Ore, Pelts | - |
| Corsair's Rest | Mediterranean | The Shadow Ports | rival (C:13830) | - | Silk | Pelts, Rum | - |
| Ironhaven | North Atlantic | The Iron Pact | hostile (C:13767) | 76, galleon, 0.22 (C:2297) | Silk | Iron Ore, Pelts | - |
| Stormwall | North Atlantic | The Iron Pact | hostile (C:13767) | - | Silk | Grain | - |
| Thornport | North Atlantic | The Free Ports | neutral (C:13823) | - | Porcelain, Silk, Spice | Pelts | - |
| Sun Harbor | West Africa | The Gold Coast Compact | neutral (C:13795) | - | Porcelain, Silk, Spice | Cotton, Iron Ore, Pelts | C:11424 |
| Palm Cove | West Africa | The Gold Coast Compact | neutral (C:13795) | - | Silk | Cotton, Rum | - |
| Iron Point | West Africa | The Gold Coast Compact | neutral (C:13795) | - | Porcelain | Iron Ore | C:11436 |
| Pearl Shallows | West Africa | The Gold Coast Compact | neutral (C:13795) | - | Silk | Cotton | - |
| Monsoon Reach | East Indies | The Free Ports | neutral (C:13823) | 20, sloop, 0.08 (C:2162) | Porcelain | - | - |
| Silk Haven | East Indies | The Silk Circle | same bloc | 18, sloop, 0.07 (C:2171) | Spice | - | C:11508, C:11676 |
| Crosswind Isle | East Indies | The Free Ports | neutral (C:13823) | 28, sloop, 0.1 (C:2126) | - | - | - |
| Dragon's Gate | East Indies | The Silk Circle | same bloc | 22, sloop, 0.09 (C:2180) | - | - | - |
| Spice Narrows | East Indies | The Shadow Ports | rival (C:13830) | - | Silk | - | - |
| Ember Isle | South Seas | The Coral Crown | allied (C:13837) | - | - | Iron Ore | - |
| Typhoon Anchorage | South Seas | The Coral Crown | allied (C:13837) | - | Porcelain, Silk | - | - |
| Coral Throne | South Seas | The Coral Crown | allied (C:13837) | - | Porcelain, Silk, Tea | Rum | - |
<!-- generated:end -->

- Lane lore: "The Porcelain Lane": "From the free port to the kiln masters. Every piece of porcelain in the west passed through here." (C:2126).
- Lane lore: "The Silk Road by Sea": "Where porcelain meets silk. The two oldest trades in the East, connected by the shortest route." (C:2171).
- Lane lore: "The Northern Passage": "Iron west, porcelain east. The longest profitable route in the world." (C:2297).
- Asymmetry flag: Silk Haven names Jade Port as local rival (C:13624); Jade Port names none (C:13615). Canon reads local_rival_port as directional: a grievance, not a mutual feud. See DF-6.
- Asymmetry flag: Spice Narrows names Jade Port as local rival (C:13680); Jade Port names none (C:13615). Canon reads local_rival_port as directional: a grievance, not a mutual feud. See DF-6.

## G. World details (undefined in data; all [invented] unless cited)

- History (shared timeline in README.md): T7: The Silk Circle is formalised; Jade Port's magistrate sits at its head [invented]. T13: Spice Narrows begins undercutting Jade Port's spice (rivalry text).
- Population [invented]: about 25,000.
- Language [invented]: Silk Waters trade speech; potters' grading terms.
- Currency: silver is the only money in the sim (every price, fee and wage is in silver). Local coin names, if any, are [invented] flavour and never replace 'silver' in copy.
- Religion [invented]: ancestor halls; kilns are lit with a short rite.
- Climate [invented port detail]: warm and humid, monsoon rains. Regional weather [cite]: "Incense drifts from a passing junk. The air is thick with spice." (C:11892 area, weather_flavor).
- Pirates in these waters [cite]: The Monsoon Syndicate (C:2888).

## I. Sound and ambience (placeholder for the Sound seat)

- Status: placeholder. No per-port sound exists (inventory 'Undefined'; studio SOUND_BIBLE covers the Mediterranean slice generically; music is on hold).
- Cited hooks: atmosphere "Kiln heat, wet clay, and the delicate tap-tap-tap of artisans painting glaze patterns."; dock scene "Bamboo cranes lift crates padded with straw from the hold. Each piece of porcelain is inspected by guild markers before it leaves the dock." (C:12116 area).
- Invented bed idea: kiln fires roaring, porcelain ringing when tapped. No assets are requested.

## J. Flags for this port

- DF-6: Local rivals are one-sided
- DF-8: NPC name drift across records
