# Marco (marco_the_blade)

Base commit: d2b544f (main, which includes phases 6A and 6B; content.json, the sim and the godot UI as cited are unchanged since 515a488). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at d2b544f. Python cites read portlight@9b02494 path:line. Legend: [exists] code or data fact with a cite | [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

Every table row below that carries a cite is [exists] (a code or data fact); rows tagged otherwise say so.

| Key | Value | Cite |
|---|---|---|
| Catalog name | Marco | C:5871 |
| Title | Master Weaponsmith | C:5873 |
| Home port | [Porto Novo](../ports/porto_novo.md) | C:5872 |
| Bloc of the home port | [The Exchange Alliance](../factions/exchange_alliance.md) | ports/porto_novo.md section 0 |
| Region (picks the stock) | Mediterranean | C:213 |
| Personality (data word) | charming [flavour, no mechanic] | C:5874 |
| Sells (types) | melee weapons, ranged weapons, ammunition | C:5877 |
| Markup | x1.1 (+10% in Python's merchant list) | C:5882; portlight@9b02494 src/portlight/app/combat_views.py:740 |
| Items in stock at this port today | 10 (section 1) | session.rs:3553-3609 |
| Cheapest and dearest, at markup | Crossbow Bolts 4 silver; Wheellock Pistol 132 silver | section 1 table |
| Player sees today | the Godot Crew desk Armoury section (SECTION_GEAR): name, title, personality, markup, description, greeting, and regional stock rows with Buy buttons (DF-46 resolved) | crew_screen.rs; logic.rs; game.rs |
| Mechanic status | name, port, title, personality, description, greeting, types, and markup are live in merchant.rs, Session::buy_gear, and the Godot Crew desk (DF-46 resolved) | merchant.rs:15-85; session.rs; logic.rs |
| Python record | same strings (compared field by field, no differences); Python's merchant command lists the greeting, types and markup, and sells | portlight@9b02494 src/portlight/content/merchants.py:38; src/portlight/app/cli.py:1072-1136 |

## 1. Grounded facts (every data line quoted) and the stock

- description: "A stocky Genoese smith with oil-stained hands and a shop full of blades hanging from the ceiling." (C:5875)
- greeting (a spoken line, quoted inside the data string): "Ah, a captain with taste! Come, see what Marco has forged this week." (C:5876)
- title: "Master Weaponsmith" (C:5873)
- personality: "charming" (C:5874)
- markup: 1.1 (C:5882)

Quotes are as in content.json; an em dash is folded to ' - ' and tagged [ascii-folded].

Stock at Porto Novo, computed at the base commit: every catalog item of a type this merchant sells whose region list holds Mediterranean (session.rs:3553-3609). Price is max(1, round(catalog cost x markup)) (session.rs:3540-3542).

<!-- hand-maintained:begin -->
| Item | Type | Catalog cost | At markup | Cite (catalog record) |
|---|---|---|---|---|
| Cutlass | melee | 25 | 28 | C:3842; regions C:3849 |
| Rapier | melee | 45 | 50 | C:3861; regions C:3868 |
| Dagger | melee | 10 | 11 | C:3896; regions C:3903 |
| Halberd | melee | 60 | 66 | C:3967; regions C:3974 |
| Matchlock Pistol | ranged | 50 | 55 | C:3984; regions C:3993 |
| Wheellock Pistol | ranged | 120 | 132 | C:4002; regions C:4011 |
| Hand Crossbow | ranged | 40 | 44 | C:4038; regions C:4047 |
| Throwing Knife | ranged | 8 | 9 | C:4055; regions C:4064 |
| Pistol Shot & Powder | ammo | 5 | 6 | C:5330; regions C:5335 |
| Crossbow Bolts | ammo | 4 | 4 | C:5343; regions C:5348 |
<!-- hand-maintained:end -->

10 items. The catalog region lists decide the stock; no merchant record names an item.

## 2. Role in play (what the sim does today; no proposals here)

- [exists] Selection: the sim looks up the merchants at the docked port (content.rs:1001-1006); it takes the first merchant whose stock holds the requested item, else the first merchant (session.rs:3218-3227). Every port with a merchant has exactly one today, so the choice is moot (computed from C:5868-6075).
- [exists] Price and sale: the cost is the marked price, taken from the captain's silver, and the item is equipped (session.rs:3228-3243, 3540-3542); buying is by item id only and one at a time (the quantity passed is the literal 1 at session.rs:3243). Nothing is written for provenance or quality tier (session.rs:3204-3208).
- [exists] Reach: buy_gear is called only by the script runner's gear command and by tests (script.rs:422-428; session.rs:4816). The Godot build has no gear shop, so no player meets this merchant (rg of crates/portlight-godot/src for buy_gear and merchants finds nothing; DF-46).
- [exists] Text reach: personality, title, description and greeting have no reader in the crates; MerchantDef carries them as fields only (content.rs:376-386; the lookups at content.rs:997-1006 read id and port only). Python prints the greeting and the title in its merchant list (portlight@9b02494 src/portlight/app/combat_views.py:727-748) and the greeting above the stock (combat_views.py:751-785).
- [exists] Other gear routes in the crates: post-combat loot can equip melee, armour, ranged and ammunition drops (loot.rs:113-160); the crew desk and the harbour screens carry no shop.

## 3. Look and feel (words only; the character hold stands)

- Voice [cite]: warm and salesmanlike; he sells by flattering the buyer - "Ah, a captain with taste!" and a promise of what was forged this week (C:5876).
- Manner [invented] He lets a customer pick a blade up before he names a price, and names the price only after the weight has settled in the hand.
- Setting [cite]: "A stocky Genoese smith with oil-stained hands and a shop full of blades hanging from the ceiling." (C:5875). [invented] The ceiling blades are hung by length, shortest nearest the door, so the shop reads like a rack of oars.
- No face, age, build or dress is set beyond the quoted data lines; canon adds none. Character plates are held.

## 4. [invented] lore

- [invented] He keeps a ledger of every blade by the week it was hung, and will tell a captain which one has waited longest.
- [invented] The guard of the Grain Exchange buys from him each quarter, so a Marco blade is a common sight on the hall steps.

## 5. Hooks ([proposal] only; never a mechanic)

- [proposal] Journal or shop-copy hook: a line about the longest-hung blade in the shop, to be written as copy only if a shop screen is ever briefed.
- Sound: ambience only. No sfx id is cited.

## 6. Links

- Home port: [Porto Novo](../ports/porto_novo.md); bloc [The Exchange Alliance](../factions/exchange_alliance.md)
- Merchant index: [merchants/README.md](README.md)
- [The Grain Exchange](../landmarks/grain_exchange.md) - the landmark of the same port
- [Maestro Luciano](../style-masters/maestro_luciano.md) - the style master at the same port
- [Old Vasquez](../skill-trainers/old_vasquez.md) - the skill trainer at the same port
- Crew hired in this region: [officers/mediterranean.md](../officers/mediterranean.md)

## 7. Flags and open questions

- DF-46 (resolved: Armoury section and merchant text/stock wired on the Godot Crew desk), DF-47 (coverage gaps): see ../data-fixes.md.
- Given-name overlap: Marco is also an officer name in the Mediterranean pool (officers/mediterranean.md; DF-38). A hired officer is a namesake, not this smith.
- Building a shop, or showing any merchant text, is out of scope for canon; it is a design decision.
