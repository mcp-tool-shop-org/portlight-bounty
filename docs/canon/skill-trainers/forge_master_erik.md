# Forge Master Erik (forge_master_erik)

Base commit: af3d105 (merged main, which includes phase 6A; content.json, the sim and the godot UI as cited are unchanged since 515a488). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at af3d105. Python cites read portlight@9b02494 path:line. Legend: [exists] code or data fact with a cite | [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

Every table row below that carries a cite is [exists] (a code or data fact); rows tagged otherwise say so.

| Key | Value | Cite |
|---|---|---|
| Catalog name | Forge Master Erik | C:5400 |
| Skill taught | Blacksmithing ([skill table](README.md#2-the-skill-and-its-levels)) | C:5401 |
| Home port | [Ironhaven](../ports/ironhaven.md) | C:5402 |
| Bloc of the home port | [The Iron Pact](../factions/iron_pact.md) | ports/ironhaven.md section 0 |
| Region | North Atlantic | C:559 |
| Teaches up to | level 3 (Master) | C:5403 |
| Silver and days for the levels taught | Apprentice 50 silver, 3 days; Journeyman 150 silver, 5 days; Master 400 silver, 8 days | C:5374, C:5381, C:5388 |
| Port has the shipyard feature | yes (the skill's training_port_feature is shipyard; the sim does not read it) | C:560; C:5370; skills.rs:51 |
| Player sees today | the trainer's name in the skill line, '(teaches to 3)', and the next level's cost and days; never the description or dialog | logic.rs:2063-2128 |
| Not shown | description, dialog | no reader in the crates (section 2) |
| Mechanic status | the trainer's port and teaching level are live (they gate the skill); the text is [flavour, no mechanic] | skills.rs:32-82; DF-39 |
| Python record | same strings (compared field by field, no differences); Python's learn-skill command prints the trainer name and dialog | portlight@9b02494 src/portlight/content/skills.py:103; src/portlight/app/cli.py:2733-2760 |

## 1. Grounded facts (every data line quoted)

- description: "The best smith in the North Atlantic. His blades hold an edge for years." (C:5404)
- dialog (a spoken line, quoted inside the data string): "Steel remembers every hammer blow. I'll teach your hands to speak its language." (C:5405)
- teaches up to level 3 (C:5403); the skill has 3 levels (C:5371)

Quotes are as in content.json.

## 2. Role in play (what the sim does today; no proposals here)

- [exists] Skill gate: a captain docked here can learn Blacksmithing only up to level 3; at a port with no trainer the sim answers 'No Blacksmithing trainer at this port.' (skills.rs:32-82, the trainer check at 51-56, the teaching limit at 57-72; the Godot desk uses the trainers_at list at logic.rs:2072 and content.rs:948-955).
- [exists] Cost: level 1 costs 50 silver and 3 days, level 2 150 and 5, level 3 400 and 8, paid in one go for each level (C:5374, C:5381, C:5388; skills.rs:85-104; session.rs:3380-3405).
- [exists] What the desk prints, built in code not authored [exists]: the skill line 'Blacksmithing  <level>. Forge Master Erik (teaches to 3).' (logic.rs:2112-2115); the confirm line 'Learn <level> Blacksmithing for N silver. The calendar will advance N days.' (logic.rs:1889-1891); the notice 'Trained blacksmith. The calendar advanced N days.' with the raw id (game.rs:5716-5735, line 5731; DF-40).
- [exists] Text reach: description and dialog have no reader in the crates. Python prints the dialog of every trainer at the port (src/portlight/app/cli.py:2742-2760).
- [exists] Skill effects: the discount and degrade numbers are live in the sim but their routes do not reach the Godot player (README section 3; DF-39).

## 3. Look and feel (words only; the character hold stands)

- Voice [cite]: proud and craftsmanlike: steel remembers every hammer blow - "Steel remembers every hammer blow. I'll teach your hands to speak its language." (C:5405).
- Manner [invented] He judges a captain by the edge on the blade carried in, and says so.
- Setting [cite]: "The best smith in the North Atlantic. His blades hold an edge for years." (C:5404). [invented] His forge is in the lee of the Great Foundry's yard wall, one of the small shops the Foundry's smoke feeds.
- No face, age, build or dress is set beyond the quoted data lines; canon adds none. Character plates are held.

## 4. [invented] lore

- [invented] He is a journeyman-trained smith of the Pact who stayed a smith when others became guildsmen; the Guild title 'Forge Master' is held by him as a craft and by Henrik Brandt as an office.
- [invented] He and the silent Smith of the same port (C:8027) do not compete: one teaches the keeping of an edge, the other makes the blade.

## 5. Hooks ([proposal] only; never a mechanic)

- [proposal] Copy hook: a line for the skill row naming him as the best smith of the Iron Coast; flavour only.
- Sound: ambience only. No sfx id is cited.

## 6. Links

- Home port: [Ironhaven](../ports/ironhaven.md); bloc [The Iron Pact](../factions/iron_pact.md)
- Skill index: [skill-trainers/README.md](README.md)
- [The Great Foundry](../landmarks/great_foundry.md) - the landmark of the same port
- [Red Hamish](../style-masters/red_hamish.md) - the style master at the same port
- [Iron Marta](../companions/iron_marta.md) - the companion at the same port
- Crew hired in this region: [officers/north_atlantic.md](../officers/north_atlantic.md)

## 7. Flags and open questions

- DF-39 (trainer text unread; skill training_port_feature is data only): see ../data-fixes.md. DF-40 (the skill notice prints a raw id).
- Title overlap: 'Forge Master' is also Henrik Brandt's title (C:7983); resolved in lore as a craft and an office. Given-name overlap: Erik Strand at Thornport (C:8533) is a different person (DF-37).
- Wiring trainer text into the desk is out of scope for canon; it is a design decision.
