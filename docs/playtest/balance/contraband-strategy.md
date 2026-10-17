# Contraband strategy for the balance bot (design note)

Status: proposal, docs only. Nothing here changes the game, the bot or any tuned value.

## Why
The smuggler reads -43.3% against the field (30975 vs a median of 54686, day 60, seeds 1001-1030). Its three bot strategies are all plain trading: the bot never buys or sells contraband, so the smuggler's own trade (black markets, a detection edge) is not measured. The balance notes call the remaining gap a "perk-measured-by-bot gap" and gate any inspection change on a contraband strategy existing. This note says what that strategy should be, so the numbers it produces mean something.

## What the game does (read from the code)
- **Goods.** Three contraband goods: opium (base 85), black powder (65), stolen cargo (40). A contraband good can be sold only at a port with the `black_market` feature, and only two ports have it: Corsair's Rest (Mediterranean) and Spice Narrows (East Indies). Only those two ports stock contraband.
- **Prices at the start (seed 1001).** Black powder buys at 40 in Corsair's Rest and sells at 134 in Spice Narrows. Opium buys at 52 in Spice Narrows and sells at 150 in Corsair's Rest. Stolen cargo buys at 30-32 and sells at 26-28, so it has no margin. The two real legs are about +94 and +98 silver a unit.
- **Stock is thin.** Corsair's Rest opens with 8 black powder (target 6); Spice Narrows opens with 10 opium (target 8). Restock is 0.3-1.0 a day. A sloop holds 30. So one leg carries about 8-10 units, worth roughly 750-950 silver of margin.
- **Distance.** The direct lane between the two ports (80) needs a galleon: a sloop or cutter is blocked, a brigantine gets a warning. The open route is Corsair's Rest, Silva Bay, Al Manar, Monsoon Reach, Spice Narrows: distance 132, about 17 days for a sloop (speed 8). The smuggler starts at Palm Cove, about 58 distance (7-8 days) from Corsair's Rest.
- **Heat.** A contraband sale scores 4 suspicion points before margin and dump terms (plus 1 for a smuggler), so it usually reaches 6 and adds up to 8 regional heat, with a standing and trust penalty. Heat falls by `max(1, heat / 10)` each time the ship arrives in a port of that region.
- **Inspection at sea.** An inspection event can happen on any sea day. Its weight is multiplied by the captain's `inspection_chance_mult` (smuggler 1.3), by a heat factor (x1.2 from heat 15, x1.4 from 25, x1.8 from 40) and by the wanted level. When it fires with contraband aboard, detection is 0.40 plus 0.10 per point of the highest regional heat above 15, minus 0.15 for a smuggler, clamped to 0.1-0.9. A detection seizes all contraband and costs a fine of 3 x 30 x units (about 720 for 8 units) plus the usual fee, and a reputation hit. Without contraband the inspection is a small fee.

## Proposed strategy: S4, the contraband runner
A fourth bot strategy, `s4_contraband`, played by every captain type, so the smuggler's result can be read against what the other types get from the same trade.

1. **Setup.** Sail to the nearer black market by open lanes only (never a blocked lane), refit nothing, and keep the opportunist's provisions and hull rules. If no open route reaches both black markets within the day budget, the strategy trades like the cautious bot and records zero contraband.
2. **Goods and ports.** Buy only the good with a positive margin to the other black market (black powder at Corsair's Rest, opium at Spice Narrows), at most the stock shown and at most the hold. Never stolen cargo. Sell at the far black market the day the ship arrives.
3. **Day budget.** Start a contraband leg only if the open route's estimated days plus a two-day buffer end before day 60. Expect one round trip in a 60-day run, two legs.
4. **Heat rule.** Do not carry contraband when the highest regional heat is 20 or more (detection would be 0.75 or more for a smuggler, 0.9 for others). When heat is 15-19, carry at most half the stock. Otherwise trade legal goods on the way.
5. **Inspection handling.** The bot does not evade. A seizure is a recorded loss, not a retry.
6. **Everything else** (provisions, repairs, selling winners, sea encounters) is unchanged from the cautious strategy, so the difference between S4 and S1 is the contraband legs.

## What counts as success
This is a measurement, not a retune.
- The strategy is deterministic and plays all 9 types x 30 seeds, like the others.
- It is judged by whether the numbers are readable, not by a target score: contraband legs are attempted on most seeds for the smuggler, and the columns below add up (silver in the net worth change equals the traded margin minus fines).
- S4 is reported next to the three existing strategies but is **not** added to the best-of score, so the band, the field median and every existing row stay as they are. If design review wants S4 in the best-of, that is a separate decision.

## Columns (measurement only)
Added to the CSV and summary for S4 rows, zero elsewhere: `contraband_units_bought`, `contraband_units_sold`, `contraband_margin` (sale silver minus purchase silver), `inspections` (inspection events at sea), `contraband_seizures`, `contraband_fines`, `max_heat`, and `contraband_days` (days at sea carrying contraband). The summary gets one table: per type, the S4 median net worth, seeds that attempted a leg, margin per run, seizure rate.

## How it reads against the smuggler's -43.3% and the opportunist
- Ceiling. One round trip carries about 8-10 units each way at about +95 a unit: roughly +1,500 silver at best, or about +5% of the smuggler's 28-31k score. The gap to the band edge (-15%, 46483) is about 15,500. So the strategy cannot close the gap on its own, and the note expects the smuggler to stay out of band. The result that matters is how much of that ceiling the inspection rate eats: each inspection with contraband aboard is detected with probability 0.25 at low heat for a smuggler, and a detection loses the cargo plus a fine near 700 silver. How often an inspection fires on a 17-day leg is one of the things the new columns measure.
- Against the opportunist (S3). The smuggler's S3 median (28304) is within 1% of its cautious median (28565); it claims no bounties (0/30), so the opportunist adds nothing for this type. S4 is the first strategy that uses a smuggler-only edge. A non-smuggler running S4 shows what the edge is worth: the smuggler's detection is 15 points lower, and its inspection weight is 1.3x.
- Reading for design review. If S4's smuggler median is at least about 1,000 above S1 and the seizure rate is low, the smuggler's edge is real but small, and the open question is whether the gap should be closed by price levers or accepted as a floor. If S4 is below S1, contraband does not pay at these route lengths and the inspection weight is not what holds the smuggler back.

## Later, separate PRs
1. **The bot (test only).** Implements S4 and the columns, regenerates the committed sweep. No game or content change, no pins.
2. **The inspection step.** `inspection_chance_mult` for the smuggler from 1.3 to 1.1, only after the S4 numbers are read. It needs an overlay allow-list extension in `tuned.rs`: today the loader allows only buy, sell and port-fee multipliers, provision burn, contract bias and the hunter's claim reward. The extension would allow `captains[id=smuggler].inspection.inspection_chance_mult`, lowering only (tuned below base, not below 1.0), smuggler only, with loader tests for each refusal. The pins are regenerated with the generator (the inspection weight changes which event a day picks, so smuggler scripts that sail can move), the `python-base` run stays the proof, and any moved frame goes to review. One lever per PR.
3. **A retune** only if the first two leave the smuggler's reading unexplained. Pins are regenerated with every retune, so retunes are batched.

## Open points for design review
- Should S4 join the best-of, or stay a side measurement? (The note proposes a side measurement.)
- Should the heat thresholds (20 and 15-19) be bot rules, or should the bot carry contraband whatever the heat to measure the raw risk? (The note proposes rules, because a bot that always gets caught measures nothing about the smuggler's edge.)
- The direct lane needs a galleon, which no start has. Whether the smuggler should be able to reach it earlier is a game question outside this note.
