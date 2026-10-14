# Ironhaven (ironhaven)

Base commit: 143e284 (content.json unchanged since 45b7378). Status: draft canon, phase 1. Written 2026-10-14.
Cites `C:n` are crates/portlight-sim/data/content.json line n at 143e284. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] (design-notes pattern) | DF-n see data-fixes.md | T-n see README shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Region | North Atlantic ("The Iron Coast") | C:556, C:11782 |
| Bloc | The Iron Pact (Ironhaven, Stormwall) | C:13559, C:13410 |
| Features | shipyard | C:556-642 |
| Warehouse tiers | depot, regional, commercial | C:6536 |
| Port fee / provisions / repair / crew | 6 / 2 / 1 per hull point / 4 | C:556-642 |
| Map | x 8, y 4 | C:642 |
| Lanes | 5 | section F |
| Player description | "Industrial port city wreathed in forge smoke. Weapons and iron flow out, everything else flows in." | C:558 |

## A. Specialties and trade goods

<!-- hand-maintained:begin -->
| Good | Category | Base | Affinity | Stock target | Restock | Role |
|---|---|---|---|---|---|---|
| Iron Ore | commodity | 25 | 1.7 | 45 | 4.0 | exporter |
| Weapons | military | 45 | 1.6 | 30 | 3.0 | exporter |
| Pelts | commodity | 8 | 1.2 | 12 | 1.5 | exporter |
| Tobacco | provision | 28 | 0.9 | 15 | 1.0 | importer |
| Timber | commodity | 18 | 0.8 | 18 | 1.5 | importer |
| Rum | provision | 20 | 0.6 | 8 | 0.8 | importer |
| Grain | commodity | 12 | 0.55 | 15 | 1.0 | importer |
| Cotton | commodity | 15 | 0.5 | 12 | 1.0 | importer |
| Silk | luxury | 70 | 0.45 | 5 | 0.5 | importer |
<!-- hand-maintained:end -->

Market cite: C:556-642. Price rule: base x (target / stock) / affinity (crates/portlight-sim/src/economy.rs:37); affinity above 1 is cheap here.
Not traded here: Dyes, Spice, Porcelain, Tea, Pearls, Medicines, Opium, Black Powder, Stolen Cargo.
Region goods [flavour, no mechanic]: sacred Medicines; prized Tea, Tobacco, Weapons; forbidden none (C:11782 area).
Festival: The Iron Days, every 75 days for 4, market effects Iron Ore x1.6, Weapons x1.8, Timber x1.3 (C:11807).
Festival: Midwinter Market, every 90 days for 3, market effects Medicines x2.0, Tea x1.7, Tobacco x1.5 (C:11821).
Contract templates touching North Atlantic: `proc_weapons_garrison` (procurement), `short_tea_crisis` (shortage), `ret_cotton_return` (return_freight), `ret_spice_restock` (return_freight), `ret_dyes_textile` (return_freight), `circ_south_seas` (circuit), `rep_charter_atlantic` (reputation_charter), `smug_opium_run` (smuggling). Source port Ironhaven: none.

## B. People

- Cultural group: The Iron Guild: "Masters of the forge. They set the iron price and every tool bears their mark." (C:12046 area).
- Local custom: "Greeting the harbor master with a nod, not words. The north wastes neither." (C:12046 area).
- Regional ethos: "Strength is built, not born. The forge, the hull, the garrison - everything here is made to endure. Softness is a southern luxury." Greeting: "State your business, Captain. These waters have little patience." Proverb: "The storm tests the hull, not the cargo." (C:11782 area).
- Tavern rumour (in-world, unreliable): "The foundry master has been buying every scrap of timber that comes in. Some say he's building something in the deep sheds - something big." (C:12046 area).

| Name | Title | Temperament | Institution | Cite |
|---|---|---|---|---|
| Gunnar Stahl | Harbor Master | blunt | harbor_master | C:7960 |
| Forge Master Henrik Brandt | Guild Master | imperious | exchange | C:7982 |
| Astrid Vekhren | Pact Commissioner | strategic | governor | C:8004 |
| The Smith | Master Weaponsmith | silent | shipyard | C:8026 |
| Olga Strand | Tavern Keeper | stoic | tavern | C:8048 |
| Inspector Kross | Weapons Inspector | methodical | customs | C:8070 |
| Jan Eriksson | Senior Broker | pragmatic | broker | C:8092 |
| Sgt. Gruber | Arsenal Master (merchant) | gruff | outfitter | C:5915 |
| Red Hamish | style master | - | - | C:6189 |

- Dress [invented]: leather and wool, scorched aprons, guild arm-rings. The character hold stands: no likenesses or portraits.
- Speech [cite]: built from the greeting lines in each NPC record above (greeting_neutral, friendly, hostile).
- Temperament of the town: read from the power structure and tension in section E.
- Network (merchant, rival): Henrik Brandt - Foreman Kofi Mensah, linking Iron Point (C:11448). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Network (inspector, allied): Inspector Kross - Inspector Berg, linking Stormwall (C:11688). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Network (inspector, professional): Inspector Sun - Inspector Kross, linking Dragon's Gate (C:11700). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Sea sighting filed under this port: The Smith (C:12834).
- Sea sighting filed under this port: Sergeant Kruze (C:12840).

## C. Visual identity (guidance only; commissions nothing; the character hold stands)

Pattern: the shared visual-identity headings 1-7 (design notes). Shared camera, envelope and do/don't blocks are in README.md and not repeated here.

1. Architecture. Identity line [invented]: The Foundry is the town: its chimney is the port's lighthouse. Landmark [cite]: "The Great Foundry - its chimney visible twenty leagues at sea, glowing red at night." (C:12046 area). Cited buildings: The Iron Dock (harbor_master, C:7888); The Great Foundry (exchange, C:7898); The Pact Office (governor, C:7908); The Deep Forge (shipyard, C:7918); The Forge & Anchor (tavern, C:7928); The Weapons Registry (customs, C:7938); The Contract Hall (broker, C:7948). Kit use [proposal]: compose from the shipped will-use plates; identity carriers are ground choice and building frequency, never new ids. Gaps follow the art gap list (design notes).
2. Materials: local and abundant (exports): Iron Ore, Weapons, Pelts; scarce or luxury (imports): Timber, Grain, Cotton, Tobacco, Silk, Rum (section A). Atmosphere [cite]: "Coal smoke, ringing hammers, and the deep bass hum of bellows forcing air into furnaces." (C:12046 area). Climate: section G.
3. Palette (references, not UI tokens; UI stays INK / GOLD / CREAM / MUTED / DUE). Judged in sRGB against the plate envelope (luma 65-141, chroma p98 35):

| Role | Family | sRGB | Luma / chroma | Status | Tie |
|---|---|---|---|---|---|
| Dominant | soot-stained brick | (96, 80, 72) #605048 | 83 / 24 | in range | forge smoke over brick |
| Secondary 1 | cold iron grey | (98, 100, 104) #626468 | 100 / 6 | in range | iron ore and blades |
| Secondary 2 | northern sea slate | (80, 88, 92) #50585C | 87 / 12 | in range | harbour stone |
| Accent | furnace glow | (150, 96, 64) #966040 | 105 / 86 | mood, not values | the chimney 'glowing red at night' |

4. Dress: see section B. 5. Signage: ASCII only, no UI-duplicating words; motifs [invented]: a hammer over an anvil, the Iron Guild mark. 6. Silhouette: chart scale is the shared `chart_port_marker`; harbour-scale skyline [invented]: one great chimney above a crowd of workshop roofs; cranes on the ore quay. 7. Do and don't: README shared list.

## D. Historical analogies (feed [invented] prose; never a cite)

- **Sheffield and the Weald ironworks.** Fits: an iron and weapons town where furnace and harbour feed each other (Iron Ore 1.7, Weapons 1.6). Suggests: nods instead of words; guild marks on blades. Limit: no coal mining in data; keep to iron and forge. Avoid: Victorian industrial grime cliches.

## E. Governance

- Title: Pact Commissioner (C:7882).
- Power structure: "Ironhaven is governed by the Iron Guild's production and the Iron Pact's politics - a dual authority. Henrik controls the foundry and sets iron prices. Astrid coordinates Pact strategy and enforces the weapons embargo. They need each other and neither fully trusts the other. The Smith operates outside both systems - his custom forge answers to craft alone. Gunnar runs the harbor as logistics, not politics. Olga's tavern is the only space where rank dissolves. Kross watches the weapons chain with military precision. Jan brokers what Astrid allows, resenting the boundaries he won't cross." [ascii-folded] (C:7884).
- Internal tension: "The core tension is between Henrik's industrial ambition and Astrid's political control. Henrik wants to sell iron to anyone who pays. Astrid wants to restrict weapons sales to maintain Pact leverage. The embargo makes the Pact powerful but costs the Guild revenue - Jan's private ledger of lost deals grows monthly. The Smith is the wildcard: he builds for whoever he chooses, including pirates (he built a sword for Scarlet Ana), and neither Henrik nor Astrid can stop him because his talent is irreplaceable. Meanwhile, Gunnar shares a history with the Iron Wolves he never discusses, and Kross traces weapons that sometimes lead to uncomfortable conclusions about where the Guild's loyalty ends and the black market begins." (C:7885).
- Bloc stance: "Ironhaven IS the Pact. The Iron Guild's word is law north of the strait." (C:13559 area).
- Local rival: Iron Point: "Both export iron. Ironhaven's is refined; Iron Point's is raw and cheaper. The foundry masters resent the undercutting." (C:13559 area).
- Trade embargo (data, unread by the sim; crates/portlight-sim/src/content.rs:676): Weapons.
- Grudge: "A shipment of Ironhaven weapons was found at Corsair's Rest. The merchant who sold them was never seen again." (C:13559 area).
- Bloc philosophy: "Strength through industry. Trade serves the fortress." (C:13410 area). Loyalty and spillover numbers are [flavour, no mechanic] (DF-15).

## F. Relations with every other port

Hand-maintained from content.json. Disposition is bloc-to-bloc (symmetric in data); lanes are undirected; flows are directional by market affinity; networks are symmetric.

<!-- hand-maintained:begin -->
| Port | Region | Bloc | Disposition | Lane (distance, min class, danger) | Flows out (they import) | Flows in (we import) | Network |
|---|---|---|---|---|---|---|---|
| Porto Novo | Mediterranean | The Exchange Alliance | allied (C:13718) | 36, cutter, 0.12 (C:2000) | Pelts | Grain, Rum | - |
| Al-Manar | Mediterranean | The Exchange Alliance | allied (C:13718) | - | Iron Ore, Pelts | - | - |
| Silva Bay | Mediterranean | The Exchange Alliance | allied (C:13718) | 32, brigantine, 0.11 (C:2009) | Weapons | Timber | - |
| Corsair's Rest | Mediterranean | The Shadow Ports | hostile (C:13781) | - | - | Rum, Tobacco | - |
| Stormwall | North Atlantic | The Iron Pact | same bloc | 20, sloop, 0.09 (C:1973) | Pelts | Grain | C:11688 |
| Thornport | North Atlantic | The Free Ports | neutral (C:13774) | 22, sloop, 0.08 (C:1982) | - | Timber, Tobacco | - |
| Sun Harbor | West Africa | The Gold Coast Compact | rival (C:13760) | - | - | Cotton | - |
| Palm Cove | West Africa | The Gold Coast Compact | rival (C:13760) | - | Iron Ore, Pelts | Cotton, Rum, Tobacco | - |
| Iron Point | West Africa | The Gold Coast Compact | rival (C:13760) | - | Pelts, Weapons | - | C:11448 |
| Pearl Shallows | West Africa | The Gold Coast Compact | rival (C:13760) | - | - | Cotton | - |
| Jade Port | East Indies | The Silk Circle | hostile (C:13767) | 76, galleon, 0.22 (C:2297) | Iron Ore, Pelts | Silk | - |
| Monsoon Reach | East Indies | The Free Ports | neutral (C:13774) | - | Pelts | Silk | - |
| Silk Haven | East Indies | The Silk Circle | hostile (C:13767) | - | Iron Ore, Pelts | Silk | - |
| Crosswind Isle | East Indies | The Free Ports | neutral (C:13774) | - | Pelts | - | - |
| Dragon's Gate | East Indies | The Silk Circle | hostile (C:13767) | - | Iron Ore, Pelts | - | C:11700 |
| Spice Narrows | East Indies | The Shadow Ports | hostile (C:13781) | - | Pelts, Weapons | - | - |
| Ember Isle | South Seas | The Coral Crown | neutral (C:13788) | - | Pelts, Weapons | - | - |
| Typhoon Anchorage | South Seas | The Coral Crown | neutral (C:13788) | - | Pelts | Timber | - |
| Coral Throne | South Seas | The Coral Crown | neutral (C:13788) | - | Pelts, Weapons | Rum, Tobacco | - |
<!-- hand-maintained:end -->

- Lane lore: "The Iron Strait": "Iron from the foundries, soldiers from the fortress. This lane never sleeps." (C:1973).
- Lane lore: "The Northern Passage": "Iron west, porcelain east. The longest profitable route in the world." (C:2297).

## G. World details (undefined in data; all [invented] unless cited)

- History (shared timeline in README.md): T11: Ironhaven founds the Iron Pact with Stormwall; its foundries supply the garrison (bloc text). T15: Ironhaven weapons turn up at Corsair's Rest; the seller vanishes (grudge text). T11: The Great Foundry chimney is raised; it is seen twenty leagues out (landmark).
- Population [invented]: about 18,000.
- Language [invented]: Northern trade speech, clipped.
- Currency: silver is the only money in the sim (every price, fee and wage is in silver). Local coin names, if any, are [invented] flavour and never replace 'silver' in copy.
- Religion [invented]: the Hearth: the forge fire is kept lit in every guild hall; letting it die is a bad omen.
- Climate [invented port detail]: cold, wet, smoky; snow on the foundry roofs in winter. Regional weather [cite]: "Grey skies press low. The Atlantic rolls in long, heavy swells." (C:11782 area, weather_flavor).
- Pirates in these waters [cite]: The Crimson Tide (C:2879), The Iron Wolves (C:2905).

## I. Sound and ambience (all [invented] sound design unless quoted from a cite)

- **Soundscape:** "ringing hammers, and the deep bass hum of bellows forcing air into furnaces"; crane chains; spark crackle.
- **Signature:** hammer on anvil (short, dry - no ring-out per section 1); bellows breath; ingot set on barge deck.
- **Music/work songs (NOTE ONLY):** none; northern reticence (custom: a nod, not words).
- **Vs neighbours:** the only metal-industrial bed; Stormwall is military order, Thornport is wood smoke and fish.
- **Fragments [proposal; no fragment or asset exists yet]:** `src.anvil_hammer`, `src.bellows`, `place.foundry_quay`.
- Cited hooks: atmosphere "Coal smoke, ringing hammers, and the deep bass hum of bellows forcing air into furnaces."; dock scene "Iron-hulled barges sit low in the water. Cranes swing overhead, loading ingots onto merchant vessels. Sparks drift from the foundry like orange snowflakes." (C:12046 area).
- Invented bed idea: trip hammers, bellows, the foundry horn at shift change. No assets are requested.

## J. Flags for this port

- DF-5: Ironhaven embargoes the weapons it exports
- DF-8: NPC name drift across records
- DF-13: Silva Bay 'cheapest repairs' is a tie
- DF-17: Pirate captains sighted under ports that are not their homes
