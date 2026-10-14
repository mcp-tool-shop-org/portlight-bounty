# Stormwall (stormwall)

Base commit: 143e284 (content.json unchanged since 45b7378). Status: draft canon, phase 1. Written 2026-10-14.
Cites `C:n` are crates/portlight-sim/data/content.json line n at 143e284. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] (design-notes pattern) | DF-n see data-fixes.md | T-n see README shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Region | North Atlantic ("The Iron Coast") | C:645, C:11782 |
| Bloc | The Iron Pact (Ironhaven, Stormwall) | C:13570, C:13410 |
| Features | none | C:645-721 |
| Warehouse tiers | none listed | C:6520 |
| Port fee / provisions / repair / crew | 10 / 2 / 2 per hull point / 3 | C:645-721 |
| Map | x 4, y 8 | C:721 |
| Lanes | 3 | section F |
| Player description | "Fortress port guarding the northern straits. Military outpost with strict inspections." | C:647 |
| Captain start here | The Privateer | C:2431 |

## A. Specialties and trade goods

<!-- hand-maintained:begin -->
| Good | Category | Base | Affinity | Stock target | Restock | Role |
|---|---|---|---|---|---|---|
| Medicines | medicine | 35 | 1.4 | 12 | 1.5 | exporter |
| Weapons | military | 45 | 1.1 | 18 | 1.5 | exporter |
| Grain | commodity | 12 | 1.1 | 22 | 2.0 | exporter |
| Timber | commodity | 18 | 1.0 | 20 | 1.5 | neutral |
| Pelts | commodity | 8 | 0.8 | 10 | 1.0 | importer |
| Tobacco | provision | 28 | 0.55 | 10 | 0.8 | importer |
| Rum | provision | 20 | 0.5 | 12 | 1.0 | importer |
| Silk | luxury | 70 | 0.5 | 5 | 0.5 | importer |
<!-- hand-maintained:end -->

Market cite: C:645-721. Price rule: base x (target / stock) / affinity (crates/portlight-sim/src/economy.rs:37); affinity above 1 is cheap here.
Not traded here: Iron Ore, Cotton, Dyes, Spice, Porcelain, Tea, Pearls, Opium, Black Powder, Stolen Cargo.
Region goods [flavour, no mechanic]: sacred Medicines; prized Tea, Tobacco, Weapons; forbidden none (C:11782 area).
Festival: The Iron Days, every 75 days for 4, market effects Iron Ore x1.6, Weapons x1.8, Timber x1.3 (C:11807).
Festival: Midwinter Market, every 90 days for 3, market effects Medicines x2.0, Tea x1.7, Tobacco x1.5 (C:11821).
Contract templates touching North Atlantic: `proc_weapons_garrison` (procurement), `short_tea_crisis` (shortage), `ret_cotton_return` (return_freight), `ret_spice_restock` (return_freight), `ret_dyes_textile` (return_freight), `circ_south_seas` (circuit), `rep_charter_atlantic` (reputation_charter), `smug_opium_run` (smuggling). Source port Stormwall: none.

## B. People

- Cultural group: The Northern Garrison: "Military authority that governs the port. Trade is tolerated, not celebrated." (C:12056 area).
- Local custom: "All cargo is inspected. Complaining about it marks you as a southerner." (C:12056 area).
- Regional ethos: "Strength is built, not born. The forge, the hull, the garrison - everything here is made to endure. Softness is a southern luxury." Greeting: "State your business, Captain. These waters have little patience." Proverb: "The storm tests the hull, not the cargo." (C:11782 area).
- Tavern rumour (in-world, unreliable): "The garrison commander has doubled the night watch. Nobody says why, but the soldiers look north when they think no one's watching." (C:12056 area).

| Name | Title | Temperament | Institution | Cite |
|---|---|---|---|---|
| Commander Elias Vogt | Garrison Commander | rigid | governor | C:8194 |
| Quartermaster Maren Dahl | Quartermaster | efficient | exchange | C:8215 |
| Dr. Kristian Halvorsen | Garrison Surgeon | compassionate | apothecary | C:8236 |
| Ingrid Norum | Tavern Keeper | resilient | tavern | C:8257 |
| Inspector Berg | Senior Customs Inspector | thorough | customs | C:8278 |
| Siv Lindgren | Trade Liaison | optimistic | broker | C:8299 |
| Widow Kael | Outfitter (merchant) | quiet | outfitter | C:5931 |

- Dress [invented]: garrison wool in grey, inspectors' sashes; civilians in plain wool. The character hold stands: no likenesses or portraits.
- Speech [cite]: built from the greeting lines in each NPC record above (greeting_neutral, friendly, hostile).
- Temperament of the town: read from the power structure and tension in section E.
- Network (merchant, allied): Herbalist Sera - Dr. Halvorsen, linking Ember Isle (C:11472). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Network (merchant, allied): Old Cassius - Quartermaster Maren, linking Palm Cove (C:11484). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Network (tavern, personal): Rosa Carvalho - Ingrid Norum, linking Silva Bay (C:11568). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Network (inspector, allied): Inspector Kross - Inspector Berg, linking Ironhaven (C:11688). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Sea sighting filed under this port: Siv Lindgren (C:12846).

## C. Visual identity (guidance only; commissions nothing; the character hold stands)

Pattern: the shared visual-identity headings 1-7 (design notes). Shared camera, envelope and do/don't blocks are in README.md and not repeated here.

1. Architecture. Identity line [invented]: The Fortress is the town: Stormwall is a garrison with a harbour, not a harbour with a fort. Landmark [cite]: "The Fortress - grey stone walls rising from the sea, older than anyone remembers." (C:12056 area). Cited buildings: The Fortress Gate (harbor_master, C:8122); The Supply Depot (exchange, C:8132); The Commander's Office (governor, C:8142); The Garrison Infirmary (apothecary, C:8152); The Watchtower (tavern, C:8162); The Inspection Hall (customs, C:8172); The Trade Liaison Office (broker, C:8182). Kit use [proposal]: compose from the shipped will-use plates; identity carriers are ground choice and building frequency, never new ids. Gaps follow the art gap list (design notes). Landmark canon: ../landmarks/stormwall_fortress.md.
2. Materials: local and abundant (exports): Weapons, Grain, Medicines; scarce or luxury (imports): Rum, Tobacco, Pelts, Silk (section A). Atmosphere [cite]: "Salt wind, military drums from the garrison, and the creak of heavy gates." (C:12056 area). Climate: section G.
3. Palette (references, not UI tokens; UI stays INK / GOLD / CREAM / MUTED / DUE). Judged in sRGB against the plate envelope (luma 65-141, chroma p98 35):

| Role | Family | sRGB | Luma / chroma | Status | Tie |
|---|---|---|---|---|---|
| Dominant | weathered fortress granite | (112, 112, 110) #70706E | 112 / 2 | in range | grey stone walls |
| Secondary 1 | sea-wet stone | (86, 90, 92) #565A5C | 89 / 6 | in range | walls rising from the sea |
| Secondary 2 | garrison wool | (96, 98, 92) #60625C | 97 / 6 | in range | uniforms and banners, muted |
| Accent | lamp-lit window | (140, 120, 86) #8C7856 | 122 / 54 | small-area accent (chroma 54) | the watchtower |

4. Dress: see section B. 5. Signage: ASCII only, no UI-duplicating words; motifs [invented]: the Pact mark on the gate; tally marks for inspections. 6. Silhouette: chart scale is the shared `chart_port_marker`; harbour-scale skyline [invented]: sheer grey walls from the waterline, a watchtower and a single gate. 7. Do and don't: README shared list.

## D. Historical analogies (feed [invented] prose; never a cite)

- **Gibraltar-style straits fortress.** Fits: a fortress port guarding a strait with strict inspections (description). Suggests: everything inspected, complaints mark you as a southerner. Limit: no empire behind it; it answers to the Pact. Avoid: redcoat imagery.

## E. Governance

- Title: Garrison Commander (C:8116).
- Power structure: "Pure military hierarchy. Commander Vogt has absolute authority. Maren runs supplies, Berg runs security, Halvorsen runs medicine, and they all report to Vogt. Siv is the civilian bridge - the only person whose job is making the fortress work as a trade port. Ingrid's tavern is the pressure valve that keeps the garrison from cracking under Vogt's rigid discipline." [ascii-folded] (C:8118).
- Internal tension: "The surface tension is military vs. civilian: Stormwall is a fortress that needs trade revenue to function. Vogt sees merchants as a necessary evil; Siv sees them as partners. The deeper tension is Halvorsen's oath - he treats everyone, including Iron Wolf deserters. Vogt either doesn't know or pretends not to. If it becomes public, it forces a confrontation between military duty and medical ethics that neither man wants. Meanwhile, Vogt's unopened letters and nightly telescope-watching hint at something personal beneath the regulation exterior - a daughter, a threat from the north, or both." (C:8119).
- Bloc stance: "Stormwall enforces the Pact's will. The garrison answers to the Pact, not the crown." (C:13570 area).
- Grudge: "The garrison commander's daughter married a trader from Corsair's Rest. He hasn't spoken to her since." (C:13570 area).
- Bloc philosophy: "Strength through industry. Trade serves the fortress." (C:13410 area). Loyalty and spillover numbers are [flavour, no mechanic] (DF-15).

## F. Relations with every other port

Hand-maintained from content.json. Disposition is bloc-to-bloc (symmetric in data); lanes are undirected; flows are directional by market affinity; networks are symmetric.

<!-- hand-maintained:begin -->
| Port | Region | Bloc | Disposition | Lane (distance, min class, danger) | Flows out (they import) | Flows in (we import) | Network |
|---|---|---|---|---|---|---|---|
| Porto Novo | Mediterranean | The Exchange Alliance | allied (C:13718) | - | - | Rum | - |
| Al-Manar | Mediterranean | The Exchange Alliance | allied (C:13718) | - | Grain | - | - |
| Silva Bay | Mediterranean | The Exchange Alliance | allied (C:13718) | - | Grain, Weapons | Pelts | C:11568 |
| Corsair's Rest | Mediterranean | The Shadow Ports | hostile (C:13781) | 40, brigantine, 0.15 (C:2018) | Grain, Medicines | Pelts, Rum, Tobacco | - |
| Ironhaven | North Atlantic | The Iron Pact | same bloc | 20, sloop, 0.09 (C:1973) | Grain | Pelts | C:11688 |
| Thornport | North Atlantic | The Free Ports | neutral (C:13774) | 18, sloop, 0.1 (C:1991) | Medicines | Pelts, Tobacco | - |
| Sun Harbor | West Africa | The Gold Coast Compact | rival (C:13760) | - | - | Pelts | - |
| Palm Cove | West Africa | The Gold Coast Compact | rival (C:13760) | - | Grain | Rum, Tobacco | C:11484 |
| Iron Point | West Africa | The Gold Coast Compact | rival (C:13760) | - | Grain, Medicines, Weapons | - | - |
| Pearl Shallows | West Africa | The Gold Coast Compact | rival (C:13760) | - | Grain, Medicines | - | - |
| Jade Port | East Indies | The Silk Circle | hostile (C:13767) | - | Grain | Silk | - |
| Monsoon Reach | East Indies | The Free Ports | neutral (C:13774) | - | Grain, Medicines | Silk | - |
| Silk Haven | East Indies | The Silk Circle | hostile (C:13767) | - | Grain | Silk | - |
| Crosswind Isle | East Indies | The Free Ports | neutral (C:13774) | - | - | - | - |
| Dragon's Gate | East Indies | The Silk Circle | hostile (C:13767) | - | Medicines | - | - |
| Spice Narrows | East Indies | The Shadow Ports | hostile (C:13781) | - | Grain, Weapons | - | - |
| Ember Isle | South Seas | The Coral Crown | neutral (C:13788) | - | Grain, Weapons | - | C:11472 |
| Typhoon Anchorage | South Seas | The Coral Crown | neutral (C:13788) | - | Grain, Medicines | - | - |
| Coral Throne | South Seas | The Coral Crown | neutral (C:13788) | - | Grain, Weapons | Rum, Tobacco | - |
<!-- hand-maintained:end -->

- Lane lore: "The Iron Strait": "Iron from the foundries, soldiers from the fortress. This lane never sleeps." (C:1973).
- Lane lore: "The Tea and Tobacco Road": "Thornport's warmth heading to Stormwall's cold. The northern comfort trade." (C:1991).

## G. World details (undefined in data; all [invented] unless cited)

- History (shared timeline in README.md): T3: The Fortress walls are raised by the old Northern Crown, 'older than anyone remembers' (landmark). T8: The Northern Crown dissolves; the garrison holds the straits alone. Lore resolution: 'answers to the Pact, not the crown' (C:13575) refers to this vanished crown. T11: The garrison joins the Iron Pact. T15: The Brotherhood's refusal to build Pact warships is taken as an insult (Silva Bay grudge).
- Population [invented]: about 5,000, half of them soldiers and their families.
- Language [invented]: Northern trade speech; drill-yard commands.
- Currency: silver is the only money in the sim (every price, fee and wage is in silver). Local coin names, if any, are [invented] flavour and never replace 'silver' in copy.
- Religion [invented]: the Hearth, kept in the garrison chapel; soldiers swear on the Fortress stones.
- Climate [invented port detail]: grey northern straits; gales and short winter days. Regional weather [cite]: "Grey skies press low. The Atlantic rolls in long, heavy swells." (C:11782 area, weather_flavor).
- Pirates in these waters [cite]: The Crimson Tide (C:2879), The Iron Wolves (C:2905).

## I. Sound and ambience (all [invented] sound design unless quoted from a cite)

- **Soundscape:** "Salt wind, military drums from the garrison, and the creak of heavy gates" (C:12059); boots of grey-coat patrols; paper checked against sealed orders.
- **Signature:** heavy gate hinge creak; manifest page flipped and stamped; drum is ambient-distant only.
- **Music/work songs (NOTE ONLY):** garrison drums are canon - treat as far, unpitched military cadence if music ever opens.
- **Vs neighbours:** ordered and watched; vs Ironhaven's roar and Thornport's homely sheds. Contrast with Dragon's Gate: wind + gates here, chains + tea there.
- **Fragments [proposal; no fragment or asset exists yet]:** `src.heavy_gate`, `src.inspection_stamp`, `place.garrison_quay`.
- Cited hooks: atmosphere "Salt wind, military drums from the garrison, and the creak of heavy gates."; dock scene "Navy vessels line the inner harbor. Soldiers in grey coats patrol the docks, checking manifests against sealed orders. Everything is orderly. Everything is watched." (C:12056 area).
- Invented bed idea: the watch bell, inspectors calling manifests, gulls and gale. No assets are requested.

## J. Flags for this port

- DF-8: NPC name drift across records
- DF-12: Rosa and Ingrid network says 'Northern solidarity'
- DF-14: Brotherhood 'never builds warships' conflicts with the sim
- Resolved in lore: 'Not the crown' refers to the vanished Northern Crown (T3, T8).
