# Nneka Ironfist (nneka_ironfist)

Base commit: af3d105 (merged main, which includes phase 6A; content.json, the sim and the godot UI as cited are unchanged since 515a488). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at af3d105. Python cites read portlight@9b02494 path:line. Legend: [exists] code or data fact with a cite | [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

Every table row below that carries a cite is [exists] (a code or data fact); rows tagged otherwise say so.

| Key | Value | Cite |
|---|---|---|
| Catalog name | Nneka Ironfist | C:6214 |
| Teaches | Dambe ([style table](README.md#3-styles-5)) | C:6215 |
| Home port | [Pearl Shallows](../ports/pearl_shallows.md) | C:6216 |
| Bloc of the home port | [The Gold Coast Compact](../factions/gold_coast_compact.md) | ports/pearl_shallows.md section 0 |
| Region | West Africa | C:4214 |
| Training cost | 60 silver | C:4244 |
| Training time | 6 days | C:4245 |
| Styles needed first | 0 | C:4246 |
| Body parts the style needs (gate is dead, DF-41) | leg | C:4247 |
| Player sees today | the style row at this port (name, silver, days, styles needed, known yes or no), a confirm line, and a notice after training; never the master | logic.rs:2028-2062; logic.rs:1884; game.rs:5693-5714 |
| Not shown | master name, description, dialog | no reader in the crates (section 2) |
| Mechanic status | the master record is [flavour, no mechanic]; the real training gate is the style's own port list (DF-39) | training.rs:9-52; content.rs:1027-1032 |
| Python record | same strings (compared field by field, no differences); Python's train command prints the master name and dialog | portlight@9b02494 src/portlight/content/fighting_styles.py:322; src/portlight/app/cli.py:2426-2455 |

## 1. Grounded facts (every data line quoted)

- description: "A woman who earned her name in the fighting circles of the Gold Coast. She teaches with bruises, not words." (C:6217)
- dialog (a spoken line, quoted inside the data string): "You think fighting is about your arms? Your legs carry the fight. Everything begins from the ground." (C:6218)
- style, description: "The fighting art of the Hausa people. The lead fist is the spear, the rear hand is the shield, and the legs finish every fight." (C:4215)
- style, historical note (real-world, shipped data): "Practiced across West Africa since at least the 15th century. Dambe bouts were a central part of harvest festivals and market days." (C:4216)
- style, special action: Leg Sweep (leg_sweep): counter, stamina 2, beats thrust, loses to slash, dodge, damage +1, cooldown 3 (C:4224)
- style, special action flavour: "A low spinning sweep - the opponent's legs are taken from under them. They crash to the deck." (C:4237)

Quotes are as in content.json; an em dash is folded to ' - ' and a tilde n to n, and both are tagged [ascii-folded].

## 2. Role in play (what the sim does today; no proposals here)

- [exists] Training gate: the sim never reads this master. A captain may learn Dambe at any port in the style's own list, which today is Sun Harbor and Pearl Shallows (C:4240); the master's port is one of the two, so the data agrees but is redundant (training.rs:9-52, the check at 22-24; content.rs:1027-1032; training.rs:73-79; the only caller is a test, training.rs:101).
- [exists] Cost and time: 60 silver, then 6 days of the calendar, the same price at both ports; the master changes neither (training.rs:55-65).
- [exists] Injury gate: the sim checks the style's body parts only when it is handed a list of injured parts, and the session hands it an empty list, so an injured captain is never refused (session.rs:3331-3351, the empty list at 3337; DF-41).
- [exists] After training: the style is stored in the captain's learned styles (training.rs:55-65). No Rust code sets the active style, so a learned style gives no passive and no special action in a game started in Rust (model.rs:523; DF-41).
- [exists] What the Godot desk prints, built in code not authored [exists]: the row 'Dambe  60 silver  6 days  needs 0 styles  known no' (logic.rs:2041-2045); the confirm line 'Train Dambe for 60 silver. The calendar will advance 6 days.' (logic.rs:1884); the notice 'Learned dambe. The calendar advanced N days.' with the raw id (game.rs:5708; DF-40).
- [exists] Text reach: the master's name, description and dialog have no reader in the crates (rg for style_master and masters_at in crates/portlight-godot/src finds nothing; the sim has the one test caller). Python prints the name and the dialog in its training view (cli.py:2426-2455; combat_views.py:359-412).

## 3. Look and feel (words only; the character hold stands)

- Voice [cite]: hard and plain: arms are the least of it, the legs carry the fight - "You think fighting is about your arms? Your legs carry the fight. Everything begins from the ground." (C:6218).
- Manner [invented] She says a thing once and lets the ground say it again.
- Setting [cite]: "A woman who earned her name in the fighting circles of the Gold Coast. She teaches with bruises, not words." (C:6217). [invented] The ring is a patch of packed sand behind the pearl graders' sheds, marked with a rope.
- No face, age, build or dress is set beyond the quoted data lines; canon adds none. Character plates are held.

## 4. [invented] lore

- [invented] She won her name in the fighting circles of the Gold Coast (C:6217) and carried it to the pearl divers' port, where bruises are an old and accepted form of tuition.
- [invented] The divers' elder tolerates her because the pupils learn to take a fall as well as give one.

## 5. Hooks ([proposal] only; never a mechanic)

- [proposal] Copy hook: a line contrasting her 'bruises, not words' with the words she does say; flavour only.
- Sound: ambience only. No sfx id is cited.

## 6. Links

- Home port: [Pearl Shallows](../ports/pearl_shallows.md); bloc [The Gold Coast Compact](../factions/gold_coast_compact.md)
- Style index: [style-masters/README.md](README.md)
- Other master of the same style: [Kolo the Fist](kolo_the_fist.md) (Sun Harbor)
- [The Blessing Pool](../landmarks/blessing_pool.md) - the landmark of the same port
- Crew hired in this region: [officers/west_africa.md](../officers/west_africa.md)

## 7. Flags and open questions

- DF-39 (masters and master text unread), DF-41 (the style injury gate is dead and no path equips a style): see ../data-fixes.md.
- Her description says she teaches 'with bruises, not words' (C:6217) and her dialog is a speech (C:6218); canon reads the speech as the one she gives on the first day (resolved in lore).
- Wiring master text into the desk, or a style equip control, is out of scope for canon; it is a design decision.
