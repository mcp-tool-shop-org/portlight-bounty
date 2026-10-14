# Spice Narrows (spice_narrows)

Base commit: 143e284 (content.json unchanged since 45b7378). Status: draft canon, phase 1. Written 2026-10-14.
Cites `C:n` are crates/portlight-sim/data/content.json line n at 143e284. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] (design-notes pattern) | DF-n see data-fixes.md | T-n see README shared timeline.
Depth: one of the five thinnest ports in data (inventory 'Undefined'); section H adds extra invented depth.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Region | East Indies ("The Silk Waters") | C:1566, C:11892 |
| Bloc | The Shadow Ports (Corsair's Rest, Spice Narrows) | C:13680, C:13493 |
| Features | black_market | C:1566-1660 |
| Warehouse tiers | none listed | C:6520 |
| Port fee / provisions / repair / crew | 5 / 3 / 5 per hull point / 6 | C:1566-1660 |
| Map | x 38, y 20 | C:1660 |
| Lanes | 5 | section F |
| Player description | "Hidden anchorage in the spice archipelago. The most concentrated spice market in the world." | C:1568 |

## A. Specialties and trade goods

<!-- hand-maintained:begin -->
| Good | Category | Base | Affinity | Stock target | Restock | Role |
|---|---|---|---|---|---|---|
| Spice | luxury | 55 | 1.8 | 40 | 4.0 | exporter |
| Opium | contraband | 85 | 1.4 | 8 | 0.5 | exporter |
| Pearls | luxury | 95 | 1.3 | 6 | 1.0 | exporter |
| Stolen Cargo | contraband | 40 | 1.0 | 6 | 0.8 | neutral |
| Silk | luxury | 70 | 0.9 | 12 | 1.0 | importer |
| Pelts | commodity | 8 | 0.8 | 6 | 0.8 | importer |
| Rum | provision | 20 | 0.6 | 8 | 0.8 | importer |
| Black Powder | contraband | 65 | 0.6 | 4 | 0.3 | importer |
| Weapons | military | 45 | 0.5 | 5 | 0.5 | importer |
| Grain | commodity | 12 | 0.5 | 8 | 0.5 | importer |
<!-- hand-maintained:end -->

Market cite: C:1566-1660. Price rule: base x (target / stock) / affinity (crates/portlight-sim/src/economy.rs:37); affinity above 1 is cheap here.
Not traded here: Timber, Iron Ore, Cotton, Dyes, Porcelain, Tea, Tobacco, Medicines.
Region goods [flavour, no mechanic]: sacred Porcelain, Silk; prized Spice, Tea; forbidden Weapons (C:11892 area).
Festival: Silk Weavers' Fair, every 75 days for 4, market effects Silk x2.0, Porcelain x1.5, Dyes x1.4 (C:11919).
Festival: Monsoon Prayer, every 90 days for 3, market effects Spice x1.8, Medicines x1.6, Tea x1.4 (C:11933).
Contract templates touching East Indies: `proc_grain_feed` (procurement), `proc_timber_shipyard` (procurement), `proc_iron_forge` (procurement), `proc_medicines_relief` (procurement), `lux_spice_monopolist` (luxury_discreet), `ret_cotton_return` (return_freight), `ret_spice_restock` (return_freight), `circ_indies_loop` (circuit), `smug_opium_run` (smuggling), `smug_powder_delivery` (smuggling). Source port Spice Narrows: `smug_opium_run`.

## B. People

- Cultural group: The Spice Lords: "They control access to the richest spice grounds in the world. Cross them once." (C:12166 area).
- Local custom: "Prices are whispered, never spoken aloud. The walls have ears and the water carries sound." (C:12166 area).
- Regional ethos: "Patience is mastery. The artisan who perfects one bolt of silk in a lifetime is honored above the merchant who trades a thousand. Everything here was old when your civilization was young." Greeting: "You honor us, Captain. Tea will be prepared while we discuss terms." Proverb: "A merchant who rushes drinks salt water." (C:11892 area).
- Tavern rumour (in-world, unreliable): "The Narrows have a second market, deeper in the cliffs, where things other than spice change hands. Finding the entrance is the first test of membership." (C:12166 area).

| Name | Title | Temperament | Institution | Cite |
|---|---|---|---|---|
| The Mouth | Anchorage Keeper | invisible | harbor_master | C:10666 |
| Spice Lord Kiran | Lord of the Narrows | dangerous | governor | C:10688 |
| The Weigher | Price Master | mathematical | exchange | C:10710 |
| Raj the Quiet's Shadow | Intelligence Officer | phantom | customs | C:10732 |
| Mama Smoke | Cave Cook | motherly | tavern | C:10754 |
| The Poison Doctor | Apothecary | ambiguous | apothecary | C:10776 |
| The Ghost Broker | Contract Handler | anonymous | broker | C:10798 |

- Dress [invented]: dark cotton, faces partly covered, no guild marks. The character hold stands: no likenesses or portraits.
- Speech [cite]: built from the greeting lines in each NPC record above (greeting_neutral, friendly, hostile).
- Temperament of the town: read from the power structure and tension in section E.
- Network (tavern, personal): Mama Lucia - Mama Smoke, linking Corsair's Rest (C:11532). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Network (broker, allied): Ghost - The Ghost Broker, linking Corsair's Rest (C:11652). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Sea sighting filed under this port: Typhoon Mei (C:12894).

## C. Visual identity (guidance only; commissions nothing; the character hold stands)

Pattern: the shared visual-identity headings 1-7 (design notes). Shared camera, envelope and do/don't blocks are in README.md and not repeated here.

1. Architecture. Identity line [invented]: The Hanging Market is the town: everything in Spice Narrows hangs over water and can be cut loose. Landmark [cite]: "The Hanging Market - stalls suspended on ropes between cliff faces above the water." (C:12166 area). Cited buildings: The Volcanic Channel (harbor_master, C:10594); The Dark Room (exchange, C:10604); The Deep Throne (governor, C:10614); Mama Smoke's Cave (tavern, C:10624); The Intelligence Cave (customs, C:10634); The Poison Lab (apothecary, C:10644); The Contract Wall (broker, C:10654). Kit use [proposal]: compose from the shipped will-use plates; identity carriers are ground choice and building frequency, never new ids. Gaps follow the art gap list (design notes). Landmark canon: ../landmarks/hanging_market.md.
2. Materials: local and abundant (exports): Spice, Pearls, Opium; scarce or luxury (imports): Silk, Weapons, Grain, Rum, Pelts, Black Powder (section A). Atmosphere [cite]: "Concentrated spice - overwhelming, intoxicating. The air itself burns your eyes." (C:12166 area). Climate: section G.
3. Palette (references, not UI tokens; UI stays INK / GOLD / CREAM / MUTED / DUE). Judged in sRGB against the plate envelope (luma 65-141, chroma p98 35):

| Role | Family | sRGB | Luma / chroma | Status | Tie |
|---|---|---|---|---|---|
| Dominant | wet cliff rock | (84, 82, 74) #54524A | 82 / 10 | in range | cliff faces |
| Secondary 1 | rope and plank | (102, 86, 68) #665644 | 88 / 34 | in range | suspended stalls |
| Secondary 2 | deep shadow green | (66, 78, 70) #424E46 | 75 / 12 | in range | the archipelago |
| Accent | spice red, small | (134, 93, 71) #865D47 | 100 / 63 | small-area accent (chroma 63) | spice, its top export |

4. Dress: see section B. 5. Signage: ASCII only, no UI-duplicating words; motifs [invented]: none; a knot that marks a stall as open. 6. Silhouette: chart scale is the shared `chart_port_marker`; harbour-scale skyline [invented]: no skyline: rope bridges and stalls strung between cliffs above dark water. 7. Do and don't: README shared list.

## D. Historical analogies (feed [invented] prose; never a cite)

- **Port Royal, as an outlaw market.** Fits: a hidden anchorage selling spice, opium and pearls outside any law (black market, Spice 1.8, Opium 1.4). Suggests: prices whispered; brokers without names. Limit: trade only, never a people (analogy rule 3). Avoid: pirate costume.
- **Malacca-strait smuggling anchorages, as geography.** Fits: an archipelago hideout controlling a spice route. Suggests: cliff markets, look-outs, boats that move at night. Limit: geography and logistics only. Avoid: any ethnic coding.
- **A Venetian fondaco turned inside out.** Fits: a merchant house with rules, but the rules are silence. Suggests: whispered bids, the Weigher's word final. Limit: form only. Avoid: carnival masks.
- Contrast, not model: Jade Port's posted, inspected trade; the Narrows undercuts it in the dark.

## E. Governance

- Title: Spice Lord (C:10588).
- Power structure: "Spice Narrows is the Monsoon Syndicate's heart - Kiran controls everything from the deepest cave. The Mouth guards access. The Weigher prices in darkness. Raj's Shadow collects intelligence on every captain who enters. The Ghost Broker posts contracts on cave walls. Mama Smoke feeds people in spice smoke. The Poison Doctor heals and occasionally does other things. Every person here is partially invisible, partially anonymous, and entirely controlled by Kiran's intelligence network." [ascii-folded] (C:10590).
- Internal tension: "The Narrows has no internal tension - because Kiran's information control eliminates the possibility of opposition. He knows everything before it happens. The EXTERNAL tension is the navy: Dragon's Gate's Commander Zhang and Inspector Sun are systematically mapping the Narrows' supply lines. Every raid Kiran predicts and escapes makes him more confident, which makes him less careful. Mama Smoke's back exit - the one she denies exists - is the Narrows' secret survival insurance. If Kiran ever fails to predict a raid, that exit becomes the most important tunnel in the East Indies." (C:10591).
- Bloc stance: "Spice Narrows is where the rules end and the real prices begin." (C:13680 area).
- Local rival: Jade Port: "The Spice Lords undercut Jade Port's legitimate spice trade with smuggled goods at lower prices. The Kiln Masters are furious." (C:13680 area).
- Grudge: "A Jade Port customs agent infiltrated the Narrows. The Spice Lords sent him back - alive, but with every piece of intelligence he'd gathered tattooed on his forehead." (C:13680 area).
- Bloc philosophy: "No names, no questions, no manifests." (C:13493 area). Loyalty and spillover numbers are [flavour, no mechanic] (DF-15).

## F. Relations with every other port

Hand-maintained from content.json. Disposition is bloc-to-bloc (symmetric in data); lanes are undirected; flows are directional by market affinity; networks are symmetric.

<!-- hand-maintained:begin -->
| Port | Region | Bloc | Disposition | Lane (distance, min class, danger) | Flows out (they import) | Flows in (we import) | Network |
|---|---|---|---|---|---|---|---|
| Porto Novo | Mediterranean | The Exchange Alliance | hostile (C:13746) | - | - | Grain, Rum | - |
| Al-Manar | Mediterranean | The Exchange Alliance | hostile (C:13746) | - | - | - | - |
| Silva Bay | Mediterranean | The Exchange Alliance | hostile (C:13746) | - | - | Pelts | - |
| Corsair's Rest | Mediterranean | The Shadow Ports | same bloc | 80, galleon, 0.25 (C:2288) | Opium | Black Powder, Pelts, Rum, Weapons | C:11532, C:11652 |
| Ironhaven | North Atlantic | The Iron Pact | hostile (C:13781) | - | - | Pelts, Weapons | - |
| Stormwall | North Atlantic | The Iron Pact | hostile (C:13781) | - | - | Grain, Weapons | - |
| Thornport | North Atlantic | The Free Ports | neutral (C:13844) | - | Spice | Pelts | - |
| Sun Harbor | West Africa | The Gold Coast Compact | neutral (C:13809) | - | Spice | Pelts | - |
| Palm Cove | West Africa | The Gold Coast Compact | neutral (C:13809) | - | - | Rum | - |
| Iron Point | West Africa | The Gold Coast Compact | neutral (C:13809) | - | - | - | - |
| Pearl Shallows | West Africa | The Gold Coast Compact | neutral (C:13809) | - | - | - | - |
| Jade Port | East Indies | The Silk Circle | rival (C:13830) | - | - | Silk | - |
| Monsoon Reach | East Indies | The Free Ports | neutral (C:13844) | 26, brigantine, 0.13 (C:2198) | - | Silk | - |
| Silk Haven | East Indies | The Silk Circle | rival (C:13830) | 20, brigantine, 0.11 (C:2207) | Spice | Silk | - |
| Crosswind Isle | East Indies | The Free Ports | neutral (C:13844) | - | - | - | - |
| Dragon's Gate | East Indies | The Silk Circle | rival (C:13830) | 28, brigantine, 0.14 (C:2216) | - | - | - |
| Ember Isle | South Seas | The Coral Crown | neutral (C:13858) | 48, galleon, 0.18 (C:2234) | - | - | - |
| Typhoon Anchorage | South Seas | The Coral Crown | neutral (C:13858) | - | - | - | - |
| Coral Throne | South Seas | The Coral Crown | neutral (C:13858) | - | - | Rum | - |
<!-- hand-maintained:end -->

- Lane lore: "The Volcanic Passage": "Warm currents and sulfur air. The locals say the sea boils near the islands on bad days." (C:2234). Lane canon: [The Volcanic Passage](../lanes/volcanic_passage.md).
- Lane lore: "The Smuggler's Run": "Only the desperate or the bold sail this direct. Pirate kings once controlled both ends." (C:2288). Lane canon: [The Smuggler's Run](../lanes/smugglers_run.md).
- Asymmetry flag: Spice Narrows names Jade Port as local rival (C:13680); Jade Port names none (C:13615). Canon reads local_rival_port as directional: a grievance, not a mutual feud. See DF-6.

## G. World details (undefined in data; all [invented] unless cited)

- History (shared timeline in README.md): T13: The Spice Lords take the archipelago anchorage and begin undercutting Jade Port (rivalry text). T13: The Monsoon Syndicate makes the Narrows its home port [invented; 'Raj the Quiet's Shadow' is the cited link]. T15: A Jade Port customs spy is sent back with his intelligence (grudge text).
- Population [invented]: about 3,000, nobody counts.
- Language [invented]: Silk Waters trade speech, whispered; a hand-sign price language.
- Currency: silver is the only money in the sim (every price, fee and wage is in silver). Local coin names, if any, are [invented] flavour and never replace 'silver' in copy.
- Religion [invented]: none spoken; offerings left in cliff niches for safe passage.
- Climate [invented port detail]: hot, humid, sheltered by cliffs; sound carries over the water (custom). Regional weather [cite]: "Incense drifts from a passing junk. The air is thick with spice." (C:11892 area, weather_flavor).
- Pirates in these waters [cite]: The Monsoon Syndicate (C:2888). Home port [invented]: The Monsoon Syndicate - 'Raj the Quiet's Shadow' sits in the Narrows' institutions and Typhoon Mei is sighted there.

## H. Extra depth (thin port)

- Daily life [invented]: the Hanging Market opens after dark. Stalls are hauled up by rope at dawn so nothing shows from the sea.
- Factions [invented]: the Spice Lords own the anchorage. The Monsoon Syndicate is a tenant that pays in silence, and the Weigher's word ends every dispute.
- A local law [invented]: a whispered price is binding; a spoken one is an insult and voids the deal.

## I. Sound and ambience (all [invented] sound design unless quoted from a cite)

- **Soundscape:** hidden between volcanic cliffs; "Prices are whispered ... the water carries sound" (C:12168); rope ladders; vertical market overhead.
- **Signature:** rope ladder rung creak; basket hauled up on rope; whisper-close cloth rustle (no voice).
- **Music/work songs (NOTE ONLY):** none - silence is canon.
- **Vs neighbours:** echoing, vertical; vs Al-Manar's whispers on marble, this is rope and cliff echo. (Only port where a short cliff echo is canon-justified - a deliberate section 1 exception if ever used.)
- **Fragments [proposal; no fragment or asset exists yet]:** `src.rope_ladder`, `src.basket_haul`, `place.cliff_anchorage`.
- Cited hooks: atmosphere "Concentrated spice - overwhelming, intoxicating. The air itself burns your eyes." (C:12169); dock scene "The anchorage is hidden between volcanic cliffs. Rope ladders drop from the Hanging Market above. Commerce here is vertical - everything climbs." (C:12170).
- Invented bed idea: whispers, ropes creaking, water below. No assets are requested.

## J. Flags for this port

- DF-6: Local rivals are one-sided
- DF-17: Pirate captain sightings imply home ports the data does not define
