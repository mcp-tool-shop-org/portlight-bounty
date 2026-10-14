# Al-Manar (al_manar)

Base commit: 143e284 (content.json unchanged since 45b7378). Status: draft canon, phase 1. Written 2026-10-14.
Cites `C:n` are crates/portlight-sim/data/content.json line n at 143e284. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] (design-notes pattern) | DF-n see data-fixes.md | T-n see README shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Region | Mediterranean ("The Middle Sea") | C:291, C:11728 |
| Bloc | The Exchange Alliance (Porto Novo, Al-Manar, Silva Bay) | C:13541, C:13388 |
| Features | none | C:291-375 |
| Warehouse tiers | depot, regional, commercial | C:6526 |
| Port fee / provisions / repair / crew | 8 / 3 / 4 per hull point / 6 | C:291-375 |
| Map | x 24, y 6 | C:375 |
| Lanes | 4 | section F |
| Player description | "Ancient port famed for its spice markets. Merchants bid fiercely for grain and iron." | C:293 |
| Captain start here | The Merchant Prince | C:2549 |

## A. Specialties and trade goods

<!-- hand-maintained:begin -->
| Good | Category | Base | Affinity | Stock target | Restock | Role |
|---|---|---|---|---|---|---|
| Spice | luxury | 55 | 1.5 | 25 | 2.5 | exporter |
| Medicines | medicine | 35 | 1.1 | 12 | 1.0 | exporter |
| Silk | luxury | 70 | 0.9 | 10 | 1.0 | importer |
| Rum | provision | 20 | 0.7 | 15 | 1.0 | importer |
| Pelts | commodity | 8 | 0.7 | 8 | 0.8 | importer |
| Tea | luxury | 40 | 0.6 | 8 | 0.8 | importer |
| Porcelain | luxury | 60 | 0.55 | 8 | 0.8 | importer |
| Iron Ore | commodity | 25 | 0.55 | 8 | 0.8 | importer |
| Grain | commodity | 12 | 0.5 | 15 | 1.5 | importer |
<!-- hand-maintained:end -->

Market cite: C:291-375. Price rule: base x (target / stock) / affinity (crates/portlight-sim/src/economy.rs:37); affinity above 1 is cheap here.
Not traded here: Timber, Cotton, Dyes, Pearls, Tobacco, Weapons, Opium, Black Powder, Stolen Cargo.
Region goods [flavour, no mechanic]: sacred Grain; prized Spice, Silk; forbidden none (C:11728 area).
Festival: Harvest of Plenty, every 60 days for 3, market effects Grain x1.8, Rum x1.4 (C:11752).
Festival: Night of Lanterns, every 90 days for 3, market effects Spice x2.0, Silk x1.5, Dyes x1.6 (C:11765).
Contract templates touching Mediterranean: `proc_iron_forge` (procurement), `short_tea_crisis` (shortage), `ret_cotton_return` (return_freight), `ret_spice_restock` (return_freight), `ret_dyes_textile` (return_freight), `circ_med_africa` (circuit), `circ_indies_loop` (circuit), `circ_south_seas` (circuit), `rep_charter_atlantic` (reputation_charter), `smug_opium_run` (smuggling), `smug_powder_delivery` (smuggling). Source port Al-Manar: none.

## B. People

- Cultural group: The Spice Merchants' Circle: "An ancient guild that controls the spice auctions and guards trade secrets." (C:12016 area).
- Local custom: "Bargaining begins with tea. Refusing the first cup insults the seller." (C:12016 area).
- Regional ethos: "Civilization is commerce. The law of contract binds harder than iron, and a merchant's word, once given, is recorded in stone." Greeting: "Welcome to port, Captain. Your cargo manifest, if you please." Proverb: "A contract honored is worth more than gold hoarded." (C:11728 area).
- Tavern rumour (in-world, unreliable): "A merchant from the east claims to know a spice island no chart has ever shown. He's been buying a lot of rum and saying little else." (C:12016 area).

| Name | Title | Temperament | Institution | Cite |
|---|---|---|---|---|
| Khalil al-Rashid | Harbor Master | ceremonial | harbor_master | C:7258 |
| Yasmin al-Nadir | The Spice Mother | imperious | exchange | C:7280 |
| Senhora Nadia Khoury | Merchant Princess | regal | governor | C:7302 |
| Old Farouk | Tea Master | philosophical | tavern | C:7324 |
| Inspector Zara Osman | Chief Customs Inspector | precise | customs | C:7346 |
| Hakim al-Nadir | Apothecary Master | gentle | apothecary | C:7368 |
| Tariq Sayed | Senior Broker | ambitious | broker | C:7390 |
| Fatima | Armorer (merchant) | shrewd | outfitter | C:5885 |
| Senora Vega | style master | - | - | C:6181 |

- Dress [invented]: layered light cotton, spice-dyed sashes on factors only. The character hold stands: no likenesses or portraits.
- Speech [cite]: built from the greeting lines in each NPC record above (greeting_neutral, friendly, hostile).
- Temperament of the town: read from the power structure and tension in section E.
- Network (merchant, rival): Yasmin al-Nadir - Priya Sundaram, linking Monsoon Reach (C:11496). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Network (tavern, personal): Old Enzo - Old Farouk, linking Porto Novo (C:11520). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Network (broker, rival): Fernanda Reis - Tariq Sayed, linking Porto Novo (C:11592). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Network (inspector, professional): Inspector Salva - Inspector Zara, linking Porto Novo (C:11664). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).

## C. Visual identity (guidance only; commissions nothing; the character hold stands)

Pattern: the shared visual-identity headings 1-7 (design notes). Shared camera, envelope and do/don't blocks are in README.md and not repeated here.

1. Architecture. Identity line [invented]: The Bazaar is the town: Al-Manar is a covered market with a harbour attached. Landmark [cite]: "The Spice Bazaar - a covered market where a hundred aromas compete for your attention." (C:12016 area). Cited buildings: The Harbor of Arrivals (harbor_master, C:7186); The Spice Bazaar (exchange, C:7196); The Khoury Palace (governor, C:7206); The Amber Glass (tavern, C:7216); The Scales of Truth (customs, C:7226); The Apothecary House (apothecary, C:7236); The Broker's Alcove (broker, C:7246). Kit use [proposal]: compose from the shipped will-use plates; identity carriers are ground choice and building frequency, never new ids. Gaps follow the art gap list (design notes).
2. Materials: local and abundant (exports): Spice, Medicines; scarce or luxury (imports): Silk, Grain, Porcelain, Rum, Tea, Iron Ore, Pelts (section A). Atmosphere [cite]: "Cinnamon, cardamom, and the murmur of a hundred negotiations conducted in whispers." (C:12016 area). Climate: section G.
3. Palette (references, not UI tokens; UI stays INK / GOLD / CREAM / MUTED / DUE). Judged in sRGB against the plate envelope (luma 65-141, chroma p98 35):

| Role | Family | sRGB | Luma / chroma | Status | Tie |
|---|---|---|---|---|---|
| Dominant | sun-baked plaster | (132, 118, 96) #847660 | 119 / 36 | mood, not values | the covered bazaar walls |
| Secondary 1 | shade under awnings | (88, 80, 70) #585046 | 81 / 18 | in range | the bazaar's covered lanes |
| Secondary 2 | pale harbour stone | (124, 120, 112) #7C7870 | 120 / 12 | in range | 'the better harbor' (grudge) |
| Accent | saffron dust | (160, 128, 72) #A08048 | 131 / 88 | mood, not values | spice, its first export |

4. Dress: see section B. 5. Signage: ASCII only, no UI-duplicating words; motifs [invented]: a tea cup, a spice scoop. 6. Silhouette: chart scale is the shared `chart_port_marker`; harbour-scale skyline [invented]: a long low domed roof line of the bazaar behind the harbour, with tea-house terraces. 7. Do and don't: README shared list.

## D. Historical analogies (feed [invented] prose; never a cite)

- **Medieval Alexandria's spice entrepot.** Fits: an ancient port reselling spice and medicines to the west (Spice 1.5, Medicines 1.1). Suggests: tea before bargaining, apothecary guilds, long memories of prices. Limit: not a crossroads of empires here, just a rich Alliance port. Avoid: orientalist palace imagery.

## E. Governance

- Title: Merchant Princess (C:7180).
- Power structure: "Al-Manar is governed by hereditary merchant aristocracy - the Khoury family has held the title 'Merchant Princess' for six generations. But real power is shared: Nadia sets policy, Yasmin controls the spice market through the Circle, and Hakim's apothecary gives Al-Manar a unique trade advantage. Old Farouk's tea house is where all three go to think - separately, each believing they're unobserved. Zara, poached from Porto Novo, protects the market's integrity. Tariq pushes for expansion at the broker's desk, supported by Nadia but watched by everyone." [ascii-folded] (C:7182).
- Internal tension: "The core tension is between tradition and ambition. Yasmin wants to preserve Al-Manar's ancient way - the Bazaar's authority, the Circle's traditions, the slow pace of tea-and-negotiation commerce. Tariq wants to modernize - faster contracts, direct Silk Circle partnerships, bypassing Porto Novo entirely. Nadia is playing both sides: she funds Tariq's ambitions while publicly deferring to Yasmin's traditions. Meanwhile, Nadia's private correspondence with the Silk Circle could reshape the entire Alliance - or destroy it. Only Farouk suspects the full picture, and he's not talking." (C:7183).
- Bloc stance: "Al-Manar is the Alliance's soul - older, prouder, and convinced it should be in charge." (C:13541 area).
- Local rival: Porto Novo: "The Spice Merchants' Circle wants Al-Manar to lead the Alliance, not Porto Novo. The rivalry is genteel but real." (C:13541 area).
- Grudge: "Porto Novo got the Alliance headquarters. Al-Manar got the better harbor. Neither has forgiven the other." (C:13541 area).
- Bloc philosophy: "Contracts are sacred. Prices are posted. Cheats are remembered." (C:13388 area). Loyalty and spillover numbers are [flavour, no mechanic] (DF-15).

## F. Relations with every other port

Hand-maintained from content.json. Disposition is bloc-to-bloc (symmetric in data); lanes are undirected; flows are directional by market affinity; networks are symmetric.

<!-- hand-maintained:begin -->
| Port | Region | Bloc | Disposition | Lane (distance, min class, danger) | Flows out (they import) | Flows in (we import) | Network |
|---|---|---|---|---|---|---|---|
| Porto Novo | Mediterranean | The Exchange Alliance | same bloc | 24, sloop, 0.12 (C:1928) | - | Grain, Rum | C:11520, C:11592, C:11664 |
| Silva Bay | Mediterranean | The Exchange Alliance | same bloc | 20, sloop, 0.1 (C:1946) | - | Iron Ore, Pelts | - |
| Corsair's Rest | Mediterranean | The Shadow Ports | hostile (C:13746) | - | Medicines | Pelts, Rum | - |
| Ironhaven | North Atlantic | The Iron Pact | allied (C:13718) | - | - | Iron Ore, Pelts | - |
| Stormwall | North Atlantic | The Iron Pact | allied (C:13718) | - | - | Grain | - |
| Thornport | North Atlantic | The Free Ports | neutral (C:13739) | - | Medicines, Spice | Pelts, Tea | - |
| Sun Harbor | West Africa | The Gold Coast Compact | neutral (C:13725) | 48, brigantine, 0.15 (C:2036) | Spice | Iron Ore, Pelts | - |
| Palm Cove | West Africa | The Gold Coast Compact | neutral (C:13725) | - | - | Rum | - |
| Iron Point | West Africa | The Gold Coast Compact | neutral (C:13725) | - | Medicines | Iron Ore | - |
| Pearl Shallows | West Africa | The Gold Coast Compact | neutral (C:13725) | - | Medicines | - | - |
| Jade Port | East Indies | The Silk Circle | rival (C:13732) | - | - | Porcelain, Silk, Tea | - |
| Monsoon Reach | East Indies | The Free Ports | neutral (C:13739) | 72, galleon, 0.22 (C:2279) | Medicines | Silk | C:11496 |
| Silk Haven | East Indies | The Silk Circle | rival (C:13732) | - | Spice | Porcelain, Silk | - |
| Crosswind Isle | East Indies | The Free Ports | neutral (C:13739) | - | - | - | - |
| Dragon's Gate | East Indies | The Silk Circle | rival (C:13732) | - | Medicines | Porcelain, Tea | - |
| Spice Narrows | East Indies | The Shadow Ports | hostile (C:13746) | - | - | - | - |
| Ember Isle | South Seas | The Coral Crown | neutral (C:13753) | - | - | Iron Ore | - |
| Typhoon Anchorage | South Seas | The Coral Crown | neutral (C:13753) | - | Medicines | - | - |
| Coral Throne | South Seas | The Coral Crown | neutral (C:13753) | - | - | Rum | - |
<!-- hand-maintained:end -->

- Lane lore: "The Grain Road": "The oldest trade route in the Mediterranean. Grain ships have sailed this lane since before memory." (C:1928).
- Lane lore: "The Monsoon Shortcut": "Timing is everything. Catch the monsoon right and you fly. Catch it wrong and you drown." (C:2279).

## G. World details (undefined in data; all [invented] unless cited)

- History (shared timeline in README.md): T4: The Spice Bazaar is roofed over; the port is already called ancient (description). T5: Al-Manar joins the Alliance but loses the headquarters to Porto Novo (grudge text). T14: Al-Manar backs the Monsoon Shortcut to the East Indies, its direct line to spice (route C). T15: The spice tariff on Porto Novo grain ships fails (grudge text).
- Population [invented]: about 30,000.
- Language [invented]: Middle Sea trade speech, with a bazaar argot for weights and grades.
- Currency: silver is the only money in the sim (every price, fee and wage is in silver). Local coin names, if any, are [invented] flavour and never replace 'silver' in copy.
- Religion [invented]: the Old Harbour faith; apothecaries keep household shrines to healing.
- Climate [invented port detail]: hot dry summers; the bazaar roof is built for shade. Regional weather [cite]: "The sea is calm and blue. Sunlight dances on ancient waters." (C:11728 area, weather_flavor).
- Pirates in these waters [cite]: The Crimson Tide (C:2879).

## I. Sound and ambience (all [invented] sound design unless quoted from a cite)

- **Soundscape:** hushed, dense - "the murmur of a hundred negotiations conducted in whispers"; porters' feet on worn marble steps; felucca lateen canvas luffing.
- **Signature:** brass-bound spice chest set down on marble; tea cup on saucer (custom: bargaining begins with tea); scoop of dry spice into a scale pan.
- **Music/work songs (NOTE ONLY):** none in canon; at most a far auctioneer's cadence from the Spice Merchants' Circle - spoken, not sung.
- **Vs neighbours:** quietest Mediterranean market - stone and whispers, where Porto Novo is shouting and Silva Bay is hammering.
- **Fragments [proposal; no fragment or asset exists yet]:** `mat.marble_step`, `src.spice_chest_brass`, `src.tea_cup_saucer`.
- Cited hooks: atmosphere "Cinnamon, cardamom, and the murmur of a hundred negotiations conducted in whispers."; dock scene "Feluccas with triangular sails crowd the harbor. Porters carry brass-bound chests of spice up marble steps worn smooth by centuries of trade." (C:12016 area).
- Invented bed idea: tea-pouring before bargaining; a hundred voices under one roof. No assets are requested.

## J. Flags for this port

- DF-8: NPC name drift across records
- Resolved in lore: 'Senhora' is the Alliance court honorific, adopted at the signing (T5).
