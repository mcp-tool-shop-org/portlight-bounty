# Armorer Devi (devi_arms)

Base commit: d2b544f (main, which includes phases 6A and 6B; content.json, the sim and the godot UI as cited are unchanged since 515a488). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at d2b544f. Python cites read portlight@9b02494 path:line. Legend: [exists] code or data fact with a cite | [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

Every table row below that carries a cite is [exists] (a code or data fact); rows tagged otherwise say so.

| Key | Value | Cite |
|---|---|---|
| Catalog name | Armorer Devi (DF-37 resolved) | C:6018 |
| Title | Armorer | C:6020 |
| Home port | [Monsoon Reach](../ports/monsoon_reach.md) | C:6019 |
| Bloc of the home port | [The Free Ports](../factions/free_ports.md) | ports/monsoon_reach.md section 0 |
| Region (picks the stock) | East Indies | C:1209 |
| Personality (data word) | charming [flavour, no mechanic] | C:6021 |
| Sells (types) | armour, ammunition | C:6024 |
| Markup | x1.1 (+10% in Python's merchant list) | C:6028; portlight@9b02494 src/portlight/app/combat_views.py:740 |
| Items in stock at this port today | 4 (section 1) | session.rs:3553-3609 |
| Cheapest and dearest, at markup | Blowgun Darts (poisoned) 3 silver; Brigandine 99 silver | section 1 table |
| Player sees today | the Godot Crew desk Armoury section (SECTION_GEAR): name, title, personality, markup, description, greeting, and regional stock rows with Buy buttons (DF-46 resolved) | crew_screen.rs; logic.rs; game.rs |
| Mechanic status | name, port, title, personality, description, greeting, types, and markup are live in merchant.rs, Session::buy_gear, and the Godot Crew desk (DF-46 resolved) | merchant.rs:15-85; session.rs; logic.rs |
| Python record | same strings (compared field by field, no differences); Python's merchant command lists the greeting, types and markup, and sells | portlight@9b02494 src/portlight/content/merchants.py:156; src/portlight/app/cli.py:1072-1136 |

## 1. Grounded facts (every data line quoted) and the stock

- description: "She imports brigandine plates from the mainland and fits them in her harbor-side workshop." (C:6022)
- greeting (a spoken line, quoted inside the data string): "Monsoon season is coming. You'll want something between you and the sea." (C:6023)
- title: "Armorer" (C:6020)
- personality: "charming" (C:6021)
- markup: 1.1 (C:6028)

Quotes are as in content.json; an em dash is folded to ' - ' and tagged [ascii-folded].

Stock at Monsoon Reach, computed at the base commit: every catalog item of a type this merchant sells whose region list holds East Indies (session.rs:3553-3609). Price is max(1, round(catalog cost x markup)) (session.rs:3540-3542).

<!-- hand-maintained:begin -->
| Item | Type | Catalog cost | At markup | Cite (catalog record) |
|---|---|---|---|---|
| Padded Coat | armor | 35 | 38 | C:3772; regions C:3779 |
| Brigandine | armor | 90 | 99 | C:3800; regions C:3807 |
| Pistol Shot & Powder | ammo | 5 | 6 | C:5330; regions C:5335 |
| Blowgun Darts (poisoned) | ammo | 3 | 3 | C:5353; regions C:5358 |
<!-- hand-maintained:end -->

4 items. The catalog region lists decide the stock; no merchant record names an item.

## 2. Role in play (what the sim does today; no proposals here)

- [exists] Selection: the sim looks up the merchants at the docked port (content.rs:1001-1006); it takes the first merchant whose stock holds the requested item, else the first merchant (session.rs:3218-3227). Every port with a merchant has exactly one today, so the choice is moot (computed from C:5868-6075).
- [exists] Price and sale: the cost is the marked price, taken from the captain's silver, and the item is equipped (session.rs:3228-3243, 3540-3542); buying is by item id only and one at a time (the quantity passed is the literal 1 at session.rs:3243). Nothing is written for provenance or quality tier (session.rs:3204-3208).
- [exists] Reach: buy_gear is called only by the script runner's gear command and by tests (script.rs:422-428; session.rs:4816). The Godot build has no gear shop, so no player meets this merchant (rg of crates/portlight-godot/src for buy_gear and merchants finds nothing; DF-46).
- [exists] Text reach: personality, title, description and greeting have no reader in the crates; MerchantDef carries them as fields only (content.rs:376-386; the lookups at content.rs:997-1006 read id and port only). Python prints the greeting and the title in its merchant list (portlight@9b02494 src/portlight/app/combat_views.py:727-748) and the greeting above the stock (combat_views.py:751-785).
- [exists] Other gear routes in the crates: post-combat loot can equip melee, armour, ranged and ammunition drops (loot.rs:113-160); the crew desk and the harbour screens carry no shop.

## 3. Look and feel (words only; the character hold stands)

- Voice [cite]: charming and weather-wise - "Monsoon season is coming. You'll want something between you and the sea." (C:6023).
- Manner [invented] She fits each plate to the buyer on the spot, and talks about the coming rains the whole time.
- Setting [cite]: "She imports brigandine plates from the mainland and fits them in her harbor-side workshop." (C:6022). [invented] The workshop faces the water, with plates stacked on racks by the open door to dry in the wind.
- No face, age, build or dress is set beyond the quoted data lines; canon adds none. Character plates are held.

## 4. [invented] lore

- [invented] She buys plates in bulk before the rains, when the mainland carriers are cheapest, and sells through the season.
- [invented] The Wind Temple monks consult her on which plates rust fastest in the wet.

## 5. Hooks ([proposal] only; never a mechanic)

- [proposal] Copy hook: a season-keyed greeting, as copy only.
- Sound: ambience only. No sfx id is cited.

## 6. Links

- Home port: [Monsoon Reach](../ports/monsoon_reach.md); bloc [The Free Ports](../factions/free_ports.md)
- Merchant index: [merchants/README.md](README.md)
- [The Wind Temple](../landmarks/wind_temple.md) - the landmark of the same port
- [Dewi Ayu](../style-masters/dewi_ayu.md) - the style master at the same port
- [Rajan the Tinker](../skill-trainers/monsoon_tinker.md) - the skill trainer at the same port
- Crew hired in this region: [officers/east_indies.md](../officers/east_indies.md)

## 7. Flags and open questions

- DF-46 (resolved: Armoury section and merchant text/stock wired on the Godot Crew desk), DF-47 (coverage gaps): see ../data-fixes.md.
- DF-37 (resolved): same-port namesake, Shipwright Devi (C:9773). She sells ammunition but, at this port, no weapon that fires it (README section 6).
- Building a shop, or showing any merchant text, is out of scope for canon; it is a design decision.
