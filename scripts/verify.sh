#!/usr/bin/env bash
# Sim tests, clippy, CLI build, and a help smoke. The Godot frame compare stays
# on the Linux CI job. This script does not run it.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo fmt --all -- --check
cargo clippy --locked --workspace --exclude portlight-godot --all-targets --all-features -- -D warnings
cargo test --locked --workspace --exclude portlight-godot
cargo test --locked --workspace --exclude portlight-godot --features python-base
cargo build --locked -p portlight-cli
cargo run --locked -q -p portlight-cli -- --help >/dev/null
echo "verify: fmt, clippy, sim tests (tuned + python-base), cli build, and --help passed"
echo "verify: Godot frame compare was not run here. CI's godot job owns it."

