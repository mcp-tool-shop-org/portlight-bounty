# Marine (companion role: marine)

Base commit: 515a488 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at 515a488. Python cites read portlight@9b02494 path:line. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../../data-fixes.md | T-n see ../../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Name | Marine | C:5483 |
| Role text (description) | "A trained fighter who stands at your side in combat." | C:5484 |
| Combat damage bonus | 1 | C:5485 |
| Interception chance | 0.25 | C:5486 |
| Speed bonus | 0.0 | C:5487 |
| Danger reduction | 0.0 | C:5488 |
| Heal rate bonus | 0.0 | C:5489 |
| Inspection evasion | 0.0 | C:5490 |
| Trade bonus | 0.0 | C:5491 |
| Companions with this role | [Iron Marta](../iron_marta.md), [Red Tomas](../red_tomas.md) | C:5546-5562 |
| Player sees today | the role name, role description, and party summary on the Crew desk (DF-34 resolved) | logic.rs; game.rs |
| Mechanic status | every effect number is live in the sim via party_combat_bonus (DF-34 resolved) | companion.rs:193-280; session.rs |

Role ids are cite keys only; the player-facing word is the catalog name above.

## 1. Grounded facts

- The role authors Combat damage bonus 1, Interception chance 0.25 (C:5485-5491); the other effect fields are 0.
- Morale row for this role, from the 13 triggers (hand-maintained; role delta before the personality delta):

<!-- hand-maintained:begin -->
| Trigger | Delta for this role | Fired by | Cite |
|---|---|---|---|
| combat_won | +5 | Session (DF-35 resolved) | C:5706 |
| combat_lost | -3 | Session (DF-35 resolved) | C:5716 |
| crew_killed | -2 | Session (DF-35 resolved) | C:5726 |
| profitable_trade | +0 | Session (DF-35 resolved) | C:5736 |
| losing_trade | +0 | Session (DF-35 resolved) | C:5746 |
| contraband_trade | -2 | Session (DF-35 resolved) | C:5756 |
| spared_enemy | +2 | session.rs:2074-2076 | C:5766 |
| took_all | +1 | session.rs:2074-2076 | C:5776 |
| fled_combat | -5 | Session (DF-35 resolved) | C:5786 |
| injury_sustained | -1 | Session (DF-35 resolved) | C:5796 |
| long_voyage | -1 | Session (DF-35 resolved) | C:5806 |
| port_arrival | +1 | Session (DF-35 resolved) | C:5816 |
| nemesis_defeated | +8 | Session (DF-35 resolved) | C:5826 |
<!-- hand-maintained:end -->

## 2. Role in play

- Where the numbers go: `party_combat_bonus` sums all seven fields over the party, each scaled by a morale multiplier that is 1.0 from morale 30 up and morale/30 below (companion.rs:185-209). `roll_interception` would let a companion with interception above 0 take a hit for the captain (companion.rs:211-233).
- Callers: none outside the test at companion.rs:327. `party_summary` (235-267) and `cohesion` (269-280) have no caller either (rg of crates). Godot builds its own party row (logic.rs:2129-2149).
- Python: portlight@9b02494 src/portlight/engine/companion_engine.py:216 and 245 define the same two functions; rg of src finds no caller. Python's `party` command does call `get_party_summary` and `get_cohesion` (src/portlight/app/cli.py:3183-3196), so the party table exists there and not in Rust.
- So a Marine companion changes nothing aboard today. The role text above is a promise the sim does not keep; no shipped sentence repeats it, because the crew desk shows the role name only.

## 3. The crew role with the same name (a separate system)

| Key | Value | Cite |
|---|---|---|
| Crew role | Marine, hired at the crew desk | session.rs:3868-3911 |
| Wage and cap | wage 2, max per ship 4; hire cost is wage x 10 silver | session.rs:3894-3899; session.rs:726 |
| Effect in the sim | no effect found beyond the wage bill and a casualty weight of 2.0 (doubled in boarding) | session.rs:3894-3899; ship.rs:138, 217, 223-225; naval.rs:523-524 |
| Python | portlight@9b02494 src/portlight/engine/ship_stats.py:197-199 defines a boarding bonus; no caller in src | portlight@9b02494 src/portlight/content/crew_roles.py |

A hired Marine is an officer (officers/README.md): a generated name and trait, no lore. Canon calls the named person a companion and the hired role an officer. A captain can have both, and they never share a record.

## 4. Look and feel (words only)

- Role text [cite]: "A trained fighter who stands at your side in combat." (C:5484). Canon adds no dress, face or figure.
- [invented] A marine companion is the person a captain trusts at the rail when a boarding starts: a named fighter, not a hired hand. Canon calls them companions and the hired crew officers or crew, so the two never blur.

## 5. [invented] lore

- [invented] The data gives both marine companions a past with armed bands (a deserted garrison; a pirate bodyguard) and no companion a past at sea.

## 6. Hooks ([proposal] only)

- [proposal] Copy hook: any future party-row or journal line for this role should say what the companion did, not promise a number. No mechanic is requested.
- Sound: none named.

## 7. Links

- Companions: [Iron Marta](../iron_marta.md), [Red Tomas](../red_tomas.md)
- Index: [companions/README.md](../README.md); crew system: [officers/README.md](../../officers/README.md)

## 8. Flags

- DF-34 (resolved): role effects, party summary, cohesion, and dismiss wired. DF-35 (resolved): all 13 morale triggers wired. See ../../data-fixes.md.
- Crew role against companion role: resolved in lore (../../data-fixes.md).
