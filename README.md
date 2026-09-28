# portlight-bounty

Rust port of [Portlight](https://github.com/mcp-tool-shop-org/portlight), a trade-first maritime strategy game. This repo is stage 1: the rules and simulation, checked against the Python game at commit `9b02494`.

The simulation crate does not depend on a UI. A dimetric map can sit on it later. The map of what is ported, and what is not, is [docs/PORTING-PLAN.md](docs/PORTING-PLAN.md).

## Layout

- `crates/portlight-sim` — goods, ports, prices, trade, voyages, victory paths, and `Session` (the turn-by-turn API)
- `crates/portlight-cli` — `portlight` binary
- `parity/` — action scripts and golden snapshots from the Python engine
- `tools/parity/` — oracle and diff harness

## Run

```
cargo run -p portlight-cli -- new --captain merchant --name Ada --seed 42
cargo run -p portlight-cli -- script parity/scripts/voyage.txt
cargo test --locked --workspace
```

Script commands: `new`, `buy`, `sell`, `depart`, `advance`, `hire`, `provision`, `work`, `duel`, `resolve_duel`.

## Parity

`cargo test` compares those scripts to `parity/golden/`. To regenerate the goldens from the Python checkout:

```
PYTHONPATH=/path/to/portlight/src python3 tools/extract_content.py
PYTHONPATH=/path/to/portlight/src python3 tools/parity/check.py --write-golden
```

The RNG is CPython's MT19937, so the same seed and script produce the same state for the systems that are ported. Full `GameSession` parity is still open; the plan lists the gaps.
