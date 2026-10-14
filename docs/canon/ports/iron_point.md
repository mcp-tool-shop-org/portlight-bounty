# Iron Point (iron_point)

Base commit: 143e284 (content.json unchanged since 45b7378). Status: draft canon, phase 1. Written 2026-10-14.
Cites `C:n` are crates/portlight-sim/data/content.json line n at 143e284. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] (design-notes pattern) | DF-n see data-fixes.md | T-n see README shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Region | West Africa ("The Gold Coast") | C:961, C:11838 |
| Bloc | The Gold Coast Compact (Sun Harbor, Palm Cove, Iron Point, Pearl Shallows) | C:13597, C:13431 |
| Features | none | C:961-1037 |
| Warehouse tiers | depot, regional | C:6548 |
| Port fee / provisions / repair / crew | 4 / 3 / 3 per hull point / 4 | C:961-1037 |
| Map | x 18, y 24 | C:1037 |
| Lanes | 3 | section F |
| Player description | "Mining settlement at the river mouth. Iron flows out, everything else flows in at a premium." | C:963 |

## A. Specialties and trade goods

<!-- hand-maintained:begin -->
| Good | Category | Base | Affinity | Stock target | Restock | Role |
|---|---|---|---|---|---|---|
| Iron Ore | commodity | 25 | 1.7 | 45 | 4.0 | exporter |
| Pelts | commodity | 8 | 0.9 | 8 | 1.0 | importer |
| Timber | commodity | 18 | 0.8 | 15 | 1.5 | importer |
| Weapons | military | 45 | 0.6 | 10 | 1.0 | importer |
| Rum | provision | 20 | 0.6 | 8 | 0.8 | importer |
| Porcelain | luxury | 60 | 0.55 | 4 | 0.5 | importer |
| Grain | commodity | 12 | 0.5 | 12 | 1.0 | importer |
| Medicines | medicine | 35 | 0.5 | 5 | 0.5 | importer |
<!-- hand-maintained:end -->

Market cite: C:961-1037. Price rule: base x (target / stock) / affinity (crates/portlight-sim/src/economy.rs:37); affinity above 1 is cheap here.
Not traded here: Cotton, Dyes, Spice, Silk, Tea, Pearls, Tobacco, Opium, Black Powder, Stolen Cargo.
Region goods [flavour, no mechanic]: sacred Pearls; prized Dyes, Cotton, Rum; forbidden none (C:11838 area).
Festival: Dyer's Moon, every 60 days for 3, market effects Dyes x2.0, Cotton x1.6 (C:11863).
Festival: Pearl Tide, every 90 days for 4, market effects Pearls x2.2, Rum x1.3 (C:11876).
Contract templates touching West Africa: `proc_grain_feed` (procurement), `proc_medicines_relief` (procurement), `ret_cotton_return` (return_freight), `ret_spice_restock` (return_freight), `ret_dyes_textile` (return_freight), `circ_med_africa` (circuit), `circ_indies_loop` (circuit), `smug_powder_delivery` (smuggling). Source port Iron Point: none.

## B. People

- Cultural group: The Red Hand: "The mining collective. Their hands are permanently stained with iron oxide - a badge of honor." (C:12096 area).
- Local custom: "Miners spit on their hands before a deal. It means the ore is honest." (C:12096 area).
- Regional ethos: "The land gives freely to those who give back. Trade is a relationship, not a transaction. The elders remember every ship that entered the harbor." Greeting: "You return! The coast remembers you, Captain." Proverb: "The market remembers who gave, not who took." (C:11838 area).
- Tavern rumour (in-world, unreliable): "The miners hit something in the deep shaft last week. Not iron - something older. The foreman sealed the tunnel and doubled the guard." (C:12096 area).

| Name | Title | Temperament | Institution | Cite |
|---|---|---|---|---|
| Foreman Kofi Mensah | Mine Foreman | determined | governor | C:9092 |
| Dock Boss Adjei | Dock Supervisor | tough | harbor_master | C:9113 |
| Mama Adwoa | Mine Cook | tireless | tavern | C:9134 |
| Assayer Mensah | Ore Assayer | precise | exchange | C:9155 |
| Customs Officer Kwesi | Customs Officer | stoic | customs | C:9176 |
| Yaa Acheampong | Ore Broker | fierce | broker | C:9197 |

- Dress [invented]: heavy cotton work clothes stained red. The character hold stands: no likenesses or portraits.
- Speech [cite]: built from the greeting lines in each NPC record above (greeting_neutral, friendly, hostile).
- Temperament of the town: read from the power structure and tension in section E.
- Network (merchant, allied): Yaa Acheampong - Factor Wu Jian, linking Jade Port (C:11436). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Network (merchant, rival): Henrik Brandt - Foreman Kofi Mensah, linking Ironhaven (C:11448). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Sea sighting filed under this port: Yaa Acheampong (C:12870).

## C. Visual identity (guidance only; commissions nothing; the character hold stands)

Pattern: the shared visual-identity headings 1-7 (design notes). Shared camera, envelope and do/don't blocks are in README.md and not repeated here.

1. Architecture. Identity line [invented]: The mines are the town: Iron Point lives in the red cliffs and sells what it digs. Landmark [cite]: "The River Mouth Mines - tunnels carved into red cliffs where the river meets the sea." (C:12096 area). Cited buildings: The River Mouth Dock (harbor_master, C:9030); The Assay Office (exchange, C:9040); The Foreman's Post (governor, C:9050); The Mine Kitchen (tavern, C:9060); The Dock Desk (customs, C:9070); Yaa's Wall (broker, C:9080). Kit use [proposal]: compose from the shipped will-use plates; identity carriers are ground choice and building frequency, never new ids. Gaps follow the art gap list (design notes). Landmark canon: ../landmarks/river_mouth_mines.md.
2. Materials: local and abundant (exports): Iron Ore; scarce or luxury (imports): Grain, Timber, Porcelain, Weapons, Rum, Medicines, Pelts (section A). Atmosphere [cite]: "Red dust, the metallic tang of iron, and the echo of picks from the mine shafts." (C:12096 area). Climate: section G.
3. Palette (references, not UI tokens; UI stays INK / GOLD / CREAM / MUTED / DUE). Judged in sRGB against the plate envelope (luma 65-141, chroma p98 35):

| Role | Family | sRGB | Luma / chroma | Status | Tie |
|---|---|---|---|---|---|
| Dominant | red cliff earth, muted | (107, 84, 72) #6B5448 | 88 / 35 | in range | red cliffs |
| Secondary 1 | river silt | (104, 96, 82) #686052 | 97 / 22 | in range | the river mouth |
| Secondary 2 | ore grey | (96, 94, 92) #605E5C | 94 / 4 | in range | iron ore |
| Accent | rust | (138, 98, 75) #8A624B | 105 / 63 | small-area accent (chroma 63) | oxidised ore heaps |

4. Dress: see section B. 5. Signage: ASCII only, no UI-duplicating words; motifs [invented]: a red hand print. 6. Silhouette: chart scale is the shared `chart_port_marker`; harbour-scale skyline [invented]: tunnel mouths in red cliffs above an ore jetty. 7. Do and don't: README shared list.

## D. Historical analogies (feed [invented] prose; never a cite)

- **Rio Tinto / Cornish mine ports, as a mining function.** Fits: a mining settlement at a river mouth exporting raw ore (Iron Ore 1.7). Suggests: spit on the hand to seal a deal; the assayer's word is law. Limit: ore only, no smelting in data. Avoid: any colonial-extraction framing.

## E. Governance

- Title: Mine Foreman (C:9024).
- Power structure: "Iron Point is a company town - the mine IS the town. Foreman Kofi leads by working harder than anyone. Mensah's assays are the commercial foundation. Adjei runs the dock. Mama Adwoa feeds the machine. Yaa fights the pricing war against Ironhaven from her wall of contracts. There's no political complexity because there's only one institution that matters: the mine." [ascii-folded] (C:9026).
- Internal tension: "The tension is external, not internal: Iron Point vs. Ironhaven. Kofi dreams of a smelter that would let Iron Point sell refined iron directly. The Compact supports the idea in principle. The Iron Pact would consider it an act of war. Mensah's assay report - proving Iron Point ore outgrades Ironhaven's refined product - is a weapon waiting to be deployed. The question isn't whether Kofi will use it, but when, and whether the coast is ready for the consequences." (C:9027).
- Bloc stance: "Iron Point's miners vote with the Compact but dream of selling direct to the East." (C:13597 area).
- Local rival: Ironhaven: "Same ore, different prices. Iron Point sells raw; Ironhaven sells refined at triple the markup. The miners feel cheated by industrial middlemen." (C:13597 area).
- Grudge: "Ironhaven called Iron Point ore 'unrefined swamp metal' at a trade fair. The Red Hand has a long memory." (C:13597 area).
- Bloc philosophy: "Generosity first, then business. The coast remembers." (C:13431 area). Loyalty and spillover numbers are [flavour, no mechanic] (DF-15).

## F. Relations with every other port

Hand-maintained from content.json. Disposition is bloc-to-bloc (symmetric in data); lanes are undirected; flows are directional by market affinity; networks are symmetric.

<!-- hand-maintained:begin -->
| Port | Region | Bloc | Disposition | Lane (distance, min class, danger) | Flows out (they import) | Flows in (we import) | Network |
|---|---|---|---|---|---|---|---|
| Porto Novo | Mediterranean | The Exchange Alliance | neutral (C:13725) | - | - | Grain, Rum | - |
| Al-Manar | Mediterranean | The Exchange Alliance | neutral (C:13725) | - | Iron Ore | Medicines | - |
| Silva Bay | Mediterranean | The Exchange Alliance | neutral (C:13725) | - | - | Pelts, Timber | - |
| Corsair's Rest | Mediterranean | The Shadow Ports | neutral (C:13809) | - | - | Pelts, Rum, Weapons | - |
| Ironhaven | North Atlantic | The Iron Pact | rival (C:13760) | - | - | Pelts, Weapons | C:11448 |
| Stormwall | North Atlantic | The Iron Pact | rival (C:13760) | - | - | Grain, Medicines, Weapons | - |
| Thornport | North Atlantic | The Free Ports | allied (C:13802) | - | - | Pelts, Timber | - |
| Sun Harbor | West Africa | The Gold Coast Compact | same bloc | 18, sloop, 0.09 (C:2063) | - | Pelts | - |
| Palm Cove | West Africa | The Gold Coast Compact | same bloc | 22, sloop, 0.11 (C:2072) | Iron Ore | Rum | - |
| Pearl Shallows | West Africa | The Gold Coast Compact | same bloc | - | - | - | - |
| Jade Port | East Indies | The Silk Circle | neutral (C:13795) | - | Iron Ore | Porcelain | C:11436 |
| Monsoon Reach | East Indies | The Free Ports | allied (C:13802) | - | - | - | - |
| Silk Haven | East Indies | The Silk Circle | neutral (C:13795) | - | Iron Ore | Porcelain | - |
| Crosswind Isle | East Indies | The Free Ports | allied (C:13802) | 60, brigantine, 0.16 (C:2108) | - | - | - |
| Dragon's Gate | East Indies | The Silk Circle | neutral (C:13795) | - | Iron Ore | Porcelain | - |
| Spice Narrows | East Indies | The Shadow Ports | neutral (C:13809) | - | - | - | - |
| Ember Isle | South Seas | The Coral Crown | allied (C:13816) | - | - | Medicines | - |
| Typhoon Anchorage | South Seas | The Coral Crown | allied (C:13816) | - | - | Timber | - |
| Coral Throne | South Seas | The Coral Crown | allied (C:13816) | - | - | Rum | - |
<!-- hand-maintained:end -->


## G. World details (undefined in data; all [invented] unless cited)

- History (shared timeline in README.md): T10: Iron Point joins the Compact for protection of its mines [invented]. T15: Ironhaven calls its ore 'unrefined swamp metal' (grudge); the Compact blocks the Pact purchase (Sun Harbor grudge).
- Population [invented]: about 5,000 miners and families.
- Language [invented]: Gold Coast trade speech; miners' tally words.
- Currency: silver is the only money in the sim (every price, fee and wage is in silver). Local coin names, if any, are [invented] flavour and never replace 'silver' in copy.
- Religion [invented]: ancestor rites and a cliff shrine for miners.
- Climate [invented port detail]: hot, humid river mouth; red dust in the dry season. Regional weather [cite]: "Warm air, heavy with moisture. The coast shimmers in the heat." (C:11838 area, weather_flavor).
- Pirates in these waters [cite]: The Deep Reef Brotherhood (C:2896).

## I. Sound and ambience (all [invented] sound design unless quoted from a cite)

- **Soundscape:** river-mouth mining town; "the echo of picks from the mine shafts" (C:12099); ore carts rattling down tracks.
- **Signature:** ore cart on rails; ore tipped into a hold; spit-and-clasp hands (custom) - very close, optional.
- **Music/work songs (NOTE ONLY):** none in canon.
- **Vs neighbours:** the only percussive-industrial West Africa port; vs Ironhaven, it is open-air picks and rails, not furnaces.
- **Fragments [proposal; no fragment or asset exists yet]:** `src.ore_cart_rail`, `src.pick_far`, `place.river_mouth`.
- Cited hooks: atmosphere "Red dust, the metallic tang of iron, and the echo of picks from the mine shafts."; dock scene "Ore carts rattle down tracks to the loading docks. The river runs red with mine runoff. Everything here - the buildings, the clothes, the skin - is stained rust." (C:12096 area).
- Invented bed idea: pick work echoing in tunnels, ore carts on rails. No assets are requested.

## J. Flags for this port

- DF-10: 'Gold Coast' names both a region and a bloc
