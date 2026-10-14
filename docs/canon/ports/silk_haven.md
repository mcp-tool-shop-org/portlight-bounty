# Silk Haven (silk_haven)

Base commit: 143e284 (content.json unchanged since 45b7378). Status: draft canon, phase 1. Written 2026-10-14.
Cites `C:n` are crates/portlight-sim/data/content.json line n at 143e284. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] (design-notes pattern) | DF-n see data-fixes.md | T-n see README shared timeline.
Depth: one of the five thinnest ports in data (inventory 'Undefined'); section H adds extra invented depth.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Region | East Indies ("The Silk Waters") | C:1295, C:11892 |
| Bloc | The Silk Circle (Jade Port, Silk Haven, Dragon's Gate) | C:13624, C:13453 |
| Features | none | C:1295-1379 |
| Warehouse tiers | depot, regional | C:6562 |
| Port fee / provisions / repair / crew | 7 / 3 / 5 per hull point / 8 | C:1295-1379 |
| Map | x 42, y 8 | C:1379 |
| Lanes | 4 | section F |
| Player description | "Premier silk market of the eastern waters. Rum and iron are scarce luxuries here." | C:1297 |

## A. Specialties and trade goods

<!-- hand-maintained:begin -->
| Good | Category | Base | Affinity | Stock target | Restock | Role |
|---|---|---|---|---|---|---|
| Silk | luxury | 70 | 1.7 | 35 | 3.5 | exporter |
| Porcelain | luxury | 60 | 1.2 | 12 | 1.5 | exporter |
| Spice | luxury | 55 | 0.9 | 12 | 1.0 | importer |
| Pelts | commodity | 8 | 0.6 | 5 | 0.5 | importer |
| Cotton | commodity | 15 | 0.55 | 8 | 0.8 | importer |
| Grain | commodity | 12 | 0.55 | 6 | 0.5 | importer |
| Dyes | commodity | 22 | 0.5 | 5 | 0.5 | importer |
| Iron Ore | commodity | 25 | 0.5 | 5 | 0.5 | importer |
| Rum | provision | 20 | 0.45 | 8 | 0.8 | importer |
<!-- hand-maintained:end -->

Market cite: C:1295-1379. Price rule: base x (target / stock) / affinity (crates/portlight-sim/src/economy.rs:37); affinity above 1 is cheap here.
Not traded here: Timber, Tea, Pearls, Tobacco, Weapons, Medicines, Opium, Black Powder, Stolen Cargo.
Region goods [flavour, no mechanic]: sacred Porcelain, Silk; prized Spice, Tea; forbidden Weapons (C:11892 area).
Festival: Silk Weavers' Fair, every 75 days for 4, market effects Silk x2.0, Porcelain x1.5, Dyes x1.4 (C:11919).
Festival: Monsoon Prayer, every 90 days for 3, market effects Spice x1.8, Medicines x1.6, Tea x1.4 (C:11933).
Contract templates touching East Indies: `proc_grain_feed` (procurement), `proc_timber_shipyard` (procurement), `proc_iron_forge` (procurement), `proc_medicines_relief` (procurement), `lux_spice_monopolist` (luxury_discreet), `ret_cotton_return` (return_freight), `ret_spice_restock` (return_freight), `circ_indies_loop` (circuit), `smug_opium_run` (smuggling), `smug_powder_delivery` (smuggling). Source port Silk Haven: none.

## B. People

- Cultural group: The Silk Weavers' Guild: "Master artisans whose patterns are family secrets passed down for centuries." (C:12136 area).
- Local custom: "Silk is presented folded, never rolled. Rolling insults the weaver's art." (C:12136 area).
- Regional ethos: "Patience is mastery. The artisan who perfects one bolt of silk in a lifetime is honored above the merchant who trades a thousand. Everything here was old when your civilization was young." Greeting: "You honor us, Captain. Tea will be prepared while we discuss terms." Proverb: "A merchant who rushes drinks salt water." (C:11892 area).
- Tavern rumour (in-world, unreliable): "The eldest weaver has created a pattern that changes color in different light. Three merchants have bid their entire fortunes for a single bolt." (C:12136 area).

| Name | Title | Temperament | Institution | Cite |
|---|---|---|---|---|
| Harbor Keeper Jun | Harbor Keeper | graceful | harbor_master | C:9964 |
| Grand Weaver Seo-yeon | Guild Matriarch | proud | exchange | C:9986 |
| Lady Sato Hana | Circle Magistrate | subtle | governor | C:10008 |
| Master Ink | Calligrapher | eccentric | tavern | C:10030 |
| Merchant Feng | Silk Factor | commercial | shipyard | C:10052 |
| Inspector Yuki | Silk Inspector | meticulous | customs | C:10074 |
| Broker Chang | Silk Broker | refined | broker | C:10096 |
| Li Wei | Mechanist (merchant) | shrewd | outfitter | C:6031 |

- Dress [invented]: silk only for guild officers; weavers in fine cotton, sleeves bound back. The character hold stands: no likenesses or portraits.
- Speech [cite]: built from the greeting lines in each NPC record above (greeting_neutral, friendly, hostile).
- Temperament of the town: read from the power structure and tension in section E.
- Network (merchant, rival): Merchant Feng - Kiln Master Chen, linking Jade Port (C:11508). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Network (broker, professional): Broker Chang - Fernanda Reis, linking Porto Novo (C:11628). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Network (inspector, professional): Inspector Zhao Min - Inspector Yuki, linking Jade Port (C:11676). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Sea sighting filed under this port: Master Ink (C:12900).

## C. Visual identity (guidance only; commissions nothing; the character hold stands)

Pattern: the shared visual-identity headings 1-7 (design notes). Shared camera, envelope and do/don't blocks are in README.md and not repeated here.

1. Architecture. Identity line [invented]: The Loom Quarter is the town: Silk Haven measures time in weaving patterns a thousand years old. Landmark [cite]: "The Loom Quarter - where silk is woven in patterns a thousand years old." (C:12136 area). Cited buildings: The Arranged Harbor (harbor_master, C:9892); The Loom Quarter (exchange, C:9902); The Silk Court (governor, C:9912); The Brush and Bowl (tavern, C:9922); The Silk Warehouse (shipyard, C:9932); The Touch Room (customs, C:9942); The Story Room (broker, C:9952). Kit use [proposal]: compose from the shipped will-use plates; identity carriers are ground choice and building frequency, never new ids. Gaps follow the art gap list (design notes). Landmark canon: ../landmarks/loom_quarter.md.
2. Materials: local and abundant (exports): Silk, Porcelain; scarce or luxury (imports): Spice, Rum, Dyes, Iron Ore, Cotton, Grain, Pelts (section A). Atmosphere [cite]: "The rhythmic clack of a hundred looms, the rustle of silk, and green tea steam." (C:12136 area). Climate: section G.
3. Palette (references, not UI tokens; UI stays INK / GOLD / CREAM / MUTED / DUE). Judged in sRGB against the plate envelope (luma 65-141, chroma p98 35):

| Role | Family | sRGB | Luma / chroma | Status | Tie |
|---|---|---|---|---|---|
| Dominant | dark lacquered timber | (85, 69, 61) #55453D | 72 / 24 | in range | loom houses |
| Secondary 1 | raw silk, muted | (136, 128, 112) #888070 | 129 / 24 | in range | silk |
| Secondary 2 | grey roof tile | (98, 98, 100) #626264 | 98 / 2 | in range | quarter roofs |
| Accent | dyed silk thread, small | (138, 96, 90) #8A605A | 104 / 48 | small-area accent (chroma 48) | the dyes it imports |

4. Dress: see section B. 5. Signage: ASCII only, no UI-duplicating words; motifs [invented]: a folded bolt, a shuttle. 6. Silhouette: chart scale is the shared `chart_port_marker`; harbour-scale skyline [invented]: a dense quarter of low loom houses with long windows; a guild hall with a stepped roof. 7. Do and don't: README shared list.

## D. Historical analogies (feed [invented] prose; never a cite)

- **Lyon's silk guilds, as a guild structure.** Fits: a premier silk market run by a weavers' guild (Silk 1.7). Suggests: presentation rules (folded, never rolled), guild matriarchs, calligraphy as commerce. Limit: guild structure only; Silk Haven is eastern in data. Avoid: no French costume or place-names.
- **Byzantine imperial silk workshops, as a monopoly.** Fits: silk as a jealously guarded state art with inspectors (Inspector Yuki, Silk Inspector). Suggests: patterns as state secrets; inspection seals. Limit: no empire in data; the Circle stands in. Avoid: imperial pageantry.
- **Kyoto's Nishijin weaving district, as a district form.** Fits: a dense quarter of family looms. Suggests: quiet streets, the sound of looms, apprenticeships across generations. Limit: district form only. Avoid: geisha and samurai cliches.
- Contrast, not model: Jade Port's kilns: fire and clay against thread and patience. The weavers think porcelain loud.

## E. Governance

- Title: Circle Magistrate (C:9886).
- Power structure: "Silk Haven is governed by art. Seo-yeon's Guild produces the silk. Sato protects independence from Jade Port. Master Ink creates beauty that transcends commerce. Jun arranges the harbor as poetry. Chang sells meaning. Yuki guards authenticity. Feng moves volume and resents the pace of art." [ascii-folded] (C:9888).
- Internal tension: "Surface: Silk Haven vs. Jade Port - the monkey tapestry vs. 'decorated thread.' Sato encourages the rivalry because decentralization protects independence. Internal: Feng vs. the Guild on scaling production. Deep thread: Yuki and Zhao (Jade Port) trained under the same master - the inspectors are closer to each other than to their own ports. Master Ink painted a scroll for Scarlet Ana that hangs in Corsair's Rest - art connecting the pirate cove to the silk capital." (C:9889).
- Bloc stance: "Silk Haven believes art matters more than politics. They're wrong, but their silk is too good to argue with." (C:13624 area).
- Local rival: Jade Port: "Silk vs. porcelain - which is the supreme art? The weavers and the potters have argued for centuries. Both are convinced they're right." (C:13624 area).
- Grudge: "Jade Port once called silk 'decorated thread.' The Silk Weavers' Guild responded by weaving a tapestry depicting the Kiln Masters' ancestors as monkeys. It hangs in the Loom Quarter to this day." (C:13624 area).
- Bloc philosophy: "Quality over quantity. Hierarchy governs all." (C:13453 area). Loyalty and spillover numbers are [flavour, no mechanic] (DF-15).

## F. Relations with every other port

Hand-maintained from content.json. Disposition is bloc-to-bloc (symmetric in data); lanes are undirected; flows are directional by market affinity; networks are symmetric.

<!-- hand-maintained:begin -->
| Port | Region | Bloc | Disposition | Lane (distance, min class, danger) | Flows out (they import) | Flows in (we import) | Network |
|---|---|---|---|---|---|---|---|
| Porto Novo | Mediterranean | The Exchange Alliance | rival (C:13732) | - | Porcelain | Grain, Rum | C:11628 |
| Al-Manar | Mediterranean | The Exchange Alliance | rival (C:13732) | - | Porcelain, Silk | Spice | - |
| Silva Bay | Mediterranean | The Exchange Alliance | rival (C:13732) | - | Silk | Iron Ore, Pelts | - |
| Corsair's Rest | Mediterranean | The Shadow Ports | rival (C:13830) | - | Silk | Pelts, Rum | - |
| Ironhaven | North Atlantic | The Iron Pact | hostile (C:13767) | - | Silk | Iron Ore, Pelts | - |
| Stormwall | North Atlantic | The Iron Pact | hostile (C:13767) | - | Silk | Grain | - |
| Thornport | North Atlantic | The Free Ports | neutral (C:13823) | - | Porcelain, Silk | Pelts | - |
| Sun Harbor | West Africa | The Gold Coast Compact | neutral (C:13795) | - | Porcelain, Silk | Cotton, Dyes, Iron Ore, Pelts | - |
| Palm Cove | West Africa | The Gold Coast Compact | neutral (C:13795) | - | Silk | Cotton, Rum | - |
| Iron Point | West Africa | The Gold Coast Compact | neutral (C:13795) | - | Porcelain | Iron Ore | - |
| Pearl Shallows | West Africa | The Gold Coast Compact | neutral (C:13795) | - | Silk | Cotton, Dyes | - |
| Jade Port | East Indies | The Silk Circle | same bloc | 18, sloop, 0.07 (C:2171) | - | Spice | C:11508, C:11676 |
| Monsoon Reach | East Indies | The Free Ports | neutral (C:13823) | 22, sloop, 0.1 (C:2189) | Porcelain | Spice | - |
| Crosswind Isle | East Indies | The Free Ports | neutral (C:13823) | 32, brigantine, 0.12 (C:2144) | - | - | - |
| Dragon's Gate | East Indies | The Silk Circle | same bloc | - | - | - | - |
| Spice Narrows | East Indies | The Shadow Ports | rival (C:13830) | 20, brigantine, 0.11 (C:2207) | Silk | Spice | - |
| Ember Isle | South Seas | The Coral Crown | allied (C:13837) | - | - | Dyes, Iron Ore | - |
| Typhoon Anchorage | South Seas | The Coral Crown | allied (C:13837) | - | Porcelain, Silk | Spice | - |
| Coral Throne | South Seas | The Coral Crown | allied (C:13837) | - | Porcelain, Silk | Rum | - |
<!-- hand-maintained:end -->

- Lane lore: "The Silk Road by Sea": "Where porcelain meets silk. The two oldest trades in the East, connected by the shortest route." (C:2171).
- Asymmetry flag: Silk Haven names Jade Port as local rival (C:13624); Jade Port names none (C:13615). Canon reads local_rival_port as directional: a grievance, not a mutual feud. See DF-6.

## G. World details (undefined in data; all [invented] unless cited)

- History (shared timeline in README.md): T1: The first Loom Quarter patterns are set down; the guild still weaves them (landmark: 'a thousand years old'). T7: Silk Haven helps found the Silk Circle [invented]. T13: The tapestry insult against Jade Port's Kiln Masters (grudge text).
- Population [invented]: about 14,000; a third of households keep a loom.
- Language [invented]: Silk Waters trade speech; a written pattern-script used only by weavers.
- Currency: silver is the only money in the sim (every price, fee and wage is in silver). Local coin names, if any, are [invented] flavour and never replace 'silver' in copy.
- Religion [invented]: ancestor halls; a thread from each new bolt is burned for the founders.
- Climate [invented port detail]: humid, mild winters; still air in the Loom Quarter that the weavers prize. Regional weather [cite]: "Incense drifts from a passing junk. The air is thick with spice." (C:11892 area, weather_flavor).
- Pirates in these waters [cite]: The Monsoon Syndicate (C:2888).

## H. Extra depth (thin port)

- Daily life [invented]: looms start at first light; the quarter is silent at noon for the guild's inspection walk.
- Factions [invented]: the Weavers' Guild, the Circle Magistrate and the merchant factors. The guild outranks the factors on any question of quality.
- A local law [invented]: no bolt leaves without the inspector's seal; rolled silk is refused at the quay (custom extends this).

## I. Sound and ambience (all [invented] sound design unless quoted from a cite)

- **Soundscape:** "The rhythmic clack of a hundred looms, the rustle of silk, and green tea steam" (C:12139); sampans gliding.
- **Signature:** loom shuttle clack; silk bolt unfolded (custom: folded, never rolled); tea steam hiss.
- **Music/work songs (NOTE ONLY):** none; loom rhythm is the pulse.
- **Vs neighbours:** soft fabric + wood looms vs Jade Port's ceramic taps.
- **Fragments [proposal; no fragment or asset exists yet]:** `src.loom_shuttle`, `mat.silk_fold`, `src.sampan_scull`.
- Cited hooks: atmosphere "The rhythmic clack of a hundred looms, the rustle of silk, and green tea steam." (C:12139); dock scene "Sampans glide between junks with crimson sails. Merchants in silk robes examine bolts of fabric with magnifying glasses, judging thread count by touch." (C:12140).
- Invented bed idea: hundreds of looms clacking; the calligrapher's brush. No assets are requested.

## J. Flags for this port

- DF-4: Institutions of type `shipyard` at ports without the shipyard feature
- DF-6: Local rivals are one-sided
