# Type balance sweep (day 60)

Seeds 1001..1030 (30 seeds), captain "Balance", 9 types x 3 strategies. Score = median over seeds of the best of the 3 strategies. Gate: within +/-15% of the field median.

Field median: **51100**

| Type | Score | vs field | Gate | Winning strategy | Spread |
|---|---:|---:|---|---|---:|
| merchant | 44632 | -12.6% | PASS | s3_opportunist | 48% |
| smuggler | 25728 | -49.6% | FLAG | s3_opportunist | 30% |
| navigator | 51100 | +0.0% | PASS | s1_cautious | 62% |
| privateer | 67784 | +32.6% | FLAG | s1_cautious | 113% |
| corsair | 64645 | +26.5% | FLAG | s3_opportunist | 79% |
| scholar | 27751 | -45.6% | FLAG | s1_cautious | 26% |
| merchant_prince | 63449 | +24.1% | FLAG | s3_opportunist | 73% |
| dockhand | 77841 | +52.3% | FLAG | s3_opportunist | 117% |
| bounty_hunter | 18230 | -64.3% | FLAG | s1_cautious | 2% |

## Per-strategy medians

| Type | s1_cautious | s2_contracts | s3_opportunist |
|---|---:|---:|---:|
| merchant | 32762 | 12609 | 37349 |
| smuggler | 23165 | 7816 | 23587 |
| navigator | 48693 | 16542 | 42675 |
| privateer | 62908 | 4898 | 55536 |
| corsair | 58910 | 18228 | 57240 |
| scholar | 26329 | 12757 | 23811 |
| merchant_prince | 58006 | 20815 | 58347 |
| dockhand | 63877 | 11107 | 71152 |
| bounty_hunter | 14879 | 14551 | 13613 |

## Game over by strategy

| Strategy | Game over | Runs | Broken bot (>20%) |
|---|---:|---:|---|
| s1_cautious | 0 | 270 | no |
| s2_contracts | 0 | 270 | no |
| s3_opportunist | 0 | 270 | no |
