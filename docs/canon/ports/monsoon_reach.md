# Monsoon Reach (monsoon_reach)

Base commit: 143e284 (content.json unchanged since 45b7378). Status: draft canon, phase 1. Written 2026-10-14.
Cites `C:n` are crates/portlight-sim/data/content.json line n at 143e284. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] (design-notes pattern) | DF-n see data-fixes.md | T-n see README shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Region | East Indies ("The Silk Waters") | C:1206, C:11892 |
| Bloc | The Free Ports (Crosswind Isle, Monsoon Reach, Thornport) | C:13653, C:13475 |
| Features | shipyard | C:1206-1292 |
| Warehouse tiers | depot, regional, commercial | C:6557 |
| Port fee / provisions / repair / crew | 8 / 2 / 2 per hull point / 6 | C:1206-1292 |
| Map | x 38, y 14 | C:1292 |
| Lanes | 6 | section F |
| Player description | "Seasonal winds funnel the spice trade through this crossroads. The shipyard builds for endurance." | C:1208 |
| Captain start here | The Scholar | C:2511 |

## A. Specialties and trade goods

<!-- hand-maintained:begin -->
| Good | Category | Base | Affinity | Stock target | Restock | Role |
|---|---|---|---|---|---|---|
| Spice | luxury | 55 | 1.4 | 22 | 2.5 | exporter |
| Silk | luxury | 70 | 1.2 | 18 | 2.0 | exporter |
| Medicines | medicine | 35 | 0.9 | 10 | 1.0 | importer |
| Porcelain | luxury | 60 | 0.8 | 10 | 1.0 | importer |
| Pelts | commodity | 8 | 0.8 | 8 | 0.8 | importer |
| Rum | provision | 20 | 0.6 | 10 | 1.0 | importer |
| Grain | commodity | 12 | 0.6 | 10 | 1.0 | importer |
| Cotton | commodity | 15 | 0.5 | 8 | 0.8 | importer |
| Timber | commodity | 18 | 0.45 | 8 | 0.8 | importer |
<!-- hand-maintained:end -->

Market cite: C:1206-1292. Price rule: base x (target / stock) / affinity (crates/portlight-sim/src/economy.rs:37); affinity above 1 is cheap here.
Not traded here: Iron Ore, Dyes, Tea, Pearls, Tobacco, Weapons, Opium, Black Powder, Stolen Cargo.
Region goods [flavour, no mechanic]: sacred Porcelain, Silk; prized Spice, Tea; forbidden Weapons (C:11892 area).
Festival: Silk Weavers' Fair, every 75 days for 4, market effects Silk x2.0, Porcelain x1.5, Dyes x1.4 (C:11919).
Festival: Monsoon Prayer, every 90 days for 3, market effects Spice x1.8, Medicines x1.6, Tea x1.4 (C:11933).
Contract templates touching East Indies: `proc_grain_feed` (procurement), `proc_timber_shipyard` (procurement), `proc_iron_forge` (procurement), `proc_medicines_relief` (procurement), `lux_spice_monopolist` (luxury_discreet), `ret_cotton_return` (return_freight), `ret_spice_restock` (return_freight), `circ_indies_loop` (circuit), `smug_opium_run` (smuggling), `smug_powder_delivery` (smuggling). Source port Monsoon Reach: none.

## B. People

- Cultural group: The Monsoon Brotherhood: "Monks and sailors who read the wind. Their forecasts guide every departure." (C:12126 area).
- Local custom: "Captains consult the monks before sailing. Their wind forecasts are uncannily accurate." (C:12126 area).
- Regional ethos: "Patience is mastery. The artisan who perfects one bolt of silk in a lifetime is honored above the merchant who trades a thousand. Everything here was old when your civilization was young." Greeting: "You honor us, Captain. Tea will be prepared while we discuss terms." Proverb: "A merchant who rushes drinks salt water." (C:11892 area).
- Tavern rumour (in-world, unreliable): "The monks say this monsoon season will be the worst in a generation. The shipwrights are quietly raising their prices." (C:12126 area).

| Name | Title | Temperament | Institution | Cite |
|---|---|---|---|---|
| Harbor Master Ravi | Harbor Master | calm | harbor_master | C:9728 |
| Brother Anand | Wind Reader | enigmatic | apothecary | C:9750 |
| Shipwright Devi | Master Shipwright | inventive | shipyard | C:9772 |
| Priya Sundaram | Spice Factor | pragmatic | exchange | C:9794 |
| The Harbor Council | Governing Council | deliberate | governor | C:9816 |
| Customs Officer Sanjay | Customs Officer | easygoing | customs | C:9838 |
| Kamala Nair | Broker | fluid | broker | C:9860 |
| Devi | Armorer (merchant) | charming | outfitter | C:6017 |
| Dewi Ayu | style master | - | - | C:6229 |

- Dress [invented]: light cotton; monks in plain robes. The character hold stands: no likenesses or portraits.
- Speech [cite]: built from the greeting lines in each NPC record above (greeting_neutral, friendly, hostile).
- Temperament of the town: read from the power structure and tension in section E.
- Network (merchant, rival): Yasmin al-Nadir - Priya Sundaram, linking Al-Manar (C:11496). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Network (broker, allied): Kamala Nair - Fatou Ndiaye, linking Sun Harbor (C:11616). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Sea sighting filed under this port: Brother Anand (C:12888).

## C. Visual identity (guidance only; commissions nothing; the character hold stands)

Pattern: the shared visual-identity headings 1-7 (design notes). Shared camera, envelope and do/don't blocks are in README.md and not repeated here.

1. Architecture. Identity line [invented]: The Wind Temple keeps the calendar: Monsoon Reach is a port timed by the monsoon. Landmark [cite]: "The Wind Temple - a pagoda on the headland where monks track the monsoon." (C:12126 area). Cited buildings: The Headland Harbor (harbor_master, C:9656); The Wind Temple (apothecary, C:9666); The Monsoon Yard (shipyard, C:9676); The Spice Crossroads (exchange, C:9686); The Temple Hall (governor, C:9696); The Flow Desk (customs, C:9706); The Bridge Desk (broker, C:9716). Kit use [proposal]: compose from the shipped will-use plates; identity carriers are ground choice and building frequency, never new ids. Gaps follow the art gap list (design notes). Landmark canon: ../landmarks/wind_temple.md.
2. Materials: local and abundant (exports): Spice, Silk; scarce or luxury (imports): Cotton, Timber, Rum, Medicines, Grain, Porcelain, Pelts (section A). Atmosphere [cite]: "Salt spray, incense from the Wind Temple, and the creak of monsoon-tested rigging." (C:12126 area). Climate: section G.
3. Palette (references, not UI tokens; UI stays INK / GOLD / CREAM / MUTED / DUE). Judged in sRGB against the plate envelope (luma 65-141, chroma p98 35):

| Role | Family | sRGB | Luma / chroma | Status | Tie |
|---|---|---|---|---|---|
| Dominant | teak and lacquer, muted | (111, 91, 77) #6F5B4D | 94 / 34 | in range | the shipyard and temple |
| Secondary 1 | monsoon cloud grey | (110, 112, 116) #6E7074 | 112 / 6 | in range | seasonal winds |
| Secondary 2 | jungle green, muted | (76, 92, 74) #4C5C4A | 87 / 18 | in range | headland |
| Accent | rain-dark temple tile, small | (84, 96, 124) #54607C | 95 / 40 | small-area accent (chroma 40) | the Wind Temple |

4. Dress: see section B. 5. Signage: ASCII only, no UI-duplicating words; motifs [invented]: a wind vane. 6. Silhouette: chart scale is the shared `chart_port_marker`; harbour-scale skyline [invented]: a pagoda on the headland above slipways and spice godowns. 7. Do and don't: README shared list.

## D. Historical analogies (feed [invented] prose; never a cite)

- **Monsoon entrepots such as Calicut, as a seasonal-wind function.** Fits: a crossroads where seasonal winds funnel the spice trade (description). Suggests: consult the wind readers before sailing; the yard builds for endurance. Limit: borrow the seasonal rhythm only. Avoid: ethnic costume.

## E. Governance

- Title: Harbor Council (C:9650).
- Power structure: "Monsoon Reach is governed by a five-person council (2 monks, 2 merchants, 1 shipwright) that meets in the Wind Temple. Brother Anand's forecasts are de facto law - no ship departs without his clearance. Ravi runs the harbor as an extension of the Temple's teachings. Devi builds ships to survive what Anand predicts. Priya moves the spice. Kamala connects the world. The power structure is unusually harmonious because everyone agrees on the port's purpose: trade in harmony with the monsoon." [ascii-folded] (C:9652).
- Internal tension: "The tension is between faith and commerce. The monks believe the wind speaks truth and should be obeyed. The merchants believe the wind is data to be exploited. When Anand closes the harbor for a dream, the merchants lose money. When Anand's dreams prove accurate, the merchants are grateful. This cycle - frustration, then vindication - keeps the balance intact but never fully resolved. The deeper thread: Devi's correspondence with Elena in Silva Bay is building a cross-hemisphere shipwright relationship that could transform both yards. If they ever meet, the collaboration could produce ships neither culture imagined alone." (C:9653).
- Bloc stance: "Monsoon Reach trades with everyone and judges no one. The Wind Temple monks believe commerce is prayer." (C:13653 area).
- Grudge: "The Monsoon Brotherhood predicted a typhoon that saved a Silk Circle fleet. The Circle offered alliance. The monks declined. 'We read the wind, not politics.'" (C:13653 area).
- Bloc philosophy: "All flags fly here. No nation claims us." (C:13475 area). Loyalty and spillover numbers are [flavour, no mechanic] (DF-15).

## F. Relations with every other port

Hand-maintained from content.json. Disposition is bloc-to-bloc (symmetric in data); lanes are undirected; flows are directional by market affinity; networks are symmetric.

<!-- hand-maintained:begin -->
| Port | Region | Bloc | Disposition | Lane (distance, min class, danger) | Flows out (they import) | Flows in (we import) | Network |
|---|---|---|---|---|---|---|---|
| Porto Novo | Mediterranean | The Exchange Alliance | neutral (C:13739) | - | - | Grain, Rum | - |
| Al-Manar | Mediterranean | The Exchange Alliance | neutral (C:13739) | 72, galleon, 0.22 (C:2279) | Silk | Medicines | C:11496 |
| Silva Bay | Mediterranean | The Exchange Alliance | neutral (C:13739) | - | Silk | Pelts, Timber | - |
| Corsair's Rest | Mediterranean | The Shadow Ports | neutral (C:13844) | - | Silk | Pelts, Rum | - |
| Ironhaven | North Atlantic | The Iron Pact | neutral (C:13774) | - | Silk | Pelts | - |
| Stormwall | North Atlantic | The Iron Pact | neutral (C:13774) | - | Silk | Grain, Medicines | - |
| Thornport | North Atlantic | The Free Ports | same bloc | - | Silk, Spice | Pelts, Timber | - |
| Sun Harbor | West Africa | The Gold Coast Compact | allied (C:13802) | - | Silk, Spice | Cotton, Pelts | C:11616 |
| Palm Cove | West Africa | The Gold Coast Compact | allied (C:13802) | - | Silk | Cotton, Rum | - |
| Iron Point | West Africa | The Gold Coast Compact | allied (C:13802) | - | - | - | - |
| Pearl Shallows | West Africa | The Gold Coast Compact | allied (C:13802) | - | Silk | Cotton | - |
| Jade Port | East Indies | The Silk Circle | neutral (C:13823) | 20, sloop, 0.08 (C:2162) | - | Porcelain | - |
| Silk Haven | East Indies | The Silk Circle | neutral (C:13823) | 22, sloop, 0.1 (C:2189) | Spice | Porcelain | - |
| Crosswind Isle | East Indies | The Free Ports | same bloc | 24, sloop, 0.09 (C:2135) | - | - | - |
| Dragon's Gate | East Indies | The Silk Circle | neutral (C:13823) | - | - | Porcelain | - |
| Spice Narrows | East Indies | The Shadow Ports | neutral (C:13844) | 26, brigantine, 0.13 (C:2198) | Silk | - | - |
| Ember Isle | South Seas | The Coral Crown | neutral (C:13851) | - | - | Medicines | - |
| Typhoon Anchorage | South Seas | The Coral Crown | neutral (C:13851) | 52, galleon, 0.2 (C:2225) | Silk | Timber | - |
| Coral Throne | South Seas | The Coral Crown | neutral (C:13851) | - | Silk | Rum | - |
<!-- hand-maintained:end -->

- Lane lore: "Typhoon Alley": "Named for obvious reasons. The monsoon winds funnel through here like a gauntlet. Timing is survival." (C:2225). Lane canon: [Typhoon Alley](../lanes/typhoon_alley.md).
- Lane lore: "The Monsoon Shortcut": "Timing is everything. Catch the monsoon right and you fly. Catch it wrong and you drown." (C:2279). Lane canon: [The Monsoon Shortcut](../lanes/monsoon_shortcut.md).

## G. World details (undefined in data; all [invented] unless cited)

- History (shared timeline in README.md): T12: Monsoon Reach joins the Free Ports; the monks decline a Silk Circle alliance (grudge text). T14: The Monsoon Shortcut ties the Reach to Al-Manar (route C).
- Population [invented]: about 12,000, doubling in trading season.
- Language [invented]: Silk Waters trade speech.
- Currency: silver is the only money in the sim (every price, fee and wage is in silver). Local coin names, if any, are [invented] flavour and never replace 'silver' in copy.
- Religion [invented]: the Wind Temple's monks read the monsoon (landmark).
- Climate [invented port detail]: two monsoons a year; dry clear months between. Regional weather [cite]: "Incense drifts from a passing junk. The air is thick with spice." (C:11892 area, weather_flavor).
- Pirates in these waters [cite]: The Monsoon Syndicate (C:2888).

## I. Sound and ambience (all [invented] sound design unless quoted from a cite)

- **Soundscape:** "Salt spray, incense ... and the creak of monsoon-tested rigging" (C:12129); wind round a headland.
- **Signature:** heavy rigging creak under load; deep-keel hull settling; temple incense is smell only; the lower-hall pagoda bells (C:9700) stay distant ambience, never a cue.
- **Sea mark:** "The Wind Temple pagoda catches the sunset. Monsoon Reach is near." (crates/portlight-sim/src/voyage.rs:596); see ../landmarks/wind_temple.md
- **Music/work songs (NOTE ONLY):** monk chant would be invented; canon names forecasts, not chant - avoid.
- **Vs neighbours:** wind-and-rope port; vs Typhoon Anchorage, steady seasonal wind, not storm.
- **Fragments [proposal; no fragment or asset exists yet]:** `src.rigging_strain`, `place.headland_harbour`.
- Cited hooks: atmosphere "Salt spray, incense from the Wind Temple, and the creak of monsoon-tested rigging." (C:12129); dock scene "The harbor curves around a headland crowned by the Wind Temple. Ships here are built with deeper keels and stronger masts - the monsoon demands it." (C:12130).
- Invented bed idea: temple wind chimes, rain on palm roofs, the yard. No assets are requested.

## J. Flags for this port

- none beyond the shared ones in data-fixes.md.
