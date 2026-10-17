# Rosa the Fence (rosa_the_fence)

Base commit: 515a488 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at 515a488. Python cites read portlight@9b02494 path:line. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Catalog name | Rosa the Fence | C:5657 |
| Role | [Smuggler](roles/smuggler.md) | C:5658 |
| Personality (data key) | pragmatic | C:5662 |
| Home port | [Corsair's Rest](../ports/corsairs_rest.md) | C:5659 |
| Bloc of the home port | [The Shadow Ports](../factions/shadow_ports.md) | ports/corsairs_rest.md section 0 |
| Region | Mediterranean | C:5660 |
| Hire cost | 70 silver | C:5663 |
| Required regional regard | 0 in Mediterranean | C:5664; the shipped row prints it as 'standing 0 in Mediterranean' (logic.rs:2176) |
| Hire gate | docked at the home port; not already in the party; not departed; party under 2; silver enough; regional regard enough | companion.rs:51-93; model.rs:431,439 |
| Starting morale | 70 | companion.rs:95-108 (morale at line 102) |
| Player sees today | offer row (name, role, silver, regard and region), a confirm line, a notice, a party row, and the departure line in the victory recap | logic.rs:2150-2181; logic.rs:1923; game.rs:5748; logic.rs:2129-2149; recap.rs:1085-1093 |
| Not shown | description, greeting, hire dialog, loyalty line, warning line | no reader in crates (section 2) |
| Mechanic status | role effects [flavour, no mechanic] (DF-34); morale reacts to 2 of 13 triggers (DF-35) | companion.rs:193-281; session.rs:2074-2076 |
| Python record | same strings in the Python reference (compared field by field, no differences) | portlight@9b02494 src/portlight/content/companions.py:218 |

## 1. Grounded facts (every data line quoted)

- description: "A woman who knows the price of everything and the customs schedule of every port." (C:5661)
- greeting (scene text): "A woman with too many rings counts coins at a corner table. She's been watching you since you walked in." (C:5665)
- hire dialog: "Every captain needs someone who knows where the inspectors aren't. That's me. Fifty-fifty on the extras." (C:5666)
- loyalty line: "We're making good silver together. Long may it last." (C:5667)
- warning line: "The margins are getting thin and the heat's getting thick. Watch yourself." (C:5668)
- departure line: "Bad for business, Captain. I'm moving on before your luck runs out on both of us." (C:5669)

Quotes are as in content.json; only an em dash is folded to ' - ' and tagged.

## 2. Role in play (what the sim does today; no proposals here)

- Recruit: only at Corsair's Rest, for 70 silver, with regional regard 0 or more in Mediterranean, a free party slot and a companion who has not already left (companion.rs:51-93). The cost is taken in the session, then morale starts at 70 (session.rs:3359-3373; companion.rs:95-108).
- Regional regard is a per-region number (model.rs:283-287). It rises by 1 on each arrival at a port of that region (reputation.rs:137-141) and moves with trades and inspections (reputation.rs:111, 172); the sources were not traced further here.
- Role effect: Rosa the Fence is a Smuggler. Every effect number on the role is authored and unread (roles/smuggler.md; DF-34). Aboard, this companion changes no price, speed, damage, danger, healing or inspection in the sim.
- Morale that can move today: only after a duel victory, as spared_enemy or took_all (session.rs:2074-2076). For this companion those total +0 (spared) and +3 (took all): role delta plus the pragmatic personality delta, applied only where the role delta is not zero (companion.rs:130-141; C:5766, C:5776).
- Departure is not reachable today [inferred from companion.rs:130-176]: take-all victories give +3 and spared victories +0, so morale never falls toward the departure line (10 or less) from the two live triggers. The departure line, the loyalty line and the warning line have no live path for this companion (DF-35).
- Text reach: the crew desk shows name, role, cost, required regard and region only (logic.rs:2150-2181); greeting, hire dialog, loyalty line, warning line and description have no reader in the crates. Only the departure line is read (companion.rs:157-176; recap.rs:1085-1093).
- Dismiss: there is no control and no Session method, so a recruited companion stays until they leave (crew_screen.rs:6-7).
- DF-36: after a confirmed recruit the notice reads 'Recruited rosa_the_fence.' with the raw id (game.rs:5748); the confirm line before it uses the catalog name (logic.rs:1923).

## 3. Look and feel (words only; the character hold stands)

- Voice [cite]: brisk, mercantile - "Fifty-fifty on the extras." (C:5666). The greeting scene is a woman with too many rings counting coins at a corner table, watching the captain since they walked in (C:5665); quote as data.
- Manner [invented]: she names a number before she says hello.
- Setting [invented]: a table near the Whisper Market (exchange, C:7664), where Whisper keeps the prices.
- No face, age, build or dress is set beyond the quoted data lines; character plates are held.

## 4. [invented] lore

- [invented] She is a receiver of goods that arrive without papers; the data has a good for that trade (Stolen Cargo, C:117), and she is the person a captain would ask about it.
- [invented] She and Whisper (price keeper, C:7664) trade price news and do not trade goods.
- [invented] Her 'customs schedule of every port' (C:5661) is a notebook, not a network; the notebook is why the brokers of the cove keep their own copies.

## 5. Hooks ([proposal] only; never a mechanic)

- [proposal] Copy hook: her loyalty line is the one that could read as a trade receipt if journal copy ever names profit - flavour only.
- Sound: ambience only, no cue named. No sfx id is cited.

## 6. Links

- Home port: [Corsair's Rest](../ports/corsairs_rest.md); bloc [The Shadow Ports](../factions/shadow_ports.md)
- Role: [Smuggler](roles/smuggler.md); index: [companions/README.md](README.md)
- Same role: [Shadow Kai](shadow_kai.md) (Spice Narrows)
- Same port: [Red Tomas](red_tomas.md)
- [The Shadow Ports](../factions/shadow_ports.md)
- [The Crimson Tide](../factions/crimson_tide.md)
- [The Smuggler](../captains/smuggler.md)
- Crew system with the same role name: [officers/README.md](../officers/README.md)

## 7. Flags and open questions

- DF-34 (role effects unread), DF-35 (morale triggers), DF-36 (raw id in the recruit notice): see ../data-fixes.md.
- Given-name overlap with Rosa Carvalho at Silva Bay (C:7581): DF-37, resolved in lore.
- Shares Corsair's Rest with Red Tomas (the only port with two companions).
- Crew role and companion role share a name but are separate systems: resolved in lore (../data-fixes.md).
