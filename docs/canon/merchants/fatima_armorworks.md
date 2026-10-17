# Fatima (fatima_armorworks)

Base commit: d2b544f (main, which includes phases 6A and 6B; content.json, the sim and the godot UI as cited are unchanged since 515a488). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at d2b544f. Python cites read portlight@9b02494 path:line. Legend: [exists] code or data fact with a cite | [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

Every table row below that carries a cite is [exists] (a code or data fact); rows tagged otherwise say so.

| Key | Value | Cite |
|---|---|---|
| Catalog name | Fatima | C:5886 |
| Title | Armorer | C:5888 |
| Home port | [Al-Manar](../ports/al_manar.md) | C:5887 |
| Bloc of the home port | [The Exchange Alliance](../factions/exchange_alliance.md) | ports/al_manar.md section 0 |
| Region (picks the stock) | Mediterranean | C:294 |
| Personality (data word) | shrewd [flavour, no mechanic] | C:5889 |
| Sells (types) | armour, melee weapons | C:5892 |
| Markup | x1.15 (+14% in Python's merchant list) | C:5896; portlight@9b02494 src/portlight/app/combat_views.py:740 |
| Items in stock at this port today | 6 (section 1) | session.rs:3553-3609 |
| Cheapest and dearest, at markup | Dagger 12 silver; Chain Shirt 92 silver | section 1 table |
| Player sees today | nothing: no gear shop in the Godot build | rg of crates/portlight-godot/src finds no reader; script.rs:422-428 |
| Not shown | name, title, description, greeting, personality, stock, prices | no reader in the crates (section 2) |
| Mechanic status | name, port, types and markup are live in the sim (script and tests only); personality, description, greeting and title are [flavour, no mechanic] | session.rs:3209-3245; content.rs:376-386 |
| Python record | same strings (compared field by field, no differences); Python's merchant command lists the greeting, types and markup, and sells | portlight@9b02494 src/portlight/content/merchants.py:49; src/portlight/app/cli.py:1072-1136 |

## 1. Grounded facts (every data line quoted) and the stock

- description: "She runs the finest armor workshop south of the strait. Her chain is tight and her prices tighter." (C:5890)
- greeting (a spoken line, quoted inside the data string): "You want protection? Protection costs. But so does a funeral." (C:5891)
- title: "Armorer" (C:5888)
- personality: "shrewd" (C:5889)
- markup: 1.15 (C:5896)

Quotes are as in content.json; an em dash is folded to ' - ' and tagged [ascii-folded].

Stock at Al-Manar, computed at the base commit: every catalog item of a type this merchant sells whose region list holds Mediterranean (session.rs:3553-3609). Price is max(1, round(catalog cost x markup)) (session.rs:3540-3542).

<!-- hand-maintained:begin -->
| Item | Type | Catalog cost | At markup | Cite (catalog record) |
|---|---|---|---|---|
| Cutlass | melee | 25 | 29 | C:3842; regions C:3849 |
| Rapier | melee | 45 | 52 | C:3861; regions C:3868 |
| Dagger | melee | 10 | 12 | C:3896; regions C:3903 |
| Halberd | melee | 60 | 69 | C:3967; regions C:3974 |
| Leather Vest | armor | 30 | 34 | C:3757; regions C:3764 |
| Chain Shirt | armor | 80 | 92 | C:3786; regions C:3793 |
<!-- hand-maintained:end -->

6 items. The catalog region lists decide the stock; no merchant record names an item.

## 2. Role in play (what the sim does today; no proposals here)

- [exists] Selection: the sim looks up the merchants at the docked port (content.rs:1001-1006); it takes the first merchant whose stock holds the requested item, else the first merchant (session.rs:3218-3227). Every port with a merchant has exactly one today, so the choice is moot (computed from C:5868-6075).
- [exists] Price and sale: the cost is the marked price, taken from the captain's silver, and the item is equipped (session.rs:3228-3243, 3540-3542); buying is by item id only and one at a time (the quantity passed is the literal 1 at session.rs:3243). Nothing is written for provenance or quality tier (session.rs:3204-3208).
- [exists] Reach: buy_gear is called only by the script runner's gear command and by tests (script.rs:422-428; session.rs:4816). The Godot build has no gear shop, so no player meets this merchant (rg of crates/portlight-godot/src for buy_gear and merchants finds nothing; DF-46).
- [exists] Text reach: personality, title, description and greeting have no reader in the crates; MerchantDef carries them as fields only (content.rs:376-386; the lookups at content.rs:997-1006 read id and port only). Python prints the greeting and the title in its merchant list (portlight@9b02494 src/portlight/app/combat_views.py:727-748) and the greeting above the stock (combat_views.py:751-785).
- [exists] Other gear routes in the crates: post-combat loot can equip melee, armour, ranged and ammunition drops (loot.rs:113-160); the crew desk and the harbour screens carry no shop.

## 3. Look and feel (words only; the character hold stands)

- Voice [cite]: shrewd and dry; she prices protection against the cost of a funeral - "Protection costs. But so does a funeral." (C:5891).
- Manner [invented] She quotes the price of a fitting first and the price of the mail second, and never lowers either.
- Setting [cite]: "She runs the finest armor workshop south of the strait. Her chain is tight and her prices tighter." (C:5890). [invented] The workshop is a long room of hanging mail, tuned by ear: she taps a shirt and tells the buyer where it will fail.
- No face, age, build or dress is set beyond the quoted data lines; canon adds none. Character plates are held.

## 4. [invented] lore

- [invented] She trained under no guild and her chain is stamped with a small mark of her own, which captains call her receipt.
- [invented] The Bazaar stalls send her their guards for refits, and she charges them the same as a captain.

## 5. Hooks ([proposal] only; never a mechanic)

- [proposal] Copy hook: a short refusal line for a buyer who haggles, in her dry register. Copy only.
- Sound: ambience only. No sfx id is cited.

## 6. Links

- Home port: [Al-Manar](../ports/al_manar.md); bloc [The Exchange Alliance](../factions/exchange_alliance.md)
- Merchant index: [merchants/README.md](README.md)
- [The Spice Bazaar](../landmarks/spice_bazaar.md) - the landmark of the same port
- [Coin Mariam](../companions/coin_mariam.md) - the companion at the same port
- [Senora Vega](../style-masters/senora_vega.md) - the style master at the same port
- Crew hired in this region: [officers/mediterranean.md](../officers/mediterranean.md)

## 7. Flags and open questions

- DF-46 (merchants never reach Godot; no reader for their text), DF-47 (coverage gaps): see ../data-fixes.md.
- Given-name overlap: Fatima is also an officer name in the Mediterranean pool (officers/mediterranean.md; DF-38).
- Building a shop, or showing any merchant text, is out of scope for canon; it is a design decision.
