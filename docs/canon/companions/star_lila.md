# Star Lila (star_lila)

Base commit: 515a488 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at 515a488. Python cites read portlight@9b02494 path:line. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Catalog name | Star Lila | C:5577 |
| Role | [Navigator](roles/navigator.md) | C:5578 |
| Personality (data key) | gentle | C:5582 |
| Home port | [Crosswind Isle](../ports/crosswind_isle.md) | C:5579 |
| Bloc of the home port | [The Free Ports](../factions/free_ports.md) | ports/crosswind_isle.md section 0 |
| Region | East Indies | C:5580 |
| Hire cost | 100 silver | C:5583 |
| Required regional regard | 10 in East Indies | C:5584; the shipped row prints it as 'standing 10 in East Indies' (logic.rs:2176) |
| Hire gate | docked at the home port; not already in the party; not departed; party under 2; silver enough; regional regard enough | companion.rs:51-93; model.rs:431,439 |
| Starting morale | 70 | companion.rs:95-108 (morale at line 102) |
| Player sees today | offer row (name, role, silver, regard and region), a confirm line, a notice, a party row, and the departure line in the victory recap | logic.rs:2150-2181; logic.rs:1923; game.rs:5748; logic.rs:2129-2149; recap.rs:1085-1093 |
| Not shown | description, greeting, hire dialog | loyalty/warning lines and role description reach the Crew desk (DF-34, DF-35 resolved) |
| Mechanic status | role effects, cohesion, dismiss, and all 13 morale triggers are live (DF-34, DF-35 resolved) | companion.rs:124-280; session.rs |
| Python record | same strings in the Python reference (compared field by field, no differences) | portlight@9b02494 src/portlight/content/companions.py:134 |

## 1. Grounded facts (every data line quoted)

- description: "A Javanese navigator who reads the stars like others read books. She's never been lost." (C:5581)
- greeting (scene text): "A young woman sits on the dock, sketching star charts on palm bark with quiet concentration." (C:5585)
- hire dialog: "The stars brought me here. Perhaps they brought you too. I'll navigate - you decide where." (C:5586) [ascii-folded]
- loyalty line: "The stars say we're on the right path. I believe them." (C:5587)
- warning line: "The currents are wrong, Captain. Not the sea - this ship's course." (C:5588) [ascii-folded]
- departure line: "The stars say it's time to part. I wish you fair winds." (C:5589)

Quotes are as in content.json; only an em dash is folded to ' - ' and tagged.

## 2. Role in play (what the sim does today; no proposals here)

- Recruit: only at Crosswind Isle, for 100 silver, with regional regard 10 or more in East Indies, a free party slot and a companion who has not already left (companion.rs:51-93). The cost is taken in the session, then morale starts at 70 (session.rs:3359-3373; companion.rs:95-108).
- Regional regard is a per-region number (model.rs:283-287). It rises by 1 on each arrival at a port of that region (reputation.rs:137-141) and moves with trades and inspections (reputation.rs:111, 172); the sources were not traced further here.
- Role effect: Star Lila is a Navigator. Every effect number on the role is live in the sim (DF-34 resolved; formerly authored and unread (roles/navigator.md; DF-34). Aboard, this companion contributes its role effects to party_combat_bonus (DF-34 resolved).
- Morale triggers: all 13 morale triggers and personality deltas now fire in Session (DF-35 resolved; at the phase 6A base commit, only after a duel victory as spared_enemy or took_all (session.rs:2074-2076). For this companion those total +4 (spared) and -4 (took all): role delta plus the gentle personality delta, applied only where the role delta is not zero (companion.rs:130-141; C:5766, C:5776).
- Departure is reachable [inferred from companion.rs:130-176]: from 70, take-all victories alone cut morale by 4 each, so after 15 of them in a row morale reaches 10 and the companion leaves (departure at 10 or less, companion.rs:157-176); a spared victory adds +4. A departed companion cannot be hired again (companion.rs:66-70). The departure line is the one authored text of this companion that a player can read (recap.rs:1085-1093).
- Text reach: the crew desk shows name, role, cost, required regard and region only (logic.rs:2150-2181); greeting, hire dialog, loyalty line, warning line and description have no reader in the crates. Only the departure line is read (companion.rs:157-176; recap.rs:1085-1093).
- Dismiss: Session::dismiss_companion and the Crew desk Dismiss button remove a companion from the party (DF-34 resolved).
- DF-36 (resolved): the recruit notice now prints the catalog display name (at the base commit it read 'Recruited star_lila.' with the raw id (game.rs:5748); the confirm line before it uses the catalog name (logic.rs:1923).

## 3. Look and feel (words only; the character hold stands)

- Voice [cite]: soft, certain, a little formal - "The stars brought me here. Perhaps they brought you too. I'll navigate - you decide where." (C:5586). The greeting scene is a young woman on the dock sketching star charts on palm bark (C:5585); quote as data, add no more.
- Manner [invented]: she answers with a bearing, not a promise.
- Setting [invented]: the Free Dock (harbor_master, C:10126), where Dock Master Tao keeps the berths; the bark charts dry on the rail of the Bell and Board (tavern, C:10156).
- No face, age, build or dress is set beyond the quoted data lines; character plates are held.

## 4. [invented] lore

- [invented] Her charts are copies of copies of Old Farouk's Monsoon Shortcut charts (T27), with her own corrections in the margin; she has not been lost on the lane (lanes/monsoon_shortcut.md).
- [invented] She came to Crosswind Isle because a free port asks no one where they learned their stars.
- [cite] 'Javanese' is quoted as data (C:5581) and not extended; canon invents no real-world place.

## 5. Hooks ([proposal] only; never a mechanic)

- [proposal] Copy hook: a chart-screen line when she is aboard and the lane is the Monsoon Shortcut - flavour, not a speed rule.
- Sound: ambience only, no cue named. No sfx id is cited.

## 6. Links

- Home port: [Crosswind Isle](../ports/crosswind_isle.md); bloc [The Free Ports](../factions/free_ports.md)
- Role: [Navigator](roles/navigator.md); index: [companions/README.md](README.md)
- Same role: [Old Henry](old_henry.md) (Thornport)
- [The Free Ports](../factions/free_ports.md)
- [The Monsoon Shortcut](../lanes/monsoon_shortcut.md)
- [The Navigator](../captains/navigator.md) - the type that shares her trade
- Crew system with the same role name: [officers/README.md](../officers/README.md)

## 7. Flags and open questions

- DF-34 (resolved: role effects, cohesion, party summary, and dismiss wired), DF-35 (resolved: all 13 morale triggers and loyalty/warning lines wired), DF-36 (resolved: catalog name in recruit notice): see ../data-fixes.md.
- Crewmate-role overlap: she is a Navigator companion; a hired Navigator is a different record (resolved in lore, ../README.md).
- Crew role and companion role share a name but are separate systems: resolved in lore (../data-fixes.md).
