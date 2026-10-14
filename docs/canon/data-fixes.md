# Canon data fixes (proposals only; nothing changed)

Written 2026-10-14. Found while writing docs/canon phases 1 (ports, base 143e284), 2 (landmarks, base 71c6aaa), 3 (lanes, base ab03ace), 4 (factions and captains, base 4f08125) and 5 (player types, base c413788). Each item gives where (file:line), the proposed fix and why. None is applied: the game is unchanged. Sim data changes need their own PR, with a parity note where gossip, sighting or save text is involved.

Count: 33.

Contradictions resolved in lore (no data fix needed) are listed at the end.

## DF-1. Dimitri Andros sighting filed under Silva Bay

Where:
- content.json:12804 (sighting port_id silva_bay)
- crates/portlight-sim/data/content.json:7074 (NPC at Porto Novo)

Proposed fix: Change the sighting's `port_id` to `porto_novo`, and its text 'A Silva Bay hull' to 'A Porto Novo hull'.

Why: Andros is Porto Novo's shipwright. Canon reads the sighting as Andros testing a Porto Novo hull.

## DF-2. Porto Novo power structure says 'the Brotherhood answers to craft'

Where:
- crates/portlight-sim/data/content.json:6954

Proposed fix: Replace 'the Brotherhood answers to craft, not politics' with 'the yard answers to craft, not politics'.

Why: 'Brotherhood' is Silva Bay's governing body and cultural group.

## DF-3. Bounty hunter home region does not match home port

Where:
- content.json:2625 (bounty_hunter: home_port_id crosswind_isle, home_region North Atlantic)

Proposed fix: Set `home_region` to `East Indies`, or move `home_port_id` to a North Atlantic port. The design notes prefer `East Indies`, because Crosswind Isle as a free port suits a bounty hunter.

Why: The region and port fields disagree. This is a sim data change and needs a parity note.

## DF-4. Institutions of type `shipyard` at ports without the shipyard feature

Where:
- content.json:8368 (Thornport, `tp_boat_yard`)
- content.json:8592 (Sun Harbor, `sh_cotton_house`)
- content.json:8836 (Palm Cove, `pc_dock`)
- content.json:9256 (Pearl Shallows, `ps_dye_workshop`)
- content.json:9932 (Silk Haven, `slh_warehouse`)
- content.json:10868 (Ember Isle, `ei_dye_works`)

Proposed fix: Change `institution_type` to a non-gating type that matches each institution's own text: `boatyard` for Thornport, `dock` for Palm Cove, `warehouse` for Sun Harbor and Silk Haven, `workshop` for Pearl Shallows and Ember Isle. Do not add the `shipyard` feature (that would open ship purchases).

Why: The sim gates shipyard use on the feature (crates/portlight-sim/src/session.rs:1395), so these labels promise a service the port does not give.

## DF-5. Ironhaven embargoes the weapons it exports

Where:
- content.json:13559 area (trade_embargo weapons)
- content.json:556 (weapons affinity 1.6)

Proposed fix: Remove `weapons` from Ironhaven's `trade_embargo`. Its intent ('anyone selling weapons to pirates is a traitor', Iron Pact bloc text) stays in prose. Canon reads it as 'Ironhaven will not sell weapons into the Shadow Ports'.

Why: The field is unread by the sim (content.rs:676), so there is no gameplay change, but the data says the opposite of the description.

## DF-6. Local rivals are one-sided

Where:
- Corsair's Rest names Porto Novo as local rival (C:13671); Porto Novo names Al-Manar (C:13532). Canon reads local_rival_port as directional: a grievance, not a mutual feud. See DF-6.
- Silk Haven names Jade Port as local rival (C:13624); Jade Port names none (C:13615). Canon reads local_rival_port as directional: a grievance, not a mutual feud. See DF-6.
- Spice Narrows names Jade Port as local rival (C:13680); Jade Port names none (C:13615). Canon reads local_rival_port as directional: a grievance, not a mutual feud. See DF-6.

Proposed fix: No data change. Document `local_rival_port` as directional in content.rs (a port's grievance, not a mutual feud), or add the reciprocal rival where both sides are meant to feud. The design notes prefer documenting it as directional.

Why: Canon renders each rivalry as one side's grievance and flags it in section F.

## DF-7. Bloc rival and hostile lists are not symmetric

Where:
- The Silk Circle lists The Iron Pact in `hostile_to` (C:13453) but not the reverse (C:13410); the relationships table says `hostile` (C:13767).

Proposed fix: Make each bloc's `rivals` and `hostile_to` lists agree with the symmetric `relationships` table (add the missing reverse entries), or document the lists as one-sided attitudes. The design notes prefer adding the reverse entries.

Why: The relationships table is symmetric; the per-bloc lists are not.

## DF-8. NPC name drift across records

Where:
- content.json:11448 network name 'Henrik Brandt' vs record 'Forge Master Henrik Brandt' (C:7982)
- content.json:11472 network name 'Herbalist Sera' vs record 'Head Herbalist Sera' (C:10932)
- content.json:11472 network name 'Dr. Halvorsen' vs record 'Dr. Kristian Halvorsen' (C:8236)
- content.json:11484 network name 'Quartermaster Maren' vs record 'Quartermaster Maren Dahl' (C:8215)
- content.json:11508 network name 'Kiln Master Chen' vs record 'Kiln Master Chen Bai' (C:9516)
- content.json:11664 network name 'Inspector Zara' vs record 'Inspector Zara Osman' (C:7346)
- content.json:12936 sighting 'Healer Sera' vs record 'Head Herbalist Sera'
- crates/portlight-sim/data/content.json:7542 'Timber Exchange' vs institution 'The Timber Yard' (Silva Bay)

Proposed fix: Use each NPC's record `name` verbatim in network and sighting text, and rename 'Timber Exchange' to 'Timber Yard'.

Why: One person, one name. Canon uses the record name.

## DF-9. Harbor vs Harbour

Where:
- content.json (US 'Harbor' throughout, for example Sun Harbor at C:803)
- crates/portlight-godot/src/harbour_screen.rs (UI 'Harbour')

Proposed fix: No data change. House style: proper names keep their data spelling (Sun Harbor, harbor_master); UI and canon prose use 'Harbour' as the common noun. Record the rule in the copy locks.

Why: The canon has to pick one rule.

## DF-10. 'Gold Coast' names both a region and a bloc

Where:
- content.json:11838 area (region cultural_name 'The Gold Coast')
- content.json:13431 (bloc 'The Gold Coast Compact')

Proposed fix: No data change. Canon rule: 'the Gold Coast' means the region, and 'the Compact' means the bloc. Optionally rename the region's cultural_name later.

Why: The two are easy to confuse.

## DF-11. Non-ASCII NPC id `sb_tom` + a-acute + `s`

Where:
- content.json:7536 and every relationship_notes key that names it

Proposed fix: Rename the id to `sb_tomas` everywhere (a sim data change with a save and parity check, since ids can reach saves through memories).

Why: Ids must be ASCII (copy rule R5). Prose already says 'Tomas'.

## DF-12. Rosa and Ingrid network says 'Northern solidarity'

Where:
- crates/portlight-sim/data/content.json:11576

Proposed fix: Replace it with 'Sea-widows' solidarity ... The way working-harbour women communicate.' This is gossip text output by the sim, so it needs a parity note.

Why: Rosa is Mediterranean.

## DF-13. Silva Bay 'cheapest repairs' is a tie

Where:
- crates/portlight-sim/data/content.json:7425

Proposed fix: Change it to 'among the cheapest repairs (1 silver per hull point)'.

Why: Ironhaven also has repair_cost 1.

## DF-14. Brotherhood 'never builds warships' conflicts with the sim

Where:
- crates/portlight-sim/data/content.json:7499
- crates/portlight-sim/data/content.json:13556

Proposed fix: Reword Elena's agenda to 'will never build a war fleet to a power's order' and the grudge to 'refused to build a war fleet for the Iron Pact'.

Why: Design ruling 2: the sim sells the Royal Man-of-War at Silva Bay (session.rs:3037-3043), so the lore now says the yard builds single hulls of every class but no fleets for powers.

## DF-15. Bloc loyalty_bonus and disloyalty_penalty have no live caller; spillover is unread

Where:
- C:13388 and the other blocs; relationships from C:13718
- crates/portlight-sim/src/culture.rs:225-245 (price_multiplier reads loyalty_bonus and disloyalty_penalty; no live caller, only tests at :352-362)
- spillover (relationships, from C:13718) has no reader

Proposed fix: No data change. Add content.rs doc comments: on TradeBlocDef.loyalty_bonus and disloyalty_penalty (content.rs:666-667), that only culture::price_multiplier (culture.rs:225) reads them and it has no live caller (tests only); on BlocRelationshipDef.spillover (content.rs:687), that it is unread. Keep both comments until a mechanic is briefed.

Why: the rates exist in code but no live path applies them, so copy and canon must not promise a price effect.

## DF-16. 'Corsairs Rest' spelling in docs

Where:
- docs/PORTING-PLAN.md:232

Proposed fix: Change it to 'Corsair's Rest'.

Why: Spelling.

## DF-17. Pirate captain sightings imply home ports the data does not define

Where:
- content.json:12816 (Scarlet Ana under Corsair's Rest)
- content.json:12840 (Sergeant Kruze under Ironhaven)
- content.json:12894 (Typhoon Mei under Spice Narrows)

Proposed fix: No data change required. Optionally add a `home_port_id` to pirate factions matching canon (Crimson Tide at Corsair's Rest, Monsoon Syndicate at Spice Narrows; Iron Wolves and Deep Reef have none).

Why: Canon assigns invented home ports; data could follow later.

## DF-18. Porto Novo's beacon exists only in code

Where:
- crates/portlight-sim/src/voyage.rs:594 ("The beacon of Porto Novo breaks through the haze.")
- crates/portlight-sim/data/content.json:6962 (The Harbor Master's Tower: tower and telescope, no beacon)

Proposed fix: Add one clause to the pn_harbor description, for example "A beacon lamp burns on the roof from dusk to dawn." Canon already places the beacon there (landmarks/porto_novo_beacon.md).

Why: the voyage line names a structure the data does not have. A data addition is safer than changing shipped event text.

## DF-19. The lighthouse event ignores region

Where:
- crates/portlight-sim/src/voyage.rs:592-607 (`resolve_lighthouse` picks any of 5 lines with no region or lane check)

Proposed fix: Pick only beacons in or next to the voyage's origin or destination region; fall back to no lighthouse event. This changes the RNG draw sequence, so it needs a parity update and new goldens.

Why: a captain on a South Seas lane can see Porto Novo's beacon or the Whale Arch. Canon does not explain this away.

## DF-20. Two institutions share the name 'The Inspection Hall'

Where:
- crates/portlight-sim/data/content.json:8173 (Stormwall, customs, `sw_inspection`)
- crates/portlight-sim/data/content.json:9463 (Jade Port, customs, `jp_quality`)

Proposed fix: Rename Jade Port's to 'The Quality Hall' (matches its id `jp_quality` and its porcelain-testing role). Low priority: institution names are not shown to the player today.

Why: two catalog names are identical across regions. Canon distinguishes them as 'Stormwall's Inspection Hall' and 'Jade Port's Inspection Hall' until then.

## DF-21. Three lanes join the west straight to the East Indies

Where:
- content.json:2284 (The Monsoon Shortcut, Al-Manar - Monsoon Reach)
- content.json:2293 (The Smuggler's Run, Corsair's Rest - Spice Narrows)
- content.json:2302 (The Northern Passage, Ironhaven - Jade Port)

Proposed fix: No data change. Canon reads them as long-haul open-ocean routes that pass other regions without calling. Their data already fits that reading: distances 72-80 and galleon minimum. If chart art ever draws lanes, route them around the other regions' coasts.

Why: on the map they cross other regions. This is a flag, not an error.

## DF-22. Saves drop lane names, so loaded games lose lane labels and route encounters

Where:
- crates/portlight-sim/src/save.rs:1832-1840 (`route_value` writes port_a, port_b, distance, danger, min_ship_class; no lore_name or lore)
- crates/portlight-sim/src/save.rs:1842-1853 (`route_from` sets lore_name and lore to empty); used at save.rs:722-726
- crates/portlight-sim/tests/save_parity.rs:192-193 (the test asserts the empty values)
- Effect: crates/portlight-chart/src/view.rs:214-215 falls back to the destination name only; crates/portlight-sim/src/sea_culture.rs:129-133 falls back to the region table.

Proposed fix: on load, refill lore_name and lore from content.json routes by port pair (no save format change). Sim/parity PR with a parity note, since the test encodes the current behaviour. Verify in that PR that no other load path rebuilds routes from content.

Why: after a save and load, the 15 named lanes stop showing their names on the chart and their 15 encounter tables never fire. Canon describes the fresh-session experience.

## DF-23. 'The gentlest route in the Known World' is not the gentlest

Where:
- content.json:12243 (Grain Road encounter: "The Grain Road earns its reputation as the gentlest route in the Known World.")
- content.json:1931 (Grain Road danger 0.12); 19 of 43 routes have lower danger, the lowest being The Silk Road by Sea at 0.07 (content.json:2174)

Proposed fix: copy change to "The Grain Road earns its reputation as a gentle route." ("gentlest in the Mediterranean" also fails: The Timber Run is 0.08, content.json:1940.) Copy PR with parity note (voyage flavour text).

Why: a superlative the data contradicts.

## DF-24. The Long Crossing says 'months'; the game says about 16 days

Where:
- content.json:2105 (lore: "Months of open water.")
- content.json:2101 (distance 64), content.json:2103 (min class galleon), content.json:182 (galleon speed 4); voyage.rs:726-728 gives 16 days
- Consistent text on the same lane: "Day twelve of the crossing" (content.json:12403)

Proposed fix: "Weeks of open water." Copy PR. No sim change.

Why: the chart shows the day estimate next to the name, so the player sees both.

## DF-25. 'The longest profitable route in the world' is not the longest

Where:
- content.json:2303 and content.json:12618 (Northern Passage, distance 76)
- content.json:2290 (The Smuggler's Run, distance 80; its own encounter says "The longest, most dangerous route in the Known World", content.json:12598)

Proposed fix: No data change. Canon reads 'profitable' as 'lawful': the Run is longer but is a smuggling lane. If copy is touched, "the longest lawful route in the world". Flag, not error.

Why: both superlatives appear in shipped voyage text.

## DF-26. Two archetypes share the title 'Licensed Hunter'

Where:
- content.json:2433 (privateer title 'Licensed Hunter')
- content.json:2627 (bounty_hunter title 'Licensed Hunter')
- Shown side by side on the new game captain buttons (crates/portlight-godot/src/logic.rs:1358-1365; game.rs:2967-2974)

Proposed fix: retitle one. Suggested: The Privateer 'Letter of Marque' (garrison-licensed, Stormwall), keeping 'Licensed Hunter' for The Bounty Hunter. Copy PR; check the new game smoke and any test that asserts the title.

Why: two of nine choices read the same title on one screen, so the title does not tell them apart.

## DF-27. Silk Circle and Shadow Ports are rivals in the table but in neither bloc's list

Where:
- content.json:13830-13833 (relationships: silk_circle - shadow_ports, rival)
- content.json:13465 (Silk Circle `rivals`: exchange_alliance only); content.json:13502 (Shadow Ports `rivals`: empty)

Proposed fix: fold into the DF-7 fix: add shadow_ports to the Silk Circle's `rivals` and silk_circle to the Shadow Ports' `rivals`, or document the lists as one-sided. Same sim PR as DF-7.

Why: DF-7 names only the hostile gap; this is the matching rival gap. No live effect today (DF-15).

## DF-28. Faction spillover and vendetta triggers are unread, but journal copy implies an effect

Where:
- content.json:13866-13914 (`faction_relationships`: spillover and vendetta_trigger on every row)
- crates/portlight-sim/src/content.rs:707-708 (fields); only `faction_enemies` reads the table, by disposition (content.rs:1156; narrative.rs:655, 672, 736)
- crates/portlight-sim/src/narrative.rs:356-361 ('The Price of Friends': "The enemies of your friends are now watching you with different eyes." and hint "Standing with one faction affects your reputation with their rivals and enemies.")
- Faction regard changes only for the faction fought (crates/portlight-sim/src/session.rs:3695-3706)

Proposed fix: No data change. Either brief a spillover mechanic, or soften the hint to describe a journal beat, not a rule (the hint also uses 'Standing', against Option A, if it is ever shown in godot; the godot journal copy omits it, crates/portlight-godot/src/journal_screen.rs:984-987). Add content.rs doc comments marking spillover and vendetta_trigger unread, as in DF-15.

Why: the beat fires on regard 15 with any one faction (narrative.rs:631-638), whether or not that faction has enemies, and nothing changes with the rivals.

## DF-29. Python perk copy contradicts the captain data

Where:
- portlight@9b02494 src/portlight/engine/captain_identity.py:198 (smuggler '5% cheaper buys') against content.json:2365 (buy_price_mult 1.0)
- captain_identity.py:446 (scholar 'Best provision efficiency') against content.json:2527 (provision_burn 0.75) and content.json:2409 (navigator 0.7)
- captain_identity.py:506 (merchant prince 'lowest inspections') against content.json:2571 (inspection_chance_mult 0.4) and content.json:2453 (privateer 0.3)

Proposed fix: when perk copy is ported to the Rust/Godot build, write it from content.json, not from the Python strings. Either drop the three claims or change the data (smuggler buy x0.95; scholar burn under 0.7; merchant prince inspection under 0.3); the canon prefers fixing the copy. Copy PR; sim data change only with a parity note.

Why: the copy is not shipped today (Python only), but it is the only perk text and would mislead players if ported as is.

## DF-30. Gnaw (strength 9) pays the same bounty as strength-7 pirates

Where:
- content.json:2968 (Gnaw strength 9); crates/portlight-sim/src/bounty.rs:104 (reward 200)
- content.json:2961 and bounty.rs:95 (Sergeant Kruze, strength 7, reward 200); content.json:2940 and bounty.rs:68 (Typhoon Mei, strength 7, reward 200)
- content.json:2926 and bounty.rs:50 (The Butcher, strength 8, reward 220)

Proposed fix: raise Gnaw's reward above The Butcher's (suggested 250), or record that rewards are not tied to strength. The bounty table is hard-coded (bounty.rs:35-110) and hand-copied into the hunt screen; change both with a parity note. Coordinate with PT-S2.

Why: the strongest pirate pays less than a weaker one, so the hardest hunt is the worst deal.

## DF-31. Scarlet Ana is on the North Atlantic board but sighted at Corsair's Rest

Where:
- crates/portlight-sim/src/bounty.rs:40 (scarlet_ana region North Atlantic)
- content.json:12816-12818 (sighting under corsairs_rest, region Mediterranean)
- content.json:2877 area (The Crimson Tide holds both the Mediterranean and the North Atlantic)

Proposed fix: set the bounty region to Mediterranean, or move the sighting to a North Atlantic port. Canon prefers Mediterranean, matching the invented Crimson Tide home at Corsair's Rest (DF-17). Parity note for the bounty and sighting text.

Why: the board sends a hunter to one sea while the game shows her in another.

## DF-32. The Privateer's negative Iron Wolves start is lost on the first duel win

Where:
- content.json:2466 (privateer underworld iron_wolves -5)
- crates/portlight-sim/src/session.rs:3701 (underworld standing clamped to 0-100)

Proposed fix: clamp underworld standing to -100..100 (or at least keep values below 0), so a take-all duel win against the Iron Wolves moves -5 to -3, not 0. The -5 survives save and load; only the clamp drops it. Sim PR with a parity note against the Python oracle; separate follow-up (review triage).

Why: the Privateer's one pirate grudge disappears after a single win, which also undercuts PT-S3.

## DF-33. The Bounty Hunter and the Dockhand share a home port and mentor

Where:
- content.json:2591-2592 (dockhand: crosswind_isle, ci_dock_master_tao)
- content.json:2629-2630 (bounty_hunter: crosswind_isle, ci_dock_master_tao)

Proposed fix: flag only; no change proposed now. Revisit with DF-3 (Bounty Hunter region) if the home port moves.

Why: two of nine starts open in the same port with the same mentor, which weakens type identity.

## Resolved in lore (no data change)

- Stormwall 'answers to the Pact, not the crown' (C:13570 area): the crown is the old Northern Crown that raised the Fortress (T3) and dissolved (T8).
- Corsair's Rest: the Cliff Tavern (landmark) and Mama Lucia's Kitchen (institution) are one place. The Kitchen is the tavern's back room, carved deeper into the rock.
- Al-Manar's 'Senhora' governor: 'Senhora' is the Alliance's court honorific for any governing lady, adopted from Porto Novo's usage at the Alliance's signing (T5).
- Silva Bay tavern rumour about storm-lost ships: an in-world rumour, unreliable. Elena builds for anyone who respects ships.
- Pirate factions with no home port: canon assigns invented homes (README).
- Name reuse across ports (Mensah, Strand, Astrid, Esi): different people. Common names in a region, not kin.
- Map scale (route distance against chart distance): route distance is sailing days under prevailing winds, not straight-line distance.
- Bloc against region mismatches (Free Ports and Shadow Ports span regions): blocs are political, regions are geographic, by design.
- Thornport whaler names (C:8362 beams, C:8499 Arch, C:12348 Arch): the Arch holds the originals carved by Bones; the Whale Bone's beams hold copies (landmarks/whale_arch.md).
- Porto Novo bells (C:12008, C:6998, C:11753): two bells. The salvaged bell over the Harbor Bell tavern door is the one captains ring on arrival; the Exchange's great bell rings for the Harvest of Plenty (landmarks/grain_exchange.md).
- Near-duplicate names (landmark vs institution; institution names are not shown today): The Cliff Tavern / Mama Lucia's Kitchen (one place, the Kitchen is its back room); The Fortress / The Fortress Gate (the Gate is the sea gate in the walls); The Gate Fortress / The Chain Harbor / The Gate Command (parts of one fortress); The Hanging Market / The Dark Room (open market above, the Weigher's cave behind); The Distillery Row / The Third Distillery (one house of seven); The Free Port Bell / The Bell and Board (two bells); The River Mouth Mines / The River Mouth Dock (the dock is the mines' loading quay).
- The Chain Harbor (Dragon's Gate, C:10361) and The Chained Harbor (Typhoon Anchorage, C:11025): different places. One bars a strait, the other moors ships.
- The Storm Wall (Typhoon Anchorage) and Stormwall (North Atlantic port): unrelated.
- The Deep Reef (exchange, C:11034) and The Deep Reef Brotherhood (C:2896): the pirates are named for the reef ground; no tie.
- Dimitri Andros sighting under Silva Bay: already DF-1.
- 'Timber Exchange' at Silva Bay: already DF-8.
- Grain Road cliff-carved first Exchange (C:12228) vs the Exchange on temple ruins (C:12011): the cliff was an older open-air counting place; the hall came later (lanes/grain_road.md).
- 'The route that clothed an empire' (C:2033): a figure of speech for the Alliance cotton trade; no empire exists.
- Smuggler's Run 'pirate kings once controlled both ends' (C:2294) vs the Crimson Tide and Monsoon Syndicate today: one line of kings before T13, two crews since (T28).
- Ember Isle 'obsidian-hulled' medicine boats (C:12538): obsidian-inlaid, black-lacquered hulls.
- Free Ports 'refuse to join any bloc' (C:13482) while listed as a bloc: a bloc of non-alignment (factions/free_ports.md).
- Coral Crown 'three island ports under the Coral King's authority' (C:13518) vs Ember Isle's Keeper Elder and Typhoon Anchorage's Storm Chief: an overlordship of tribute; each island keeps its own ruler (factions/coral_crown.md).
- Raj the Quiet as a sea captain (C:2929) and as Kiran's intelligence chief (C:10703): Raj sails; his Shadow keeps the desk (C:10738).
- An Iron Wolves captain trading at Coral Throne (C:11335), outside Wolf territory: Wolves sell anywhere and raid at home.
- No One, Gunnar and the Iron Wolves all deserted 'the same garrison' (C:7781, C:7971): canon reads it as Stormwall's garrison, in order T31, T32, T33.
