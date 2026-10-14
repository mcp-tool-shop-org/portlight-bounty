# Palm Cove (palm_cove)

Base commit: 143e284 (content.json unchanged since 45b7378). Status: draft canon, phase 1. Written 2026-10-14.
Cites `C:n` are crates/portlight-sim/data/content.json line n at 143e284. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] (design-notes pattern) | DF-n see data-fixes.md | T-n see README shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Region | West Africa ("The Gold Coast") | C:882, C:11838 |
| Bloc | The Gold Coast Compact (Sun Harbor, Palm Cove, Iron Point, Pearl Shallows) | C:13588, C:13431 |
| Features | none | C:882-958 |
| Warehouse tiers | depot | C:6545 |
| Port fee / provisions / repair / crew | 3 / 1 / 5 per hull point / 3 | C:882-958 |
| Map | x 10, y 26 | C:958 |
| Lanes | 4 | section F |
| Player description | "A sheltered cove where rum barrels outnumber the inhabitants. Cheapest provisions on the coast." | C:884 |
| Captain start here | The Smuggler | C:2355 |

## A. Specialties and trade goods

<!-- hand-maintained:begin -->
| Good | Category | Base | Affinity | Stock target | Restock | Role |
|---|---|---|---|---|---|---|
| Rum | provision | 20 | 1.7 | 35 | 3.0 | exporter |
| Tobacco | provision | 28 | 1.3 | 15 | 2.0 | exporter |
| Cotton | commodity | 15 | 1.1 | 18 | 1.5 | exporter |
| Pelts | commodity | 8 | 0.9 | 10 | 1.0 | importer |
| Grain | commodity | 12 | 0.7 | 15 | 1.5 | importer |
| Timber | commodity | 18 | 0.6 | 10 | 1.0 | importer |
| Iron Ore | commodity | 25 | 0.6 | 8 | 0.8 | importer |
| Silk | luxury | 70 | 0.5 | 5 | 0.5 | importer |
<!-- hand-maintained:end -->

Market cite: C:882-958. Price rule: base x (target / stock) / affinity (crates/portlight-sim/src/economy.rs:37); affinity above 1 is cheap here.
Not traded here: Dyes, Spice, Porcelain, Tea, Pearls, Weapons, Medicines, Opium, Black Powder, Stolen Cargo.
Region goods [flavour, no mechanic]: sacred Pearls; prized Dyes, Cotton, Rum; forbidden none (C:11838 area).
Festival: Dyer's Moon, every 60 days for 3, market effects Dyes x2.0, Cotton x1.6 (C:11863).
Festival: Pearl Tide, every 90 days for 4, market effects Pearls x2.2, Rum x1.3 (C:11876).
Contract templates touching West Africa: `proc_grain_feed` (procurement), `proc_medicines_relief` (procurement), `ret_cotton_return` (return_freight), `ret_spice_restock` (return_freight), `ret_dyes_textile` (return_freight), `circ_med_africa` (circuit), `circ_indies_loop` (circuit), `smug_powder_delivery` (smuggling). Source port Palm Cove: none.

## B. People

- Cultural group: The Seven Houses: "Seven distillery families who have been rivals for a century and allies for longer." (C:12086 area).
- Local custom: "A cup of rum for every stranger. Refusing is an insult that lasts generations." (C:12086 area).
- Regional ethos: "The land gives freely to those who give back. Trade is a relationship, not a transaction. The elders remember every ship that entered the harbor." Greeting: "You return! The coast remembers you, Captain." Proverb: "The market remembers who gave, not who took." (C:11838 area).
- Tavern rumour (in-world, unreliable): "Old Cassius at the third distillery claims his rum can cure fever, heal wounds, and make you invisible to customs inspectors. Two of those are probably true." (C:12086 area).

| Name | Title | Temperament | Institution | Cite |
|---|---|---|---|---|
| Captain Abel | Harbor Chief | easygoing | harbor_master | C:8868 |
| Old Cassius | Master Distiller | boastful | exchange | C:8890 |
| Elder Kwame | Council Elder | patient | governor | C:8912 |
| Mama Joy | Tavern Keeper | radiant | tavern | C:8934 |
| Esi Owusu | Dock Master | capable | shipyard | C:8956 |
| Customs Officer Abu | Customs Officer | relaxed | customs | C:8978 |
| Chioma Obi | Broker | savvy | broker | C:9000 |
| Kofi | Leatherworker (merchant) | quiet | outfitter | C:5976 |

- Dress [invented]: light cotton, house colours on aprons in the distilleries. The character hold stands: no likenesses or portraits.
- Speech [cite]: built from the greeting lines in each NPC record above (greeting_neutral, friendly, hostile).
- Temperament of the town: read from the power structure and tension in section E.
- Network (merchant, allied): Old Cassius - Quartermaster Maren, linking Stormwall (C:11484). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Network (tavern, allied): Mama Joy - Yusuf Diallo, linking Sun Harbor (C:11544). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Sea sighting filed under this port: Old Cassius (C:12864).

## C. Visual identity (guidance only; commissions nothing; the character hold stands)

Pattern: the shared visual-identity headings 1-7 (design notes). Shared camera, envelope and do/don't blocks are in README.md and not repeated here.

1. Architecture. Identity line [invented]: Distillery Row is the town: seven rum houses and the cove that serves them. Landmark [cite]: "The Distillery Row - seven rum houses, each with its own secret recipe and fierce pride." (C:12086 area). Cited buildings: The Hammock Dock (harbor_master, C:8796); The Third Distillery (exchange, C:8806); The Council Shade (governor, C:8816); The Rum House (tavern, C:8826); The Working Dock (shipyard, C:8836); The Stamp Table (customs, C:8846); Chioma's Table (broker, C:8856). Kit use [proposal]: compose from the shipped will-use plates; identity carriers are ground choice and building frequency, never new ids. Gaps follow the PB-001 gap list.
2. Materials: local and abundant (exports): Rum, Cotton, Tobacco; scarce or luxury (imports): Grain, Timber, Pelts, Silk, Iron Ore (section A). Atmosphere [cite]: "Sweet molasses, fermenting sugar cane, and the lazy hum of bees." (C:12086 area). Climate: section G.
3. Palette (references, not UI tokens; UI stays INK / GOLD / CREAM / MUTED / DUE). Judged in sRGB against the plate envelope (luma 65-141, chroma p98 35):

| Role | Family | sRGB | Luma / chroma | Status | Tie |
|---|---|---|---|---|---|
| Dominant | palm-thatch brown | (110, 96, 74) #6E604A | 97 / 36 | mood, not values | thatched rum houses |
| Secondary 1 | palm green, muted | (84, 98, 72) #546248 | 93 / 26 | in range | sheltered cove palms |
| Secondary 2 | cask oak | (96, 78, 60) #604E3C | 81 / 36 | mood, not values | barrels outnumber the inhabitants |
| Accent | amber rum | (150, 110, 62) #966E3E | 115 / 88 | mood, not values | rum, its top export |

4. Dress: see section B. 5. Signage: ASCII only, no UI-duplicating words; motifs [invented]: seven marks for the seven houses, a cask. 6. Silhouette: chart scale is the shared `chart_port_marker`; harbour-scale skyline [invented]: seven distillery chimneys in a row along a crescent beach. 7. Do and don't: README shared list.

## D. Historical analogies (feed [invented] prose; never a cite)

- **Caribbean distilling coves, as an industry.** Fits: a sheltered cove of rum houses selling cheap provisions (Rum 1.7, cheapest provisions). Suggests: hospitality as law; house pride between the seven distilleries. Limit: the analogy covers distilling only. Avoid: plantation imagery of any kind.

## E. Governance

- Title: Council Elder (C:8790).
- Power structure: "Palm Cove is governed by Elder Kwame's patience, Mama Joy's warmth, and the Seven Houses' competitive pride. Captain Abel is technically in charge of the harbor but Esi actually runs it. Cassius sets rum prices by tasting. It's the most informal power structure in the Known World, and it works because everyone in Palm Cove has the same agenda: rum, happiness, and being left alone by people with clipboards." [ascii-folded] (C:8792).
- Internal tension: "Palm Cove has no internal tension - and that IS the tension. Every other port in the Known World has competing factions, power struggles, and political ambitions. Palm Cove's lack of ambition is both its charm and its vulnerability. Chioma sees this: she's building export contracts to make the Cove economically necessary, because 'charming' doesn't survive a trade war. The deeper worry: the Seven Houses have been balanced for a century, but rum trends change. If one house finds a better recipe or a bigger buyer, the balance shifts. Cassius is getting old. His secret recipe - whatever it actually is - will pass to someone. Who inherits it will determine whether Palm Cove's century of peace continues." (C:8793).
- Bloc stance: "Palm Cove doesn't do politics. They make rum and share it. That's their entire foreign policy." (C:13588 area).
- Grudge: "None. Palm Cove is incapable of holding a grudge. It's a cultural impossibility." (C:13588 area).
- Bloc philosophy: "Generosity first, then business. The coast remembers." (C:13431 area). Loyalty and spillover numbers are [flavour, no mechanic] (DF-15).

## F. Relations with every other port

Hand-maintained from content.json. Disposition is bloc-to-bloc (symmetric in data); lanes are undirected; flows are directional by market affinity; networks are symmetric.

<!-- hand-maintained:begin -->
| Port | Region | Bloc | Disposition | Lane (distance, min class, danger) | Flows out (they import) | Flows in (we import) | Network |
|---|---|---|---|---|---|---|---|
| Porto Novo | Mediterranean | The Exchange Alliance | neutral (C:13725) | - | Cotton | Grain | - |
| Al-Manar | Mediterranean | The Exchange Alliance | neutral (C:13725) | - | Rum | - | - |
| Silva Bay | Mediterranean | The Exchange Alliance | neutral (C:13725) | 44, brigantine, 0.13 (C:2045) | Cotton, Rum | Iron Ore, Pelts, Timber | - |
| Corsair's Rest | Mediterranean | The Shadow Ports | neutral (C:13809) | - | - | Pelts | - |
| Ironhaven | North Atlantic | The Iron Pact | rival (C:13760) | - | Cotton, Rum, Tobacco | Iron Ore, Pelts | - |
| Stormwall | North Atlantic | The Iron Pact | rival (C:13760) | - | Rum, Tobacco | Grain | C:11484 |
| Thornport | North Atlantic | The Free Ports | allied (C:13802) | - | - | Pelts, Timber | - |
| Sun Harbor | West Africa | The Gold Coast Compact | same bloc | 20, sloop, 0.1 (C:2054) | Rum | Iron Ore, Pelts | C:11544 |
| Iron Point | West Africa | The Gold Coast Compact | same bloc | 22, sloop, 0.11 (C:2072) | Rum | Iron Ore | - |
| Pearl Shallows | West Africa | The Gold Coast Compact | same bloc | 20, sloop, 0.09 (C:2090) | Rum | - | - |
| Jade Port | East Indies | The Silk Circle | neutral (C:13795) | - | Cotton, Rum | Silk | - |
| Monsoon Reach | East Indies | The Free Ports | allied (C:13802) | - | Cotton, Rum | Silk | - |
| Silk Haven | East Indies | The Silk Circle | neutral (C:13795) | - | Cotton, Rum | Silk | - |
| Crosswind Isle | East Indies | The Free Ports | allied (C:13802) | - | - | - | - |
| Dragon's Gate | East Indies | The Silk Circle | neutral (C:13795) | - | Cotton, Rum, Tobacco | - | - |
| Spice Narrows | East Indies | The Shadow Ports | neutral (C:13809) | - | Rum | - | - |
| Ember Isle | South Seas | The Coral Crown | allied (C:13816) | - | Cotton, Rum | Iron Ore | - |
| Typhoon Anchorage | South Seas | The Coral Crown | allied (C:13816) | - | Rum | Timber | - |
| Coral Throne | South Seas | The Coral Crown | allied (C:13816) | - | - | - | - |
<!-- hand-maintained:end -->


## G. World details (undefined in data; all [invented] unless cited)

- History (shared timeline in README.md): T10: The Seven Houses swear into the Gold Coast Compact [invented]. T13: Smugglers learn Palm Cove asks no questions; the smuggler archetype starts here (C captains).
- Population [invented]: about 4,000.
- Language [invented]: Gold Coast trade speech.
- Currency: silver is the only money in the sim (every price, fee and wage is in silver). Local coin names, if any, are [invented] flavour and never replace 'silver' in copy.
- Religion [invented]: ancestor rites; a first pour of rum to the sea.
- Climate [invented port detail]: sheltered and humid; palms break the wind. Regional weather [cite]: "Warm air, heavy with moisture. The coast shimmers in the heat." (C:11838 area, weather_flavor).
- Pirates in these waters [cite]: The Deep Reef Brotherhood (C:2896).

## I. Sound and ambience (all [invented] sound design unless quoted from a cite)

- **Soundscape:** sheltered, lazy; "the lazy hum of bees"; small boats bobbing; children diving for coins.
- **Signature:** rum barrel rolling down a wooden ramp; coin plop into shallow water; cup of rum poured (custom).
- **Music/work songs (NOTE ONLY):** none in canon.
- **Vs neighbours:** the softest, warmest bed on the coast; no singing (Sun Harbor), no silence ritual (Pearl Shallows).
- **Fragments [proposal; no fragment or asset exists yet]:** `src.barrel_ramp`, `src.coin_water_plop`, `place.sheltered_cove`.
- Cited hooks: atmosphere "Sweet molasses, fermenting sugar cane, and the lazy hum of bees."; dock scene "Small boats bob in the sheltered cove. Rum barrels roll down wooden ramps from the hilltop distilleries. Children dive for coins tossed by arriving crews." (C:12086 area).
- Invented bed idea: cask hammering, laughter, a cup offered to every stranger. No assets are requested.

## J. Flags for this port

- DF-4: Institutions of type `shipyard` at ports without the shipyard feature
- DF-10: 'Gold Coast' names both a region and a bloc
