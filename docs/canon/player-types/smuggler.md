# The Smuggler - player type (smuggler)

Base commit: c413788 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 5. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n; `S/` is crates/portlight-sim/src/, `G/` is crates/portlight-godot/src/. Legend: [exists file:line] in the sim today | [proposal; planned slice PT-Sn] intended, not shipped | [invented] lore | [flavour, no mechanic] | DF-n see ../data-fixes.md.
Who this captain is: [../captains/smuggler.md](../captains/smuggler.md). This file is how the type plays.

## 1. Identity and fantasy

The shadow trader of Palm Cove. Luxury margins and quiet holds, always one inspection from ruin. The fantasy is the hidden compartment and the friendly pirate wave. [invented] where not data.

The player's verb: "I sell what others can't" [invented]

## 2. Unique mechanics and perks

Starting values (all [exists], cited):

| Key | Value | Tag |
|---|---|---|
| Silver - ship - provisions | 500 - Coastal Sloop (sloop) - 35 | [exists C:2361-2363] |
| Trade terms | buy x1.0 - sell x0.98 - luxury bonus 0.28 - port fees x0.9 | [exists C:2364-2368] |
| Voyage terms | provision burn x0.85 - speed +0.0 - storm resist +0.0 - cargo damage x0.8 | [exists C:2370-2374] |
| Inspection terms | chance x1.3 - seizure 0.05 - fines x1.2 | [exists C:2376-2379] |
| Starting regard (data keys, not player terms) | customs_heat 8, west_africa 5 | [exists C:2381; S/model.rs:255-270] |
| Pirate regard | none | [exists C:2381] |
| Contract bias (x2 offer weight) | proc_weapons_garrison, short_rum_drought, lux_silk_collector, lux_porcelain_estate, lux_spice_monopolist, lux_pearls_crown, smug_opium_run, smug_powder_delivery, smug_fence_the_take, smug_faction_supply | [exists crates/portlight-sim/src/contracts.rs:75-80] |
| captain_type branch | five branches: inspection suspicion +1, contraband detection -0.15, +5 effective regard with friendly or respectful pirate factions, +15 shadow-path score | [exists S/reputation.rs:58-60; S/voyage.rs:473-475; S/encounter.rs:49-56; S/campaign.rs:1304-1307] |

Perks:

- Luxury sell bonus 0.28, the largest of any type [exists C:2367]
- Contraband is harder to find aboard (detection -0.15) [exists S/voyage.rs:473-475]
- +5 effective regard with pirate factions whose attitude to smugglers is friendly or respectful (Monsoon Syndicate, Deep Reef Brotherhood) [exists S/encounter.rs:49-56; C:2893, C:2902]
- Provision burn x0.85 and cargo damage x0.8 [exists C:2371, C:2374]
- Smuggling and luxury contract bias (10 templates) [exists S/contracts.rs:75-80]
- Shadow-path career score +15 [exists S/campaign.rs:1304-1307]
- Trade-off: inspection x1.3, seizure 0.05, fines x1.2, suspicion +1, starts with heat 8 [exists C:2377-2379, C:2381; S/reputation.rs:58-60]
- Buy terms are x1.0, not the '5% cheaper buys' of the Python copy (DF-29) [exists C:2365]
- No new perk planned: already the most distinct type; PT-S0 checks it [proposal; planned slice PT-S0]

## 3. How the 20 ports treat this type

Basis: [exists] means a real effect of this type's numbers (regional regard, pirate regard, fees, inspection, home or mentor); [invented] is flavour only and implies no mechanic. Fees and inspection terms apply at every port alike (C:2364-2379); no port has a per-type rule.

| Port | Bloc | How they treat this type | Basis |
|---|---|---|---|
| [Porto Novo](../ports/porto_novo.md) | [The Exchange Alliance](../factions/exchange_alliance.md) | Alliance inspectors watch a shadow trader closely | [invented] |
| [Al-Manar](../ports/al_manar.md) | [The Exchange Alliance](../factions/exchange_alliance.md) | Alliance inspectors watch a shadow trader closely | [invented] |
| [Silva Bay](../ports/silva_bay.md) | [The Exchange Alliance](../factions/exchange_alliance.md) | Alliance inspectors watch a shadow trader closely | [invented] |
| [Corsair's Rest](../ports/corsairs_rest.md) | [The Shadow Ports](../factions/shadow_ports.md) | Shadow Ports are natural markets; black-market friendly shore | [invented] [flavour, no mechanic] |
| [Ironhaven](../ports/ironhaven.md) | [The Iron Pact](../factions/iron_pact.md) | Pact customs are strict; a hold is searched | [invented] |
| [Stormwall](../ports/stormwall.md) | [The Iron Pact](../factions/iron_pact.md) | Pact customs are strict; a hold is searched | [invented] |
| [Thornport](../ports/thornport.md) | [The Free Ports](../factions/free_ports.md) | Free Ports ask no questions | [invented] |
| [Sun Harbor](../ports/sun_harbor.md) | [The Gold Coast Compact](../factions/gold_coast_compact.md) | starts with warm West Africa regard (5); home region bloc; Palm Cove knows the type | [exists C:2381] [invented] |
| [Palm Cove](../ports/palm_cove.md) | [The Gold Coast Compact](../factions/gold_coast_compact.md) | home port; the mentor is here; starts with warm West Africa regard (5); home region bloc; Palm Cove knows the type | [exists C:2359-2360] [exists C:2381] [invented] |
| [Iron Point](../ports/iron_point.md) | [The Gold Coast Compact](../factions/gold_coast_compact.md) | starts with warm West Africa regard (5); home region bloc; Palm Cove knows the type | [exists C:2381] [invented] |
| [Pearl Shallows](../ports/pearl_shallows.md) | [The Gold Coast Compact](../factions/gold_coast_compact.md) | starts with warm West Africa regard (5); home region bloc; Palm Cove knows the type | [exists C:2381] [invented] |
| [Jade Port](../ports/jade_port.md) | [The Silk Circle](../factions/silk_circle.md) | Silk Circle buyers pay for discreet luxury | [invented] |
| [Monsoon Reach](../ports/monsoon_reach.md) | [The Free Ports](../factions/free_ports.md) | Free Ports ask no questions | [invented] |
| [Silk Haven](../ports/silk_haven.md) | [The Silk Circle](../factions/silk_circle.md) | Silk Circle buyers pay for discreet luxury | [invented] |
| [Crosswind Isle](../ports/crosswind_isle.md) | [The Free Ports](../factions/free_ports.md) | Free Ports ask no questions | [invented] |
| [Dragon's Gate](../ports/dragons_gate.md) | [The Silk Circle](../factions/silk_circle.md) | Silk Circle buyers pay for discreet luxury | [invented] |
| [Spice Narrows](../ports/spice_narrows.md) | [The Shadow Ports](../factions/shadow_ports.md) | Shadow Ports are natural markets; black-market friendly shore | [invented] [flavour, no mechanic] |
| [Ember Isle](../ports/ember_isle.md) | [The Coral Crown](../factions/coral_crown.md) | Coral Crown courts trade pearls quietly | [invented] |
| [Typhoon Anchorage](../ports/typhoon_anchorage.md) | [The Coral Crown](../factions/coral_crown.md) | Coral Crown courts trade pearls quietly | [invented] |
| [Coral Throne](../ports/coral_throne.md) | [The Coral Crown](../factions/coral_crown.md) | Coral Crown courts trade pearls quietly | [invented] |

## 4. Reputation and faction effects

- Starting regard is copied from the type's data into the standing at new game (S/model.rs:255-270). Regional customs inspection rises with heat in a region (S/reputation.rs:216-227). Pirate hostility reads pirate regard (S/encounter.rs:42-62).
- Starts with West Africa 5 and customs heat 8 in every region (C:2381; S/model.rs:255-270)
- Heat raises regional inspection once it reaches 15 (S/reputation.rs:216-227)
- Monsoon Syndicate and Deep Reef Brotherhood meet the Smuggler warmer (+5) (S/encounter.rs:49-56)
- No spillover: regard with one bloc or faction does not move its rivals (DF-28). Canon claims none.

## 5. Progression

Uses existing systems: milestone and career scoring (S/campaign.rs:1245-1310), the ship ladder (C:131 ships) and contract offers (S/contracts.rs:75-80).

- Early: Luxury runs out of Palm Cove; keep heat under 15
- Mid: Smuggling contracts and faction supply jobs
- Late: Shadow-path career (S/campaign.rs:1304-1307)

## 6. Look and feel (words only)

Words, not art direction. Character plates stay held; canon sets no face, age, gender or dress for the captain (../captains/README.md rules).

- Mood: hushed, close, ready to vanish - nothing on deck that catches light [invented; AD note]
- Materials: tarred canvas covers, false-floor planking, muffled oars, a shuttered lantern with smoked glass [invented; AD note]
- Silhouette: low and lean - flat sheer, nothing tall above the rail [invented; AD note]
- Palette: tar-black shadows, bilge green-grey, damp rope brown; low-end values, cool; tiny accent of shuttered-lamp amber [invented; AD note]
- A plain deck that hides a second floor [invented]
- Cabin: false panels, a lamp kept low [invented]
- Flag manners: the honest flag, struck to the letter of the law [invented]
- The crew talk little and never about the hold [invented]

## 7. Sound hooks

- sfx_ui_trade_coin (G/game.rs:4749): a quick, quiet sale
- sfx_chart_sail_depart (G/game.rs:4529): slipping out before the inspector
- New sounds are ideas only: a hidden panel latch (idea, no asset requested)

## 8. Flags

- DF-29: Python perk copy claims 5% cheaper buys (C:2365 is 1.0)
