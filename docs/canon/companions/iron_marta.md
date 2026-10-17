# Iron Marta (iron_marta)

Base commit: 515a488 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at 515a488. Python cites read portlight@9b02494 path:line. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Catalog name | Iron Marta | C:5545 |
| Role | [Marine](roles/marine.md) | C:5546 |
| Personality (data key) | reckless | C:5550 |
| Home port | [Ironhaven](../ports/ironhaven.md) | C:5547 |
| Bloc of the home port | [The Iron Pact](../factions/iron_pact.md) | ports/ironhaven.md section 0 |
| Region | North Atlantic | C:5548 |
| Hire cost | 80 silver | C:5551 |
| Required regional regard | 5 in North Atlantic | C:5552; the shipped row prints it as 'standing 5 in North Atlantic' (logic.rs:2176) |
| Hire gate | docked at the home port; not already in the party; not departed; party under 2; silver enough; regional regard enough | companion.rs:51-93; model.rs:431,439 |
| Starting morale | 70 | companion.rs:95-108 (morale at line 102) |
| Player sees today | offer row (name, role, silver, regard and region), a confirm line, a notice, a party row, and the departure line in the victory recap | logic.rs:2150-2181; logic.rs:1923; game.rs:5748; logic.rs:2129-2149; recap.rs:1085-1093 |
| Not shown | description, greeting, hire dialog, loyalty line, warning line | no reader in crates (section 2) |
| Mechanic status | role effects [flavour, no mechanic] (DF-34); morale reacts to 2 of 13 triggers (DF-35) | companion.rs:193-281; session.rs:2074-2076 |
| Python record | same strings in the Python reference (compared field by field, no differences) | portlight@9b02494 src/portlight/content/companions.py:100 |

## 1. Grounded facts (every data line quoted)

- description: "A former garrison soldier who deserted for the sea. Fights like she has nothing to lose." (C:5549)
- greeting (scene text): "A woman in a soldier's coat sits alone at the bar, sharpening a boarding axe." (C:5553)
- hire dialog: "I've been looking for a captain worth fighting for. Pay's secondary. Show me you're not a coward." (C:5554)
- loyalty line: "Best decision I ever made, signing on with you." (C:5555)
- warning line: "Captain... I'm starting to wonder if this ship is heading anywhere worth going." (C:5556)
- departure line: "I didn't leave the garrison to follow another fool. I'm done." (C:5557)

Quotes are as in content.json; only an em dash is folded to ' - ' and tagged.

## 2. Role in play (what the sim does today; no proposals here)

- Recruit: only at Ironhaven, for 80 silver, with regional regard 5 or more in North Atlantic, a free party slot and a companion who has not already left (companion.rs:51-93). The cost is taken in the session, then morale starts at 70 (session.rs:3359-3373; companion.rs:95-108).
- Regional regard is a per-region number (model.rs:283-287). It rises by 1 on each arrival at a port of that region (reputation.rs:137-141) and moves with trades and inspections (reputation.rs:111, 172); the sources were not traced further here.
- Role effect: Iron Marta is a Marine. Every effect number on the role is authored and unread (roles/marine.md; DF-34). Aboard, this companion changes no price, speed, damage, danger, healing or inspection in the sim.
- Morale that can move today: only after a duel victory, as spared_enemy or took_all (session.rs:2074-2076). For this companion those total +2 (spared) and +1 (took all): role delta plus the reckless personality delta, applied only where the role delta is not zero (companion.rs:130-141; C:5766, C:5776).
- Departure is not reachable today [inferred from companion.rs:130-176]: take-all victories give +1 and spared victories +2, so morale never falls toward the departure line (10 or less) from the two live triggers. The departure line, the loyalty line and the warning line have no live path for this companion (DF-35).
- Text reach: the crew desk shows name, role, cost, required regard and region only (logic.rs:2150-2181); greeting, hire dialog, loyalty line, warning line and description have no reader in the crates. Only the departure line is read (companion.rs:157-176; recap.rs:1085-1093).
- Dismiss: there is no control and no Session method, so a recruited companion stays until they leave (crew_screen.rs:6-7).
- DF-36: after a confirmed recruit the notice reads 'Recruited iron_marta.' with the raw id (game.rs:5748); the confirm line before it uses the catalog name (logic.rs:1923).

## 3. Look and feel (words only; the character hold stands)

- Voice [cite]: short, hard, tested - "Pay's secondary. Show me you're not a coward." (C:5554). The greeting scene is a woman alone at a bar, in a soldier's coat, sharpening a boarding axe (C:5553); quote that as data, add no face, age or build.
- Manner [invented]: she answers a question with a question and never sits with her back to the door.
- Setting [invented]: the bar in the greeting is the Forge & Anchor, the Ironhaven tavern (C:7928); the port's forge smoke and ringing hammers (ports/ironhaven.md section I) sit behind her.
- No face, age, build or dress is set beyond the quoted data lines; character plates are held.

## 4. [invented] lore

- [invented] The garrison she left is Stormwall's, the one the Iron Pact's foundries supply (factions/iron_pact.md; C:13416). Data says only 'the garrison' (C:5557, C:5549); this placement is canon's own (Q4), not data.
- [invented] T38 (README timeline): she walks out of the Stormwall garrison about 26 BS, after No One (T31), Gunnar (T32) and the Iron Wolves (T33), and before the Wolves split from the Tide (T34). Unlike them she takes ship for the sea, not for the Crimson Tide.
- [invented] She reaches Ironhaven because the foundries still feed the garrison she left, and the garrison's old comrades pass through the Pact's docks; she watches the quay from the bar for that reason.
- [invented] She and Gunnar Stahl, the Iron Dock's harbor master, left the same garrison in different years (T32) and do not speak of it.

## 5. Hooks ([proposal] only; never a mechanic)

- [proposal] Copy hook: a one-line journal beat when a captain with her aboard docks at Stormwall - her silence, not a mechanic.
- [proposal] Ambience hook: a whetstone on steel at the bar; no cue id is named or requested.
- Sound: ambience only, no cue named. No sfx id is cited.

## 6. Links

- Home port: [Ironhaven](../ports/ironhaven.md); bloc [The Iron Pact](../factions/iron_pact.md)
- Role: [Marine](roles/marine.md); index: [companions/README.md](README.md)
- Same role: [Red Tomas](red_tomas.md) (Corsair's Rest)
- [Stormwall garrison and the Iron Pact](../factions/iron_pact.md)
- [The Privateer](../captains/privateer.md) - the Stormwall-licensed type (player-types/privateer.md:9); the garrison reading is shared
- [The Iron Wolves](../factions/iron_wolves.md) - deserted the same garrison (T33)
- Crew system with the same role name: [officers/README.md](../officers/README.md)

## 7. Flags and open questions

- DF-34 (role effects unread), DF-35 (morale triggers), DF-36 (raw id in the recruit notice): see ../data-fixes.md.
- Garrison placement is [invented] (Q4 approved); recorded as resolved in lore in ../data-fixes.md.
- Given-name overlap with Marta Soares, Guild Factor at Porto Novo (C:7053): different people (DF-37, resolved in lore).
- Crew role and companion role share a name but are separate systems: resolved in lore (../data-fixes.md).
