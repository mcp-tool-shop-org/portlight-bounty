# Player types canon index

Base commit: c413788. Status: draft canon, phase 5. Hand-maintained from content.json `captains` (C:2315); there is no generator.

Mike's ask: build out unique qualities for each player type. A player type is a catalog captain archetype (`captain_type`, crates/portlight-sim/src/model.rs:502). The captains/ files are the who; these files are how each type plays. Custom captains are out of scope. Mike's names: Trader = The Merchant, Pirate = The Corsair, Bounty Hunter = The Bounty Hunter.

## Tags

- [exists file:line] - in the sim today
- [proposal; planned slice PT-Sn] - decided, not shipped; the tag stays until the slice merges, then becomes [exists]
- [invented] - lore; [flavour, no mechanic] - data with no reader

## Comparison

| Type | Also called | Silver | Ship | Buy - sell | Fees | Inspection | Burn | Signature quality | File |
|---|---|---|---|---|---|---|---|---|---|
| The Merchant | Trader | 550 | Coastal Sloop | x0.92 - x1.05 | x0.7 | x0.5 | x1.0 | clean-sale trust (exists) | [merchant.md](merchant.md) |
| The Smuggler | - | 500 | Coastal Sloop | x1.0 - x0.98 | x0.9 | x1.3 | x0.85 | five sim branches (exists) | [smuggler.md](smuggler.md) |
| The Navigator | - | 450 | Coastal Sloop | x1.05 - x1.0 | x1.0 | x1.0 | x0.7 | star line, x0.7 burn (exists) | [navigator.md](navigator.md) |
| The Privateer | - | 475 | Swift Cutter | x0.95 - x1.02 | x0.6 | x0.3 | x0.9 | letter of marque (PT-S3) | [privateer.md](privateer.md) |
| The Corsair | Pirate | 500 | Coastal Sloop | x1.05 - x0.95 | x0.5 | x1.5 | x0.9 | pirate-port fee cut (PT-S4) | [corsair.md](corsair.md) |
| The Scholar | - | 400 | Coastal Sloop | x1.0 - x1.0 | x0.85 | x0.7 | x0.75 | route insight (PT-S5) | [scholar.md](scholar.md) |
| The Merchant Prince | - | 700 | Coastal Sloop | x0.9 - x1.08 | x0.65 | x0.4 | x1.0 | family obligation (PT-S7) | [merchant_prince.md](merchant_prince.md) |
| The Dockhand | - | 300 | Coastal Sloop | x1.0 - x1.0 | x1.0 | x1.0 | x0.85 | self-made growth (PT-S6) | [dockhand.md](dockhand.md) |
| The Bounty Hunter | Bounty Hunter | 425 | Swift Cutter | x1.05 - x0.98 | x0.7 | x0.6 | x0.9 | bounty reward +15 pct (PT-S2) | [bounty_hunter.md](bounty_hunter.md) |

Values: C:2315-2660 (captains). Code branches exist only for The Smuggler (5), The Merchant (1) and The Navigator (1) (crates/portlight-sim/src/reputation.rs:58-60, 107-109; voyage.rs:295-300, 473-475; encounter.rs:49-56; campaign.rs:1304-1307).

## Balance note

- On paper The Merchant Prince is the strongest start (most silver, best trade terms) and The Dockhand the weakest. Canon must not widen that gap.
- Planned slices, in order, each its own gated PR after DF-22 and docked-consistency: PT-S0 balance sweep - PT-S1 favoured contracts (data only) - PT-S2 Bounty Hunter - PT-S3 Privateer - PT-S4 Corsair - PT-S5 Scholar - PT-S6 Dockhand - PT-S7 Merchant Prince. The Merchant, The Smuggler and The Navigator get no new perk unless PT-S0 says so.
- Guardrail: each type's median day-60 net worth within +/-15 pct of the field median; no type best at trade, combat and route time at once.

## Sound

Type files cite only the seven existing ids: sfx_ui_newgame_start, sfx_ui_port_open, sfx_chart_sail_depart, sfx_chart_next_day, sfx_ui_trade_coin, sfx_ui_contracts_open, sfx_ui_contract_accept (crates/portlight-godot/src/game.rs:2111, 3137, 4529, 4651, 4749, 6826, 6861). Anything else is an idea, no asset requested.

## Rules

- Words only for look and feel; character plates stay held; no face, age, gender or dress.
- No spillover claims (DF-28). Port treatment never implies a per-port mechanic unless tagged [proposal].
- Phase 5 data issues are DF-29 to DF-33 in ../data-fixes.md.
