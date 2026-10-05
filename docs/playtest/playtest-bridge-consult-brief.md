# Consult brief — harden the Portlight playtest bridge

A consult, not a build. Read the measurements and the code paths below and answer the questions at the end. Do not re-derive the protocol, do not score the product gates, and do not start a seat. Nothing here quotes a person. The decisions are the ones written in this packet.

**Role.** You are a different model family from the one that landed the bridge. The job is a harden order for two repositories that now speak to each other: the private game `mcp-tool-shop-org/portlight-bounty`, and the public harness `mcp-tool-shop-org/ai-playtest`. Return the shape at the bottom. Do not open a pull request from this consult.

## The two trees

| repo | what it is | revision this brief cites |
|---|---|---|
| `portlight-bounty` (private) | Godot 4.7 view over a UI-free sim. The bridge lives in the view. | Branch `playtest/godot-rpc-bridge`, commit `7002213687edb21a32685185b15942c81de175a0`. Parent is `0a4d412` (handoff pack, PR #50). The criteria name the game UI at `cc2c50e4`. |
| `ai-playtest` (public, `@mcptoolshop/ai-playtest`) | The harness. RPC is newline-delimited JSON over TCP. | Launcher commit `aace53fdf549d1ff7fd5b8b5518018cce1818bcb` on `feat/rpc-launch`, branched from `main` at `b0f5ede`. The same launcher is also `9feb4a61a86f537caa0e414e4b8a77d4a55a8b3d` on `feat/persona-profiles`. |

The persona-branch copy exists so that checkout is not missing the launcher. It is not the way onto `main`. Open pull requests #14, #15, and #16 are a stack (local seats, then Jev and Harrow Gate, then persona profiles) and must not be merged to ship this launcher.

## What was measured

Editor on this machine: `godot 4.7.stable.official.5b4e0cb0f`. CI tests against Godot 4.7.2. Reports name both. Do not bump the CI pin to match this machine.

After commit `7002213`, from the repo root:

```
node playtest/probe-bridge.mjs
```

printed:

```
godot 4.7.stable.official.5b4e0cb0f
bridge 7777
probe ok: title -> Ada merchant -> Hire -> reset title
```

That probe uses the parent environment, not the harness allowlist. It is a scripted TCP client. It does not load a model and it is not a product score. Path it asserts: `hello` protocol 1, first screen `title` offering `newgame.captains`, `choose` that id, `line` `Ada`, `choose` `newgame.start.merchant`, `chart.hire` offered, status text matches Ada and Docked, `state.hire_on_screen` true, `reset` returns screen `title` and `chart.hire` is absent, then `quit`.

The config the seat will use sets `inheritEnv` false. Under that allowlist (PATH and the harness list, `OPENROUTER_API_KEY` stripped), the same Ada path reached `state.screen` `chart`, `chart.hire` offered, status text matching Ada and Docked, `state.docked` `porto_novo`, `state.place` `Porto Novo`. That allowlist run did not call `reset`. The parent-env probe did.

Harness, on `feat/rpc-launch` at `aace53f`: `tsc --noEmit` clean, and `test/rpc-driver.test.ts` 10 passed. Two of those tests are the new ones: the announced port wins over a closed `driver.port` of 9, and two parallel seats with `game.command` set throw a config error whose message contains "one process" and whose hint contains `--serial`. The hint is not `Error.message`.

`ai-playtest check` on `playtest/portlight-godot.playtest.json` exits 0. It warns, and does not fail, on eleven criteria whose check text joins claims (`but`, `;`, or two sentences): `first-goal-after-begin`, `next-verb-after-accept`, `contract-complete-path`, `encounter-interrupt`, `journal-closure`, `escape-closes-overlay`, `contracts-close-on-undock`, `one-docked-overlay`, `day-report-clarity`, `hire-findable`, `contract-strip-glance`. The wording is locked. Do not split it to silence the warning.

`cargo fmt --all -- --check` passed. `cargo clippy --locked -p portlight-godot --all-targets -- -D warnings` passed. `cargo test -p portlight-godot --lib playtest::` passed `tip_rows_and_payloads_roundtrip` (the six tip rows and the local payloads).

Rust is pinned by `rust-toolchain.toml` to channel `1.98.1`. Use that. Do not reach for an older pin.

## What the handoff pack still says

`docs/playtest/` at `0a4d412` is the pack a reader opens first. It still says the rpc implementation is on hold, and that the game command is the echo fixture, until a go that had not been given when the pack was written. The go was given after the pack. The bridge is commit `7002213`. The pack was not rewritten to pretend it knew that.

The pack cites `playtest-rpc-choice-ids.md` and `playtest-rpc-bridge-brief.md` (sections S1–S4) as locked. Neither file is in the pack, neither is on `main`, and a GitHub code search of the repository did not find them. PR #50's file list does not include them. The six tip rows named in `playtest-harness-status.md` are in the code and must stay spelled this way. Every other id is a local scheme, labeled as such at the top of `crates/portlight-godot/src/playtest.rs`. Do not treat the local strings as a recovered contract.

## The mechanism

Host is the existing Godot view (gdext), not a sidecar and not the batch `portlight script` CLI. The sim crate and `portlight-chart` stay untouched by this bridge.

1. **Flag.** `godot/playtest_bridge.gd` is an autoload. Without `--playtest-port=`, or with port 0, it turns its process off. Launch form, bare `--` required so the flag is a user arg: `godot --headless --path godot -- --playtest-port=7777`. The config in `playtest/` uses command `godot` and args `--headless --path ../godot -- --playtest-port=7777`. Do not pass the smoke flag. Smoke quits after capture frames. `listen(0)` is not used. The port is the flag's port. The process listens on `127.0.0.1`.
2. **When it listens.** Autoload `_ready` runs before the main scene. `_process` waits until the node `Game` exists, then listens, then prints `PLAYTEST_BRIDGE_PORT=<n>`. Piped stdout from `godot` (not the console binary), with `windowsHide`, did print that line. If `Game` is still null, an observation is the text "The game is not ready." with reason `timeout`.
3. **Harness launch.** If `game.command` is non-empty, `src/rpc-launch.ts` spawns it, reads the banner from stdout or stderr, and connects there. The announced port wins over `driver.port`. If the banner never comes, it falls back to the configured port after `connectTimeoutMs`. If the process exits before settle, the error includes the output tail and the child is killed. Exit after settle is ignored. `stop` closes the client and kills the child. Connect failure kills the child. An empty command still attaches to a game that is already listening. `quitInputs` on the Godot config is `[]`, so a turn-limit stop sends protocol `quit` and does not type the word quit.
4. **One client.** The autoload keeps one peer. A new connection calls `playtest_reset` before any observe. `--serial` also sends `reset` between seats. Two seats with `game.command` set, without `--serial`, are a config error. Parallel seats need one process each, and an empty command, because this bridge cannot host them.
5. **Protocol.** `hello` fails closed unless `protocol` is 1, and answers `{protocol:1, game:"Portlight", capabilities:["reset"]}`. `observe` returns the dictionary below. `act` calls `playtest_apply`. An empty string is success and the reply is the new observation. A non-empty string is an error reply and the world does not have to have changed. `reset` returns to a fresh Title. `quit` sets `done` true and `reason` `quit`, then quits the tree. Unknown methods are an error reply.
6. **What a choose does.** Buttons are stamped with meta `playtest_id` in the button factories, before the action moves into the closure. A choose is accepted only when a visible, enabled, stamped button has that id. The id is then parsed and applied with `perform` or `perform_hunt` on the already-bound game. It does not emit `pressed`. A `#[func]` that `bind_mut`s `PortlightGame` must not emit `pressed` on a button whose closure also `bind_mut`s that object. Unstamped buttons are skipped. There is no debug-slug fallback. First-wins dedupe. `pier.hunt.open` is stamped on both the sea Hunt button and the port-row Hunt button. Refresh shows the port row only when docked, and the sea Hunt button only when not docked and a session exists.
7. **Observation.** `text` is screen name, `status_text()`, visible notices, then `id — label` for each offered button, then the log (already capped at 8). `state` is `screen`, `captain`, `day`, `silver`, `docked` (port id), `place`, `open` (desks actually open), `hire_on_screen`. No image field is set. `done` is false until quit. Screen priority: day-report, encounter, hunt, crew, contracts, shipyard, harbour, journal, then the new-game page, else chart. Market is not in `open`. With no session: captain `""`, day 0, silver 0, docked `""`, place `""`.
8. **Line and keys.** `line` writes the visible name `LineEdit` and `draft.name` together, because the next menu action copies the field over the draft. Any other line field returns "no line field is on screen". `key` `escape` or `ui_cancel` calls the same dismiss as real `ui_cancel`, which today closes only Day's report. A no-op is success, so the next observation shows the overlay still open. Any other key returns "that key does nothing here". `call` returns "call is not a Portlight action".
9. **Reset.** `playtest_reset_to_title` drops the line-edit handles first, clears the session, the log, market, stances, encounter, hunt desk, day-report document and memory, and every desk flag, restores the default draft, and opens the Title page. It is not `Action::NewGame`. That action returns to the title while keeping the session, which shows "Back to the chart".
10. **Merchant Ada.** `newgame.start.merchant` is `Action::StartCaptain("merchant")`. The sim constants are name `Ada`, captain id `merchant`, seed `1`. The probe types Ada before that choose because the captains screen reads the name field.

The local ids a first voyage will actually press, all of them local except the six tip rows, include `newgame.captains`, `newgame.start.merchant`, `chart.contracts.open`, `chart.contracts.close`, `contracts.accept.<id>`, `chart.sail.<dest>`, `chart.next_day`, `chart.journal.open`, `chart.journal.close`. Hunt ids are parsed before action ids. `contracts.abandon.confirm` and `contracts.abandon.cancel` are matched before a bare abandon id. The full list is the match in `playtest.rs`.

## The eighteen checks

They live in `playtest/portlight-godot.playtest.json` and match the v0 list. Status is part of the check text. Do not rename an id. Do not flip a status in this harden.

| id | standing |
|---|---|
| `responds-to-input`, `world-moves`, `never-stuck`, `identity-clear`, `contracts-findable`, `maiden-sail-progress`, `contract-complete-path`, `victory-receipt-payoff`, `journal-closure` | Live product gates. |
| `encounter-interrupt` | Live, with a written exception: a seed-1 Grain Road that skips combat is not a fail. The preferred path is Hunt on seed 1. A same-process seed-4 encounter remains the alt. |
| `first-goal-after-begin`, `next-verb-after-accept` | Soft until the Fix A chart-hints pull request merges. Scored, not a product-fail yet. |
| `escape-closes-overlay`, `contracts-close-on-undock`, `one-docked-overlay` | Known-fail until the docked-consistency pull request merges. Do not flip them live. |
| `day-report-clarity`, `hire-findable` | Tip product gates. |
| `contract-strip-glance` | Armed. Not live until PR #48 merges. |

The probe did not press Hire, did not open Contracts, Sail, Day's report, Journal, Hunt, or Escape, and did not undock. `hire-findable` wants a click and a visible crew-count or gold change, or a visible reason when unaffordable. Seeing the button is not that gate.

## Known holes, already measured, not questions

- The probe takes the first reply line and does not check that its `id` matches the request. One call is in flight. The harness driver is the client a seat will use. The probe is the regression script.
- The probe's `exit` listener rejects the port promise even after the port was announced. A later process exit can surface as an unhandled rejection. `rpc-launch.ts` ignores exit after settle. Do not copy the probe's listener into the harness.
- The bridge does not send an image. Headless has no viewport texture. Pixels are an exhibit attached to a failure, not the action channel. Do not build a pixel action channel.
- English README and the handbook drivers page on `feat/rpc-launch` describe launch. The translated READMEs on `main` still say rpc does not need a command. This was not a release, so those files were not rewritten by hand. The next release translates before publish.
- `playtest/README.md` tells a local checkout to run the harness from a sibling `../ai-playtest` build. That is a layout note for this machine's two clones, not a public path.
- Atlas 1.24.0 treats `godot/playtest_bridge.gd` as a Godot entry beside `godot/scenes/main.tscn`. The first CI run failed `ATLAS_STRUCTURE_DRIFT` until `atlas map` rewrote `atlas/`. `atlas check` passes against that map. Do not revert the map to drop the autoload.

## Constraints that are not up for debate

- OpenRouter stays off. No Ollama Cloud. No seat, and no `ai-playtest run`, until a GPU-clear for playtest names a local model tag. Do not load a model to answer this consult.
- Do not edit the sim crate or `portlight-chart` from a bridge or screen change.
- Do not rustfmt all of `game.rs`. Format only the lines this work adds.
- Do not hide chart buttons under the new-game overlay to make a probe look clean. If they are visible and enabled, that is what the seat should see.
- Do not change catalog seed 1. Do not rename locked criteria. Do not treat a soft or known-fail red as a surprise regression.
- Do not merge pull requests #48 or #49 as part of answering this. #50 is already on `main`.
- The public harness must not gain Portlight paths, a home directory, a mailbox, a tailnet address, or a legal name.
- A scripted probe records what it asserted. It does not print PASS or FAIL for the seventeen.

## Questions — only what local measurement cannot settle

**Q1 — the missing contract.** The choice-id file is cited as locked and is not in the tree. The code froze six spellings and labeled the rest local. Should the next pass write the contract from the code (and correct the status doc that cites the missing file), or leave every non-tip id provisional until a recovered file arrives? What would make a later rename of a local id a breaking change for a transcript that already ran?

**Q2 — which gates a probe may assert.** For each of the eighteen ids, say one of: a deterministic assert a no-model probe may add (name the `state` or `text` or `actions` fact), a judgment that stays with a person or a later seat, or not-yet-live. A seed-1 encounter skip is not a fail. Seeing Hire is not `hire-findable`. Do not turn any of these asserts into a product PASS.

**Q3 — Escape and undock.** Today Escape only closes Day's report, and the bridge reports success on a no-op so the next observation still shows the overlay. The known-fail trio stays red until docked-consistency. Should the bridge keep mirroring today's buttons, or should a harden pass implement the intended membership (Escape closes Hunt, Crew, and Day's report, Contracts and Crew force-close on undock) before that pull request? The pack says do not flip the criteria. Argue only if mirroring is the wrong harden.

**Q4 — transcript versus exhibit.** Is there a live criterion whose check cannot be judged from `text` + `state` + `actions`, even by a person reading the transcript? Name the id and the missing exhibit. The observation has no image field. Do not propose driving the game from pixels.

**Q5 — the second consumer.** The smallest next client is either a windowed editor beside a seat, or two seats. What is the smallest protocol harden that consumer needs (matched reply ids, a second peer, a port per process, an image exhibit, something else), and what should stay single-client until that consumer exists?

**Q6 — defects in this mechanism.** Name any reentry, stamp gap, or reset hole you can see, with the file and the function. A choose must keep calling `perform` / `perform_hunt` rather than emitting `pressed`. Unstamped buttons stay omitted. `line` stays on the visible name field until a criterion needs another field.

**Q7 — order.** Give the harden order against four gates that are not this bridge: PR #48 (un-arm `contract-strip-glance` only after it merges), Fix A (the soft pair becomes hard only after it merges), docked-consistency (the trio becomes live only after it merges), and Q1's contract decision. Say what must stay red the day each of those lands.

## Return shape

1. **HARDEN ORDER** — the sequence, in the first lines, and the list of what this pass must not touch.
2. One section per question above, in order. If you disagree with a measurement, name the file and the line. Do not re-derive a measurement you can read.
3. **DEFECTS** — file and function, or "none".
4. **THE FIRST PROBE TO ADD** — assertions only, no model, no product score. Prefer extending `playtest/probe-bridge.mjs` over a new harness verb.
5. **WHAT STAYS RED** — the soft pair, the known-fail trio, the armed strip, and a seed-1 encounter skip.

## Disposition

The return is applied on this branch, with two corrections from the code.

Escape closes Day's report. The probe asserts that. It does not assert that Escape leaves the report open.

An encounter is stepped with the dismiss it offers. Stance ids are recorded when they are present. They are not required on a choice or naval screen.

The observation's `text` is unchanged. Day-report and journal bodies are not copied into the transcript. That waits on a nod, because it changes what a seat reads.

Seed 1 measured on the probe: the first accept id is not fulfilled by selling grain at Al-Manar, so `contracts.complete.<id>` stays off the board and the press is the offered-check. The Grain Road cargo event removes up to three units, so one `chart.buy.grain` leaves an empty hold and the sell is refused. The probe buys four. That is a recorded fact, not a product score.

`docs/playtest/playtest-rpc-choice-ids.md` is the reconstruction. The hire log spells the silver change with `->`.
