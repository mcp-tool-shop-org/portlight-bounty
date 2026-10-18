# Captain canon index

Base commit: 4f08125. Status: draft canon, phase 4. Hand-maintained from content.json `captains` (C:2315), `pirate_captains` (C:2913) and `captain_loot` (C:5027); there is no generator.

## Player archetypes (9)

The new game screen lists these in `CAPTAIN_ORDER` (crates/portlight-sim/src/custom_captain.rs:432-442; iterated by `starting_captains()` at custom_captain.rs:479-505), one button each: name - title - home port - ship class - silver (crates/portlight-godot/src/logic.rs:1355-1366; game.rs:2967-2974). A custom captain is a tenth option and is not canon (game.rs:2975-2979).

| Archetype | Title | Home port | Region (data) | Mentor | Silver | Ship | Pirate regard | Cite | File |
|---|---|---|---|---|---|---|---|---|---|
| The Merchant | Licensed Trader | [Porto Novo](../ports/porto_novo.md) | Mediterranean | Marta Soares | 550 | Coastal Sloop | - | C:2317 | [merchant.md](merchant.md) |
| The Smuggler | Shadow Trader | [Palm Cove](../ports/palm_cove.md) | West Africa | Mama Joy | 500 | Coastal Sloop | - | C:2355 | [smuggler.md](smuggler.md) |
| The Navigator | Route Specialist | [Silva Bay](../ports/silva_bay.md) | Mediterranean | Elena Madeira | 450 | Coastal Sloop | - | C:2393 | [navigator.md](navigator.md) |
| The Privateer | Letter of Marque | [Stormwall](../ports/stormwall.md) | North Atlantic | Commander Elias Vogt | 475 | Swift Cutter | The Iron Wolves -5 | C:2431 | [privateer.md](privateer.md) |
| The Corsair | Crimson Tide Captain | [Corsair's Rest](../ports/corsairs_rest.md) | Mediterranean | No One | 500 | Coastal Sloop | The Crimson Tide 25 | C:2471 | [corsair.md](corsair.md) |
| The Scholar | Wind Temple Student | [Monsoon Reach](../ports/monsoon_reach.md) | East Indies | Brother Anand | 400 | Coastal Sloop | - | C:2511 | [scholar.md](scholar.md) |
| The Merchant Prince | Khoury Dynasty Scion | [Al-Manar](../ports/al_manar.md) | Mediterranean | Senhora Nadia Khoury | 700 | Coastal Sloop | - | C:2549 | [merchant_prince.md](merchant_prince.md) |
| The Dockhand | Self-Made Captain | [Crosswind Isle](../ports/crosswind_isle.md) | East Indies | Dock Master Tao | 300 | Coastal Sloop | - | C:2587 | [dockhand.md](dockhand.md) |
| The Bounty Hunter | Licensed Hunter | [Crosswind Isle](../ports/crosswind_isle.md) | East Indies | Dock Master Tao | 425 | Swift Cutter | - | C:2625 | [bounty_hunter.md](bounty_hunter.md) |

## Pirate captains (8)

Named in encounters and the duel prompt (logic.rs:1086-1094, 2203-2212). Personality drives duel and combat stance weights (duel.rs:43-50; combat.rs:593-620).

| Captain | Faction | Personality | Strength | Sighting | Cite | File |
|---|---|---|---|---|---|---|
| Scarlet Ana | The Crimson Tide | balanced | 6 | C:12816 (Corsair's Rest) | C:2915 | [scarlet_ana.md](pirates/scarlet_ana.md) |
| The Butcher | The Crimson Tide | aggressive | 8 | - | C:2922 | [the_butcher.md](pirates/the_butcher.md) |
| Raj the Quiet | The Monsoon Syndicate | defensive | 5 | - | C:2929 | [raj_the_quiet.md](pirates/raj_the_quiet.md) |
| Typhoon Mei | The Monsoon Syndicate | wild | 7 | C:12894 (Spice Narrows) | C:2936 | [typhoon_mei.md](pirates/typhoon_mei.md) |
| Old Coral | The Deep Reef Brotherhood | defensive | 6 | - | C:2943 | [old_coral.md](pirates/old_coral.md) |
| The Diver | The Deep Reef Brotherhood | aggressive | 7 | - | C:2950 | [the_diver.md](pirates/the_diver.md) |
| Sergeant Kruze | The Iron Wolves | balanced | 7 | C:12840 (Ironhaven) | C:2957 | [sergeant_kruze.md](pirates/sergeant_kruze.md) |
| Gnaw | The Iron Wolves | aggressive | 9 | - | C:2964 | [gnaw.md](pirates/gnaw.md) |

## Rules

- Archetypes are roles; the player names the captain. Canon gives archetypes a background but no gender, age, face or dress.
- Pirate captains are people, but canon sets no appearance, dress, ship art or portrait. Quoted sighting text stays data. The character hold stands; character plates are held.
- Four pirates (The Butcher, Old Coral, The Diver, Gnaw) have only a record in data. Canon keeps them thin; any backstory beyond one [invented] line is a future data line.
- Named NPCs (governors, mentors, merchants) stay in their port files. This index links mentors through the archetype files.
- Companions, officer names, style masters, skill trainers and merchants: see [companions/README.md](../companions/README.md), [officers/README.md](../officers/README.md), [style-masters/README.md](../style-masters/README.md), [skill-trainers/README.md](../skill-trainers/README.md) and [merchants/README.md](../merchants/README.md) (phases 6A, 6B and 6C).
