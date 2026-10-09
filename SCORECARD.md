# Scorecard

**Repo:** portlight-bounty
**Date:** 2026-10-09
**Type tags:** `[all]` local game, Rust workspace, Godot chart. Not npm.

## Pre-remediation

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
2. The MIT file covered the pictures by silence, and the five vendored props have no recorded maker or license.
3. No release, no landing page, no security policy.

## This release

Hard gates A–D are checked or skipped in SHIP_GATE.md. Translations are skipped on purpose. The landing page and the GitHub homepage land in the same release.
