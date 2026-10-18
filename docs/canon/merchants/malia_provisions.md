# Malia (malia_provisions)

Base commit: d2b544f (main, which includes phases 6A and 6B; content.json, the sim and the godot UI as cited are unchanged since 515a488). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at d2b544f. Python cites read portlight@9b02494 path:line. Legend: [exists] code or data fact with a cite | [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

Every table row below that carries a cite is [exists] (a code or data fact); rows tagged otherwise say so.

| Key | Value | Cite |
|---|---|---|
| Catalog name | Malia | C:6061 |
| Title | Outfitter | C:6063 |
| Home port | [Typhoon Anchorage](../ports/typhoon_anchorage.md) | C:6062 |
| Bloc of the home port | [The Coral Crown](../factions/coral_crown.md) | ports/typhoon_anchorage.md section 0 |
| Region (picks the stock) | South Seas | C:1753 |
| Personality (data word) | charming [flavour, no mechanic] | C:6064 |
| Sells (types) | melee weapons, armour, ammunition | C:6067 |
| Markup | x1.1 (+10% in Python's merchant list) | C:6072; portlight@9b02494 src/portlight/app/combat_views.py:740 |
| Items in stock at this port today | 6 (section 1) | session.rs:3553-3609 |
| Cheapest and dearest, at markup | Blowgun Darts (poisoned) 3 silver; Kris 44 silver | section 1 table |
| Player sees today | the Godot Crew desk Armoury section (SECTION_GEAR): name, title, personality, markup, description, greeting, and regional stock rows with Buy buttons (DF-46 resolved) | crew_screen.rs; logic.rs; game.rs |
| Mechanic status | name, port, title, personality, description, greeting, types, and markup are live in merchant.rs, Session::buy_gear, and the Godot Crew desk (DF-46 resolved) | merchant.rs:15-85; session.rs; logic.rs |
| Python record | same strings (compared field by field, no differences); Python's merchant command lists the greeting, types and markup, and sells | portlight@9b02494 src/portlight/content/merchants.py:192; src/portlight/app/cli.py:1072-1136 |

## 1. Grounded facts (every data line quoted) and the stock

- description: "She runs the only proper shop between the reefs. If she doesn't have it, you don't need it." (C:6065)
- greeting (a spoken line, quoted inside the data string): "Typhoon Anchorage! Where the brave come to buy and the foolish come to sell." (C:6066)
- title: "Outfitter" (C:6063)
- personality: "charming" (C:6064)
- markup: 1.1 (C:6072)

Quotes are as in content.json; an em dash is folded to ' - ' and tagged [ascii-folded].

Stock at Typhoon Anchorage, computed at the base commit: every catalog item of a type this merchant sells whose region list holds South Seas (session.rs:3553-3609). Price is max(1, round(catalog cost x markup)) (session.rs:3540-3542).

<!-- hand-maintained:begin -->
| Item | Type | Catalog cost | At markup | Cite (catalog record) |
|---|---|---|---|---|
| Cutlass | melee | 25 | 28 | C:3842; regions C:3849 |
| Dagger | melee | 10 | 11 | C:3896; regions C:3903 |
| Kris | melee | 40 | 44 | C:3915; regions C:3922 |
| Leiomano | melee | 30 | 33 | C:3950; regions C:3957 |
| Coconut Fiber Armor | armor | 20 | 22 | C:3814; regions C:3821 |
| Blowgun Darts (poisoned) | ammo | 3 | 3 | C:5353; regions C:5358 |
<!-- hand-maintained:end -->

6 items. The catalog region lists decide the stock; no merchant record names an item.

## 2. Role in play (what the sim does today; no proposals here)

- [exists] Selection: the sim looks up the merchants at the docked port (content.rs:1001-1006); it takes the first merchant whose stock holds the requested item, else the first merchant (session.rs:3218-3227). Every port with a merchant has exactly one today, so the choice is moot (computed from C:5868-6075).
- [exists] Price and sale: the cost is the marked price, taken from the captain's silver, and the item is equipped (session.rs:3228-3243, 3540-3542); buying is by item id only and one at a time (the quantity passed is the literal 1 at session.rs:3243). Nothing is written for provenance or quality tier (session.rs:3204-3208).
- [exists] Reach: buy_gear is called only by the script runner's gear command and by tests (script.rs:422-428; session.rs:4816). The Godot build has no gear shop, so no player meets this merchant (rg of crates/portlight-godot/src for buy_gear and merchants finds nothing; DF-46).
- [exists] Text reach: personality, title, description and greeting have no reader in the crates; MerchantDef carries them as fields only (content.rs:376-386; the lookups at content.rs:997-1006 read id and port only). Python prints the greeting and the title in its merchant list (portlight@9b02494 src/portlight/app/combat_views.py:727-748) and the greeting above the stock (combat_views.py:751-785).
- [exists] Other gear routes in the crates: post-combat loot can equip melee, armour, ranged and ammunition drops (loot.rs:113-160); the crew desk and the harbour screens carry no shop.

## 3. Look and feel (words only; the character hold stands)

- Voice [cite]: charming and boastful - "Typhoon Anchorage! Where the brave come to buy and the foolish come to sell." (C:6066).
- Manner [invented] She greets by naming the port aloud, as if it were part of the stock.
- Setting [cite]: "She runs the only proper shop between the reefs. If she doesn't have it, you don't need it." (C:6065). [invented] A shop of salvaged timber from the breakwater, with stock racked in the hulls of old ship's boats.
- No face, age, build or dress is set beyond the quoted data lines; canon adds none. Character plates are held.

## 4. [invented] lore

- [invented] She says the shop is the only one because no other has survived a typhoon season, and she has survived all of them.
- [invented] Foolish sellers, in her phrase, are captains who sell their old gear back to her at a loss; she never corrects them.

## 5. Hooks ([proposal] only; never a mechanic)

- [proposal] Copy hook: a port-name greeting per day of the typhoon season, as copy only.
- Sound: ambience only. No sfx id is cited.

## 6. Links

- Home port: [Typhoon Anchorage](../ports/typhoon_anchorage.md); bloc [The Coral Crown](../factions/coral_crown.md)
- Merchant index: [merchants/README.md](README.md)
- [The Storm Wall](../landmarks/storm_wall.md) - the landmark of the same port
- [Patches](../companions/patches.md) - the companion at the same port
- [Leilani Bonecatcher](../style-masters/leilani_bonecatcher.md) - the style master at the same port
- [Kai the Shaper](../skill-trainers/typhoon_smith.md) - the skill trainer at the same port
- Crew hired in this region: [officers/south_seas.md](../officers/south_seas.md)

## 7. Flags and open questions

- DF-46 (resolved: Armoury section and merchant text/stock wired on the Godot Crew desk), DF-47 (coverage gaps): see ../data-fixes.md.
- DF-45 (declined - resolved in lore): the data key says provisions but she sells arms and armour; the key is never shown. Provisions are bought at the crew desk (session.rs:759).
- Building a shop, or showing any merchant text, is out of scope for canon; it is a design decision.
