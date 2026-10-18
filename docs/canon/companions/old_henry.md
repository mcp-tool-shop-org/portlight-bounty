# Old Henry (old_henry)

Base commit: 515a488 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at 515a488. Python cites read portlight@9b02494 path:line. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Catalog name | Old Henry | C:5593 |
| Role | [Navigator](roles/navigator.md) | C:5594 |
| Personality (data key) | pragmatic | C:5598 |
| Home port | [Thornport](../ports/thornport.md) | C:5595 |
| Bloc of the home port | [The Free Ports](../factions/free_ports.md) | ports/thornport.md section 0 |
| Region | North Atlantic | C:5596 |
| Hire cost | 70 silver | C:5599 |
| Required regional regard | 5 in North Atlantic | C:5600; the shipped row prints it as 'standing 5 in North Atlantic' (logic.rs:2176) |
| Hire gate | docked at the home port; not already in the party; not departed; party under 2; silver enough; regional regard enough | companion.rs:51-93; model.rs:431,439 |
| Starting morale | 70 | companion.rs:95-108 (morale at line 102) |
| Player sees today | offer row (name, role, silver, regard and region), a confirm line, a notice, a party row, and the departure line in the victory recap | logic.rs:2150-2181; logic.rs:1923; game.rs:5748; logic.rs:2129-2149; recap.rs:1085-1093 |
| Not shown | description, greeting, hire dialog | loyalty/warning lines and role description reach the Crew desk (DF-34, DF-35 resolved) |
| Mechanic status | role effects, cohesion, dismiss, and all 13 morale triggers are live (DF-34, DF-35 resolved) | companion.rs:124-280; session.rs |
| Python record | same strings in the Python reference (compared field by field, no differences) | portlight@9b02494 src/portlight/content/companions.py:150 |

## 1. Grounded facts (every data line quoted)

- description: "Fifty years at sea. He's sailed every route twice and remembers every reef." (C:5597)
- greeting (scene text): "An old man mends nets by the harbor, humming a tune you don't recognize. His eyes are sharp." (C:5601)
- hire dialog: "I'm too old to captain my own ship, but I'm not too old to keep yours off the rocks. Deal?" (C:5602)
- loyalty line: "Reminds me of the old days. Good captain, good ship, good sea." (C:5603)
- warning line: "I've seen captains make these mistakes before. It doesn't end well." (C:5604)
- departure line: "I'm retiring. For real this time. Good luck, Captain." (C:5605)

Quotes are as in content.json; only an em dash is folded to ' - ' and tagged.

## 2. Role in play (what the sim does today; no proposals here)

- Recruit: only at Thornport, for 70 silver, with regional regard 5 or more in North Atlantic, a free party slot and a companion who has not already left (companion.rs:51-93). The cost is taken in the session, then morale starts at 70 (session.rs:3359-3373; companion.rs:95-108).
- Regional regard is a per-region number (model.rs:283-287). It rises by 1 on each arrival at a port of that region (reputation.rs:137-141) and moves with trades and inspections (reputation.rs:111, 172); the sources were not traced further here.
- Role effect: Old Henry is a Navigator. Every effect number on the role is live in the sim (DF-34 resolved; formerly authored and unread (roles/navigator.md; DF-34). Aboard, this companion contributes its role effects to party_combat_bonus (DF-34 resolved).
- Morale triggers: all 13 morale triggers and personality deltas now fire in Session (DF-35 resolved; at the phase 6A base commit, only after a duel victory as spared_enemy or took_all (session.rs:2074-2076). For this companion those total +1 (spared) and -1 (took all): role delta plus the pragmatic personality delta, applied only where the role delta is not zero (companion.rs:130-141; C:5766, C:5776).
- Departure is reachable [inferred from companion.rs:130-176]: from 70, take-all victories alone cut morale by 1 each, so after 60 of them in a row morale reaches 10 and the companion leaves (departure at 10 or less, companion.rs:157-176); a spared victory adds +1. A departed companion cannot be hired again (companion.rs:66-70). The departure line is the one authored text of this companion that a player can read (recap.rs:1085-1093).
- Text reach: the crew desk shows name, role, cost, required regard and region only (logic.rs:2150-2181); greeting, hire dialog, loyalty line, warning line and description have no reader in the crates. Only the departure line is read (companion.rs:157-176; recap.rs:1085-1093).
- Dismiss: Session::dismiss_companion and the Crew desk Dismiss button remove a companion from the party (DF-34 resolved).
- DF-36 (resolved): the recruit notice now prints the catalog display name (at the base commit it read 'Recruited old_henry.' with the raw id (game.rs:5748); the confirm line before it uses the catalog name (logic.rs:1923).

## 3. Look and feel (words only; the character hold stands)

- Voice [cite]: dry, unhurried - "I'm too old to captain my own ship, but I'm not too old to keep yours off the rocks. Deal?" (C:5602). The greeting scene is an old man mending nets by the harbor, humming a tune the captain does not recognise, eyes sharp (C:5601).
- Manner [invented]: he finishes the knot before he looks up.
- Setting [invented]: the Whale Dock (harbor_master, C:8328); the net-mending bench is in the lee of the Whale Bone (tavern, C:8358).
- No face, age, build or dress is set beyond the quoted data lines; character plates are held.

## 4. [invented] lore

- [invented] The tune is a whaling work song from the Thornport whaling era (T9); he learned it from men who are now names on the Whale Arch (T19).
- [invented] 'Every reef' (C:5597) is a habit of the trade: he will name the reef that sank the last ship that ignored it.

## 5. Hooks ([proposal] only; never a mechanic)

- [proposal] Copy hook: a line on the party row when the captain docks at a lane end he has sailed 'twice' - flavour only.
- Sound: ambience only, no cue named. No sfx id is cited.

## 6. Links

- Home port: [Thornport](../ports/thornport.md); bloc [The Free Ports](../factions/free_ports.md)
- Role: [Navigator](roles/navigator.md); index: [companions/README.md](README.md)
- Same role: [Star Lila](star_lila.md) (Crosswind Isle)
- [The Free Ports](../factions/free_ports.md)
- [The Whale Arch](../landmarks/whale_arch.md)
- [The Navigator](../captains/navigator.md)
- Crew system with the same role name: [officers/README.md](../officers/README.md)

## 7. Flags and open questions

- DF-34 (resolved: role effects, cohesion, party summary, and dismiss wired), DF-35 (resolved: all 13 morale triggers and loyalty/warning lines wired), DF-36 (resolved: catalog name in recruit notice): see ../data-fixes.md.
- Crew role and companion role share a name but are separate systems: resolved in lore (../data-fixes.md).
