# Style master canon index

Base commit: af3d105 (merged main, which includes phase 6A; content.json, the sim and the godot UI as cited are unchanged since 515a488). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at af3d105. Python cites read portlight@9b02494 path:line. Legend: [exists] code or data fact with a cite | [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

Every table row below that carries a cite is [exists] (a code or data fact); rows tagged otherwise say so.

| Key | Value | Cite |
|---|---|---|
| Style masters (named, authored) | 10 | C:6171 |
| Fighting styles | 5 | C:4129 |
| Masters per style | 2 (one at each of the style's two training ports) | C:6171 |
| Ports with a master | 10 of 20 | style_masters[*].port_id |
| Region coverage | each of the 5 regions has 2 masters | style_masters[*].port_id |
| Training gate in the sim | the style's own port list, silver, styles known, and (never, today) injuries | training.rs:9-52 |
| Rust record shape | id, name, style, port, description, dialog | content.rs:401-408 |
| Python reference | same 10 records, same strings (compared field by field, no differences) | portlight@9b02494 src/portlight/content/fighting_styles.py:251-392 |

Masters are the authored teachers of the five fighting styles. Skill trainers (the blacksmith skill) are a separate index: [skill-trainers/README.md](../skill-trainers/README.md). Port NPCs, merchants, companions and officers stay in their own files.

## 1. What the player sees today

Every table row below that carries a cite is [exists] (a code or data fact); rows tagged otherwise say so.

| What | Reaches the player? | Cite |
|---|---|---|
| One row per style taught at the docked port: 'Name  N silver  N days  needs N styles  known yes or no' | Yes (the row is built in code; 'needs 1 styles' is the shipped plural) | logic.rs:2028-2062 (row at 2041-2045) |
| 'Known styles: none' or the list of learned style names | Yes | logic.rs:2051-2062 |
| 'No fighting master at this port.' | Yes, at a port that teaches no style: the only line where the word master reaches the player | logic.rs:1864; game.rs:5897-5898 |
| A confirm line: 'Train Name for N silver. The calendar will advance N days.' | Yes, every training asks first | logic.rs:1884; game.rs:5608-5625 |
| A notice after training: 'Learned dambe. The calendar advanced N days.' | Yes, with the raw id (DF-40) | game.rs:5693-5714 (line 5708) |
| Master name, description, dialog | No: no reader in the crates | content.rs:401-408 (fields only); rg of crates |
| Style description, historical note, passives, special action | No: the desk shows name, cost, days, styles needed, known | logic.rs:2028-2062 |
| Style passives and special action in a duel | No in a game started in Rust: no code sets the active style | combat.rs:273-283; model.rs:523; DF-41 |
| An injury refusal | No: the gate gets an empty list | session.rs:3337; DF-41 |
| Character art | None: the character hold stands | crew_screen.rs:3-6 |

## 2. Index (10 masters)

<!-- hand-maintained:begin -->
| Master | Style | Port | Region | Bloc | Cite | File |
|---|---|---|---|---|---|---|
| Maestro Luciano | La Destreza | [Porto Novo](../ports/porto_novo.md) | Mediterranean | [The Exchange Alliance](../factions/exchange_alliance.md) | C:6174 | [maestro_luciano.md](maestro_luciano.md) |
| Senora Vega | La Destreza | [Al-Manar](../ports/al_manar.md) | Mediterranean | [The Exchange Alliance](../factions/exchange_alliance.md) | C:6182 | [senora_vega.md](senora_vega.md) |
| Red Hamish | Highland Broadsword | [Ironhaven](../ports/ironhaven.md) | North Atlantic | [The Iron Pact](../factions/iron_pact.md) | C:6190 | [red_hamish.md](red_hamish.md) |
| Jarl Solveig | Highland Broadsword | [Thornport](../ports/thornport.md) | North Atlantic | [The Free Ports](../factions/free_ports.md) | C:6198 | [jarl_solveig.md](jarl_solveig.md) |
| Kolo the Fist | Dambe | [Sun Harbor](../ports/sun_harbor.md) | West Africa | [The Gold Coast Compact](../factions/gold_coast_compact.md) | C:6206 | [kolo_the_fist.md](kolo_the_fist.md) |
| Nneka Ironfist | Dambe | [Pearl Shallows](../ports/pearl_shallows.md) | West Africa | [The Gold Coast Compact](../factions/gold_coast_compact.md) | C:6214 | [nneka_ironfist.md](nneka_ironfist.md) |
| Pak Reza | Silat | [Jade Port](../ports/jade_port.md) | East Indies | [The Silk Circle](../factions/silk_circle.md) | C:6222 | [pak_reza.md](pak_reza.md) |
| Dewi Ayu | Silat | [Monsoon Reach](../ports/monsoon_reach.md) | East Indies | [The Free Ports](../factions/free_ports.md) | C:6230 | [dewi_ayu.md](dewi_ayu.md) |
| Kahoku the Elder | Lua | [Coral Throne](../ports/coral_throne.md) | South Seas | [The Coral Crown](../factions/coral_crown.md) | C:6238 | [kahoku_the_elder.md](kahoku_the_elder.md) |
| Leilani Bonecatcher | Lua | [Typhoon Anchorage](../ports/typhoon_anchorage.md) | South Seas | [The Coral Crown](../factions/coral_crown.md) | C:6246 | [leilani_bonecatcher.md](leilani_bonecatcher.md) |
<!-- hand-maintained:end -->

The cite is the name line; every field line is in the entity file. Rows are in data order.

## 3. Styles (5)

<!-- hand-maintained:begin -->
| Style | Region | Training ports | Silver | Days | Styles needed | Body parts | Cite |
|---|---|---|---|---|---|---|---|
| La Destreza | Mediterranean | [Porto Novo](../ports/porto_novo.md) and [Al-Manar](../ports/al_manar.md) | 80 | 5 | 0 | hand, arm | C:4159, C:4163, C:4166 |
| Highland Broadsword | North Atlantic | [Ironhaven](../ports/ironhaven.md) and [Thornport](../ports/thornport.md) | 100 | 7 | 0 | arm | C:4200, C:4204, C:4207 |
| Dambe | West Africa | [Sun Harbor](../ports/sun_harbor.md) and [Pearl Shallows](../ports/pearl_shallows.md) | 60 | 6 | 0 | leg | C:4240, C:4244, C:4247 |
| Silat | East Indies | [Jade Port](../ports/jade_port.md) and [Monsoon Reach](../ports/monsoon_reach.md) | 120 | 8 | 1 | hand, leg | C:4280, C:4284, C:4287 |
| Lua | South Seas | [Coral Throne](../ports/coral_throne.md) and [Typhoon Anchorage](../ports/typhoon_anchorage.md) | 150 | 10 | 1 | hand, arm | C:4321, C:4325, C:4328 |
<!-- hand-maintained:end -->

Silat and Lua each need one known style first (C:4286, C:4327), so a captain meets them after another style; the other three need none.

<!-- hand-maintained:begin -->
| Style | Passives (non-zero) | Special action | Cite |
|---|---|---|---|
| La Destreza | thrust 1, parry 1 | Estocada (estocada): attack, stamina 3, beats parry, slash, loses to dodge, damage +2, cooldown 2 | C:4136-4142, C:4143 |
| Highland Broadsword | slash 1, hp 2 | Cleave (cleave): attack, stamina 2, beats parry, loses to thrust, dodge, damage +4, cooldown 1 | C:4177-4183, C:4184 |
| Dambe | dodge counter 1 | Leg Sweep (leg_sweep): counter, stamina 2, beats thrust, loses to slash, dodge, damage +1, cooldown 3 | C:4217-4223, C:4224 |
| Silat | ranged accuracy 0.15 | Keris Strike (keris_strike): attack, stamina 3, beats slash, loses to parry, dodge, damage +1, cooldown 2 | C:4257-4263, C:4264 |
| Lua | parry 1, injury 0.2 | Joint Lock (joint_lock): utility, stamina 3, beats parry, loses to thrust, slash, damage +0, cooldown 3 | C:4298-4304, C:4305 |
<!-- hand-maintained:end -->

All passives and special actions are [flavour, no mechanic] for a Rust-started game: nothing equips a style (DF-41). They are authored and wired in combat.rs, and a save that carries an active style from Python would use them (save.rs:934; combat.rs:104-112, 273-283).

<!-- hand-maintained:begin -->
| Style | Description (data) | Historical note (data, real-world) | Cite |
|---|---|---|---|
| La Destreza | "The Spanish science of defense. Every movement calculated, every thrust placed with geometric precision." | "Systematized by Jeronimo Sanchez de Carranza in 1569. The first European martial art treated as an academic discipline." | C:4134, C:4135 |
| Highland Broadsword | "The broadsword tradition of the Scottish Highlands and Norse raiders. No finesse, no subtlety - overwhelming force." | "Highland sword-and-targe techniques documented across Scotland and Scandinavia. The two-handed claymore was already legendary by the 1500s." | C:4175, C:4176 |
| Dambe | "The fighting art of the Hausa people. The lead fist is the spear, the rear hand is the shield, and the legs finish every fight." | "Practiced across West Africa since at least the 15th century. Dambe bouts were a central part of harvest festivals and market days." | C:4215, C:4216 |
| Silat | "The blade and grappling art of the Malay Archipelago. Low stances, curved blades, trapping hands, and devastating close-range techniques." | "Pencak Silat was already ancient when Portuguese traders first documented it in the 1500s. The kris dagger is both weapon and cultural artifact." | C:4255, C:4256 |
| Lua | "The bone-breaking art of Polynesia. Joint locks, pressure points, and techniques designed not to wound but to permanently disable." | "Hawaiian Lua was restricted to the warrior class. Its techniques were so dangerous that training was conducted in secret, at night, under oath." | C:4296, C:4297 |
<!-- hand-maintained:end -->

The two lore columns are data, shown to no player today (section 1). Quotes are ascii-folded.

## 4. How training works

- [exists] The call chain: the crew desk button asks to confirm, then calls train_crew, which calls can_learn_style and then learn_style, and advances the calendar one day at a time (game.rs:5608-5625, 5693-5714; session.rs:3331-3351; training.rs:9-65).
- [exists] can_learn_style refuses, in order: an unknown style; a style already known; a port not in the style's list ('No <style> master at this port.'); too little silver; too few styles known; injured body parts (training.rs:9-52). The desk lists only styles taught at the port, so the third refusal is not reachable from the desk (logic.rs:2028-2036).
- [exists] Masters play no part: masters_at (content.rs:1027-1032) and masters_at_port (training.rs:73-79) have no non-test caller (training.rs:101). All 10 master ports sit in their style's port list today (computed from C:6171-6250 and the style lists; DF-39).
- [exists] Injuries are real: a lost personal fight can add one to the captain (session.rs:1905-1912; model.rs:535), but train_crew passes an empty list, so the check never fires (session.rs:3337; crew_screen.rs:11-12; DF-41).
- [exists] Equip: Python has an equip-style command that sets the active style (portlight@9b02494 src/portlight/app/cli.py:2475-2504). Rust has no equivalent, and nothing else writes the active style (model.rs:523; world.rs:66; save.rs:934 loads it).

## 5. Duel screens: any style text?

- [exists] The Godot duel screen has no style word: rg for special, style and the style fields in encounter_screen.rs finds only UI style boxes (crates/portlight-godot/src/encounter_screen.rs).
- [exists] The only authored style text in the duel path is a special action's flavour line. It would show as the round's flavour only if a special action were offered, and one is offered only when the combatant has an active style (combat.rs:273-283; resolve_style_action, combat.rs:414-433; the flavour at 426; the round text path, session.rs:2231-2232, logic.rs:313-319). No Rust code sets the active style, so the line never shows in a game started in Rust (DF-41).
- [exists] The style description and historical note have no reader in the Godot crate; the sim reads only the passives and the special action (rg of crates).
- [exists] The built refusal 'Your Estocada is countered!' and 'Your Estocada meets resistance.' are code strings that use the special action's catalog name (combat.rs:429-431); they share the same unreachable path.

## 6. Python reference

- [exists] The 10 master records and 5 style records are the same in Rust and Python, field by field (portlight@9b02494 src/portlight/content/fighting_styles.py:56-63, 71-247, 251-392).
- [exists] Python's train command with no argument lists the masters at the port and prints the first master's dialog with the master's name; with a style it checks the captain's real injuries (src/portlight/app/cli.py:2426-2472; src/portlight/app/combat_views.py:359-412; src/portlight/engine/training.py:10-33, 54-56).
- [exists] Python's equip-style command sets the active style and refuses an injured captain (src/portlight/app/cli.py:2475-2504).
- [exists] Python's training view reads a 'prerequisites' key that the command does not pass (it passes 'prereqs'), so its 'Requires' column always reads none (src/portlight/app/combat_views.py:385; src/portlight/app/cli.py:2436-2446). Noted, not logged: Python is the reference, not the game.

## 7. Canon conventions

- Names are the catalog names; ids appear only in cite tables and file names. The Maestro, the Senora, the Jarl and the Pak are part of the catalog name; keep them.
- Quote every data line with its C:n; mark every addition [invented]; put contradictions in data-fixes.md and never fix the data in a canon PR.
- A master is never described as gating training or as visible to the player: the sim does neither (DF-39).
- The Senora Vega name is folded to ASCII here (the data carries a tilde on the n, in Rust and in Python) and tagged [ascii-folded]; the accent never reaches the desk, because no master name does.

## F. Flags

- DF-39: masters and the master text are unread; the real gate is the style's port list. DF-40: Senora Vega's 'only fencing school east of Gibraltar' contradicts Maestro Luciano; the train and skill notices print raw ids. DF-41: the style injury gate is dead and no path equips a style.
- Showing master text, wiring the injury list, or adding an equip control is out of scope for canon; it is a design decision.
- The shipped row says 'needs 1 styles' for Silat and Lua (logic.rs:2041-2045); a copy nit, not logged.

## G. Gaps in the data (computed from content.json at the base commit)

Ports in `ports[*].id` with no style master (no record has that port as `port_id`). Facts only; canon does not say these ports have or lack anyone else.

<!-- hand-maintained:begin -->
| Port | Region | Bloc |
|---|---|---|
| [Silva Bay](../ports/silva_bay.md) | Mediterranean | [The Exchange Alliance](../factions/exchange_alliance.md) |
| [Corsair's Rest](../ports/corsairs_rest.md) | Mediterranean | [The Shadow Ports](../factions/shadow_ports.md) |
| [Stormwall](../ports/stormwall.md) | North Atlantic | [The Iron Pact](../factions/iron_pact.md) |
| [Palm Cove](../ports/palm_cove.md) | West Africa | [The Gold Coast Compact](../factions/gold_coast_compact.md) |
| [Iron Point](../ports/iron_point.md) | West Africa | [The Gold Coast Compact](../factions/gold_coast_compact.md) |
| [Silk Haven](../ports/silk_haven.md) | East Indies | [The Silk Circle](../factions/silk_circle.md) |
| [Crosswind Isle](../ports/crosswind_isle.md) | East Indies | [The Free Ports](../factions/free_ports.md) |
| [Dragon's Gate](../ports/dragons_gate.md) | East Indies | [The Silk Circle](../factions/silk_circle.md) |
| [Spice Narrows](../ports/spice_narrows.md) | East Indies | [The Shadow Ports](../factions/shadow_ports.md) |
| [Ember Isle](../ports/ember_isle.md) | South Seas | [The Coral Crown](../factions/coral_crown.md) |
<!-- hand-maintained:end -->

10 of 20 ports have no style master. Masters by region (home ports, computed):

<!-- hand-maintained:begin -->
| Region | Ports in the region | Masters | Cite |
|---|---|---|---|
| Mediterranean | 4 | Maestro Luciano, Senora Vega | style_masters[*].port_id |
| North Atlantic | 3 | Red Hamish, Jarl Solveig | style_masters[*].port_id |
| West Africa | 4 | Kolo the Fist, Nneka Ironfist | style_masters[*].port_id |
| East Indies | 6 | Pak Reza, Dewi Ayu | style_masters[*].port_id |
| South Seas | 3 | Kahoku the Elder, Leilani Bonecatcher | style_masters[*].port_id |
<!-- hand-maintained:end -->

Every region has two masters; a region with none would show 'none' above.

## H. Invented filler (not in the game)

None written. This section is empty by default. If design review asks for it, it may hold at most one line per gap port, each tagged [invented], saying what a captain might plausibly meet there; no stats, prices or fees, no name that already exists in data, no file per filler, and never linked from a port file as fact.
