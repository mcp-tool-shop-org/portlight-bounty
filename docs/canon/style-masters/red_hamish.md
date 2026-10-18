# Red Hamish (red_hamish)

Base commit: af3d105 (merged main, which includes phase 6A; content.json, the sim and the godot UI as cited are unchanged since 515a488). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at af3d105. Python cites read portlight@9b02494 path:line. Legend: [exists] code or data fact with a cite | [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

Every table row below that carries a cite is [exists] (a code or data fact); rows tagged otherwise say so.

| Key | Value | Cite |
|---|---|---|
| Catalog name | Red Hamish | C:6190 |
| Teaches | Highland Broadsword ([style table](README.md#3-styles-5)) | C:6191 |
| Home port | [Ironhaven](../ports/ironhaven.md) | C:6192 |
| Bloc of the home port | [The Iron Pact](../factions/iron_pact.md) | ports/ironhaven.md section 0 |
| Region | North Atlantic | C:4174 |
| Training cost | 100 silver | C:4204 |
| Training time | 7 days | C:4205 |
| Styles needed first | 0 | C:4206 |
| Body parts the style needs (checked on train and equip, DF-41 resolved) | arm | C:4207 |
| Player sees today | the style row at this port (name, silver, days, styles needed, known yes or no), a confirm line, and a notice after training; never the master | logic.rs:2028-2062; logic.rs:1884; game.rs:5693-5714 |
| Master text on desk | master name, description, dialog, and style description are shown on the Crew desk (DF-39 resolved) | logic.rs; game.rs |
| Mechanic status | masters_at_port gates training alongside the style's port list, and learned styles can be equipped (DF-39, DF-41 resolved) | training.rs:9-52; session.rs |
| Python record | same strings (compared field by field, no differences); Python's train command prints the master name and dialog | portlight@9b02494 src/portlight/content/fighting_styles.py:280; src/portlight/app/cli.py:2426-2455 |

## 1. Grounded facts (every data line quoted)

- description: "A massive Scotsman with arms like anchor chains. He teaches broadsword in the foundry yard, sparking steel against iron." (C:6193)
- dialog (a spoken line, quoted inside the data string): "Ye dinnae need fancy footwork when ye can split a man's shield in two. Hit harder. Again." (C:6194)
- style, description: "The broadsword tradition of the Scottish Highlands and Norse raiders. No finesse, no subtlety - overwhelming force." (C:4175)
- style, historical note (real-world, shipped data): "Highland sword-and-targe techniques documented across Scotland and Scandinavia. The two-handed claymore was already legendary by the 1500s." (C:4176)
- style, special action: Cleave (cleave): attack, stamina 2, beats parry, loses to thrust, dodge, damage +4, cooldown 1 (C:4184)
- style, special action flavour: "A devastating overhead arc that shatters guards and bones alike." (C:4197)

Quotes are as in content.json; an em dash is folded to ' - ' and a tilde n to n, and both are tagged [ascii-folded].

## 2. Role in play (what the sim does today; no proposals here)

- [exists] Training gate: the sim never reads this master. A captain may learn Highland Broadsword at any port in the style's own list, which today is Ironhaven and Thornport (C:4200); the master's port is one of the two, so the data agrees but is redundant (training.rs:9-52, the check at 22-24; content.rs:1027-1032; training.rs:73-79; the only caller is a test, training.rs:101).
- [exists] Cost and time: 100 silver, then 7 days of the calendar, the same price at both ports; the master changes neither (training.rs:55-65).
- [exists] Injury gate: the sim checks the style's body parts only when it is handed a list of injured parts, and the session hands it an empty list, so an injured captain is never refused (session.rs:3331-3351, the empty list at 3337; DF-41).
- [exists] After training: the style is stored in the captain's learned styles (training.rs:55-65). No Rust code sets the active style, so a learned style gives no passive and no special action in a game started in Rust (model.rs:523; DF-41).
- [exists] What the Godot desk prints, built in code not authored [exists]: the row 'Highland Broadsword  100 silver  7 days  needs 0 styles  known no' (logic.rs:2041-2045); the confirm line 'Train Highland Broadsword for 100 silver. The calendar will advance 7 days.' (logic.rs:1884); the notice 'Learned highland_broadsword. The calendar advanced N days.' with the raw id (game.rs:5708; DF-40).
- [exists] Text reach: the master's name, description and dialog have no reader in the crates (rg for style_master and masters_at in crates/portlight-godot/src finds nothing; the sim has the one test caller). Python prints the name and the dialog in its training view (cli.py:2426-2455; combat_views.py:359-412).

## 3. Look and feel (words only; the character hold stands)

- Voice [cite]: broad and blunt, in a Scots turn of speech (dinnae, ye); the lesson is to hit harder - "Ye dinnae need fancy footwork when ye can split a man's shield in two. Hit harder. Again." (C:6194).
- Manner [invented] He teaches by repetition and counts aloud; 'Again' is most of his vocabulary.
- Setting [cite]: "A massive Scotsman with arms like anchor chains. He teaches broadsword in the foundry yard, sparking steel against iron." (C:6193). The foundry yard is the data's own setting (sparking steel against iron); [invented] it is the yard behind the Great Foundry, where scrap is cut up for the furnace.
- No face, age, build or dress is set beyond the quoted data lines; canon adds none. Character plates are held.

## 4. [invented] lore

- [invented] He came south with a cargo of Highland steel, was paid in iron, and stayed because the Foundry never stops hammering and nobody minds the noise of a lesson.
- [invented] The Pact's arsenal sergeants send him recruits who have been taught a drill but not a swing.

## 5. Hooks ([proposal] only; never a mechanic)

- [proposal] Sound hook: ambience only - a lesson heard through the yard wall under the foundry's own noise. No cue id is cited.
- Sound: ambience only. No sfx id is cited.

## 6. Links

- Home port: [Ironhaven](../ports/ironhaven.md); bloc [The Iron Pact](../factions/iron_pact.md)
- Style index: [style-masters/README.md](README.md)
- Other master of the same style: [Jarl Solveig](jarl_solveig.md) (Thornport)
- [The Great Foundry](../landmarks/great_foundry.md) - the landmark of the same port
- [Iron Marta](../companions/iron_marta.md) - the companion at the same port
- [Forge Master Erik](../skill-trainers/forge_master_erik.md) - the skill trainer at the same port
- Crew hired in this region: [officers/north_atlantic.md](../officers/north_atlantic.md)

## 7. Flags and open questions

- DF-39 (resolved: master gate and desk text wired), DF-40 (resolved: Senora Vega line and catalog-name notices), DF-41 (resolved: injury gate and style equipping wired), DF-48 (resolved: singular prerequisite grammar): see ../data-fixes.md.
- Given-name overlap: the bladesmith Hamish at Thornport (C:5947) is a different person (data-fixes resolved-in-lore row). Real-world word in data: Scotsman, quoted as data.
- Wiring master text into the desk, or a style equip control, is out of scope for canon; it is a design decision.
