# Kai the Shaper (typhoon_smith)

Base commit: af3d105 (merged main, which includes phase 6A; content.json, the sim and the godot UI as cited are unchanged since 515a488). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at af3d105. Python cites read portlight@9b02494 path:line. Legend: [exists] code or data fact with a cite | [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

Every table row below that carries a cite is [exists] (a code or data fact); rows tagged otherwise say so.

| Key | Value | Cite |
|---|---|---|
| Catalog name | Kai the Shaper | C:5436 |
| Skill taught | Blacksmithing ([skill table](README.md#2-the-skill-and-its-levels)) | C:5437 |
| Home port | [Typhoon Anchorage](../ports/typhoon_anchorage.md) | C:5438 |
| Bloc of the home port | [The Coral Crown](../factions/coral_crown.md) | ports/typhoon_anchorage.md section 0 |
| Region | South Seas | C:1753 |
| Teaches up to | level 1 (Apprentice) | C:5439 |
| Silver and days for the levels taught | Apprentice 50 silver, 3 days | C:5374 |
| Port has the shipyard feature | yes (the skill's training_port_feature is shipyard; the sim does not read it) | C:1754; C:5370; skills.rs:51 |
| Player sees today | the trainer's name in the skill line, '(teaches to 1)', and the next level's cost and days; never the description or dialog | logic.rs:2063-2128 |
| Not shown | description, dialog | no reader in the crates (section 2) |
| Mechanic status | the trainer's port and teaching level are live (they gate the skill); the text is [flavour, no mechanic] | skills.rs:32-82; DF-39 |
| Python record | same strings (compared field by field, no differences); Python's learn-skill command prints the trainer name and dialog | portlight@9b02494 src/portlight/content/skills.py:139; src/portlight/app/cli.py:2733-2760 |

## 1. Grounded facts (every data line quoted)

- description: "A young smith who learned from sailors. Basic work, but honest." (C:5440)
- dialog (a spoken line, quoted inside the data string): "I'm no master, but I can keep your steel from rusting in the salt air. That's something." (C:5441)
- teaches up to level 1 (C:5439); the skill has 3 levels (C:5371)

Quotes are as in content.json.

## 2. Role in play (what the sim does today; no proposals here)

- [exists] Skill gate: a captain docked here can learn Blacksmithing only up to level 1; at a port with no trainer the sim answers 'No Blacksmithing trainer at this port.' (skills.rs:32-82, the trainer check at 51-56, the teaching limit at 57-72; the Godot desk uses the trainers_at list at logic.rs:2072 and content.rs:948-955).
- [exists] Cost: level 1 costs 50 silver and 3 days, level 2 150 and 5, level 3 400 and 8, paid in one go for each level (C:5374, C:5381, C:5388; skills.rs:85-104; session.rs:3380-3405).
- [exists] What the desk prints, built in code not authored [exists]: the skill line 'Blacksmithing  <level>. Kai the Shaper (teaches to 1).' (logic.rs:2112-2115); the confirm line 'Learn <level> Blacksmithing for N silver. The calendar will advance N days.' (logic.rs:1889-1891); the notice 'Trained blacksmith. The calendar advanced N days.' with the raw id (game.rs:5716-5735, line 5731; DF-40).
- [exists] Text reach: description and dialog have no reader in the crates. Python prints the dialog of every trainer at the port (src/portlight/app/cli.py:2742-2760).
- [exists] Skill effects: the discount and degrade numbers are live in the sim but their routes do not reach the Godot player (README section 3; DF-39).

## 3. Look and feel (words only; the character hold stands)

- Voice [cite]: modest and honest: no master, but steel kept from rusting in salt air - "I'm no master, but I can keep your steel from rusting in the salt air. That's something." (C:5441).
- Manner [invented] He apologises for his limits before he is asked and then does the work well.
- Setting [cite]: "A young smith who learned from sailors. Basic work, but honest." (C:5440). [invented] His shop is a lean-to behind the Storm Shipwright's yard.
- No face, age, build or dress is set beyond the quoted data lines; canon adds none. Character plates are held.

## 4. [invented] lore

- [invented] He learned from sailors rather than from a guild, which is why he teaches only the first level and is easy to find.
- [invented] The Coral Crown's captains bring him work after a storm, when everything aboard has started to rust.

## 5. Hooks ([proposal] only; never a mechanic)

- [proposal] Copy hook: his 'I am no master' line as the honest label on a level-one trainer; flavour only.
- Sound: ambience only. No sfx id is cited.

## 6. Links

- Home port: [Typhoon Anchorage](../ports/typhoon_anchorage.md); bloc [The Coral Crown](../factions/coral_crown.md)
- Skill index: [skill-trainers/README.md](README.md)
- [The Storm Wall](../landmarks/storm_wall.md) - the landmark of the same port
- [Leilani Bonecatcher](../style-masters/leilani_bonecatcher.md) - the style master at the same port
- [Patches](../companions/patches.md) - the companion at the same port
- Crew hired in this region: [officers/south_seas.md](../officers/south_seas.md)

## 7. Flags and open questions

- DF-39 (trainer text unread; skill training_port_feature is data only): see ../data-fixes.md. DF-40 (the skill notice prints a raw id).
- Given-name overlap: Shadow Kai (C:5641) and Elder Kai of Ember Isle (C:10891) are different people (DF-37). He teaches to level 1 only (C:5439).
- Wiring trainer text into the desk is out of scope for canon; it is a design decision.
