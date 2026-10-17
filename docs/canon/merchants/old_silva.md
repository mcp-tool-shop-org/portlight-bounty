# Old Silva (old_silva)

Base commit: d2b544f (main, which includes phases 6A and 6B; content.json, the sim and the godot UI as cited are unchanged since 515a488). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at d2b544f. Python cites read portlight@9b02494 path:line. Legend: [exists] code or data fact with a cite | [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

Every table row below that carries a cite is [exists] (a code or data fact); rows tagged otherwise say so.

| Key | Value | Cite |
|---|---|---|
| Catalog name | Old Silva | C:5900 |
| Title | Ship Outfitter | C:5902 |
| Home port | [Silva Bay](../ports/silva_bay.md) | C:5901 |
| Bloc of the home port | [The Exchange Alliance](../factions/exchange_alliance.md) | ports/silva_bay.md section 0 |
| Region (picks the stock) | Mediterranean | C:381 |
| Personality (data word) | quiet [flavour, no mechanic] | C:5903 |
| Sells (types) | melee weapons, ranged weapons, ammunition, armour | C:5906 |
| Markup | x1.05 (+5% in Python's merchant list) | C:5912; portlight@9b02494 src/portlight/app/combat_views.py:740 |
| Items in stock at this port today | 12 (section 1) | session.rs:3553-3609 |
| Cheapest and dearest, at markup | Crossbow Bolts 4 silver; Wheellock Pistol 126 silver | section 1 table |
| Player sees today | nothing: no gear shop in the Godot build | rg of crates/portlight-godot/src finds no reader; script.rs:422-428 |
| Not shown | name, title, description, greeting, personality, stock, prices | no reader in the crates (section 2) |
| Mechanic status | name, port, types and markup are live in the sim (script and tests only); personality, description, greeting and title are [flavour, no mechanic] | session.rs:3209-3245; content.rs:376-386 |
| Python record | same strings (compared field by field, no differences); Python's merchant command lists the greeting, types and markup, and sells | portlight@9b02494 src/portlight/content/merchants.py:60; src/portlight/app/cli.py:1072-1136 |

## 1. Grounded facts (every data line quoted) and the stock

- description: "A retired shipwright who sells what sailors need. His shop smells of tar and salt." (C:5904)
- greeting (a spoken line, quoted inside the data string): "What do you need. I have it or I don't." (C:5905)
- title: "Ship Outfitter" (C:5902)
- personality: "quiet" (C:5903)
- markup: 1.05 (C:5912)

Quotes are as in content.json; an em dash is folded to ' - ' and tagged [ascii-folded].

Stock at Silva Bay, computed at the base commit: every catalog item of a type this merchant sells whose region list holds Mediterranean (session.rs:3553-3609). Price is max(1, round(catalog cost x markup)) (session.rs:3540-3542).

<!-- hand-maintained:begin -->
| Item | Type | Catalog cost | At markup | Cite (catalog record) |
|---|---|---|---|---|
| Cutlass | melee | 25 | 26 | C:3842; regions C:3849 |
| Rapier | melee | 45 | 47 | C:3861; regions C:3868 |
| Dagger | melee | 10 | 10 | C:3896; regions C:3903 |
| Halberd | melee | 60 | 63 | C:3967; regions C:3974 |
| Leather Vest | armor | 30 | 32 | C:3757; regions C:3764 |
| Chain Shirt | armor | 80 | 84 | C:3786; regions C:3793 |
| Matchlock Pistol | ranged | 50 | 52 | C:3984; regions C:3993 |
| Wheellock Pistol | ranged | 120 | 126 | C:4002; regions C:4011 |
| Hand Crossbow | ranged | 40 | 42 | C:4038; regions C:4047 |
| Throwing Knife | ranged | 8 | 8 | C:4055; regions C:4064 |
| Pistol Shot & Powder | ammo | 5 | 5 | C:5330; regions C:5335 |
| Crossbow Bolts | ammo | 4 | 4 | C:5343; regions C:5348 |
<!-- hand-maintained:end -->

12 items. The catalog region lists decide the stock; no merchant record names an item.

## 2. Role in play (what the sim does today; no proposals here)

- [exists] Selection: the sim looks up the merchants at the docked port (content.rs:1001-1006); it takes the first merchant whose stock holds the requested item, else the first merchant (session.rs:3218-3227). Every port with a merchant has exactly one today, so the choice is moot (computed from C:5868-6075).
- [exists] Price and sale: the cost is the marked price, taken from the captain's silver, and the item is equipped (session.rs:3228-3243, 3540-3542); buying is by item id only and one at a time (the quantity passed is the literal 1 at session.rs:3243). Nothing is written for provenance or quality tier (session.rs:3204-3208).
- [exists] Reach: buy_gear is called only by the script runner's gear command and by tests (script.rs:422-428; session.rs:4816). The Godot build has no gear shop, so no player meets this merchant (rg of crates/portlight-godot/src for buy_gear and merchants finds nothing; DF-46).
- [exists] Text reach: personality, title, description and greeting have no reader in the crates; MerchantDef carries them as fields only (content.rs:376-386; the lookups at content.rs:997-1006 read id and port only). Python prints the greeting and the title in its merchant list (portlight@9b02494 src/portlight/app/combat_views.py:727-748) and the greeting above the stock (combat_views.py:751-785).
- [exists] Other gear routes in the crates: post-combat loot can equip melee, armour, ranged and ammunition drops (loot.rs:113-160); the crew desk and the harbour screens carry no shop.

## 3. Look and feel (words only; the character hold stands)

- Voice [cite]: quiet and clipped, almost curt - "What do you need. I have it or I don't." (C:5905).
- Manner [invented] He answers in as few words as the question allows and goes back to his tar bucket.
- Setting [cite]: "A retired shipwright who sells what sailors need. His shop smells of tar and salt." (C:5904). [invented] Coils of rope and tins of tar share the shelves with the blades and mail; the weapons are the smallest part of the shop.
- No face, age, build or dress is set beyond the quoted data lines; canon adds none. Character plates are held.

## 4. [invented] lore

- [invented] He built hulls at the yard for decades and sells weapons only because sailors ask for them as they buy rope.
- [invented] The yard hands call his shop the back shelf, and a young shipwright gets a first cutlass from it.

## 5. Hooks ([proposal] only; never a mechanic)

- [proposal] Copy hook: a one-line stock check in his clipped voice. Copy only.
- Sound: ambience only. No sfx id is cited.

## 6. Links

- Home port: [Silva Bay](../ports/silva_bay.md); bloc [The Exchange Alliance](../factions/exchange_alliance.md)
- Merchant index: [merchants/README.md](README.md)
- [The Master Shipyard](../landmarks/master_shipyard.md) - the landmark of the same port
- [The Silva Bay Smithy](../skill-trainers/silva_bay_smiths.md) - the skill trainer at the same port
- Crew hired in this region: [officers/mediterranean.md](../officers/mediterranean.md)

## 7. Flags and open questions

- DF-46 (merchants never reach Godot; no reader for their text), DF-47 (coverage gaps): see ../data-fixes.md.
- The name Silva appears three times at Silva Bay (the port, this outfitter, the Smithy): unrelated uses of a place name (data-fixes resolved in lore).
- Building a shop, or showing any merchant text, is out of scope for canon; it is a design decision.
