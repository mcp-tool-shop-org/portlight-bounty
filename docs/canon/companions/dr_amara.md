# Dr. Amara (dr_amara)

Base commit: 515a488 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at 515a488. Python cites read portlight@9b02494 path:line. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Catalog name | Dr. Amara | C:5609 |
| Role | [Surgeon](roles/surgeon.md) | C:5610 |
| Personality (data key) | gentle | C:5614 |
| Home port | [Sun Harbor](../ports/sun_harbor.md) | C:5611 |
| Bloc of the home port | [The Gold Coast Compact](../factions/gold_coast_compact.md) | ports/sun_harbor.md section 0 |
| Region | West Africa | C:5612 |
| Hire cost | 120 silver | C:5615 |
| Required regional regard | 10 in West Africa | C:5616; the shipped row prints it as 'standing 10 in West Africa' (logic.rs:2176) |
| Hire gate | docked at the home port; not already in the party; not departed; party under 2; silver enough; regional regard enough | companion.rs:51-93; model.rs:431,439 |
| Starting morale | 70 | companion.rs:95-108 (morale at line 102) |
| Player sees today | offer row (name, role, silver, regard and region), a confirm line, a notice, a party row, and the departure line in the victory recap | logic.rs:2150-2181; logic.rs:1923; game.rs:5748; logic.rs:2129-2149; recap.rs:1085-1093 |
| Not shown | description, greeting, hire dialog, loyalty line, warning line | no reader in crates (section 2) |
| Mechanic status | role effects [flavour, no mechanic] (DF-34); morale reacts to 2 of 13 triggers (DF-35) | companion.rs:193-281; session.rs:2074-2076 |
| Python record | same strings in the Python reference (compared field by field, no differences) | portlight@9b02494 src/portlight/content/companions.py:168 |

## 1. Grounded facts (every data line quoted)

- description: "A healer trained in both Western and African medicine. She's saved more lives than most doctors dream of." (C:5613)
- greeting (scene text): "A woman in a clean apron tends to a sailor's wound at the dockside clinic. Efficient, gentle hands." (C:5617)
- hire dialog: "I go where people need healing. Your crew looks like they could use it. I have conditions - no unnecessary cruelty." (C:5618) [ascii-folded]
- loyalty line: "Your crew trusts you. That's the best medicine there is." (C:5619)
- warning line: "I became a healer to save lives, Captain. Not to watch them wasted." (C:5620)
- departure line: "I cannot stay on a ship that causes more harm than it heals. Goodbye, Captain." (C:5621)

Quotes are as in content.json; only an em dash is folded to ' - ' and tagged.

## 2. Role in play (what the sim does today; no proposals here)

- Recruit: only at Sun Harbor, for 120 silver, with regional regard 10 or more in West Africa, a free party slot and a companion who has not already left (companion.rs:51-93). The cost is taken in the session, then morale starts at 70 (session.rs:3359-3373; companion.rs:95-108).
- Regional regard is a per-region number (model.rs:283-287). It rises by 1 on each arrival at a port of that region (reputation.rs:137-141) and moves with trades and inspections (reputation.rs:111, 172); the sources were not traced further here.
- Role effect: Dr. Amara is a Surgeon. Every effect number on the role is authored and unread (roles/surgeon.md; DF-34). Aboard, this companion changes no price, speed, damage, danger, healing or inspection in the sim.
- Morale that can move today: only after a duel victory, as spared_enemy or took_all (session.rs:2074-2076). For this companion those total +8 (spared) and -6 (took all): role delta plus the gentle personality delta, applied only where the role delta is not zero (companion.rs:130-141; C:5766, C:5776).
- Departure is reachable [inferred from companion.rs:130-176]: from 70, take-all victories alone cut morale by 6 each, so after 10 of them in a row morale reaches 10 and the companion leaves (departure at 10 or less, companion.rs:157-176); a spared victory adds +8. A departed companion cannot be hired again (companion.rs:66-70). The departure line is the one authored text of this companion that a player can read (recap.rs:1085-1093).
- Text reach: the crew desk shows name, role, cost, required regard and region only (logic.rs:2150-2181); greeting, hire dialog, loyalty line, warning line and description have no reader in the crates. Only the departure line is read (companion.rs:157-176; recap.rs:1085-1093).
- Dismiss: there is no control and no Session method, so a recruited companion stays until they leave (crew_screen.rs:6-7).
- DF-36: after a confirmed recruit the notice reads 'Recruited dr_amara.' with the raw id (game.rs:5748); the confirm line before it uses the catalog name (logic.rs:1923).

## 3. Look and feel (words only; the character hold stands)

- Voice [cite]: calm, conditional - "I go where people need healing... I have conditions - no unnecessary cruelty." (C:5618). The greeting scene is a woman in a clean apron tending a sailor's wound at a dockside clinic, efficient, gentle hands (C:5617); quote as data.
- Manner [invented]: she asks what happened before she asks who did it.
- Setting [invented]: Sun Harbor has no apothecary institution in the data (institution types at the port: harbor_master, exchange, governor, shipyard, tavern, customs, broker; C:8562-8622), so the 'dockside clinic' is a room by the Welcome Dock (harbor_master, C:8562) that canon adds as lore, not as a building.
- No face, age, build or dress is set beyond the quoted data lines; character plates are held.

## 4. [invented] lore

- [invented] Her 'conditions' are the reason a gentle captain keeps her: a captain who takes everything (took_all) costs her morale faster than any other companion in the data (see section 2).
- [invented] She and Inspector Amara of the Honest Scale (customs, C:8612) share a given name and are two people. Canon makes no kinship.
- [cite] 'Western and African medicine' is quoted as data (C:5613) and not extended.

## 5. Hooks ([proposal] only; never a mechanic)

- [proposal] Copy hook: the departure line is the one authored text a player can read today (recap); a later journal beat could name what caused it - flavour only.
- Sound: ambience only, no cue named. No sfx id is cited.

## 6. Links

- Home port: [Sun Harbor](../ports/sun_harbor.md); bloc [The Gold Coast Compact](../factions/gold_coast_compact.md)
- Role: [Surgeon](roles/surgeon.md); index: [companions/README.md](README.md)
- Same role: [Patches](patches.md) (Typhoon Anchorage)
- [The Gold Coast Compact](../factions/gold_coast_compact.md)
- [The Honest Scale / Inspector Amara](../ports/sun_harbor.md)
- Crew system with the same role name: [officers/README.md](../officers/README.md)

## 7. Flags and open questions

- DF-34 (role effects unread), DF-35 (morale triggers), DF-36 (raw id in the recruit notice): see ../data-fixes.md.
- Same-port name collision with Inspector Amara (C:8745): DF-37, resolved in lore until fixed.
- Crew role and companion role share a name but are separate systems: resolved in lore (../data-fixes.md).
