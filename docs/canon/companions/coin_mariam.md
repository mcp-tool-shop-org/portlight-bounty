# Coin Mariam (coin_mariam)

Base commit: 515a488 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at 515a488. Python cites read portlight@9b02494 path:line. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Catalog name | Coin Mariam | C:5673 |
| Role | [Quartermaster](roles/quartermaster.md) | C:5674 |
| Personality (data key) | lawful | C:5678 |
| Home port | [Al-Manar](../ports/al_manar.md) | C:5675 |
| Bloc of the home port | [The Exchange Alliance](../factions/exchange_alliance.md) | ports/al_manar.md section 0 |
| Region | Mediterranean | C:5676 |
| Hire cost | 100 silver | C:5679 |
| Required regional regard | 10 in Mediterranean | C:5680; the shipped row prints it as 'standing 10 in Mediterranean' (logic.rs:2176) |
| Hire gate | docked at the home port; not already in the party; not departed; party under 2; silver enough; regional regard enough | companion.rs:51-93; model.rs:431,439 |
| Starting morale | 70 | companion.rs:95-108 (morale at line 102) |
| Player sees today | offer row (name, role, silver, regard and region), a confirm line, a notice, a party row, and the departure line in the victory recap | logic.rs:2150-2181; logic.rs:1923; game.rs:5748; logic.rs:2129-2149; recap.rs:1085-1093 |
| Not shown | description, greeting, hire dialog, loyalty line, warning line | no reader in crates (section 2) |
| Mechanic status | role effects [flavour, no mechanic] (DF-34); morale reacts to 2 of 13 triggers (DF-35) | companion.rs:193-281; session.rs:2074-2076 |
| Python record | same strings in the Python reference (compared field by field, no differences) | portlight@9b02494 src/portlight/content/companions.py:236 |

## 1. Grounded facts (every data line quoted)

- description: "A merchant's daughter who can appraise cargo by smell and negotiate prices in seven languages." (C:5677)
- greeting (scene text): "A well-dressed woman inspects cargo manifests at the harbormaster's office. She spots errors in seconds." (C:5681)
- hire dialog: "Your books are a mess. Your cargo holds are inefficient. Your pricing is amateur. Hire me and I'll fix all three." (C:5682)
- loyalty line: "The ledgers are clean, the holds are full, and the silver is flowing. My kind of ship." (C:5683)
- warning line: "The numbers don't lie, Captain. We're heading for a loss if you keep this up." (C:5684)
- departure line: "I don't serve captains who ignore the ledger. Find another quartermaster." (C:5685)

Quotes are as in content.json; only an em dash is folded to ' - ' and tagged.

## 2. Role in play (what the sim does today; no proposals here)

- Recruit: only at Al-Manar, for 100 silver, with regional regard 10 or more in Mediterranean, a free party slot and a companion who has not already left (companion.rs:51-93). The cost is taken in the session, then morale starts at 70 (session.rs:3359-3373; companion.rs:95-108).
- Regional regard is a per-region number (model.rs:283-287). It rises by 1 on each arrival at a port of that region (reputation.rs:137-141) and moves with trades and inspections (reputation.rs:111, 172); the sources were not traced further here.
- Role effect: Coin Mariam is a Quartermaster. Every effect number on the role is authored and unread (roles/quartermaster.md; DF-34). Aboard, this companion changes no price, speed, damage, danger, healing or inspection in the sim.
- Morale that can move today: only after a duel victory, as spared_enemy or took_all (session.rs:2074-2076). For this companion those total +0 (spared) and +0 (took all): role delta plus the lawful personality delta, applied only where the role delta is not zero (companion.rs:130-141; C:5766, C:5776).
- Departure is not reachable today [inferred from companion.rs:130-176]: take-all victories give +0 and spared victories +0, so morale never falls toward the departure line (10 or less) from the two live triggers. The departure line, the loyalty line and the warning line have no live path for this companion (DF-35).
- Text reach: the crew desk shows name, role, cost, required regard and region only (logic.rs:2150-2181); greeting, hire dialog, loyalty line, warning line and description have no reader in the crates. Only the departure line is read (companion.rs:157-176; recap.rs:1085-1093).
- Dismiss: there is no control and no Session method, so a recruited companion stays until they leave (crew_screen.rs:6-7).
- DF-36: after a confirmed recruit the notice reads 'Recruited coin_mariam.' with the raw id (game.rs:5748); the confirm line before it uses the catalog name (logic.rs:1923).

## 3. Look and feel (words only; the character hold stands)

- Voice [cite]: crisp, corrective - "Your books are a mess... Hire me and I'll fix all three." (C:5682). The greeting scene is a well-dressed woman inspecting cargo manifests at the harbormaster's office, spotting errors in seconds (C:5681); quote as data.
- Manner [invented]: she reads the totals first and the cargo second.
- Setting [invented]: the Harbor of Arrivals (harbor_master, C:7186; Khalil al-Rashid), the office the greeting names.
- No face, age, build or dress is set beyond the quoted data lines; character plates are held.

## 4. [invented] lore

- [invented] She learned her trade in the Spice Bazaar's counting rooms (exchange, C:7196); 'seven languages' (C:5677) are the Bazaar's trade tongues.
- [invented] Inspector Zara Osman at the Scales of Truth (customs, C:7226) sends her the manifests she cannot reconcile.

## 5. Hooks ([proposal] only; never a mechanic)

- [proposal] Copy hook: her warning line ('We're heading for a loss') could back a future ledger beat - flavour only; the Day's report does not show morale.
- Sound: ambience only, no cue named. No sfx id is cited.

## 6. Links

- Home port: [Al-Manar](../ports/al_manar.md); bloc [The Exchange Alliance](../factions/exchange_alliance.md)
- Role: [Quartermaster](roles/quartermaster.md); index: [companions/README.md](README.md)
- Same role: [Dockhand Pete](dockhand_pete.md) (Palm Cove)
- [The Exchange Alliance](../factions/exchange_alliance.md)
- [The Merchant](../captains/merchant.md)
- Crew system with the same role name: [officers/README.md](../officers/README.md)

## 7. Flags and open questions

- DF-34 (role effects unread), DF-35 (morale triggers), DF-36 (raw id in the recruit notice): see ../data-fixes.md.
- Officer-pool overlap: the West Africa pool has the name Mariam (C:6115); a hired officer is a namesake (officers/README.md, DF-38 declined).
- Crew role and companion role share a name but are separate systems: resolved in lore (../data-fixes.md).
