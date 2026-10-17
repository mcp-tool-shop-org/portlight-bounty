# Adaeze (adaeze_arms)

Base commit: d2b544f (main, which includes phases 6A and 6B; content.json, the sim and the godot UI as cited are unchanged since 515a488). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at d2b544f. Python cites read portlight@9b02494 path:line. Legend: [exists] code or data fact with a cite | [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

Every table row below that carries a cite is [exists] (a code or data fact); rows tagged otherwise say so.

| Key | Value | Cite |
|---|---|---|
| Catalog name | Adaeze | C:5961 |
| Title | Arms Dealer | C:5963 |
| Home port | [Sun Harbor](../ports/sun_harbor.md) | C:5962 |
| Bloc of the home port | [The Gold Coast Compact](../factions/gold_coast_compact.md) | ports/sun_harbor.md section 0 |
| Region (picks the stock) | West Africa | C:806 |
| Personality (data word) | charming [flavour, no mechanic] | C:5964 |
| Sells (types) | melee weapons, armour, ranged weapons, ammunition | C:5967 |
| Markup | x1.08 (+8% in Python's merchant list) | C:5973; portlight@9b02494 src/portlight/app/combat_views.py:740 |
| Items in stock at this port today | 9 (section 1) | session.rs:3553-3609 |
| Cheapest and dearest, at markup | Pistol Shot & Powder 5 silver; Brigandine 97 silver | section 1 table |
| Player sees today | nothing: no gear shop in the Godot build | rg of crates/portlight-godot/src finds no reader; script.rs:422-428 |
| Not shown | name, title, description, greeting, personality, stock, prices | no reader in the crates (section 2) |
| Mechanic status | name, port, types and markup are live in the sim (script and tests only); personality, description, greeting and title are [flavour, no mechanic] | session.rs:3209-3245; content.rs:376-386 |
| Python record | same strings (compared field by field, no differences); Python's merchant command lists the greeting, types and markup, and sells | portlight@9b02494 src/portlight/content/merchants.py:109; src/portlight/app/cli.py:1072-1136 |

## 1. Grounded facts (every data line quoted) and the stock

- description: "A trader's daughter who turned her father's cloth stall into a weapons bazaar. She haggles like she breathes." (C:5965)
- greeting (a spoken line, quoted inside the data string): "Welcome, captain! Everything you see is the best in the harbor. I should know - I tested it myself." (C:5966)
- title: "Arms Dealer" (C:5963)
- personality: "charming" (C:5964)
- markup: 1.08 (C:5973)

Quotes are as in content.json; an em dash is folded to ' - ' and tagged [ascii-folded].

Stock at Sun Harbor, computed at the base commit: every catalog item of a type this merchant sells whose region list holds West Africa (session.rs:3553-3609). Price is max(1, round(catalog cost x markup)) (session.rs:3540-3542).

<!-- hand-maintained:begin -->
| Item | Type | Catalog cost | At markup | Cite (catalog record) |
|---|---|---|---|---|
| Cutlass | melee | 25 | 27 | C:3842; regions C:3849 |
| Dagger | melee | 10 | 11 | C:3896; regions C:3903 |
| Ida Sword | melee | 35 | 38 | C:3933; regions C:3940 |
| Leather Vest | armor | 30 | 32 | C:3757; regions C:3764 |
| Brigandine | armor | 90 | 97 | C:3800; regions C:3807 |
| Arquebus Pistol | ranged | 65 | 70 | C:4020; regions C:4029 |
| Throwing Knife | ranged | 8 | 9 | C:4055; regions C:4064 |
| Sling | ranged | 5 | 5 | C:4076; regions C:4085 |
| Pistol Shot & Powder | ammo | 5 | 5 | C:5330; regions C:5335 |
<!-- hand-maintained:end -->

9 items. The catalog region lists decide the stock; no merchant record names an item.

## 2. Role in play (what the sim does today; no proposals here)

- [exists] Selection: the sim looks up the merchants at the docked port (content.rs:1001-1006); it takes the first merchant whose stock holds the requested item, else the first merchant (session.rs:3218-3227). Every port with a merchant has exactly one today, so the choice is moot (computed from C:5868-6075).
- [exists] Price and sale: the cost is the marked price, taken from the captain's silver, and the item is equipped (session.rs:3228-3243, 3540-3542); buying is by item id only and one at a time (the quantity passed is the literal 1 at session.rs:3243). Nothing is written for provenance or quality tier (session.rs:3204-3208).
- [exists] Reach: buy_gear is called only by the script runner's gear command and by tests (script.rs:422-428; session.rs:4816). The Godot build has no gear shop, so no player meets this merchant (rg of crates/portlight-godot/src for buy_gear and merchants finds nothing; DF-46).
- [exists] Text reach: personality, title, description and greeting have no reader in the crates; MerchantDef carries them as fields only (content.rs:376-386; the lookups at content.rs:997-1006 read id and port only). Python prints the greeting and the title in its merchant list (portlight@9b02494 src/portlight/app/combat_views.py:727-748) and the greeting above the stock (combat_views.py:751-785).
- [exists] Other gear routes in the crates: post-combat loot can equip melee, armour, ranged and ammunition drops (loot.rs:113-160); the crew desk and the harbour screens carry no shop.

## 3. Look and feel (words only; the character hold stands)

- Voice [cite]: bright and persuasive; she sells by testimony - "Everything you see is the best in the harbor. I should know - I tested it myself." [ascii-folded] (C:5966).
- Manner [invented] She demonstrates before she prices, and the demonstration is always on a bale of cloth.
- Setting [cite]: "A trader's daughter who turned her father's cloth stall into a weapons bazaar. She haggles like she breathes." (C:5965). [invented] The cloth stall frame is still there, with blades laid out on the old cloth tables under striped awnings.
- No face, age, build or dress is set beyond the quoted data lines; canon adds none. Character plates are held.

## 4. [invented] lore

- [invented] She still sells a little cloth at the back, so the bazaar keeps calling her the trader's daughter.
- [invented] She tests each blade on a rolled bale, and a bale with a clean cut is hung up as a sign.

## 5. Hooks ([proposal] only; never a mechanic)

- [proposal] Copy hook: a testimony line per item class, as copy only.
- Sound: ambience only. No sfx id is cited.

## 6. Links

- Home port: [Sun Harbor](../ports/sun_harbor.md); bloc [The Gold Coast Compact](../factions/gold_coast_compact.md)
- Merchant index: [merchants/README.md](README.md)
- [The Cotton Steps](../landmarks/cotton_steps.md) - the landmark of the same port
- [Dr. Amara](../companions/dr_amara.md) - the companion at the same port
- [Kolo the Fist](../style-masters/kolo_the_fist.md) - the style master at the same port
- Crew hired in this region: [officers/west_africa.md](../officers/west_africa.md)

## 7. Flags and open questions

- DF-46 (merchants never reach Godot; no reader for their text), DF-47 (coverage gaps): see ../data-fixes.md.
- Given-name overlap: Adaeze is also an officer name in the West Africa pool (officers/west_africa.md; DF-38).
- Building a shop, or showing any merchant text, is out of scope for canon; it is a design decision.
