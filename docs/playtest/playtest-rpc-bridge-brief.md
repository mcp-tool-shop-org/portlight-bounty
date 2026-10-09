> Copied from the Builder outbox for reference. It is the spike plan from before the bridge landed. Where it disagrees, `playtest/README.md` and `docs/playtest/playtest-rpc-choice-ids.md` are current.

# Playtest RPC bridge brief — Portlight Godot (spike plan)

**Date:** 2026-10-05 · **Seat:** Builder · **Consumer:** portlight-bounty  
**Status:** Design note + spike plan only. **No implementation PR. No cloud agent. No merge** unless Coordinator says go.  
**Spec:** `E:\AI\ai-playtest\docs\engine-bridge.md` (ai-playtest protocol 1)  
**Locked criteria:** `playtest-criteria-draft.md`  
**Config (later swap):** `playtest-portlight-v0.playtest.json` — keep fixture echo until driver flips to `rpc`  
**Why now:** Game Tester Phase 1 (`playtest-cli-probe.md`) — `portlight script` is batch EOF→one JSON; stdio cannot drive interactive first-voyage. Skip to rpc/Godot.

---

## 1. Goal

Expose Portlight’s **Session-backed Godot host** to `ai-playtest`’s **rpc** driver so a seat can walk the first-voyage spine without pixels:

**New game → Contracts → Harbour peek → Sail → Sell/Complete → Encounter leave → Victory receipt → Journal.**

Sim stays authoritative. The bridge **never invents rules**, offer ids, prices, or encounter outcomes — it only surfaces what `PortlightGame` already shows and dispatches the same `Action` paths UI buttons already call.

---

## 2. Protocol (ai-playtest, do not fork)

Newline-delimited JSON over TCP. Request/response, one message per line.

| Method | Role |
|--------|------|
| `hello` | Handshake; answer `{protocol:1, game:"Portlight", capabilities:[…]}`. Fail closed if missing / not protocol 1. |
| `observe` | Return observation (below). |
| `act` | Typed action: prefer `choose`; also `key` / `line` / `call`. **Reply only when ready for the next input.** |
| `reset` | Fresh session without relaunching the process. |
| `quit` | Final observation with `done:true`, then quit the process. |

Observation fields used by the harness:

| Field | Portlight use |
|-------|----------------|
| `text` | Player-facing prose: recent log lines + overlay body (ASCII). Model channel. |
| `state` | Structured coverage keys (§6). Object preferred. |
| `actions` | Prefer `{"kind":"choice","options":[{id,label},…]}` = **currently visible** buttons only. |
| `done` | Session over (rare on first-voyage; use when quit / hard fail). |
| `reason` | `win` / `lose` / `quit` / `stuck` / `timeout` when applicable. |
| `image` | **Out of scope for spike** — headless has no viewport texture. |

Godot wiring lessons from the spec (must follow):

1. Autoload (or host node) `process_mode = PROCESS_MODE_ALWAYS` so pause does not deafen the bridge.
2. Parse `--playtest-port=N` from **user args after bare `--`** (tolerate full cmdline as fallback).
3. `set_no_delay(true)`; buffer **bytes**, split on `\n`.
4. Single client paste is OK; multi-seat runs use **`--serial`** (one Godot, `reset` between seats). Missing/throwing `reset` → `E_RESET`.
5. Portlight’s `Action` handlers are largely sync today; if any await appears later, handlers must `await` observation/reset.

Print once on listen: `PLAYTEST_BRIDGE_PORT=<n>` (matches engine-bridge sample).

---

## 3. Host placement (spike default)

| Choice | Recommendation |
|--------|----------------|
| Language | **gdext / Rust inside `portlight-godot`** (Decision §14.4) — same `Action` paths as UI. GDScript paste from engine-bridge only if the spike proves too slow; note that outcome here if so. |
| Gate | CLI user-arg **`--playtest-port=7777`** (same pattern as `--contracts-screen` / `--encounter-screen`). Never enabled in shipping builds without the flag. |
| Default port | **`7777`** (engine-bridge sample + ai-playtest rpc default). Bind `127.0.0.1` only. |
| Launch | `godot --headless --path godot -- --playtest-port=7777` (bare `--` required for user args). Interactive GL optional for human debug; seats should use headless + text/state. |
| Pause | After `act`, pause tree (or freeze input), apply action, poll `is_ready_for_input` with a frame cap (~600), then reply. Chart day tween (`DAY_TWEEN_SECS` ≈ 0.45s) must finish before ready. |

No playtest/TCP code exists in tree today (verified on flat-quay tip and by grepping `portlight-godot`).

---

## 4. First-voyage spine → protocol mapping

Base tip for behaviour: GitHub `main` at or after **`97a1c314`** (#41 victory receipt). Overlay set includes Contracts (#33), Shipyard (#37), Harbour office (#38), Journal (#39), Encounter overlay, New game (#30). Art-only tips (e.g. #42/#44) do not change Action surface.

### 4.1 Screens / overlays (observation `state.ui`)

| Voyage step | Godot surface | How player enters | Close / leave |
|-------------|---------------|-------------------|---------------|
| New game | `NewgamePage` overlay | Boot → Title; `StartCaptain("merchant")` / Begin | Hidden after begin |
| Chart (docked) | Main chart + `port_row` | After begin | — |
| Contracts | Contracts overlay | `Action::OpenContracts` (`port_row`) | `CloseContracts` |
| Harbour peek | Harbour office overlay | `Action::OpenHarbour` | `CloseHarbour` (peek only; no finance curriculum) |
| Market sell | Panel market toggle | `ToggleMarket`; `Sell(good_id)` | Toggle off |
| Sail / Next day | Chart | `Sail(dest)` or chart arm; `NextDay` | — |
| Encounter | Encounter overlay | Sea agency / scripted | `LeaveEncounter` (**UI-only** — clears overlay, **no** `Session` call) |
| Victory receipt | Log / notice after Spare/Take all | `Spare` / `TakeAll` → pending victory finish | — |
| Journal | Journal overlay | `OpenJournal` (top row; works at sea) | `CloseJournal` |

`leave_encounter` today (`game.rs`): sets `encounter = None`, clears scripted captain, `refresh()` — matches CLI probe finding that **Leave is not a sim verb**.

### 4.2 `act` → `Action` (prefer `choose`)

Expose **only visible** buttons as `choice` options. Suggested stable `id`s (labels can stay player-facing ASCII):

| Choice id | Maps to | Spine use |
|-----------|---------|-----------|
| `newgame.open_captains` | `OpenCaptains` | Title |
| `newgame.start_merchant` | `StartCaptain("merchant")` | Begin Ada merchant (seed 1) |
| `newgame.back` | `NewgameBack` | |
| `chart.next_day` | `NextDay` | Maiden sail ticks |
| `chart.sail.<port_id>` | `Sail(port_id)` | e.g. `chart.sail.al_manar` |
| `chart.journal.open` / `.close` | `OpenJournal` / `CloseJournal` | Closure beat |
| `chart.market.toggle` | `ToggleMarket` | |
| `chart.buy.<good>` / `chart.sell.<good>` | `Buy` / `Sell` | Contract goods |
| `pier.contracts.open` / `.close` | `OpenContracts` / `CloseContracts` | |
| `pier.contracts.refresh` | `RefreshContracts` | Empty board |
| `pier.contracts.accept.<id>` | `AcceptContract(id)` | Offer id from board |
| `pier.contracts.complete.<id>` | `CompleteContract(id)` | After delivery |
| `pier.harbour.open` / `.close` | `OpenHarbour` / `CloseHarbour` | Peek only |
| `encounter.choice.<negotiate\|flee\|fight>` | `EncounterChoice` | Approach |
| `encounter.naval.<action>` | `Naval` | |
| `encounter.board` | `Board` | |
| `encounter.combat.<action>` | `Combat` | |
| `encounter.spare` / `.capture` / `.take_all` | `Spare` / `Capture` / `TakeAll` | Victory finish |
| `encounter.leave` | `LeaveEncounter` | After outcome / Return |
| `encounter_seed4` | Scripted Fix B path (Ada privateer, seed 4, thornport / `raj_the_quiet` approach) — same process, **no** second Godot | Encounter beat; mirrors `--encounter-screen` without auto-smoke playthrough |
| `ui.close` (optional) | Same as visible Close | For docked-consistency once Escape PR lands |

**Do not** expose Shipyard / Hire / Work / Hunt / Crew on the first-voyage choice set unless the seat config asks for curiosity beats. Illegal / hidden ids → bridge error `{message:…}` (harness records `illegal-action`).

`key` / `line`: optional later for Escape (`ui_cancel`) once docked-consistency PR lands; spike can omit Escape and use explicit Close choices.

### 4.3 Encounter seed policy (Fix B)

| Seed path | When |
|-----------|------|
| **Default reset** | Ada / merchant / **`FIRST_PLAYABLE_SEED = 1`** — Grain Road may skip combat (**not a fail** for `encounter-interrupt`). |
| **Encounter beat** | Reuse scripted constants already in `logic.rs`: Ada **privateer**, **`SCRIPTED_SEED = 4`**, depart **thornport**, captain **`raj_the_quiet`** — same as `--encounter-screen`. Bridge may accept `reset` params *or* a one-shot `call` / dedicated choose `playtest.encounter_screen` that runs `prepare_scripted_voyage` + open approach **without** auto-playing the smoke script. |

Do **not** change catalog seed 1 for product.

---

## 5. Reset semantics

| Concern | Spike behaviour |
|---------|-----------------|
| Default `reset` | Full **Title → Begin** path (Decision §14.1): clear overlays/encounter, land on new-game **Title**, ready for `newgame.open_captains` → `newgame.start_merchant` (Ada / merchant / seed 1). Do **not** auto-`start_game()` past Begin on default reset. Optional later: fast-docked Ada seed 1 for spine-only seats. |
| Identity for criteria | Name Ada, captain merchant, seed **1**, Porto Novo. |
| Per-connection | On new TCP peer: restore starting world (engine-bridge lesson) — same as `reset`, not “wherever the last client left off”. |
| Schedules | Prefer world clock only; if any host-side counter exists outside Session, document and reset it or require one process per seat. |
| `quit` | Ends Godot process. `--serial` sends once after last seat. |

Open: whether `reset` params may select `{seed, captain, scripted_encounter:true}` without a second process. Spike should stub params or a single extra choose id; full PR hardens the schema.

---

## 6. `state` keys for coverage / criteria

Keep small, stable, JSON-serializable. Scorer and absorbing-SCC hash this object.

| Key | Type | Criterion / use |
|-----|------|-----------------|
| `seed` | number | Fixture |
| `day` | number | `world-moves`, `maiden-sail-progress` |
| `silver` | number | `world-moves`, `identity-clear` |
| `provisions` | number | optional |
| `hull` / `hull_max` | number | optional |
| `captain_name` / `captain_type` | string | `identity-clear` |
| `docked_port_id` / `docked_port_name` | string\|null | identity, sail, undock |
| `voyage_status` | `"in_port"`\|`"at_sea"` | sail |
| `voyage_progress` / `voyage_distance` | number | `maiden-sail-progress` |
| `destination_port_id` | string\|null | |
| `ui.overlay` | `"none"`\|`"newgame"`\|`"contracts"`\|`"harbour"`\|`"shipyard"`\|`"journal"`\|`"encounter"` | findability, never-stuck, one-overlay (Market **not** an overlay in spike — §14.5) |
| `ui.market_open` | bool | Panel flag only (spike); docked-consistency may fold Market into overlay rule later |
| `ui.newgame_page` | string\|null | |
| `ui.port_row_visible` | bool | pier |
| `ui.hint` | string\|null | Fix A soft criteria when present |
| `contract.active_id` | string\|null | |
| `contract.offer_ids` | string[] | accept path |
| `contract.can_complete` | bool | `contract-complete-path` |
| `encounter.phase` | string\|null | `encounter-interrupt` |
| `encounter.leave_visible` | bool | Escape policy later |
| `pending_duel` | bool | stall detection (CLI probe) |
| `pending_victory` | bool | receipt path |
| `victory_paths` | string[] | `identity-clear`, journal |
| `log_tail` | string[] (last N) | receipts / hints (also mirrored in `text`) |
| `save_slot_line` | string\|null | optional Save beat |

`text` should be a readable concatenation: status one-liner + overlay title/body excerpt + last few log lines (strip Rich markup if any). Prefer human chart copy over dumping full Session JSON.

---

## 7. Criteria observability (rpc vs CLI)

| Criterion | Needs Godot rpc? |
|-----------|------------------|
| `responds-to-input`, `world-moves`, `maiden-sail-progress`, `contract-complete-path` | Scorable in sim **or** rpc |
| `identity-clear`, `contracts-findable`, `first-goal-after-begin`, `next-verb-after-accept`, `encounter-interrupt`, `victory-receipt-payoff`, `journal-closure`, `never-stuck` | **rpc/Godot** |
| `escape-closes-overlay`, `contracts-close-on-undock`, `one-docked-overlay` | **rpc/Godot**; known-fail until docked-consistency PR |

---

## 8. Config flip (when bridge exists)

Today (`playtest-portlight-v0.playtest.json`): fixture echo + stdio patterns.

Target shape (illustrative — do not edit config until bridge boots):

```json
{
  "driver": { "kind": "rpc", "port": 7777 },
  "game": {
    "command": "godot",
    "args": ["--headless", "--path", "godot", "--", "--playtest-port=7777"]
  }
}
```

Exact `command`/`args` paths are Robot-local; Game Tester owns the config swap. Criteria ids stay LOCKED. Seats stay dark until Mike clears GPU / RunPod. OpenRouter remains forbidden. Run with **`--serial`**.

---

## 9. Spike plan (no PR required yet)

| Step | Work | Exit |
|------|------|------|
| S0 | This brief (done when landed in outbox) | Coord ACK |
| S1 | Spike branch locally: TCP hello/observe/act/reset/quit against a minimal `_observation` / `_apply` that wraps `PortlightGame::perform` + visible-button enumeration | Manual `nc`/tiny client round-trip |
| S2 | Map first-voyage happy path seed 1: begin → contracts accept → harbour open/close → sail Al-Manar → next day → sell → complete → journal | Observation asserts vs CLI oracle (`playtest-cli-probe.md` §6) |
| S3 | Encounter path seed 4 / scripted approach → spare/take_all → leave → receipt lines in `text`/`log_tail` | `encounter-interrupt` + `victory-receipt-payoff` wireable |
| S4 | Document launch line + `PLAYTEST_BRIDGE_PORT` + failure modes | Hand to Game Tester for config draft |

**Out of spike:** Escape/`ui_cancel`, full Shipyard/Hunt/Crew, `image` frames, OpenRouter, any merge, cloud agent.

---

## 10. Estimate

| Track | Size | Notes |
|-------|------|-------|
| **Spike** (S1–S4) | ~0.5–1.5 day Builder | Thin TCP + observation/action mirror; headless; no art; no criteria rename |
| **Full PR** | ~2–4 days + review | Hardened choice id scheme, reset params, ready-predicate for day tween / encounter resolution, `--serial` soak, Game Tester config + plumbing seat (still no model until GPU clear), docs in repo `docs/` or playtest folder |
| **Follow-ons** | separate PRs | Docked-consistency (Escape etc.); Fix A hints → soft→hard; Hunt encounter breadcrumb |

Risk: enumerating “visible buttons” cleanly may need a small refactor so `refresh()` builds an `Vec<(id,label,Action)>` the bridge can read — today buttons are constructed imperatively in `game.rs`.

---

## 11. Open questions (Coordinator / Game Tester / GD)

**Resolved 2026-10-05 by Coordinator** — see §14 Decisions. Left here for history; do not re-litigate without Coord.

---

## 12. Related outbox

- `playtest-cli-probe.md` — stdio verdict; skip-to-rpc
- `playtest-criteria-draft.md` — LOCKED ids
- `playtest-portlight-v0.playtest.json` — fixture until rpc
- `playtest-harness-status.md` — Phase 1 status
- `playtest-first-voyage.md` — spine + Fix A/B
- `design-call-docked-consistency.md` — Escape / one-overlay (pending PR)

---

## 13. Handoff

Landed: `outbox-PB-001/playtest-rpc-bridge-brief.md`  
§14 Decisions appended 2026-10-05 (Coordinator ACK).  
Builder **idle on implementation** until Mike/Coordinator say go.  
Game Tester: lock choice-id table in outbox after a pass. GD FYI on reset/encounter.

---

## 14. Decisions (Coordinator 2026-10-05)

ACK on brief. Builder stays **idle on impl**. Impl PR / cloud agent still **hold** (quota tight) until Mike/Coordinator go.

| # | Call |
|---|------|
| 1 | **Reset (default):** full new-game **Title → Begin** path — needed for `identity-clear` / `first-goal-after-begin`. Optional later: fast-docked Ada seed 1 for spine-only seats. |
| 2 | **Encounter:** same Godot process — `act` choose `encounter_seed4` (or reset param) mirroring Fix B / `--encounter-screen` seed 4. **No** second Godot process in the spike. |
| 3 | **Choice ids:** freeze this brief’s proposed ids as the contract. Game Tester locks the table in outbox after a pass. |
| 4 | **Host:** prefer **gdext in `portlight-godot`** (same `Action` paths as UI). GDScript paste from engine-bridge only if the spike proves too slow — note that outcome in the brief if it happens. |
| 5 | **Market:** **panel flag only** for the spike (matches docked-consistency: full-overlay rule may include Market later; do not invent overlay semantics now). |
| 6 | **Impl PR:** **hold** until Mike/Coordinator go. |

Routing: GT locks choice-id table; GD FYI on reset/encounter.
