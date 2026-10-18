# Navigator (companion role: navigator)

Base commit: 515a488 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at 515a488. Python cites read portlight@9b02494 path:line. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../../data-fixes.md | T-n see ../../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Name | Navigator | C:5495 |
| Role text (description) | "Reads the stars and currents. Gets you there faster and safer." | C:5496 |
| Combat damage bonus | 0 | C:5497 |
| Interception chance | 0.0 | C:5498 |
| Speed bonus | 0.5 | C:5499 |
| Danger reduction | 0.03 | C:5500 |
| Heal rate bonus | 0.0 | C:5501 |
| Inspection evasion | 0.0 | C:5502 |
| Trade bonus | 0.0 | C:5503 |
| Companions with this role | [Star Lila](../star_lila.md), [Old Henry](../old_henry.md) | C:5578-5594 |
| Player sees today | the role name, role description, and party summary on the Crew desk (DF-34 resolved) | logic.rs; game.rs |
| Mechanic status | every effect number is live in the sim via party_combat_bonus (DF-34 resolved) | companion.rs:193-280; session.rs |

Role ids are cite keys only; the player-facing word is the catalog name above.

## 1. Grounded facts

- The role authors Speed bonus 0.5, Danger reduction 0.03 (C:5497-5503); the other effect fields are 0.
- Morale row for this role, from the 13 triggers (hand-maintained; role delta before the personality delta):

<!-- hand-maintained:begin -->
| Trigger | Delta for this role | Fired by | Cite |
|---|---|---|---|
| combat_won | +1 | Session (DF-35 resolved) | C:5706 |
| combat_lost | -2 | Session (DF-35 resolved) | C:5716 |
| crew_killed | -1 | Session (DF-35 resolved) | C:5726 |
| profitable_trade | +0 | Session (DF-35 resolved) | C:5736 |
| losing_trade | +0 | Session (DF-35 resolved) | C:5746 |
| contraband_trade | -1 | Session (DF-35 resolved) | C:5756 |
| spared_enemy | +1 | session.rs:2074-2076 | C:5766 |
| took_all | -1 | session.rs:2074-2076 | C:5776 |
| fled_combat | +2 | Session (DF-35 resolved) | C:5786 |
| injury_sustained | -2 | Session (DF-35 resolved) | C:5796 |
| long_voyage | +2 | Session (DF-35 resolved) | C:5806 |
| port_arrival | +1 | Session (DF-35 resolved) | C:5816 |
| nemesis_defeated | +3 | Session (DF-35 resolved) | C:5826 |
<!-- hand-maintained:end -->

## 2. Role in play

- Where the numbers go: `party_combat_bonus` sums all seven fields over the party, each scaled by a morale multiplier that is 1.0 from morale 30 up and morale/30 below (companion.rs:185-209). `roll_interception` would let a companion with interception above 0 take a hit for the captain (companion.rs:211-233).
- Callers: none outside the test at companion.rs:327. `party_summary` (235-267) and `cohesion` (269-280) have no caller either (rg of crates). Godot builds its own party row (logic.rs:2129-2149).
- Python: portlight@9b02494 src/portlight/engine/companion_engine.py:216 and 245 define the same two functions; rg of src finds no caller. Python's `party` command does call `get_party_summary` and `get_cohesion` (src/portlight/app/cli.py:3183-3196), so the party table exists there and not in Rust.
- So a Navigator companion changes nothing aboard today. The role text above is a promise the sim does not keep; no shipped sentence repeats it, because the crew desk shows the role name only.

## 3. The crew role with the same name (a separate system)

| Key | Value | Cite |
|---|---|---|
| Crew role | Navigator, hired at the crew desk | session.rs:3868-3911 |
| Wage and cap | wage 3, max per ship 1; hire cost is wage x 10 silver | session.rs:3882-3887; session.rs:726 |
| Effect in the sim | +0.5 voyage speed while one is aboard; casualty weight 0.5 | session.rs:3882-3887; ship.rs:124-130, 136, 215; voyage.rs:1043 |
| Python | portlight@9b02494 src/portlight/engine/ship_stats.py:182-184 | portlight@9b02494 src/portlight/content/crew_roles.py |

A hired Navigator is an officer (officers/README.md): a generated name and trait, no lore. Canon calls the named person a companion and the hired role an officer. A captain can have both, and they never share a record.

## 4. Look and feel (words only)

- Role text [cite]: "Reads the stars and currents. Gets you there faster and safer." (C:5496). Canon adds no dress, face or figure.
- [invented] A navigator companion is a person with a name, a tune and a set of charts; the hired Navigator on the crew list is a role. Lore: the companion is the one the crew quotes.

## 5. [invented] lore

- [invented] Both navigator companions come from the far ends of a lane network (a free port and a whaling harbour), in step with the lanes' own lore (lanes/monsoon_shortcut.md, ../landmarks/whale_arch.md).

## 6. Hooks ([proposal] only)

- [proposal] Copy hook: any future party-row or journal line for this role should say what the companion did, not promise a number. No mechanic is requested.
- Sound: none named.

## 7. Links

- Companions: [Star Lila](../star_lila.md), [Old Henry](../old_henry.md)
- Index: [companions/README.md](../README.md); crew system: [officers/README.md](../../officers/README.md)

## 8. Flags

- DF-34 (resolved): role effects, party summary, cohesion, and dismiss wired. DF-35 (resolved): all 13 morale triggers wired. See ../../data-fixes.md.
- Crew role against companion role: resolved in lore (../../data-fixes.md).
