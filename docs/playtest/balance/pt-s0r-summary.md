# Type balance sweep (day 60)

Seeds 1001..1030 (30 seeds), captain "Balance", 9 types x 3 strategies. Score = median over seeds of the best of the 3 strategies. Gate: within +/-15% of the field median.

Field median: **54686**

| Type | Score | vs field | Gate | Winning strategy | Spread |
|---|---:|---:|---|---|---:|
| merchant | 38525 | -29.5% | FLAG | s3_opportunist | 25% |
| smuggler | 30975 | -43.3% | FLAG | s3_opportunist | 31% |
| navigator | 52078 | -4.7% | PASS | s1_cautious | 49% |
| privateer | 61163 | +11.8% | PASS | s3_opportunist | 74% |
| corsair | 62896 | +15.0% | FLAG | s1_cautious | 66% |
| scholar | 63643 | +16.3% | FLAG | s1_cautious | 52% |
| merchant_prince | 54686 | +0.0% | PASS | s1_cautious | 48% |
| dockhand | 58328 | +6.6% | PASS | s1_cautious | 69% |
| bounty_hunter | 22312 | -59.1% | FLAG | s2_contracts | 13% |

## Per-strategy medians

| Type | s1_cautious | s2_contracts | s3_opportunist |
|---|---:|---:|---:|
| merchant | 32762 | 20838 | 34806 |
| smuggler | 28565 | 11225 | 28304 |
| navigator | 48693 | 21410 | 46980 |
| privateer | 51714 | 14684 | 55155 |
| corsair | 58910 | 22664 | 58954 |
| scholar | 48383 | 19829 | 38402 |
| merchant_prince | 50868 | 24103 | 45803 |
| dockhand | 50625 | 12635 | 43036 |
| bounty_hunter | 16068 | 20684 | 13495 |

## Completion rates

- s2_contracts: a contract completed on 265/270 runs (gate: at least 98%).
- s3_opportunist: a bounty claimed on 58/270 runs (reported, not gated). The opportunist hunts only if armed or holding at least 1000 silver.

## Bounty edge (s3_opportunist)

| Type | Claim seeds | Claims | Bounty silver | Silver per run | vs privateer |
|---|---:|---:|---:|---:|---:|
| merchant | 0/30 | 0 | 0 | 0 | 0.00x |
| smuggler | 0/30 | 0 | 0 | 0 | 0.00x |
| navigator | 0/30 | 0 | 0 | 0 | 0.00x |
| privateer | 30/30 | 115 | 18410 | 613 | 1.00x |
| corsair | 0/30 | 0 | 0 | 0 | 0.00x |
| scholar | 0/30 | 0 | 0 | 0 | 0.00x |
| merchant_prince | 0/30 | 0 | 0 | 0 | 0.00x |
| dockhand | 0/30 | 0 | 0 | 0 | 0.00x |
| bounty_hunter | 28/30 | 110 | 21168 | 705 | 1.15x |

## Game over by strategy

| Strategy | Game over | Busts (net worth < 200) | Runs | Broken bot (>20%) |
|---|---:|---:|---:|---|
| s1_cautious | 0 | 0 | 270 | no |
| s2_contracts | 0 | 0 | 270 | no |
| s3_opportunist | 0 | 3 | 270 | no |
