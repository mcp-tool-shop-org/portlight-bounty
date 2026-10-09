# Sim tests, CLI build, and a help smoke. The Godot frame compare stays
# on the Linux CI job. This script does not run it.
$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '..')
cargo fmt --all -- --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo test --locked --workspace --exclude portlight-godot
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo build --locked -p portlight-cli
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo run --locked -q -p portlight-cli -- --help | Out-Null
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
Write-Host 'verify: sim tests, cli build, and --help passed'
Write-Host 'verify: Godot frame compare was not run here. CI''s godot job owns it.'
