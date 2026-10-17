# Widow Kael (widow_kael)

Base commit: d2b544f (main, which includes phases 6A and 6B; content.json, the sim and the godot UI as cited are unchanged since 515a488). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at d2b544f. Python cites read portlight@9b02494 path:line. Legend: [exists] code or data fact with a cite | [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

Every table row below that carries a cite is [exists] (a code or data fact); rows tagged otherwise say so.

| Key | Value | Cite |
|---|---|---|
| Catalog name | Widow Kael | C:5932 |
| Title | Outfitter | C:5934 |
| Home port | [Stormwall](../ports/stormwall.md) | C:5933 |
| Bloc of the home port | [The Iron Pact](../factions/iron_pact.md) | ports/stormwall.md section 0 |
| Region (picks the stock) | North Atlantic | C:648 |
| Personality (data word) | quiet [flavour, no mechanic] | C:5935 |
| Sells (types) | armour, melee weapons, ammunition | C:5938 |
| Markup | x1.08 (+8% in Python's merchant list) | C:5943; portlight@9b02494 src/portlight/app/combat_views.py:740 |
| Items in stock at this port today | 9 (section 1) | session.rs:3553-3609 |
| Cheapest and dearest, at markup | Pistol Shot & Powder 5 silver; Steel Cuirass 216 silver | section 1 table |
| Player sees today | nothing: no gear shop in the Godot build | rg of crates/portlight-godot/src finds no reader; script.rs:422-428 |
| Not shown | name, title, description, greeting, personality, stock, prices | no reader in the crates (section 2) |
| Mechanic status | name, port, types and markup are live in the sim (script and tests only); personality, description, greeting and title are [flavour, no mechanic] | session.rs:3209-3245; content.rs:376-386 |
| Python record | same strings (compared field by field, no differences); Python's merchant command lists the greeting, types and markup, and sells | portlight@9b02494 src/portlight/content/merchants.py:85; src/portlight/app/cli.py:1072-1136 |

## 1. Grounded facts (every data line quoted) and the stock

- description: "She inherited her husband's shop when he didn't come back from the ice. She knows every item by touch." (C:5936)
- greeting (a spoken line, quoted inside the data string): "The wind's picking up. Buy what you need before it gets worse." (C:5937)
- title: "Outfitter" (C:5934)
- personality: "quiet" (C:5935)
- markup: 1.08 (C:5943)

Quotes are as in content.json; an em dash is folded to ' - ' and tagged [ascii-folded].

Stock at Stormwall, computed at the base commit: every catalog item of a type this merchant sells whose region list holds North Atlantic (session.rs:3553-3609). Price is max(1, round(catalog cost x markup)) (session.rs:3540-3542).

<!-- hand-maintained:begin -->
| Item | Type | Catalog cost | At markup | Cite (catalog record) |
|---|---|---|---|---|
| Cutlass | melee | 25 | 27 | C:3842; regions C:3849 |
| Rapier | melee | 45 | 49 | C:3861; regions C:3868 |
| Boarding Axe | melee | 35 | 38 | C:3879; regions C:3886 |
| Dagger | melee | 10 | 11 | C:3896; regions C:3903 |
| Leather Vest | armor | 30 | 32 | C:3757; regions C:3764 |
| Padded Coat | armor | 35 | 38 | C:3772; regions C:3779 |
| Chain Shirt | armor | 80 | 86 | C:3786; regions C:3793 |
| Steel Cuirass | armor | 200 | 216 | C:3827; regions C:3834 |
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

- Voice [cite]: plain and weathered; she sells against the weather - "The wind's picking up. Buy what you need before it gets worse." (C:5937).
- Manner [invented] She picks an item up, names it without looking, and puts it down in the same place.
- Setting [cite]: "She inherited her husband's shop when he didn't come back from the ice. She knows every item by touch." (C:5936). [invented] The shop keeps the husband's old hooks by the door, unsold, each with the date he last used it.
- No face, age, build or dress is set beyond the quoted data lines; canon adds none. Character plates are held.

## 4. [invented] lore

- [invented] She kept the shop open through the first winter alone and now supplies the garrison's spare kit on credit.
- [invented] Captains who have bought from her once are remembered by the size of their coat.

## 5. Hooks ([proposal] only; never a mechanic)

- [proposal] Copy hook: a weather-first greeting for a gale day, as copy only.
- Sound: ambience only. No sfx id is cited.

## 6. Links

- Home port: [Stormwall](../ports/stormwall.md); bloc [The Iron Pact](../factions/iron_pact.md)
- Merchant index: [merchants/README.md](README.md)
- [The Fortress](../landmarks/stormwall_fortress.md) - the landmark of the same port
- Crew hired in this region: [officers/north_atlantic.md](../officers/north_atlantic.md)

## 7. Flags and open questions

- DF-46 (merchants never reach Godot; no reader for their text), DF-47 (coverage gaps): see ../data-fixes.md.
- She sells ammunition but, at this port, no weapon that fires it (README section 6).
- Building a shop, or showing any merchant text, is out of scope for canon; it is a design decision.
