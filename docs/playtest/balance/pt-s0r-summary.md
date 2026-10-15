# Type balance sweep (day 60)

Seeds 1001..1030 (30 seeds), captain "Balance", 9 types x 3 strategies. Score = median over seeds of the best of the 3 strategies. Gate: within +/-15% of the field median.

Field median: **58493**

| Type | Score | vs field | Gate | Winning strategy | Spread |
|---|---:|---:|---|---|---:|
| merchant | 46391 | -20.6% | FLAG | s3_opportunist | 31% |
| smuggler | 25502 | -56.4% | FLAG | s1_cautious | 21% |
| navigator | 59543 | +1.7% | PASS | s3_opportunist | 58% |
| privateer | 72156 | +23.3% | FLAG | s3_opportunist | 100% |
| corsair | 62228 | +6.3% | PASS | s1_cautious | 61% |
| scholar | 27775 | -52.5% | FLAG | s1_cautious | 14% |
| merchant_prince | 58493 | +0.0% | PASS | s1_cautious | 58% |
| dockhand | 63877 | +9.2% | PASS | s1_cautious | 108% |
| bounty_hunter | 18925 | -67.6% | FLAG | s2_contracts | 4% |

## Per-strategy medians

| Type | s1_cautious | s2_contracts | s3_opportunist |
|---|---:|---:|---:|
| merchant | 32762 | 20838 | 39048 |
| smuggler | 23165 | 10592 | 23355 |
| navigator | 48693 | 21410 | 55883 |
| privateer | 62908 | 11416 | 70319 |
| corsair | 58910 | 22664 | 57830 |
| scholar | 26329 | 19097 | 18048 |
| merchant_prince | 58006 | 23967 | 40004 |
| dockhand | 63877 | 12687 | 135 |
| bounty_hunter | 14879 | 15053 | 12484 |

## Completion rates

- s2_contracts: a contract completed on 266/270 runs (gate: at least 98%).
- s3_opportunist: a bounty claimed on 58/270 runs (reported, not gated).

## Game over by strategy

| Strategy | Game over | Runs | Broken bot (>20%) |
|---|---:|---:|---|
| s1_cautious | 0 | 270 | no |
| s2_contracts | 0 | 270 | no |
| s3_opportunist | 0 | 270 | no |
