# Corsair's Rest (corsairs_rest)

Base commit: 143e284 (content.json unchanged since 45b7378). Status: draft canon, phase 1. Written 2026-10-14.
Cites `C:n` are crates/portlight-sim/data/content.json line n at 143e284. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] (design-notes pattern) | DF-n see data-fixes.md | T-n see README shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Region | Mediterranean ("The Middle Sea") | C:459, C:11728 |
| Bloc | The Shadow Ports (Corsair's Rest, Spice Narrows) | C:13671, C:13493 |
| Features | black_market | C:459-553 |
| Warehouse tiers | none listed | C:6520 |
| Port fee / provisions / repair / crew | 3 / 2 / 3 per hull point / 2 | C:459-553 |
| Map | x 21, y 13 | C:553 |
| Lanes | 4 | section F |
| Player description | "A lawless harbor tucked between cliffs. Smugglers, pirates, and those who trade with them." | C:461 |
| Captain start here | The Corsair | C:2471 |

## A. Specialties and trade goods

<!-- hand-maintained:begin -->
| Good | Category | Base | Affinity | Stock target | Restock | Role |
|---|---|---|---|---|---|---|
| Weapons | military | 45 | 1.5 | 20 | 2.5 | exporter |
| Rum | provision | 20 | 1.5 | 25 | 3.0 | exporter |
| Tobacco | provision | 28 | 1.3 | 18 | 2.0 | exporter |
| Black Powder | contraband | 65 | 1.3 | 6 | 0.5 | exporter |
| Stolen Cargo | contraband | 40 | 1.2 | 10 | 1.0 | exporter |
| Pelts | commodity | 8 | 1.1 | 8 | 1.0 | exporter |
| Grain | commodity | 12 | 0.7 | 10 | 1.0 | importer |
| Opium | contraband | 85 | 0.7 | 4 | 0.3 | importer |
| Silk | luxury | 70 | 0.5 | 5 | 0.5 | importer |
| Medicines | medicine | 35 | 0.5 | 6 | 0.5 | importer |
<!-- hand-maintained:end -->

Market cite: C:459-553. Price rule: base x (target / stock) / affinity (crates/portlight-sim/src/economy.rs:37); affinity above 1 is cheap here.
Not traded here: Timber, Iron Ore, Cotton, Dyes, Spice, Porcelain, Tea, Pearls.
Region goods [flavour, no mechanic]: sacred Grain; prized Spice, Silk; forbidden none (C:11728 area).
Festival: Harvest of Plenty, every 60 days for 3, market effects Grain x1.8, Rum x1.4 (C:11752).
Festival: Night of Lanterns, every 90 days for 3, market effects Spice x2.0, Silk x1.5, Dyes x1.6 (C:11765).
Contract templates touching Mediterranean: `proc_iron_forge` (procurement), `short_tea_crisis` (shortage), `ret_cotton_return` (return_freight), `ret_spice_restock` (return_freight), `ret_dyes_textile` (return_freight), `circ_med_africa` (circuit), `circ_indies_loop` (circuit), `circ_south_seas` (circuit), `rep_charter_atlantic` (reputation_charter), `smug_opium_run` (smuggling), `smug_powder_delivery` (smuggling). Source port Corsair's Rest: none.

## B. People

- Cultural group: The Brotherhood of the Cove: "Not a guild - a silence. They protect each other by knowing nothing." (C:12036 area).
- Local custom: "No names. Every captain here is 'friend' until proven otherwise." (C:12036 area).
- Regional ethos: "Civilization is commerce. The law of contract binds harder than iron, and a merchant's word, once given, is recorded in stone." Greeting: "Welcome to port, Captain. Your cargo manifest, if you please." Proverb: "A contract honored is worth more than gold hoarded." (C:11728 area).
- Tavern rumour (in-world, unreliable): "There's a passage through the cliffs that leads to a second harbor. Only the oldest pirates know the way, and they aren't talking." (C:12036 area).

| Name | Title | Temperament | Institution | Cite |
|---|---|---|---|---|
| One-Eye Basso | Dockmaster | watchful | harbor_master | C:7726 |
| Whisper | Price Keeper | secretive | exchange | C:7748 |
| No One | The Tide's Voice | cold | governor | C:7770 |
| Mama Lucia | Cook and Keeper | fierce | tavern | C:7792 |
| The Physician | Doctor | weary | apothecary | C:7814 |
| Ghost | Cargo Master | efficient | broker | C:7836 |
| Little Fish | Runner | sharp | customs | C:7858 |

- Dress [invented]: whatever was taken last; hoods and plain coats, no colours that identify a ship. The character hold stands: no likenesses or portraits.
- Speech [cite]: built from the greeting lines in each NPC record above (greeting_neutral, friendly, hostile).
- Temperament of the town: read from the power structure and tension in section E.
- Network (tavern, personal): Mama Lucia - Mama Smoke, linking Spice Narrows (C:11532). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Network (broker, allied): Ghost - The Ghost Broker, linking Spice Narrows (C:11652). Surfaces as sim gossip (crates/portlight-sim/src/cross_port_networks.rs:49).
- Sea sighting filed under this port: Scarlet Ana (C:12816).
- Sea sighting filed under this port: Ghost (C:12828).

## C. Visual identity (guidance only; commissions nothing; the character hold stands)

Pattern: the shared visual-identity headings 1-7 (design notes). Shared camera, envelope and do/don't blocks are in README.md and not repeated here.

1. Architecture. Identity line [invented]: The cove is the town: Corsair's Rest is cliff, tavern and hidden berths, and nothing is built where the sea can see it. Landmark [cite]: "The Cliff Tavern - carved into the rock face, visible only from the sea." (C:12036 area). Cited buildings: The Cove Mouth (harbor_master, C:7654); The Whisper Market (exchange, C:7664); The Crimson Chair (governor, C:7674); Mama Lucia's Kitchen (tavern, C:7684); The Surgery (apothecary, C:7694); The Cliff Caves (broker, C:7704); The Cliff Watch (customs, C:7714). Kit use [proposal]: compose from the shipped will-use plates; identity carriers are ground choice and building frequency, never new ids. Gaps follow the PB-001 gap list.
2. Materials: local and abundant (exports): Weapons, Rum, Tobacco, Pelts, Black Powder, Stolen Cargo; scarce or luxury (imports): Silk, Medicines, Grain, Opium (section A). Atmosphere [cite]: "Torch smoke, cheap rum, and the clink of coins changing hands under the table." (C:12036 area). Climate: section G.
3. Palette (references, not UI tokens; UI stays INK / GOLD / CREAM / MUTED / DUE). Judged in sRGB against the plate envelope (luma 65-141, chroma p98 35):

| Role | Family | sRGB | Luma / chroma | Status | Tie |
|---|---|---|---|---|---|
| Dominant | shadowed cliff rock | (84, 78, 72) #544E48 | 79 / 12 | in range | cliff-sheltered harbour |
| Secondary 1 | tarred rope and dark hulls | (66, 60, 54) #423C36 | 61 / 12 | mood, not values | hidden berths |
| Secondary 2 | sea-cave grey-green | (76, 90, 84) #4C5A54 | 87 / 14 | in range | the cove water's edge rock |
| Accent | lantern amber | (150, 112, 60) #96703C | 116 / 90 | mood, not values | the Cliff Tavern seen only from the sea |

4. Dress: see section B. 5. Signage: ASCII only, no UI-duplicating words; motifs [invented]: no signs at all; a chalk 'friend' mark. 6. Silhouette: chart scale is the shared `chart_port_marker`; harbour-scale skyline [invented]: nothing above the cliff line; terraces and doors cut into the rock face, lanterns visible only seaward. 7. Do and don't: README shared list.

## D. Historical analogies (feed [invented] prose; never a cite)

- **Port Royal before 1692 (as a trade function only).** Fits: a harbour where outlaw cargo was sold openly to anyone who came (black market, Stolen Cargo 1.2). Suggests: no-names custom, cash on the plank, protection paid quietly. Limit: keep it about the trade, not any people (Analogist risk 3). Avoid: eyepatch-and-parrot pirate costume.
- **Sale (Bou Regreg) corsair port, as harbour logistics.** Fits: a cliff-sheltered harbour run by a captains' council that sold prizes. Suggests: one voice speaks for the cove; disputes settled in the tavern. Limit: the analogy covers harbour logistics only. Avoid: any ethnic or religious coding.

## E. Governance

- Title: The Tide's Voice (C:7648).
- Power structure: "Corsair's Rest has no government - it has an arrangement. The Crimson Tide protects the cove through No One's authority, collecting tribute and settling disputes. One-Eye Basso controls physical access - if he doesn't wave you in, you don't enter. Whisper sets prices in the shadow market. Ghost moves the cargo. Mama Lucia feeds everyone and enforces the only law that holds: no violence in her kitchen. The Physician keeps people alive. Little Fish watches everything from the cliffs, learning, and growing into something the cove hasn't figured out yet." [ascii-folded] (C:7650).
- Internal tension: "The surface tension is between the Crimson Tide's authority (No One) and the cove's organic independence (everyone else). The Tide protects the Rest but takes a cut. If the cut gets too big, the cove dies. If the protection falters, the navy finds them. It's a balance maintained by mutual need. The deeper tension is generational: One-Eye, Mama Lucia, and the Physician are aging. Ghost and Little Fish are the future. Ghost is Tide-loyal - whatever the Tide becomes, Ghost follows. Little Fish is loyal to Mama and the cove itself. When the old guard is gone, the question is whether the cove serves the Tide or the Tide serves the cove. No One watches this tension carefully and says nothing, which is the most unsettling thing of all." (C:7651).
- Bloc stance: "Corsair's Rest has no politics. It has silence, silver, and the understanding that names are optional." (C:13671 area).
- Local rival: Porto Novo: "Corsair's Rest exists because Porto Novo's laws are too strict. Every smuggler at the Rest is a trader Porto Novo drove away." (C:13671 area).
- Grudge: "Porto Novo's harbor master publicly burned a cargo manifest from Corsair's Rest. The Brotherhood of the Cove took his favorite ship. He got it back - without the cargo." (C:13671 area).
- Bloc philosophy: "No names, no questions, no manifests." (C:13493 area). Loyalty and spillover numbers are [flavour, no mechanic] (DF-15).

## F. Relations with every other port

Hand-maintained from content.json. Disposition is bloc-to-bloc (symmetric in data); lanes are undirected; flows are directional by market affinity; networks are symmetric.

<!-- hand-maintained:begin -->
| Port | Region | Bloc | Disposition | Lane (distance, min class, danger) | Flows out (they import) | Flows in (we import) | Network |
|---|---|---|---|---|---|---|---|
| Porto Novo | Mediterranean | The Exchange Alliance | hostile (C:13746) | 18, sloop, 0.18 (C:1955) | Pelts | Grain | - |
| Al-Manar | Mediterranean | The Exchange Alliance | hostile (C:13746) | - | Pelts, Rum | Medicines | - |
| Silva Bay | Mediterranean | The Exchange Alliance | hostile (C:13746) | 14, sloop, 0.15 (C:1964) | Rum, Weapons | - | - |
| Ironhaven | North Atlantic | The Iron Pact | hostile (C:13781) | - | Rum, Tobacco | - | - |
| Stormwall | North Atlantic | The Iron Pact | hostile (C:13781) | 40, brigantine, 0.15 (C:2018) | Pelts, Rum, Tobacco | Grain, Medicines | - |
| Thornport | North Atlantic | The Free Ports | neutral (C:13844) | - | - | - | - |
| Sun Harbor | West Africa | The Gold Coast Compact | neutral (C:13809) | - | Rum | - | - |
| Palm Cove | West Africa | The Gold Coast Compact | neutral (C:13809) | - | Pelts | - | - |
| Iron Point | West Africa | The Gold Coast Compact | neutral (C:13809) | - | Pelts, Rum, Weapons | - | - |
| Pearl Shallows | West Africa | The Gold Coast Compact | neutral (C:13809) | - | Rum | - | - |
| Jade Port | East Indies | The Silk Circle | rival (C:13830) | - | Pelts, Rum | Silk | - |
| Monsoon Reach | East Indies | The Free Ports | neutral (C:13844) | - | Pelts, Rum | Silk | - |
| Silk Haven | East Indies | The Silk Circle | rival (C:13830) | - | Pelts, Rum | Silk | - |
| Crosswind Isle | East Indies | The Free Ports | neutral (C:13844) | - | Pelts | - | - |
| Dragon's Gate | East Indies | The Silk Circle | rival (C:13830) | - | Pelts, Rum, Tobacco | - | - |
| Spice Narrows | East Indies | The Shadow Ports | same bloc | 80, galleon, 0.25 (C:2288) | Black Powder, Pelts, Rum, Weapons | Opium | C:11532, C:11652 |
| Ember Isle | South Seas | The Coral Crown | neutral (C:13858) | - | Pelts, Rum, Weapons | Medicines | - |
| Typhoon Anchorage | South Seas | The Coral Crown | neutral (C:13858) | - | Pelts, Rum | - | - |
| Coral Throne | South Seas | The Coral Crown | neutral (C:13858) | - | Pelts, Weapons | - | - |
<!-- hand-maintained:end -->

- Lane lore: "The Shadow Lane": "Every merchant denies using it. Every merchant uses it. The line between trade and smuggling runs through these waters." (C:1955).
- Lane lore: "The Smuggler's Run": "Only the desperate or the bold sail this direct. Pirate kings once controlled both ends." (C:2288).
- Asymmetry flag: Corsair's Rest names Porto Novo as local rival (C:13671); Porto Novo names Al-Manar (C:13532). Canon reads local_rival_port as directional: a grievance, not a mutual feud. See DF-6.

## G. World details (undefined in data; all [invented] unless cited)

- History (shared timeline in README.md): T13: Smugglers driven out by Porto Novo's new code settle the cove (rivalry text). T13: The Crimson Tide makes the cove its home port [invented; see pirate home ports]; its governor title 'The Tide's Voice' is the cited link.
- Population [invented]: about 2,500, and never the same 2,500 twice.
- Language [invented]: Middle Sea trade speech; a cove cant where no one uses a name.
- Currency: silver is the only money in the sim (every price, fee and wage is in silver). Local coin names, if any, are [invented] flavour and never replace 'silver' in copy.
- Religion [invented]: none spoken of; sailors leave a coin on the Cliff Tavern step.
- Climate [invented port detail]: Mediterranean, but the cliffs keep the cove in shadow by mid-afternoon. Regional weather [cite]: "The sea is calm and blue. Sunlight dances on ancient waters." (C:11728 area, weather_flavor).
- Pirates in these waters [cite]: The Crimson Tide (C:2879). Home port [invented]: The Crimson Tide - The cove's governor is 'The Tide's Voice' and Scarlet Ana is sighted off the Rest.

## I. Sound and ambience (all [invented] sound design unless quoted from a cite)

- **Soundscape:** cliff-enclosed cove, close and muffled; torch crackle; "the clink of coins changing hands under the table"; a single rowboat's oars.
- **Signature:** small silver coins palmed (dull, low clink - darker than `ui_trade_coin`); torch sputter; oarlock creak of the fee boy's rowboat.
- **Music/work songs (NOTE ONLY):** none - the Brotherhood is "a silence"; absence is the design.
- **Vs neighbours:** the one Med port with no flags and no voices - intimate and furtive vs Porto Novo's open quays.
- **Fragments [proposal; no fragment or asset exists yet]:** `place.hidden_cove`, `src.silver_palmed`, `src.oarlock_rowboat`.
- Cited hooks: atmosphere "Torch smoke, cheap rum, and the clink of coins changing hands under the table."; dock scene "Ships anchor in the hidden cove with no flags flying. A boy in a rowboat approaches to collect the docking fee - payable in silver, no questions." (C:12036 area).
- Invented bed idea: water slapping in caves; low talk; no bells. No assets are requested.

## J. Flags for this port

- DF-6: Local rivals are one-sided
- DF-16: 'Corsairs Rest' spelling in docs
- DF-17: Pirate captains sighted under ports that are not their homes
- Resolved in lore: The Cliff Tavern and Mama Lucia's Kitchen are one place; the Kitchen is the tavern's back room.
