# Crosswind Isle (crosswind_isle)

Base commit: 143e284 (content.json unchanged since 45b7378). Status: draft canon, phase 1. Written 2026-10-14.
Cites `C:n` are crates/portlight-sim/data/content.json line n at 143e284. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] (design-notes pattern) | DF-n see data-fixes.md | T-n see README shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Region | East Indies ("The Silk Waters") | C:1382, C:11892 |
| Bloc | The Free Ports (Crosswind Isle, Monsoon Reach, Thornport) | C:13644, C:13475 |
| Features | safe_harbor | C:1382-1484 |
| Warehouse tiers | depot, regional | C:6566 |
| Port fee / provisions / repair / crew | 6 / 2 / 3 per hull point / 5 | C:1382-1484 |
| Map | x 32, y 16 | C:1484 |
| Lanes | 8 | section F |
| Player description | "Free port at the junction of all trade winds. Everything passes through, nothing stays cheap." | C:1384 |
| Captain start here | The Dockhand, The Bounty Hunter | C:2587, C:2625 |

## A. Specialties and trade goods

<!-- hand-maintained:begin -->
| Good | Category | Base | Affinity | Stock target | Restock | Role |
|---|---|---|---|---|---|---|
| Grain | commodity | 12 | 1.0 | 15 | 1.5 | neutral |
| Timber | commodity | 18 | 1.0 | 12 | 1.2 | neutral |
| Iron Ore | commodity | 25 | 1.0 | 10 | 1.0 | neutral |
| Cotton | commodity | 15 | 1.0 | 10 | 1.0 | neutral |
| Spice | luxury | 55 | 1.0 | 10 | 1.0 | neutral |
| Silk | luxury | 70 | 1.0 | 8 | 0.8 | neutral |
| Rum | provision | 20 | 1.0 | 10 | 1.0 | neutral |
| Porcelain | luxury | 60 | 1.0 | 6 | 0.6 | neutral |
| Tea | luxury | 40 | 1.0 | 8 | 0.8 | neutral |
| Tobacco | provision | 28 | 1.0 | 8 | 0.8 | neutral |
| Pelts | commodity | 8 | 0.9 | 8 | 1.0 | importer |
<!-- hand-maintained:end -->

Market cite: C:1382-1484. Price rule: base x (target / stock) / affinity (crates/portlight-sim/src/economy.rs:37); affinity above 1 is cheap here.
Not traded here: Dyes, Pearls, Weapons, Medicines, Opium, Black Powder, Stolen Cargo.
Region goods [flavour, no mechanic]: sacred Porcelain, Silk; prized Spice, Tea; forbidden Weapons (C:11892 area).
Festival: Silk Weavers' Fair, every 75 days for 4, market effects Silk x2.0, Porcelain x1.5, Dyes x1.4 (C:11919).
Festival: Monsoon Prayer, every 90 days for 3, market effects Spice x1.8, Medicines x1.6, Tea x1.4 (C:11933).
Contract templates touching East Indies: `proc_grain_feed` (procurement), `proc_timber_shipyard` (procurement), `proc_iron_forge` (procurement), `proc_medicines_relief` (procurement), `lux_spice_monopolist` (luxury_discreet), `ret_cotton_return` (return_freight), `ret_spice_restock` (return_freight), `circ_indies_loop` (circuit), `smug_opium_run` (smuggling), `smug_powder_delivery` (smuggling). Source port Crosswind Isle: none.

## B. People

- Cultural group: The Free Port Council: "An elected body of captains who enforce the one rule: no one rules." (C:12146 area).
- Local custom: "All flags fly here. No nation claims the isle and all are welcome. That's the law." (C:12146 area).
- Regional ethos: "Patience is mastery. The artisan who perfects one bolt of silk in a lifetime is honored above the merchant who trades a thousand. Everything here was old when your civilization was young." Greeting: "You honor us, Captain. Tea will be prepared while we discuss terms." Proverb: "A merchant who rushes drinks salt water." (C:11892 area).
- Tavern rumour (in-world, unreliable): "Someone tried to claim the isle for an eastern dynasty last year. By morning, every captain in harbor had their guns trained on his ship. He left." (C:12146 area).

| Name | Title | Temperament | Institution | Cite |
|---|---|---|---|---|
| The Captain's Table | Governing Council | democratic | governor | C:10198 |
| Dock Master Tao | Dock Master | efficient | harbor_master | C:10220 |
| Every Merchant | Exchange Keeper | chaotic | exchange | C:10242 |
| Mother Ko | Keeper of the Peace | iron | tavern | C:10264 |
| Hassan al-Farsi | Money Changer | precise | customs | C:10286 |
| (Nobody) | (No Customs) | absent | apothecary | C:10308 |
| The Dock Brokers | Freelance Brokers | competitive | broker | C:10330 |

- Dress [invented]: every region's dress at once. The character hold stands: no likenesses or portraits.
- Speech [cite]: built from the greeting lines in each NPC record above (greeting_neutral, friendly, hostile).
- Temperament of the town: read from the power structure and tension in section E.
- Network (tavern, professional): Mother Ko - Old Enzo, linking Porto Novo (C:11556). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).

## C. Visual identity (guidance only; commissions nothing; the character hold stands)

Pattern: the shared visual-identity headings 1-7 (design notes). Shared camera, envelope and do/don't blocks are in README.md and not repeated here.

1. Architecture. Identity line [invented]: The Bell is the law: Crosswind Isle is a meeting place with no master. Landmark [cite]: "The Free Port Bell - rung once for every ship that enters, never for one that leaves." (C:12146 area). Cited buildings: The Free Dock (harbor_master, C:10126); The Open Market (exchange, C:10136); The Captain's Table (governor, C:10146); The Bell and Board (tavern, C:10156); The Empty Chair (apothecary, C:10166); Hassan's Table (customs, C:10176); The Dock Scrum (broker, C:10186). Kit use [proposal]: compose from the shipped will-use plates; identity carriers are ground choice and building frequency, never new ids. Gaps follow the art gap list (design notes). Landmark canon: ../landmarks/free_port_bell.md.
2. Materials: local and abundant (exports): none; scarce or luxury (imports): Pelts (section A). Atmosphere [cite]: "A dozen languages, cooking smoke from every cuisine, and the constant negotiation of trade." (C:12146 area). Climate: section G.
3. Palette (references, not UI tokens; UI stays INK / GOLD / CREAM / MUTED / DUE). Judged in sRGB against the plate envelope (luma 65-141, chroma p98 35):

| Role | Family | sRGB | Luma / chroma | Status | Tie |
|---|---|---|---|---|---|
| Dominant | weathered mixed timber | (133, 123, 111) #857B6F | 124 / 22 | in range | a port built from every region |
| Secondary 1 | wind-scoured stone | (114, 112, 106) #72706A | 112 / 8 | in range | the junction of trade winds |
| Secondary 2 | faded flag cloth | (104, 94, 96) #685E60 | 96 / 10 | in range | all flags fly |
| Accent | bell bronze | (145, 120, 81) #917851 | 122 / 64 | small-area accent (chroma 64) | the Free Port Bell |

4. Dress: see section B. 5. Signage: ASCII only, no UI-duplicating words; motifs [invented]: the bell; no flags of its own. 6. Silhouette: chart scale is the shared `chart_port_marker`; harbour-scale skyline [invented]: a bell tower over a harbour crowded with masts of every rig. 7. Do and don't: README shared list.

## D. Historical analogies (feed [invented] prose; never a cite)

- **Medieval free ports such as Livorno, as a neutral-port function.** Fits: a free port where all flags are welcome and no nation rules (custom). Suggests: neutrality enforced by every captain's guns; money changers everywhere. Limit: Livorno had a duke; Crosswind has the Captain's Table. Avoid: no single real nation.

## E. Governance

- Title: Captain's Table (C:10120).
- Power structure: "Crosswind Isle has no permanent government - the Captain's Table rotates monthly. Tao maintains the dock. Mother Ko enforces neutrality. Hassan changes money. The merchants shout. The brokers compete. The customs chair is empty. It's the closest thing to anarchy that actually works." [ascii-folded] (C:10122).
- Internal tension: "The tension IS the system. Crosswind Isle works because everyone agrees on one thing: no one rules. The moment someone tries, the system activates - Mother Ko throws them out, the Table votes them down, and every captain in harbor trains their guns. The real vulnerability: what happens when the trade blocs get desperate enough to claim the isle by force? The Table has no navy. Mother Ko has a stool." (C:10123).
- Bloc stance: "Crosswind Isle is neutral or dead. The Free Port Council enforces exactly one rule: no one rules." (C:13644 area).
- Grudge: "Someone tried to claim the isle for an eastern dynasty. Every captain in harbor trained their guns on his ship. He left." (C:13644 area).
- Bloc philosophy: "All flags fly here. No nation claims us." (C:13475 area). Loyalty and spillover numbers are [flavour, no mechanic] (DF-15).

## F. Relations with every other port

Hand-maintained from content.json. Disposition is bloc-to-bloc (symmetric in data); lanes are undirected; flows are directional by market affinity; networks are symmetric.

<!-- hand-maintained:begin -->
| Port | Region | Bloc | Disposition | Lane (distance, min class, danger) | Flows out (they import) | Flows in (we import) | Network |
|---|---|---|---|---|---|---|---|
| Porto Novo | Mediterranean | The Exchange Alliance | neutral (C:13739) | - | - | - | C:11556 |
| Al-Manar | Mediterranean | The Exchange Alliance | neutral (C:13739) | - | - | - | - |
| Silva Bay | Mediterranean | The Exchange Alliance | neutral (C:13739) | - | - | Pelts | - |
| Corsair's Rest | Mediterranean | The Shadow Ports | neutral (C:13844) | - | - | Pelts | - |
| Ironhaven | North Atlantic | The Iron Pact | neutral (C:13774) | - | - | Pelts | - |
| Stormwall | North Atlantic | The Iron Pact | neutral (C:13774) | - | - | - | - |
| Thornport | North Atlantic | The Free Ports | same bloc | - | - | Pelts | - |
| Sun Harbor | West Africa | The Gold Coast Compact | allied (C:13802) | 64, galleon, 0.18 (C:2099) | - | Pelts | - |
| Palm Cove | West Africa | The Gold Coast Compact | allied (C:13802) | - | - | - | - |
| Iron Point | West Africa | The Gold Coast Compact | allied (C:13802) | 60, brigantine, 0.16 (C:2108) | - | - | - |
| Pearl Shallows | West Africa | The Gold Coast Compact | allied (C:13802) | 56, brigantine, 0.15 (C:2117) | - | - | - |
| Jade Port | East Indies | The Silk Circle | neutral (C:13823) | 28, sloop, 0.1 (C:2126) | - | - | - |
| Monsoon Reach | East Indies | The Free Ports | same bloc | 24, sloop, 0.09 (C:2135) | - | - | - |
| Silk Haven | East Indies | The Silk Circle | neutral (C:13823) | 32, brigantine, 0.12 (C:2144) | - | - | - |
| Dragon's Gate | East Indies | The Silk Circle | neutral (C:13823) | 30, brigantine, 0.11 (C:2153) | - | - | - |
| Spice Narrows | East Indies | The Shadow Ports | neutral (C:13844) | - | - | - | - |
| Ember Isle | South Seas | The Coral Crown | neutral (C:13851) | 56, galleon, 0.19 (C:2243) | - | - | - |
| Typhoon Anchorage | South Seas | The Coral Crown | neutral (C:13851) | - | - | - | - |
| Coral Throne | South Seas | The Coral Crown | neutral (C:13851) | - | - | - | - |
<!-- hand-maintained:end -->

- Lane lore: "The Long Crossing": "Months of open water. The route that separates traders from merchants. Many set out. Fewer arrive." (C:2099). Lane canon: [The Long Crossing](../lanes/long_crossing.md).
- Lane lore: "The Porcelain Lane": "From the free port to the kiln masters. Every piece of porcelain in the west passed through here." (C:2126). Lane canon: [The Porcelain Lane](../lanes/porcelain_lane.md).

## G. World details (undefined in data; all [invented] unless cited)

- History (shared timeline in README.md): T12: The Captain's Table charter makes the isle a free port [invented date]. T12: A claimant for an eastern dynasty is seen off by every gun in harbour (grudge text).
- Population [invented]: about 8,000 residents and as many sailors in port.
- Language [invented]: every trade speech; a pidgin of all five regions.
- Currency: silver is the only money in the sim (every price, fee and wage is in silver). Local coin names, if any, are [invented] flavour and never replace 'silver' in copy.
- Religion [invented]: none official; every faith's shrine on one street.
- Climate [invented port detail]: windy junction of the trade winds; squalls but rarely storms. Regional weather [cite]: "Incense drifts from a passing junk. The air is thick with spice." (C:11892 area, weather_flavor).
- Pirates in these waters [cite]: The Monsoon Syndicate (C:2888).

## I. Sound and ambience (all [invented] sound design unless quoted from a cite)

- **Soundscape:** crowded free port; "A dozen languages" (C:12149), cooking smoke; ships jostling; flags snapping.
- **Signature:** money-changer's coin table set down; many-flag halyard flap; the Free Port Bell tolling on entry (ambience only).
- **Music/work songs (NOTE ONLY):** none; the language mix is walla - on hold.
- **Vs neighbours:** busiest, most mixed East Indies bed; Dragon's Gate is disciplined, Spice Narrows is hushed.
- **Fragments [proposal; no fragment or asset exists yet]:** `src.changer_table`, `src.flag_snap`, `place.free_port_quay`.
- Cited hooks: atmosphere "A dozen languages, cooking smoke from every cuisine, and the constant negotiation of trade." (C:12149); dock scene "Ships from every region jostle for berth. The Free Port Bell tolls as you enter. Money-changers set up tables on the dock before your anchor hits bottom." (C:12150).
- Invented bed idea: the Free Port Bell for every arrival; many languages at once. No assets are requested.

## J. Flags for this port

- DF-3: Bounty hunter home region does not match home port
- Resolved in lore: The bounty hunter starts here as an East Indies captain (DF-3).
