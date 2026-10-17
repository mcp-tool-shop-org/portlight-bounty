# Quartermaster (companion role: quartermaster)

Base commit: 515a488 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at 515a488. Python cites read portlight@9b02494 path:line. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../../data-fixes.md | T-n see ../../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Name | Quartermaster | C:5531 |
| Role text (description) | "Manages cargo, negotiates prices, and counts every coin." | C:5532 |
| Combat damage bonus | 0 | C:5533 |
| Interception chance | 0.0 | C:5534 |
| Speed bonus | 0.0 | C:5535 |
| Danger reduction | 0.0 | C:5536 |
| Heal rate bonus | 0.0 | C:5537 |
| Inspection evasion | 0.0 | C:5538 |
| Trade bonus | 0.05 | C:5539 |
| Companions with this role | [Coin Mariam](../coin_mariam.md), [Dockhand Pete](../dockhand_pete.md) | C:5674-5690 |
| Player sees today | the role name on the offer row and the party row; never the description or the numbers | logic.rs:2129-2181 |
| Mechanic status | every effect number is [flavour, no mechanic] (DF-34) | companion.rs:193-233 |

Role ids are cite keys only; the player-facing word is the catalog name above.

## 1. Grounded facts

- The role authors Trade bonus 0.05 (C:5533-5539); the other effect fields are 0.
- Morale row for this role, from the 13 triggers (hand-maintained; role delta before the personality delta):

<!-- hand-maintained:begin -->
| Trigger | Delta for this role | Fired by | Cite |
|---|---|---|---|
| combat_won | +0 | nothing in Rust | C:5706 |
| combat_lost | -2 | nothing in Rust | C:5716 |
| crew_killed | -1 | nothing in Rust | C:5726 |
| profitable_trade | +5 | nothing in Rust | C:5736 |
| losing_trade | -3 | nothing in Rust | C:5746 |
| contraband_trade | -1 | nothing in Rust | C:5756 |
| spared_enemy | +0 | session.rs:2074-2076 | C:5766 |
| took_all | +2 | session.rs:2074-2076 | C:5776 |
| fled_combat | +0 | nothing in Rust | C:5786 |
| injury_sustained | -2 | nothing in Rust | C:5796 |
| long_voyage | -1 | nothing in Rust | C:5806 |
| port_arrival | +2 | nothing in Rust | C:5816 |
| nemesis_defeated | +3 | nothing in Rust | C:5826 |
<!-- hand-maintained:end -->

## 2. Role in play

- Where the numbers go: `party_combat_bonus` sums all seven fields over the party, each scaled by a morale multiplier that is 1.0 from morale 30 up and morale/30 below (companion.rs:185-209). `roll_interception` would let a companion with interception above 0 take a hit for the captain (companion.rs:211-233).
- Callers: none outside the test at companion.rs:327. `party_summary` (235-267) and `cohesion` (269-280) have no caller either (rg of crates). Godot builds its own party row (logic.rs:2129-2149).
- Python: portlight@9b02494 src/portlight/engine/companion_engine.py:216 and 245 define the same two functions; rg of src finds no caller. Python's `party` command does call `get_party_summary` and `get_cohesion` (src/portlight/app/cli.py:3183-3196), so the party table exists there and not in Rust.
- So a Quartermaster companion changes nothing aboard today. The role text above is a promise the sim does not keep; no shipped sentence repeats it, because the crew desk shows the role name only.

## 3. The crew role with the same name (a separate system)

| Key | Value | Cite |
|---|---|---|
| Crew role | Quartermaster, hired at the crew desk | session.rs:3868-3911 |
| Wage and cap | wage 2, max per ship 1; hire cost is wage x 10 silver | session.rs:3900-3905; session.rs:726 |
| Effect in the sim | cuts the daily wage bill by 10 percent while one is aboard; casualty weight 0.5 | session.rs:3900-3905; ship.rs:139-145, 218 |
| Python | portlight@9b02494 src/portlight/engine/ship_stats.py:202-209 (wage cut and a 5% sell bonus; the sell bonus has no caller in src) | portlight@9b02494 src/portlight/content/crew_roles.py |

A hired Quartermaster is an officer (officers/README.md): a generated name and trait, no lore. Canon calls the named person a companion and the hired role an officer. A captain can have both, and they never share a record.

## 4. Look and feel (words only)

- Role text [cite]: "Manages cargo, negotiates prices, and counts every coin." (C:5532). Canon adds no dress, face or figure.
- [invented] A quartermaster companion keeps the books a captain does not; both in the data frame the work as correction (a mess to fix; scales to watch).

## 5. [invented] lore

- [invented] The crew list's Quartermaster is a wage cut. The companion is the person who finds the leak the cut hides.

## 6. Hooks ([proposal] only)

- [proposal] Copy hook: any future party-row or journal line for this role should say what the companion did, not promise a number. No mechanic is requested.
- Sound: none named.

## 7. Links

- Companions: [Coin Mariam](../coin_mariam.md), [Dockhand Pete](../dockhand_pete.md)
- Index: [companions/README.md](../README.md); crew system: [officers/README.md](../../officers/README.md)

## 8. Flags

- DF-34: role effects authored and unread. DF-35: morale triggers. See ../../data-fixes.md.
- Crew role against companion role: resolved in lore (../../data-fixes.md).
