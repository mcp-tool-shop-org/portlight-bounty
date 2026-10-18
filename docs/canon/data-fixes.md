# Canon data fixes (proposals only; nothing changed; resolved items are marked)

Written 2026-10-14. Found while writing docs/canon phases 1 (ports, base 143e284), 2 (landmarks, base 71c6aaa), 3 (lanes, base ab03ace), 4 (factions and captains, base 4f08125), 5 (player types, base c413788), 6A (companions and officers, base 515a488) and 6B (style masters and skill trainers, base af3d105) and 6C (merchants, base d2b544f). Each item gives where (file:line), the proposed fix and why. None is applied: the game is unchanged. Sim data changes need their own PR, with a parity note where gossip, sighting or save text is involved.

Count: 48.

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
- crates/portlight-sim/data/content.json:2628-2629 (bounty_hunter: home_region North Atlantic, home_port_id crosswind_isle)
- crates/portlight-sim/data/content.json:1385 (Crosswind Isle region East Indies)
- portlight@9b02494 src/portlight/engine/captain_identity.py:597-598 (the same two values: the Python reference has the same mismatch)

Proposed fix: Set `home_region` to `East Indies`, or move `home_port_id` to a North Atlantic port. The design notes prefer `East Indies`, because Crosswind Isle as a free port suits a bounty hunter.

Why: The region and port fields disagree, and the oracle has the same disagreement, so this is inherited, not a port error. This is a sim data change and needs a parity note naming the Python value it departs from.

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

## DF-22. Saves drop lane names, so loaded games lose lane labels and route encounters (resolved in #97)

Status: resolved in #97 (merged 2026-10-14). The Rust loader now refills lore_name and lore from content.json routes by port pair (crates/portlight-sim/src/save.rs:727, 1860). The Python loader still leaves them empty; that is the Python bug, and the divergence is intended. The text below records the problem as found.

Where:
- crates/portlight-sim/src/save.rs:1832-1840 (`route_value` writes port_a, port_b, distance, danger, min_ship_class; no lore_name or lore)
- crates/portlight-sim/src/save.rs:1842-1853 (`route_from` sets lore_name and lore to empty); used at save.rs:722-726
- crates/portlight-sim/tests/save_parity.rs:192-193 (the test asserts the empty values)
- Effect: crates/portlight-chart/src/view.rs:214-215 falls back to the destination name only; crates/portlight-sim/src/sea_culture.rs:129-133 falls back to the region table.

Fix applied: on load, refill lore_name and lore from content.json routes by port pair (no save format change). Sim/parity PR with a parity note, since the test encodes the current behaviour. Verify in that PR that no other load path rebuilds routes from content.

Why: after a save and load, the 15 named lanes stop showing their names on the chart and their 15 encounter tables never fire. Canon describes the fresh-session experience.

## DF-23. 'The gentlest route in the Known World' is not the gentlest (resolved)

Status: resolved in #110 (copy override). The Grain Road calm-weather encounter now reads "The Grain Road earns its reputation as a gentle route."; `COPY_OVERRIDES` in tools/extract_content.py applies it and content.json is regenerated by the tool. The Python text is unchanged, so the difference from Python is intended. The text below records the problem as found.

Where:
- content.json:12243 (Grain Road encounter: "The Grain Road earns its reputation as the gentlest route in the Known World.")
- content.json:1931 (Grain Road danger 0.12); 19 of 43 routes have lower danger, the lowest being The Silk Road by Sea at 0.07 (content.json:2174)

Proposed fix: copy change to "The Grain Road earns its reputation as a gentle route." ("gentlest in the Mediterranean" also fails: The Timber Run is 0.08, content.json:1940.) Copy PR with parity note (voyage flavour text).

Why: a superlative the data contradicts.

## DF-24. The Long Crossing says 'months'; the game says about 16 days (resolved)

Status: resolved in #110 (copy override). The Long Crossing lore now reads "Weeks of open water."; `COPY_OVERRIDES` in tools/extract_content.py applies it and content.json is regenerated by the tool. The Python text is unchanged, so the difference from Python is intended. The text below records the problem as found.

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

## DF-26. Two archetypes share the title 'Licensed Hunter' (resolved)

Status: resolved in #110 (copy override). The Privateer's title is now "Letter of Marque"; `COPY_OVERRIDES` in tools/extract_content.py applies it and content.json is regenerated by the tool. The roster test allows exactly this one title difference from the Python golden. The text below records the problem as found.

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

## DF-34. Companion role effects are authored and unread

Where:
- crates/portlight-sim/data/content.json:5480-5541 (companions.roles: damage, interception, speed, danger, heal, inspection evasion and trade numbers on 5 roles)
- crates/portlight-sim/src/companion.rs:193, 211, 235, 269 (party_combat_bonus, roll_interception, party_summary, cohesion: no caller outside the test at companion.rs:327)
- portlight@9b02494 src/portlight/engine/companion_engine.py:216, 245 (the same two combat functions; no caller in src). Python's `party` command does call get_party_summary and get_cohesion (src/portlight/app/cli.py:3183-3196); Rust has no such caller.

Proposed fix: none proposed; logged only. Wiring the effects, or rewording the role text, is a separate design decision (owner: sim, size M). Until then canon says no role effect applies.

Why: role text such as 'A trained fighter who stands at your side in combat.' (content.json:5484) promises effects the sim does not deliver. No shipped sentence repeats it today: the crew desk shows the role name only (crates/portlight-godot/src/logic.rs:2129-2181).

Status: open, log only.

## DF-35. 11 of 13 morale triggers never fire, so loyalty and warning lines are unreachable

Where:
- crates/portlight-sim/data/content.json:5704-5835 (13 triggers) and :5836-5867 (4 personalities)
- crates/portlight-sim/src/session.rs:2074-2076 (only spared_enemy and took_all are fired, once per duel victory); companion.rs:124 has no other caller
- crates/portlight-sim/src/companion.rs:130-141 (a personality delta applies only where the role delta for that trigger is not zero; the rest is dead data, for example the lawful spared_enemy +2, content.json:5838, on a role that ignores it)
- portlight@9b02494 src/portlight/app/cli.py:2888-2906 and src/portlight/app/tui/screens/encounter.py:863-882 (the same two triggers)

Proposed fix: none proposed; logged only. Firing the other triggers, or trimming the table, is a separate design decision (owner: sim, size M).

Why: loyalty_line and warning_line (every companion record) imply a morale arc that only duel choices can drive. Today a departure is reachable only for navigators and surgeons, by repeated take-all victories (companions/README.md section 5). The pragmatic personality has no deltas at all (content.json:5846).

Status: open, log only.

## DF-36. The recruit notice prints a raw id (resolved)

Status: resolved (Godot copy). `apply_recruit` now builds the notice with `recruit_notice_line`, reading the companion's catalog name (for example "Recruited Iron Marta."). The text below records the problem as found.

Where:
- crates/portlight-godot/src/game.rs:5748 ('Recruited {id}.' reads 'Recruited iron_marta.')
- crates/portlight-godot/src/logic.rs:1923 (the confirm line before it uses the catalog name: 'Recruit Iron Marta for 80 silver.')

Proposed fix: build the notice from the catalog name, as arm_recruit does with the offer name (game.rs:5666), for example 'Recruited Iron Marta.' Owner: copy (godot); size XS; add a logic test. Route it with DF-37 as one later copy PR; the other copy items are logged in a later canon pass.

Why: it breaks the no-raw-ids rule; ids are cite keys, not player words. The same notice is also pushed to the log (game.rs:5749).

## DF-37. Same-name people at the same port, and across ports

Where:
- Same port: companion Dr. Amara (content.json:5609) and Inspector Amara, Sun Harbor (content.json:8745); merchant Devi (content.json:6018) and Shipwright Devi, Monsoon Reach (content.json:9773); merchant Tane (content.json:6047) and War Chief Tane, Coral Throne (content.json:11346); merchant Master Chen (content.json:6004) and Kiln Master Chen Bai, Jade Port (content.json:9517)
- Same full name, different ports: merchant Li Wei, Silk Haven (content.json:6032) and Harbor Director Li Wei, Jade Port (content.json:9495)
- Given-name overlaps across ports are not logged as faults: Kofi, Yaa, Marta, Rosa, Erik, Kai (for example Iron Marta and Marta Soares, content.json:5545 and :7053; Rosa the Fence and Rosa Carvalho, content.json:5657 and :7581)

Proposed fix: give about five display names a surname or a title in the content strings, one COPY_OVERRIDES entry each (owner: copy; size S). Check goldens and saves for those display strings before applying. Canon needs none of it.

Why: two people with one name in one harbour cannot be told apart in any line that names them. Canon reads each pair as two different people until fixed (resolved in lore below for the given-name overlaps).

Status: open; fix proposed for the later copy PR. Canon changes nothing.

## DF-38. Officer name pools reuse 19 authored given names

Where:
- crates/portlight-sim/data/content.json:6078-6152 (5 pools of 10), against port NPC, merchant, companion, style master and skill trainer names. The 19 pool names are: Marco C:6080, Fatima C:6083, Dimitri C:6086, Henrik C:6097, Gunnar C:6101, Astrid C:6102, Kwame C:6110, Kofi C:6112, Adaeze C:6113, Mariam C:6115, Yusuf C:6118, Zara C:6119, Rajan C:6125, Kamala C:6134, Tane C:6140, Moana C:6141, Rangi C:6142, Leilani C:6143, Koa C:6146
- portlight@9b02494 src/portlight/content/officer_names.py:12-33 (the same pools)

Proposed fix: rename the pool entries in place (owner: content; size S, 19 strings). A rename does not change the number of random draws, but it changes receipts and any golden or save that prints an officer name, so it needs a parity note against the Python pools.

Why: a hired navigator named Tane can stand beside War Chief Tane of Coral Throne, and the crew desk prints only the given name.

Status: declined - resolved in lore. A hired officer is a namesake of the authored person, not that person; two officers who share a name are told apart by role (officers/README.md section 5). Logged so the choice is on record.

## DF-39. Style masters and skill trainers are authored and unread; the real gates are other lists

Where:
- crates/portlight-sim/src/content.rs:1027-1032 and crates/portlight-sim/src/training.rs:73-79 (masters_at and masters_at_port: no caller outside the test at training.rs:101)
- crates/portlight-sim/src/training.rs:9-52 (the training gate is the style's own training_port_ids, content.json:4159, 4200, 4240, 4280, 4321 for the five styles; today all 10 masters sit in their style's list, content.json:6171-6250)
- crates/portlight-sim/src/skills.rs:32-82 and crates/portlight-sim/src/content.rs:290 (the skill gate is the trainer list; training_port_feature, content.json:5370, has no reader; all 5 trainer ports have the shipyard feature, content.json:214, 382, 560, 1210, 1754)
- crates/portlight-godot/src/logic.rs:2028-2128 (the desk builds style rows and skill lines from names and numbers; no master or trainer description or dialog; rg of crates/portlight-godot/src finds no reader)
- crates/portlight-sim/src/session.rs:3249-3258, 1928-1960 and crates/portlight-sim/src/script.rs:427, 529 (the blacksmith effects: wear slowing is live only with a melee weapon or armour equipped; the maintenance discount is reachable only through the script runner; the upgrade discount and field repair have no Session caller)
- portlight@9b02494 src/portlight/app/cli.py:2426-2455 and 2742-2760, src/portlight/app/combat_views.py:359-412 (Python prints master names and dialog, and every trainer's dialog)

Proposed fix: none proposed; logged only. Showing master and trainer text on the crew desk, reading the master list as the gate, or building a gear shop so the blacksmith effects have a route, are separate design decisions (owner: UI and sim, size M). Until then canon says no master gates training and no trainer text is seen.

Why: two lists (master port, style port list) hold one fact and can diverge silently; authored text reaches no player; and a 600-silver skill has no Godot route to its effects. Canon documents each as it is.

Status: open, log only.

## DF-40. Senora Vega's 'only fencing school east of Gibraltar' contradicts Maestro Luciano; the train and skill notices print raw ids (resolved)

Status: resolved (copy override + Godot copy). Senora Vega's description now reads "A Castilian expatriate who runs a fencing school east of Gibraltar." via `COPY_OVERRIDES` in tools/extract_content.py; `apply_train` and `apply_skill` build their notices from the catalog names (`train_notice_line`, `skill_notice_line`); and the empty-trainer line uses the skill's catalog name ("No Blacksmithing trainer at this port."). The text below records the problem as found.

Where:
- crates/portlight-sim/data/content.json:6185 (Senora Vega: 'runs the only fencing school east of Gibraltar'), content.json:6177 (Maestro Luciano teaches 'the Spanish school from a warehouse overlooking the harbor' [data spelling]), content.json:6183 and 6175 (both teach La Destreza), content.json:4159 (the style's two training ports, Porto Novo and Al-Manar, both in the Mediterranean, content.json:4133). The Python text is the same (portlight@9b02494 src/portlight/content/fighting_styles.py:266-278).
- crates/portlight-godot/src/game.rs:5708 (the style notice 'Learned dambe.' prints the style id) and game.rs:5731 (the skill notice 'Trained blacksmith.' prints the skill id); the confirm lines before them use catalog names (crates/portlight-godot/src/logic.rs:1884, 1889-1891). The empty-trainer line prints the skill's id word (logic.rs:2084).

Proposed fix: (1) change 'the only' to 'a' in the one Vega string; (2) build the two notices from the style and skill catalog names, as DF-36 proposes for the recruit notice (for example 'Learned Dambe.' and 'Trained Blacksmithing.'); (3) read the skill's name in the empty-trainer line. Owner: copy; size XS to S. Route (1) as a COPY_OVERRIDES entry (tools/extract_content.py:437) and (2) to (3) as Godot copy changes with logic tests, in the same later copy PR as DF-36 and DF-37. Check goldens and saves for the Vega string first. Canon option while unfixed: Luciano keeps a warehouse floor, not a school (Vega's is the one with a hall and a register).

Why: two masters of one style cannot both run 'the only' school; and raw ids break the no-raw-ids rule (ids are cite keys, not player words). The notices are also pushed to the log (game.rs:5709, 5732).

## DF-41. The style injury gate is dead, and no path equips a learned style (resolved)

Status: resolved (sim + Godot). `Session::train_crew` now passes `injuries::injured_body_parts(&self.world.captain.injuries)` into `training::can_learn_style`; `Session::equip_style` equips or clears a learned style with the same injury gate; and the Godot Crew desk renders learned style rows with an `Equip` button (`Action::CrewEquipStyle` / `crew.equip.<id>`). The text below records the problem as found.

Where:
- crates/portlight-sim/src/session.rs:3337 (train_crew passes an empty injury list to can_learn_style, so required_body_parts, content.json:4166, 4207, 4247, 4287, 4328, never block training); crates/portlight-godot/src/crew_screen.rs:11-12 and crates/portlight-godot/src/logic.rs:1881-1883 say so in comments
- crates/portlight-sim/src/session.rs:1905-1912 and crates/portlight-sim/src/model.rs:535 (injuries are real: a lost personal fight can add one, so the list could be non-empty)
- crates/portlight-sim/src/model.rs:523, crates/portlight-sim/src/world.rs:66 and crates/portlight-sim/src/save.rs:934 (the active style is None at a new game and is only loaded from a save; no Rust code writes it), crates/portlight-sim/src/combat.rs:104-112 and 273-283 (a style's passives and special action apply only through the active style)
- portlight@9b02494 src/portlight/app/cli.py:2452-2459 (Python passes the captain's real injured parts) and 2475-2504 (equip-style sets the active style)

Proposed fix: none proposed; logged only. Passing the real injury list, adding an equip control, or dropping the body-part data are design decisions (owner: sim and UI, size S to M), not copy. Until then canon says a learned style changes nothing in a game started in Rust.

Why: the five styles cost 60 to 150 silver and 5 to 10 days each, and their passives and special actions are authored, but in Rust the training buys no effect and the injury gate never refuses.

## DF-42. Li Wei 'builds hand crossbows', but the Silk Haven shop cannot stock them (resolved)

Status: resolved (copy override). Li Wei's description now reads "A craftsman who builds blowguns and arquebus pistols from bamboo and brass. Quiet weapons for quiet work." via `COPY_OVERRIDES` in tools/extract_content.py and content.json is regenerated by the tool. The text below records the problem as found.

Where:
- crates/portlight-sim/data/content.json:6036 (the description: "A craftsman who builds hand crossbows from bamboo and horn. Quiet weapons for quiet work.")
- content.json:4038 and :4047 (Hand Crossbow, available_regions Mediterranean only); content.json:5343 and :5348 (Crossbow Bolts, Mediterranean only); content.json:1298 (Silk Haven is East Indies)
- crates/portlight-sim/src/session.rs:3553-3609 (stock is every item of the merchant's types whose region list holds the port's region). Li Wei's stock at Silk Haven, computed: Cutlass, Dagger, Kris, Arquebus Pistol, Throwing Knife, Blowgun, Pistol Shot & Powder, Blowgun Darts (poisoned); no hand crossbow and no crossbow bolts (merchants/li_crossbow.md section 1)
- portlight@9b02494 src/portlight/engine/merchant.py:25-73 (the same rule in Python)

Proposed fix: reword the description to what the shop can hold, for example the blowgun and the arquebus pistol of the region (owner: copy; size XS; one COPY_OVERRIDES entry, tools/extract_content.py:437). Not a stock change: adding East Indies to the hand crossbow's regions would alter every East Indies shop and the loot tables' reach. Check goldens and saves for the string first.

Why: the one line a captain could read about this shop describes a weapon it never sells. The string is not shown in Godot today (no reader), so the defect is latent.

## DF-43. Hamish 'forges axes and broadswords', but no broadsword is in the catalog (resolved)

Status: resolved (copy override). Hamish's description now reads "A Highland smith who forges axes and blades in a workshop built into the cliff face." via `COPY_OVERRIDES` in tools/extract_content.py and content.json is regenerated by the tool. The text below records the problem as found.

Where:
- crates/portlight-sim/data/content.json:5951 (the description: "A Highland smith who forges axes and broadswords in a workshop built into the cliff face.")
- content.json:3879 (Boarding Axe, North Atlantic only, the one axe) and :3890 (its compatible_styles holds highland_broadsword); content.json:4172 (Highland Broadsword is a fighting style, not a weapon); the eight melee records are C:3842 to C:3967 and none is a broadsword
- Hamish's stock at Thornport, computed: Cutlass, Rapier, Boarding Axe, Dagger, Matchlock Pistol, Wheellock Pistol, Throwing Knife (merchants/hamish_iron.md section 1)

Proposed fix: reword to axes and blades (owner: copy; size XS; one COPY_OVERRIDES entry). Not a catalog change.

Why: the shop sells one axe and no broadsword; 'broadswords' is the style's name used as a weapon. The string is not shown in Godot today.

## DF-44. War Chief Tane is blamed for a '700% markup' that no code applies (resolved)

Status: resolved (copy override). War Chief Tane's agenda now reads "He's the reason weapons are dear at Coral Throne." via `COPY_OVERRIDES` in tools/extract_content.py and content.json is regenerated by the tool. The text below records the problem as found.

Where:
- crates/portlight-sim/data/content.json:11352 (the agenda: "He's the reason weapons sell for 700% markup at Coral Throne."), institution War Chief Tane, content.json:11346
- content.json:6057 (the Coral Throne merchant's price_markup is 1.08); crates/portlight-sim/src/session.rs:3540-3542 (the only markup in the sim is the merchant's; rg of crates finds no 700% and no other markup)
- crates/portlight-sim/src/content.rs:532 (agenda is a field; rg of crates/portlight-godot/src finds no reader, so the line is not shown today)
- portlight@9b02494 src/portlight/content/port_institutions_east.py:725 (the same string)

Proposed fix: soften the agenda line, for example 'weapons are dear at Coral Throne' (owner: copy; size XS; one COPY_OVERRIDES entry; the line is in gossip-adjacent institution text, so check goldens). Not a price change.

Why: the line promises a price the game does not charge: the dearest item at the Coral Throne shop is Kris at 43 silver against a catalog cost of 40.

## DF-45. Merchant key malia_provisions: she sells arms and armour, not provisions

Where:
- crates/portlight-sim/data/content.json:6060 (id malia_provisions; title Outfitter, :6063; types melee, armor, ammo, :6067 to :6070)
- crates/portlight-sim/src/session.rs:759 (provisions are bought at the crew desk by the provision call, not from a merchant)
- the key is read only by the lookup at crates/portlight-sim/src/content.rs:997-998, whose one caller is a test (crates/portlight-sim/src/merchant.rs:19), and is never shown

Proposed fix: rename the key, for example to malia_outfitter (owner: content; size XS), which touches that test and any save or golden that names the id; a parity note is needed. A copy-only variant is not needed: no player text uses the id.

Why: a data key that names the wrong goods can mislead a writer who reads the data. It cannot mislead a player.

Status: declined - resolved in lore. The id is a data key and never reaches a player; Malia is the Outfitter of her port, and 'provisions' in the key is read as the old word for general outfitting. Logged so the choice is on record.

## DF-46. Merchants never reach Godot, and their text has no reader (ammunition loss resolved)

Where:
- crates/portlight-sim/src/script.rs:422-428 and crates/portlight-sim/src/session.rs:3209-3245 (buy_gear: reachable only from the script runner's gear command and tests, session.rs:4816); rg of crates/portlight-godot/src finds no call and no merchant reader
- crates/portlight-sim/src/content.rs:376-386 (personality, title, description, greeting are fields only; the lookups at content.rs:997-1006 read id and port)
- two copies of the markup formula: crates/portlight-sim/src/merchant.rs:8-10 (used only by its own test) and crates/portlight-sim/src/session.rs:3540-3542 (the live one)
- docs/PORTING-PLAN.md:326 (engine/merchant.py deprioritised; Session::buy_gear kept for the gear scripts)
- noted while writing, Rust against Python: buy_gear takes an item id and buys one (session.rs:3243); Python takes merchant, item and quantity (portlight@9b02494 src/portlight/engine/merchant.py:76-113). Ammunition is stored on the weapon in Rust (`Session::buy_gear` now refuses `"ammo"` before taking silver when the captain carries no matching `firearm` or `mechanical` launcher, and `apply_purchased_item` preserves existing ammunition when replacing a `firearm` or `mechanical` weapon); Python keeps ammunition in counters (merchant.py:138-144). Python's armory and sell-gear commands (src/portlight/app/cli.py:2506-2560, 1143-1183) have no Rust counterpart.

Proposed fix: none proposed for the Godot shop; the orphan-ammunition purchase refusal and launcher-upgrade ammunition preservation are resolved in `Session::buy_gear`. Building a gear shop, removing the duplicate formula, or showing merchant text remain design decisions (owner: sim and UI; size M). Until then canon says no player meets a merchant.

Why: 14 authored shopkeepers, their stock rules and their lines are invisible in the game build. Canon documents each as it is.

Status: ammunition loss resolved in sim; Godot merchant UI open, log only.

## DF-47. Coverage gaps: ports with no merchant, master, trainer or companion

Where:
- computed at the base commit from content.json, 20 ports:
- 6 ports have no merchant: merchants[*].port_id, content.json:5868 to 6075 (Corsair's Rest, Iron Point, Crosswind Isle, Dragon's Gate, Spice Narrows, Ember Isle)
- 10 ports have no style master (content.json:6171 to 6250), 15 have no skill trainer (content.json:5397 onward), 11 have no companion (content.json:5542 onward); only one skill exists (content.json:5367, id blacksmith)
- portlight@9b02494 src/portlight/content/merchants.py:70, 130, 177, 202 (Python comments say why for merchants: black market, mining port, trade and pirate ports, volcanic outpost)

Proposed fix: none proposed; logged only. Canon lists the gaps in each README (section G) and never invents data to fill them; an 'Invented filler (not in the game)' section exists in each, empty by default.

Why: a writer who assumes every port has a shop, a master, a trainer or a companion would be wrong for most ports; the gaps are facts about the data, not errors.

Status: open, log only.

## DF-48. The training row says 'needs 1 styles' (resolved)

Status: resolved (Godot copy). `CrewStyleLine` now formats `prerequisite_styles == 1` as "needs 1 style". The text below records the problem as found.

Where:
- crates/portlight-godot/src/logic.rs:2042-2045 (the row text is built with the format 'needs {} styles' and the style's prerequisite_styles number, with no singular case)
- content.json:4286 and :4327 (Silat and Lua, prerequisite_styles 1); the other three styles read 'needs 0 styles'

Proposed fix: build the phrase with singular and plural handling, for example 'needs 1 style' (owner: copy; size XS; a Godot string change with a logic test, not a COPY_OVERRIDES entry). Route it with the later copy PR that carries DF-36, DF-37, DF-40, DF-42, DF-43 and DF-44.

Why: a player-facing grammar slip on the crew desk, on every Silat and Lua row. Canon quotes the shipped row as it is.

## Resolved in lore (no data change)

- Stormwall 'answers to the Pact, not the crown' (C:13570 area): the crown is the old Northern Crown that raised the Fortress (T3) and dissolved (T8).
- Corsair's Rest: the Cliff Tavern (landmark) and Mama Lucia's Kitchen (institution) are one place. The Kitchen is the tavern's back room, carved deeper into the rock.
- Al-Manar's 'Senhora' governor: 'Senhora' is the Alliance's court honorific for any governing lady, adopted from Porto Novo's usage at the Alliance's signing (T5).
- Silva Bay tavern rumour about storm-lost ships: an in-world rumour, unreliable. Elena builds for anyone who respects ships.
- Pirate factions with no home port: canon assigns invented homes (README).
- Name reuse across ports (Mensah, Strand, Astrid, Esi): different people. Common names in a region, not kin. The same holds for given names shared between companions, merchants, trainers, officer-pool names and port NPCs (Marta, Rosa, Kai, Kofi, Yaa, Erik, Mariam; DF-37, DF-38): namesakes, not kin.
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
- Crew roles against companion roles (Marine, Navigator, Surgeon, Quartermaster): two systems with the same four names. Canon calls the hired role's person an officer and the named party member a companion; a captain can have both and they never share a record (companions/README.md, officers/README.md).
- Iron Marta 'deserted the garrison' (C:5549, C:5557), with no garrison named: canon reads it as Stormwall's garrison, at T38, after No One, Gunnar and the Iron Wolves (T31 to T33). [invented]
- Real-world words in data (Javanese, for Star Lila, C:5581): quoted as data and never extended; canon adds no real-world place.
- Dr. Amara's 'dockside clinic' (C:5617): Sun Harbor has no apothecary institution in the data, so canon reads it as a room by the Welcome Dock. [invented]
- Given-name overlap Red Hamish (Ironhaven, C:6190) and the bladesmith Hamish (Thornport, C:5947): two people. The same holds for Kai the Shaper (C:5436) with Shadow Kai (C:5641) and Elder Kai (C:10891), and for Forge Master Erik (C:5400) with Erik Strand (C:8533): namesakes, not kin (DF-37, DF-38).
- 'Forge Master' as a title for Erik (C:5400) and for Henrik Brandt (C:7983), both at Ironhaven: a craft and an office. Henrik heads the Guild; Erik is a master smith of the same trade. [invented]
- Three 'master' words in one trade: the skill level named Master (C:5388), the merchant Master Chen titled Master Smith (C:6004-6006) at Jade Port, and the fighting-style masters (style-masters/README.md): different things, read from context.
- Senora Vega's fencing school (C:6185, DF-40 resolved) against Maestro Luciano (C:6177): Luciano teaches on a warehouse floor; Vega's school has a walled courtyard and a ledger. [invented]
- Nneka Ironfist teaches 'with bruises, not words' (C:6217) yet speaks a full line (C:6218): the line is her first-day speech.
- Real-world words in master, trainer and style text (Scotsman, Norse, Castilian, Hausa, Polynesia, Hawaiian, Portuguese cannons, Spanish science of defense; C:6185, C:6193, C:6201, C:4215, C:4296, C:5431, C:4134): quoted as data and never extended; canon adds no real-world place.
- Silva Bay's three uses of the name Silva (the port, Old Silva the outfitter at C:5900, the Silva Bay Smithy at C:5418): unrelated uses of a place name.
- War Chief Tane's 'weapons are dear at Coral Throne' (C:11352, DF-44 resolved) alongside the Coral Throne shop's 1.08 markup (C:6057): reflects the tribute in arms the War Chief demands.
- Hamish's 'axes and blades' (C:5951, DF-43 resolved) and Li Wei's 'blowguns and arquebus pistols' (C:6036, DF-42 resolved): both descriptions now match the regional stock via `COPY_OVERRIDES`.
- Malia's key 'provisions' (C:6060) with an Outfitter's stock: the key is a data key never shown to a player; DF-45 is declined.
- Two titles shared by two merchants each: Armorer (Fatima, Devi) and Outfitter (Widow Kael, Malia) are trades, not offices; the shops are a coast apart.
- Marco is 'Genoese' (C:5875): a real-world word in shipped data, quoted as data and never extended; canon adds no real-world place.
