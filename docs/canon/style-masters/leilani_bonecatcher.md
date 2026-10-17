# Leilani Bonecatcher (leilani_bonecatcher)

Base commit: af3d105 (merged main, which includes phase 6A; content.json, the sim and the godot UI as cited are unchanged since 515a488). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at af3d105. Python cites read portlight@9b02494 path:line. Legend: [exists] code or data fact with a cite | [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

Every table row below that carries a cite is [exists] (a code or data fact); rows tagged otherwise say so.

| Key | Value | Cite |
|---|---|---|
| Catalog name | Leilani Bonecatcher | C:6246 |
| Teaches | Lua ([style table](README.md#3-styles-5)) | C:6247 |
| Home port | [Typhoon Anchorage](../ports/typhoon_anchorage.md) | C:6248 |
| Bloc of the home port | [The Coral Crown](../factions/coral_crown.md) | ports/typhoon_anchorage.md section 0 |
| Region | South Seas | C:4295 |
| Training cost | 150 silver | C:4325 |
| Training time | 10 days | C:4326 |
| Styles needed first | 1 | C:4327 |
| Body parts the style needs (gate is dead, DF-41) | hand, arm | C:4328 |
| Player sees today | the style row at this port (name, silver, days, styles needed, known yes or no), a confirm line, and a notice after training; never the master | logic.rs:2028-2062; logic.rs:1884; game.rs:5693-5714 |
| Not shown | master name, description, dialog | no reader in the crates (section 2) |
| Mechanic status | the master record is [flavour, no mechanic]; the real training gate is the style's own port list (DF-39) | training.rs:9-52; content.rs:1027-1032 |
| Python record | same strings (compared field by field, no differences); Python's train command prints the master name and dialog | portlight@9b02494 src/portlight/content/fighting_styles.py:379; src/portlight/app/cli.py:2426-2455 |

## 1. Grounded facts (every data line quoted)

- description: "A young woman with an old name. She learned Lua from her grandfather and teaches it to those the sea has tested." (C:6249)
- dialog (a spoken line, quoted inside the data string): "The bones tell you everything. How strong they are, where they bend, where they break. Listen to the bones." (C:6250)
- style, description: "The bone-breaking art of Polynesia. Joint locks, pressure points, and techniques designed not to wound but to permanently disable." (C:4296)
- style, historical note (real-world, shipped data): "Hawaiian Lua was restricted to the warrior class. Its techniques were so dangerous that training was conducted in secret, at night, under oath." (C:4297)
- style, special action: Joint Lock (joint_lock): utility, stamina 3, beats parry, loses to thrust, slash, damage +0, cooldown 3 (C:4305)
- style, special action flavour: "You seize the opponent's arm and twist. Bone grinds against joint. They can only writhe and dodge." (C:4318)

Quotes are as in content.json; an em dash is folded to ' - ' and a tilde n to n, and both are tagged [ascii-folded].

## 2. Role in play (what the sim does today; no proposals here)

- [exists] Training gate: the sim never reads this master. A captain may learn Lua at any port in the style's own list, which today is Coral Throne and Typhoon Anchorage (C:4321); the master's port is one of the two, so the data agrees but is redundant (training.rs:9-52, the check at 22-24; content.rs:1027-1032; training.rs:73-79; the only caller is a test, training.rs:101).
- [exists] Cost and time: 150 silver, then 10 days of the calendar, the same price at both ports; the master changes neither (training.rs:55-65).
- [exists] Injury gate: the sim checks the style's body parts only when it is handed a list of injured parts, and the session hands it an empty list, so an injured captain is never refused (session.rs:3331-3351, the empty list at 3337; DF-41).
- [exists] After training: the style is stored in the captain's learned styles (training.rs:55-65). No Rust code sets the active style, so a learned style gives no passive and no special action in a game started in Rust (model.rs:523; DF-41).
- [exists] What the Godot desk prints, built in code not authored [exists]: the row 'Lua  150 silver  10 days  needs 1 styles  known no' (logic.rs:2041-2045); the confirm line 'Train Lua for 150 silver. The calendar will advance 10 days.' (logic.rs:1884); the notice 'Learned lua. The calendar advanced N days.' with the raw id (game.rs:5708; DF-40).
- [exists] Text reach: the master's name, description and dialog have no reader in the crates (rg for style_master and masters_at in crates/portlight-godot/src finds nothing; the sim has the one test caller). Python prints the name and the dialog in its training view (cli.py:2426-2455; combat_views.py:359-412).

## 3. Look and feel (words only; the character hold stands)

- Voice [cite]: quiet and attentive: listen to the bones - "The bones tell you everything. How strong they are, where they bend, where they break. Listen to the bones." (C:6250).
- Manner [invented] She teaches with her hands on the pupil's arm and says what she feels there.
- Setting [cite]: "A young woman with an old name. She learned Lua from her grandfather and teaches it to those the sea has tested." (C:6249). [invented] She teaches on the lee side of the harbour wall, out of the typhoon wind.
- No face, age, build or dress is set beyond the quoted data lines; canon adds none. Character plates are held.

## 4. [invented] lore

- [invented] She learned Lua from her grandfather, who gave her the old name when she set her first bone; she teaches those the sea has tested (C:6249).
- [invented] Patches, the surgeon who hires out of the same port, sends her the patients whose bones have set wrong.

## 5. Hooks ([proposal] only; never a mechanic)

- [proposal] Sound hook: ambience only - the harbour wall under typhoon wind. The Storm Wall landmark is at this port. No cue id is cited.
- Sound: ambience only. No sfx id is cited.

## 6. Links

- Home port: [Typhoon Anchorage](../ports/typhoon_anchorage.md); bloc [The Coral Crown](../factions/coral_crown.md)
- Style index: [style-masters/README.md](README.md)
- Other master of the same style: [Kahoku the Elder](kahoku_the_elder.md) (Coral Throne)
- [The Storm Wall](../landmarks/storm_wall.md) - the landmark of the same port
- [Patches](../companions/patches.md) - the companion at the same port
- [Kai the Shaper](../skill-trainers/typhoon_smith.md) - the skill trainer at the same port
- Crew hired in this region: [officers/south_seas.md](../officers/south_seas.md)

## 7. Flags and open questions

- DF-39 (masters and master text unread), DF-41 (the style injury gate is dead and no path equips a style): see ../data-fixes.md.
- Given-name overlap: an officer name pool entry Leilani (C:6143) is a namesake (DF-38, declined, resolved in lore). Real-world words in data: Lua, Hawaiian (C:4296-4297).
- Wiring master text into the desk, or a style equip control, is out of scope for canon; it is a design decision.
