---
title: The simulation
description: The crates, the parity witness, and the numbers you do not fix.
sidebar:
  order: 3
---

`portlight-sim` has no UI. `portlight-cli` runs a script and prints the snapshot JSON. `portlight-chart` is the dimetric math and does not link Godot. `portlight-godot` is the view.

A script plus a seed reproduces the same game. The random source is CPython's MT19937. Prices use Python 3 rounding. Do not swap in Rust's `round`.

## Check it

```bash
cargo test --locked --workspace --exclude portlight-godot
cargo run -p portlight-cli -- --help
```

`scripts/verify.ps1` and `scripts/verify.sh` run the formatter, those tests, a CLI build, and `--help`. They do not run the Godot frame compare. That job is Linux CI, because it needs a display.

The live Python oracle in CI checks out `mcp-tool-shop-org/portlight` at `9b02494`. Goldens under `parity/golden/` are the recorded witness when the oracle is not in the job.

## Numbers that stay

Python and the goldens agree. A later screen calls the simulation. It does not correct these.

- Sail-picker days use raw ship speed. Crew, morale, season, and the captain stay off that estimate.
- The picker lists lanes that depart may then block. A rank gap of 1 is a warning. A gap of 2 or more is blocked.
- The victory path id `commercial_empire` is not the milestone family `commercial_finance`.
- Advancing the day passes day 0 into the market tick on that path. Seasonal stock drains stay off. Python does the same.
- A duel's standing change is shown and is not written onto reputation.
- The navigator's storm resistance and the quartermaster's sell bonus exist and are never called. Leave them uncalled.

## Errors

A failed action is a sentence, the same sentence the Python game uses where the ports match. The CLI does not wrap that sentence in a code and a hint. Doing so would change the golden text.

Exit codes for the binary: 0 when the run worked, 1 when the simulation or a file failed, 2 when the command was not understood. There is no `--debug` flag and no log level. The program does not print a credential, because it never reads one.

## Where a change belongs

A screen change stays in `portlight-godot` and `godot/`. A rules change belongs in `portlight-sim`, with a parity script and a golden, and it does not reformat the shared files it only had to touch. See [CONTRIBUTING.md](https://github.com/mcp-tool-shop-org/portlight-bounty/blob/main/CONTRIBUTING.md).
