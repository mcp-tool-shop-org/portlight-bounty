# Yaa (yaa_sharpedge)

Base commit: d2b544f (main, which includes phases 6A and 6B; content.json, the sim and the godot UI as cited are unchanged since 515a488). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at d2b544f. Python cites read portlight@9b02494 path:line. Legend: [exists] code or data fact with a cite | [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

Every table row below that carries a cite is [exists] (a code or data fact); rows tagged otherwise say so.

| Key | Value | Cite |
|---|---|---|
| Catalog name | Yaa | C:5990 |
| Title | Blade Merchant | C:5992 |
| Home port | [Pearl Shallows](../ports/pearl_shallows.md) | C:5991 |
| Bloc of the home port | [The Gold Coast Compact](../factions/gold_coast_compact.md) | ports/pearl_shallows.md section 0 |
| Region (picks the stock) | West Africa | C:1043 |
| Personality (data word) | shrewd [flavour, no mechanic] | C:5993 |
| Sells (types) | melee weapons, ranged weapons | C:5996 |
| Markup | x1.12 (+12% in Python's merchant list) | C:6000; portlight@9b02494 src/portlight/app/combat_views.py:740 |
| Items in stock at this port today | 6 (section 1) | session.rs:3553-3609 |
| Cheapest and dearest, at markup | Sling 6 silver; Arquebus Pistol 73 silver | section 1 table |
| Player sees today | the Godot Crew desk Armoury section (SECTION_GEAR): name, title, personality, markup, description, greeting, and regional stock rows with Buy buttons (DF-46 resolved) | crew_screen.rs; logic.rs; game.rs |
| Mechanic status | name, port, title, personality, description, greeting, types, and markup are live in merchant.rs, Session::buy_gear, and the Godot Crew desk (DF-46 resolved) | merchant.rs:15-85; session.rs; logic.rs |
| Python record | same strings (compared field by field, no differences); Python's merchant command lists the greeting, types and markup, and sells | portlight@9b02494 src/portlight/content/merchants.py:132; src/portlight/app/cli.py:1072-1136 |

## 1. Grounded facts (every data line quoted) and the stock

- description: "She sources knives and slings from inland villages and sells them to sailors at the dock." (C:5994)
- greeting (a spoken line, quoted inside the data string): "Pearl Shallows has the best steel on the coast. Don't let anyone tell you otherwise." (C:5995)
- title: "Blade Merchant" (C:5992)
- personality: "shrewd" (C:5993)
- markup: 1.12 (C:6000)

Quotes are as in content.json; an em dash is folded to ' - ' and tagged [ascii-folded].

Stock at Pearl Shallows, computed at the base commit: every catalog item of a type this merchant sells whose region list holds West Africa (session.rs:3553-3609). Price is max(1, round(catalog cost x markup)) (session.rs:3540-3542).

<!-- hand-maintained:begin -->
| Item | Type | Catalog cost | At markup | Cite (catalog record) |
|---|---|---|---|---|
| Cutlass | melee | 25 | 28 | C:3842; regions C:3849 |
| Dagger | melee | 10 | 11 | C:3896; regions C:3903 |
| Ida Sword | melee | 35 | 39 | C:3933; regions C:3940 |
| Arquebus Pistol | ranged | 65 | 73 | C:4020; regions C:4029 |
| Throwing Knife | ranged | 8 | 9 | C:4055; regions C:4064 |
| Sling | ranged | 5 | 6 | C:4076; regions C:4085 |
<!-- hand-maintained:end -->

6 items. The catalog region lists decide the stock; no merchant record names an item.

## 2. Role in play (what the sim does today; no proposals here)

- [exists] Selection: the sim looks up the merchants at the docked port (content.rs:1001-1006); it takes the first merchant whose stock holds the requested item, else the first merchant (session.rs:3218-3227). Every port with a merchant has exactly one today, so the choice is moot (computed from C:5868-6075).
- [exists] Price and sale: the cost is the marked price, taken from the captain's silver, and the item is equipped (session.rs:3228-3243, 3540-3542); buying is by item id only and one at a time (the quantity passed is the literal 1 at session.rs:3243). Nothing is written for provenance or quality tier (session.rs:3204-3208).
- [exists] Reach: buy_gear is called only by the script runner's gear command and by tests (script.rs:422-428; session.rs:4816). The Godot build has no gear shop, so no player meets this merchant (rg of crates/portlight-godot/src for buy_gear and merchants finds nothing; DF-46).
- [exists] Text reach: personality, title, description and greeting have no reader in the crates; MerchantDef carries them as fields only (content.rs:376-386; the lookups at content.rs:997-1006 read id and port only). Python prints the greeting and the title in its merchant list (portlight@9b02494 src/portlight/app/combat_views.py:727-748) and the greeting above the stock (combat_views.py:751-785).
- [exists] Other gear routes in the crates: post-combat loot can equip melee, armour, ranged and ammunition drops (loot.rs:113-160); the crew desk and the harbour screens carry no shop.

## 3. Look and feel (words only; the character hold stands)

- Voice [cite]: shrewd and proud of her coast - "Pearl Shallows has the best steel on the coast. Don't let anyone tell you otherwise." (C:5995).
- Manner [invented] She argues for her stock by naming the village each piece came from.
- Setting [cite]: "She sources knives and slings from inland villages and sells them to sailors at the dock." (C:5994). [invented] A dock stall under a sail awning, with knives laid in rows on cloth and slings coiled beside them.
- No face, age, build or dress is set beyond the quoted data lines; canon adds none. Character plates are held.

## 4. [invented] lore

- [invented] Her supply runs on seasons: the inland villages send knives after harvest and slings before the dry months.
- [invented] Sailors know her by her stall number on the dock, not by her name.

## 5. Hooks ([proposal] only; never a mechanic)

- [proposal] Copy hook: a village-of-origin tag line per item, as copy only.
- Sound: ambience only. No sfx id is cited.

## 6. Links

- Home port: [Pearl Shallows](../ports/pearl_shallows.md); bloc [The Gold Coast Compact](../factions/gold_coast_compact.md)
- Merchant index: [merchants/README.md](README.md)
- [The Blessing Pool](../landmarks/blessing_pool.md) - the landmark of the same port
- [Nneka Ironfist](../style-masters/nneka_ironfist.md) - the style master at the same port
- Crew hired in this region: [officers/west_africa.md](../officers/west_africa.md)

## 7. Flags and open questions

- DF-46 (resolved: Armoury section and merchant text/stock wired on the Godot Crew desk), DF-47 (coverage gaps): see ../data-fixes.md.
- Given-name overlap: Yaa Acheampong, a broker at Iron Point (C:9198; DF-37): namesakes. Her greeting (C:5995) says "Pearl Shallows has the best steel on the coast" while the description (C:5994) has her sourcing knives and slings from inland villages.
- Building a shop, or showing any merchant text, is out of scope for canon; it is a design decision.
