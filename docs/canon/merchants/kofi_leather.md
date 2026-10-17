# Kofi (kofi_leather)

Base commit: d2b544f (main, which includes phases 6A and 6B; content.json, the sim and the godot UI as cited are unchanged since 515a488). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at d2b544f. Python cites read portlight@9b02494 path:line. Legend: [exists] code or data fact with a cite | [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

Every table row below that carries a cite is [exists] (a code or data fact); rows tagged otherwise say so.

| Key | Value | Cite |
|---|---|---|
| Catalog name | Kofi | C:5977 |
| Title | Leatherworker | C:5979 |
| Home port | [Palm Cove](../ports/palm_cove.md) | C:5978 |
| Bloc of the home port | [The Gold Coast Compact](../factions/gold_coast_compact.md) | ports/palm_cove.md section 0 |
| Region (picks the stock) | West Africa | C:885 |
| Personality (data word) | quiet [flavour, no mechanic] | C:5980 |
| Sells (types) | armour | C:5983 |
| Markup | x1.05 (+5% in Python's merchant list) | C:5986; portlight@9b02494 src/portlight/app/combat_views.py:740 |
| Items in stock at this port today | 2 (section 1) | session.rs:3553-3609 |
| Cheapest and dearest, at markup | Leather Vest 32 silver; Brigandine 94 silver | section 1 table |
| Player sees today | nothing: no gear shop in the Godot build | rg of crates/portlight-godot/src finds no reader; script.rs:422-428 |
| Not shown | name, title, description, greeting, personality, stock, prices | no reader in the crates (section 2) |
| Mechanic status | name, port, types and markup are live in the sim (script and tests only); personality, description, greeting and title are [flavour, no mechanic] | session.rs:3209-3245; content.rs:376-386 |
| Python record | same strings (compared field by field, no differences); Python's merchant command lists the greeting, types and markup, and sells | portlight@9b02494 src/portlight/content/merchants.py:120; src/portlight/app/cli.py:1072-1136 |

## 1. Grounded facts (every data line quoted) and the stock

- description: "He works crocodile hide into armor that turns blades. His fingers are scarred from the stitching awl." (C:5981)
- greeting (a spoken line, quoted inside the data string): "Good leather takes time. I have what's ready." (C:5982)
- title: "Leatherworker" (C:5979)
- personality: "quiet" (C:5980)
- markup: 1.05 (C:5986)

Quotes are as in content.json; an em dash is folded to ' - ' and tagged [ascii-folded].

Stock at Palm Cove, computed at the base commit: every catalog item of a type this merchant sells whose region list holds West Africa (session.rs:3553-3609). Price is max(1, round(catalog cost x markup)) (session.rs:3540-3542).

<!-- hand-maintained:begin -->
| Item | Type | Catalog cost | At markup | Cite (catalog record) |
|---|---|---|---|---|
| Leather Vest | armor | 30 | 32 | C:3757; regions C:3764 |
| Brigandine | armor | 90 | 94 | C:3800; regions C:3807 |
<!-- hand-maintained:end -->

2 items. The catalog region lists decide the stock; no merchant record names an item.

## 2. Role in play (what the sim does today; no proposals here)

- [exists] Selection: the sim looks up the merchants at the docked port (content.rs:1001-1006); it takes the first merchant whose stock holds the requested item, else the first merchant (session.rs:3218-3227). Every port with a merchant has exactly one today, so the choice is moot (computed from C:5868-6075).
- [exists] Price and sale: the cost is the marked price, taken from the captain's silver, and the item is equipped (session.rs:3228-3243, 3540-3542); buying is by item id only and one at a time (the quantity passed is the literal 1 at session.rs:3243). Nothing is written for provenance or quality tier (session.rs:3204-3208).
- [exists] Reach: buy_gear is called only by the script runner's gear command and by tests (script.rs:422-428; session.rs:4816). The Godot build has no gear shop, so no player meets this merchant (rg of crates/portlight-godot/src for buy_gear and merchants finds nothing; DF-46).
- [exists] Text reach: personality, title, description and greeting have no reader in the crates; MerchantDef carries them as fields only (content.rs:376-386; the lookups at content.rs:997-1006 read id and port only). Python prints the greeting and the title in its merchant list (portlight@9b02494 src/portlight/app/combat_views.py:727-748) and the greeting above the stock (combat_views.py:751-785).
- [exists] Other gear routes in the crates: post-combat loot can equip melee, armour, ranged and ammunition drops (loot.rs:113-160); the crew desk and the harbour screens carry no shop.

## 3. Look and feel (words only; the character hold stands)

- Voice [cite]: quiet and unhurried, with a craftsman's patience - "Good leather takes time. I have what's ready." (C:5982).
- Manner [invented] He shows what is finished and says plainly what is not.
- Setting [cite]: "He works crocodile hide into armor that turns blades. His fingers are scarred from the stitching awl." (C:5981). [invented] The shop smells of cured hide and wax; unfinished pieces hang stretched on frames along the wall.
- No face, age, build or dress is set beyond the quoted data lines; canon adds none. Character plates are held.

## 4. [invented] lore

- [invented] He learned the stitch from a cousin and keeps the awl that scarred him on a hook as a reminder.
- [invented] He makes no promise of a date, and captains who wait a tide for a fitting are rarely sorry.

## 5. Hooks ([proposal] only; never a mechanic)

- [proposal] Copy hook: a ready-or-not line for a stock check. Copy only.
- Sound: ambience only. No sfx id is cited.

## 6. Links

- Home port: [Palm Cove](../ports/palm_cove.md); bloc [The Gold Coast Compact](../factions/gold_coast_compact.md)
- Merchant index: [merchants/README.md](README.md)
- [The Distillery Row](../landmarks/distillery_row.md) - the landmark of the same port
- [Dockhand Pete](../companions/dockhand_pete.md) - the companion at the same port
- Crew hired in this region: [officers/west_africa.md](../officers/west_africa.md)

## 7. Flags and open questions

- DF-46 (merchants never reach Godot; no reader for their text), DF-47 (coverage gaps): see ../data-fixes.md.
- Given-name overlap: Kofi Asante (Sun Harbor), Foreman Kofi Mensah (Iron Point) and the officer name Kofi (officers/west_africa.md; DF-37, DF-38): namesakes.
- Building a shop, or showing any merchant text, is out of scope for canon; it is a design decision.
