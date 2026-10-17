# Contraband strategy for the balance bot (design note)

Status: proposal, docs only. Nothing here changes the game, the bot or any tuned value. Figures marked MEASURED come from sweeps of the current bot; figures marked ESTIMATE are hand arithmetic, to be replaced by the probe in step 0 below.

## Why
The smuggler reads -43.3% against the field (30975 vs a median of 54686, day 60, seeds 1001-1030). The three bot strategies are plain trading and never touch contraband, so the smuggler's own trade is not measured. This note describes a fourth strategy, `s4_shadow`, that measures it, and says when to stop.

## What the game does (read from the code)
- **Goods and ports.** Contraband is opium (base 85), black powder (65) and stolen cargo (40). It is stocked and sold only at ports with a black market: Corsair's Rest and Spice Narrows. Selling it anywhere else is refused. Opium is cheap at Spice Narrows and dear at Corsair's Rest; black powder is the reverse; stolen cargo is nearly flat, so it has no margin. Stock is tiny (targets 4-10, restock 0.3-1.0 a day), so a batch is 4-10 units.
- **The lane problem.** Lanes into Spice Narrows are brigantine class, and the direct Corsair's Rest to Spice Narrows run is galleon class. A ship two or more ranks below a lane is blocked. So a sloop (seven of nine starters, including the smuggler) can never sail to Spice Narrows, and a cutter can only go the long way over warning lanes. **With the ship it starts with, the smuggler has no contraband loop.** Any loop needs a ship purchase first (the brigantine, 800 silver, hold 80), which is itself a big change to a trading run.
- **Inspections are at sea only.** Each sea day can draw one. Its weight is the base chance times the captain's `inspection_chance_mult`, a regional heat factor (x1.2 at heat 15, x1.4 at 25, x1.8 at 40) and the wanted level. There is no port inspection, no way to dodge and no fine to choose. With contraband aboard, detection is 0.40 plus 0.10 per heat point above 15 (less 0.15 for the smuggler), clamped to 0.10-0.90; a detection seizes all contraband and fines 3 x 30 x units. The bot's only controls are what it carries, for how long, and the heat it carries into a leg.
- **Heat.** A contraband sale adds 3-8 regional heat; heat decays about 1 a day (2 at 20 or more). The smuggler starts at heat 8 and the corsair at 12.
- **The inspection row is not a lever on a trade-only bot (MEASURED).** Smuggler best-of: 30975 today, 30628 at chance 1.1, 32016 at 0.9, 30277 with no inspections at all. Everything is inside the plus or minus 2 point noise.

## Step 0: the probe (first, before any strategy code)
A test or example with no game change, under a minute. For each of the nine types print: the starting ship's class and which black-market ports it can reach over open lanes; the cheapest ship that makes a black-market pair reachable and its price against the type's starting silver; and the day-0 quote table for both ports at that type's multipliers (buy price, sell price, unit margin, and the profit of a full stock-limited batch with a 20% haircut).
- ESTIMATE: only the brigantine makes a pair reachable; a good round trip nets roughly 200-300 silver gross; a detected seizure costs about 1100.
- **Stop rule.** If the best realistic contraband profit is under 3% of the field median (about 1600) a run, stop. Record that the black-market trade is dead content in this economy, a design question rather than a tuning one, and build nothing beyond the probe. That is a legitimate outcome.

## Step 1 (only if the probe passes): `s4_shadow`
`s4_shadow` is the cautious trader plus additive contraband rules. It is type-blind (no type or ship ids in the bot), every type plays it, and a run with no contraband opportunity must play exactly like the cautious trader.
1. **Loop gate, data-derived.** From the ports and routes, find a pair of black-market ports reachable over open lanes for the current ship. If there is none, play the cautious trader for the turn (every sloop and cutter start).
2. **The one ship decision.** If no pair is reachable, it is day 25 or earlier, the bot is docked at a shipyard and has at least 1.5 times the price of the cheapest ship that makes a pair reachable, it buys that ship, hires the minimum crew and enters loop mode. It never detours to a shipyard. If this almost never fires, that is a finding, not a reason to add detours.
3. **Loop mode, per docked turn.** Sell held contraband at a black market when proceeds reach 110% of cost (or the lot is over 14 days old). Heat gate: with regional heat at 15 or more, buy no contraband and trade legally. Buy for the other port only while the discounted quote beats the next unit's price by 20 silver and by 25%, with caps on hold space, stock, 40% of silver, 12 units, and an exposure cap (potential fines plus the cost of contraband held at most 35% of net worth). Fill leftover hold with legal goods only if they sell on the way. Never start a leg that cannot arrive by day 59.
4. **Abort rule.** Fall back to the cautious trader for the rest of the run if silver plus cargo falls under 400, two completed loop legs make no profit, contraband is seized twice, or a leg cannot be planned.
5. **Determinism.** Integer math, sorted iteration, and read-only helpers, so a run that buys nothing consumes the random stream exactly like the cautious trader.

## Scoring and checks
- All nine types play S4 on the same 30 seeds (1080 runs). The best-of becomes the best of four strategies; the CSV and summary keep the old best-of-three next to the new best-of-four so the series stays comparable. S4 adds no new pass or fail rule; if it moves a type across the band edge or the field median by over 1%, the PR must say which columns caused it.
- **Identity test.** A run with no contraband event (no contraband bought and no ship bought) must equal the cautious trader's net worth on the same seed, to the silver. It runs on all nine types x 30 seeds (270 runs). So S4 can raise a score only through a counted contraband or ship event.
- **Bust handling.** A bust is net worth under 200 at the end. A contraband seizure is a loss, not a game over, and the abort rule is the only risk brake. A type that busts on more than 20% of S4 seeds marks S4 as broken for that type.
- **Runtime limits.** The 1080-run balance step must stay under the 5-minute CI cap (S4 adds under 30 seconds in release), with at most 12 loop legs per run and the existing loop guard of 2000.
- Other tests: determinism with two more seeds that enter loop mode; no refused contraband sale; no departure with contraband bought at heat 15 or more; the reachability table asserted for every starting ship class (so a content change that opens a lane is noticed); at most 20% of loop runs ending with unsold contraband; the other three strategies' columns byte-identical to the committed sweep; runtime under 30 seconds added.
- New CSV columns, appended at the end: contraband buys, units sold, profit (revenue minus cost of units sold), seized cost, inspections, fines paid, loop legs, shipyard bought, max heat; zero for the other strategies. The summary adds a per-type table of seeds in loop mode, legs, profit per run, seized and fines.

## How to read the result (artefact or imbalance)
1. Is S4 doing anything? Seeds in loop mode per type. Zero means S4 equals the cautious trader for that type and says nothing about it.
2. Attribute by column: the contraband share is profit minus fines minus seized cost; the rest is the ship (hold 30 to 80 changes everything). If the ship is more than half of the lift, the lift is an artefact of the upgrade and is not tuned on. For a clean read, run a diagnostic control with the same ship purchase and contraband buying disabled, on the loop seeds only; it is not committed.
3. Imbalance: the smuggler off-band under all strategies, a contraband share under 20% of its lift, and other types' share no larger than the smuggler's. That would say the black-market edge is not what separates the types.
4. Artefact flags: bust or game over on over 20% of seeds for a type, fines over 25% of contraband gross, over 20% of loop runs ending with unsold contraband, or a result that rests on one or two seeds (bootstrap noise is about plus or minus 8-10 points).
5. A move that helps the best-of but worsens the cautious and opportunist columns for that type is rejected.

## Against the smuggler's -43.3%
Even a good result is small: a brigantine round trip grosses roughly 200-300 silver (ESTIMATE), and the realistic ceiling with the inspection change is plus 0 to 3 points, so the smuggler stays out of band (about -40 to -43). That is accepted. If anyone wants more, the levers are black-market depth (stock target and restock on the two slots, content that affects every type) or starting silver, each a separate design decision.

## Later, separate PRs
1. **The probe, then the bot (test only).** No game or content change, no pins. Order: probe, stop rule, then S4.
2. **The inspection step, only after the S4 report is read and only if it shows a contraband share where risk is the limiter.** `inspection_chance_mult` for the smuggler from 1.3 to 1.1 (fallback 1.0). The loader floor is 0.9: the row accepts tuned values from 0.9 up to the base 1.3 and refuses anything lower, because 0.8 moves a captain golden. The intended value stays above 1.0 so the smuggler remains more inspected than the navigator and dockhand. It needs an overlay allow-list row in `tuned.rs` for `captains[id=smuggler].inspection.inspection_chance_mult`: base must be 1.3, lowering only, smuggler only, with loader tests for each refusal, compiled out on `python-base`. At 1.1 no golden is expected to change; at 1.0 one script (`smuggler.txt`) does. Pins are regenerated with the generator, and any moved frame goes to review. Never in the same PR as S4.
3. **A retune** only if the first two leave the reading unexplained.

## Open points
- If the probe says stop, is the black market meant to be dead content, or should a design change open the loop (a lane, a starting ship, market depth)?
- Should S4 join the best-of or stay a side measurement? (This note follows the best-of-four with both columns kept.)
