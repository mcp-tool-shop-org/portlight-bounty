# North Atlantic officer pool (north_atlantic)

Base commit: 515a488 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at 515a488. Python cites read portlight@9b02494 path:line. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Region | North Atlantic (cultural name: The Iron Coast) | C:6093; C:11783 |
| Names | 10 | C:6094 |
| Used for | officers hired at any port of this region | session.rs:701, 743-750; content.rs:1010-1018 |
| Ports in the region (computed) | [Ironhaven](../ports/ironhaven.md), [Stormwall](../ports/stormwall.md), [Thornport](../ports/thornport.md) | ports[*].region |
| Fallback pool | no | content.rs:1010-1018 |
| Python | same names in the same order | portlight@9b02494 src/portlight/content/officer_names.py:17 |
| Player sees today | only 'Name - Role - trait' on the crew desk | logic.rs:2013-2027 |

## 1. Names (hand-maintained from C:6095-6104)

<!-- hand-maintained:begin -->
| # | Name | Cite | Authored person with the same given name |
|---|---|---|---|
| 1 | William | C:6095 | - |
| 2 | Margaret | C:6096 | - |
| 3 | Henrik | C:6097 | Forge Master Henrik Brandt (Guild Master, Ironhaven; C:7983) |
| 4 | Brigitte | C:6098 | - |
| 5 | Duncan | C:6099 | - |
| 6 | Eleanor | C:6100 | - |
| 7 | Gunnar | C:6101 | Gunnar Stahl (Harbor Master, Ironhaven; C:7961) |
| 8 | Astrid | C:6102 | Astrid Vekhren (Pact Commissioner, Ironhaven; C:8005); Elder Astrid (Town Elder, Thornport; C:8445) |
| 9 | Thomas | C:6103 | - |
| 10 | Catherine | C:6104 | - |
<!-- hand-maintained:end -->

3 of 10 names here are also the given name of an authored person (DF-38, declined, resolved in lore: a hired officer is a namesake).

## 2. Region voice (data, quoted)

- greeting: "State your business, Captain. These waters have little patience." (C:11795)
- farewell: "Keep your hull tight. The Atlantic forgives nothing." (C:11796)
- proverb: "The storm tests the hull, not the cargo." (C:11797)

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
- Port: [Ironhaven](../ports/ironhaven.md)
- Port: [Stormwall](../ports/stormwall.md)
- Port: [Thornport](../ports/thornport.md)

## 8. Flags

- DF-38: pool names reuse authored given names; declined, resolved in lore (../data-fixes.md).
