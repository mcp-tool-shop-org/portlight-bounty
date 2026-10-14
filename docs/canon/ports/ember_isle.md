# Ember Isle (ember_isle)

Base commit: 143e284 (content.json unchanged since 45b7378). Status: draft canon, phase 1. Written 2026-10-14.
Cites `C:n` are crates/portlight-sim/data/content.json line n at 143e284. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] (design-notes pattern) | DF-n see data-fixes.md | T-n see README shared timeline.
Depth: one of the five thinnest ports in data (inventory 'Undefined'); section H adds extra invented depth.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Region | South Seas ("The Reef Kingdoms") | C:1663, C:11950 |
| Bloc | The Coral Crown (Ember Isle, Typhoon Anchorage, Coral Throne) | C:13689, C:13511 |
| Features | none | C:1663-1747 |
| Warehouse tiers | none listed | C:6520 |
| Port fee / provisions / repair / crew | 6 / 2 / 4 per hull point / 5 | C:1663-1747 |
| Map | x 34, y 28 | C:1747 |
| Lanes | 4 | section F |
| Player description | "Volcanic island with obsidian beaches. Rich in rare minerals and medicinal plants." | C:1665 |

## A. Specialties and trade goods

<!-- hand-maintained:begin -->
| Good | Category | Base | Affinity | Stock target | Restock | Role |
|---|---|---|---|---|---|---|
| Medicines | medicine | 35 | 1.6 | 20 | 2.5 | exporter |
| Dyes | commodity | 22 | 1.5 | 18 | 2.0 | exporter |
| Iron Ore | commodity | 25 | 1.2 | 12 | 1.5 | exporter |
| Pelts | commodity | 8 | 0.7 | 8 | 0.8 | importer |
| Rum | provision | 20 | 0.6 | 8 | 0.8 | importer |
| Cotton | commodity | 15 | 0.55 | 6 | 0.5 | importer |
| Grain | commodity | 12 | 0.5 | 8 | 0.5 | importer |
| Timber | commodity | 18 | 0.5 | 8 | 0.5 | importer |
| Weapons | military | 45 | 0.5 | 5 | 0.5 | importer |
<!-- hand-maintained:end -->

Market cite: C:1663-1747. Price rule: base x (target / stock) / affinity (crates/portlight-sim/src/economy.rs:37); affinity above 1 is cheap here.
Not traded here: Spice, Silk, Porcelain, Tea, Pearls, Tobacco, Opium, Black Powder, Stolen Cargo.
Region goods [flavour, no mechanic]: sacred Pearls; prized Weapons, Silk, Tea; forbidden none (C:11950 area).
Festival: Coral Coronation, every 90 days for 5, market effects Weapons x2.5, Silk x2.0, Pearls x1.8 (C:11975).
Festival: Fire Walking, every 75 days for 3, market effects Medicines x2.0, Dyes x1.8 (C:11989).
Contract templates touching South Seas: `proc_grain_feed` (procurement), `proc_timber_shipyard` (procurement), `proc_medicines_relief` (procurement), `proc_weapons_garrison` (procurement), `circ_south_seas` (circuit), `smug_powder_delivery` (smuggling). Source port Ember Isle: none.

## B. People

- Cultural group: The Ember Keepers: "Herbalists and healers who know the volcanic plants. Their medicines are unmatched." (C:12176 area).
- Local custom: "Visitors place a stone at the volcano's base. It's said to appease the mountain." (C:12176 area).
- Regional ethos: "The sea gives life and the sea takes it back. The island kings answer to no empire. Trade here is tribute - offered willingly or not at all. Respect the reef, or it will tear your hull apart." Greeting: "The Coral King permits you to anchor. State your tribute." Proverb: "The reef welcomes the patient; it drowns the greedy." (C:11950 area).
- Tavern rumour (in-world, unreliable): "The volcano speaks more often now. The islanders aren't worried - they say it always grumbles before a good harvest of medicine plants." (C:12176 area).

| Name | Title | Temperament | Institution | Cite |
|---|---|---|---|---|
| Elder Kai | Keeper Elder | mystical | governor | C:10890 |
| Harbor Chief Manu | Harbor Chief | watchful | harbor_master | C:10911 |
| Head Herbalist Sera | Head Herbalist | passionate | exchange | C:10932 |
| Dye Keeper Lono | Dye Master | artistic | shipyard | C:10953 |
| Spring Keeper Olo | Hot Springs Guardian | serene | tavern | C:10974 |
| Broker Tia | Island Broker | dedicated | broker | C:10995 |

- Dress [invented]: woven plant-fibre wraps, green-stained hands among herbalists. The character hold stands: no likenesses or portraits.
- Speech [cite]: built from the greeting lines in each NPC record above (greeting_neutral, friendly, hostile).
- Temperament of the town: read from the power structure and tension in section E.
- Network (merchant, allied): Herbalist Sera - Dr. Halvorsen, linking Stormwall (C:11472). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Network (tavern, personal): Cook Hine - Spring Keeper Olo, linking Typhoon Anchorage (C:11580). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Network (broker, allied): Broker Ariki - Broker Tia, linking Coral Throne (C:11640). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Sea sighting filed under this port: Healer Sera (C:12936).

## C. Visual identity (guidance only; commissions nothing; the character hold stands)

Pattern: the shared visual-identity headings 1-7 (design notes). Shared camera, envelope and do/don't blocks are in README.md and not repeated here.

1. Architecture. Identity line [invented]: The mountain is the town: Ember Isle lives by what the volcano gives. Landmark [cite]: "Ember Peak - the smoldering volcano visible from fifty leagues, its glow a navigator's beacon." (C:12176 area). Cited buildings: The Black Sand Harbor (harbor_master, C:10828); The Volcanic Garden (exchange, C:10838); The Hot Springs Throne (governor, C:10848); The Healing Springs (tavern, C:10858); The Pigment Cave (shipyard, C:10868); The Export Hut (broker, C:10878). Kit use [proposal]: compose from the shipped will-use plates; identity carriers are ground choice and building frequency, never new ids. Gaps follow the art gap list (design notes). Landmark canon: ../landmarks/ember_peak.md.
2. Materials: local and abundant (exports): Medicines, Dyes, Iron Ore; scarce or luxury (imports): Grain, Timber, Weapons, Rum, Pelts, Cotton (section A). Atmosphere [cite]: "Sulfur, tropical flowers, and the low rumble of a volcano that never fully sleeps." (C:12176 area). Climate: section G.
3. Palette (references, not UI tokens; UI stays INK / GOLD / CREAM / MUTED / DUE). Judged in sRGB against the plate envelope (luma 65-141, chroma p98 35):

| Role | Family | sRGB | Luma / chroma | Status | Tie |
|---|---|---|---|---|---|
| Dominant | obsidian beach, lifted | (72, 70, 72) #484648 | 71 / 2 | in range | obsidian beaches |
| Secondary 1 | volcanic soil | (98, 82, 72) #625248 | 85 / 26 | in range | slopes |
| Secondary 2 | herb green, muted | (82, 100, 78) #52644E | 95 / 22 | in range | medicinal plants |
| Accent | ember glow, small | (138, 100, 74) #8A644A | 106 / 64 | small-area accent (chroma 64) | Ember Peak |

4. Dress: see section B. 5. Signage: ASCII only, no UI-duplicating words; motifs [invented]: a stone cairn. 6. Silhouette: chart scale is the shared `chart_port_marker`; harbour-scale skyline [invented]: the smoking cone above black beaches and terraced herb gardens. 7. Do and don't: README shared list.

## D. Historical analogies (feed [invented] prose; never a cite)

- **Volcanic healing islands with hot springs, as an economy.** Fits: a volcanic island of medicinal plants, dyes and minerals (Medicines 1.6, Dyes 1.5). Suggests: stones left at the mountain's base; healing as duty. Limit: function only. Avoid: no real islands or peoples named.
- **Monastic herbal infirmaries, as an institution.** Fits: healers who keep recipes and sell at fair prices (Sera's agenda). Suggests: recipes as trust, never sold to a power. Limit: institution form only. Avoid: religious caricature.
- **Obsidian trade sites, as a material economy.** Fits: black glass as a traded material on the beaches. Suggests: tools and mirrors of obsidian [invented]. Limit: [flavour, no mechanic]: no obsidian good in the sim.
- Contrast, not model: Ironhaven's foundry fire is made; Ember Isle's fire is given.

## E. Governance

- Title: Keeper Elder (C:10822).
- Power structure: "Ember Isle is governed by the mountain - through Kai, who interprets what the volcano provides. Sera harvests medicines. Lono creates dyes. Olo guards the springs. Manu manages the beach. Tia sells to the world. The power structure is spiritual: the mountain gives, the Keepers receive, and outsiders are guests." [ascii-folded] (C:10824).
- Internal tension: "The volcano speaks more often. Kai says it grumbles before a good harvest. Others aren't sure. If the mountain is building toward eruption, Ember Isle's entire existence is at risk - and the Keepers would be the last to leave because leaving would mean abandoning the medicine plants that only grow here. Sera's quest for the missing fever-cure ingredient is the island's scientific hope: if she completes the compound, it could justify the Keepers' existence even without the mountain." (C:10825).
- Bloc stance: "Ember Isle serves the Crown but keeps its own counsel. The volcano doesn't care about politics." (C:13689 area).
- Grudge: "The Ember Keepers refused to share their medicine recipes with the Iron Pact. 'The mountain teaches us. It didn't teach you.'" (C:13689 area).
- Bloc philosophy: "Tribute earns trade. The reef remembers." (C:13511 area). Loyalty and spillover numbers are [flavour, no mechanic] (DF-15).
- Faction canon: bloc [The Coral Crown](../factions/coral_crown.md); pirate faction [The Deep Reef Brotherhood](../factions/deep_reef_brotherhood.md).

## F. Relations with every other port

Hand-maintained from content.json. Disposition is bloc-to-bloc (symmetric in data); lanes are undirected; flows are directional by market affinity; networks are symmetric.

<!-- hand-maintained:begin -->
| Port | Region | Bloc | Disposition | Lane (distance, min class, danger) | Flows out (they import) | Flows in (we import) | Network |
|---|---|---|---|---|---|---|---|
| Porto Novo | Mediterranean | The Exchange Alliance | neutral (C:13753) | - | Dyes | Grain, Rum | - |
| Al-Manar | Mediterranean | The Exchange Alliance | neutral (C:13753) | - | Iron Ore | - | - |
| Silva Bay | Mediterranean | The Exchange Alliance | neutral (C:13753) | - | - | Pelts, Timber | - |
| Corsair's Rest | Mediterranean | The Shadow Ports | neutral (C:13858) | - | Medicines | Pelts, Rum, Weapons | - |
| Ironhaven | North Atlantic | The Iron Pact | neutral (C:13788) | - | - | Pelts, Weapons | - |
| Stormwall | North Atlantic | The Iron Pact | neutral (C:13788) | - | - | Grain, Weapons | C:11472 |
| Thornport | North Atlantic | The Free Ports | neutral (C:13851) | - | Medicines | Pelts, Timber | - |
| Sun Harbor | West Africa | The Gold Coast Compact | allied (C:13816) | - | - | Cotton, Pelts | - |
| Palm Cove | West Africa | The Gold Coast Compact | allied (C:13816) | - | Iron Ore | Cotton, Rum | - |
| Iron Point | West Africa | The Gold Coast Compact | allied (C:13816) | - | Medicines | - | - |
| Pearl Shallows | West Africa | The Gold Coast Compact | allied (C:13816) | - | Medicines | Cotton | - |
| Jade Port | East Indies | The Silk Circle | allied (C:13837) | - | Iron Ore | - | - |
| Monsoon Reach | East Indies | The Free Ports | neutral (C:13851) | - | Medicines | - | - |
| Silk Haven | East Indies | The Silk Circle | allied (C:13837) | - | Dyes, Iron Ore | - | - |
| Crosswind Isle | East Indies | The Free Ports | neutral (C:13851) | 56, galleon, 0.19 (C:2243) | - | - | - |
| Dragon's Gate | East Indies | The Silk Circle | allied (C:13837) | - | Iron Ore, Medicines | - | - |
| Spice Narrows | East Indies | The Shadow Ports | neutral (C:13858) | 48, galleon, 0.18 (C:2234) | - | - | - |
| Typhoon Anchorage | South Seas | The Coral Crown | same bloc | 24, sloop, 0.14 (C:2252) | Medicines | Timber | C:11580 |
| Coral Throne | South Seas | The Coral Crown | same bloc | 28, sloop, 0.15 (C:2261) | - | Rum | C:11640 |
<!-- hand-maintained:end -->

- Lane lore: "The Volcanic Passage": "Warm currents and sulfur air. The locals say the sea boils near the islands on bad days." (C:2234). Lane canon: [The Volcanic Passage](../lanes/volcanic_passage.md).

## G. World details (undefined in data; all [invented] unless cited)

- History (shared timeline in README.md): T2: The Ember Keepers settle the slopes [invented date]. T11: The Keepers refuse the Iron Pact's medicine exclusivity (grudge text). T16: The mountain grumbles more often (internal tension); Kai reads it as a harvest sign.
- Population [invented]: about 1,500.
- Language [invented]: Reef Kingdoms trade speech; plant names in an old island tongue.
- Currency: silver is the only money in the sim (every price, fee and wage is in silver). Local coin names, if any, are [invented] flavour and never replace 'silver' in copy.
- Religion [invented]: the mountain as provider; Keepers interpret it.
- Climate [invented port detail]: warm, humid, ash on the wind when the mountain speaks. Regional weather [cite]: "Turquoise water so clear you can see the reef below. Beautiful and deadly." (C:11950 area, weather_flavor).
- Pirates in these waters [cite]: The Deep Reef Brotherhood (C:2896).

## H. Extra depth (thin port)

- Daily life [invented]: herb gathering at dawn, dye vats in the afternoon, the springs at dusk.
- Factions [invented]: the Keepers of the mountain, the herbalists and the harbour. Kai's reading of the volcano outranks trade every time.
- A local law [invented]: no medicine recipe is written down for an outsider; captains carry finished medicine, never the knowledge.

## I. Sound and ambience (all [invented] sound design unless quoted from a cite)

- **Soundscape:** "the low rumble of a volcano that never fully sleeps" (C:12179); hot-spring steam vents.
- **Signature:** steam vent hiss; obsidian scree underfoot; stone placed at the volcano's base (custom).
- **Sea mark:** "Ember Peak's volcanic glow marks the horizon. The South Seas await." (crates/portlight-sim/src/voyage.rs:597); see ../landmarks/ember_peak.md
- **Music/work songs (NOTE ONLY):** none in canon.
- **Vs neighbours:** the only geological bed (sub rumble); vs Coral Throne's drums and Typhoon's wind.
- **Fragments [proposal; no fragment or asset exists yet]:** `src.steam_vent`, `mat.obsidian_scree`, `src.stone_placed`.
- Cited hooks: atmosphere "Sulfur, tropical flowers, and the low rumble of a volcano that never fully sleeps." (C:12179); dock scene "Black sand beaches and obsidian-sharp rocks frame the harbor. Steam rises from hot springs near the dock. Herbalists sort medicinal plants in the warm volcanic soil." (C:12180).
- Invented bed idea: the mountain's rumble, hot springs hissing. No assets are requested.

## J. Flags for this port

- DF-4: Institutions of type `shipyard` at ports without the shipyard feature
- DF-8: NPC name drift across records
