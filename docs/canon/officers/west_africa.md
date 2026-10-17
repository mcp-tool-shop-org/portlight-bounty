# West Africa officer pool (west_africa)

Base commit: 515a488 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at 515a488. Python cites read portlight@9b02494 path:line. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Region | West Africa (cultural name: The Gold Coast) | C:6108; C:11839 |
| Names | 10 | C:6109 |
| Used for | officers hired at any port of this region | session.rs:701, 743-750; content.rs:1010-1018 |
| Ports in the region (computed) | [Sun Harbor](../ports/sun_harbor.md), [Palm Cove](../ports/palm_cove.md), [Iron Point](../ports/iron_point.md), [Pearl Shallows](../ports/pearl_shallows.md) | ports[*].region |
| Fallback pool | no | content.rs:1010-1018 |
| Python | same names in the same order | portlight@9b02494 src/portlight/content/officer_names.py:21 |
| Player sees today | only 'Name - Role - trait' on the crew desk | logic.rs:2013-2027 |

## 1. Names (hand-maintained from C:6110-6119)

<!-- hand-maintained:begin -->
| # | Name | Cite | Authored person with the same given name |
|---|---|---|---|
| 1 | Kwame | C:6110 | Elder Kwame (Council Elder, Palm Cove; C:8913) |
| 2 | Aminata | C:6111 | - |
| 3 | Kofi | C:6112 | Kofi Asante (Cotton Master, Sun Harbor; C:8701); Foreman Kofi Mensah (Mine Foreman, Iron Point; C:9093); Kofi (merchant, Palm Cove; C:5977) |
| 4 | Adaeze | C:6113 | Adaeze (merchant, Sun Harbor; C:5961) |
| 5 | Sekou | C:6114 | - |
| 6 | Mariam | C:6115 | Coin Mariam (companion, Al-Manar; C:5673) |
| 7 | Ousmane | C:6116 | - |
| 8 | Aisha | C:6117 | - |
| 9 | Yusuf | C:6118 | Yusuf Diallo (Provisions Master, Sun Harbor; C:8723) |
| 10 | Zara | C:6119 | Inspector Zara Osman (Chief Customs Inspector, Al-Manar; C:7347) |
<!-- hand-maintained:end -->

6 of 10 names here are also the given name of an authored person (DF-38, declined, resolved in lore: a hired officer is a namesake).

## 2. Region voice (data, quoted)

- greeting: "You return! The coast remembers you, Captain." (C:11851)
- farewell: "Go well. The sea carries our memory of you." (C:11852)
- proverb: "The market remembers who gave, not who took." (C:11853)

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
- Port: [Sun Harbor](../ports/sun_harbor.md)
- Port: [Palm Cove](../ports/palm_cove.md)
- Port: [Iron Point](../ports/iron_point.md)
- Port: [Pearl Shallows](../ports/pearl_shallows.md)

## 8. Flags

- DF-38: pool names reuse authored given names; declined, resolved in lore (../data-fixes.md).
