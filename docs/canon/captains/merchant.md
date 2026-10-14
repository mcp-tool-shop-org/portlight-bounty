# The Merchant (merchant)

Base commit: 9acf9e2 (content.json, the sim and the godot UI as cited). Status: draft canon, phase 4. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Name | The Merchant | C:2318 |
| Title | Licensed Trader | C:2319 |
| Home port | [Porto Novo](../ports/porto_novo.md) | C:2321 |
| Home region (data) | Mediterranean | C:2320 |
| Mentor | Marta Soares, Guild Factor, at [Porto Novo](../ports/porto_novo.md) | C:2322; C:7052-7054 |
| Starting silver | 550 | C:2323 |
| Starting ship | Coastal Sloop (sloop) | C:2324; C:133 |
| Starting provisions | 30 | C:2325 |
| Trade terms | buy x0.92 - sell x1.05 - luxury bonus 0.0 - port fees x0.7 | C:2327-2330 |
| Voyage terms | provision burn x1.0 - speed +0.0 - storm resist +0.0 - cargo damage x1.0 | C:2333-2336 |
| Inspection terms | chance x0.5 - seizure 0.0 - fines x0.6 | C:2339-2341 |
| Starting regard (data keys, not player terms) | commercial_trust 15, mediterranean 10 | C:2343 |
| Pirate regard | none | C:2351 |
| Player sees | "The Merchant  --  Licensed Trader  --  Porto Novo  --  sloop  --  550 silver" on the new game captain button | crates/portlight-godot/src/logic.rs:1355-1366; game.rs:2918-2925 |

## 1. The archetype (cited)

A guild-licensed trader of Porto Novo, schooled by the Exchange Guild's factor (C:2322; C:7052-7054).

## 2. Background (all [invented])

[invented] Came up counting sacks in the Grain Exchange and sails to prove the Alliance's posted-price creed works at sea.

Archetypes are roles, not fixed people: the player names the captain (custom_captain.rs, new game Name row, game.rs:2911-2916). Canon gives no gender, age, face or dress; the character hold stands.

## 3. Links

- Home port: [Porto Novo](../ports/porto_novo.md) (bloc [The Exchange Alliance](../factions/exchange_alliance.md))

## 4. Flags

- None.
