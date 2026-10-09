# Contributing

Portlight Bounty is a public game. Issues and pull requests are welcome.

## Before you open a pull request

- The simulation stays UI-free. A screen change does not edit `crates/portlight-sim` unless the ticket says the rules changed.
- A rules change needs a parity script and a golden. Do not "fix" a number the Python game still produces. The locked quirks are listed in the handbook.
- Format with `cargo fmt --all`. Clippy on the sim workspace is `-D warnings`.
- Do not add a personal mailbox, a home path, a Tailscale address, or a token path to a file that will be committed.
- Commit as a GitHub noreply if you can. Do not put a legal name in the author line.

## What we will not take in this release

- A claim that the pictures are MIT, or free to reuse.
- A generated plate with a baked shadow and no transparency, dropped in as a finished prop.
- A translation pass. English is the source until a later release says otherwise.

## Checks

`scripts/verify.ps1` (or `scripts/verify.sh`) runs the sim tests, builds the CLI, and checks `--help`. The Godot frame compare stays on the Linux CI job. Say so if you could not run it.
