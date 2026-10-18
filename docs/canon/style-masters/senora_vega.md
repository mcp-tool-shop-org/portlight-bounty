# Senora Vega (senora_vega)

Base commit: af3d105 (merged main, which includes phase 6A; content.json, the sim and the godot UI as cited are unchanged since 515a488). Status: draft canon, phase 6. Written 2026-10-17. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n at af3d105. Python cites read portlight@9b02494 path:line. Legend: [exists] code or data fact with a cite | [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | UNVERIFIED | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

Every table row below that carries a cite is [exists] (a code or data fact); rows tagged otherwise say so.

| Key | Value | Cite |
|---|---|---|
| Catalog name | Senora Vega [ascii-folded: the data name has a tilde on the n] | C:6182 |
| Teaches | La Destreza ([style table](README.md#3-styles-5)) | C:6183 |
| Home port | [Al-Manar](../ports/al_manar.md) | C:6184 |
| Bloc of the home port | [The Exchange Alliance](../factions/exchange_alliance.md) | ports/al_manar.md section 0 |
| Region | Mediterranean | C:4133 |
| Training cost | 80 silver | C:4163 |
| Training time | 5 days | C:4164 |
| Styles needed first | 0 | C:4165 |
| Body parts the style needs (live gate, DF-41 resolved) | hand, arm | C:4166 |
| Player sees today | the style row at this port (name, silver, days, styles needed, known yes or no), a confirm line, a notice after training, and a learned-style row with an `Equip` control; never the master | logic.rs:2028-2062; logic.rs:1884; game.rs:5693-5714 |
| Master text on desk | master name, description, dialog, and style description are shown on the Crew desk (DF-39 resolved) | logic.rs; game.rs |
| Mechanic status | masters_at_port gates training alongside the style's port list, and learned styles can be equipped (DF-39, DF-41 resolved) | training.rs:9-52; session.rs |
| Python record | same strings except description (`COPY_OVERRIDES`: 'runs a fencing school east of Gibraltar' vs Python's 'runs the only fencing school east of Gibraltar', DF-40 resolved); Python's train command prints the master name and dialog | portlight@9b02494 src/portlight/content/fighting_styles.py:266; src/portlight/app/cli.py:2426-2455 |

## 1. Grounded facts (every data line quoted)

- description: "A Castilian expatriate who runs a fencing school east of Gibraltar." (C:6185; `COPY_OVERRIDES` in tools/extract_content.py, DF-40 resolved)
- dialog (a spoken line, quoted inside the data string): "You fight like a dockhand. I will teach you to fight like a mathematician." (C:6186)
- style, description: "The Spanish science of defense. Every movement calculated, every thrust placed with geometric precision." (C:4134)
- style, historical note (real-world, shipped data): "Systematized by Jeronimo Sanchez de Carranza in 1569. The first European martial art treated as an academic discipline." (C:4135)
- style, special action: Estocada (estocada): attack, stamina 3, beats parry, slash, loses to dodge, damage +2, cooldown 2 (C:4143)
- style, special action flavour: "A geometric thrust - the blade finds the only open line." (C:4156)

Quotes are as in content.json; an em dash is folded to ' - ' and a tilde n to n, and both are tagged [ascii-folded].

## 2. Role in play (what the sim does today; no proposals here)

- [exists] Training gate: the sim never reads this master. A captain may learn La Destreza at any port in the style's own list, which today is Porto Novo and Al-Manar (C:4159); the master's port is one of the two, so the data agrees but is redundant (training.rs:9-52, the check at 22-24; content.rs:1027-1032; training.rs:73-79; the only caller is a test, training.rs:101).
- [exists] Cost and time: 80 silver, then 5 days of the calendar, the same price at both ports; the master changes neither (training.rs:55-65).
- [exists] Injury gate: the sim checks the style's body parts against `injuries::injured_body_parts(&self.world.captain.injuries)` (session.rs, DF-41 resolved).
- [exists] After training: the style is stored in the captain's learned styles (training.rs:55-65) and can be equipped via `Session::equip_style` / the Crew screen's `Equip` button when uninjured in its required body parts (DF-41 resolved).
- [exists] What the Godot desk prints, built in code not authored [exists]: the row 'La Destreza  80 silver  5 days  needs 0 styles  known no' (logic.rs:2041-2045); the confirm line 'Train La Destreza for 80 silver. The calendar will advance 5 days.' (logic.rs:1884); the notice 'Learned La Destreza. The calendar advanced N days.' with the catalog name (logic.rs:1936-1946; DF-40 resolved).
- [exists] Text reach: the master's name, description and dialog have no reader in the crates (rg for style_master and masters_at in crates/portlight-godot/src finds nothing; the sim has the one test caller). Python prints the name and the dialog in its training view (cli.py:2426-2455; combat_views.py:359-412).

## 3. Look and feel (words only; the character hold stands)

- Voice [cite]: dry and exacting: she measures a pupil against a mathematician, and the pupil loses - "You fight like a dockhand. I will teach you to fight like a mathematician." (C:6186).
- Manner [invented] She speaks in questions that have one right answer and waits for it.
- Setting [cite]: "A Castilian expatriate who runs a fencing school east of Gibraltar." (C:6185). [invented] The school is a walled courtyard off the spice quarter, with a gallery for parents and a ledger of fees on the gatepost.
- No face, age, build or dress is set beyond the quoted data lines; canon adds none. Character plates are held.

## 4. [invented] lore

- [invented] She left Castile with a reputation and little else, and chose Al-Manar because its court honours a teacher who keeps accounts.
- [invented] Her pupils are factors' children and the odd customs clerk; she is proud of the word 'school' and quarrels with anyone who uses it for the warehouse at Porto Novo.

## 5. Hooks ([proposal] only; never a mechanic)

- [proposal] Copy hook: a line in which she calls herself 'a' school of the Middle Sea; flavour only.
- Sound: ambience only. No sfx id is cited.

## 6. Links

- Home port: [Al-Manar](../ports/al_manar.md); bloc [The Exchange Alliance](../factions/exchange_alliance.md)
- Style index: [style-masters/README.md](README.md)
- Other master of the same style: [Maestro Luciano](maestro_luciano.md) (Porto Novo)
- [Coin Mariam](../companions/coin_mariam.md) - the companion at the same port
- Crew hired in this region: [officers/mediterranean.md](../officers/mediterranean.md)

## 7. Flags and open questions

- DF-39 (masters and master text unread): see ../data-fixes.md. DF-41 (resolved): the style injury gate and `equip_style` / `Equip` control are wired.
- DF-40 (resolved): description now reads 'runs a fencing school east of Gibraltar' (C:6185) via `COPY_OVERRIDES` in tools/extract_content.py, reconciling with Maestro Luciano at Porto Novo (C:6183, C:4159).
- Wiring master text into the desk is out of scope for canon; it is a design decision.
