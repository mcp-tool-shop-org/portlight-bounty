# Porto Novo (porto_novo)

Base commit: 143e284 (content.json unchanged since 45b7378). Status: draft canon, phase 1. Written 2026-10-14.
Cites `C:n` are crates/portlight-sim/data/content.json line n at 143e284. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] (design-notes pattern) | DF-n see data-fixes.md | T-n see README shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Region | Mediterranean ("The Middle Sea") | C:210, C:11728 |
| Bloc | The Exchange Alliance (Porto Novo, Al-Manar, Silva Bay) | C:13532, C:13388 |
| Features | shipyard | C:210-288 |
| Warehouse tiers | depot, regional, commercial | C:6521 |
| Port fee / provisions / repair / crew | 5 / 1 / 2 per hull point / 4 | C:210-288 |
| Map | x 18, y 8 | C:288 |
| Lanes | 5 | section F |
| Player description | "A bustling harbor city, gateway to inland trade. Grain ships fill the docks." | C:212 |
| Captain start here | The Merchant | C:2317 |

## A. Specialties and trade goods

<!-- hand-maintained:begin -->
| Good | Category | Base | Affinity | Stock target | Restock | Role |
|---|---|---|---|---|---|---|
| Grain | commodity | 12 | 1.4 | 35 | 3.0 | exporter |
| Rum | provision | 20 | 1.1 | 20 | 1.5 | exporter |
| Iron Ore | commodity | 25 | 1.0 | 15 | 1.5 | neutral |
| Timber | commodity | 18 | 0.8 | 25 | 2.0 | importer |
| Pelts | commodity | 8 | 0.8 | 10 | 1.0 | importer |
| Cotton | commodity | 15 | 0.6 | 12 | 1.0 | importer |
| Dyes | commodity | 22 | 0.6 | 10 | 1.0 | importer |
| Porcelain | luxury | 60 | 0.55 | 6 | 0.5 | importer |
<!-- hand-maintained:end -->

Market cite: C:210-288. Price rule: base x (target / stock) / affinity (crates/portlight-sim/src/economy.rs:37); affinity above 1 is cheap here.
Not traded here: Spice, Silk, Tea, Pearls, Tobacco, Weapons, Medicines, Opium, Black Powder, Stolen Cargo.
Region goods [flavour, no mechanic]: sacred Grain; prized Spice, Silk; forbidden none (C:11728 area).
Festival: Harvest of Plenty, every 60 days for 3, market effects Grain x1.8, Rum x1.4 (C:11752).
Festival: Night of Lanterns, every 90 days for 3, market effects Spice x2.0, Silk x1.5, Dyes x1.6 (C:11765).
Contract templates touching Mediterranean: `proc_iron_forge` (procurement), `short_tea_crisis` (shortage), `ret_cotton_return` (return_freight), `ret_spice_restock` (return_freight), `ret_dyes_textile` (return_freight), `circ_med_africa` (circuit), `circ_indies_loop` (circuit), `circ_south_seas` (circuit), `rep_charter_atlantic` (reputation_charter), `smug_opium_run` (smuggling), `smug_powder_delivery` (smuggling). Source port Porto Novo: none.

## B. People

- Cultural group: The Exchange Guild: "Keepers of the price - they set the dawn rates and arbitrate disputes." (C:12006 area).
- Local custom: "Captains ring the harbor bell on first arrival. It's bad luck not to." (C:12006 area).
- Regional ethos: "Civilization is commerce. The law of contract binds harder than iron, and a merchant's word, once given, is recorded in stone." Greeting: "Welcome to port, Captain. Your cargo manifest, if you please." Proverb: "A contract honored is worth more than gold hoarded." (C:11728 area).
- Tavern rumour (in-world, unreliable): "They say the Grain Exchange is built on the ruins of an older temple. Some nights the clerks hear chanting from beneath the floor." (C:12006 area).

| Name | Title | Temperament | Institution | Cite |
|---|---|---|---|---|
| Vasco da Reira | Harbor Master | meticulous | harbor_master | C:7030 |
| Marta Soares | Guild Factor | shrewd | exchange | C:7052 |
| Dimitri Andros | Master Shipwright | gruff | shipyard | C:7073 |
| Senhora Isabela Costa | Port Governor | diplomatic | governor | C:7094 |
| Old Enzo | Tavern Keeper | jovial | tavern | C:7115 |
| Inspector Salva | Chief Customs Inspector | relentless | customs | C:7136 |
| Fernanda Reis | Senior Broker | calculating | broker | C:7157 |
| Marco | Master Weaponsmith (merchant) | charming | outfitter | C:5870 |
| Maestro Luciano | style master | - | - | C:6173 |

- Dress [invented]: clerks' dark coats with ledger pouches; dockers in undyed cotton. The character hold stands: no likenesses or portraits.
- Speech [cite]: built from the greeting lines in each NPC record above (greeting_neutral, friendly, hostile).
- Temperament of the town: read from the power structure and tension in section E.
- Network (merchant, professional): Marta Soares - Ama Mensah, linking Sun Harbor (C:11412). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Network (tavern, personal): Old Enzo - Old Farouk, linking Al-Manar (C:11520). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Network (tavern, professional): Mother Ko - Old Enzo, linking Crosswind Isle (C:11556). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Network (broker, rival): Fernanda Reis - Tariq Sayed, linking Al-Manar (C:11592). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Network (broker, professional): Fernanda Reis - Ana Sousa, linking Silva Bay (C:11604). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Network (broker, professional): Broker Chang - Fernanda Reis, linking Silk Haven (C:11628). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Network (inspector, professional): Inspector Salva - Inspector Zara, linking Al-Manar (C:11664). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Network (inspector, respected): Inspector Salva - Inspector Sun, linking Dragon's Gate (C:11712). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Sea sighting filed under this port: Marta Soares (C:12810).
- Sea sighting filed under this port: Inspector Salva (C:12822).
- Hire and training canon (links only): [Maestro Luciano](../style-masters/maestro_luciano.md) (style master, La Destreza); [Old Vasquez](../skill-trainers/old_vasquez.md) (skill trainer, Blacksmithing); [Marco](../merchants/marco_the_blade.md) (merchant, Master Weaponsmith).

## C. Visual identity (guidance only; commissions nothing; the character hold stands)

Pattern: the shared visual-identity headings 1-7 (design notes). Shared camera, envelope and do/don't blocks are in README.md and not repeated here.

1. Architecture. Identity line [invented]: The Exchange is the town: every street runs downhill to the Grain Exchange, and the bell decides the day. Landmark [cite]: "The Grain Exchange - a columned hall where prices are read aloud at dawn." (C:12006 area). Cited buildings: The Harbor Master's Tower (harbor_master, C:6958); The Grain Exchange (exchange, C:6968); Andros & Sons Shipyard (shipyard, C:6978); The Governor's Residence (governor, C:6988); The Harbor Bell (tavern, C:6998); The Customs House (customs, C:7008); The Broker's Desk (broker, C:7018). Kit use [proposal]: compose from the shipped will-use plates; identity carriers are ground choice and building frequency, never new ids. Gaps follow the art gap list (design notes). Landmark canon: ../landmarks/grain_exchange.md, ../landmarks/porto_novo_beacon.md.
2. Materials: local and abundant (exports): Grain, Rum; scarce or luxury (imports): Timber, Cotton, Dyes, Pelts, Porcelain (section A). Atmosphere [cite]: "Flour dust, sea salt, and the shouts of dockhands loading grain barges." (C:12006 area). Climate: section G.
3. Palette (references, not UI tokens; UI stays INK / GOLD / CREAM / MUTED / DUE). Judged in sRGB against the plate envelope (luma 65-141, chroma p98 35):

| Role | Family | sRGB | Luma / chroma | Status | Tie |
|---|---|---|---|---|---|
| Dominant | limewashed stone in shade | (120, 114, 104) #787268 | 115 / 16 | in range | the columned Grain Exchange |
| Secondary 1 | grain-sack hemp | (126, 112, 91) #7E705B | 113 / 35 | in range | grain, its main export |
| Secondary 2 | harbour slate | (92, 96, 98) #5C6062 | 95 / 6 | in range | quay and civic roofs |
| Accent | Exchange brass | (148, 126, 84) #947E54 | 128 / 64 | small-area accent (chroma 64) | the harbour bell and scale pans |

4. Dress: see section B. 5. Signage: ASCII only, no UI-duplicating words; motifs [invented]: a balance scale, wheat sheaf, the bell. 6. Silhouette: chart scale is the shared `chart_port_marker`; harbour-scale skyline [invented]: a columned hall on a rise with warehouses stepping down to the quay; the harbour bell tower is the tallest point. 7. Do and don't: README shared list.

## D. Historical analogies (feed [invented] prose; never a cite)

- **Antwerp's Bourse (16th c.).** Fits: a posted-price exchange hall at the centre of a grain and general-goods port (Grain Exchange, prices read aloud at dawn). Suggests: price boards, punctual bell-ringing, clerks who know every captain's ledger. Limit: Antwerp was a money market; Porto Novo is grain and contract law first. Avoid: burgher costume, Flemish names.
- **Hanseatic Kontor rulebooks.** Fits: a port bound by written contract law and shared weights (Exchange Alliance description). Suggests: formal politeness, written manifests, distrust of verbal deals. Limit: no league of cities here beyond the three Alliance ports. Avoid: league cogs and Lubeck imagery.

## E. Governance

- Title: Port Governor (C:6952).
- Power structure: "Porto Novo is governed by Senhora Costa, but real power is triangulated: Costa sets policy, Marta controls prices through the Exchange Guild, and Salva enforces the law. Dimitri's shipyard is independent - the Brotherhood answers to craft, not politics. Old Enzo's tavern is the shadow channel where information flows that the official institutions pretend doesn't exist. Fernanda's broker desk is where policy meets practice - she decides which contracts reach which captains." [ascii-folded] (C:6954).
- Internal tension: "The core tension is between Marta and Costa. Marta wants market dominance - she'd squeeze every margin to make the Exchange richer. Costa wants political stability - she'd sacrifice some profit for Alliance cohesion. They need each other: Costa can't govern without the Guild's revenue, and Marta can't trade without Costa's political protection. The tension keeps Porto Novo honest. Meanwhile, Old Enzo quietly facilitates the grey economy that neither woman acknowledges exists, and Inspector Salva watches everything with the patience of a man who knows that sooner or later, everyone makes a mistake." (C:6955).
- Bloc stance: "Porto Novo is the Alliance's heart. The Exchange Guild sets policy here." (C:13532 area).
- Local rival: Al-Manar: "Both claim to be the Mediterranean's premier trading port. The grain exchange vs. the spice bazaar - centuries of competition." (C:13532 area).
- Grudge: "Al-Manar once tried to impose a spice tariff on Porto Novo grain ships. Porto Novo responded by raising port fees for Al-Manar flagged vessels. The tariff lasted three days." (C:13532 area).
- Bloc philosophy: "Contracts are sacred. Prices are posted. Cheats are remembered." (C:13388 area). Loyalty and spillover numbers are [flavour, no mechanic] (DF-15).
- Faction canon: bloc [The Exchange Alliance](../factions/exchange_alliance.md); archetypes starting here [The Merchant](../captains/merchant.md).

## F. Relations with every other port

Hand-maintained from content.json. Disposition is bloc-to-bloc (symmetric in data); lanes are undirected; flows are directional by market affinity; networks are symmetric.

<!-- hand-maintained:begin -->
| Port | Region | Bloc | Disposition | Lane (distance, min class, danger) | Flows out (they import) | Flows in (we import) | Network |
|---|---|---|---|---|---|---|---|
| Al-Manar | Mediterranean | The Exchange Alliance | same bloc | 24, sloop, 0.12 (C:1928) | Grain, Rum | - | C:11520, C:11592, C:11664 |
| Silva Bay | Mediterranean | The Exchange Alliance | same bloc | 16, sloop, 0.08 (C:1937) | Grain, Rum | Pelts, Timber | C:11604 |
| Corsair's Rest | Mediterranean | The Shadow Ports | hostile (C:13746) | 18, sloop, 0.18 (C:1955) | Grain | Pelts | - |
| Ironhaven | North Atlantic | The Iron Pact | allied (C:13718) | 36, cutter, 0.12 (C:2000) | Grain, Rum | Pelts | - |
| Stormwall | North Atlantic | The Iron Pact | allied (C:13718) | - | Rum | - | - |
| Thornport | North Atlantic | The Free Ports | neutral (C:13739) | - | - | Pelts, Timber | - |
| Sun Harbor | West Africa | The Gold Coast Compact | neutral (C:13725) | 40, cutter, 0.12 (C:2027) | Rum | Cotton, Dyes, Pelts | C:11412 |
| Palm Cove | West Africa | The Gold Coast Compact | neutral (C:13725) | - | Grain | Cotton | - |
| Iron Point | West Africa | The Gold Coast Compact | neutral (C:13725) | - | Grain, Rum | - | - |
| Pearl Shallows | West Africa | The Gold Coast Compact | neutral (C:13725) | - | Grain, Rum | Cotton, Dyes | - |
| Jade Port | East Indies | The Silk Circle | rival (C:13732) | - | Grain, Rum | Porcelain | - |
| Monsoon Reach | East Indies | The Free Ports | neutral (C:13739) | - | Grain, Rum | - | - |
| Silk Haven | East Indies | The Silk Circle | rival (C:13732) | - | Grain, Rum | Porcelain | C:11628 |
| Crosswind Isle | East Indies | The Free Ports | neutral (C:13739) | - | - | - | C:11556 |
| Dragon's Gate | East Indies | The Silk Circle | rival (C:13732) | - | Rum | Porcelain | C:11712 |
| Spice Narrows | East Indies | The Shadow Ports | hostile (C:13746) | - | Grain, Rum | - | - |
| Ember Isle | South Seas | The Coral Crown | neutral (C:13753) | - | Grain, Rum | Dyes | - |
| Typhoon Anchorage | South Seas | The Coral Crown | neutral (C:13753) | - | Grain, Rum | Timber | - |
| Coral Throne | South Seas | The Coral Crown | neutral (C:13753) | - | Grain | - | - |
<!-- hand-maintained:end -->

- Lane lore: "The Grain Road": "The oldest trade route in the Mediterranean. Grain ships have sailed this lane since before memory." (C:1928). Lane canon: [The Grain Road](../lanes/grain_road.md).
- Lane lore: "The Timber Run": "Short and safe - the shipwrights' lifeline. Silva Bay's timber built Porto Novo's fleet." (C:1937). Lane canon: [The Timber Run](../lanes/timber_run.md).
- Lane lore: "The Shadow Lane": "Every merchant denies using it. Every merchant uses it. The line between trade and smuggling runs through these waters." (C:1955). Lane canon: [The Shadow Lane](../lanes/shadow_lane.md).
- Lane lore: "The Cotton Crossing": "Mediterranean grain south, Gold Coast cotton north. The route that clothed an empire." (C:2027). Lane canon: [The Cotton Crossing](../lanes/cotton_crossing.md).
- Asymmetry flag: Corsair's Rest names Porto Novo as local rival (C:13671); Porto Novo names Al-Manar (C:13532). Canon reads local_rival_port as directional: a grievance, not a mutual feud. See DF-6.

## G. World details (undefined in data; all [invented] unless cited)

- History (shared timeline in README.md): T4: The Grain Exchange is raised on the ruins of an older temple [invented date; ruins per inventory culture text]. T5: Porto Novo hosts the signing of the Exchange Alliance and keeps its headquarters (grudge text cites the HQ). T13: Porto Novo tightens its customs code; the traders it drives out found Corsair's Rest (rivalry text cites the cause).
- Population [invented]: about 40,000; the largest Alliance port.
- Language [invented]: Middle Sea trade speech; clerks write in the Exchange hand.
- Currency: silver is the only money in the sim (every price, fee and wage is in silver). Local coin names, if any, are [invented] flavour and never replace 'silver' in copy.
- Religion [invented]: the Old Harbour faith: a bell for arrival, a coin for the sea; quiet and civic.
- Climate [invented port detail]: dry summers, wet winters; olive country inland. Regional weather [cite]: "The sea is calm and blue. Sunlight dances on ancient waters." (C:11728 area, weather_flavor).
- Pirates in these waters [cite]: The Crimson Tide (C:2879).

## I. Sound and ambience (all [invented] sound design unless quoted from a cite)

- **Soundscape:** big, busy stone quays; "shouts of dockhands loading grain barges" (C:12009); barges knocking fenders; sacks thudding.
- **Signature:** heavy grain sack dropped on stone; stylus on wax tablet (clerks tallying bushels); the harbour bell rung on first arrival (custom) - port-ambience only.
- **Sea mark:** "The beacon of Porto Novo breaks through the haze. You're on course." (crates/portlight-sim/src/voyage.rs:594); see ../landmarks/porto_novo_beacon.md
- **Music/work songs (NOTE ONLY):** dockhand call-and-response is plausible but canon says "shouts", not song - keep as shouts (and voice is on hold).
- **Vs neighbours:** the loud, open, grain-dusty Med hub; Al-Manar whispers, Silva Bay is woodwork.
- **Fragments [proposal; no fragment or asset exists yet]:** `src.grain_sack_drop`, `mat.wax_tablet`, `place.stone_quay`.
- Cited hooks: atmosphere "Flour dust, sea salt, and the shouts of dockhands loading grain barges." (C:12009); dock scene "Grain barges jostle for position along the stone quays. Clerks with wax tablets tally every bushel while porters sweat under heavy sacks." (C:12010).
- Invented bed idea: dawn price-calling in the Exchange; the arrival bell; grain chutes. No assets are requested.

## J. Flags for this port

- DF-1: Dimitri Andros sighting filed under Silva Bay
- DF-2: Porto Novo power structure says 'the Brotherhood answers to craft'
- DF-6: Local rivals are one-sided
