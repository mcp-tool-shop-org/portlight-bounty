# Hamish (hamish_iron)

Base commit: d2b544f (main, which includes phases 6A and 6B; content.json, the sim and the godot UI as cited are unchanged since 515a488). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at d2b544f. Python cites read portlight@9b02494 path:line. Legend: [exists] code or data fact with a cite | [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

Every table row below that carries a cite is [exists] (a code or data fact); rows tagged otherwise say so.

| Key | Value | Cite |
|---|---|---|
| Catalog name | Hamish | C:5947 |
| Title | Bladesmith | C:5949 |
| Home port | [Thornport](../ports/thornport.md) | C:5948 |
| Bloc of the home port | [The Free Ports](../factions/free_ports.md) | ports/thornport.md section 0 |
| Region (picks the stock) | North Atlantic | C:727 |
| Personality (data word) | gruff [flavour, no mechanic] | C:5950 |
| Sells (types) | melee weapons, ranged weapons | C:5953 |
| Markup | x1.1 (+10% in Python's merchant list) | C:5957; portlight@9b02494 src/portlight/app/combat_views.py:740 |
| Items in stock at this port today | 7 (section 1) | session.rs:3553-3609 |
| Cheapest and dearest, at markup | Throwing Knife 9 silver; Wheellock Pistol 132 silver | section 1 table |
| Player sees today | nothing: no gear shop in the Godot build | rg of crates/portlight-godot/src finds no reader; script.rs:422-428 |
| Not shown | name, title, description, greeting, personality, stock, prices | no reader in the crates (section 2) |
| Mechanic status | name, port, types and markup are live in the sim (script and tests only); personality, description, greeting and title are [flavour, no mechanic] | session.rs:3209-3245; content.rs:376-386 |
| Python record | same strings except description (`COPY_OVERRIDES`: 'forges axes and blades' vs Python's 'forges axes and broadswords', DF-43 resolved); Python's merchant command lists the greeting, types and markup, and sells | portlight@9b02494 src/portlight/content/merchants.py:96; src/portlight/app/cli.py:1072-1136 |

## 1. Grounded facts (every data line quoted) and the stock

- description: "A Highland smith who forges axes and blades in a workshop built into the cliff face." (C:5951; `COPY_OVERRIDES` in tools/extract_content.py, DF-43 resolved)
- greeting (a spoken line, quoted inside the data string): "If it's sharp and it swings, I made it. If it's dull, someone else made it." (C:5952)
- title: "Bladesmith" (C:5949)
- personality: "gruff" (C:5950)
- markup: 1.1 (C:5957)

Quotes are as in content.json; an em dash is folded to ' - ' and tagged [ascii-folded].

Stock at Thornport, computed at the base commit: every catalog item of a type this merchant sells whose region list holds North Atlantic (session.rs:3553-3609). Price is max(1, round(catalog cost x markup)) (session.rs:3540-3542).

<!-- hand-maintained:begin -->
| Item | Type | Catalog cost | At markup | Cite (catalog record) |
|---|---|---|---|---|
| Cutlass | melee | 25 | 28 | C:3842; regions C:3849 |
| Rapier | melee | 45 | 50 | C:3861; regions C:3868 |
| Boarding Axe | melee | 35 | 38 | C:3879; regions C:3886 |
| Dagger | melee | 10 | 11 | C:3896; regions C:3903 |
| Matchlock Pistol | ranged | 50 | 55 | C:3984; regions C:3993 |
| Wheellock Pistol | ranged | 120 | 132 | C:4002; regions C:4011 |
| Throwing Knife | ranged | 8 | 9 | C:4055; regions C:4064 |
<!-- hand-maintained:end -->

7 items. The catalog region lists decide the stock; no merchant record names an item.

## 2. Role in play (what the sim does today; no proposals here)

- [exists] Selection: the sim looks up the merchants at the docked port (content.rs:1001-1006); it takes the first merchant whose stock holds the requested item, else the first merchant (session.rs:3218-3227). Every port with a merchant has exactly one today, so the choice is moot (computed from C:5868-6075).
- [exists] Price and sale: the cost is the marked price, taken from the captain's silver, and the item is equipped (session.rs:3228-3243, 3540-3542); buying is by item id only and one at a time (the quantity passed is the literal 1 at session.rs:3243). Nothing is written for provenance or quality tier (session.rs:3204-3208).
- [exists] Reach: buy_gear is called only by the script runner's gear command and by tests (script.rs:422-428; session.rs:4816). The Godot build has no gear shop, so no player meets this merchant (rg of crates/portlight-godot/src for buy_gear and merchants finds nothing; DF-46).
- [exists] Text reach: personality, title, description and greeting have no reader in the crates; MerchantDef carries them as fields only (content.rs:376-386; the lookups at content.rs:997-1006 read id and port only). Python prints the greeting and the title in its merchant list (portlight@9b02494 src/portlight/app/combat_views.py:727-748) and the greeting above the stock (combat_views.py:751-785).
- [exists] Other gear routes in the crates: post-combat loot can equip melee, armour, ranged and ammunition drops (loot.rs:113-160); the crew desk and the harbour screens carry no shop.

## 3. Look and feel (words only; the character hold stands)

- Voice [cite]: gruff and boastful about his trade - "If it's sharp and it swings, I made it. If it's dull, someone else made it." (C:5952).
- Manner [invented] He tests an edge on a hanging rope before he names the price, and the rope is always the same rope.
- Setting [cite]: "A Highland smith who forges axes and blades in a workshop built into the cliff face." (C:5951). [invented] The shop is the cliff itself: racks cut into the rock, with the forge at the back warming the whole wall.
- No face, age, build or dress is set beyond the quoted data lines; canon adds none. Character plates are held.

## 4. [invented] lore

- [invented] He came to the whaling town for harpoon heads and stayed for the boarding trade.
- [invented] The whalers call his edges honest, which in Thornport is a fair price on its own.

## 5. Hooks ([proposal] only; never a mechanic)

- [proposal] Copy hook: a rope-test line for the greeting. Copy only.
- Sound: ambience only. No sfx id is cited.

## 6. Links

- Home port: [Thornport](../ports/thornport.md); bloc [The Free Ports](../factions/free_ports.md)
- Merchant index: [merchants/README.md](README.md)
- [The Whale Arch](../landmarks/whale_arch.md) - the landmark of the same port
- [Old Henry](../companions/old_henry.md) - the companion at the same port
- [Jarl Solveig](../style-masters/jarl_solveig.md) - the style master at the same port
- Crew hired in this region: [officers/north_atlantic.md](../officers/north_atlantic.md)

## 7. Flags and open questions

- DF-46 (merchants never reach Godot; no reader for their text), DF-47 (coverage gaps): see ../data-fixes.md.
- DF-43 (resolved): his description now names axes and blades via `COPY_OVERRIDES` in tools/extract_content.py. Given-name overlap: Red Hamish teaches at Ironhaven (style-masters/red_hamish.md): two people.
- Building a shop, or showing any merchant text, is out of scope for canon; it is a design decision.
