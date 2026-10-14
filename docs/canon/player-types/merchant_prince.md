# The Merchant Prince - player type (merchant_prince)

Base commit: c413788 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 5. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n; `S/` is crates/portlight-sim/src/, `G/` is crates/portlight-godot/src/. Legend: [exists file:line] in the sim today | [proposal; planned slice PT-Sn] intended, not shipped | [invented] lore | [flavour, no mechanic] | DF-n see ../data-fixes.md.
Who this captain is: [../captains/merchant_prince.md](../captains/merchant_prince.md). This file is how the type plays.

## 1. Identity and fantasy

Khoury Dynasty scion of Al-Manar. Every advantage and every expectation. [invented] where not data.

The player's verb: "I spend old money to make more" [invented]

## 2. Unique mechanics and perks

Starting values (all [exists], cited):

| Key | Value | Tag |
|---|---|---|
| Silver - ship - provisions | 700 - Coastal Sloop (sloop) - 30 | [exists C:2555-2557] |
| Trade terms | buy x0.9 - sell x1.08 - luxury bonus 0.12 - port fees x0.65 | [exists C:2558-2562] |
| Voyage terms | provision burn x1.0 - speed +0.0 - storm resist +0.0 - cargo damage x1.0 | [exists C:2564-2568] |
| Inspection terms | chance x0.4 - seizure 0.0 - fines x0.4 | [exists C:2570-2573] |
| Starting regard (data keys, not player terms) | commercial_trust 18, mediterranean 12, east_indies 5 | [exists C:2575; S/model.rs:255-270] |
| Pirate regard | none | [exists C:2575] |
| Contract bias (x2 offer weight) | none today | [exists crates/portlight-sim/src/contracts.rs:75-80] |
| captain_type branch | no captain_type branch; differs by data only | [exists custom_captain.rs:432-442 (catalog only)] |

Perks:

- Silver 700, the most of any type [exists C:2555]
- Buy x0.9, sell x1.08, luxury 0.12: the best trade terms [exists C:2559-2561]
- Commercial trust 18, Mediterranean 12 [exists C:2575]
- Inspection x0.4 (not the lowest; the Privateer is x0.3, DF-29) [exists C:2571]
- Luxury and charter contract bias [proposal; planned slice PT-S1]
- Family obligation: a periodic Khoury remittance or a trust-loss risk on failed contracts, sized to bring PT-S0 numbers into band. Trade-off by design [proposal; planned slice PT-S7]

## 3. How the 20 ports treat this type

Basis: [exists] means a real effect of this type's numbers (regional regard, pirate regard, fees, inspection, home or mentor); [invented] is flavour only and implies no mechanic. Fees and inspection terms apply at every port alike (C:2558-2573); no port has a per-type rule.

| Port | Bloc | How they treat this type | Basis |
|---|---|---|---|
| [Porto Novo](../ports/porto_novo.md) | [The Exchange Alliance](../factions/exchange_alliance.md) | starts with warm Mediterranean regard (12); home bloc; dynasty name opens counting rooms | [exists C:2575] [invented] |
| [Al-Manar](../ports/al_manar.md) | [The Exchange Alliance](../factions/exchange_alliance.md) | home port; the mentor is here; starts with warm Mediterranean regard (12); home bloc; dynasty name opens counting rooms | [exists C:2553-2554] [exists C:2575] [invented] |
| [Silva Bay](../ports/silva_bay.md) | [The Exchange Alliance](../factions/exchange_alliance.md) | starts with warm Mediterranean regard (12); home bloc; dynasty name opens counting rooms | [exists C:2575] [invented] |
| [Corsair's Rest](../ports/corsairs_rest.md) | [The Shadow Ports](../factions/shadow_ports.md) | starts with warm Mediterranean regard (12); Shadow Ports see a mark | [exists C:2575] [invented] |
| [Ironhaven](../ports/ironhaven.md) | [The Iron Pact](../factions/iron_pact.md) | the Pact respects money | [invented] |
| [Stormwall](../ports/stormwall.md) | [The Iron Pact](../factions/iron_pact.md) | the Pact respects money | [invented] |
| [Thornport](../ports/thornport.md) | [The Free Ports](../factions/free_ports.md) | Free Ports care for silver, not names | [invented] |
| [Sun Harbor](../ports/sun_harbor.md) | [The Gold Coast Compact](../factions/gold_coast_compact.md) | Compact brokers court the dynasty | [invented] |
| [Palm Cove](../ports/palm_cove.md) | [The Gold Coast Compact](../factions/gold_coast_compact.md) | Compact brokers court the dynasty | [invented] |
| [Iron Point](../ports/iron_point.md) | [The Gold Coast Compact](../factions/gold_coast_compact.md) | Compact brokers court the dynasty | [invented] |
| [Pearl Shallows](../ports/pearl_shallows.md) | [The Gold Coast Compact](../factions/gold_coast_compact.md) | Compact brokers court the dynasty | [invented] |
| [Jade Port](../ports/jade_port.md) | [The Silk Circle](../factions/silk_circle.md) | starts with warm East Indies regard (5); Silk Circle buyers know Khoury seals | [exists C:2575] [invented] |
| [Monsoon Reach](../ports/monsoon_reach.md) | [The Free Ports](../factions/free_ports.md) | starts with warm East Indies regard (5); Free Ports care for silver, not names | [exists C:2575] [invented] |
| [Silk Haven](../ports/silk_haven.md) | [The Silk Circle](../factions/silk_circle.md) | starts with warm East Indies regard (5); Silk Circle buyers know Khoury seals | [exists C:2575] [invented] |
| [Crosswind Isle](../ports/crosswind_isle.md) | [The Free Ports](../factions/free_ports.md) | starts with warm East Indies regard (5); Free Ports care for silver, not names | [exists C:2575] [invented] |
| [Dragon's Gate](../ports/dragons_gate.md) | [The Silk Circle](../factions/silk_circle.md) | starts with warm East Indies regard (5); Silk Circle buyers know Khoury seals | [exists C:2575] [invented] |
| [Spice Narrows](../ports/spice_narrows.md) | [The Shadow Ports](../factions/shadow_ports.md) | starts with warm East Indies regard (5); Shadow Ports see a mark | [exists C:2575] [invented] |
| [Ember Isle](../ports/ember_isle.md) | [The Coral Crown](../factions/coral_crown.md) | Coral Crown courts expect gifts | [invented] |
| [Typhoon Anchorage](../ports/typhoon_anchorage.md) | [The Coral Crown](../factions/coral_crown.md) | Coral Crown courts expect gifts | [invented] |
| [Coral Throne](../ports/coral_throne.md) | [The Coral Crown](../factions/coral_crown.md) | Coral Crown courts expect gifts | [invented] |

## 4. Reputation and faction effects

- Starting regard is copied from the type's data into the standing at new game (S/model.rs:255-270). Regional customs inspection rises with heat in a region (S/reputation.rs:216-227). Pirate hostility reads pirate regard (S/encounter.rs:42-62).
- Starts Mediterranean 12, East Indies 5, trust 18 (C:2575)
- No pirate regard
- No spillover: regard with one bloc or faction does not move its rivals (DF-28). Canon claims none.

## 5. Progression

Uses existing systems: milestone and career scoring (S/campaign.rs:1245-1310), the ship ladder (C:131 ships) and contract offers (S/contracts.rs:75-80).

- Early: Luxury buys from Al-Manar
- Mid: Charters; PT-S7 obligation begins to bite
- Late: Trading house and bank

## 6. Look and feel (words only)

Words, not art direction. Character plates stay held; canon sets no face, age, gender or dress for the captain (../captains/README.md rules).

- Mood: composed, rich, unhurried - restraint that still costs money [invented; AD note]
- Materials: lacquered teak panelling, tooled-leather chests, a silver-chased sea chest, a brocade-covered cabin table [invented; AD note]
- Silhouette: high and ornate - a tall stern, layered trim, heavy upper mass [invented; AD note]
- Palette: deep claret-brown, tarnished silver, dark teak; mid-low values, warm; small accent of an old-gilt edge (thin lines only) [invented; AD note]
- A polished deck [invented]
- Cabin: silk, a family seal [invented]
- The crew are paid on time and say so [invented]

## 7. Sound hooks

- sfx_ui_trade_coin (G/game.rs:4749): a heavy purse
- sfx_ui_contract_accept (G/game.rs:6861): the family seal
- New sounds are ideas only: a seal press on a remittance (idea, no asset requested)

## 8. Flags

- DF-29: Python 'lowest inspections' (C:2571 0.4 vs C:2453 0.3)
