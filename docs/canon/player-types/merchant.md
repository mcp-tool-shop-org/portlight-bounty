# The Merchant - player type (merchant)

Base commit: c413788 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 5. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n; `S/` is crates/portlight-sim/src/, `G/` is crates/portlight-godot/src/. Legend: [exists file:line] in the sim today | [proposal; planned slice PT-Sn] intended, not shipped | [invented] lore | [flavour, no mechanic] | DF-n see ../data-fixes.md.
Who this captain is: [../captains/merchant.md](../captains/merchant.md). This file is how the type plays.

## 1. Identity and fantasy

Mike's 'Trader'. A guild-licensed operator whose edge is the ledger: steady prices, cheap fees and a clean name in the settled ports. The fantasy is the trading house built one honest margin at a time. [invented] where not data; the trade and inspection numbers are data (C:2326-2341).

The player's verb: "I buy low, sell on time" [invented]

## 2. Unique mechanics and perks

Starting values (all [exists], cited):

| Key | Value | Tag |
|---|---|---|
| Silver - ship - provisions | 550 - Coastal Sloop (sloop) - 30 | [exists C:2323-2325] |
| Trade terms | buy x0.92 - sell x1.05 - luxury bonus 0.0 - port fees x0.7 | [exists C:2326-2330] |
| Voyage terms | provision burn x1.0 - speed +0.0 - storm resist +0.0 - cargo damage x1.0 | [exists C:2332-2336] |
| Inspection terms | chance x0.5 - seizure 0.0 - fines x0.6 | [exists C:2338-2341] |
| Starting regard (data keys, not player terms) | commercial_trust 15, mediterranean 10 | [exists C:2343; S/model.rs:255-270] |
| Pirate regard | none | [exists C:2343] |
| Contract bias (x2 offer weight) | proc_grain_feed, proc_timber_shipyard, proc_iron_forge, proc_medicines_relief, short_grain_famine, short_tea_crisis, lux_porcelain_estate, lux_pearls_crown, ret_spice_restock, ret_dyes_textile, rep_charter_luxury, rep_charter_atlantic, smug_fence_the_take | [exists crates/portlight-sim/src/contracts.rs:75-80] |
| captain_type branch | clean high-margin sale (suspicion under 3, margin over 20 pct) adds commercial trust | [exists S/reputation.rs:107-109] |

Perks:

- Best-in-class buy and sell terms after the Merchant Prince (buy x0.92, sell x1.05) [exists C:2327-2328]
- Port fees x0.7 everywhere [exists C:2330]
- Half the inspection chance and lower fines (x0.5, x0.6) [exists C:2339-2341]
- Clean high-margin sales build commercial trust (unique branch) [exists S/reputation.rs:107-109]
- Widest contract bias of any type: 13 templates across procurement, shortage, luxury, return, charter and one fence job [exists S/contracts.rs:75-80]
- Default first captain on the new game screen [exists crates/portlight-chart/src/view.rs:21]
- No new perk planned: existing knobs already distinguish the type; revisit only if the PT-S0 sweep says so [proposal; planned slice PT-S0]
- Trade-off: no voyage or luxury edge [exists C:2329, C:2333-2336]

## 3. How the 20 ports treat this type

Basis: [exists] means a real effect of this type's numbers (regional regard, pirate regard, fees, inspection, home or mentor); [invented] is flavour only and implies no mechanic. Fees and inspection terms apply at every port alike (C:2326-2341); no port has a per-type rule.

| Port | Bloc | How they treat this type | Basis |
|---|---|---|---|
| [Porto Novo](../ports/porto_novo.md) | [The Exchange Alliance](../factions/exchange_alliance.md) | home port; the mentor is here; starts with warm Mediterranean regard (10); the Alliance's own: licensed, posted-price trading suits them | [exists C:2321-2322] [exists C:2343] [invented] |
| [Al-Manar](../ports/al_manar.md) | [The Exchange Alliance](../factions/exchange_alliance.md) | starts with warm Mediterranean regard (10); the Alliance's own: licensed, posted-price trading suits them | [exists C:2343] [invented] |
| [Silva Bay](../ports/silva_bay.md) | [The Exchange Alliance](../factions/exchange_alliance.md) | starts with warm Mediterranean regard (10); the Alliance's own: licensed, posted-price trading suits them | [exists C:2343] [invented] |
| [Corsair's Rest](../ports/corsairs_rest.md) | [The Shadow Ports](../factions/shadow_ports.md) | starts with warm Mediterranean regard (10); the Shadow Ports see a guild trader as a customer, not a friend | [exists C:2343] [invented] |
| [Ironhaven](../ports/ironhaven.md) | [The Iron Pact](../factions/iron_pact.md) | the Pact deals with licensed traders, but iron is political | [invented] |
| [Stormwall](../ports/stormwall.md) | [The Iron Pact](../factions/iron_pact.md) | the Pact deals with licensed traders, but iron is political | [invented] |
| [Thornport](../ports/thornport.md) | [The Free Ports](../factions/free_ports.md) | Free Ports take anyone's silver; the licence opens no extra door | [invented] |
| [Sun Harbor](../ports/sun_harbor.md) | [The Gold Coast Compact](../factions/gold_coast_compact.md) | Compact brokers honour a guild letter as written | [invented] |
| [Palm Cove](../ports/palm_cove.md) | [The Gold Coast Compact](../factions/gold_coast_compact.md) | Compact brokers honour a guild letter as written | [invented] |
| [Iron Point](../ports/iron_point.md) | [The Gold Coast Compact](../factions/gold_coast_compact.md) | Compact brokers honour a guild letter as written | [invented] |
| [Pearl Shallows](../ports/pearl_shallows.md) | [The Gold Coast Compact](../factions/gold_coast_compact.md) | Compact brokers honour a guild letter as written | [invented] |
| [Jade Port](../ports/jade_port.md) | [The Silk Circle](../factions/silk_circle.md) | the Silk Circle prizes a patient buyer over a loud one | [invented] |
| [Monsoon Reach](../ports/monsoon_reach.md) | [The Free Ports](../factions/free_ports.md) | Free Ports take anyone's silver; the licence opens no extra door | [invented] |
| [Silk Haven](../ports/silk_haven.md) | [The Silk Circle](../factions/silk_circle.md) | the Silk Circle prizes a patient buyer over a loud one | [invented] |
| [Crosswind Isle](../ports/crosswind_isle.md) | [The Free Ports](../factions/free_ports.md) | Free Ports take anyone's silver; the licence opens no extra door | [invented] |
| [Dragon's Gate](../ports/dragons_gate.md) | [The Silk Circle](../factions/silk_circle.md) | the Silk Circle prizes a patient buyer over a loud one | [invented] |
| [Spice Narrows](../ports/spice_narrows.md) | [The Shadow Ports](../factions/shadow_ports.md) | the Shadow Ports see a guild trader as a customer, not a friend | [invented] |
| [Ember Isle](../ports/ember_isle.md) | [The Coral Crown](../factions/coral_crown.md) | Coral Crown courts weigh tribute before trade | [invented] |
| [Typhoon Anchorage](../ports/typhoon_anchorage.md) | [The Coral Crown](../factions/coral_crown.md) | Coral Crown courts weigh tribute before trade | [invented] |
| [Coral Throne](../ports/coral_throne.md) | [The Coral Crown](../factions/coral_crown.md) | Coral Crown courts weigh tribute before trade | [invented] |

## 4. Reputation and faction effects

- Starting regard is copied from the type's data into the standing at new game (S/model.rs:255-270). Regional customs inspection rises with heat in a region (S/reputation.rs:216-227). Pirate hostility reads pirate regard (S/encounter.rs:42-62).
- Starts warm in the Mediterranean (10) and with commercial trust 15 (C:2343)
- No pirate regard: every pirate faction meets the Merchant neutral (S/encounter.rs:42-62)
- Clean high-margin sales are the type's one trust engine (S/reputation.rs:107-109)
- No spillover: regard with one bloc or faction does not move its rivals (DF-28). Canon claims none.

## 5. Progression

Uses existing systems: milestone and career scoring (S/campaign.rs:1245-1310), the ship ladder (C:131 ships) and contract offers (S/contracts.rs:75-80).

- Early: Grain and timber runs out of Porto Novo on contract offers weighted to the type (S/contracts.rs:75-80)
- Mid: Return freight and Atlantic charters; trade up the ship ladder for cargo
- Late: Luxury and charter contracts; a counting house account (banks) and milestone scoring as a trading house (S/campaign.rs:1245-1310)

## 6. Look and feel (words only)

Words, not art direction. Character plates stay held; canon sets no face, age, gender or dress for the captain (../captains/README.md rules).

- Mood: orderly, warm, patient - a tidy deck that looks like it pays its bills [invented; AD note]
- Materials: oiled oak cargo hatches, hemp-lashed crates, a brass balance scale and tally board in the cabin [invented; AD note]
- Silhouette: squat and broad - stacked cargo, low rounded mass [invented; AD note]
- Palette: honey oak, sacking tan, worn-brass glint; mid-warm values; one small accent of ledger-ribbon red [invented; AD note]
- A tidy deck, cargo lashed in rows and a tally board by the hatch [invented]
- Cabin: a ledger desk, a scale, the guild licence framed [invented]
- Flies the Exchange Guild flag correctly and early [invented]
- The crew talk in prices and dates [invented]

## 7. Sound hooks

- sfx_ui_trade_coin (G/game.rs:4749): the type's signature; every clean sale is this coin
- sfx_ui_contract_accept (G/game.rs:6861): a signed guild contract
- sfx_ui_port_open (G/game.rs:3137): arrival at a settled port
- New sounds are ideas only: a ledger page turn on a clean high-margin sale (idea, no asset requested)

## 8. Flags

- None.
