# Dewi Ayu (dewi_ayu)

Base commit: af3d105 (merged main, which includes phase 6A; content.json, the sim and the godot UI as cited are unchanged since 515a488). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at af3d105. Python cites read portlight@9b02494 path:line. Legend: [exists] code or data fact with a cite | [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

Every table row below that carries a cite is [exists] (a code or data fact); rows tagged otherwise say so.

| Key | Value | Cite |
|---|---|---|
| Catalog name | Dewi Ayu | C:6230 |
| Teaches | Silat ([style table](README.md#3-styles-5)) | C:6231 |
| Home port | [Monsoon Reach](../ports/monsoon_reach.md) | C:6232 |
| Bloc of the home port | [The Free Ports](../factions/free_ports.md) | ports/monsoon_reach.md section 0 |
| Region | East Indies | C:4254 |
| Training cost | 120 silver | C:4284 |
| Training time | 8 days | C:4285 |
| Styles needed first | 1 | C:4286 |
| Body parts the style needs (checked on train and equip, DF-41 resolved) | hand, leg | C:4287 |
| Player sees today | the style row at this port (name, silver, days, styles needed, known yes or no), a confirm line, and a notice after training; never the master | logic.rs:2028-2062; logic.rs:1884; game.rs:5693-5714 |
| Master text on desk | master name, description, dialog, and style description are shown on the Crew desk (DF-39 resolved) | logic.rs; game.rs |
| Mechanic status | masters_at_port gates training alongside the style's port list, and learned styles can be equipped (DF-39, DF-41 resolved) | training.rs:9-52; session.rs |
| Python record | same strings (compared field by field, no differences); Python's train command prints the master name and dialog | portlight@9b02494 src/portlight/content/fighting_styles.py:350; src/portlight/app/cli.py:2426-2455 |

## 1. Grounded facts (every data line quoted)

- description: "A trader's daughter who learned silat from her grandmother. She teaches in the monsoon rain because 'the deck is always wet.'" (C:6233)
- dialog (a spoken line, quoted inside the data string): "Western fencing stands tall and proud. Silat crouches low and survives. Which do you think lasts longer at sea?" (C:6234)
- style, description: "The blade and grappling art of the Malay Archipelago. Low stances, curved blades, trapping hands, and devastating close-range techniques." (C:4255)
- style, historical note (real-world, shipped data): "Pencak Silat was already ancient when Portuguese traders first documented it in the 1500s. The kris dagger is both weapon and cultural artifact." (C:4256)
- style, special action: Keris Strike (keris_strike): attack, stamina 3, beats slash, loses to parry, dodge, damage +1, cooldown 2 (C:4264)
- style, special action flavour: "The curved kris finds the gap between guard and grip - the opponent's weapon arm goes numb." (C:4277)

Quotes are as in content.json; an em dash is folded to ' - ' and a tilde n to n, and both are tagged [ascii-folded].

## 2. Role in play (what the sim does today; no proposals here)

- [exists] Training gate: the sim never reads this master. A captain may learn Silat at any port in the style's own list, which today is Jade Port and Monsoon Reach (C:4280); the master's port is one of the two, so the data agrees but is redundant (training.rs:9-52, the check at 22-24; content.rs:1027-1032; training.rs:73-79; the only caller is a test, training.rs:101).
- [exists] Cost and time: 120 silver, then 8 days of the calendar, the same price at both ports; the master changes neither (training.rs:55-65).
- [exists] Injury gate: the sim checks the style's body parts only when it is handed a list of injured parts, and the session hands it an empty list, so an injured captain is never refused (session.rs:3331-3351, the empty list at 3337; DF-41).
- [exists] After training: the style is stored in the captain's learned styles (training.rs:55-65). No Rust code sets the active style, so a learned style gives no passive and no special action in a game started in Rust (model.rs:523; DF-41).
- [exists] What the Godot desk prints, built in code not authored [exists]: the row 'Silat  120 silver  8 days  needs 1 styles  known no' (logic.rs:2041-2045); the confirm line 'Train Silat for 120 silver. The calendar will advance 8 days.' (logic.rs:1884); the notice 'Learned silat. The calendar advanced N days.' with the raw id (game.rs:5708; DF-40).
- [exists] Text reach: the master's name, description and dialog have no reader in the crates (rg for style_master and masters_at in crates/portlight-godot/src finds nothing; the sim has the one test caller). Python prints the name and the dialog in its training view (cli.py:2426-2455; combat_views.py:359-412).

## 3. Look and feel (words only; the character hold stands)

- Voice [cite]: wry and low to the deck: silat crouches and survives - "Western fencing stands tall and proud. Silat crouches low and survives. Which do you think lasts longer at sea?" (C:6234).
- Manner [invented] She answers a question with a bigger question about the sea.
- Setting [cite]: "A trader's daughter who learned silat from her grandmother. She teaches in the monsoon rain because 'the deck is always wet.'" (C:6233). The monsoon rain and the wet deck are the data's own setting; [invented] the deck is an old hulk in the shallows kept for the purpose.
- No face, age, build or dress is set beyond the quoted data lines; canon adds none. Character plates are held.

## 4. [invented] lore

- [invented] She is a trader's daughter whose grandmother taught her silat on the family's own deck; she opened the lessons to captains because the Free Ports pay in coin and ask nothing else.
- [invented] Her dialog needles Western fencing; the masters of the Middle Sea would answer that a tall stance wins on dry land.

## 5. Hooks ([proposal] only; never a mechanic)

- [proposal] Sound hook: ambience only - rain on planking, the Wind Temple's chimes beyond. No cue id is cited.
- Sound: ambience only. No sfx id is cited.

## 6. Links

- Home port: [Monsoon Reach](../ports/monsoon_reach.md); bloc [The Free Ports](../factions/free_ports.md)
- Style index: [style-masters/README.md](README.md)
- Other master of the same style: [Pak Reza](pak_reza.md) (Jade Port)
- [The Wind Temple](../landmarks/wind_temple.md) - the landmark of the same port
- [Rajan the Tinker](../skill-trainers/monsoon_tinker.md) - the skill trainer at the same port
- Crew hired in this region: [officers/east_indies.md](../officers/east_indies.md)

## 7. Flags and open questions

- DF-39 (resolved: master gate and desk text wired), DF-40 (resolved: Senora Vega line and catalog-name notices), DF-41 (resolved: injury gate and style equipping wired), DF-48 (resolved: singular prerequisite grammar): see ../data-fixes.md.
- Her description uses single quotes for a spoken phrase inside the data line ('the deck is always wet.', C:6233); quoted as data.
- Wiring master text into the desk, or a style equip control, is out of scope for canon; it is a design decision.
