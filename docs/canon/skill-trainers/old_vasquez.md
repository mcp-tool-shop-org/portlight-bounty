# Old Vasquez (old_vasquez)

Base commit: af3d105 (merged main, which includes phase 6A; content.json, the sim and the godot UI as cited are unchanged since 515a488). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at af3d105. Python cites read portlight@9b02494 path:line. Legend: [exists] code or data fact with a cite | [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

Every table row below that carries a cite is [exists] (a code or data fact); rows tagged otherwise say so.

| Key | Value | Cite |
|---|---|---|
| Catalog name | Old Vasquez | C:5409 |
| Skill taught | Blacksmithing ([skill table](README.md#2-the-skill-and-its-levels)) | C:5410 |
| Home port | [Porto Novo](../ports/porto_novo.md) | C:5411 |
| Bloc of the home port | [The Exchange Alliance](../factions/exchange_alliance.md) | ports/porto_novo.md section 0 |
| Region | Mediterranean | C:213 |
| Teaches up to | level 2 (Journeyman) | C:5412 |
| Silver and days for the levels taught | Apprentice 50 silver, 3 days; Journeyman 150 silver, 5 days | C:5374, C:5381 |
| Port has the shipyard feature | yes (the skill's training_port_feature is shipyard; the sim does not read it) | C:214; C:5370; skills.rs:51 |
| Player sees today | the trainer's name in the skill line, '(teaches to 2)', and the next level's cost and days; never the description or dialog | logic.rs:2063-2128 |
| Not shown | description, dialog | no reader in the crates (section 2) |
| Mechanic status | the trainer's port and teaching level are live (they gate the skill); the text is [flavour, no mechanic] | skills.rs:32-82; DF-39 |
| Python record | same strings (compared field by field, no differences); Python's learn-skill command prints the trainer name and dialog | portlight@9b02494 src/portlight/content/skills.py:112; src/portlight/app/cli.py:2733-2760 |

## 1. Grounded facts (every data line quoted)

- description: "A retired armorer who sharpens blades in the harbor market. Fast and cheap, but not a master." (C:5413)
- dialog (a spoken line, quoted inside the data string): "I can teach you to keep a blade sharp and a pistol clean. That's worth more than fancy swordplay." (C:5414)
- teaches up to level 2 (C:5412); the skill has 3 levels (C:5371)

Quotes are as in content.json.

## 2. Role in play (what the sim does today; no proposals here)

- [exists] Skill gate: a captain docked here can learn Blacksmithing only up to level 2; at a port with no trainer the sim answers 'No Blacksmithing trainer at this port.' (skills.rs:32-82, the trainer check at 51-56, the teaching limit at 57-72; the Godot desk uses the trainers_at list at logic.rs:2072 and content.rs:948-955).
- [exists] Cost: level 1 costs 50 silver and 3 days, level 2 150 and 5, level 3 400 and 8, paid in one go for each level (C:5374, C:5381, C:5388; skills.rs:85-104; session.rs:3380-3405).
- [exists] What the desk prints, built in code not authored [exists]: the skill line 'Blacksmithing  <level>. Old Vasquez (teaches to 2).' (logic.rs:2112-2115); the confirm line 'Learn <level> Blacksmithing for N silver. The calendar will advance N days.' (logic.rs:1889-1891); the notice 'Trained blacksmith. The calendar advanced N days.' with the raw id (game.rs:5716-5735, line 5731; DF-40).
- [exists] Text reach: description and dialog have no reader in the crates. Python prints the dialog of every trainer at the port (src/portlight/app/cli.py:2742-2760).
- [exists] Skill effects: the discount and degrade numbers are live in the sim but their routes do not reach the Godot player (README section 3; DF-39).

## 3. Look and feel (words only; the character hold stands)

- Voice [cite]: plain and practical: keep a blade sharp and a pistol clean - "I can teach you to keep a blade sharp and a pistol clean. That's worth more than fancy swordplay." (C:5414).
- Manner [invented] He gives the cheapest advice first and charges for the rest.
- Setting [cite]: "A retired armorer who sharpens blades in the harbor market. Fast and cheap, but not a master." (C:5413). The harbour market is the data's own setting; [invented] he works from a stall with a grindstone and a pot of oil.
- No face, age, build or dress is set beyond the quoted data lines; canon adds none. Character plates are held.

## 4. [invented] lore

- [invented] He is a retired armorer of the Alliance who gave up the great forges for a stall where a captain can walk in off the quay.
- [invented] He is the first smith many captains meet and teaches only the first two levels because, he says, a master should be found, not bought.

## 5. Hooks ([proposal] only; never a mechanic)

- [proposal] Copy hook: his 'not a master' line as the honest label on a level-two trainer; flavour only.
- Sound: ambience only. No sfx id is cited.

## 6. Links

- Home port: [Porto Novo](../ports/porto_novo.md); bloc [The Exchange Alliance](../factions/exchange_alliance.md)
- Skill index: [skill-trainers/README.md](README.md)
- [The Grain Exchange](../landmarks/grain_exchange.md) - a landmark of the same port
- [Maestro Luciano](../style-masters/maestro_luciano.md) - the style master at the same port
- Crew hired in this region: [officers/mediterranean.md](../officers/mediterranean.md)

## 7. Flags and open questions

- DF-39 (trainer text unread; skill training_port_feature is data only): see ../data-fixes.md. DF-40 (the skill notice prints a raw id).
- His dialog mentions a pistol (C:5414); no firearm rule is attached to the trainer. He teaches to level 2 only (C:5412).
- Wiring trainer text into the desk is out of scope for canon; it is a design decision.
