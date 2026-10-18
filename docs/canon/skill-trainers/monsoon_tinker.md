# Rajan the Tinker (monsoon_tinker)

Base commit: af3d105 (merged main, which includes phase 6A; content.json, the sim and the godot UI as cited are unchanged since 515a488). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at af3d105. Python cites read portlight@9b02494 path:line. Legend: [exists] code or data fact with a cite | [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

Every table row below that carries a cite is [exists] (a code or data fact); rows tagged otherwise say so.

| Key | Value | Cite |
|---|---|---|
| Catalog name | Rajan the Tinker | C:5427 |
| Skill taught | Blacksmithing ([skill table](README.md#2-the-skill-and-its-levels)) | C:5428 |
| Home port | [Monsoon Reach](../ports/monsoon_reach.md) | C:5429 |
| Bloc of the home port | [The Free Ports](../factions/free_ports.md) | ports/monsoon_reach.md section 0 |
| Region | East Indies | C:1209 |
| Teaches up to | level 2 (Journeyman) | C:5430 |
| Silver and days for the levels taught | Apprentice 50 silver, 3 days; Journeyman 150 silver, 5 days | C:5374, C:5381 |
| Port has the shipyard feature | yes (the skill's training_port_feature is shipyard; checked in skills::can_learn_skill, DF-39 resolved) | C:1210; C:5370; skills.rs:51 |
| Player sees today | the trainer's name in the skill line, '(teaches to 2)', and the next level's cost and days; never the description or dialog | logic.rs:2063-2128 |
| Trainer text on desk | trainer description, dialog, and level description are shown on the Crew desk (DF-39 resolved) | logic.rs; game.rs |
| Mechanic status | trainer port, 	raining_port_feature, teaching level, and Blacksmithing maintain/upgrade discounts are live (DF-39 resolved) | skills.rs:32-95; session.rs |
| Python record | same strings (compared field by field, no differences); Python's learn-skill command prints the trainer name and dialog | portlight@9b02494 src/portlight/content/skills.py:130; src/portlight/app/cli.py:2733-2760 |

## 1. Grounded facts (every data line quoted)

- description: "An East Indies metalworker who repairs anything from kris daggers to Portuguese cannons." (C:5431)
- dialog (a spoken line, quoted inside the data string): "Every weapon that passes through this port, I have fixed. Bring me yours." (C:5432)
- teaches up to level 2 (C:5430); the skill has 3 levels (C:5371)

Quotes are as in content.json.

## 2. Role in play (what the sim does today; no proposals here)

- [exists] Skill gate: a captain docked here can learn Blacksmithing only up to level 2; at a port with no trainer the sim answers 'No Blacksmithing trainer at this port.' (skills.rs:32-82, the trainer check at 51-56, the teaching limit at 57-72; the Godot desk uses the trainers_at list at logic.rs:2072 and content.rs:948-955).
- [exists] Cost: level 1 costs 50 silver and 3 days, level 2 150 and 5, level 3 400 and 8, paid in one go for each level (C:5374, C:5381, C:5388; skills.rs:85-104; session.rs:3380-3405).
- [exists] What the desk prints, built in code not authored [exists]: the skill line 'Blacksmithing  <level>. Rajan the Tinker (teaches to 2).' (logic.rs:2112-2115); the confirm line 'Learn <level> Blacksmithing for N silver. The calendar will advance N days.' (logic.rs:1889-1891); the notice 'Trained blacksmith. The calendar advanced N days.' with the raw id (game.rs:5716-5735, line 5731; DF-40).
- [exists] Text reach: description and dialog have no reader in the crates. Python prints the dialog of every trainer at the port (src/portlight/app/cli.py:2742-2760).
- [exists] Skill effects: the discount and degrade numbers are live in the sim but their routes do not reach the Godot player (README section 3; DF-39).

## 3. Look and feel (words only; the character hold stands)

- Voice [cite]: cheerful and capable: every weapon that passed through this port has been fixed - "Every weapon that passes through this port, I have fixed. Bring me yours." (C:5432).
- Manner [invented] He turns each blade over twice and names the harbour it was last mended in.
- Setting [cite]: "An East Indies metalworker who repairs anything from kris daggers to Portuguese cannons." (C:5431). [invented] His bench is under the shipyard's lean-to, among the spare fittings.
- No face, age, build or dress is set beyond the quoted data lines; canon adds none. Character plates are held.

## 4. [invented] lore

- [invented] He mends kris daggers and Portuguese cannons alike (C:5431), which is why captains from every bloc bring him work.
- [invented] He teaches to level 2 and says the third level is a luxury; the Free Ports' captains rarely stay long enough to learn it.

## 5. Hooks ([proposal] only; never a mechanic)

- [proposal] Sound hook: ambience only - a tinker's hammer among shipyard sounds. No cue id is cited.
- Sound: ambience only. No sfx id is cited.

## 6. Links

- Home port: [Monsoon Reach](../ports/monsoon_reach.md); bloc [The Free Ports](../factions/free_ports.md)
- Skill index: [skill-trainers/README.md](README.md)
- [The Wind Temple](../landmarks/wind_temple.md) - the landmark of the same port
- [Dewi Ayu](../style-masters/dewi_ayu.md) - the style master at the same port
- Crew hired in this region: [officers/east_indies.md](../officers/east_indies.md)

## 7. Flags and open questions

- DF-39 (resolved: trainer text, 	raining_port_feature, and Blacksmithing maintain/upgrade routes wired): see ../data-fixes.md. DF-40 (resolved: catalog name in skill notice and empty-trainer line).
- Real-world word in data: Portuguese (cannons), quoted as data. Given-name overlap: an officer name pool entry Rajan (C:6125) is a namesake (DF-38, declined).
- Wiring trainer text into the desk is out of scope for canon; it is a design decision.
