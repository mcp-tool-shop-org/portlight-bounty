# The Silva Bay Smithy (silva_bay_smiths)

Base commit: af3d105 (merged main, which includes phase 6A; content.json, the sim and the godot UI as cited are unchanged since 515a488). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at af3d105. Python cites read portlight@9b02494 path:line. Legend: [exists] code or data fact with a cite | [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

Every table row below that carries a cite is [exists] (a code or data fact); rows tagged otherwise say so.

| Key | Value | Cite |
|---|---|---|
| Catalog name | The Silva Bay Smithy | C:5418 |
| Skill taught | Blacksmithing ([skill table](README.md#2-the-skill-and-its-levels)) | C:5419 |
| Home port | [Silva Bay](../ports/silva_bay.md) | C:5420 |
| Bloc of the home port | [The Exchange Alliance](../factions/exchange_alliance.md) | ports/silva_bay.md section 0 |
| Region | Mediterranean | C:381 |
| Teaches up to | level 3 (Master) | C:5421 |
| Silver and days for the levels taught | Apprentice 50 silver, 3 days; Journeyman 150 silver, 5 days; Master 400 silver, 8 days | C:5374, C:5381, C:5388 |
| Port has the shipyard feature | yes (the skill's training_port_feature is shipyard; checked in skills::can_learn_skill, DF-39 resolved) | C:382; C:5370; skills.rs:51 |
| Player sees today | the trainer's name in the skill line, '(teaches to 3)', and the next level's cost and days; never the description or dialog | logic.rs:2063-2128 |
| Trainer text on desk | trainer description, dialog, and level description are shown on the Crew desk (DF-39 resolved) | logic.rs; game.rs |
| Mechanic status | trainer port, 	raining_port_feature, teaching level, and Blacksmithing maintain/upgrade discounts are live (DF-39 resolved) | skills.rs:32-95; session.rs |
| Python record | same strings (compared field by field, no differences); Python's learn-skill command prints the trainer name and dialog | portlight@9b02494 src/portlight/content/skills.py:121; src/portlight/app/cli.py:2733-2760 |

## 1. Grounded facts (every data line quoted)

- description: "A family of shipwrights who also forge the finest blades in the Mediterranean." (C:5422)
- dialog (a spoken line, quoted inside the data string): "We build ships and swords from the same iron. Both need to survive the sea." (C:5423)
- teaches up to level 3 (C:5421); the skill has 3 levels (C:5371)

Quotes are as in content.json.

## 2. Role in play (what the sim does today; no proposals here)

- [exists] Skill gate: a captain docked here can learn Blacksmithing only up to level 3; at a port with no trainer the sim answers 'No Blacksmithing trainer at this port.' (skills.rs:32-82, the trainer check at 51-56, the teaching limit at 57-72; the Godot desk uses the trainers_at list at logic.rs:2072 and content.rs:948-955).
- [exists] Cost: level 1 costs 50 silver and 3 days, level 2 150 and 5, level 3 400 and 8, paid in one go for each level (C:5374, C:5381, C:5388; skills.rs:85-104; session.rs:3380-3405).
- [exists] What the desk prints, built in code not authored [exists]: the skill line 'Blacksmithing  <level>. The Silva Bay Smithy (teaches to 3).' (logic.rs:2112-2115); the confirm line 'Learn <level> Blacksmithing for N silver. The calendar will advance N days.' (logic.rs:1889-1891); the notice 'Trained blacksmith. The calendar advanced N days.' with the raw id (game.rs:5716-5735, line 5731; DF-40).
- [exists] Text reach: description and dialog have no reader in the crates. Python prints the dialog of every trainer at the port (src/portlight/app/cli.py:2742-2760).
- [exists] Skill effects: the discount and degrade numbers are live in the sim but their routes do not reach the Godot player (README section 3; DF-39).

## 3. Look and feel (words only; the character hold stands)

- Voice [cite]: collective and steady: ships and swords are made from the same iron - "We build ships and swords from the same iron. Both need to survive the sea." (C:5423).
- Manner [invented] Several voices answer in turn; the eldest speaks last.
- Setting [cite]: "A family of shipwrights who also forge the finest blades in the Mediterranean." (C:5422). [invented] The Smithy stands on the slip beside the Master Shipyard and uses the yard's spare iron.
- No face, age, build or dress is set beyond the quoted data lines; canon adds none. Character plates are held.

## 4. [invented] lore

- [invented] The family are shipwrights first; the blades began as a way to use the offcuts from the keel bolts and became the finest in the Middle Sea.
- [invented] The Brotherhood Council (C:7559) counts the Smithy among its craft houses; the shipyard's master (C:7493) and the family share a slipway.

## 5. Hooks ([proposal] only; never a mechanic)

- [proposal] Copy hook: the family's line as the title of a future smithing note; flavour only.
- Sound: ambience only. No sfx id is cited.

## 6. Links

- Home port: [Silva Bay](../ports/silva_bay.md); bloc [The Exchange Alliance](../factions/exchange_alliance.md)
- Skill index: [skill-trainers/README.md](README.md)
- [The Master Shipyard](../landmarks/master_shipyard.md) - the landmark of the same port
- Crew hired in this region: [officers/mediterranean.md](../officers/mediterranean.md)

## 7. Flags and open questions

- DF-39 (resolved: trainer text, 	raining_port_feature, and Blacksmithing maintain/upgrade routes wired): see ../data-fixes.md. DF-40 (resolved: catalog name in skill notice and empty-trainer line).
- The trainer is a group, not a person (C:5418). The name Silva also belongs to the merchant Old Silva of the same port (C:5900) and to the port itself; canon reads the three as unrelated uses of a common place name (resolved in lore).
- Wiring trainer text into the desk is out of scope for canon; it is a design decision.
