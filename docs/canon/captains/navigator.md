# The Navigator (navigator)

Base commit: 9acf9e2 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 4. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Name | The Navigator | C:2394 |
| Title | Route Specialist | C:2395 |
| Home port | [Silva Bay](../ports/silva_bay.md) | C:2397 |
| Home region (data) | Mediterranean | C:2396 |
| Mentor | Elena Madeira, Master Shipwright, at [Silva Bay](../ports/silva_bay.md) | C:2398; C:7492-7494 |
| Starting silver | 450 | C:2399 |
| Starting ship | Coastal Sloop (sloop) | C:2400; C:133 |
| Starting provisions | 30 | C:2401 |
| Trade terms | buy x1.05 - sell x1.0 - luxury bonus 0.0 - port fees x1.0 | C:2403-2406 |
| Voyage terms | provision burn x0.7 - speed +1.5 - storm resist +0.15 - cargo damage x0.7 | C:2409-2412 |
| Inspection terms | chance x1.0 - seizure 0.0 - fines x1.0 | C:2415-2417 |
| Starting regard (data keys, not player terms) | commercial_trust 5, mediterranean 5, east_indies 5 | C:2419 |
| Pirate regard | none | C:2427 |
| Player sees | "The Navigator  --  Route Specialist  --  Silva Bay  --  sloop  --  450 silver" on the new game captain button | crates/portlight-godot/src/logic.rs:1355-1366; game.rs:2918-2925 |

## 1. The archetype (cited)

A Silva Bay route specialist under Elena Madeira (C:2398; C:7492-7494). Fastest and most storm-hardy start (speed_bonus 1.5, storm_resist_bonus 0.15).

## 2. Background (all [invented])

[invented] Grew up among the Brotherhood's carved ship names and reads weather the way the yard reads grain in timber.

Archetypes are roles, not fixed people: the player names the captain (custom_captain.rs, new game Name row, game.rs:2911-2916). Canon gives no gender, age, face or dress; the character hold stands.

## 3. Links

- Home port: [Silva Bay](../ports/silva_bay.md) (bloc [The Exchange Alliance](../factions/exchange_alliance.md))

## 4. Flags

- None.
