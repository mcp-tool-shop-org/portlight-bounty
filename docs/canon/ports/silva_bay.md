# Silva Bay (silva_bay)

Base commit: 143e284 (content.json unchanged since 45b7378). Status: draft canon, phase 1. Written 2026-10-14.
Cites `C:n` are crates/portlight-sim/data/content.json line n at 143e284. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] (design-notes pattern) | DF-n see data-fixes.md | T-n see README shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Region | Mediterranean ("The Middle Sea") | C:378, C:11728 |
| Bloc | The Exchange Alliance (Porto Novo, Al-Manar, Silva Bay) | C:13550, C:13388 |
| Features | shipyard | C:378-456 |
| Warehouse tiers | depot, regional, commercial | C:6531 |
| Port fee / provisions / repair / crew | 4 / 2 / 1 per hull point / 5 | C:378-456 |
| Map | x 14, y 10 | C:456 |
| Lanes | 5 | section F |
| Player description | "Timber-rich bay surrounded by dense forests. The shipwrights here are the best in the region." | C:380 |
| Captain start here | The Navigator | C:2393 |

## A. Specialties and trade goods

<!-- hand-maintained:begin -->
| Good | Category | Base | Affinity | Stock target | Restock | Role |
|---|---|---|---|---|---|---|
| Timber | commodity | 18 | 1.6 | 40 | 3.5 | exporter |
| Iron Ore | commodity | 25 | 1.3 | 18 | 2.0 | exporter |
| Pelts | commodity | 8 | 1.3 | 10 | 1.5 | exporter |
| Grain | commodity | 12 | 0.9 | 18 | 1.5 | importer |
| Rum | provision | 20 | 0.7 | 10 | 1.0 | importer |
| Cotton | commodity | 15 | 0.6 | 10 | 1.0 | importer |
| Weapons | military | 45 | 0.55 | 8 | 0.8 | importer |
| Silk | luxury | 70 | 0.5 | 5 | 0.5 | importer |
<!-- hand-maintained:end -->

Market cite: C:378-456. Price rule: base x (target / stock) / affinity (crates/portlight-sim/src/economy.rs:37); affinity above 1 is cheap here.
Not traded here: Dyes, Spice, Porcelain, Tea, Pearls, Tobacco, Medicines, Opium, Black Powder, Stolen Cargo.
Region goods [flavour, no mechanic]: sacred Grain; prized Spice, Silk; forbidden none (C:11728 area).
Festival: Harvest of Plenty, every 60 days for 3, market effects Grain x1.8, Rum x1.4 (C:11752).
Festival: Night of Lanterns, every 90 days for 3, market effects Spice x2.0, Silk x1.5, Dyes x1.6 (C:11765).
Contract templates touching Mediterranean: `proc_iron_forge` (procurement), `short_tea_crisis` (shortage), `ret_cotton_return` (return_freight), `ret_spice_restock` (return_freight), `ret_dyes_textile` (return_freight), `circ_med_africa` (circuit), `circ_indies_loop` (circuit), `circ_south_seas` (circuit), `rep_charter_atlantic` (reputation_charter), `smug_opium_run` (smuggling), `smug_powder_delivery` (smuggling). Source port Silva Bay: none.

## B. People

- Cultural group: The Shipwrights' Brotherhood: "Master builders who guard their hull designs like state secrets." (C:12026 area).
- Local custom: "Touching another captain's ship without permission is grounds for a duel." (C:12026 area).
- Regional ethos: "Civilization is commerce. The law of contract binds harder than iron, and a merchant's word, once given, is recorded in stone." Greeting: "Welcome to port, Captain. Your cargo manifest, if you please." Proverb: "A contract honored is worth more than gold hoarded." (C:11728 area).
- Tavern rumour (in-world, unreliable): "The master shipwright refuses to build for anyone who has lost a ship to storms. She says it's bad wood-luck." (C:12026 area).

| Name | Title | Temperament | Institution | Cite |
|---|---|---|---|---|
| Elena Madeira | Master Shipwright | exacting | shipyard | C:7492 |
| Nuno Ferreira | Harbor Master | practical | harbor_master | C:7514 |
| Tomas Verdelho | Timber Factor | patient | exchange | C:7536 |
| The Brotherhood Council | Governing Council | collective | governor | C:7558 |
| Rosa Carvalho | Tavern Keeper | warm | tavern | C:7580 |
| Customs Officer Pires | Customs Officer | awkward | customs | C:7602 |
| Ana Sousa | Broker | earnest | broker | C:7624 |
| Old Silva | Ship Outfitter (merchant) | quiet | outfitter | C:5899 |

- Dress [invented]: leather aprons, canvas smocks dusted with sawdust, hair tied back (cited for Elena); the Alliance customs coat is the only uniform. The character hold stands: no likenesses or portraits.
- Speech [cite]: built from the greeting lines in each NPC record above (greeting_neutral, friendly, hostile).
- Temperament of the town: read from the power structure and tension in section E.
- Network (tavern, personal): Rosa Carvalho - Ingrid Norum, linking Stormwall (C:11568). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Network (broker, professional): Fernanda Reis - Ana Sousa, linking Porto Novo (C:11604). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Sea sighting filed under this port: Dimitri Andros (C:12804).
- Hire and training canon (links only): [The Silva Bay Smithy](../skill-trainers/silva_bay_smiths.md) (skill trainer, Blacksmithing); [Old Silva](../merchants/old_silva.md) (merchant, Ship Outfitter).

## C. Visual identity (guidance only; commissions nothing; the character hold stands)

Pattern: the shared visual-identity headings 1-7 (design notes). Shared camera, envelope and do/don't blocks are in README.md and not repeated here.

1. Architecture. Identity line [invented]: The yard is the town (C:7414, 'The yard IS the town'). Landmark [cite]: "The Master Shipyard - where the finest hulls in the Mediterranean take shape." (C:12026 area). Cited buildings: The Master Shipyard (shipyard, C:7420); The Working Harbor (harbor_master, C:7430); The Timber Yard (exchange, C:7440); The Brotherhood Hall (governor, C:7450); The Dry Dock (tavern, C:7460); The Customs Shed (customs, C:7470); The Broker's Bench (broker, C:7480). Kit use [proposal]: compose from the shipped will-use plates; identity carriers are ground choice and building frequency, never new ids. Gaps follow the art gap list (design notes). Landmark canon: ../landmarks/master_shipyard.md.
2. Materials: local and abundant (exports): Timber, Iron Ore, Pelts; scarce or luxury (imports): Grain, Cotton, Weapons, Rum, Silk (section A). Atmosphere [cite]: "Fresh-cut timber, hot pitch, and the rhythmic sound of adzes shaping keels." (C:12026 area). Climate: section G.
3. Palette (references, not UI tokens; UI stays INK / GOLD / CREAM / MUTED / DUE). Judged in sRGB against the plate envelope (luma 65-141, chroma p98 35):

| Role | Family | sRGB | Luma / chroma | Status | Tie |
|---|---|---|---|---|---|
| Dominant | weathered tarred timber | (103, 86, 68) #675644 | 88 / 35 | in range | plank walls, roofs, slipways (art-reviewed value) |
| Secondary 1 | forest canopy green | (70, 90, 64) #465A40 | 84 / 26 | in range | 'dense forests' (art-reviewed value) |
| Secondary 2 | fieldstone and wet quay grey | (118, 116, 108) #76746C | 116 / 10 | in range | footings and quay (art-reviewed value) |
| Accent | fresh-cut oak and sawdust | (163, 137, 99) #A38963 | 140 / 64 | small-area accent (chroma 64) | fresh cuts and sawdust only (art-reviewed value) |

4. Dress: see section B. 5. Signage: ASCII only, no UI-duplicating words; motifs [invented]: timber grade chalk marks A, B, C; a keel-section silhouette; an adze. 6. Silhouette: chart scale is the shared `chart_port_marker`; harbour-scale skyline [invented]: low serrated shed ridges stepping back from the quay, warehouses behind, the forest band beyond; the slipway with a half-built hull's ribs is the only tall shape. 7. Do and don't: README shared list.
Man-of-war note (design ruling 2): the Royal Man-of-War stays in lore as a Silva Bay yard product, since the sim sells it at every shipyard (crates/portlight-sim/src/session.rs:3037-3043; C:193). No plate art exists for it (the art hold covers `man_of_war`), so no art depicts it until that hold lifts.

## D. Historical analogies (feed [invented] prose; never a cite)

- **Zaandam (Dutch Republic, 17th c.).** Fits: a private-yard shipbuilding town that built for outside buyers (best shipwrights, C:378-456). Suggests: buyers judged by how they treat the work; prices before titles. Limit: Silva Bay grows its own timber; no windmills. Avoid: clogs, tulips, Dutch-coded names.
- **Basque cofradias.** Fits: a self-governing brotherhood running a harbour by assembly under a distant power (C:7414, C:13550). Suggests: decisions announced by the Hall bell; oath to the brotherhood above the bloc. Limit: cofradias were mostly fishermen; borrow the governance form only. Avoid: nationalist imagery, invented Basque words.
- **Baltic timber wrackers.** Fits: sworn graders whose grade set the price (Tomas grades A, B, C). Suggests: the grade mark is a public promise; 'a B-grade promise' as insult. Limit: [flavour, no mechanic]: the sim has no timber grades. Avoid: implying A timber sells for more.
- Contrast, not model: The Venetian Arsenal: a state war yard. Silva Bay is private and refuses to be any power's arsenal (see the warship ruling).

## E. Governance

- Title: Brotherhood Council (C:7414).
- Power structure: "Silva Bay is a shipwrights' republic - no single ruler, no hereditary title. The Brotherhood Council of seven governs by consensus, chaired by the elected Master Shipwright (Elena). The yard IS the town: Elena builds the ships, Tomas grades the timber, Nuno runs the harbor, and everything else exists to support that work. Rosa's tavern is the unofficial second chamber where real opinions emerge. Ana's broker bench funds it all. Pires, the Alliance customs officer, is tolerated but never included." [ascii-folded] (C:7416).
- Internal tension: "The core tension is independence vs. relevance. The Brotherhood governs itself and answers to no one - but the Alliance provides trade protection and market access that Silva Bay needs. Elena wants to stay free; the Alliance wants Silva Bay's ships. The leverage is mutual, which creates a fragile equilibrium. Tomas adds a second tension: his harvest limits protect the forests but frustrate merchants who want more timber faster. The quiet tension is Pires - an outsider who represents everything the Brotherhood distrusts (external authority), yet who genuinely tries to belong. Ana is his only ally, and she's still earning her own place." (C:7417).
- Bloc stance: "Silva Bay stays out of politics and builds ships. The Alliance needs them too much to push." (C:13550 area).
- Grudge: "The Shipwrights' Brotherhood refused to build warships for the Iron Pact. Stormwall hasn't forgotten." (C:13550 area).
- Bloc philosophy: "Contracts are sacred. Prices are posted. Cheats are remembered." (C:13388 area). Loyalty and spillover numbers are [flavour, no mechanic] (DF-15).
- Faction canon: bloc [The Exchange Alliance](../factions/exchange_alliance.md); archetypes starting here [The Navigator](../captains/navigator.md).

## F. Relations with every other port

Hand-maintained from content.json. Disposition is bloc-to-bloc (symmetric in data); lanes are undirected; flows are directional by market affinity; networks are symmetric.

<!-- hand-maintained:begin -->
| Port | Region | Bloc | Disposition | Lane (distance, min class, danger) | Flows out (they import) | Flows in (we import) | Network |
|---|---|---|---|---|---|---|---|
| Porto Novo | Mediterranean | The Exchange Alliance | same bloc | 16, sloop, 0.08 (C:1937) | Pelts, Timber | Grain, Rum | C:11604 |
| Al-Manar | Mediterranean | The Exchange Alliance | same bloc | 20, sloop, 0.1 (C:1946) | Iron Ore, Pelts | - | - |
| Corsair's Rest | Mediterranean | The Shadow Ports | hostile (C:13746) | 14, sloop, 0.15 (C:1964) | - | Rum, Weapons | - |
| Ironhaven | North Atlantic | The Iron Pact | allied (C:13718) | 32, brigantine, 0.11 (C:2009) | Timber | Weapons | - |
| Stormwall | North Atlantic | The Iron Pact | allied (C:13718) | - | Pelts | Grain, Weapons | C:11568 |
| Thornport | North Atlantic | The Free Ports | neutral (C:13739) | - | - | - | - |
| Sun Harbor | West Africa | The Gold Coast Compact | neutral (C:13725) | - | - | Cotton | - |
| Palm Cove | West Africa | The Gold Coast Compact | neutral (C:13725) | 44, brigantine, 0.13 (C:2045) | Iron Ore, Pelts, Timber | Cotton, Rum | - |
| Iron Point | West Africa | The Gold Coast Compact | neutral (C:13725) | - | Pelts, Timber | - | - |
| Pearl Shallows | West Africa | The Gold Coast Compact | neutral (C:13725) | - | - | Cotton | - |
| Jade Port | East Indies | The Silk Circle | rival (C:13732) | - | Iron Ore, Pelts | Silk | - |
| Monsoon Reach | East Indies | The Free Ports | neutral (C:13739) | - | Pelts, Timber | Silk | - |
| Silk Haven | East Indies | The Silk Circle | rival (C:13732) | - | Iron Ore, Pelts | Silk | - |
| Crosswind Isle | East Indies | The Free Ports | neutral (C:13739) | - | Pelts | - | - |
| Dragon's Gate | East Indies | The Silk Circle | rival (C:13732) | - | Iron Ore, Pelts | - | - |
| Spice Narrows | East Indies | The Shadow Ports | hostile (C:13746) | - | Pelts | - | - |
| Ember Isle | South Seas | The Coral Crown | neutral (C:13753) | - | Pelts, Timber | - | - |
| Typhoon Anchorage | South Seas | The Coral Crown | neutral (C:13753) | - | Pelts | - | - |
| Coral Throne | South Seas | The Coral Crown | neutral (C:13753) | - | Pelts | Rum | - |
<!-- hand-maintained:end -->

- Lane lore: "The Timber Run": "Short and safe - the shipwrights' lifeline. Silva Bay's timber built Porto Novo's fleet." (C:1937). Lane canon: [The Timber Run](../lanes/timber_run.md).

## G. World details (undefined in data; all [invented] unless cited)

- History (shared timeline in README.md): T6: The Brotherhood Hall's carved ship-name walls begin (two hundred years of names, C:7454 area). T5: Silva Bay accepts Alliance membership on conditions (C:7414 'membership is by choice'). T15: The Brotherhood refuses the Iron Pact's commission for a war fleet (C:13556). Ruling: it still builds single hulls of every class for captains at its own yard, including the Royal Man-of-War; it will not build fleets to a power's order.
- Population [invented]: about 6,000, most of them in the yard trades.
- Language [invented]: Middle Sea trade speech; yard cant for grades and joints.
- Currency: silver is the only money in the sim (every price, fee and wage is in silver). Local coin names, if any, are [invented] flavour and never replace 'silver' in copy.
- Religion [invented]: the Old Harbour faith, plus 'wood-luck': a keel is blessed with a timber offcut thrown into the mast pond.
- Climate [invented port detail]: Mediterranean coast with a wetter forested hinterland; morning sea fog over the mast pond. Regional weather [cite]: "The sea is calm and blue. Sunlight dances on ancient waters." (C:11728 area, weather_flavor).
- Pirates in these waters [cite]: The Crimson Tide (C:2879).

## I. Sound and ambience (all [invented] sound design unless quoted from a cite)

- **Soundscape:** forest-backed bay; "the rhythmic sound of adzes shaping keels" (C:12029); sawdust-soft quay; hot pitch bubbling.
- **Signature:** adze bite into green timber; mallet on treenail; pitch pot simmer.
- **Music/work songs (NOTE ONLY):** none in canon; adze rhythm is the work-song stand-in.
- **Vs neighbours:** wood-on-wood (Ironhaven is iron-on-iron); a working yard, not a market.
- **Fragments [proposal; no fragment or asset exists yet]:** proposed `place.dockside_yard`, `place.timber_quay`, `src.adze_keel`, `src.pitch_pot`.
- Cited hooks: atmosphere "Fresh-cut timber, hot pitch, and the rhythmic sound of adzes shaping keels." (C:12029); dock scene "Sawdust drifts like snow across the waterfront. Half-finished hulls rise on slipways, their ribs exposed like the skeletons of great fish." (C:12030).
- Invented bed idea: adzes, saws and mallets; the Hall bell opening council; timber barges at dawn. No assets are requested.

## J. Flags for this port

- DF-1: Dimitri Andros sighting filed under Silva Bay
- DF-2: Porto Novo power structure says 'the Brotherhood answers to craft'
- DF-8: NPC name drift across records
- DF-11: Non-ASCII NPC id `sb_tom` + a-acute + `s`
- DF-12: Rosa and Ingrid network says 'Northern solidarity'
- DF-13: Silva Bay 'cheapest repairs' is a tie
- DF-14: Brotherhood 'never builds warships' conflicts with the sim
- Resolved in lore: Warships (design ruling 2): the yard builds single hulls of every class, including the Royal Man-of-War, for any captain at its own slipways. It refuses to build a war fleet to any power's order; that was the Iron Pact refusal (T15). Data wording: DF-14.
- Resolved in lore: The tavern rumour about storm-lost captains is an unreliable rumour. Elena builds for anyone who respects ships.
- Resolved in lore: Rosa's network text: canon treats her as Mediterranean (DF-12).
