# The Smuggler (smuggler)

Base commit: 9acf9e2 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 4. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Name | The Smuggler | C:2356 |
| Title | Shadow Trader | C:2357 |
| Home port | [Palm Cove](../ports/palm_cove.md) | C:2359 |
| Home region (data) | West Africa | C:2358 |
| Mentor | Mama Joy, Tavern Keeper, at [Palm Cove](../ports/palm_cove.md) | C:2360; C:8934-8936 |
| Starting silver | 500 | C:2361 |
| Starting ship | Coastal Sloop (sloop) | C:2362; C:133 |
| Starting provisions | 35 | C:2363 |
| Trade terms | buy x1.0 - sell x0.98 - luxury bonus 0.28 - port fees x0.9 | C:2365-2368 |
| Voyage terms | provision burn x0.85 - speed +0.0 - storm resist +0.0 - cargo damage x0.8 | C:2371-2374 |
| Inspection terms | chance x1.3 - seizure 0.05 - fines x1.2 | C:2377-2379 |
| Starting regard (data keys, not player terms) | customs_heat 8, west_africa 5 | C:2381 |
| Pirate regard | none | C:2389 |
| Player sees | "The Smuggler  --  Shadow Trader  --  Palm Cove  --  sloop  --  500 silver" on the new game captain button | crates/portlight-godot/src/logic.rs:1355-1366; game.rs:2918-2925 |

## 1. The archetype (cited)

A Palm Cove runner taught by the tavern keeper Mama Joy (C:2360; C:8934-8936). Starts already known to customs (C:2383); profits on luxuries (luxury_sell_bonus 0.28).

## 2. Background (all [invented])

[invented] Learned the trade from what Mama Joy overheard and did not repeat; treats the Compact's generosity as a debt to honour.

Archetypes are roles, not fixed people: the player names the captain (custom_captain.rs, new game Name row, game.rs:2911-2916). Canon gives no gender, age, face or dress; the character hold stands.

## 3. Links

- Home port: [Palm Cove](../ports/palm_cove.md) (bloc [The Gold Coast Compact](../factions/gold_coast_compact.md))

## 4. Flags

- None.
