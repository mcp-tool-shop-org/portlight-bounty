# Thornport (thornport)

Base commit: 143e284 (content.json unchanged since 45b7378). Status: draft canon, phase 1. Written 2026-10-14.
Cites `C:n` are crates/portlight-sim/data/content.json line n at 143e284. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] (design-notes pattern) | DF-n see data-fixes.md | T-n see README shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Region | North Atlantic ("The Iron Coast") | C:724, C:11782 |
| Bloc | The Free Ports (Crosswind Isle, Monsoon Reach, Thornport) | C:13662, C:13475 |
| Features | none | C:724-800 |
| Warehouse tiers | none listed | C:6520 |
| Port fee / provisions / repair / crew | 5 / 1 / 3 per hull point / 3 | C:724-800 |
| Map | x 11, y 10 | C:800 |
| Lanes | 2 | section F |
| Player description | "Whaling town turned trading post. Tea and tobacco are the local currency. Medicines fetch gold." | C:726 |

## A. Specialties and trade goods

<!-- hand-maintained:begin -->
| Good | Category | Base | Affinity | Stock target | Restock | Role |
|---|---|---|---|---|---|---|
| Tobacco | provision | 28 | 1.6 | 25 | 3.0 | exporter |
| Tea | luxury | 40 | 1.5 | 22 | 2.5 | exporter |
| Pelts | commodity | 8 | 1.5 | 15 | 2.0 | exporter |
| Timber | commodity | 18 | 1.2 | 18 | 2.0 | exporter |
| Silk | luxury | 70 | 0.5 | 5 | 0.5 | importer |
| Spice | luxury | 55 | 0.5 | 6 | 0.6 | importer |
| Porcelain | luxury | 60 | 0.5 | 4 | 0.3 | importer |
| Medicines | medicine | 35 | 0.45 | 8 | 0.8 | importer |
<!-- hand-maintained:end -->

Market cite: C:724-800. Price rule: base x (target / stock) / affinity (crates/portlight-sim/src/economy.rs:37); affinity above 1 is cheap here.
Not traded here: Grain, Iron Ore, Cotton, Dyes, Pearls, Rum, Weapons, Opium, Black Powder, Stolen Cargo.
Region goods [flavour, no mechanic]: sacred Medicines; prized Tea, Tobacco, Weapons; forbidden none (C:11782 area).
Festival: The Iron Days, every 75 days for 4, market effects Iron Ore x1.6, Weapons x1.8, Timber x1.3 (C:11807).
Festival: Midwinter Market, every 90 days for 3, market effects Medicines x2.0, Tea x1.7, Tobacco x1.5 (C:11821).
Contract templates touching North Atlantic: `proc_weapons_garrison` (procurement), `short_tea_crisis` (shortage), `ret_cotton_return` (return_freight), `ret_spice_restock` (return_freight), `ret_dyes_textile` (return_freight), `circ_south_seas` (circuit), `rep_charter_atlantic` (reputation_charter), `smug_opium_run` (smuggling). Source port Thornport: none.

## B. People

- Cultural group: The Whaler-Merchants: "Former whalers turned traders. They measure trust in shared hardship." (C:12066 area).
- Local custom: "Tea is offered before any negotiation. Tobacco is shared after a deal closes." (C:12066 area).
- Regional ethos: "Strength is built, not born. The forge, the hull, the garrison - everything here is made to endure. Softness is a southern luxury." Greeting: "State your business, Captain. These waters have little patience." Proverb: "The storm tests the hull, not the cargo." (C:11782 area).
- Tavern rumour (in-world, unreliable): "The whales are swimming further north each year. The old captains say the sea is warming. The young ones say the old ones talk too much." (C:12066 area).

| Name | Title | Temperament | Institution | Cite |
|---|---|---|---|---|
| Harbormaster Ake | Harbor Master | laconic | harbor_master | C:8400 |
| Sigrid Halvdan | Tea Mistress | warm | exchange | C:8422 |
| Elder Astrid | Town Elder | pragmatic | governor | C:8444 |
| Old Magnus | Storyteller | nostalgic | tavern | C:8466 |
| Bones Thorsen | Fleet Captain | weathered | shipyard | C:8488 |
| Customs Officer Lena | Customs Officer | earnest | customs | C:8510 |
| Erik Strand | Broker | cheerful | broker | C:8532 |
| Hamish | Bladesmith (merchant) | gruff | outfitter | C:5946 |
| Jarl Solveig | style master | - | - | C:6197 |

- Dress [invented]: oiled wool, sealskin boots; tea mistresses in dark aprons. The character hold stands: no likenesses or portraits.
- Speech [cite]: built from the greeting lines in each NPC record above (greeting_neutral, friendly, hostile).
- Temperament of the town: read from the power structure and tension in section E.
- Network (merchant, professional): Sigrid Halvdan - Tea Merchant Liu, linking Dragon's Gate (C:11460). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Sea sighting filed under this port: Bones Thorsen (C:12852).

## C. Visual identity (guidance only; commissions nothing; the character hold stands)

Pattern: the shared visual-identity headings 1-7 (design notes). Shared camera, envelope and do/don't blocks are in README.md and not repeated here.

1. Architecture. Identity line [invented]: The Whale Arch is the town's gate: Thornport is a whaling town that learned to trade. Landmark [cite]: "The Whale Arch - a jawbone arch spanning the harbor entrance, bleached white by decades of salt." (C:12066 area). Cited buildings: The Whale Dock (harbor_master, C:8328); The Blending Room (exchange, C:8338); The Meeting Bench (governor, C:8348); The Whale Bone (tavern, C:8358); Bones' Boat Yard (shipyard, C:8368); The Stamp Desk (customs, C:8378); Erik's Desk (broker, C:8388). Kit use [proposal]: compose from the shipped will-use plates; identity carriers are ground choice and building frequency, never new ids. Gaps follow the art gap list (design notes). Landmark canon: ../landmarks/whale_arch.md.
2. Materials: local and abundant (exports): Tea, Tobacco, Timber, Pelts; scarce or luxury (imports): Medicines, Silk, Spice, Porcelain (section A). Atmosphere [cite]: "Wood smoke, drying fish, and the bitter-sweet scent of tobacco curing in the sheds." (C:12066 area). Climate: section G.
3. Palette (references, not UI tokens; UI stays INK / GOLD / CREAM / MUTED / DUE). Judged in sRGB against the plate envelope (luma 65-141, chroma p98 35):

| Role | Family | sRGB | Luma / chroma | Status | Tie |
|---|---|---|---|---|---|
| Dominant | weathered grey shingle | (100, 98, 92) #64625C | 98 / 8 | in range | wood-shingled houses |
| Secondary 1 | peat-smoked timber | (88, 78, 66) #584E42 | 79 / 22 | in range | timber export |
| Secondary 2 | fog-bank grey | (110, 116, 118) #6E7476 | 115 / 8 | in range | fog banks |
| Accent | bleached bone | (144, 140, 126) #908C7E | 140 / 18 | in range | the Whale Arch |

4. Dress: see section B. 5. Signage: ASCII only, no UI-duplicating words; motifs [invented]: the arch, a tea kettle. 6. Silhouette: chart scale is the shared `chart_port_marker`; harbour-scale skyline [invented]: a white jawbone arch over the harbour mouth, low shingled houses behind. 7. Do and don't: README shared list.

## D. Historical analogies (feed [invented] prose; never a cite)

- **New Bedford / Nantucket.** Fits: a whaling town turned trading post (description). Suggests: tea before negotiation, tobacco after; storytellers as memory. Limit: whaling is history here, trade is the present. Avoid: harpoon-heroics cliches.

## E. Governance

- Title: Town Elder (C:8322).
- Power structure: "Thornport is governed by consensus under Elder Astrid's gentle questions. Sigrid built the economy. Magnus holds the memory. Bones keeps the boats floating. Ake runs the harbor with gestures and pipe smoke. It's the least political port in the Known World - not because they don't care about power, but because they've decided that selling tea to everyone is a better strategy than picking sides." [ascii-folded] (C:8324).
- Internal tension: "The existential question: what happens when the whales are gone entirely? Sigrid built the tea trade as a replacement, but the transition isn't complete. The old whalers (represented by Bones and Magnus) carry the town's identity; the new traders (represented by Sigrid and Erik) carry its future. Astrid bridges them, but the bridge is generational - Astrid and Magnus are aging. Sigrid will lead the economics; Erik wants to lead the contracts. Lena is the Alliance's lightest footprint, and Thornport wants to keep it that way. The quiet tension: Magnus saw something in the far north he won't discuss, and Vogt at Stormwall is watching the same direction. Whatever is up there, Thornport might not be able to stay neutral forever." (C:8325).
- Bloc stance: "Thornport is too practical for politics. They sell tea and tobacco to anyone with silver." (C:13662 area).
- Grudge: "Stormwall once tried to tax Thornport's tea trade. Thornport stopped selling tea to soldiers. The tax lasted two weeks." (C:13662 area).
- Bloc philosophy: "All flags fly here. No nation claims us." (C:13475 area). Loyalty and spillover numbers are [flavour, no mechanic] (DF-15).
- Faction canon: bloc [The Free Ports](../factions/free_ports.md).

## F. Relations with every other port

Hand-maintained from content.json. Disposition is bloc-to-bloc (symmetric in data); lanes are undirected; flows are directional by market affinity; networks are symmetric.

<!-- hand-maintained:begin -->
| Port | Region | Bloc | Disposition | Lane (distance, min class, danger) | Flows out (they import) | Flows in (we import) | Network |
|---|---|---|---|---|---|---|---|
| Porto Novo | Mediterranean | The Exchange Alliance | neutral (C:13739) | - | Pelts, Timber | - | - |
| Al-Manar | Mediterranean | The Exchange Alliance | neutral (C:13739) | - | Pelts, Tea | Medicines, Spice | - |
| Silva Bay | Mediterranean | The Exchange Alliance | neutral (C:13739) | - | - | - | - |
| Corsair's Rest | Mediterranean | The Shadow Ports | neutral (C:13844) | - | - | - | - |
| Ironhaven | North Atlantic | The Iron Pact | neutral (C:13774) | 22, sloop, 0.08 (C:1982) | Timber, Tobacco | - | - |
| Stormwall | North Atlantic | The Iron Pact | neutral (C:13774) | 18, sloop, 0.1 (C:1991) | Pelts, Tobacco | Medicines | - |
| Sun Harbor | West Africa | The Gold Coast Compact | allied (C:13802) | - | - | - | - |
| Palm Cove | West Africa | The Gold Coast Compact | allied (C:13802) | - | Pelts, Timber | - | - |
| Iron Point | West Africa | The Gold Coast Compact | allied (C:13802) | - | Pelts, Timber | - | - |
| Pearl Shallows | West Africa | The Gold Coast Compact | allied (C:13802) | - | - | - | - |
| Jade Port | East Indies | The Silk Circle | neutral (C:13823) | - | Pelts | Porcelain, Silk, Spice | - |
| Monsoon Reach | East Indies | The Free Ports | same bloc | - | Pelts, Timber | Silk, Spice | - |
| Silk Haven | East Indies | The Silk Circle | neutral (C:13823) | - | Pelts | Porcelain, Silk | - |
| Crosswind Isle | East Indies | The Free Ports | same bloc | - | Pelts | - | - |
| Dragon's Gate | East Indies | The Silk Circle | neutral (C:13823) | - | Pelts, Tobacco | Porcelain | C:11460 |
| Spice Narrows | East Indies | The Shadow Ports | neutral (C:13844) | - | Pelts | Spice | - |
| Ember Isle | South Seas | The Coral Crown | neutral (C:13851) | - | Pelts, Timber | Medicines | - |
| Typhoon Anchorage | South Seas | The Coral Crown | neutral (C:13851) | - | Pelts | Spice | - |
| Coral Throne | South Seas | The Coral Crown | neutral (C:13851) | - | Pelts, Tea | - | - |
<!-- hand-maintained:end -->

- Lane lore: "The Tea and Tobacco Road": "Thornport's warmth heading to Stormwall's cold. The northern comfort trade." (C:1991). Lane canon: [The Tea and Tobacco Road](../lanes/tea_and_tobacco_road.md).

## G. World details (undefined in data; all [invented] unless cited)

- History (shared timeline in README.md): T9: Whaling begins; the jawbone arch is raised at the harbour mouth (landmark, 'decades of salt'). T12: Thornport joins the Free Ports charter rather than the Iron Pact [invented reason: the tea tax quarrel]. T14: Stormwall's tea tax lasts two weeks (grudge text).
- Population [invented]: about 3,000.
- Language [invented]: Northern trade speech; old whaling words.
- Currency: silver is the only money in the sim (every price, fee and wage is in silver). Local coin names, if any, are [invented] flavour and never replace 'silver' in copy.
- Religion [invented]: the Hearth, and a sea-mother told of by the storytellers.
- Climate [invented port detail]: cold coast, fog banks, long summer light. Regional weather [cite]: "Grey skies press low. The Atlantic rolls in long, heavy swells." (C:11782 area, weather_flavor).
- Pirates in these waters [cite]: The Crimson Tide (C:2879), The Iron Wolves (C:2905).

## I. Sound and ambience (all [invented] sound design unless quoted from a cite)

- **Soundscape:** smoky whaling pier; wood-smoke crackle; fish racks in wind; tea brewing beside crates.
- **Signature:** harpoon set on rack (wood shaft, iron head); tea kettle on a wooden crate; tobacco leaf rustle in curing sheds.
- **Sea mark:** "The Whale Arch of Thornport stands white against the grey sky." (crates/portlight-sim/src/voyage.rs:598); see ../landmarks/whale_arch.md
- **Music/work songs (NOTE ONLY):** none in canon (shanties would be invented - avoid).
- **Vs neighbours:** domestic, slower North Atlantic port; no forge (Ironhaven), no drums (Stormwall).
- **Fragments [proposal; no fragment or asset exists yet]:** `src.harpoon_rack`, `src.kettle_crate`, `mat.tobacco_leaf`.
- Cited hooks: atmosphere "Wood smoke, drying fish, and the bitter-sweet scent of tobacco curing in the sheds." (C:12069); dock scene "Whaling boats line the pier, their harpoons cleaned and ready. Tea merchants squat beside wooden crates, brewing samples for prospective buyers." (C:12070).
- Invented bed idea: the creak of the boat yard; Old Magnus telling stories. No assets are requested.

## J. Flags for this port

- DF-4: Institutions of type `shipyard` at ports without the shipyard feature
