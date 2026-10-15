# Type balance sweep (day 60)

Seeds 1001..1030 (30 seeds), captain "Balance", 9 types x 3 strategies. Score = median over seeds of the best of the 3 strategies. Gate: within +/-15% of the field median.

Field median: **52078**

| Type | Score | vs field | Gate | Winning strategy | Spread |
|---|---:|---:|---|---|---:|
| merchant | 38525 | -26.0% | FLAG | s3_opportunist | 26% |
| smuggler | 24918 | -52.1% | FLAG | s1_cautious | 24% |
| navigator | 52078 | +0.0% | PASS | s1_cautious | 52% |
| privateer | 72156 | +38.5% | FLAG | s3_opportunist | 113% |
| corsair | 62896 | +20.7% | FLAG | s1_cautious | 69% |
| scholar | 28019 | -46.1% | FLAG | s3_opportunist | 13% |
| merchant_prince | 59979 | +15.1% | FLAG | s1_cautious | 65% |
| dockhand | 77020 | +47.8% | FLAG | s3_opportunist | 112% |
| bounty_hunter | 18925 | -63.6% | FLAG | s2_contracts | 4% |

## Per-strategy medians

| Type | s1_cautious | s2_contracts | s3_opportunist |
|---|---:|---:|---:|
| merchant | 32762 | 20838 | 34806 |
| smuggler | 23165 | 10592 | 21541 |
| navigator | 48693 | 21410 | 46980 |
| privateer | 62908 | 11416 | 70319 |
| corsair | 58910 | 22664 | 58954 |
| scholar | 26329 | 19097 | 25606 |
| merchant_prince | 58006 | 23967 | 54600 |
| dockhand | 63877 | 12687 | 71124 |
| bounty_hunter | 14879 | 15053 | 12484 |

## Completion rates

- s2_contracts: a contract completed on 266/270 runs (gate: at least 98%).
- s3_opportunist: a bounty claimed on 58/270 runs (reported, not gated). The opportunist hunts only if armed or holding at least 1000 silver.

## Game over by strategy

| Strategy | Game over | Busts (net worth < 200) | Runs | Broken bot (>20%) |
|---|---:|---:|---:|---|
| s1_cautious | 0 | 0 | 270 | no |
| s2_contracts | 0 | 0 | 270 | no |
| s3_opportunist | 0 | 3 | 270 | no |
