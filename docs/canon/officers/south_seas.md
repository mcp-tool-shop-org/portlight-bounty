# South Seas officer pool (south_seas)

Base commit: 515a488 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at 515a488. Python cites read portlight@9b02494 path:line. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Region | South Seas (cultural name: The Reef Kingdoms) | C:6138; C:11951 |
| Names | 10 | C:6139 |
| Used for | officers hired at any port of this region | session.rs:701, 743-750; content.rs:1010-1018 |
| Ports in the region (computed) | [Ember Isle](../ports/ember_isle.md), [Typhoon Anchorage](../ports/typhoon_anchorage.md), [Coral Throne](../ports/coral_throne.md) | ports[*].region |
| Fallback pool | no | content.rs:1010-1018 |
| Python | same names in the same order | portlight@9b02494 src/portlight/content/officer_names.py:29 |
| Player sees today | only 'Name - Role - trait' on the crew desk | logic.rs:2013-2027 |

## 1. Names (hand-maintained from C:6140-6149)

<!-- hand-maintained:begin -->
| # | Name | Cite | Authored person with the same given name |
|---|---|---|---|
| 1 | Tane | C:6140 | War Chief Tane (War Chief, Coral Throne; C:11346); Tane (merchant, Coral Throne; C:6047) |
| 2 | Moana | C:6141 | Dive Boss Moana (Pearl Boss, Typhoon Anchorage; C:11129) |
| 3 | Rangi | C:6142 | Storm Chief Rangi (Storm Chief, Typhoon Anchorage; C:11087) |
| 4 | Leilani | C:6143 | Leilani Bonecatcher (style master, Typhoon Anchorage; C:6246) |
| 5 | Makoa | C:6144 | - |
| 6 | Aroha | C:6145 | - |
| 7 | Koa | C:6146 | Harbor Master Koa (Chain Master, Typhoon Anchorage; C:11108) |
| 8 | Nalani | C:6147 | - |
| 9 | Ioane | C:6148 | - |
| 10 | Mele | C:6149 | - |
<!-- hand-maintained:end -->

5 of 10 names here are also the given name of an authored person (DF-38, declined, resolved in lore: a hired officer is a namesake).

## 2. Region voice (data, quoted)

- greeting: "The Coral King permits you to anchor. State your tribute." (C:11963)
- farewell: "The reef remembers your passage. Return only if you mean to trade." (C:11964)
- proverb: "The reef welcomes the patient; it drowns the greedy." (C:11965)

## 3. Role in play

- A hire at a port of this region draws one name from the table above, uniformly (content.rs:1010-1018; session.rs:3935-3938). The name carries no meaning in the sim: no price, no skill, no morale.
- The same pool serves every non-sailor role. Only the role (and the role's real effects, officers/README.md section 3) distinguishes two officers.

## 4. Look and feel (words only)

- Voice [cite]: the region's greeting and proverb above set the tone officers are quoted in; canon adds no dress, face or figure.
- Manner [invented]: aboard, an officer is called by the pool name and the role, 'Name the surgeon', and answers to it in the region's speech.

## 5. [invented] lore

- [invented] An officer signs the ship's articles with the name on this list and keeps the name for the voyage; two aboard with the same name are told apart by role.
- [invented] Officers hired in this region know the region's ports by name and its custom by habit (the cultural name above), and carry no kin or history in the data.

## 6. Hooks ([proposal] only)

- [proposal] Copy hook: a hire notice that names the officer ('Kofi signs on as navigator.') would use this table and needs no data change; not requested here.
- Sound: none named.

## 7. Links

- Index: [officers/README.md](README.md); companions: [companions/README.md](../companions/README.md)
- Port: [Ember Isle](../ports/ember_isle.md)
- Port: [Typhoon Anchorage](../ports/typhoon_anchorage.md)
- Port: [Coral Throne](../ports/coral_throne.md)

## 8. Flags

- DF-38: pool names reuse authored given names; declined, resolved in lore (../data-fixes.md).
