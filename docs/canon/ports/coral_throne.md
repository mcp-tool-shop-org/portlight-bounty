# Coral Throne (coral_throne)

Base commit: 143e284 (content.json unchanged since 45b7378). Status: draft canon, phase 1. Written 2026-10-14.
Cites `C:n` are crates/portlight-sim/data/content.json line n at 143e284. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] (design-notes pattern) | DF-n see data-fixes.md | T-n see README shared timeline.
Depth: one of the five thinnest ports in data (inventory 'Undefined'); section H adds extra invented depth.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Region | South Seas ("The Reef Kingdoms") | C:1839, C:11950 |
| Bloc | The Coral Crown (Ember Isle, Typhoon Anchorage, Coral Throne) | C:13707, C:13511 |
| Features | none | C:1839-1923 |
| Warehouse tiers | none listed | C:6520 |
| Port fee / provisions / repair / crew | 7 / 1 / 4 per hull point / 5 | C:1839-1923 |
| Map | x 44, y 26 | C:1923 |
| Lanes | 3 | section F |
| Player description | "Island kingdom built on coral reefs. The king trades pearls for weapons and demands tribute in silk." | C:1841 |

## A. Specialties and trade goods

<!-- hand-maintained:begin -->
| Good | Category | Base | Affinity | Stock target | Restock | Role |
|---|---|---|---|---|---|---|
| Pearls | luxury | 95 | 1.5 | 10 | 1.5 | exporter |
| Tobacco | provision | 28 | 1.4 | 18 | 2.0 | exporter |
| Rum | provision | 20 | 1.3 | 15 | 2.0 | exporter |
| Pelts | commodity | 8 | 0.7 | 8 | 0.8 | importer |
| Grain | commodity | 12 | 0.55 | 6 | 0.5 | importer |
| Tea | luxury | 40 | 0.5 | 5 | 0.5 | importer |
| Porcelain | luxury | 60 | 0.45 | 4 | 0.3 | importer |
| Weapons | military | 45 | 0.4 | 4 | 0.3 | importer |
| Silk | luxury | 70 | 0.4 | 4 | 0.3 | importer |
<!-- hand-maintained:end -->

Market cite: C:1839-1923. Price rule: base x (target / stock) / affinity (crates/portlight-sim/src/economy.rs:37); affinity above 1 is cheap here.
Not traded here: Timber, Iron Ore, Cotton, Dyes, Spice, Medicines, Opium, Black Powder, Stolen Cargo.
Region goods [flavour, no mechanic]: sacred Pearls; prized Weapons, Silk, Tea; forbidden none (C:11950 area).
Festival: Coral Coronation, every 90 days for 5, market effects Weapons x2.5, Silk x2.0, Pearls x1.8 (C:11975).
Festival: Fire Walking, every 75 days for 3, market effects Medicines x2.0, Dyes x1.8 (C:11989).
Contract templates touching South Seas: `proc_grain_feed` (procurement), `proc_timber_shipyard` (procurement), `proc_medicines_relief` (procurement), `proc_weapons_garrison` (procurement), `circ_south_seas` (circuit), `smug_powder_delivery` (smuggling). Source port Coral Throne: none.

## B. People

- Cultural group: The Coral Court: "The island kingdom's royal court. Tribute buys access. Generosity buys favor." (C:12196 area).
- Local custom: "All trade begins with tribute to the Coral King. Silk or weapons preferred. Refusal is exile." (C:12196 area).
- Regional ethos: "The sea gives life and the sea takes it back. The island kings answer to no empire. Trade here is tribute - offered willingly or not at all. Respect the reef, or it will tear your hull apart." Greeting: "The Coral King permits you to anchor. State your tribute." Proverb: "The reef welcomes the patient; it drowns the greedy." (C:11950 area).
- Tavern rumour (in-world, unreliable): "The Coral King has no heir. Three princes compete for the throne, each courting foreign captains for weapons and alliances. Smart traders play all three." (C:12196 area).

| Name | Title | Temperament | Institution | Cite |
|---|---|---|---|---|
| The Coral King | King | imperious | governor | C:11282 |
| Reef Pilot Iti | Reef Pilot | proud | harbor_master | C:11303 |
| Pearl Trader Hana | Royal Pearl Trader | shrewd | exchange | C:11324 |
| War Chief Tane | War Chief | fierce | customs | C:11345 |
| Drum Keeper Miri | Keeper of Traditions | reverent | tavern | C:11366 |
| Broker Ariki | Royal Broker | diplomatic | broker | C:11387 |
| Tane | Warrior Outfitter (merchant) | gruff | outfitter | C:6046 |
| Kahoku the Elder | style master | - | - | C:6237 |

- Dress [invented]: court dress in woven fibre and pearl; tribute silk worn only by the king's household. The character hold stands: no likenesses or portraits.
- Speech [cite]: built from the greeting lines in each NPC record above (greeting_neutral, friendly, hostile).
- Temperament of the town: read from the power structure and tension in section E.
- Network (broker, allied): Broker Ariki - Broker Tia, linking Ember Isle (C:11640). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Sea sighting filed under this port: Reef Pilot Iti (C:12924).
- Sea sighting filed under this port: War Chief Tane (C:12930).

## C. Visual identity (guidance only; commissions nothing; the character hold stands)

Pattern: the shared visual-identity headings 1-7 (design notes). Shared camera, envelope and do/don't blocks are in README.md and not repeated here.

1. Architecture. Identity line [invented]: The Palace is the kingdom: Coral Throne is the Coral King's court with a harbour attached. Landmark [cite]: "The Coral Palace - grown over centuries from living reef, rising from the lagoon like a crown." (C:12196 area). Cited buildings: The Lagoon Entrance (harbor_master, C:11220); The Pearl Hall (exchange, C:11230); The Coral Palace (governor, C:11240); The Drum Circle (tavern, C:11250); The War Canoe Dock (customs, C:11260); The Royal Trading Post (broker, C:11270). Kit use [proposal]: compose from the shipped will-use plates; identity carriers are ground choice and building frequency, never new ids. Gaps follow the art gap list (design notes).
2. Materials: local and abundant (exports): Pearls, Tobacco, Rum; scarce or luxury (imports): Weapons, Silk, Tea, Grain, Pelts, Porcelain (section A). Atmosphere [cite]: "Warm lagoon water, crushed coral underfoot, and the distant beat of ceremonial drums." (C:12196 area). Climate: section G.
3. Palette (references, not UI tokens; UI stays INK / GOLD / CREAM / MUTED / DUE). Judged in sRGB against the plate envelope (luma 65-141, chroma p98 35):

| Role | Family | sRGB | Luma / chroma | Status | Tie |
|---|---|---|---|---|---|
| Dominant | pale reef stone | (132, 126, 112) #847E70 | 126 / 20 | in range | coral reefs |
| Secondary 1 | lagoon shallows, muted | (92, 112, 108) #5C706C | 107 / 20 | in range | the lagoon |
| Secondary 2 | woven fibre | (118, 104, 82) #766852 | 105 / 36 | mood, not values | houses |
| Accent | pearl and tribute silk, small | (148, 136, 120) #948878 | 137 / 28 | in range | pearls and tribute |

4. Dress: see section B. 5. Signage: ASCII only, no UI-duplicating words; motifs [invented]: a crown of coral branches. 6. Silhouette: chart scale is the shared `chart_port_marker`; harbour-scale skyline [invented]: a palace of grown coral rising from the lagoon, smaller reef houses ringed around it. 7. Do and don't: README shared list.

## D. Historical analogies (feed [invented] prose; never a cite)

- **Tribute courts, as an institution.** Fits: a court that requires tribute before trade (custom). Suggests: tribute first, then business; refusal is exile. Limit: institution only; not any real people (analogy rule 1). Avoid: exotic-kingdom costume.
- **Reef-built lagoon towns, as architecture.** Fits: a palace grown from living reef (landmark). Suggests: building as slow cultivation; the reef pilot as gatekeeper. Limit: architecture only. Avoid: no real place named.
- **Ceremonial drum courts, as a calendar.** Fits: a Drum Keeper keeps traditions (NPC). Suggests: drums announce tribute, judgments and arrivals. Limit: calendar function only.
- Contrast, not model: Crosswind Isle has no master; Coral Throne has nothing else.

## E. Governance

- Title: King (C:11214).
- Power structure: "Absolute monarchy. The Coral King's authority is total within the lagoon. Iti controls reef access. Tane commands the military. Hana manages pearls. Miri maintains culture. Ariki handles commerce. The King sits at the center, wearing a crown that is literally growing. Every institution radiates from the Palace like spokes from a hub." [ascii-folded] (C:11216).
- Internal tension: "The succession. The King has no heir, and three princes compete. Each courts foreign captains for weapons. Each visits Drum Keeper Miri seeking her endorsement - because the coronation music determines which prince the gods favor. Miri plays each the same neutral rhythm. She plays a fourth rhythm alone at midnight on the reef. Nobody knows what it means. The succession crisis isn't violent - yet. But when the King dies, three princes, their foreign alliances, and Miri's drums will determine whether the Coral Crown continues or shatters." (C:11217).
- Bloc stance: "The Coral Throne IS the Crown. The king's word shapes trade from Ember Isle to Typhoon Anchorage." (C:13707 area).
- Grudge: "A Western merchant refused to pay tribute. The Coral King displayed his ship's nameplate in the palace as a decoration. The merchant swam home." (C:13707 area).
- Bloc philosophy: "Tribute earns trade. The reef remembers." (C:13511 area). Loyalty and spillover numbers are [flavour, no mechanic] (DF-15).

## F. Relations with every other port

Hand-maintained from content.json. Disposition is bloc-to-bloc (symmetric in data); lanes are undirected; flows are directional by market affinity; networks are symmetric.

<!-- hand-maintained:begin -->
| Port | Region | Bloc | Disposition | Lane (distance, min class, danger) | Flows out (they import) | Flows in (we import) | Network |
|---|---|---|---|---|---|---|---|
| Porto Novo | Mediterranean | The Exchange Alliance | neutral (C:13753) | - | - | Grain | - |
| Al-Manar | Mediterranean | The Exchange Alliance | neutral (C:13753) | - | Rum | - | - |
| Silva Bay | Mediterranean | The Exchange Alliance | neutral (C:13753) | - | Rum | Pelts | - |
| Corsair's Rest | Mediterranean | The Shadow Ports | neutral (C:13858) | - | - | Pelts, Weapons | - |
| Ironhaven | North Atlantic | The Iron Pact | neutral (C:13788) | - | Rum, Tobacco | Pelts, Weapons | - |
| Stormwall | North Atlantic | The Iron Pact | neutral (C:13788) | - | Rum, Tobacco | Grain, Weapons | - |
| Thornport | North Atlantic | The Free Ports | neutral (C:13851) | - | - | Pelts, Tea | - |
| Sun Harbor | West Africa | The Gold Coast Compact | allied (C:13816) | - | Rum | Pelts | - |
| Palm Cove | West Africa | The Gold Coast Compact | allied (C:13816) | - | - | - | - |
| Iron Point | West Africa | The Gold Coast Compact | allied (C:13816) | - | Rum | - | - |
| Pearl Shallows | West Africa | The Gold Coast Compact | allied (C:13816) | 68, galleon, 0.2 (C:2306) | Rum | - | - |
| Jade Port | East Indies | The Silk Circle | allied (C:13837) | - | Rum | Porcelain, Silk, Tea | - |
| Monsoon Reach | East Indies | The Free Ports | neutral (C:13851) | - | Rum | Silk | - |
| Silk Haven | East Indies | The Silk Circle | allied (C:13837) | - | Rum | Porcelain, Silk | - |
| Crosswind Isle | East Indies | The Free Ports | neutral (C:13851) | - | - | - | - |
| Dragon's Gate | East Indies | The Silk Circle | allied (C:13837) | - | Rum, Tobacco | Porcelain, Tea | - |
| Spice Narrows | East Indies | The Shadow Ports | neutral (C:13858) | - | Rum | - | - |
| Ember Isle | South Seas | The Coral Crown | same bloc | 28, sloop, 0.15 (C:2261) | Rum | - | C:11640 |
| Typhoon Anchorage | South Seas | The Coral Crown | same bloc | 22, brigantine, 0.13 (C:2270) | Rum | - | - |
<!-- hand-maintained:end -->

- Lane lore: "The Deep South Run": "Pearl to pearl. The divers of the shallows and the divers of the reef - same craft, different kingdoms." (C:2306).

## G. World details (undefined in data; all [invented] unless cited)

- History (shared timeline in README.md): T2: The Coral Palace begins to grow (landmark: 'grown over centuries'). T12: The Coral Crown bloc forms with Ember Isle and Typhoon Anchorage [invented date]. T15: A Western merchant's nameplate hangs in the palace after he refused tribute (grudge text).
- Population [invented]: about 6,000 across the reef islands.
- Language [invented]: Reef Kingdoms trade speech; a court speech only the king's household uses.
- Currency: silver is the only money in the sim (every price, fee and wage is in silver). Local coin names, if any, are [invented] flavour and never replace 'silver' in copy.
- Religion [invented]: the Coral King is the reef's voice; drums mark rites.
- Climate [invented port detail]: warm lagoon, calm inside the reef, storms outside. Regional weather [cite]: "Turquoise water so clear you can see the reef below. Beautiful and deadly." (C:11950 area, weather_flavor).
- Pirates in these waters [cite]: The Deep Reef Brotherhood (C:2896).

## H. Extra depth (thin port)

- Daily life [invented]: tribute is received each morning at the palace steps to the sound of drums; trade opens after.
- Factions [invented]: the king's household, the war chief's guard and the reef pilots. The pilots are quietly the most powerful, because no ship enters the lagoon without one.
- A local law [invented]: a refused tribute ends in exile from the lagoon (custom, cited), never in a fight in harbour.

## I. Sound and ambience (all [invented] sound design unless quoted from a cite)

- **Soundscape:** lagoon calm; "the distant beat of ceremonial drums"; crushed coral underfoot; war-canoe escort paddles.
- **Signature:** footstep on crushed coral; war-canoe paddle strokes in unison; tribute chest set before the court.
- **Music/work songs (NOTE ONLY):** ceremonial drums are canon - strongest drum note after Stormwall; keep distinct (royal, slow) if music opens.
- **Vs neighbours:** ceremonial court; vs Ember Isle's geology, vs Typhoon's wind.
- **Fragments [proposal; no fragment or asset exists yet]:** `mat.crushed_coral`, `src.war_canoe_paddles`, `place.lagoon`.
- Cited hooks: atmosphere "Warm lagoon water, crushed coral underfoot, and the distant beat of ceremonial drums."; dock scene "The lagoon entrance is narrow and treacherous - local pilots guide every ship through the reef. The Coral Palace rises ahead, its walls glittering with embedded shells and pearls. Warriors in war canoes escort you to the royal dock." (C:12196 area).
- Invented bed idea: drums from the palace, surf on the outer reef. No assets are requested.

## J. Flags for this port

- none beyond the shared ones in data-fixes.md.
