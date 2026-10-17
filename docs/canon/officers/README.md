# Officer canon index

Base commit: 515a488 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at 515a488. Python cites read portlight@9b02494 path:line. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Authored officers | 0: officers are generated at hire, never authored | session.rs:691-750 |
| Name pools | 5 regions, 10 names each (50 names) | C:6075-6152 |
| Traits | 15 | C:6153-6169 |
| Crew roles | 6 (sailor and 5 specialist roles) | session.rs:3868-3911 |
| Officer record | name, role, origin port, trait | model.rs:173-178 |
| Authored names that a pool name also is | 19 of 50 | section 5 |
| Python reference | the same 5 pools and 15 traits (compared, equal) | portlight@9b02494 src/portlight/content/officer_names.py:12-39 |

## 1. What the player sees today

| What | Reaches the player? | Cite |
|---|---|---|
| One line per officer on the crew desk: 'Name - Role - trait' | Yes | logic.rs:2013-2027 (format at 2023) |
| One row per crew role: 'Role  count  max  hire cost' | Yes | logic.rs:1999-2012 |
| A hire notice: 'Hired n Role. Crew n. Silver n.' | Yes; it does not name the officer | game.rs:5549 |
| Origin port of an officer | No: stored and saved, never shown | model.rs:176; save.rs:1548 |
| Any lore, portrait, dialogue or effect of a trait | No: none exists | rg of trait_name |
| Firing | Removes officers from the end of the list | session.rs:1419-1447 |

Sailors are anonymous: no officer record (session.rs:743-750).

## 2. How an officer is made

1. The captain hires a non-sailor role at a port. The cost is the role's wage x 10 silver per hire, capped by the role's maximum per ship (session.rs:691-733).
2. For each hire the sim draws a name from the pool of the port's region, then a trait from the trait list, with the seeded random stream (session.rs:743-750; officer_name 3935-3938, officer_trait 3940-3943).
3. A region with no pool falls back to the first pool in the file, which is Mediterranean today (content.rs:1010-1018). Python falls back to Mediterranean by name (portlight@9b02494 src/portlight/content/officer_names.py:44).
4. Names are not unique: two officers aboard can share a name. Nothing reads the trait.
5. Each hire consumes two draws from the seeded stream (name, then trait). Changing the length of a pool can change what the stream yields; renaming a name in place does not (that is the form DF-38's proposed fix takes, ../data-fixes.md).

## 3. The six crew roles (code only; not in content.json)

<!-- hand-maintained:begin -->
| Crew role | Wage | Max per ship | Hire cost | Effect found in the sim | Cite |
|---|---|---|---|---|---|
| Sailor | 1 | no cap | port crew cost | no officer record; crew count | session.rs:3870-3875 |
| Gunner | 2 | 3 | 20 | casualty weight 1.5 only | session.rs:3876-3881; ship.rs:214 |
| Navigator | 3 | 1 | 30 | +0.5 voyage speed; casualty weight 0.5 | session.rs:3882-3887; ship.rs:124-130; voyage.rs:1043 |
| Surgeon | 3 | 1 | 30 | wage and casualty weight 0.5 only | session.rs:3888-3893; ship.rs:216 |
| Marine | 2 | 4 | 20 | wage and casualty weight 2.0 (x2 in boarding) only | session.rs:3894-3899; ship.rs:217, 223-225 |
| Quartermaster | 2 | 1 | 20 | daily wage bill cut by 10 percent; casualty weight 0.5 | session.rs:3900-3905; ship.rs:139-145 |
<!-- hand-maintained:end -->

Daily wage is the larger of the ship template's wage and the role wage (ship.rs:132-146). Hiring the Gunner role changes no firepower in the code read here (rg of .gunners finds only casualty and roster uses). The Python reference defines gunner, navigator storm, surgeon, marine and quartermaster sell effects (portlight@9b02494 src/portlight/engine/ship_stats.py:177-209) and rg of src finds no caller for them; whether Rust or Python should apply them is a design question, not logged as a DF in this slice (UNVERIFIED: not checked against Python tests).

Crew roles and companion roles share four names; they are separate systems (companions/roles/). Canon calls a hired role's person an officer and a named party member a companion.

## 4. Traits (15, all [flavour, no mechanic])

<!-- hand-maintained:begin -->
| Trait | Cite |
|---|---|
| loyal | C:6154 |
| cautious | C:6155 |
| bold | C:6156 |
| superstitious | C:6157 |
| sharp-eyed | C:6158 |
| steady | C:6159 |
| hot-tempered | C:6160 |
| quiet | C:6161 |
| gregarious | C:6162 |
| shrewd | C:6163 |
| resourceful | C:6164 |
| fearless | C:6165 |
| meticulous | C:6166 |
| jovial | C:6167 |
| stoic | C:6168 |
<!-- hand-maintained:end -->

The desk prints the trait as stored, and falls back to 'steady' if a trait is empty or not ASCII, and to 'Officer' for a name (logic.rs:2020-2022, 1369-1375).

## 5. Pools and name collisions

One file per pool (region key from `officer_names.regions`):

<!-- hand-maintained:begin -->
| Region | Names | Ports in the region (computed) | Cite | File |
|---|---|---|---|---|
| Mediterranean | 10 | 4 | C:6078 | [mediterranean.md](mediterranean.md) |
| North Atlantic | 10 | 3 | C:6093 | [north_atlantic.md](north_atlantic.md) |
| West Africa | 10 | 4 | C:6108 | [west_africa.md](west_africa.md) |
| East Indies | 10 | 6 | C:6123 | [east_indies.md](east_indies.md) |
| South Seas | 10 | 3 | C:6138 | [south_seas.md](south_seas.md) |
<!-- hand-maintained:end -->

Pool names that are also the given name of an authored person (19 names; computed by whole-word match against port NPCs, companions, merchants, style masters and skill trainers; the match is on the written name, so kinship is never implied):

<!-- hand-maintained:begin -->
| Pool | Name | Pool cite | Authored person with that name | Cite |
|---|---|---|---|---|
| Mediterranean | Marco | C:6080 | Marco (merchant, Porto Novo) | C:5871 |
| Mediterranean | Fatima | C:6083 | Fatima (merchant, Al-Manar) | C:5886 |
| Mediterranean | Dimitri | C:6086 | Dimitri Andros (Master Shipwright, Porto Novo) | C:7074 |
| North Atlantic | Henrik | C:6097 | Forge Master Henrik Brandt (Guild Master, Ironhaven) | C:7983 |
| North Atlantic | Gunnar | C:6101 | Gunnar Stahl (Harbor Master, Ironhaven) | C:7961 |
| North Atlantic | Astrid | C:6102 | Astrid Vekhren (Pact Commissioner, Ironhaven); Elder Astrid (Town Elder, Thornport) | C:8005; C:8445 |
| West Africa | Kwame | C:6110 | Elder Kwame (Council Elder, Palm Cove) | C:8913 |
| West Africa | Kofi | C:6112 | Kofi Asante (Cotton Master, Sun Harbor); Foreman Kofi Mensah (Mine Foreman, Iron Point); Kofi (merchant, Palm Cove) | C:8701; C:9093; C:5977 |
| West Africa | Adaeze | C:6113 | Adaeze (merchant, Sun Harbor) | C:5961 |
| West Africa | Mariam | C:6115 | Coin Mariam (companion, Al-Manar) | C:5673 |
| West Africa | Yusuf | C:6118 | Yusuf Diallo (Provisions Master, Sun Harbor) | C:8723 |
| West Africa | Zara | C:6119 | Inspector Zara Osman (Chief Customs Inspector, Al-Manar) | C:7347 |
| East Indies | Rajan | C:6125 | Rajan the Tinker (skill trainer, Monsoon Reach) | C:5427 |
| East Indies | Kamala | C:6134 | Kamala Nair (Broker, Monsoon Reach) | C:9861 |
| South Seas | Tane | C:6140 | War Chief Tane (War Chief, Coral Throne); Tane (merchant, Coral Throne) | C:11346; C:6047 |
| South Seas | Moana | C:6141 | Dive Boss Moana (Pearl Boss, Typhoon Anchorage) | C:11129 |
| South Seas | Rangi | C:6142 | Storm Chief Rangi (Storm Chief, Typhoon Anchorage) | C:11087 |
| South Seas | Leilani | C:6143 | Leilani Bonecatcher (style master, Typhoon Anchorage) | C:6246 |
| South Seas | Koa | C:6146 | Harbor Master Koa (Chain Master, Typhoon Anchorage) | C:11108 |
<!-- hand-maintained:end -->

The 19 names agree with DF-38's list. Resolved in lore: a hired officer is a namesake of the authored person, not that person; aboard, two officers who share a name are told apart by role ('Kofi the navigator'). DF-38 (renaming the pool) is declined for now.

## 6. Canon conventions

- Officers have no lore of their own. Canon does not invent named officers: any it needed would be [invented] and need design review. This slice writes none.
- The word 'officer' means the generated record. The word 'companion' means the authored party member (companions/README.md).
- Each pool file gives the region's voice from data (greeting, proverb, cultural name) and one [invented] line on how names are worn aboard; it adds no real-world place and no appearance.

## 7. Links

- Companions: [companions/README.md](../companions/README.md)
- Pool files: [Mediterranean](mediterranean.md), [North Atlantic](north_atlantic.md), [West Africa](west_africa.md), [East Indies](east_indies.md), [South Seas](south_seas.md)
- Open data issues: [data-fixes.md](../data-fixes.md) (DF-38; resolved-in-lore rows)

## F. Flags

- DF-38: 19 pool names reuse authored given names. Logged, proposed fix recorded, declined and resolved in lore.
- The crew-role effect gap (surgeon, marine, gunner and quartermaster sell effects defined in Python and applied in neither port) is noted in section 3 and has no DF number in this slice.

## G. Gaps in the data (computed from content.json at the base commit)

Regions in `ports[*].region` with no name pool (the sim then uses the first pool, content.rs:1010-1018):

None: every region that has a port has a pool (5 regions, 5 pools).

There are no authored officers, so there is no officer-per-port gap to list.

## H. Invented filler (not in the game)

None written. This section is empty by default. If design review asks for it, it may hold at most one line per gap, each tagged [invented]; no stats, no name that already exists in data, no file per filler, and never linked from a port file as fact.
