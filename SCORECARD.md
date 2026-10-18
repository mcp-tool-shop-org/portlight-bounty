# Scorecard

**Repo:** portlight-bounty
**Date:** 2026-10-18
**Type tags:** `[all]` local game, Rust workspace, Godot chart. Not npm.

## Pre-remediation (pre-0.1.0)

| Category | Score | Notes |
|----------|-------|-------|
| A. Security | 2/10 | No SECURITY.md. README had no threat paragraph. |
| B. Error Handling | 4/10 | Sentences and exit codes exist. They are not the shop envelope. Parity locks the sentences. |
| C. Operator Docs | 5/10 | README was a long operator dump and still said 70 PNGs, v0.3.0. No CHANGELOG. MIT file had no art carve-out. `--help` listed a subset of the script commands. |
| D. Shipping Hygiene | 6/10 | CI, Cargo.lock, and tests were already real. No verify script, no dependency audit job, no tag. |
| E. Identity | 1/10 | No logo in the README, no landing page, no homepage. |
| **Overall** | **18/50** | |

## Key gaps

1. The public README described an older art bundle and read as a porting log.
2. Before remediation, the MIT file covered the pictures by silence and did not record the `ai-rpg-stage` provenance for the five vendored props (now documented in `LICENSE`).
3. No release, no landing page, no security policy.

## This release (0.3.0)

Hard gates A–D and soft gates E (including 8-language polyglot-mcp README translations) are checked or justified in SHIP_GATE.md. The landing page, handbook, canon (Phases 1–6), and workspace version are aligned at 0.3.0.
