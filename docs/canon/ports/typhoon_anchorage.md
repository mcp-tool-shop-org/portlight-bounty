# Typhoon Anchorage (typhoon_anchorage)

Base commit: 143e284 (content.json unchanged since 45b7378). Status: draft canon, phase 1. Written 2026-10-14.
Cites `C:n` are crates/portlight-sim/data/content.json line n at 143e284. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] (design-notes pattern) | DF-n see data-fixes.md | T-n see README shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Region | South Seas ("The Reef Kingdoms") | C:1750, C:11950 |
| Bloc | The Coral Crown (Ember Isle, Typhoon Anchorage, Coral Throne) | C:13698, C:13511 |
| Features | shipyard | C:1750-1836 |
| Warehouse tiers | depot, regional, commercial | C:6570 |
| Port fee / provisions / repair / crew | 4 / 1 / 2 per hull point / 4 | C:1750-1836 |
| Map | x 40, y 30 | C:1836 |
| Lanes | 3 | section F |
| Player description | "Storm-battered harbor that only the boldest captains visit. Pearls and rare goods reward the brave." | C:1752 |

## A. Specialties and trade goods

<!-- hand-maintained:begin -->
| Good | Category | Base | Affinity | Stock target | Restock | Role |
|---|---|---|---|---|---|---|
| Pearls | luxury | 95 | 1.7 | 15 | 2.0 | exporter |
| Timber | commodity | 18 | 1.4 | 22 | 2.5 | exporter |
| Spice | luxury | 55 | 1.2 | 10 | 1.5 | exporter |
| Pelts | commodity | 8 | 0.9 | 8 | 1.0 | importer |
| Medicines | medicine | 35 | 0.8 | 10 | 1.0 | importer |
| Rum | provision | 20 | 0.6 | 8 | 0.8 | importer |
| Grain | commodity | 12 | 0.55 | 6 | 0.5 | importer |
| Silk | luxury | 70 | 0.45 | 5 | 0.5 | importer |
| Porcelain | luxury | 60 | 0.45 | 4 | 0.5 | importer |
<!-- hand-maintained:end -->

Market cite: C:1750-1836. Price rule: base x (target / stock) / affinity (crates/portlight-sim/src/economy.rs:37); affinity above 1 is cheap here.
Not traded here: Iron Ore, Cotton, Dyes, Tea, Tobacco, Weapons, Opium, Black Powder, Stolen Cargo.
Region goods [flavour, no mechanic]: sacred Pearls; prized Weapons, Silk, Tea; forbidden none (C:11950 area).
Festival: Coral Coronation, every 90 days for 5, market effects Weapons x2.5, Silk x2.0, Pearls x1.8 (C:11975).
Festival: Fire Walking, every 75 days for 3, market effects Medicines x2.0, Dyes x1.8 (C:11989).
Contract templates touching South Seas: `proc_grain_feed` (procurement), `proc_timber_shipyard` (procurement), `proc_medicines_relief` (procurement), `proc_weapons_garrison` (procurement), `circ_south_seas` (circuit), `smug_powder_delivery` (smuggling). Source port Typhoon Anchorage: none.

## B. People

- Cultural group: The Storm Riders: "Captains who have weathered the typhoons. Their stories are the price of entry." (C:12186 area).
- Local custom: "Captains who survive a typhoon carve their ship's name into the Storm Wall. It's an honor." (C:12186 area).
- Regional ethos: "The sea gives life and the sea takes it back. The island kings answer to no empire. Trade here is tribute - offered willingly or not at all. Respect the reef, or it will tear your hull apart." Greeting: "The Coral King permits you to anchor. State your tribute." Proverb: "The reef welcomes the patient; it drowns the greedy." (C:11950 area).
- Tavern rumour (in-world, unreliable): "A captain carved her name into the Storm Wall seven times - one for each typhoon survived. They call her the Unkillable. She drinks alone." (C:12186 area).

| Name | Title | Temperament | Institution | Cite |
|---|---|---|---|---|
| Storm Chief Rangi | Storm Chief | fearless | governor | C:11086 |
| Harbor Master Koa | Chain Master | methodical | harbor_master | C:11107 |
| Dive Boss Moana | Pearl Boss | bold | exchange | C:11128 |
| Shipwright Ahe | Storm Shipwright | innovative | shipyard | C:11149 |
| Cook Hine | Storm Cook | resilient | tavern | C:11170 |
| Broker Wiremu | Trade Broker | steady | broker | C:11191 |
| Malia | Outfitter (merchant) | charming | outfitter | C:6060 |
| Leilani Bonecatcher | style master | - | - | C:6245 |

- Dress [invented]: tight-bound cotton, salt-stained; carved name tokens. The character hold stands: no likenesses or portraits.
- Speech [cite]: built from the greeting lines in each NPC record above (greeting_neutral, friendly, hostile).
- Temperament of the town: read from the power structure and tension in section E.
- Network (tavern, personal): Cook Hine - Spring Keeper Olo, linking Ember Isle (C:11580). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Sea sighting filed under this port: Storm Chief Rangi (C:12912).
- Sea sighting filed under this port: Dive Boss Moana (C:12918).

## C. Visual identity (guidance only; commissions nothing; the character hold stands)

Pattern: the shared visual-identity headings 1-7 (design notes). Shared camera, envelope and do/don't blocks are in README.md and not repeated here.

1. Architecture. Identity line [invented]: The Storm Wall is the town: a breakwater of dead ships that keeps the living ones afloat. Landmark [cite]: "The Storm Wall - a massive breakwater built from the hulls of ships that didn't make it." (C:12186 area). Cited buildings: The Chained Harbor (harbor_master, C:11024); The Deep Reef (exchange, C:11034); The Storm Wall (governor, C:11044); Hine's Storm Grill (tavern, C:11054); The Wreck Yard (shipyard, C:11064); Wiremu's Steady Desk (broker, C:11074). Kit use [proposal]: compose from the shipped will-use plates; identity carriers are ground choice and building frequency, never new ids. Gaps follow the art gap list (design notes). Landmark canon: ../landmarks/storm_wall.md.
2. Materials: local and abundant (exports): Pearls, Timber, Spice; scarce or luxury (imports): Silk, Porcelain, Medicines, Rum, Pelts, Grain (section A). Atmosphere [cite]: "Wind that never fully dies, salt-crusted everything, and the pride of survival." (C:12186 area). Climate: section G.
3. Palette (references, not UI tokens; UI stays INK / GOLD / CREAM / MUTED / DUE). Judged in sRGB against the plate envelope (luma 65-141, chroma p98 35):

| Role | Family | sRGB | Luma / chroma | Status | Tie |
|---|---|---|---|---|---|
| Dominant | storm-wet timber | (92, 84, 76) #5C544C | 85 / 16 | in range | wreck hulls |
| Secondary 1 | typhoon sky grey | (104, 108, 112) #686C70 | 107 / 8 | in range | storms |
| Secondary 2 | reef stone | (110, 104, 96) #6E6860 | 105 / 14 | in range | breakwater base |
| Accent | storm-flag red, small | (128, 80, 72) #805048 | 90 / 56 | small-area accent (chroma 56) | storm flags on the wall |

4. Dress: see section B. 5. Signage: ASCII only, no UI-duplicating words; motifs [invented]: carved ship names. 6. Silhouette: chart scale is the shared `chart_port_marker`; harbour-scale skyline [invented]: a long dark breakwater of hulls around a small shipyard. 7. Do and don't: README shared list.

## D. Historical analogies (feed [invented] prose; never a cite)

- **Storm-coast salvage harbours, as a function.** Fits: a storm-battered harbour that builds its breakwater from wrecks (landmark). Suggests: carve your ship's name after a typhoon; salvage as honour. Limit: function only. Avoid: shipwreck romance cliches.

## E. Governance

- Title: Storm Chief (C:11018).
- Power structure: "Typhoon Anchorage is ruled by survival. Rangi leads because she's survived more. Koa chains ships. Moana dives deeper than anyone. Ahe builds from wreckage. Hine feeds through storms. Wiremu sells what the rest produce. Every institution exists to answer one question: can it survive a typhoon? If yes, it stays. If no, it's rebuilt." [ascii-folded] (C:11020).
- Internal tension: "The rivalry with Pearl Shallows is the deepest wound. Moana uses Abena's breathing technique - poached, improved, and wielded as proof of Storm Rider superiority. The fist-sized pearl she brought up from the deep reef is the physical evidence. Pearl Shallows hasn't spoken to them since. Within the Anchorage, the tension is simpler: Rangi glorifies survival. Ahe wants to eliminate the need for survival through better engineering. Both are right. The storm doesn't care about their debate." (C:11021).
- Bloc stance: "Typhoon Anchorage obeys the Coral King but takes pride in surviving what kills everyone else." (C:13698 area).
- Local rival: Pearl Shallows: "Pearl rivalries run deep. The Storm Riders dive deeper and in worse conditions. The Breath-Holders say they lack the sacred technique." (C:13698 area).
- Grudge: "Pearl Shallows accused Typhoon Anchorage of reef poaching. The Storm Riders responded by diving the deepest reef in the South Seas and bringing up a pearl the size of a fist. Case closed." (C:13698 area).
- Bloc philosophy: "Tribute earns trade. The reef remembers." (C:13511 area). Loyalty and spillover numbers are [flavour, no mechanic] (DF-15).

## F. Relations with every other port

Hand-maintained from content.json. Disposition is bloc-to-bloc (symmetric in data); lanes are undirected; flows are directional by market affinity; networks are symmetric.

<!-- hand-maintained:begin -->
| Port | Region | Bloc | Disposition | Lane (distance, min class, danger) | Flows out (they import) | Flows in (we import) | Network |
|---|---|---|---|---|---|---|---|
| Porto Novo | Mediterranean | The Exchange Alliance | neutral (C:13753) | - | Timber | Grain, Rum | - |
| Al-Manar | Mediterranean | The Exchange Alliance | neutral (C:13753) | - | - | Medicines | - |
| Silva Bay | Mediterranean | The Exchange Alliance | neutral (C:13753) | - | - | Pelts | - |
| Corsair's Rest | Mediterranean | The Shadow Ports | neutral (C:13858) | - | - | Pelts, Rum | - |
| Ironhaven | North Atlantic | The Iron Pact | neutral (C:13788) | - | Timber | Pelts | - |
| Stormwall | North Atlantic | The Iron Pact | neutral (C:13788) | - | - | Grain, Medicines | - |
| Thornport | North Atlantic | The Free Ports | neutral (C:13851) | - | Spice | Pelts | - |
| Sun Harbor | West Africa | The Gold Coast Compact | allied (C:13816) | - | Spice | Pelts | - |
| Palm Cove | West Africa | The Gold Coast Compact | allied (C:13816) | - | Timber | Rum | - |
| Iron Point | West Africa | The Gold Coast Compact | allied (C:13816) | - | Timber | - | - |
| Pearl Shallows | West Africa | The Gold Coast Compact | allied (C:13816) | - | - | - | - |
| Jade Port | East Indies | The Silk Circle | allied (C:13837) | - | - | Porcelain, Silk | - |
| Monsoon Reach | East Indies | The Free Ports | neutral (C:13851) | 52, galleon, 0.2 (C:2225) | Timber | Silk | - |
| Silk Haven | East Indies | The Silk Circle | allied (C:13837) | - | Spice | Porcelain, Silk | - |
| Crosswind Isle | East Indies | The Free Ports | neutral (C:13851) | - | - | - | - |
| Dragon's Gate | East Indies | The Silk Circle | allied (C:13837) | - | - | Porcelain | - |
| Spice Narrows | East Indies | The Shadow Ports | neutral (C:13858) | - | - | - | - |
| Ember Isle | South Seas | The Coral Crown | same bloc | 24, sloop, 0.14 (C:2252) | Timber | Medicines | C:11580 |
| Coral Throne | South Seas | The Coral Crown | same bloc | 22, brigantine, 0.13 (C:2270) | - | Rum | - |
<!-- hand-maintained:end -->

- Lane lore: "Typhoon Alley": "Named for obvious reasons. The monsoon winds funnel through here like a gauntlet. Timing is survival." (C:2225). Lane canon: [Typhoon Alley](../lanes/typhoon_alley.md).

## G. World details (undefined in data; all [invented] unless cited)

- History (shared timeline in README.md): T14: The Storm Wall reaches its present length from wrecked hulls [invented]. T16: The Storm Riders dive Pearl Shallows' reef (grudge text).
- Population [invented]: about 3,500.
- Language [invented]: Reef Kingdoms trade speech.
- Currency: silver is the only money in the sim (every price, fee and wage is in silver). Local coin names, if any, are [invented] flavour and never replace 'silver' in copy.
- Religion [invented]: names carved into the Storm Wall honour survivors.
- Climate [invented port detail]: typhoon season every year; violent storms. Regional weather [cite]: "Turquoise water so clear you can see the reef below. Beautiful and deadly." (C:11950 area, weather_flavor).
- Pirates in these waters [cite]: The Deep Reef Brotherhood (C:2896).

## I. Sound and ambience (all [invented] sound design unless quoted from a cite)

- **Soundscape:** "Wind that never fully dies" (C:12189); ships "chained, not anchored" (C:11028) - chains straining; leeward cliff shelter.
- **Signature:** mooring chain strain/snatch; salt-crusted timber creak; knife carving into the Storm Wall (custom).
- **Music/work songs (NOTE ONLY):** none; stories as price of entry are voice - on hold.
- **Vs neighbours:** constant wind + chain; vs Dragon's Gate chains (harbour boom, controlled) these are survival moorings.
- **Fragments [proposal; no fragment or asset exists yet]:** `src.mooring_chain_strain`, `src.carve_name`, `place.leeward_cliff`.

---
- Cited hooks: atmosphere "Wind that never fully dies, salt-crusted everything, and the pride of survival." (C:12189); dock scene "The harbor is carved into the leeward cliff. Ships are chained, not anchored - anchors aren't enough here. Pearl divers work the outer reef between storms." (C:12190).
- Invented bed idea: surf against the wall, carving tools, wind. No assets are requested.

## J. Flags for this port

- none beyond the shared ones in data-fixes.md.
