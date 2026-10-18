# Bladesmith Chen (chen_forge)

Base commit: d2b544f (main, which includes phases 6A and 6B; content.json, the sim and the godot UI as cited are unchanged since 515a488). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at d2b544f. Python cites read portlight@9b02494 path:line. Legend: [exists] code or data fact with a cite | [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

Every table row below that carries a cite is [exists] (a code or data fact); rows tagged otherwise say so.

| Key | Value | Cite |
|---|---|---|
| Catalog name | Bladesmith Chen (DF-37 resolved) | C:6004 |
| Title | Master Smith | C:6006 |
| Home port | [Jade Port](../ports/jade_port.md) | C:6005 |
| Bloc of the home port | [The Silk Circle](../factions/silk_circle.md) | ports/jade_port.md section 0 |
| Region (picks the stock) | East Indies | C:1122 |
| Personality (data word) | quiet [flavour, no mechanic] | C:6007 |
| Sells (types) | melee weapons, armour | C:6010 |
| Markup | x1.15 (+14% in Python's merchant list) | C:6014; portlight@9b02494 src/portlight/app/combat_views.py:740 |
| Items in stock at this port today | 5 (section 1) | session.rs:3553-3609 |
| Cheapest and dearest, at markup | Dagger 12 silver; Brigandine 103 silver | section 1 table |
| Player sees today | the Godot Crew desk Armoury section (SECTION_GEAR): name, title, personality, markup, description, greeting, and regional stock rows with Buy buttons (DF-46 resolved) | crew_screen.rs; logic.rs; game.rs |
| Mechanic status | name, port, title, personality, description, greeting, types, and markup are live in merchant.rs, Session::buy_gear, and the Godot Crew desk (DF-46 resolved) | merchant.rs:15-85; session.rs; logic.rs |
| Python record | same strings (compared field by field, no differences); Python's merchant command lists the greeting, types and markup, and sells | portlight@9b02494 src/portlight/content/merchants.py:145; src/portlight/app/cli.py:1072-1136 |

## 1. Grounded facts (every data line quoted) and the stock

- description: "Third generation bladesmith. His kris daggers are considered sacred objects in three kingdoms." (C:6008)
- greeting (a spoken line, quoted inside the data string): "A captain who values craftsmanship. Please, examine my work." (C:6009)
- title: "Master Smith" (C:6006)
- personality: "quiet" (C:6007)
- markup: 1.15 (C:6014)

Quotes are as in content.json; an em dash is folded to ' - ' and tagged [ascii-folded].

Stock at Jade Port, computed at the base commit: every catalog item of a type this merchant sells whose region list holds East Indies (session.rs:3553-3609). Price is max(1, round(catalog cost x markup)) (session.rs:3540-3542).

<!-- hand-maintained:begin -->
| Item | Type | Catalog cost | At markup | Cite (catalog record) |
|---|---|---|---|---|
| Cutlass | melee | 25 | 29 | C:3842; regions C:3849 |
| Dagger | melee | 10 | 12 | C:3896; regions C:3903 |
| Kris | melee | 40 | 46 | C:3915; regions C:3922 |
| Padded Coat | armor | 35 | 40 | C:3772; regions C:3779 |
| Brigandine | armor | 90 | 103 | C:3800; regions C:3807 |
<!-- hand-maintained:end -->

5 items. The catalog region lists decide the stock; no merchant record names an item.

## 2. Role in play (what the sim does today; no proposals here)

- [exists] Selection: the sim looks up the merchants at the docked port (content.rs:1001-1006); it takes the first merchant whose stock holds the requested item, else the first merchant (session.rs:3218-3227). Every port with a merchant has exactly one today, so the choice is moot (computed from C:5868-6075).
- [exists] Price and sale: the cost is the marked price, taken from the captain's silver, and the item is equipped (session.rs:3228-3243, 3540-3542); buying is by item id only and one at a time (the quantity passed is the literal 1 at session.rs:3243). Nothing is written for provenance or quality tier (session.rs:3204-3208).
- [exists] Reach: buy_gear is called only by the script runner's gear command and by tests (script.rs:422-428; session.rs:4816). The Godot build has no gear shop, so no player meets this merchant (rg of crates/portlight-godot/src for buy_gear and merchants finds nothing; DF-46).
- [exists] Text reach: personality, title, description and greeting have no reader in the crates; MerchantDef carries them as fields only (content.rs:376-386; the lookups at content.rs:997-1006 read id and port only). Python prints the greeting and the title in its merchant list (portlight@9b02494 src/portlight/app/combat_views.py:727-748) and the greeting above the stock (combat_views.py:751-785).
- [exists] Other gear routes in the crates: post-combat loot can equip melee, armour, ranged and ammunition drops (loot.rs:113-160); the crew desk and the harbour screens carry no shop.

## 3. Look and feel (words only; the character hold stands)

- Voice [cite]: quiet, formal and courteous - "A captain who values craftsmanship. Please, examine my work." (C:6009).
- Manner [invented] He hands a blade over hilt first and waits, and says nothing until it has been examined.
- Setting [cite]: "Third generation bladesmith. His kris daggers are considered sacred objects in three kingdoms." (C:6008). [invented] A forge with an altar shelf for the daggers he will not sell, kept apart from the stock.
- No face, age, build or dress is set beyond the quoted data lines; canon adds none. Character plates are held.

## 4. [invented] lore

- [invented] The family has kept the forge for three generations and counts the third by the apprentice who will be the fourth.
- [invented] He makes only the blades he is satisfied by, and the shop is rarely fully stocked.

## 5. Hooks ([proposal] only; never a mechanic)

- [proposal] Copy hook: a hilt-first handing line. Copy only.
- Sound: ambience only. No sfx id is cited.

## 6. Links

- Home port: [Jade Port](../ports/jade_port.md); bloc [The Silk Circle](../factions/silk_circle.md)
- Merchant index: [merchants/README.md](README.md)
- [The Porcelain Quarter](../landmarks/porcelain_quarter.md) - the landmark of the same port
- [Pak Reza](../style-masters/pak_reza.md) - the style master at the same port
- Crew hired in this region: [officers/east_indies.md](../officers/east_indies.md)

## 7. Flags and open questions

- DF-46 (resolved: Armoury section and merchant text/stock wired on the Godot Crew desk), DF-47 (coverage gaps): see ../data-fixes.md.
- DF-37 (resolved): same-port namesake, Kiln Master Chen Bai (C:9517). The shop title Master Smith and the skill level Master are different things (data-fixes resolved in lore).
- Building a shop, or showing any merchant text, is out of scope for canon; it is a design decision.
