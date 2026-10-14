# Dragon's Gate (dragons_gate)

Base commit: 143e284 (content.json unchanged since 45b7378). Status: draft canon, phase 1. Written 2026-10-14.
Cites `C:n` are crates/portlight-sim/data/content.json line n at 143e284. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] (design-notes pattern) | DF-n see data-fixes.md | T-n see README shared timeline.
Depth: one of the five thinnest ports in data (inventory 'Undefined'); section H adds extra invented depth.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Region | East Indies ("The Silk Waters") | C:1487, C:11892 |
| Bloc | The Silk Circle (Jade Port, Silk Haven, Dragon's Gate) | C:13633, C:13453 |
| Features | none | C:1487-1563 |
| Warehouse tiers | none listed | C:6520 |
| Port fee / provisions / repair / crew | 9 / 2 / 3 per hull point / 7 | C:1487-1563 |
| Map | x 44, y 12 | C:1563 |
| Lanes | 3 | section F |
| Player description | "Fortress harbor controlling the eastern straits. Weapons are contraband here, but medicines are gold." | C:1489 |

## A. Specialties and trade goods

<!-- hand-maintained:begin -->
| Good | Category | Base | Affinity | Stock target | Restock | Role |
|---|---|---|---|---|---|---|
| Tea | luxury | 40 | 1.5 | 25 | 3.0 | exporter |
| Porcelain | luxury | 60 | 1.3 | 18 | 2.0 | exporter |
| Pelts | commodity | 8 | 0.7 | 6 | 0.5 | importer |
| Cotton | commodity | 15 | 0.65 | 8 | 0.8 | importer |
| Rum | provision | 20 | 0.6 | 8 | 0.8 | importer |
| Iron Ore | commodity | 25 | 0.5 | 8 | 0.8 | importer |
| Tobacco | provision | 28 | 0.5 | 6 | 0.5 | importer |
| Medicines | medicine | 35 | 0.4 | 5 | 0.5 | importer |
<!-- hand-maintained:end -->

Market cite: C:1487-1563. Price rule: base x (target / stock) / affinity (crates/portlight-sim/src/economy.rs:37); affinity above 1 is cheap here.
Not traded here: Grain, Timber, Dyes, Spice, Silk, Pearls, Weapons, Opium, Black Powder, Stolen Cargo.
Region goods [flavour, no mechanic]: sacred Porcelain, Silk; prized Spice, Tea; forbidden Weapons (C:11892 area).
Festival: Silk Weavers' Fair, every 75 days for 4, market effects Silk x2.0, Porcelain x1.5, Dyes x1.4 (C:11919).
Festival: Monsoon Prayer, every 90 days for 3, market effects Spice x1.8, Medicines x1.6, Tea x1.4 (C:11933).
Contract templates touching East Indies: `proc_grain_feed` (procurement), `proc_timber_shipyard` (procurement), `proc_iron_forge` (procurement), `proc_medicines_relief` (procurement), `lux_spice_monopolist` (luxury_discreet), `ret_cotton_return` (return_freight), `ret_spice_restock` (return_freight), `circ_indies_loop` (circuit), `smug_opium_run` (smuggling), `smug_powder_delivery` (smuggling). Source port Dragon's Gate: none.

## B. People

- Cultural group: The Gate Wardens: "Military governors who control the strait. They are the law east of the gate." (C:12156 area).
- Local custom: "Weapons in cargo are declared or confiscated. There is no third option." (C:12156 area).
- Regional ethos: "Patience is mastery. The artisan who perfects one bolt of silk in a lifetime is honored above the merchant who trades a thousand. Everything here was old when your civilization was young." Greeting: "You honor us, Captain. Tea will be prepared while we discuss terms." Proverb: "A merchant who rushes drinks salt water." (C:11892 area).
- Tavern rumour (in-world, unreliable): "The fortress commander has not lowered the chains in fifteen years. The last captain who tried to run them is still chained to the seabed as a warning." (C:12156 area).

| Name | Title | Temperament | Institution | Cite |
|---|---|---|---|---|
| Commander Zhang Wei | Gate Commander | absolute | governor | C:10432 |
| Harbor Captain Li Jun | Harbor Captain | precise | harbor_master | C:10454 |
| Tea Merchant Liu | Tea Factor | patient | exchange | C:10476 |
| Inspector Sun | Weapons Inspector | relentless | customs | C:10498 |
| Healer Chen Ling | Fortress Healer | dedicated | apothecary | C:10520 |
| Inn Keeper Wu | Inn Keeper | discreet | tavern | C:10542 |
| Broker Ming | Gate Broker | cautious | broker | C:10564 |

- Dress [invented]: garrison lacquered leather in muted brown; inspectors' tabards; civilians in plain cotton. The character hold stands: no likenesses or portraits.
- Speech [cite]: built from the greeting lines in each NPC record above (greeting_neutral, friendly, hostile).
- Temperament of the town: read from the power structure and tension in section E.
- Network (merchant, professional): Sigrid Halvdan - Tea Merchant Liu, linking Thornport (C:11460). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Network (inspector, professional): Inspector Sun - Inspector Kross, linking Ironhaven (C:11700). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Network (inspector, respected): Inspector Salva - Inspector Sun, linking Porto Novo (C:11712). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).

## C. Visual identity (guidance only; commissions nothing; the character hold stands)

Pattern: the shared visual-identity headings 1-7 (design notes). Shared camera, envelope and do/don't blocks are in README.md and not repeated here.

1. Architecture. Identity line [invented]: The Gate is the town: Dragon's Gate exists to open or close a strait. Landmark [cite]: "The Gate Fortress - twin stone towers flanking the strait, chains ready to close the passage." (C:12156 area). Cited buildings: The Chain Harbor (harbor_master, C:10360); The Tea Terraces (exchange, C:10370); The Gate Command (governor, C:10380); The Gate Inn (tavern, C:10390); The Strip Room (customs, C:10400); The Strait Infirmary (apothecary, C:10410); The Permitted Desk (broker, C:10420). Kit use [proposal]: compose from the shipped will-use plates; identity carriers are ground choice and building frequency, never new ids. Gaps follow the art gap list (design notes).
2. Materials: local and abundant (exports): Tea, Porcelain; scarce or luxury (imports): Medicines, Iron Ore, Tobacco, Rum, Cotton, Pelts (section A). Atmosphere [cite]: "Military precision, jasmine tea, and the distant clank of the harbor chains." (C:12156 area). Climate: section G.
3. Palette (references, not UI tokens; UI stays INK / GOLD / CREAM / MUTED / DUE). Judged in sRGB against the plate envelope (luma 65-141, chroma p98 35):

| Role | Family | sRGB | Luma / chroma | Status | Tie |
|---|---|---|---|---|---|
| Dominant | dark fortress stone | (98, 96, 94) #62605E | 96 / 4 | in range | twin stone towers |
| Secondary 1 | chain iron | (84, 86, 88) #545658 | 86 / 4 | in range | chains ready to close the passage |
| Secondary 2 | strait fog | (118, 120, 122) #76787A | 120 / 4 | in range | [invented] morning mist |
| Accent | lacquer red, small | (132, 84, 72) #845448 | 93 / 60 | in range, small areas only | tower banners |

4. Dress: see section B. 5. Signage: ASCII only, no UI-duplicating words; motifs [invented]: crossed chains; declared-cargo tallies. 6. Silhouette: chart scale is the shared `chart_port_marker`; harbour-scale skyline [invented]: two stone towers either side of the strait with a chain sagging between them; a walled harbour below. 7. Do and don't: README shared list.

## D. Historical analogies (feed [invented] prose; never a cite)

- **Constantinople's Golden Horn chain, as a strait-control function.** Fits: twin towers and a chain that can close a strait (landmark). Suggests: declare or lose; inspections as ritual; soldiers as civil service. Limit: borrow the chain and the function. Avoid: no specific empire or dynasty.
- **Elsinore and the Sound Dues, as a toll strait.** Fits: a fortress collecting dues and inspecting every hull. Suggests: cargo declared at the tower; captains wait their turn. Limit: no toll mechanic in the sim [flavour, no mechanic]. Avoid: Hamlet references.
- **A walled garrison town.** Fits: a fortress port where medicines are gold and weapons contraband (description). Suggests: austere inns, a fortress healer, tea as the only luxury. Limit: the garrison is the town's only employer. Avoid: martial-arts cliches.
- Contrast, not model: Crosswind Isle welcomes all flags; the Gate admits one hull at a time.

## E. Governance

- Title: Gate Commander (C:10354).
- Power structure: "Dragon's Gate is Zhang's fortress. His word is absolute. Li Jun runs the harbor chains. Sun finds the weapons. Liu produces the tea that funds everything. Chen Ling heals. Wu watches and says nothing. Ming brokers what's permitted. Every institution exists within Zhang's boundaries - and those boundaries have not moved in fifteen years." [ascii-folded] (C:10356).
- Internal tension: "The tension is between security and commerce. Zhang wants zero weapons through the strait - absolute control. Liu needs trade to fund the fortress. Ming needs contracts to exist. The equilibrium works because Zhang's strictness makes the strait safe, which makes it valuable, which brings trade. Break any link and it collapses. Wu is the wildcard: his possible Syndicate connections are the one variable Zhang can't fully control, and he keeps Wu precisely because a known risk is better than an unknown one." (C:10357).
- Bloc stance: "Dragon's Gate enforces the Circle's weapons ban. The Gate Wardens are the Circle's iron fist." (C:13633 area).
- Trade embargo (data, unread by the sim; crates/portlight-sim/src/content.rs:676): Weapons.
- Grudge: "The Iron Wolves tried to smuggle weapons through the Gate. Fifteen Wolf sailors are still chained to the seabed." (C:13633 area).
- Bloc philosophy: "Quality over quantity. Hierarchy governs all." (C:13453 area). Loyalty and spillover numbers are [flavour, no mechanic] (DF-15).

## F. Relations with every other port

Hand-maintained from content.json. Disposition is bloc-to-bloc (symmetric in data); lanes are undirected; flows are directional by market affinity; networks are symmetric.

<!-- hand-maintained:begin -->
| Port | Region | Bloc | Disposition | Lane (distance, min class, danger) | Flows out (they import) | Flows in (we import) | Network |
|---|---|---|---|---|---|---|---|
| Porto Novo | Mediterranean | The Exchange Alliance | rival (C:13732) | - | Porcelain | Rum | C:11712 |
| Al-Manar | Mediterranean | The Exchange Alliance | rival (C:13732) | - | Porcelain, Tea | Medicines | - |
| Silva Bay | Mediterranean | The Exchange Alliance | rival (C:13732) | - | - | Iron Ore, Pelts | - |
| Corsair's Rest | Mediterranean | The Shadow Ports | rival (C:13830) | - | - | Pelts, Rum, Tobacco | - |
| Ironhaven | North Atlantic | The Iron Pact | hostile (C:13767) | - | - | Iron Ore, Pelts | C:11700 |
| Stormwall | North Atlantic | The Iron Pact | hostile (C:13767) | - | - | Medicines | - |
| Thornport | North Atlantic | The Free Ports | neutral (C:13823) | - | Porcelain | Pelts, Tobacco | C:11460 |
| Sun Harbor | West Africa | The Gold Coast Compact | neutral (C:13795) | - | Porcelain | Cotton, Iron Ore, Pelts | - |
| Palm Cove | West Africa | The Gold Coast Compact | neutral (C:13795) | - | - | Cotton, Rum, Tobacco | - |
| Iron Point | West Africa | The Gold Coast Compact | neutral (C:13795) | - | Porcelain | Iron Ore | - |
| Pearl Shallows | West Africa | The Gold Coast Compact | neutral (C:13795) | - | - | Cotton | - |
| Jade Port | East Indies | The Silk Circle | same bloc | 22, sloop, 0.09 (C:2180) | - | - | - |
| Monsoon Reach | East Indies | The Free Ports | neutral (C:13823) | - | Porcelain | - | - |
| Silk Haven | East Indies | The Silk Circle | same bloc | - | - | - | - |
| Crosswind Isle | East Indies | The Free Ports | neutral (C:13823) | 30, brigantine, 0.11 (C:2153) | - | - | - |
| Spice Narrows | East Indies | The Shadow Ports | rival (C:13830) | 28, brigantine, 0.14 (C:2216) | - | - | - |
| Ember Isle | South Seas | The Coral Crown | allied (C:13837) | - | - | Iron Ore, Medicines | - |
| Typhoon Anchorage | South Seas | The Coral Crown | allied (C:13837) | - | Porcelain | - | - |
| Coral Throne | South Seas | The Coral Crown | allied (C:13837) | - | Porcelain, Tea | Rum, Tobacco | - |
<!-- hand-maintained:end -->


## G. World details (undefined in data; all [invented] unless cited)

- History (shared timeline in README.md): T7: The twin towers and the chain are raised when the Silk Circle forms [invented date]. T13: The Iron Wolves try to run weapons through the Gate; fifteen are still chained to the seabed (grudge text).
- Population [invented]: about 4,000, mostly garrison and their families.
- Language [invented]: Silk Waters trade speech; tower signal calls.
- Currency: silver is the only money in the sim (every price, fee and wage is in silver). Local coin names, if any, are [invented] flavour and never replace 'silver' in copy.
- Religion [invented]: ancestor halls; a shrine at the chain winch where the gate crew pray before closing it.
- Climate [invented port detail]: strait winds and fog; the towers often vanish in morning mist. Regional weather [cite]: "Incense drifts from a passing junk. The air is thick with spice." (C:11892 area, weather_flavor).
- Pirates in these waters [cite]: The Monsoon Syndicate (C:2888).

## H. Extra depth (thin port)

- Daily life [invented]: the chain is raised at dusk and lowered at dawn; ships that arrive after dusk anchor under the towers' lamps until morning.
- Factions inside the Gate [invented]: the tower crews, the inspectors and the inn keepers. The inspectors hold the real power because they decide what counts as a weapon.
- A local law [invented]: medicines pass duty-free (consistent with 'medicines are gold', description). Weapons are declared or confiscated (custom, cited).

## I. Sound and ambience (all [invented] sound design unless quoted from a cite)

- **Soundscape:** "military precision, jasmine tea, and the distant clank of the harbor chains"; lacquered armour moving.
- **Signature:** harbour chain clank; lacquer-armour plate creak; tea poured on the inner docks.
- **Music/work songs (NOTE ONLY):** none in canon.
- **Vs neighbours:** the strait fortress; chains + lacquer vs Stormwall's gates + drums; patience vs Crosswind's bustle.
- **Fragments [proposal; no fragment or asset exists yet]:** `src.harbour_chain`, `mat.lacquer_armour`, `place.strait_fort`.
- Cited hooks: atmosphere "Military precision, jasmine tea, and the distant clank of the harbor chains."; dock scene "Soldiers in lacquered armor inspect every ship that passes the twin towers. Tea merchants wait patiently on the inner docks, knowing inspection takes time." (C:12156 area).
- Invented bed idea: the chain winch, the tower gong at closing, inspectors' calls. No assets are requested.

## J. Flags for this port

- none beyond the shared ones in data-fixes.md.
