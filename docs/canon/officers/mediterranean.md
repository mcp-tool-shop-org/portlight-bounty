# Mediterranean officer pool (mediterranean)

Base commit: 515a488 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at 515a488. Python cites read portlight@9b02494 path:line. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Region | Mediterranean (cultural name: The Middle Sea) | C:6078; C:11729 |
| Names | 10 | C:6079 |
| Used for | officers hired at any port of this region | session.rs:701, 743-750; content.rs:1010-1018 |
| Ports in the region (computed) | [Porto Novo](../ports/porto_novo.md), [Al-Manar](../ports/al_manar.md), [Silva Bay](../ports/silva_bay.md), [Corsair's Rest](../ports/corsairs_rest.md) | ports[*].region |
| Fallback pool | yes: the first pool, used for an unknown region | content.rs:1010-1018 |
| Python | same names in the same order | portlight@9b02494 src/portlight/content/officer_names.py:13 |
| Player sees today | only 'Name - Role - trait' on the crew desk | logic.rs:2013-2027 |

## 1. Names (hand-maintained from C:6080-6089)

<!-- hand-maintained:begin -->
| # | Name | Cite | Authored person with the same given name |
|---|---|---|---|
| 1 | Marco | C:6080 | Marco (merchant, Porto Novo; C:5871) |
| 2 | Sophia | C:6081 | - |
| 3 | Nikolaos | C:6082 | - |
| 4 | Fatima | C:6083 | Fatima (merchant, Al-Manar; C:5886) |
| 5 | Lorenzo | C:6084 | - |
| 6 | Valentina | C:6085 | - |
| 7 | Dimitri | C:6086 | Dimitri Andros (Master Shipwright, Porto Novo; C:7074) |
| 8 | Leila | C:6087 | - |
| 9 | Antonio | C:6088 | - |
| 10 | Isadora | C:6089 | - |
<!-- hand-maintained:end -->

3 of 10 names here are also the given name of an authored person (DF-38, declined, resolved in lore: a hired officer is a namesake).

## 2. Region voice (data, quoted)

- greeting: "Welcome to port, Captain. Your cargo manifest, if you please." (C:11740)
- farewell: "Fair winds and full holds. May your ledger balance." (C:11741)
- proverb: "A contract honored is worth more than gold hoarded." (C:11742)

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
- Port: [Porto Novo](../ports/porto_novo.md)
- Port: [Al-Manar](../ports/al_manar.md)
- Port: [Silva Bay](../ports/silva_bay.md)
- Port: [Corsair's Rest](../ports/corsairs_rest.md)

## 8. Flags

- DF-38: pool names reuse authored given names; declined, resolved in lore (../data-fixes.md).
