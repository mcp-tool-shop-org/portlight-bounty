# Shadow Kai (shadow_kai)

Base commit: 515a488 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at 515a488. Python cites read portlight@9b02494 path:line. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Catalog name | Shadow Kai | C:5641 |
| Role | [Smuggler](roles/smuggler.md) | C:5642 |
| Personality (data key) | pragmatic | C:5646 |
| Home port | [Spice Narrows](../ports/spice_narrows.md) | C:5643 |
| Bloc of the home port | [The Shadow Ports](../factions/shadow_ports.md) | ports/spice_narrows.md section 0 |
| Region | East Indies | C:5644 |
| Hire cost | 90 silver | C:5647 |
| Required regional regard | 5 in East Indies | C:5648; the shipped row prints it as 'standing 5 in East Indies' (logic.rs:2176) |
| Hire gate | docked at the home port; not already in the party; not departed; party under 2; silver enough; regional regard enough | companion.rs:51-93; model.rs:431,439 |
| Starting morale | 70 | companion.rs:95-108 (morale at line 102) |
| Player sees today | offer row (name, role, silver, regard and region), a confirm line, a notice, a party row, and the departure line in the victory recap | logic.rs:2150-2181; logic.rs:1923; game.rs:5748; logic.rs:2129-2149; recap.rs:1085-1093 |
| Not shown | description, greeting, hire dialog | loyalty/warning lines and role description reach the Crew desk (DF-34, DF-35 resolved) |
| Mechanic status | role effects, cohesion, dismiss, and all 13 morale triggers are live (DF-34, DF-35 resolved) | companion.rs:124-280; session.rs |
| Python record | same strings in the Python reference (compared field by field, no differences) | portlight@9b02494 src/portlight/content/companions.py:202 |

## 1. Grounded facts (every data line quoted)

- description: "Nobody knows his real name. He makes contraband disappear and reappear wherever it needs to be." (C:5645)
- greeting (scene text): "A man you didn't notice before is suddenly standing beside you. "Looking for someone discreet?"" (C:5649)
- hire dialog: "I have skills. You have cargo holds. Let's make each other rich. Just don't ask where things go." (C:5650)
- loyalty line: "Good captain. Doesn't ask stupid questions. My kind of arrangement." (C:5651)
- warning line: "You're drawing too much heat. That's bad for business - and for my neck." (C:5652) [ascii-folded]
- departure line: "The arrangement is over. You'll never see me again. That's the point." (C:5653)

Quotes are as in content.json; only an em dash is folded to ' - ' and tagged.

## 2. Role in play (what the sim does today; no proposals here)

- Recruit: only at Spice Narrows, for 90 silver, with regional regard 5 or more in East Indies, a free party slot and a companion who has not already left (companion.rs:51-93). The cost is taken in the session, then morale starts at 70 (session.rs:3359-3373; companion.rs:95-108).
- Regional regard is a per-region number (model.rs:283-287). It rises by 1 on each arrival at a port of that region (reputation.rs:137-141) and moves with trades and inspections (reputation.rs:111, 172); the sources were not traced further here.
- Role effect: Shadow Kai is a Smuggler. Every effect number on the role is live in the sim (DF-34 resolved; formerly authored and unread (roles/smuggler.md; DF-34). Aboard, this companion contributes its role effects to party_combat_bonus (DF-34 resolved).
- Morale triggers: all 13 morale triggers and personality deltas now fire in Session (DF-35 resolved; at the phase 6A base commit, only after a duel victory as spared_enemy or took_all (session.rs:2074-2076). For this companion those total +0 (spared) and +3 (took all): role delta plus the pragmatic personality delta, applied only where the role delta is not zero (companion.rs:130-141; C:5766, C:5776).
- Departure is not reachable today [inferred from companion.rs:130-176]: take-all victories give +3 and spared victories +0, so morale never falls toward the departure line (10 or less) from the two live triggers. The departure line, the loyalty line and the warning line have no live path for this companion (DF-35).
- Text reach: the crew desk shows name, role, cost, required regard and region only (logic.rs:2150-2181); greeting, hire dialog, loyalty line, warning line and description have no reader in the crates. Only the departure line is read (companion.rs:157-176; recap.rs:1085-1093).
- Dismiss: Session::dismiss_companion and the Crew desk Dismiss button remove a companion from the party (DF-34 resolved).
- DF-36 (resolved): the recruit notice now prints the catalog display name (at the base commit it read 'Recruited shadow_kai.' with the raw id (game.rs:5748); the confirm line before it uses the catalog name (logic.rs:1923).

## 3. Look and feel (words only; the character hold stands)

- Voice [cite]: low, transactional - "I have skills. You have cargo holds. Let's make each other rich. Just don't ask where things go." (C:5650). The greeting scene is a man the captain did not notice before, suddenly at their side - "Looking for someone discreet?" (C:5649); quote as data.
- Manner [invented]: he is never the first to name a price or a port.
- Setting [invented]: Mama Smoke's Cave (tavern, C:10624), whose back wall opens on the Contract Wall (broker, C:10654).
- No face, age, build or dress is set beyond the quoted data lines; character plates are held.

## 4. [invented] lore

- [invented] 'Shadow Kai' is a working name; 'nobody knows his real name' (C:5645). Canon reads the name Kai as a borrowed handle and not as kin to Elder Kai of Ember Isle (C:10891) or to Kai the Shaper of Typhoon Anchorage.
- [invented] He works for no syndicate; the Narrows' lord and the Weigher both know his face (C:10604, C:10614) and neither claims him.

## 5. Hooks ([proposal] only; never a mechanic)

- [proposal] Copy hook: his warning line ('drawing too much heat') names a word the game does not show players (Option A); a later copy brief would rephrase before any use.
- Sound: ambience only, no cue named. No sfx id is cited.

## 6. Links

- Home port: [Spice Narrows](../ports/spice_narrows.md); bloc [The Shadow Ports](../factions/shadow_ports.md)
- Role: [Smuggler](roles/smuggler.md); index: [companions/README.md](README.md)
- Same role: [Rosa the Fence](rosa_the_fence.md) (Corsair's Rest)
- [The Shadow Ports](../factions/shadow_ports.md)
- [The Monsoon Syndicate](../factions/monsoon_syndicate.md)
- [The Smuggler](../captains/smuggler.md)
- [The Smuggler's Run](../lanes/smugglers_run.md)
- Crew system with the same role name: [officers/README.md](../officers/README.md)

## 7. Flags and open questions

- DF-34 (resolved: role effects, cohesion, party summary, and dismiss wired), DF-35 (resolved: all 13 morale triggers and loyalty/warning lines wired), DF-36 (resolved: catalog name in recruit notice): see ../data-fixes.md.
- Given-name overlap with Elder Kai (C:10891) and Kai the Shaper (C:5436): DF-37, resolved in lore.
- warning_line uses 'heat' (C:5652), a term canon does not use as a player word (Option A); the line reaches the Crew desk when morale <= 29 (DF-35 resolved).
- Crew role and companion role share a name but are separate systems: resolved in lore (../data-fixes.md).
