---
title: Commands
description: The portlight binary, its flags, and the script verbs.
sidebar:
  order: 5
---

`portlight --help` is the list the binary prints. This page matches version 0.1.0.

```text
portlight new [--captain TYPE] [--name NAME] [--seed N] [--json]
portlight script <file> [--save-dir DIR --slot NAME]
portlight script -
portlight load <dir> <slot>
portlight help
```

`new` defaults to captain merchant, name Captain, seed 1. `--json` prints the canonical snapshot. Without it, `new` prints a short summary and the market at the dock.

`script -` reads the script from stdin. `--save-dir` writes a version-12 slot. `--slot` names that slot and is used together with `--save-dir`. The default slot name is `default`.

`load` prints one version-12 save.

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | The run worked |
| 1 | The simulation or a file failed |
| 2 | The command was not understood |

## Script verbs

One command per line. `portlight --help` prints the same set.

`new`, `custom`, `buy`, `sell`, `depart`, `advance`, `arrival_narrative`, `evaluate_consequences`, `accept_contract`, `complete_contract`, `abandon_contract`, `buy_infrastructure`, `take_credit`, `buy_insurance`, `deposit`, `withdraw`, `repay_credit`, `bank_deposit`, `bank_withdraw`, `bank_letter`, `save`, `load`, `hire`, `provision`, `work`, `duel`, `resolve_duel`, `encounter`, `naval`, `board`, `fight`, `capture`, `spare`, `take_all`, `train`, `recruit`, `skill`, `remember`, `agency`, `gear`, `buy_ship`, `upgrade`, `form_convoy`, `repair_fleet`, `repair`, `rename_ship`, `dock_current_ship`, `board_fleet_ship`, `sell_fleet_ship`, `fire`, `transfer`, `maintain`, `hunt`, `bounty`, `wanted`.

`bounty` takes `accept`, `hunt`, or `claim`, then a captain id.

`deposit` and `withdraw` move cargo. The `bank_` verbs move silver at a counting house: `bank_deposit N` (at least 50, free), `bank_withdraw N` (this port's account, fee 1%, at least 1), `bank_letter PORT N` (another port's account, fee 5%, at least 5). The fee comes out of N.

The Godot chart does not expose every verb. The script runner does, because the parity harness does.
