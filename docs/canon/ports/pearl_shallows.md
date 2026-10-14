# Pearl Shallows (pearl_shallows)

Base commit: 143e284 (content.json unchanged since 45b7378). Status: draft canon, phase 1. Seat: Game Designer, 2026-10-14.
Cites `C:n` are crates/portlight-sim/data/content.json line n at 143e284. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] (AD or Analogist pattern) | DF-n see canon-data-fixes.md | T-n see README shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Region | West Africa ("The Gold Coast") | C:1040, C:11838 |
| Bloc | The Gold Coast Compact (Sun Harbor, Palm Cove, Iron Point, Pearl Shallows) | C:13606, C:13431 |
| Features | none | C:1040-1116 |
| Warehouse tiers | none listed | C:6520 |
| Port fee / provisions / repair / crew | 4 / 2 / 4 per hull point / 4 | C:1040-1116 |
| Map | x 12, y 30 | C:1116 |
| Lanes | 4 | section F |
| Player description | "Divers bring up pearls from the warm shallows. A quiet port where fortunes are made by the patient." | C:1042 |

## A. Specialties and trade goods

<!-- generated:begin -->
| Good | Category | Base | Affinity | Stock target | Restock | Role |
|---|---|---|---|---|---|---|
| Pearls | luxury | 95 | 1.7 | 12 | 1.5 | exporter |
| Dyes | commodity | 22 | 1.4 | 12 | 1.5 | exporter |
| Cotton | commodity | 15 | 1.2 | 18 | 2.0 | exporter |
| Pelts | commodity | 8 | 1.0 | 10 | 1.0 | neutral |
| Grain | commodity | 12 | 0.7 | 12 | 1.0 | importer |
| Rum | provision | 20 | 0.7 | 8 | 0.8 | importer |
| Silk | luxury | 70 | 0.5 | 5 | 0.5 | importer |
| Medicines | medicine | 35 | 0.45 | 5 | 0.5 | importer |
<!-- generated:end -->

Market cite: C:1040-1116. Price rule: base x (target / stock) / affinity (crates/portlight-sim/src/economy.rs:37); affinity above 1 is cheap here.
Not traded here: Timber, Iron Ore, Spice, Porcelain, Tea, Tobacco, Weapons, Opium, Black Powder, Stolen Cargo.
Region goods [flavour, no mechanic]: sacred Pearls; prized Dyes, Cotton, Rum; forbidden none (C:11838 area).
Festival: Dyer's Moon, every 60 days for 3, market effects Dyes x2.0, Cotton x1.6 (C:11863).
Festival: Pearl Tide, every 90 days for 4, market effects Pearls x2.2, Rum x1.3 (C:11876).
Contract templates touching West Africa: `proc_grain_feed` (procurement), `proc_medicines_relief` (procurement), `ret_cotton_return` (return_freight), `ret_spice_restock` (return_freight), `ret_dyes_textile` (return_freight), `circ_med_africa` (circuit), `circ_indies_loop` (circuit), `smug_powder_delivery` (smuggling). Source port Pearl Shallows: none.

## B. People

- Cultural group: The Breath-Holders: "Pearl divers who can hold their breath for impossible minutes. Their craft is sacred." (C:12106 area).
- Local custom: "Pearls are never haggled over. The diver names the price. You pay or you leave." (C:12106 area).
- Regional ethos: "The land gives freely to those who give back. Trade is a relationship, not a transaction. The elders remember every ship that entered the harbor." Greeting: "You return! The coast remembers you, Captain." Proverb: "The market remembers who gave, not who took." (C:11838 area).
- Tavern rumour (in-world, unreliable): "The oldest diver says there's a pearl the size of a fist somewhere in the deep reef. Three divers have gone looking. Two came back." (C:12106 area).

| Name | Title | Temperament | Institution | Cite |
|---|---|---|---|---|
| Elder Ama Diallo | Dive Elder | serene | governor | C:9288 |
| Harbor Keeper Nana | Harbor Keeper | quiet | harbor_master | C:9309 |
| Grader Kweku | Pearl Grader | reverent | exchange | C:9330 |
| Dye Master Yaw | Dye Master | artistic | shipyard | C:9351 |
| Healer Abena | Dive Healer | caring | tavern | C:9372 |
| Esi Kumi | Pearl Broker | protective | broker | C:9393 |
| Yaa | Blade Merchant (merchant) | shrewd | outfitter | C:5989 |
| Nneka Ironfist | style master | - | - | C:6213 |

- Dress [invented]: light wraps; divers in close-fitting cotton. The character hold stands: no likenesses or portraits.
- Speech [cite]: built from the greeting lines in each NPC record above (greeting_neutral, friendly, hostile).
- Temperament of the town: read from the power structure and tension in section E.
- Sea sighting filed under this port: Elder Ama Diallo (C:12876).

## C. Visual identity (guidance only; commissions nothing; the character hold stands)

Pattern: canon-port-silva_bay-visual.md headings 1-7. Shared camera, envelope and do/don't blocks are in README.md and not repeated here.

1. Architecture. Identity line [invented]: The Blessing Pool is the town's heart: Pearl Shallows waits on its divers. Landmark [cite]: "The Blessing Pool - a tidal pool where divers pray before descending." (C:12106 area). Cited buildings: The Reef Approach (harbor_master, C:9226); The Grading Stone (exchange, C:9236); The Blessing Pool (governor, C:9246); The Color Workshop (shipyard, C:9256); The Pool's Edge (tavern, C:9266); The Pearl Mat (broker, C:9276). Kit use [proposal]: compose from the shipped will-use plates; identity carriers are ground choice and building frequency, never new ids. Gaps follow the PB-001 gap list.
2. Materials: local and abundant (exports): Pearls, Cotton, Dyes; scarce or luxury (imports): Medicines, Grain, Silk, Rum (section A). Atmosphere [cite]: "Warm shallow water, coral sand, and the silence of divers preparing their breath." (C:12106 area). Climate: section G.
3. Palette (references, not UI tokens; UI stays INK / GOLD / CREAM / MUTED / DUE). Judged in sRGB against the plate envelope (luma 65-141, chroma p98 35):

| Role | Family | sRGB | Luma / chroma | Status | Tie |
|---|---|---|---|---|---|
| Dominant | sand and shell | (136, 128, 110) #88806E | 128 / 26 | in range | the warm shallows |
| Secondary 1 | shallow water stone | (98, 112, 108) #62706C | 109 / 14 | in range | tidal pool |
| Secondary 2 | drying-rack wood | (104, 90, 72) #685A48 | 92 / 32 | in range | dye and pearl sheds |
| Accent | pearl sheen | (150, 146, 138) #96928A | 146 / 12 | mood, not values | pearls |

4. Dress: see section B. 5. Signage: ASCII only, no UI-duplicating words; motifs [invented]: a pearl in an open hand. 6. Silhouette: chart scale is the shared `chart_port_marker`; harbour-scale skyline [invented]: low huts on stilts over shallows, the Blessing Pool ringed with stones. 7. Do and don't: README shared list.

## D. Historical analogies (feed [invented] prose; never a cite)

- **Gulf pearling towns, as a diving economy.** Fits: a pearl port where divers set the price (Pearls 1.7; 'the diver names the price'). Suggests: patience, no haggling, prayer before diving. Limit: diving only. Avoid: ethnic coding.

## E. Governance

- Title: Dive Elder (C:9220).
- Power structure: "Pearl Shallows is governed by Elder Ama from the Blessing Pool - spiritual authority merged with political governance. Kweku grades the pearls. Nana protects the reef approaches. Yaw produces irreplaceable dyes. Abena heals the divers. Esi sells to the world. The port runs on reverence: the reef provides, the divers receive, the community protects." [ascii-folded] (C:9222).
- Internal tension: "Abena wants to limit dive depths to protect the divers. Ama wants to maintain the tradition of deep dives that produce the finest pearls. Both care about the divers - they disagree on what 'care' means. The external wound is Typhoon Anchorage: the Storm Riders poached the reef three seasons ago, and Abena's breathing technique leaked to them through a visiting diver. Ama considers this a betrayal. Abena says breath belongs to everyone. This philosophical disagreement - hoarding sacred knowledge vs. sharing it freely - is Pearl Shallows' deepest fault line." (C:9223).
- Bloc stance: "Pearl Shallows guards its diving traditions fiercely. They share nothing with outsiders." (C:13606 area).
- Local rival: Typhoon Anchorage: "Both export pearls. The Breath-Holders consider Typhoon Anchorage divers reckless and their pearls inferior. The feeling is mutual." (C:13606 area).
- Grudge: "Typhoon Anchorage divers poached Pearl Shallows' reef three seasons ago. The Breath-Holders haven't spoken to the Storm Riders since." (C:13606 area).
- Bloc philosophy: "Generosity first, then business. The coast remembers." (C:13431 area). Loyalty and spillover numbers are [flavour, no mechanic] (DF-15).

## F. Relations with every other port

Generated. Disposition is bloc-to-bloc (symmetric in data); lanes are undirected; flows are directional by market affinity; networks are symmetric.

<!-- generated:begin -->
| Port | Region | Bloc | Disposition | Lane (distance, min class, danger) | Flows out (they import) | Flows in (we import) | Network |
|---|---|---|---|---|---|---|---|
| Porto Novo | Mediterranean | The Exchange Alliance | neutral (C:13725) | - | Cotton, Dyes | Grain, Rum | - |
| Al-Manar | Mediterranean | The Exchange Alliance | neutral (C:13725) | - | - | Medicines | - |
| Silva Bay | Mediterranean | The Exchange Alliance | neutral (C:13725) | - | Cotton | - | - |
| Corsair's Rest | Mediterranean | The Shadow Ports | neutral (C:13809) | - | - | Rum | - |
| Ironhaven | North Atlantic | The Iron Pact | rival (C:13760) | - | Cotton | - | - |
| Stormwall | North Atlantic | The Iron Pact | rival (C:13760) | - | - | Grain, Medicines | - |
| Thornport | North Atlantic | The Free Ports | allied (C:13802) | - | - | - | - |
| Sun Harbor | West Africa | The Gold Coast Compact | same bloc | 24, sloop, 0.1 (C:2081) | - | - | - |
| Palm Cove | West Africa | The Gold Coast Compact | same bloc | 20, sloop, 0.09 (C:2090) | - | Rum | - |
| Iron Point | West Africa | The Gold Coast Compact | same bloc | - | - | - | - |
| Jade Port | East Indies | The Silk Circle | neutral (C:13795) | - | Cotton | Silk | - |
| Monsoon Reach | East Indies | The Free Ports | allied (C:13802) | - | Cotton | Silk | - |
| Silk Haven | East Indies | The Silk Circle | neutral (C:13795) | - | Cotton, Dyes | Silk | - |
| Crosswind Isle | East Indies | The Free Ports | allied (C:13802) | 56, brigantine, 0.15 (C:2117) | - | - | - |
| Dragon's Gate | East Indies | The Silk Circle | neutral (C:13795) | - | Cotton | - | - |
| Spice Narrows | East Indies | The Shadow Ports | neutral (C:13809) | - | - | - | - |
| Ember Isle | South Seas | The Coral Crown | allied (C:13816) | - | Cotton | Medicines | - |
| Typhoon Anchorage | South Seas | The Coral Crown | allied (C:13816) | - | - | - | - |
| Coral Throne | South Seas | The Coral Crown | allied (C:13816) | 68, galleon, 0.2 (C:2306) | - | Rum | - |
<!-- generated:end -->

- Lane lore: "The Deep South Run": "Pearl to pearl. The divers of the shallows and the divers of the reef - same craft, different kingdoms." (C:2306).

## G. World details (undefined in data; all [invented] unless cited)

- History (shared timeline in README.md): T10: The Breath-Holders join the Compact [invented]. T16: Typhoon Anchorage divers poach the reef three seasons ago (grudge text).
- Population [invented]: about 2,000.
- Language [invented]: Gold Coast trade speech; divers' breath-count words.
- Currency: silver is the only money in the sim (every price, fee and wage is in silver). Local coin names, if any, are [invented] flavour and never replace 'silver' in copy.
- Religion [invented]: prayer at the Blessing Pool before every descent (landmark).
- Climate [invented port detail]: warm, still shallows; calm seas most of the year. Regional weather [cite]: "Warm air, heavy with moisture. The coast shimmers in the heat." (C:11838 area, weather_flavor).
- Pirates in these waters [cite]: The Deep Reef Brotherhood (C:2896).

## I. Sound and ambience (placeholder for the Sound seat)

- Status: placeholder. No per-port sound exists (inventory 'Undefined'; studio SOUND_BIBLE covers the Mediterranean slice generically; music is on hold).
- Cited hooks: atmosphere "Warm shallow water, coral sand, and the silence of divers preparing their breath."; dock scene "Canoes glide in from the reef. Divers emerge glistening, pouches of pearls tied to their wrists. An elder sits under a baobab tree, examining each pearl by sunlight." (C:12106 area).
- Invented bed idea: water lapping, quiet prayer, divers' breath counts. No assets are requested.

## J. Flags for this port

- DF-4: Institutions of type `shipyard` at ports without the shipyard feature
- DF-10: 'Gold Coast' names both a region and a bloc
