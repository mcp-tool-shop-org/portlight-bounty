# Ship Gate

> No repo is "done" until every applicable line is checked.
> Copy this into your repo root. Check items off per-release.

**Tags:** `[all]` every repo · `[npm]` `[pypi]` `[vsix]` `[desktop]` `[container]` published artifacts · `[mcp]` MCP servers · `[cli]` CLI tools

**Repo:** portlight-bounty 0.1.0. A local game. Not an npm package, not an MCP server.

---

## A. Security Baseline

- [x] `[all]` SECURITY.md exists (report path, supported versions, response timeline) (2026-10-09)
- [x] `[all]` README includes threat model paragraph (data touched, data NOT touched, permissions required) (2026-10-09)
- [x] `[all]` No secrets, tokens, or credentials in source or diagnostics output (2026-10-09)
- [x] `[all]` No telemetry by default — state it explicitly even if obvious (2026-10-09)

### Default safety posture

- [ ] `[cli|mcp|desktop]` SKIP: local game. The CLI does not kill, delete, or restart other processes.
- [ ] `[cli|mcp|desktop]` SKIP: save and load take a directory the operator names. This is not a sandboxed agent.
- [ ] `[mcp]` SKIP: not an MCP server.
- [ ] `[mcp]` SKIP: not an MCP server.

## B. Error Handling

- [ ] `[all]` SKIP: simulation failures stay the Python-parity sentences. A code/hint envelope would change the golden text. Usage mistakes print one usage line and exit 2.
- [ ] `[cli]` SKIP: exit codes are 0 ok, 1 simulation or file failure, 2 usage. Swapping 1 and 2 to the shop standard would break existing runners. `--help` states the codes this release uses.
- [ ] `[cli]` SKIP: the CLI prints sentences and has no `--debug` flag. A panic would still print a Rust stack. That is a bug, not a log mode.
- [ ] `[mcp]` SKIP: not an MCP server.
- [ ] `[mcp]` SKIP: not an MCP server.
- [ ] `[desktop]` SKIP: Godot shows the game's own lines. There is no separate desktop shell.
- [ ] `[vscode]` SKIP: not a VS Code extension.

## C. Operator Docs

- [x] `[all]` README is current: what it does, install, usage, supported platforms + runtime versions (2026-10-09)
- [x] `[all]` CHANGELOG.md (Keep a Changelog format) (2026-10-09)
- [x] `[all]` LICENSE file present and repo states support status (2026-10-09)
- [x] `[cli]` `--help` output accurate for all commands and flags (2026-10-09)
- [ ] `[cli|mcp|desktop]` SKIP: no silent/normal/verbose/debug pipeline. The CLI prints the play log and error sentences. It does not handle credentials.
- [ ] `[mcp]` SKIP: not an MCP server.
- [ ] `[complex]` SKIP: not a daemon. Player docs are the Starlight handbook.

## D. Shipping Hygiene

- [x] `[all]` `verify` script exists (test + build + smoke in one command) (2026-10-09)
- [x] `[all]` Version in manifest matches git tag (Cargo.toml 0.1.0, release tag v0.1.0) (2026-10-09)
- [x] `[all]` Dependency scanning runs in CI (ecosystem-appropriate) (2026-10-09)
- [ ] `[all]` SKIP: Dependabot stays off unless the Director asks. No update bot in this release.
- [ ] `[npm]` SKIP: not published to npm. `publish = false` in Cargo.toml.
- [ ] `[npm]` SKIP: not an npm or PyPI package.
- [x] `[npm]` Lockfile committed (Cargo.lock). Not an npm or PyPI package (2026-10-09)
- [ ] `[vsix]` SKIP: not a VS Code extension.
- [ ] `[desktop]` SKIP: no installer. Players build with cargo and open godot/. Linux CI builds the extension. macOS has not been run.

## E. Identity (soft gate — does not block ship)

- [x] `[all]` Logo in README header (2026-10-09)
- [ ] `[all]` SKIP: no translations this pass. The Director held them because the GPU is booked. English is the source.
- [x] `[org]` Landing page (@mcptoolshop/site-theme) (2026-10-09)
- [x] `[all]` GitHub repo metadata: description, homepage, topics (2026-10-09)

---

## Gate Rules

**Hard gate (A–D):** Must pass before any version is tagged or published.
If a section doesn't apply, mark `SKIP:` with justification — don't leave it unchecked.

**Soft gate (E):** Should be done. Product ships without it, but isn't "whole."

**Checking off:**
```
- [x] `[all]` SECURITY.md exists (2026-02-27)
```

**Skipping:**
```
- [ ] `[pypi]` SKIP: not a Python project
```
