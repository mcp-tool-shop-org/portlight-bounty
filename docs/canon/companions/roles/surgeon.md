# Surgeon (companion role: surgeon)

Base commit: 515a488 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at 515a488. Python cites read portlight@9b02494 path:line. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../../data-fixes.md | T-n see ../../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Name | Surgeon | C:5507 |
| Role text (description) | "Keeps crew alive and your wounds clean." | C:5508 |
| Combat damage bonus | 0 | C:5509 |
| Interception chance | 0.0 | C:5510 |
| Speed bonus | 0.0 | C:5511 |
| Danger reduction | 0.0 | C:5512 |
| Heal rate bonus | 1.0 | C:5513 |
| Inspection evasion | 0.0 | C:5514 |
| Trade bonus | 0.0 | C:5515 |
| Companions with this role | [Dr. Amara](../dr_amara.md), [Patches](../patches.md) | C:5610-5626 |
| Player sees today | the role name on the offer row and the party row; never the description or the numbers | logic.rs:2129-2181 |
| Mechanic status | every effect number is [flavour, no mechanic] (DF-34) | companion.rs:193-233 |

Role ids are cite keys only; the player-facing word is the catalog name above.

## 1. Grounded facts

- The role authors Heal rate bonus 1.0 (C:5509-5515); the other effect fields are 0.
- Morale row for this role, from the 13 triggers (hand-maintained; role delta before the personality delta):

<!-- hand-maintained:begin -->
| Trigger | Delta for this role | Fired by | Cite |
|---|---|---|---|
| combat_won | -1 | nothing in Rust | C:5706 |
| combat_lost | -2 | nothing in Rust | C:5716 |
| crew_killed | -5 | nothing in Rust | C:5726 |
| profitable_trade | +0 | nothing in Rust | C:5736 |
| losing_trade | +0 | nothing in Rust | C:5746 |
| contraband_trade | -3 | nothing in Rust | C:5756 |
| spared_enemy | +5 | session.rs:2074-2076 | C:5766 |
| took_all | -3 | session.rs:2074-2076 | C:5776 |
| fled_combat | +2 | nothing in Rust | C:5786 |
| injury_sustained | -1 | nothing in Rust | C:5796 |
| long_voyage | -1 | nothing in Rust | C:5806 |
| port_arrival | +1 | nothing in Rust | C:5816 |
| nemesis_defeated | +2 | nothing in Rust | C:5826 |
<!-- hand-maintained:end -->

## 2. Role in play

- Where the numbers go: `party_combat_bonus` sums all seven fields over the party, each scaled by a morale multiplier that is 1.0 from morale 30 up and morale/30 below (companion.rs:185-209). `roll_interception` would let a companion with interception above 0 take a hit for the captain (companion.rs:211-233).
- Callers: none outside the test at companion.rs:327. `party_summary` (235-267) and `cohesion` (269-280) have no caller either (rg of crates). Godot builds its own party row (logic.rs:2129-2149).
- Python: portlight@9b02494 src/portlight/engine/companion_engine.py:216 and 245 define the same two functions; rg of src finds no caller. Python's `party` command does call `get_party_summary` and `get_cohesion` (src/portlight/app/cli.py:3183-3196), so the party table exists there and not in Rust.
- So a Surgeon companion changes nothing aboard today. The role text above is a promise the sim does not keep; no shipped sentence repeats it, because the crew desk shows the role name only.

## 3. The crew role with the same name (a separate system)

| Key | Value | Cite |
|---|---|---|
| Crew role | Surgeon, hired at the crew desk | session.rs:3868-3911 |
| Wage and cap | wage 3, max per ship 1; hire cost is wage x 10 silver | session.rs:3888-3893; session.rs:726 |
| Effect in the sim | no effect found beyond the wage bill and a casualty weight of 0.5 | session.rs:3888-3893; ship.rs:137, 216; naval.rs:520-522 |
| Python | portlight@9b02494 src/portlight/engine/ship_stats.py:192-194 defines a 30% death reduction; no caller in src | portlight@9b02494 src/portlight/content/crew_roles.py |

A hired Surgeon is an officer (officers/README.md): a generated name and trait, no lore. Canon calls the named person a companion and the hired role an officer. A captain can have both, and they never share a record.

## 4. Look and feel (words only)

- Role text [cite]: "Keeps crew alive and your wounds clean." (C:5508). Canon adds no dress, face or figure.
- [invented] A surgeon companion is paid in silver and in trust; both in the data warn the captain about cruelty or waste (warning lines), and one is rueful about drink.

## 5. [invented] lore

- [invented] The crew list's Surgeon is a number on the roster. The companion is the person the roster number stands for on a good day.

## 6. Hooks ([proposal] only)

- [proposal] Copy hook: any future party-row or journal line for this role should say what the companion did, not promise a number. No mechanic is requested.
- Sound: none named.

## 7. Links

- Companions: [Dr. Amara](../dr_amara.md), [Patches](../patches.md)
- Index: [companions/README.md](../README.md); crew system: [officers/README.md](../../officers/README.md)

## 8. Flags

- DF-34: role effects authored and unread. DF-35: morale triggers. See ../../data-fixes.md.
- Crew role against companion role: resolved in lore (../../data-fixes.md).
