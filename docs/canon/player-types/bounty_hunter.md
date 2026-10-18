# The Bounty Hunter - player type (bounty_hunter)

Base commit: c413788 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 5. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n; `S/` is crates/portlight-sim/src/, `G/` is crates/portlight-godot/src/. Legend: [exists file:line] in the sim today | [proposal; planned slice PT-Sn] intended, not shipped | [invented] lore | [flavour, no mechanic] | DF-n see ../data-fixes.md.
Who this captain is: [../captains/bounty_hunter.md](../captains/bounty_hunter.md). This file is how the type plays.

## 1. Identity and fantasy

Also called 'Bounty Hunter'. A freelance hunter in a fast cutter. [invented] where not data.

The player's verb: "The board is my trade route" [invented]

## 2. Unique mechanics and perks

Starting values (all [exists], cited):

| Key | Value | Tag |
|---|---|---|
| Silver - ship - provisions | 425 - Swift Cutter (cutter) - 30 | [exists C:2631-2633] |
| Trade terms | buy x1.05 - sell x0.98 - luxury bonus 0.0 - port fees x0.7 | [exists C:2634-2638] |
| Voyage terms | provision burn x0.9 - speed +0.5 - storm resist +0.05 - cargo damage x1.0 | [exists C:2640-2644] |
| Inspection terms | chance x0.6 - seizure 0.0 - fines x0.5 | [exists C:2646-2649] |
| Starting regard (data keys, not player terms) | commercial_trust 3, mediterranean 3, north_atlantic 8 | [exists C:2651; S/model.rs:255-270] |
| Pirate regard | none | [exists C:2651] |
| Contract bias (x2 offer weight) | none today | [exists crates/portlight-sim/src/contracts.rs:75-80] |
| captain_type branch | none; the bounty board is open to every type | [exists S/bounty.rs:222-241] |

Perks:

- Swift Cutter start [exists C:2632]
- Inspection x0.6, fines x0.5, fees x0.7 [exists C:2638, C:2647-2649]
- North Atlantic regard 8 [exists C:2651]
- Bounty edge: a claim pays x1.25 the table reward (silver only), and the hunt cards and day report show the paid amount (Raj the Quiet 120 on the table, 150 paid; Scarlet Ana 150 on the table, 188 paid, since 187.5 rounds half to even); bounties stay open to all. Trade-off: poor trade terms (buy x1.05, sell x0.98) [exists S/bounty.rs:219-220; overlay row captains[id=bounty_hunter].bounty_reward_mult in tuned_overlay.json]

## 3. How the 20 ports treat this type

Basis: [exists] means a real effect of this type's numbers (regional regard, pirate regard, fees, inspection, home or mentor); [invented] is flavour only and implies no mechanic. Fees and inspection terms apply at every port alike (C:2634-2649); no port has a per-type rule.

| Port | Bloc | How they treat this type | Basis |
|---|---|---|---|
| [Porto Novo](../ports/porto_novo.md) | [The Exchange Alliance](../factions/exchange_alliance.md) | starts with warm Mediterranean regard (3); Alliance ports pay for order | [exists C:2651] [invented] |
| [Al-Manar](../ports/al_manar.md) | [The Exchange Alliance](../factions/exchange_alliance.md) | starts with warm Mediterranean regard (3); Alliance ports pay for order | [exists C:2651] [invented] |
| [Silva Bay](../ports/silva_bay.md) | [The Exchange Alliance](../factions/exchange_alliance.md) | starts with warm Mediterranean regard (3); Alliance ports pay for order | [exists C:2651] [invented] |
| [Corsair's Rest](../ports/corsairs_rest.md) | [The Shadow Ports](../factions/shadow_ports.md) | starts with warm Mediterranean regard (3); Shadow Ports hide the hunted | [exists C:2651] [invented] |
| [Ironhaven](../ports/ironhaven.md) | [The Iron Pact](../factions/iron_pact.md) | starts with warm North Atlantic regard (8); the Pact issued the commission in lore | [exists C:2651] [invented] |
| [Stormwall](../ports/stormwall.md) | [The Iron Pact](../factions/iron_pact.md) | starts with warm North Atlantic regard (8); the Pact issued the commission in lore | [exists C:2651] [invented] |
| [Thornport](../ports/thornport.md) | [The Free Ports](../factions/free_ports.md) | starts with warm North Atlantic regard (8); home bloc; Dock Master Tao | [exists C:2651] [invented] |
| [Sun Harbor](../ports/sun_harbor.md) | [The Gold Coast Compact](../factions/gold_coast_compact.md) | the Compact pays for pirate heads | [invented] |
| [Palm Cove](../ports/palm_cove.md) | [The Gold Coast Compact](../factions/gold_coast_compact.md) | the Compact pays for pirate heads | [invented] |
| [Iron Point](../ports/iron_point.md) | [The Gold Coast Compact](../factions/gold_coast_compact.md) | the Compact pays for pirate heads | [invented] |
| [Pearl Shallows](../ports/pearl_shallows.md) | [The Gold Coast Compact](../factions/gold_coast_compact.md) | the Compact pays for pirate heads | [invented] |
| [Jade Port](../ports/jade_port.md) | [The Silk Circle](../factions/silk_circle.md) | Silk Circle hires hunters quietly | [invented] |
| [Monsoon Reach](../ports/monsoon_reach.md) | [The Free Ports](../factions/free_ports.md) | home bloc; Dock Master Tao | [invented] |
| [Silk Haven](../ports/silk_haven.md) | [The Silk Circle](../factions/silk_circle.md) | Silk Circle hires hunters quietly | [invented] |
| [Crosswind Isle](../ports/crosswind_isle.md) | [The Free Ports](../factions/free_ports.md) | home port; the mentor is here; home bloc; Dock Master Tao | [exists C:2629-2630] [invented] |
| [Dragon's Gate](../ports/dragons_gate.md) | [The Silk Circle](../factions/silk_circle.md) | Silk Circle hires hunters quietly | [invented] |
| [Spice Narrows](../ports/spice_narrows.md) | [The Shadow Ports](../factions/shadow_ports.md) | Shadow Ports hide the hunted | [invented] |
| [Ember Isle](../ports/ember_isle.md) | [The Coral Crown](../factions/coral_crown.md) | Coral Crown pays bounties for reef raiders | [invented] |
| [Typhoon Anchorage](../ports/typhoon_anchorage.md) | [The Coral Crown](../factions/coral_crown.md) | Coral Crown pays bounties for reef raiders | [invented] |
| [Coral Throne](../ports/coral_throne.md) | [The Coral Crown](../factions/coral_crown.md) | Coral Crown pays bounties for reef raiders | [invented] |

## 4. Reputation and faction effects

- Starting regard is copied from the type's data into the standing at new game (S/model.rs:255-270). Regional customs inspection rises with heat in a region (S/reputation.rs:216-227). Pirate hostility reads pirate regard (S/encounter.rs:42-62).
- Starts North Atlantic 8, Mediterranean 3, trust 3 (C:2651)
- No pirate regard: pirates start neutral, not hostile (Python copy says otherwise)
- No spillover: regard with one bloc or faction does not move its rivals (DF-28). Canon claims none.

## 5. Progression

Uses existing systems: milestone and career scoring (S/campaign.rs:1245-1310), the ship ladder (C:131 ships) and contract offers (S/contracts.rs:75-80).

- Early: Cheap bounties: Raj the Quiet 120 on the table, 150 paid (S/bounty.rs:59); Scarlet Ana 150 on the table, 188 paid (S/bounty.rs:41)
- Mid: Strength 7 targets
- Late: The Butcher (220) and Gnaw (200, DF-30)

## 6. Look and feel (words only)

Words, not art direction. Character plates stay held; canon sets no face, age, gender or dress for the captain (../captains/README.md rules).

- Mood: cold, watchful, purposeful - quiet like a held breath [invented; look note]
- Materials: dark-oiled leather, manacles and chain stowed tidy, a spyglass, a pinned board of notices in the cabin [invented; look note]
- Silhouette: sharp and long - forward-leaning, a keen bow, thin hard lines [invented; look note]
- Palette: gunmetal, storm-slate, oxblood leather; mid-low values, cool, the highest contrast of the nine; small accent of a notice-paper off-white [invented; look note]
- A cutter kept fast [invented]
- Cabin: wanted notices pinned [invented]
- The crew talk about targets [invented]
- Closest pair is The Privateer and The Bounty Hunter: the Privateer is square, scrubbed and lighter; the Bounty Hunter is sharp, oiled and darker [invented; look note]

## 7. Sound hooks

- sfx_ui_contracts_open (G/game.rs:6826): reading the board
- sfx_ui_trade_coin (G/game.rs:4749): a bounty paid
- New sounds are ideas only: a bell on a claimed bounty (idea, no asset requested)

## 8. Flags

- DF-3 resolved: home region aligned to East Indies (`crosswind_isle`)
- DF-26 resolved: the Privateer was retitled; 'Licensed Hunter' is now the Bounty Hunter's alone
- DF-33: shares home port and mentor with the Dockhand
