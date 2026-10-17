# Merchant canon index

Base commit: d2b544f (main, which includes phases 6A and 6B; content.json, the sim and the godot UI as cited are unchanged since 515a488). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at d2b544f. Python cites read portlight@9b02494 path:line. Legend: [exists] code or data fact with a cite | [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

Every table row below that carries a cite is [exists] (a code or data fact); rows tagged otherwise say so.

| Key | Value | Cite |
|---|---|---|
| Merchants (named, authored) | 14 | C:5868 |
| Ports with a merchant | 14 of 20 (one each) | merchants[*].port_id |
| Ports with none | 6 (section G) | ports[*].id against merchants[*].port_id |
| Item types a merchant may sell | melee, armor, ranged, ammo | content.rs:376-386; session.rs:3553-3609 |
| Markup range | x1.05 to x1.15 | merchants[*].price_markup |
| Where the sim uses them | buy_gear: script runner and tests only | session.rs:3209-3245; script.rs:422-428 |
| Rust record shape | id, name, port, title, personality, description, greeting, types, markup | content.rs:376-386 |
| Python reference | same 14 records, same strings (compared field by field, no differences) | portlight@9b02494 src/portlight/content/merchants.py:36-203 |

Merchants are the authored gear sellers (weapons, armour, ammunition). Port NPCs, companions, officers, style masters and skill trainers stay in their own files; the port files already carry a one-line row for each merchant (section B).

## 1. What the player sees today

Every table row below that carries a cite is [exists] (a code or data fact); rows tagged otherwise say so.

| What | Reaches the player? | Cite |
|---|---|---|
| A gear shop, a merchant list, or any merchant name | No: the Godot build has no shop and no merchant screen | rg of crates/portlight-godot/src for buy_gear and merchants finds nothing |
| Merchant title, personality, description, greeting | No: no reader in the crates | content.rs:376-386 (fields only) |
| Markup and stock | No: reachable only by the script runner's gear command | script.rs:422-428; session.rs:3209-3245 |
| Gear gained after a fight | Yes, in the sim: loot can equip a weapon, armour or ammunition | loot.rs:113-160 |
| Character art | None: the character hold stands | crew_screen.rs:3-6 |

## 2. Index (14 merchants)

<!-- hand-maintained:begin -->
| Merchant | Title | Port | Region | Bloc | Types | Markup | Cite | File |
|---|---|---|---|---|---|---|---|---|
| Marco | Master Weaponsmith | [Porto Novo](../ports/porto_novo.md) | Mediterranean | [The Exchange Alliance](../factions/exchange_alliance.md) | melee, ranged, ammo | x1.1 | C:5870 | [marco_the_blade.md](marco_the_blade.md) |
| Fatima | Armorer | [Al-Manar](../ports/al_manar.md) | Mediterranean | [The Exchange Alliance](../factions/exchange_alliance.md) | armor, melee | x1.15 | C:5885 | [fatima_armorworks.md](fatima_armorworks.md) |
| Old Silva | Ship Outfitter | [Silva Bay](../ports/silva_bay.md) | Mediterranean | [The Exchange Alliance](../factions/exchange_alliance.md) | melee, ranged, ammo, armor | x1.05 | C:5899 | [old_silva.md](old_silva.md) |
| Sgt. Gruber | Arsenal Master | [Ironhaven](../ports/ironhaven.md) | North Atlantic | [The Iron Pact](../factions/iron_pact.md) | melee, armor, ranged, ammo | x1.12 | C:5915 | [sergeant_gruber.md](sergeant_gruber.md) |
| Widow Kael | Outfitter | [Stormwall](../ports/stormwall.md) | North Atlantic | [The Iron Pact](../factions/iron_pact.md) | armor, melee, ammo | x1.08 | C:5931 | [widow_kael.md](widow_kael.md) |
| Hamish | Bladesmith | [Thornport](../ports/thornport.md) | North Atlantic | [The Free Ports](../factions/free_ports.md) | melee, ranged | x1.1 | C:5946 | [hamish_iron.md](hamish_iron.md) |
| Adaeze | Arms Dealer | [Sun Harbor](../ports/sun_harbor.md) | West Africa | [The Gold Coast Compact](../factions/gold_coast_compact.md) | melee, armor, ranged, ammo | x1.08 | C:5960 | [adaeze_arms.md](adaeze_arms.md) |
| Kofi | Leatherworker | [Palm Cove](../ports/palm_cove.md) | West Africa | [The Gold Coast Compact](../factions/gold_coast_compact.md) | armor | x1.05 | C:5976 | [kofi_leather.md](kofi_leather.md) |
| Yaa | Blade Merchant | [Pearl Shallows](../ports/pearl_shallows.md) | West Africa | [The Gold Coast Compact](../factions/gold_coast_compact.md) | melee, ranged | x1.12 | C:5989 | [yaa_sharpedge.md](yaa_sharpedge.md) |
| Master Chen | Master Smith | [Jade Port](../ports/jade_port.md) | East Indies | [The Silk Circle](../factions/silk_circle.md) | melee, armor | x1.15 | C:6003 | [chen_forge.md](chen_forge.md) |
| Devi | Armorer | [Monsoon Reach](../ports/monsoon_reach.md) | East Indies | [The Free Ports](../factions/free_ports.md) | armor, ammo | x1.1 | C:6017 | [devi_arms.md](devi_arms.md) |
| Li Wei | Mechanist | [Silk Haven](../ports/silk_haven.md) | East Indies | [The Silk Circle](../factions/silk_circle.md) | ranged, ammo, melee | x1.12 | C:6031 | [li_crossbow.md](li_crossbow.md) |
| Tane | Warrior Outfitter | [Coral Throne](../ports/coral_throne.md) | South Seas | [The Coral Crown](../factions/coral_crown.md) | melee, ranged | x1.08 | C:6046 | [tane_warrior.md](tane_warrior.md) |
| Malia | Outfitter | [Typhoon Anchorage](../ports/typhoon_anchorage.md) | South Seas | [The Coral Crown](../factions/coral_crown.md) | melee, armor, ammo | x1.1 | C:6060 | [malia_provisions.md](malia_provisions.md) |
<!-- hand-maintained:end -->

The cite is the record's first line; every field line is in the entity file. Rows are in data order.

## 3. Stock by region (computed from the catalog at the base commit)

The catalog, not the merchant, names what a shop holds: a merchant sells every item of its types whose region list holds the port's region (session.rs:3553-3609). Catalog names and base costs; the records are at the cited lines.

<!-- hand-maintained:begin -->

Melee weapons (melee_weapons)

| Item | Base cost | Regions | Cite |
|---|---|---|---|
| Cutlass | 25 | Mediterranean, North Atlantic, West Africa, East Indies, South Seas | C:3842; regions C:3849 |
| Rapier | 45 | Mediterranean, North Atlantic | C:3861; regions C:3868 |
| Boarding Axe | 35 | North Atlantic | C:3879; regions C:3886 |
| Dagger | 10 | Mediterranean, North Atlantic, West Africa, East Indies, South Seas | C:3896; regions C:3903 |
| Kris | 40 | East Indies, South Seas | C:3915; regions C:3922 |
| Ida Sword | 35 | West Africa | C:3933; regions C:3940 |
| Leiomano | 30 | South Seas | C:3950; regions C:3957 |
| Halberd | 60 | Mediterranean | C:3967; regions C:3974 |

Armour (armor)

| Item | Base cost | Regions | Cite |
|---|---|---|---|
| Leather Vest | 30 | Mediterranean, North Atlantic, West Africa | C:3757; regions C:3764 |
| Padded Coat | 35 | North Atlantic, East Indies | C:3772; regions C:3779 |
| Chain Shirt | 80 | Mediterranean, North Atlantic | C:3786; regions C:3793 |
| Brigandine | 90 | East Indies, West Africa | C:3800; regions C:3807 |
| Coconut Fiber Armor | 20 | South Seas | C:3814; regions C:3821 |
| Steel Cuirass | 200 | North Atlantic | C:3827; regions C:3834 |

Ranged weapons (ranged_weapons)

| Item | Base cost | Regions | Cite |
|---|---|---|---|
| Matchlock Pistol | 50 | Mediterranean, North Atlantic | C:3984; regions C:3993 |
| Wheellock Pistol | 120 | Mediterranean, North Atlantic | C:4002; regions C:4011 |
| Arquebus Pistol | 65 | East Indies, West Africa | C:4020; regions C:4029 |
| Hand Crossbow | 40 | Mediterranean | C:4038; regions C:4047 |
| Throwing Knife | 8 | Mediterranean, North Atlantic, West Africa, East Indies, South Seas | C:4055; regions C:4064 |
| Sling | 5 | West Africa, South Seas | C:4076; regions C:4085 |
| Blowgun | 20 | East Indies, South Seas | C:4094; regions C:4103 |
| Bolas | 15 | South Seas | C:4112; regions C:4121 |

Ammunition (ammo)

| Item | Base cost | Regions | Cite |
|---|---|---|---|
| Pistol Shot & Powder | 5 | Mediterranean, North Atlantic, East Indies, West Africa | C:5330; regions C:5335 |
| Crossbow Bolts | 4 | Mediterranean | C:5343; regions C:5348 |
| Blowgun Darts (poisoned) | 3 | East Indies, South Seas | C:5353; regions C:5358 |
<!-- hand-maintained:end -->

Items per region by type (computed from the tables above; a type with 0 in a region means a merchant of that type there sells nothing of it):

<!-- hand-maintained:begin -->
| Region | Melee | Armour | Ranged | Ammunition |
|---|---|---|---|---|
| Mediterranean | 4 | 2 | 4 | 2 |
| North Atlantic | 4 | 4 | 3 | 1 |
| West Africa | 3 | 2 | 3 | 1 |
| East Indies | 3 | 2 | 3 | 2 |
| South Seas | 4 | 1 | 4 | 1 |
<!-- hand-maintained:end -->

## 4. How buying works (code, today)

- [exists] buy_gear(gear_id) needs the captain docked, reads the port's region, takes the merchants at the port and picks the first whose stock holds the item (session.rs:3209-3227). With none at the port it answers 'Unknown merchant' (session.rs:3219-3221); with the item missing it answers that the merchant does not sell it, by raw item id (session.rs:3229-3234).
- [exists] The price is max(1, round(cost x markup)), with Python's half-to-even rounding (session.rs:3540-3542; util.rs:7-17). The same formula is also in merchant.rs:8-10, used only by its own test; two copies of one formula (DF-46).
- [exists] A purchase takes the silver first (session.rs:3235-3242), then equips: a melee weapon or armour replaces the one carried; a firearm or a mechanical weapon is stored with 0 ammunition; a thrown weapon adds 1 to the stack; ammunition is added to the carried launcher of its kind (session.rs:3611-3695).
- [exists] Buying ammunition with no launcher of that kind carried spends the silver and adds nothing, because the ammunition is stored on the weapon (session.rs:3242-3243 and 3674-3692, by source reading; tests were not run). A launcher bought again resets its ammunition to 0 (session.rs:3646-3660). See section 6 for which shops sell ammunition without a launcher.
- [exists] The sale is not recorded as a purchase for provenance or quality tier (session.rs:3204-3208).

## 5. Python versus Rust (noting only; no wiring is proposed)

- [exists] The 14 merchant records are the same in Rust and Python, field by field (portlight@9b02494 src/portlight/content/merchants.py:36-203).
- [exists] Python's merchant command lists the merchants at the port with greeting, types and markup, shows one merchant's stock with an affordability mark, and buys by merchant id, item id and quantity (src/portlight/app/cli.py:1072-1136; src/portlight/app/combat_views.py:727-785; src/portlight/engine/merchant.py:76-113). Rust has the buy only: no list, no per-merchant stock view, no quantity (session.rs:3209-3245; script.rs:422-428).
- [exists] Python keeps ammunition in counters on the captain, so ammunition bought without a launcher is kept and a launcher bought later keeps it (src/portlight/engine/merchant.py:138-144). Rust stores ammunition on the weapon (section 4). This is a model difference, noted and not logged apart from DF-46.
- [exists] Python has two more gear commands: armory, a no-markup quick-buy for ranged weapons and ammunition, and sell-gear, which pays 50 percent of a carried weapon or armour back (src/portlight/app/cli.py:2506-2560; src/portlight/app/cli.py:1143-1183). rg of the Rust crates finds neither.
- [exists] The porting plan marks engine/merchant.py deprioritised, because the Python session never calls it, and says Session::buy_gear buys one stocked item at a merchant markup for the gear scripts (docs/PORTING-PLAN.md:326).

## 6. Shop shape (computed from the stock at the base commit)

What each shop holds, as launchers (firearm, mechanical, thrown) and ammunition of the kinds that need a launcher (firearm, mechanical). Facts only.

<!-- hand-maintained:begin -->
| Merchant | Port | Items | Launchers sold | Ammunition sold | Note |
|---|---|---|---|---|---|
| Marco | Porto Novo | 10 | Matchlock Pistol, Wheellock Pistol, Hand Crossbow, Throwing Knife | Pistol Shot & Powder, Crossbow Bolts | - |
| Fatima | Al-Manar | 6 | none | none | - |
| Old Silva | Silva Bay | 12 | Matchlock Pistol, Wheellock Pistol, Hand Crossbow, Throwing Knife | Pistol Shot & Powder, Crossbow Bolts | - |
| Sgt. Gruber | Ironhaven | 12 | Matchlock Pistol, Wheellock Pistol, Throwing Knife | Pistol Shot & Powder | - |
| Widow Kael | Stormwall | 9 | none | Pistol Shot & Powder | ammunition for firearm with no firearm launcher in the shop |
| Hamish | Thornport | 7 | Matchlock Pistol, Wheellock Pistol, Throwing Knife | none | firearm launcher with no firearm ammunition in the shop |
| Adaeze | Sun Harbor | 9 | Arquebus Pistol, Throwing Knife, Sling | Pistol Shot & Powder | - |
| Kofi | Palm Cove | 2 | none | none | - |
| Yaa | Pearl Shallows | 6 | Arquebus Pistol, Throwing Knife, Sling | none | firearm launcher with no firearm ammunition in the shop |
| Master Chen | Jade Port | 5 | none | none | - |
| Devi | Monsoon Reach | 4 | none | Pistol Shot & Powder, Blowgun Darts (poisoned) | ammunition for firearm with no firearm launcher in the shop; ammunition for mechanical with no mechanical launcher in the shop |
| Li Wei | Silk Haven | 8 | Arquebus Pistol, Throwing Knife, Blowgun | Pistol Shot & Powder, Blowgun Darts (poisoned) | - |
| Tane | Coral Throne | 8 | Throwing Knife, Sling, Blowgun, Bolas | none | mechanical launcher with no mechanical ammunition in the shop |
| Malia | Typhoon Anchorage | 6 | none | Blowgun Darts (poisoned) | ammunition for mechanical with no mechanical launcher in the shop |
<!-- hand-maintained:end -->

Because each port has one merchant, a port whose shop sells ammunition without a launcher has no other shop where the launcher can be bought (section 2 shows one merchant per port).

## 7. Canon conventions

- Names are the catalog names (Sgt. Gruber, Master Chen, Widow Kael keep their shipped titles); ids appear only in cite tables and file names.
- Quote every data line with its C:n; mark every addition [invented]; put contradictions in data-fixes.md and never fix the data in a canon PR.
- A merchant is never described as a shop the player can visit, or as setting a price by personality: the sim does neither (DF-46).
- Stock tables are computed from the catalog region lists; a new item or merchant appears only through a sim PR with a parity note.

## F. Flags

- DF-42: Li Wei's hand crossbows cannot be stocked at Silk Haven. DF-43: Hamish's broadswords are not a catalog weapon. DF-44: War Chief Tane's 700% markup contradicts the 1.08 shop markup. DF-45 (declined - resolved in lore): Malia's key says provisions but she sells arms. DF-46: merchants never reach Godot. DF-47: coverage gaps.
- Building a shop, wiring merchant text, or changing a stock list is out of scope for canon; it is a design decision.

## G. Gaps in the data (computed from content.json at the base commit)

Ports in `ports[*].id` with no merchant (no record has that port as `port_id`). Facts only; canon does not say these ports have or lack anyone else.

<!-- hand-maintained:begin -->
| Port | Region | Bloc | Python source comment |
|---|---|---|---|
| [Corsair's Rest](../ports/corsairs_rest.md) | Mediterranean | [The Shadow Ports](../factions/shadow_ports.md) | portlight@9b02494 src/portlight/content/merchants.py:70 (no legitimate merchant; black market port) |
| [Iron Point](../ports/iron_point.md) | West Africa | [The Gold Coast Compact](../factions/gold_coast_compact.md) | portlight@9b02494 src/portlight/content/merchants.py:130 (mining port, no dedicated merchant) |
| [Crosswind Isle](../ports/crosswind_isle.md) | East Indies | [The Free Ports](../factions/free_ports.md) | portlight@9b02494 src/portlight/content/merchants.py:177 (no weapon merchants; trade and pirate ports) |
| [Dragon's Gate](../ports/dragons_gate.md) | East Indies | [The Silk Circle](../factions/silk_circle.md) | portlight@9b02494 src/portlight/content/merchants.py:177 (as above) |
| [Spice Narrows](../ports/spice_narrows.md) | East Indies | [The Shadow Ports](../factions/shadow_ports.md) | portlight@9b02494 src/portlight/content/merchants.py:177 (as above) |
| [Ember Isle](../ports/ember_isle.md) | South Seas | [The Coral Crown](../factions/coral_crown.md) | portlight@9b02494 src/portlight/content/merchants.py:202 (volcanic outpost, no merchant) |
<!-- hand-maintained:end -->

6 of 20 ports have no merchant. Merchants by region (computed):

<!-- hand-maintained:begin -->
| Region | Ports in the region | Merchants | Cite |
|---|---|---|---|
| Mediterranean | 4 | 3 (Marco, Fatima, Old Silva) | merchants[*].port_id |
| North Atlantic | 3 | 3 (Sgt. Gruber, Widow Kael, Hamish) | merchants[*].port_id |
| West Africa | 4 | 3 (Adaeze, Kofi, Yaa) | merchants[*].port_id |
| East Indies | 6 | 3 (Master Chen, Devi, Li Wei) | merchants[*].port_id |
| South Seas | 3 | 2 (Tane, Malia) | merchants[*].port_id |
<!-- hand-maintained:end -->

The Python comments are authorial intent in a source file, not data; they are cited as such and canon adds nothing to them.

## H. Invented filler (not in the game)

None written. This section is empty by default. If design review asks for it, it may hold at most one line per gap port, each tagged [invented], saying what a captain might plausibly meet there; no stats, prices or fees, no name that already exists in data, no file per filler, and never linked from a port file as fact.
