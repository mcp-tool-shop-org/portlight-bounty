# Canon data fixes (proposals only; nothing changed)

Written 2026-10-14. Base: 143e284. Found while writing docs/canon phase 1. Each item gives where (file:line), the proposed fix and why. None is applied: the game is unchanged. Sim data changes need their own PR, with a parity note where gossip, sighting or save text is involved.

Count: 17.

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

## Resolved in lore (no data change)

- Stormwall 'answers to the Pact, not the crown' (C:13570 area): the crown is the old Northern Crown that raised the Fortress (T3) and dissolved (T8).
- Corsair's Rest: the Cliff Tavern (landmark) and Mama Lucia's Kitchen (institution) are one place. The Kitchen is the tavern's back room, carved deeper into the rock.
- Al-Manar's 'Senhora' governor: 'Senhora' is the Alliance's court honorific for any governing lady, adopted from Porto Novo's usage at the Alliance's signing (T5).
- Silva Bay tavern rumour about storm-lost ships: an in-world rumour, unreliable. Elena builds for anyone who respects ships.
- Pirate factions with no home port: canon assigns invented homes (README).
- Name reuse across ports (Mensah, Strand, Astrid, Esi): different people. Common names in a region, not kin.
- Map scale (route distance against chart distance): route distance is sailing days under prevailing winds, not straight-line distance.
- Bloc against region mismatches (Free Ports and Shadow Ports span regions): blocs are political, regions are geographic, by design.
