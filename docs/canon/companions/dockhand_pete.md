# Dockhand Pete (dockhand_pete)

Base commit: 515a488 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at 515a488. Python cites read portlight@9b02494 path:line. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Catalog name | Dockhand Pete | C:5689 |
| Role | [Quartermaster](roles/quartermaster.md) | C:5690 |
| Personality (data key) | reckless | C:5694 |
| Home port | [Palm Cove](../ports/palm_cove.md) | C:5691 |
| Bloc of the home port | [The Gold Coast Compact](../factions/gold_coast_compact.md) | ports/palm_cove.md section 0 |
| Region | West Africa | C:5692 |
| Hire cost | 40 silver | C:5695 |
| Required regional regard | 0 in West Africa | C:5696; the shipped row prints it as 'standing 0 in West Africa' (logic.rs:2176) |
| Hire gate | docked at the home port; not already in the party; not departed; party under 2; silver enough; regional regard enough | companion.rs:51-93; model.rs:431,439 |
| Starting morale | 70 | companion.rs:95-108 (morale at line 102) |
| Player sees today | offer row (name, role, silver, regard and region), a confirm line, a notice, a party row, and the departure line in the victory recap | logic.rs:2150-2181; logic.rs:1923; game.rs:5748; logic.rs:2129-2149; recap.rs:1085-1093 |
| Not shown | description, greeting, hire dialog | loyalty/warning lines and role description reach the Crew desk (DF-34, DF-35 resolved) |
| Mechanic status | role effects, cohesion, dismiss, and all 13 morale triggers are live (DF-34, DF-35 resolved) | companion.rs:124-280; session.rs |
| Python record | same strings in the Python reference (compared field by field, no differences) | portlight@9b02494 src/portlight/content/companions.py:252 |

## 1. Grounded facts (every data line quoted)

- description: "Knows every dock, every scale, and every harbormaster's weakness on the West African coast." (C:5693)
- greeting (scene text): "A wiry man balances on a cargo crate, juggling oranges. He catches all three without looking." (C:5697)
- hire dialog: "I know cargo. I know ports. I know who's cheating on the scales. Hire me and you'll never overpay for grain again." (C:5698)
- loyalty line: "Good ship, good captain, good cargo. Life's good!" (C:5699)
- warning line: "Captain, I'm starting to feel like ballast. Use me or lose me." (C:5700)
- departure line: "I need a ship that's actually going somewhere. No hard feelings." (C:5701)

Quotes are as in content.json; only an em dash is folded to ' - ' and tagged.

## 2. Role in play (what the sim does today; no proposals here)

- Recruit: only at Palm Cove, for 40 silver, with regional regard 0 or more in West Africa, a free party slot and a companion who has not already left (companion.rs:51-93). The cost is taken in the session, then morale starts at 70 (session.rs:3359-3373; companion.rs:95-108).
- Regional regard is a per-region number (model.rs:283-287). It rises by 1 on each arrival at a port of that region (reputation.rs:137-141) and moves with trades and inspections (reputation.rs:111, 172); the sources were not traced further here.
- Role effect: Dockhand Pete is a Quartermaster. Every effect number on the role is live in the sim (DF-34 resolved; formerly authored and unread (roles/quartermaster.md; DF-34). Aboard, this companion contributes its role effects to party_combat_bonus (DF-34 resolved).
- Morale triggers: all 13 morale triggers and personality deltas now fire in Session (DF-35 resolved; at the phase 6A base commit, only after a duel victory as spared_enemy or took_all (session.rs:2074-2076). For this companion those total +0 (spared) and +2 (took all): role delta plus the reckless personality delta, applied only where the role delta is not zero (companion.rs:130-141; C:5766, C:5776).
- Departure is not reachable today [inferred from companion.rs:130-176]: take-all victories give +2 and spared victories +0, so morale never falls toward the departure line (10 or less) from the two live triggers. The departure line, the loyalty line and the warning line have no live path for this companion (DF-35).
- Text reach: the crew desk shows name, role, cost, required regard and region only (logic.rs:2150-2181); greeting, hire dialog, loyalty line, warning line and description have no reader in the crates. Only the departure line is read (companion.rs:157-176; recap.rs:1085-1093).
- Dismiss: Session::dismiss_companion and the Crew desk Dismiss button remove a companion from the party (DF-34 resolved).
- DF-36 (resolved): the recruit notice now prints the catalog display name (at the base commit it read 'Recruited dockhand_pete.' with the raw id (game.rs:5748); the confirm line before it uses the catalog name (logic.rs:1923).

## 3. Look and feel (words only; the character hold stands)

- Voice [cite]: cheerful, quick - "I know cargo. I know ports. I know who's cheating on the scales." (C:5698). The greeting scene is a wiry man balancing on a cargo crate, juggling oranges, catching all three without looking (C:5697); quote as data.
- Manner [invented]: he counts aloud in rhyme and loses count on purpose.
- Setting [invented]: the Working Dock (shipyard, C:8836; Esi Owusu), where crates are stacked high enough to balance on.
- No face, age, build or dress is set beyond the quoted data lines; character plates are held.

## 4. [invented] lore

- [invented] He keeps no ledger; he keeps a tally rhyme for every scale on the coast, and the harbormasters' 'weakness' (C:5693) is that each one has a favourite scale.
- [invented] He is the quartermaster a Dockhand-type captain would have found by a dock's third visit (captains/dockhand.md).

## 5. Hooks ([proposal] only; never a mechanic)

- [proposal] Copy hook: the loyalty line is exuberant ('Life's good!'); a party-row line in that voice would sit well beside Coin Mariam's - flavour only.
- Sound: ambience only, no cue named. No sfx id is cited.

## 6. Links

- Home port: [Palm Cove](../ports/palm_cove.md); bloc [The Gold Coast Compact](../factions/gold_coast_compact.md)
- Role: [Quartermaster](roles/quartermaster.md); index: [companions/README.md](README.md)
- Same role: [Coin Mariam](coin_mariam.md) (Al-Manar)
- [The Gold Coast Compact](../factions/gold_coast_compact.md)
- [The Dockhand](../captains/dockhand.md)
- Crew system with the same role name: [officers/README.md](../officers/README.md)

## 7. Flags and open questions

- DF-34 (resolved: role effects, cohesion, party summary, and dismiss wired), DF-35 (resolved: all 13 morale triggers and loyalty/warning lines wired), DF-36 (resolved: catalog name in recruit notice): see ../data-fixes.md.
- Crew role and companion role share a name but are separate systems: resolved in lore (../data-fixes.md).
