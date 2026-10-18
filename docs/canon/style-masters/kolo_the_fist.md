# Kolo the Fist (kolo_the_fist)

Base commit: af3d105 (merged main, which includes phase 6A; content.json, the sim and the godot UI as cited are unchanged since 515a488). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at af3d105. Python cites read portlight@9b02494 path:line. Legend: [exists] code or data fact with a cite | [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

Every table row below that carries a cite is [exists] (a code or data fact); rows tagged otherwise say so.

| Key | Value | Cite |
|---|---|---|
| Catalog name | Kolo the Fist | C:6206 |
| Teaches | Dambe ([style table](README.md#3-styles-5)) | C:6207 |
| Home port | [Sun Harbor](../ports/sun_harbor.md) | C:6208 |
| Bloc of the home port | [The Gold Coast Compact](../factions/gold_coast_compact.md) | ports/sun_harbor.md section 0 |
| Region | West Africa | C:4214 |
| Training cost | 60 silver | C:4244 |
| Training time | 6 days | C:4245 |
| Styles needed first | 0 | C:4246 |
| Body parts the style needs (checked on train and equip, DF-41 resolved) | leg | C:4247 |
| Player sees today | the style row at this port (name, silver, days, styles needed, known yes or no), a confirm line, and a notice after training; never the master | logic.rs:2028-2062; logic.rs:1884; game.rs:5693-5714 |
| Master text on desk | master name, description, dialog, and style description are shown on the Crew desk (DF-39 resolved) | logic.rs; game.rs |
| Mechanic status | masters_at_port gates training alongside the style's port list, and learned styles can be equipped (DF-39, DF-41 resolved) | training.rs:9-52; session.rs |
| Python record | same strings (compared field by field, no differences); Python's train command prints the master name and dialog | portlight@9b02494 src/portlight/content/fighting_styles.py:308; src/portlight/app/cli.py:2426-2455 |

## 1. Grounded facts (every data line quoted)

- description: "A retired champion whose wrapped fists have broken more guards than any siege engine. He teaches on the beach at dawn." (C:6209)
- dialog (a spoken line, quoted inside the data string): "The spear fist strikes. The shield hand protects. The leg decides. Simple. Now do it a thousand times." (C:6210)
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

- Voice [cite]: patient and rhythmic: four short lines, then 'a thousand times' - "The spear fist strikes. The shield hand protects. The leg decides. Simple. Now do it a thousand times." (C:6210).
- Manner [invented] He counts in a chant and expects the pupil to chant back.
- Setting [cite]: "A retired champion whose wrapped fists have broken more guards than any siege engine. He teaches on the beach at dawn." (C:6209). The beach at dawn is the data's own setting; [invented] he stops when the first cotton barges reach the Cotton Steps.
- No face, age, build or dress is set beyond the quoted data lines; canon adds none. Character plates are held.

## 4. [invented] lore

- [invented] He retired undefeated and has refused every offer to fight again; the wrapped fists in his description are a champion's, and the wraps are kept in a chest he does not open for visitors.
- [invented] Dr. Amara, the surgeon who hires out of Sun Harbor, has set more than one of his pupils' fingers.

## 5. Hooks ([proposal] only; never a mechanic)

- [proposal] Sound hook: ambience only - surf and a chanted count at first light. No cue id is cited.
- Sound: ambience only. No sfx id is cited.

## 6. Links

- Home port: [Sun Harbor](../ports/sun_harbor.md); bloc [The Gold Coast Compact](../factions/gold_coast_compact.md)
- Style index: [style-masters/README.md](README.md)
- Other master of the same style: [Nneka Ironfist](nneka_ironfist.md) (Pearl Shallows)
- [The Cotton Steps](../landmarks/cotton_steps.md) - the landmark of the same port
- [Dr. Amara](../companions/dr_amara.md) - the companion at the same port
- Crew hired in this region: [officers/west_africa.md](../officers/west_africa.md)

## 7. Flags and open questions

- DF-39 (resolved: master gate and desk text wired), DF-40 (resolved: Senora Vega line and catalog-name notices), DF-41 (resolved: injury gate and style equipping wired), DF-48 (resolved: singular prerequisite grammar): see ../data-fixes.md.
- Real-world words in data: the style's description names the Hausa people (C:4215); quoted as data and not extended.
- Wiring master text into the desk, or a style equip control, is out of scope for canon; it is a design decision.
