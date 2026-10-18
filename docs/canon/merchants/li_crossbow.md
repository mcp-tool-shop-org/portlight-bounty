# Mechanist Li Wei (li_crossbow)

Base commit: d2b544f (main, which includes phases 6A and 6B; content.json, the sim and the godot UI as cited are unchanged since 515a488). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at d2b544f. Python cites read portlight@9b02494 path:line. Legend: [exists] code or data fact with a cite | [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

Every table row below that carries a cite is [exists] (a code or data fact); rows tagged otherwise say so.

| Key | Value | Cite |
|---|---|---|
| Catalog name | Mechanist Li Wei (DF-37 resolved) | C:6032 |
| Title | Mechanist | C:6034 |
| Home port | [Silk Haven](../ports/silk_haven.md) | C:6033 |
| Bloc of the home port | [The Silk Circle](../factions/silk_circle.md) | ports/silk_haven.md section 0 |
| Region (picks the stock) | East Indies | C:1298 |
| Personality (data word) | shrewd [flavour, no mechanic] | C:6035 |
| Sells (types) | ranged weapons, ammunition, melee weapons | C:6038 |
| Markup | x1.12 (+12% in Python's merchant list) | C:6043; portlight@9b02494 src/portlight/app/combat_views.py:740 |
| Items in stock at this port today | 8 (section 1) | session.rs:3553-3609 |
| Cheapest and dearest, at markup | Blowgun Darts (poisoned) 3 silver; Arquebus Pistol 73 silver | section 1 table |
| Player sees today | the Godot Crew desk Armoury section (SECTION_GEAR): name, title, personality, markup, description, greeting, and regional stock rows with Buy buttons (DF-46 resolved) | crew_screen.rs; logic.rs; game.rs |
| Mechanic status | name, port, title, personality, description, greeting, types, and markup are live in merchant.rs, Session::buy_gear, and the Godot Crew desk (DF-46 resolved) | merchant.rs:15-85; session.rs; logic.rs |
| Python record | same strings except description (`COPY_OVERRIDES`: 'fittings for blowguns and arquebus pistols' vs Python's 'hand crossbows', DF-42 resolved); Python's merchant command lists the greeting, types and markup, and sells | portlight@9b02494 src/portlight/content/merchants.py:167; src/portlight/app/cli.py:1072-1136 |

## 1. Grounded facts (every data line quoted) and the stock

- description: "A craftsman who builds fittings for blowguns and arquebus pistols from bamboo and horn. Quiet weapons for quiet work." (C:6036; `COPY_OVERRIDES` in tools/extract_content.py, DF-42 resolved)
- greeting (a spoken line, quoted inside the data string): "Silk Haven values discretion. So do my weapons." (C:6037)
- title: "Mechanist" (C:6034)
- personality: "shrewd" (C:6035)
- markup: 1.12 (C:6043)

Quotes are as in content.json; an em dash is folded to ' - ' and tagged [ascii-folded].

Stock at Silk Haven, computed at the base commit: every catalog item of a type this merchant sells whose region list holds East Indies (session.rs:3553-3609). Price is max(1, round(catalog cost x markup)) (session.rs:3540-3542).

<!-- hand-maintained:begin -->
| Item | Type | Catalog cost | At markup | Cite (catalog record) |
|---|---|---|---|---|
| Cutlass | melee | 25 | 28 | C:3842; regions C:3849 |
| Dagger | melee | 10 | 11 | C:3896; regions C:3903 |
| Kris | melee | 40 | 45 | C:3915; regions C:3922 |
| Arquebus Pistol | ranged | 65 | 73 | C:4020; regions C:4029 |
| Throwing Knife | ranged | 8 | 9 | C:4055; regions C:4064 |
| Blowgun | ranged | 20 | 22 | C:4094; regions C:4103 |
| Pistol Shot & Powder | ammo | 5 | 6 | C:5330; regions C:5335 |
| Blowgun Darts (poisoned) | ammo | 3 | 3 | C:5353; regions C:5358 |
<!-- hand-maintained:end -->

8 items. The catalog region lists decide the stock; no merchant record names an item.

## 2. Role in play (what the sim does today; no proposals here)

- [exists] Selection: the sim looks up the merchants at the docked port (content.rs:1001-1006); it takes the first merchant whose stock holds the requested item, else the first merchant (session.rs:3218-3227). Every port with a merchant has exactly one today, so the choice is moot (computed from C:5868-6075).
- [exists] Price and sale: the cost is the marked price, taken from the captain's silver, and the item is equipped (session.rs:3228-3243, 3540-3542); buying is by item id only and one at a time (the quantity passed is the literal 1 at session.rs:3243). Nothing is written for provenance or quality tier (session.rs:3204-3208).
- [exists] Reach: buy_gear is called only by the script runner's gear command and by tests (script.rs:422-428; session.rs:4816). The Godot build has no gear shop, so no player meets this merchant (rg of crates/portlight-godot/src for buy_gear and merchants finds nothing; DF-46).
- [exists] Text reach: personality, title, description and greeting have no reader in the crates; MerchantDef carries them as fields only (content.rs:376-386; the lookups at content.rs:997-1006 read id and port only). Python prints the greeting and the title in its merchant list (portlight@9b02494 src/portlight/app/combat_views.py:727-748) and the greeting above the stock (combat_views.py:751-785).
- [exists] Other gear routes in the crates: post-combat loot can equip melee, armour, ranged and ammunition drops (loot.rs:113-160); the crew desk and the harbour screens carry no shop.

## 3. Look and feel (words only; the character hold stands)

- Voice [cite]: shrewd and discreet - "Silk Haven values discretion. So do my weapons." (C:6037).
- Manner [invented] He lowers his voice when he names a price and does not say to whom a piece was sold.
- Setting [cite]: "A craftsman who builds fittings for blowguns and arquebus pistols from bamboo and horn. Quiet weapons for quiet work." (C:6036). [invented] The bench is a small, tidy shop of bamboo and horn offcuts, with finished pieces under cloth.
- No face, age, build or dress is set beyond the quoted data lines; canon adds none. Character plates are held.

## 4. [invented] lore

- [invented] He claims no school; he says each weapon is designed for one quiet purpose and he does not ask which.
- [invented] The Weavers' Guild buys nothing from him, and he considers that a compliment.

## 5. Hooks ([proposal] only; never a mechanic)

- [proposal] Copy hook: a discretion line for the greeting. Copy only.
- Sound: ambience only. No sfx id is cited.

## 6. Links

- Home port: [Silk Haven](../ports/silk_haven.md); bloc [The Silk Circle](../factions/silk_circle.md)
- Merchant index: [merchants/README.md](README.md)
- [The Loom Quarter](../landmarks/loom_quarter.md) - the landmark of the same port
- Crew hired in this region: [officers/east_indies.md](../officers/east_indies.md)

## 7. Flags and open questions

- DF-46 (resolved: Armoury section and merchant text/stock wired on the Godot Crew desk), DF-47 (coverage gaps): see ../data-fixes.md.
- DF-42 (resolved): his description now names fittings for blowguns and arquebus pistols via `COPY_OVERRIDES` in tools/extract_content.py. Same full name: Harbor Director Li Wei at Jade Port (C:9495; DF-37): two people.
- Building a shop, or showing any merchant text, is out of scope for canon; it is a design decision.
