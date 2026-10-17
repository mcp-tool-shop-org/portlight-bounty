# Patches (patches)

Base commit: 515a488 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at 515a488. Python cites read portlight@9b02494 path:line. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Catalog name | Patches | C:5625 |
| Role | [Surgeon](roles/surgeon.md) | C:5626 |
| Personality (data key) | reckless | C:5630 |
| Home port | [Typhoon Anchorage](../ports/typhoon_anchorage.md) | C:5627 |
| Bloc of the home port | [The Coral Crown](../factions/coral_crown.md) | ports/typhoon_anchorage.md section 0 |
| Region | South Seas | C:5628 |
| Hire cost | 50 silver | C:5631 |
| Required regional regard | 0 in South Seas | C:5632; the shipped row prints it as 'standing 0 in South Seas' (logic.rs:2176) |
| Hire gate | docked at the home port; not already in the party; not departed; party under 2; silver enough; regional regard enough | companion.rs:51-93; model.rs:431,439 |
| Starting morale | 70 | companion.rs:95-108 (morale at line 102) |
| Player sees today | offer row (name, role, silver, regard and region), a confirm line, a notice, a party row, and the departure line in the victory recap | logic.rs:2150-2181; logic.rs:1923; game.rs:5748; logic.rs:2129-2149; recap.rs:1085-1093 |
| Not shown | description, greeting, hire dialog, loyalty line, warning line | no reader in crates (section 2) |
| Mechanic status | role effects [flavour, no mechanic] (DF-34); morale reacts to 2 of 13 triggers (DF-35) | companion.rs:193-281; session.rs:2074-2076 |
| Python record | same strings in the Python reference (compared field by field, no differences) | portlight@9b02494 src/portlight/content/companions.py:184 |

## 1. Grounded facts (every data line quoted)

- description: "A ship's surgeon who earned his nickname from the number of wounds he's stitched. Drinks too much, but his hands are steady." (C:5629)
- greeting (scene text): "A man with rum-stained fingers and a surgeon's kit sits at the bar. He's stitching his own thumb." (C:5633)
- hire dialog: "I can keep your crew alive. Mostly. I charge cheap because I drink the medicine budget. Fair warning." (C:5634)
- loyalty line: "Best gig I've had in years. The rum's flowing and nobody's dying." (C:5635)
- warning line: "I'm a doctor, not a miracle worker. This ship needs better luck - or a better captain." (C:5636) [ascii-folded]
- departure line: "I'm finding a quieter ship. One where I can drink in peace." (C:5637)

Quotes are as in content.json; only an em dash is folded to ' - ' and tagged.

## 2. Role in play (what the sim does today; no proposals here)

- Recruit: only at Typhoon Anchorage, for 50 silver, with regional regard 0 or more in South Seas, a free party slot and a companion who has not already left (companion.rs:51-93). The cost is taken in the session, then morale starts at 70 (session.rs:3359-3373; companion.rs:95-108).
- Regional regard is a per-region number (model.rs:283-287). It rises by 1 on each arrival at a port of that region (reputation.rs:137-141) and moves with trades and inspections (reputation.rs:111, 172); the sources were not traced further here.
- Role effect: Patches is a Surgeon. Every effect number on the role is authored and unread (roles/surgeon.md; DF-34). Aboard, this companion changes no price, speed, damage, danger, healing or inspection in the sim.
- Morale that can move today: only after a duel victory, as spared_enemy or took_all (session.rs:2074-2076). For this companion those total +5 (spared) and -3 (took all): role delta plus the reckless personality delta, applied only where the role delta is not zero (companion.rs:130-141; C:5766, C:5776).
- Departure is reachable [inferred from companion.rs:130-176]: from 70, take-all victories alone cut morale by 3 each, so after 20 of them in a row morale reaches 10 and the companion leaves (departure at 10 or less, companion.rs:157-176); a spared victory adds +5. A departed companion cannot be hired again (companion.rs:66-70). The departure line is the one authored text of this companion that a player can read (recap.rs:1085-1093).
- Text reach: the crew desk shows name, role, cost, required regard and region only (logic.rs:2150-2181); greeting, hire dialog, loyalty line, warning line and description have no reader in the crates. Only the departure line is read (companion.rs:157-176; recap.rs:1085-1093).
- Dismiss: there is no control and no Session method, so a recruited companion stays until they leave (crew_screen.rs:6-7).
- DF-36: after a confirmed recruit the notice reads 'Recruited patches.' with the raw id (game.rs:5748); the confirm line before it uses the catalog name (logic.rs:1923).

## 3. Look and feel (words only; the character hold stands)

- Voice [cite]: rueful, honest about his habit - "I charge cheap because I drink the medicine budget. Fair warning." (C:5634). The greeting scene is a man with rum-stained fingers and a surgeon's kit at the bar, stitching his own thumb (C:5633); quote as data.
- Manner [invented]: he warns first and works second.
- Setting [invented]: Hine's Storm Grill (tavern, C:11054; Cook Hine), whose bar is the nearest to The Wreck Yard (shipyard, C:11064).
- No face, age, build or dress is set beyond the quoted data lines; character plates are held.

## 4. [invented] lore

- [invented] His kit is made up of salvage from the hulls the Storm Wall is built from (T20, landmarks/storm_wall.md): needles from ship fittings, thread from rigging.
- [invented] The 'number of wounds he's stitched' (C:5629) is a tally he does not keep; the nickname was given by sailors who did.

## 5. Hooks ([proposal] only; never a mechanic)

- [proposal] Copy hook: a morale line on the party row that reads as banter when he is the surgeon - flavour only.
- Sound: ambience only, no cue named. No sfx id is cited.

## 6. Links

- Home port: [Typhoon Anchorage](../ports/typhoon_anchorage.md); bloc [The Coral Crown](../factions/coral_crown.md)
- Role: [Surgeon](roles/surgeon.md); index: [companions/README.md](README.md)
- Same role: [Dr. Amara](dr_amara.md) (Sun Harbor)
- [The Coral Crown](../factions/coral_crown.md)
- [The Storm Wall](../landmarks/storm_wall.md)
- Crew system with the same role name: [officers/README.md](../officers/README.md)

## 7. Flags and open questions

- DF-34 (role effects unread), DF-35 (morale triggers), DF-36 (raw id in the recruit notice): see ../data-fixes.md.
- Crew role and companion role share a name but are separate systems: resolved in lore (../data-fixes.md).
