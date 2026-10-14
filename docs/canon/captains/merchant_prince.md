# The Merchant Prince (merchant_prince)

Base commit: 4f08125 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 4. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Name | The Merchant Prince | C:2550 |
| Title | Khoury Dynasty Scion | C:2551 |
| Home port | [Al-Manar](../ports/al_manar.md) | C:2553 |
| Home region (data) | Mediterranean | C:2552 |
| Mentor | Senhora Nadia Khoury, Merchant Princess, at [Al-Manar](../ports/al_manar.md) | C:2554; C:7302-7304 |
| Starting silver | 700 | C:2555 |
| Starting ship | Coastal Sloop (sloop) | C:2556; C:133 |
| Starting provisions | 30 | C:2557 |
| Trade terms | buy x0.9 - sell x1.08 - luxury bonus 0.12 - port fees x0.65 | C:2559-2562 |
| Voyage terms | provision burn x1.0 - speed +0.0 - storm resist +0.0 - cargo damage x1.0 | C:2565-2568 |
| Inspection terms | chance x0.4 - seizure 0.0 - fines x0.4 | C:2571-2573 |
| Starting regard (data keys, not player terms) | commercial_trust 18, mediterranean 12, east_indies 5 | C:2575 |
| Pirate regard | none | C:2583 |
| Player sees | "The Merchant Prince  --  Khoury Dynasty Scion  --  Al-Manar  --  sloop  --  700 silver" on the new game captain button | crates/portlight-godot/src/logic.rs:1355-1366; game.rs:2967-2974 |

## 1. The archetype (cited)

A scion of the Khoury dynasty of Al-Manar under Senhora Nadia Khoury (C:2551-2554; C:7302-7304), whose family has held the title for six generations (C:7182). The richest start (700 silver).

## 2. Background (all [invented])

[invented] Sent to sea to earn the name before inheriting it, under the eye of Inspector-minded relatives.

Archetypes are roles, not fixed people: the player names the captain (custom_captain.rs, new game Name row, game.rs:2960-2965). Canon gives no gender, age, face or dress; the character hold stands.

## 3. Links

- Home port: [Al-Manar](../ports/al_manar.md) (bloc [The Exchange Alliance](../factions/exchange_alliance.md))

## 4. Flags

- None.
