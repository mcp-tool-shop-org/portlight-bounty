# Playtest harness status — 2026-10-05 15:37 EDT

Seat: **Game Tester** · Consumer: portlight-bounty  
**Tip pin:** GitHub `mcp-tool-shop-org/portlight-bounty` **`cc2c50e4`**

## Policy (Mike)
- OpenRouter: **off** (forbidden forever for this consumer)
- Local Ollama: **only after GPU clear**
- RunPod: **later**
- Plumbing only until seats light: checkable configs, criteria drafts for GD co-sign
- No `ai-playtest run` from this seat until Mike clears GPU or RunPod is up
- Outbox markdown/json only; no git write; no Robot edits from this probe

## Done
- **Contract-strip product-gate ARMED (15:37 EDT):** `contract-strip-glance` folded into `playtest-criteria-draft.md` + `playtest-portlight-v0.playtest.json` as **ARMED — product-gate on #48 merge** (GD PASS no edits: `design-signoff-pr48-contract-strip.md`). Stub retitled ARMED awaiting #48. **17 live product-gate; 18th armed not live until merge.** Tip pin remains `cc2c50e4`. No soft→hard / known-fail flips; no model run. (Mike Track A portable plan untouched.)
- **Fix A acceptance folded (15:50 EDT):** `fix-a-acceptance-player.md` wired into Trigger A + pier stub; soft pair still soft until Fix A merges
- **Pier-after-Fix-A stub BLOCKED (15:45 EDT):** `playtest-pier-after-fix-a-STUB.md` — time-to-first Contracts/Sail + pier misclicks; fill after Fix A merge (T2 evidence)
- **DRAFT ids co-signed and gated (15:45 EDT):** GD PASS WITH EDITS on `day-report-clarity` + `hire-findable` (`design-cosign-playtest-drafts-cc2c50e4.md`); edits applied; both product-gate. **15 LOCKED core + 2 tip product-gate** (17 ids). Soft Fix A + docked known-fail unchanged. No model run.
- **DRAFT ids awaiting GD (15:40 EDT → retired 15:45):** `playtest-draft-ids-awaiting-gd.md` now records PASS WITH EDITS applied + product-gate (was scoring PASS notes)
- **Mike runbook stub (15:35 EDT):** `playtest-mike-runbook.md` — how to clear GPU, which seats light, OpenRouter/forbidden list; soft no local model run until clear
- Bound seat brief: `seat-brief-game-tester.md`
- Owned config at outbox: `playtest-portlight-v0.playtest.json` (mirrors Robot plumbing fixture shape; both now `check` ok on Robot, see below)
- Seats sidecar: `playtest-seats-README.md` (when to fill seats; OpenRouter ban; swap provider/model without touching criteria)
- Criteria **LOCKED** after GD PASS (2026-10-05): `playtest-criteria-draft.md` — rename `goal-stated` → `first-goal-after-begin` applied; soft-until-Fix-A on that id + `next-verb-after-accept`; docked-consistency trio stays known-fail for Phase 1 before-state (15 ids)
- Still **no** `run`
- Adapter reality unchanged: Portlight CLI is batch `script`, not a REPL; fixture echo game command retained
- **Phase 1 CLI probe DONE (13:15 EDT):** `playtest-cli-probe.md`. `portlight script` reads to EOF and prints one ~40 KB JSON snapshot (only `log[]` is per-step), with no prompt, so stdio `promptPatterns` can match nothing. Captured new game, in-port days, contracts accept/abandon/complete, sail/day ticks (prefix replay), and encounter flee/negotiate plus a sea-rolled duel. `leave` is not a sim verb (in Godot, Leave is UI-only). Finding: while `pending_duel` is set, `advance` silently stalls the day. Raw captures: `/workspace/studio/scratch/pb-001/playtest-cli-probe/raw/`
- **Recommendation: skip to rpc/Godot** (primary). A stdio replay wrapper is noted as an optional sim-level seat only, because 10/15 LOCKED criteria are Godot-UI-only
- Re-checked on Robot (13:12 EDT): `_plumbing-fixture.playtest.json` `ok`; outbox `playtest-portlight-v0.playtest.json` (temp copy) `ok` with 8 compound-claim lint warnings (criteria LOCKED, not edited)

- **Choice-id contract LOCKED (13:25 EDT):** `playtest-rpc-choice-ids.md` freezes brief §4.2 ids (verified vs GitHub main; tip now `cc2c50e4`). Tip rows added: `chart.hire`, `chart.day_report.close`, `pier.hunt.open` / `.close` (UI-only — no `Action::CloseHunt`), `pier.crew.open` / `.close`. FLAG 0. Coord decisions encoded (reset Title/Begin, Hunt preferred + same-process seed4 alt, gdext, Market panel-only, no impl PR).

- **Friction route CONFIRMED by GD** (`design-call-friction-route.md`). Harden checklist **armed** at `playtest-friction-harden.md` (Trigger A = Fix A soft→hard; Trigger B = docked-consistency known-fail→live). **Trigger B** now cites Practitioner `playtest-escape-undock-criteria.md` (Coord confirmed fold) + `design-note-escape-contracts-undock.md`: Hunt·Crew·Day's report on Escape+M1; Contracts+Crew force-close on undock; Journal/Hunt/Day's report sea-capable; open-at-sail smoke + Escape-per-desk required. **Fix B tester-note** (Hunt preferred; no criteria flip). Neither Fix A nor docked-consistency PR on GitHub yet. Soft→hard and known-fail→live **not** flipped.

- **Tip harden DONE (15:30 EDT) @ `cc2c50e4`:** `playtest-harness-harden-cc2c50e4.md`. Criteria tip-alignment + tip product-gate `day-report-clarity` / `hire-findable` (GD PASS WITH EDITS applied 15:45); config name/persona/check text updated; choice ids tip rows; docked trio membership expanded in criteria+config (still known-fail); friction Trigger B membership fold.

## Adapter reality check
`portlight` CLI is **batch script** (`new` / `script file|-`), not an interactive prompt loop. This was confirmed by the Phase 1 probe (`playtest-cli-probe.md`). Phase 0/1 keep the fixture echo (`E:\AI\ai-playtest\test\fixtures\echo-game.mjs` on Robot). Interactive first-voyage seats wait on the **rpc/Godot bridge** (primary). The stdio replay wrapper is a fallback for sim-only checks.

Robot checkout note: local Robot main may lag GitHub tip `cc2c50e4`. No git writes from this seat.

## Next
- **Wait #48 merge → un-ARM `contract-strip-glance`:** record sha; drop ARMED hedges in draft+config; bump live count 17→18; rename stub drop STUB; Coord ping. Still no run.
1. **Waiting Mike GPU clear** (or RunPod) before filling seats and any `ai-playtest run`. OpenRouter stays forbidden.
2. **Wait Fix A merge → Trigger A** (`playtest-friction-harden.md`): harden `first-goal-after-begin` + `next-verb-after-accept` soft→hard; refresh criteria/config/status; Coord ping (still no run). Fix A first if one quota slot.
3. **Wait docked-consistency → Trigger B**: flip trio known-fail→live using membership in `playtest-escape-undock-criteria.md`; graduate OPTIONAL `ui.close` if Escape lands
4. **Waiting on Builder spike** (S1–S4 per `playtest-rpc-bridge-brief.md`) — brief + choice-id contract locked; **impl PR still hold** until Mike/Coordinator go. Host prefer gdext; use locked ids in `playtest-rpc-choice-ids.md`.
5. Optional, only if asked: draft a stdio replay-wrapper spec for a sim-level seat. Optional: re-probe after Robot is updated to GitHub tip
6. GD FYI: prior compound-claim lint warnings from `ai-playtest check` remain non-blocking; LOCKED core ids not renamed; tip product-gate ids `day-report-clarity` + `hire-findable` co-signed
