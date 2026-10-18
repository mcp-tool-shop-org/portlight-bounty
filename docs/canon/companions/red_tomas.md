# Red Tomas (red_tomas)

Base commit: 515a488 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at 515a488. Python cites read portlight@9b02494 path:line. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Catalog name | Red Tomas | C:5561 |
| Role | [Marine](roles/marine.md) | C:5562 |
| Personality (data key) | pragmatic | C:5566 |
| Home port | [Corsair's Rest](../ports/corsairs_rest.md) | C:5563 |
| Bloc of the home port | [The Shadow Ports](../factions/shadow_ports.md) | ports/corsairs_rest.md section 0 |
| Region | Mediterranean | C:5564 |
| Hire cost | 60 silver | C:5567 |
| Required regional regard | 0 in Mediterranean | C:5568; the shipped row prints it as 'standing 0 in Mediterranean' (logic.rs:2176) |
| Hire gate | docked at the home port; not already in the party; not departed; party under 2; silver enough; regional regard enough | companion.rs:51-93; model.rs:431,439 |
| Starting morale | 70 | companion.rs:95-108 (morale at line 102) |
| Player sees today | offer row (name, role, silver, regard and region), a confirm line, a notice, a party row, and the departure line in the victory recap | logic.rs:2150-2181; logic.rs:1923; game.rs:5748; logic.rs:2129-2149; recap.rs:1085-1093 |
| Not shown | description, greeting, hire dialog | loyalty/warning lines and role description reach the Crew desk (DF-34, DF-35 resolved) |
| Mechanic status | role effects, cohesion, dismiss, and all 13 morale triggers are live (DF-34, DF-35 resolved) | companion.rs:124-280; session.rs |
| Python record | same strings in the Python reference (compared field by field, no differences) | portlight@9b02494 src/portlight/content/companions.py:116 |

## 1. Grounded facts (every data line quoted)

- description: "A pirate-turned-bodyguard with a face full of scars and a code of honor." (C:5565)
- greeting (scene text): "A scarred man watches you from the corner. His hand never leaves the cutlass at his belt." (C:5569)
- hire dialog: "I protect. You pay. Simple arrangement. Betray me and you'll wish the pirates found you first." (C:5570)
- loyalty line: "You're straight with me, Captain. That's rare on these waters." (C:5571)
- warning line: "This isn't what I signed up for. We need to talk." (C:5572)
- departure line: "The deal's off. Don't look for me." (C:5573)

Quotes are as in content.json; only an em dash is folded to ' - ' and tagged.

## 2. Role in play (what the sim does today; no proposals here)

- Recruit: only at Corsair's Rest, for 60 silver, with regional regard 0 or more in Mediterranean, a free party slot and a companion who has not already left (companion.rs:51-93). The cost is taken in the session, then morale starts at 70 (session.rs:3359-3373; companion.rs:95-108).
- Regional regard is a per-region number (model.rs:283-287). It rises by 1 on each arrival at a port of that region (reputation.rs:137-141) and moves with trades and inspections (reputation.rs:111, 172); the sources were not traced further here.
- Role effect: Red Tomas is a Marine. Every effect number on the role is live in the sim (DF-34 resolved; formerly authored and unread (roles/marine.md; DF-34). Aboard, this companion contributes its role effects to party_combat_bonus (DF-34 resolved).
- Morale triggers: all 13 morale triggers and personality deltas now fire in Session (DF-35 resolved; at the phase 6A base commit, only after a duel victory as spared_enemy or took_all (session.rs:2074-2076). For this companion those total +2 (spared) and +1 (took all): role delta plus the pragmatic personality delta, applied only where the role delta is not zero (companion.rs:130-141; C:5766, C:5776).
- Departure is not reachable today [inferred from companion.rs:130-176]: take-all victories give +1 and spared victories +2, so morale never falls toward the departure line (10 or less) from the two live triggers. The departure line, the loyalty line and the warning line have no live path for this companion (DF-35).
- Text reach: the crew desk shows name, role, cost, required regard and region only (logic.rs:2150-2181); greeting, hire dialog, loyalty line, warning line and description have no reader in the crates. Only the departure line is read (companion.rs:157-176; recap.rs:1085-1093).
- Dismiss: Session::dismiss_companion and the Crew desk Dismiss button remove a companion from the party (DF-34 resolved).
- DF-36 (resolved): the recruit notice now prints the catalog display name (at the base commit it read 'Recruited red_tomas.' with the raw id (game.rs:5748); the confirm line before it uses the catalog name (logic.rs:1923).

## 3. Look and feel (words only; the character hold stands)

- Voice [cite]: flat and contractual - "I protect. You pay. Simple arrangement." (C:5570). The greeting scene is a scarred man in a corner whose hand never leaves the cutlass at his belt (C:5569); quote as data, add no face or dress.
- Manner [invented]: he names the terms once and does not repeat them.
- Setting [invented]: a corner table in Mama Lucia's Kitchen (tavern, C:7684), the room that is the Cliff Tavern's back (data-fixes resolved-in-lore row).
- No face, age, build or dress is set beyond the quoted data lines; character plates are held.

## 4. [invented] lore

- [invented] He guarded Crimson Tide cargo before he sold the same service to captains; the Tide's home is Corsair's Rest (README pirate home ports, [invented]), so the pirates in his warning (C:5570) are the Tide.
- [invented] His 'code of honor' (C:5565) is a bodyguard's: the client is protected, the client's business is not his. A captain who spares an enemy gets his approval; one who sells him out gets the line in his hire dialog.

## 5. Hooks ([proposal] only; never a mechanic)

- [proposal] Copy hook: when the captain walks into Corsair's Rest with him aboard, a line that the cove knows his face - flavour only.
- Sound: ambience only, no cue named. No sfx id is cited.

## 6. Links

- Home port: [Corsair's Rest](../ports/corsairs_rest.md); bloc [The Shadow Ports](../factions/shadow_ports.md)
- Role: [Marine](roles/marine.md); index: [companions/README.md](README.md)
- Same role: [Iron Marta](iron_marta.md) (Ironhaven)
- Same port: [Rosa the Fence](rosa_the_fence.md)
- [The Crimson Tide](../factions/crimson_tide.md)
- [The Shadow Ports](../factions/shadow_ports.md)
- [The Corsair](../captains/corsair.md) - the type that starts at the same port
- Crew system with the same role name: [officers/README.md](../officers/README.md)

## 7. Flags and open questions

- DF-34 (resolved: role effects, cohesion, party summary, and dismiss wired), DF-35 (resolved: all 13 morale triggers and loyalty/warning lines wired), DF-36 (resolved: catalog name in recruit notice): see ../data-fixes.md.
- Shares a home port with Rosa the Fence; Corsair's Rest is the only port with two companions (C:5563, C:5659).
- Crew role and companion role share a name but are separate systems: resolved in lore (../data-fixes.md).
