# Contraband probe (day 0, seed 1001)

Black markets: corsairs_rest and spice_narrows. Cheapest ship that makes the pair reachable over open lanes: trade_brigantine (800 silver, hold 80, speed 6, crew min 8).

## Reachability and the ship

| Type | Start ship | Silver | Loop on open lanes | Loop with warning lanes | Open distance | Can afford the enabling ship at 1.5x |
|---|---|---:|---|---|---:|---|
| bounty_hunter | swift_cutter | 425 | no | yes | none | no |
| corsair | coastal_sloop | 500 | no | no | none | no |
| dockhand | coastal_sloop | 300 | no | no | none | no |
| merchant | coastal_sloop | 550 | no | no | none | no |
| merchant_prince | coastal_sloop | 700 | no | no | none | no |
| navigator | coastal_sloop | 450 | no | no | none | no |
| privateer | swift_cutter | 475 | no | yes | none | no |
| scholar | coastal_sloop | 400 | no | no | none | no |
| smuggler | coastal_sloop | 500 | no | no | none | no |

## Day-0 quotes and stock-limited batches

Batch = units bought one at a time (each lifts the next price) while the quote beats the next price by 20 and by 25%, capped by stock minus 1, 12 units and the enabling ship's hold. Quote = 80% of the far market's sell price. Profit = units x quote - cost.

| Type | From | To | Good | Buy first | Sell there | Quote | Stock | Units | Cost | Profit |
|---|---|---|---|---:|---:|---:|---:|---:|---:|---:|
| bounty_hunter | corsairs_rest | spice_narrows | black_powder | 40 | 134 | 107 | 8 | 5 | 273 | 262 |
| bounty_hunter | corsairs_rest | spice_narrows | opium | 174 | 45 | 36 | 3 | 0 | 0 | 0 |
| bounty_hunter | corsairs_rest | spice_narrows | stolen_cargo | 30 | 28 | 22 | 12 | 0 | 0 | 0 |
| bounty_hunter | spice_narrows | corsairs_rest | black_powder | 155 | 35 | 28 | 3 | 0 | 0 | 0 |
| bounty_hunter | spice_narrows | corsairs_rest | opium | 52 | 150 | 120 | 10 | 5 | 323 | 277 |
| bounty_hunter | spice_narrows | corsairs_rest | stolen_cargo | 32 | 26 | 20 | 8 | 0 | 0 | 0 |
| corsair | corsairs_rest | spice_narrows | black_powder | 40 | 134 | 107 | 8 | 5 | 297 | 238 |
| corsair | corsairs_rest | spice_narrows | opium | 174 | 45 | 36 | 3 | 0 | 0 | 0 |
| corsair | corsairs_rest | spice_narrows | stolen_cargo | 30 | 28 | 22 | 12 | 0 | 0 | 0 |
| corsair | spice_narrows | corsairs_rest | black_powder | 155 | 35 | 28 | 3 | 0 | 0 | 0 |
| corsair | spice_narrows | corsairs_rest | opium | 52 | 150 | 120 | 10 | 5 | 351 | 249 |
| corsair | spice_narrows | corsairs_rest | stolen_cargo | 32 | 26 | 20 | 8 | 0 | 0 | 0 |
| dockhand | corsairs_rest | spice_narrows | black_powder | 40 | 134 | 107 | 8 | 4 | 216 | 212 |
| dockhand | corsairs_rest | spice_narrows | opium | 174 | 45 | 36 | 3 | 0 | 0 | 0 |
| dockhand | corsairs_rest | spice_narrows | stolen_cargo | 30 | 28 | 22 | 12 | 0 | 0 | 0 |
| dockhand | spice_narrows | corsairs_rest | black_powder | 155 | 35 | 28 | 3 | 0 | 0 | 0 |
| dockhand | spice_narrows | corsairs_rest | opium | 52 | 150 | 120 | 10 | 4 | 264 | 216 |
| dockhand | spice_narrows | corsairs_rest | stolen_cargo | 32 | 26 | 20 | 8 | 0 | 0 | 0 |
| merchant | corsairs_rest | spice_narrows | black_powder | 40 | 134 | 107 | 8 | 5 | 264 | 271 |
| merchant | corsairs_rest | spice_narrows | opium | 174 | 45 | 36 | 3 | 0 | 0 | 0 |
| merchant | corsairs_rest | spice_narrows | stolen_cargo | 30 | 28 | 22 | 12 | 0 | 0 | 0 |
| merchant | spice_narrows | corsairs_rest | black_powder | 155 | 35 | 28 | 3 | 0 | 0 | 0 |
| merchant | spice_narrows | corsairs_rest | opium | 52 | 150 | 120 | 10 | 6 | 410 | 310 |
| merchant | spice_narrows | corsairs_rest | stolen_cargo | 32 | 26 | 20 | 8 | 0 | 0 | 0 |
| merchant_prince | corsairs_rest | spice_narrows | black_powder | 40 | 134 | 107 | 8 | 5 | 260 | 275 |
| merchant_prince | corsairs_rest | spice_narrows | opium | 174 | 45 | 36 | 3 | 0 | 0 | 0 |
| merchant_prince | corsairs_rest | spice_narrows | stolen_cargo | 30 | 28 | 22 | 12 | 0 | 0 | 0 |
| merchant_prince | spice_narrows | corsairs_rest | black_powder | 155 | 35 | 28 | 3 | 0 | 0 | 0 |
| merchant_prince | spice_narrows | corsairs_rest | opium | 52 | 150 | 120 | 10 | 6 | 402 | 318 |
| merchant_prince | spice_narrows | corsairs_rest | stolen_cargo | 32 | 26 | 20 | 8 | 0 | 0 | 0 |
| navigator | corsairs_rest | spice_narrows | black_powder | 40 | 134 | 107 | 8 | 5 | 297 | 238 |
| navigator | corsairs_rest | spice_narrows | opium | 174 | 45 | 36 | 3 | 0 | 0 | 0 |
| navigator | corsairs_rest | spice_narrows | stolen_cargo | 30 | 28 | 22 | 12 | 0 | 0 | 0 |
| navigator | spice_narrows | corsairs_rest | black_powder | 155 | 35 | 28 | 3 | 0 | 0 | 0 |
| navigator | spice_narrows | corsairs_rest | opium | 52 | 150 | 120 | 10 | 5 | 351 | 249 |
| navigator | spice_narrows | corsairs_rest | stolen_cargo | 32 | 26 | 20 | 8 | 0 | 0 | 0 |
| privateer | corsairs_rest | spice_narrows | black_powder | 40 | 134 | 107 | 8 | 4 | 221 | 207 |
| privateer | corsairs_rest | spice_narrows | opium | 174 | 45 | 36 | 3 | 0 | 0 | 0 |
| privateer | corsairs_rest | spice_narrows | stolen_cargo | 30 | 28 | 22 | 12 | 0 | 0 | 0 |
| privateer | spice_narrows | corsairs_rest | black_powder | 155 | 35 | 28 | 3 | 0 | 0 | 0 |
| privateer | spice_narrows | corsairs_rest | opium | 52 | 150 | 120 | 10 | 5 | 366 | 234 |
| privateer | spice_narrows | corsairs_rest | stolen_cargo | 32 | 26 | 20 | 8 | 0 | 0 | 0 |
| scholar | corsairs_rest | spice_narrows | black_powder | 40 | 134 | 107 | 8 | 5 | 285 | 250 |
| scholar | corsairs_rest | spice_narrows | opium | 174 | 45 | 36 | 3 | 0 | 0 | 0 |
| scholar | corsairs_rest | spice_narrows | stolen_cargo | 30 | 28 | 22 | 12 | 0 | 0 | 0 |
| scholar | spice_narrows | corsairs_rest | black_powder | 155 | 35 | 28 | 3 | 0 | 0 | 0 |
| scholar | spice_narrows | corsairs_rest | opium | 52 | 150 | 120 | 10 | 5 | 337 | 263 |
| scholar | spice_narrows | corsairs_rest | stolen_cargo | 32 | 26 | 20 | 8 | 0 | 0 | 0 |
| smuggler | corsairs_rest | spice_narrows | black_powder | 40 | 134 | 107 | 8 | 5 | 249 | 286 |
| smuggler | corsairs_rest | spice_narrows | opium | 174 | 45 | 36 | 3 | 0 | 0 | 0 |
| smuggler | corsairs_rest | spice_narrows | stolen_cargo | 30 | 28 | 22 | 12 | 0 | 0 | 0 |
| smuggler | spice_narrows | corsairs_rest | black_powder | 155 | 35 | 28 | 3 | 0 | 0 | 0 |
| smuggler | spice_narrows | corsairs_rest | opium | 52 | 150 | 120 | 10 | 6 | 382 | 338 |
| smuggler | spice_narrows | corsairs_rest | stolen_cargo | 32 | 26 | 20 | 8 | 0 | 0 | 0 |

## Best realistic profit and the stop rule

One round trip is the best batch each way at day-0 stock. The two-round-trip figure is a deliberately generous ceiling, not a forecast: it assumes the stock fully restocks between visits, but restock at the key slots is only 0.3 to 0.5 a day. Travel time rules it out anyway. The two markets are 184 leagues apart over brigantine lanes, which is about 31 days one way at the brigantine's speed of 6, so a round trip is about 62 days, longer than the 60-day run, and the ship must first be bought and crewed. The cutter starters would take about 20 days one way (speed 9) over warning lanes. Seizure cost is the batch cost plus 90 per unit.

Pricing note: these quotes are read at day 0 from a new session, which prices the world without the captain's price multipliers on the first buy and on the far sell price. With the multipliers applied, the smuggler's round trip is about 693 (1386 for two) and the highest of any type is about 696 (1392 for two), so every type is still under 1600.

| Type | Best leg | Round trip | Two round trips (generous ceiling) | Seizure cost of best leg | Stop (under 1600) |
|---|---|---:|---:|---:|---|
| bounty_hunter | opium spice_narrows->corsairs_rest 277 | 539 | 1078 | 773 | STOP |
| corsair | opium spice_narrows->corsairs_rest 249 | 487 | 974 | 801 | STOP |
| dockhand | opium spice_narrows->corsairs_rest 216 | 428 | 856 | 624 | STOP |
| merchant | opium spice_narrows->corsairs_rest 310 | 581 | 1162 | 950 | STOP |
| merchant_prince | opium spice_narrows->corsairs_rest 318 | 593 | 1186 | 942 | STOP |
| navigator | opium spice_narrows->corsairs_rest 249 | 487 | 974 | 801 | STOP |
| privateer | opium spice_narrows->corsairs_rest 234 | 441 | 882 | 816 | STOP |
| scholar | opium spice_narrows->corsairs_rest 263 | 513 | 1026 | 787 | STOP |
| smuggler | opium spice_narrows->corsairs_rest 338 | 624 | 1248 | 922 | STOP |

Verdict: STOP. Even the generous two-round-trip ceiling is under 1600 silver a run for every type, so a contraband strategy cannot move any type by a measurable amount.

Reading the reachability: no starter can loop on open lanes. The two cutter starters (bounty_hunter and privateer) can reach both markets over warning lanes, but none of them can afford the 1200 silver (1.5x the 800 silver brigantine) that the bot rule needs; the best starting purse is 700.

## Decision

- The black market is dead content for the balance bot: both markets sit behind brigantine or galleon lanes, no starter can afford the enabling ship, and the run is too short to use it.
- Contraband leaves the balance model. The contraband strategy (`s4_shadow`) and the smuggler inspection change (1.3 to 1.1) are cancelled, not deferred.
- The economy is not reopened: no lane, stock, price or ship change is proposed because of this probe, since that would cascade through parity and the goldens.
- The smuggler stays improved but out of band. Any later non-price lever is a separate design note, not part of this change.
