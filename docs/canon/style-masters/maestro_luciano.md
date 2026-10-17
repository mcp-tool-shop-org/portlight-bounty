# Maestro Luciano (maestro_luciano)

Base commit: af3d105 (merged main, which includes phase 6A; content.json, the sim and the godot UI as cited are unchanged since 515a488). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at af3d105. Python cites read portlight@9b02494 path:line. Legend: [exists] code or data fact with a cite | [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

Every table row below that carries a cite is [exists] (a code or data fact); rows tagged otherwise say so.

| Key | Value | Cite |
|---|---|---|
| Catalog name | Maestro Luciano | C:6174 |
| Teaches | La Destreza ([style table](README.md#3-styles-5)) | C:6175 |
| Home port | [Porto Novo](../ports/porto_novo.md) | C:6176 |
| Bloc of the home port | [The Exchange Alliance](../factions/exchange_alliance.md) | ports/porto_novo.md section 0 |
| Region | Mediterranean | C:4133 |
| Training cost | 80 silver | C:4163 |
| Training time | 5 days | C:4164 |
| Styles needed first | 0 | C:4165 |
| Body parts the style needs (gate is dead, DF-41) | hand, arm | C:4166 |
| Player sees today | the style row at this port (name, silver, days, styles needed, known yes or no), a confirm line, and a notice after training; never the master | logic.rs:2028-2062; logic.rs:1884; game.rs:5693-5714 |
| Not shown | master name, description, dialog | no reader in the crates (section 2) |
| Mechanic status | the master record is [flavour, no mechanic]; the real training gate is the style's own port list (DF-39) | training.rs:9-52; content.rs:1027-1032 |
| Python record | same strings (compared field by field, no differences); Python's train command prints the master name and dialog | portlight@9b02494 src/portlight/content/fighting_styles.py:252; src/portlight/app/cli.py:2426-2455 |

## 1. Grounded facts (every data line quoted)

- description: "A retired fencing master who teaches the Spanish school from a warehouse overlooking the harbor." (C:6177)
- dialog (a spoken line, quoted inside the data string): "The sword is a compass. Your body is the circle. Step correctly and no blade can touch you." (C:6178)
- style, description: "The Spanish science of defense. Every movement calculated, every thrust placed with geometric precision." (C:4134)
- style, historical note (real-world, shipped data): "Systematized by Jeronimo Sanchez de Carranza in 1569. The first European martial art treated as an academic discipline." (C:4135)
- style, special action: Estocada (estocada): attack, stamina 3, beats parry, slash, loses to dodge, damage +2, cooldown 2 (C:4143)
- style, special action flavour: "A geometric thrust - the blade finds the only open line." (C:4156)

Quotes are as in content.json; an em dash is folded to ' - ' and a tilde n to n, and both are tagged [ascii-folded].

## 2. Role in play (what the sim does today; no proposals here)

- [exists] Training gate: the sim never reads this master. A captain may learn La Destreza at any port in the style's own list, which today is Porto Novo and Al-Manar (C:4159); the master's port is one of the two, so the data agrees but is redundant (training.rs:9-52, the check at 22-24; content.rs:1027-1032; training.rs:73-79; the only caller is a test, training.rs:101).
- [exists] Cost and time: 80 silver, then 5 days of the calendar, the same price at both ports; the master changes neither (training.rs:55-65).
- [exists] Injury gate: the sim checks the style's body parts only when it is handed a list of injured parts, and the session hands it an empty list, so an injured captain is never refused (session.rs:3331-3351, the empty list at 3337; DF-41).
- [exists] After training: the style is stored in the captain's learned styles (training.rs:55-65). No Rust code sets the active style, so a learned style gives no passive and no special action in a game started in Rust (model.rs:523; DF-41).
- [exists] What the Godot desk prints, built in code not authored [exists]: the row 'La Destreza  80 silver  5 days  needs 0 styles  known no' (logic.rs:2041-2045); the confirm line 'Train La Destreza for 80 silver. The calendar will advance 5 days.' (logic.rs:1884); the notice 'Learned la_destreza. The calendar advanced N days.' with the raw id (game.rs:5708; DF-40).
- [exists] Text reach: the master's name, description and dialog have no reader in the crates (rg for style_master and masters_at in crates/portlight-godot/src finds nothing; the sim has the one test caller). Python prints the name and the dialog in its training view (cli.py:2426-2455; combat_views.py:359-412).

## 3. Look and feel (words only; the character hold stands)

- Voice [cite]: precise and geometric: the sword is a compass, the body is the circle - "The sword is a compass. Your body is the circle. Step correctly and no blade can touch you." (C:6178).
- Manner [invented] He corrects with a number, never with a shout; a pupil is told which step was a hand's width off.
- Setting [cite]: "A retired fencing master who teaches the Spanish school from a warehouse overlooking the harbor." (C:6177). [invented] The warehouse belongs to a grain factor of the Exchange and stands empty between harvests; the floor is chalked with circles that are scrubbed off before the next cargo comes in.
- No face, age, build or dress is set beyond the quoted data lines; canon adds none. Character plates are held.

## 4. [invented] lore

- [invented] He is a retired fencing master who came to the Middle Sea with the Spanish school already complete in his head; he keeps no register of pupils and no sign over the door.
- [invented] The Exchange's clerks send their sons to him for deportment; the Exchange does not ask what else is taught.

## 5. Hooks ([proposal] only; never a mechanic)

- [proposal] Copy hook: a one-line crew-desk caption naming the master beside the style row, if showing master text is ever approved (DF-39).
- Sound: ambience only. No sfx id is cited.

## 6. Links

- Home port: [Porto Novo](../ports/porto_novo.md); bloc [The Exchange Alliance](../factions/exchange_alliance.md)
- Style index: [style-masters/README.md](README.md)
- Other master of the same style: [Senora Vega](senora_vega.md) (Al-Manar)
- [Old Vasquez](../skill-trainers/old_vasquez.md) - the skill trainer at the same port
- Crew hired in this region: [officers/mediterranean.md](../officers/mediterranean.md)

## 7. Flags and open questions

- DF-39 (masters and master text unread), DF-41 (the style injury gate is dead and no path equips a style): see ../data-fixes.md.
- DF-40: Senora Vega's description claims 'the only fencing school east of Gibraltar'. Canon reading while unfixed: Luciano keeps no school, only a warehouse floor; Vega's is the one school with a hall and a register ([invented]).
- Wiring master text into the desk, or a style equip control, is out of scope for canon; it is a design decision.
