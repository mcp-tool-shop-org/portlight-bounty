# Jarl Solveig (jarl_solveig)

Base commit: af3d105 (merged main, which includes phase 6A; content.json, the sim and the godot UI as cited are unchanged since 515a488). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at af3d105. Python cites read portlight@9b02494 path:line. Legend: [exists] code or data fact with a cite | [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

Every table row below that carries a cite is [exists] (a code or data fact); rows tagged otherwise say so.

| Key | Value | Cite |
|---|---|---|
| Catalog name | Jarl Solveig | C:6198 |
| Teaches | Highland Broadsword ([style table](README.md#3-styles-5)) | C:6199 |
| Home port | [Thornport](../ports/thornport.md) | C:6200 |
| Bloc of the home port | [The Free Ports](../factions/free_ports.md) | ports/thornport.md section 0 |
| Region | North Atlantic | C:4174 |
| Training cost | 100 silver | C:4204 |
| Training time | 7 days | C:4205 |
| Styles needed first | 0 | C:4206 |
| Body parts the style needs (checked on train and equip, DF-41 resolved) | arm | C:4207 |
| Player sees today | the style row at this port (name, silver, days, styles needed, known yes or no), a confirm line, and a notice after training; never the master | logic.rs:2028-2062; logic.rs:1884; game.rs:5693-5714 |
| Master text on desk | master name, description, dialog, and style description are shown on the Crew desk (DF-39 resolved) | logic.rs; game.rs |
| Mechanic status | masters_at_port gates training alongside the style's port list, and learned styles can be equipped (DF-39, DF-41 resolved) | training.rs:9-52; session.rs |
| Python record | same strings (compared field by field, no differences); Python's train command prints the master name and dialog | portlight@9b02494 src/portlight/content/fighting_styles.py:294; src/portlight/app/cli.py:2426-2455 |

## 1. Grounded facts (every data line quoted)

- description: "A Norse shieldmaiden who settled in Thornport after twenty years of raiding. She teaches the old ways." (C:6201)
- dialog (a spoken line, quoted inside the data string): "The broadsword is not a weapon. It is a statement. When you swing, mean it." (C:6202)
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

- Voice [cite]: grave and spare: the broadsword is a statement, and a swing should mean it - "The broadsword is not a weapon. It is a statement. When you swing, mean it." (C:6202).
- Manner [invented] She says little, and what she says is in the form of a rule.
- Setting [cite]: "A Norse shieldmaiden who settled in Thornport after twenty years of raiding. She teaches the old ways." (C:6201). [invented] She teaches on the quay below the Whale Arch, where whalers hang their gear to dry.
- No face, age, build or dress is set beyond the quoted data lines; canon adds none. Character plates are held.

## 4. [invented] lore

- [invented] Her twenty years of raiding were along the Iron Coast; she settled at Thornport because the Free Ports ask no captain what flag a past voyage flew.
- [invented] Old Henry, the navigator who hires out of the same port, remembers her ships and does not say so.

## 5. Hooks ([proposal] only; never a mechanic)

- [proposal] Copy hook: a line for the style row's flavour that nods to the old ways; flavour only.
- Sound: ambience only. No sfx id is cited.

## 6. Links

- Home port: [Thornport](../ports/thornport.md); bloc [The Free Ports](../factions/free_ports.md)
- Style index: [style-masters/README.md](README.md)
- Other master of the same style: [Red Hamish](red_hamish.md) (Ironhaven)
- [The Whale Arch](../landmarks/whale_arch.md) - the landmark of the same port
- [Old Henry](../companions/old_henry.md) - the companion at the same port
- Crew hired in this region: [officers/north_atlantic.md](../officers/north_atlantic.md)

## 7. Flags and open questions

- DF-39 (resolved: master gate and desk text wired), DF-40 (resolved: Senora Vega line and catalog-name notices), DF-41 (resolved: injury gate and style equipping wired), DF-48 (resolved: singular prerequisite grammar): see ../data-fixes.md.
- Real-world words in data: Norse, shieldmaiden; quoted as data and not extended. Shares a port with the bladesmith Hamish (C:5947), who is a separate trade.
- Wiring master text into the desk, or a style equip control, is out of scope for canon; it is a design decision.
