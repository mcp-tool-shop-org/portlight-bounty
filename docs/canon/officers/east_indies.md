# East Indies officer pool (east_indies)

Base commit: 515a488 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at 515a488. Python cites read portlight@9b02494 path:line. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Region | East Indies (cultural name: The Silk Waters) | C:6123; C:11893 |
| Names | 10 | C:6124 |
| Used for | officers hired at any port of this region | session.rs:701, 743-750; content.rs:1010-1018 |
| Ports in the region (computed) | [Jade Port](../ports/jade_port.md), [Monsoon Reach](../ports/monsoon_reach.md), [Silk Haven](../ports/silk_haven.md), [Crosswind Isle](../ports/crosswind_isle.md), [Dragon's Gate](../ports/dragons_gate.md), [Spice Narrows](../ports/spice_narrows.md) | ports[*].region |
| Fallback pool | no | content.rs:1010-1018 |
| Python | same names in the same order | portlight@9b02494 src/portlight/content/officer_names.py:25 |
| Player sees today | only 'Name - Role - trait' on the crew desk | logic.rs:2013-2027 |

## 1. Names (hand-maintained from C:6125-6134)

<!-- hand-maintained:begin -->
| # | Name | Cite | Authored person with the same given name |
|---|---|---|---|
| 1 | Rajan | C:6125 | Rajan the Tinker (skill trainer, Monsoon Reach; C:5427) |
| 2 | Mei Lin | C:6126 | - |
| 3 | Arjun | C:6127 | - |
| 4 | Suki | C:6128 | - |
| 5 | Bao | C:6129 | - |
| 6 | Padma | C:6130 | - |
| 7 | Kenji | C:6131 | - |
| 8 | Lien | C:6132 | - |
| 9 | Haruki | C:6133 | - |
| 10 | Kamala | C:6134 | Kamala Nair (Broker, Monsoon Reach; C:9861) |
<!-- hand-maintained:end -->

2 of 10 names here are also the given name of an authored person (DF-38, declined, resolved in lore: a hired officer is a namesake).

## 2. Region voice (data, quoted)

- greeting: "You honor us, Captain. Tea will be prepared while we discuss terms." (C:11907)
- farewell: "May your passage be swift and your cargo worthy." (C:11908)
- proverb: "A merchant who rushes drinks salt water." (C:11909)

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
- Port: [Jade Port](../ports/jade_port.md)
- Port: [Monsoon Reach](../ports/monsoon_reach.md)
- Port: [Silk Haven](../ports/silk_haven.md)
- Port: [Crosswind Isle](../ports/crosswind_isle.md)
- Port: [Dragon's Gate](../ports/dragons_gate.md)
- Port: [Spice Narrows](../ports/spice_narrows.md)

## 8. Flags

- DF-38: pool names reuse authored given names; declined, resolved in lore (../data-fixes.md).
