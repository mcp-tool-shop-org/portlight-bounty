# Portlight ai-playtest plan (portable) — for another session

**Owner handoff:** Mike → any session / Game Tester / Coordinator  
**Date:** 2026-10-05 · **Tip pin:** `cc2c50e4` (`mcp-tool-shop-org/portlight-bounty`)  
**Consumer:** portlight-bounty · **OpenRouter:** never  
**Return results to:** Coordinator (this chat)

---

## 0. What you are bringing back

One report file (preferred) or a short paste:

`outbox-PB-001/playtest-<label>-REPORT.md`

Must include:
1. Tip sha you played (`git rev-parse HEAD` or UI build note)
2. Track used (A human Godot / B dry harness / C lit Ollama — see below)
3. Per-criterion PASS / FAIL / SOFT / KNOWN-FAIL / N/A with one-line evidence
4. Top 3 frictions felt (player words)
5. Anything broken / blocked

---

## 1. Reality check (read first)

| Surface | State @ `cc2c50e4` |
|---------|-------------------|
| Criteria | **17 product-gate** (15 LOCKED core + `day-report-clarity` + `hire-findable`) |
| Soft (score, not product-fail) | `first-goal-after-begin`, `next-verb-after-accept` — until Fix A merges |
| Known-fail (expect fail) | `escape-closes-overlay`, `contracts-close-on-undock`, `one-docked-overlay` — until docked-consistency |
| ai-playtest config | `playtest-portlight-v0.playtest.json` — seats **dark**; game = **echo fixture** |
| Interactive Godot voyage via ai-playtest | **Blocked** — rpc bridge impl on hold |
| Local model seats | Only after you say **“GPU clear for playtest”** + Ollama model tag |
| RunPod | Later — same criteria, swap seat only |

**Honest ask for this handoff:** prefer **Track A** (you play Godot, score criteria). Track B/C prove harness plumbing; they do **not** yet drive the real game.

---

## 2. Paths (Robot + studio outbox)

| What | Path |
|------|------|
| This plan | `outbox-PB-001/playtest-plan-portable-for-mike.md` |
| Runbook | `outbox-PB-001/playtest-mike-runbook.md` |
| Criteria (binding) | `outbox-PB-001/playtest-criteria-draft.md` |
| Config | `outbox-PB-001/playtest-portlight-v0.playtest.json` |
| Friction triggers | `outbox-PB-001/playtest-friction-harden.md` |
| Fix A acceptance | `outbox-PB-001/fix-a-acceptance-player.md` |
| Escape known-fail | `outbox-PB-001/playtest-escape-undock-criteria.md` |
| First-voyage brief | `outbox-PB-001/playtest-first-voyage.md` (spine outdated on Hunt/Crew — use tip table below) |
| ai-playtest CLI (Robot) | `node E:\AI\ai-playtest\dist\cli.js` |
| Echo fixture | `E:\AI\ai-playtest\test\fixtures\echo-game.mjs` |
| Game checkout | `E:\AI\ai-rpg-engine` siblings / portlight-bounty local as you use |

Copy the outbox files into the other session if that machine cannot see `/workspace/studio`.

---

## 3. Track A — Human Godot first voyage (primary)

**Build:** Godot interactive on tip ≥ `cc2c50e4` (after #46 Day's report + #47 Hire; Hunt+Crew LIVE).

**Identity:** New game → Merchant → name **Ada** → Begin (seed 1).

### Script (~15–20 min)

| # | Do | Score against |
|---|-----|----------------|
| 1 | Begin at Porto Novo | `identity-clear`, `first-goal-after-begin` (**soft**) |
| 2 | Find **Contracts** on pier; Accept one starter | `contracts-findable`, `next-verb-after-accept` (**soft**) |
| 3 | Note pier label **Hire** (not “Hire sailor”) | `hire-findable` |
| 4 | Optional Harbour peek; Close | curiosity only |
| 5 | Sail toward Al-Manar / Next day as needed | `maiden-sail-progress`, `world-moves`, `responds-to-input` |
| 6 | On notable Next day: Day's report opens; Escape or Close | `day-report-clarity`; Escape also feeds `escape-closes-overlay` (**known-fail** if other desks Escape no-op) |
| 7 | At dest: sell there - the sale pays the contract | `contract-complete-path` |
| 8 | Prefer **Hunt** target for an encounter on seed 1 (Grain Road skip ≠ fail) | `encounter-interrupt` |
| 9 | Finish victory (Spare / Take all) | `victory-receipt-payoff` |
| 10 | Open **Journal** | `journal-closure` |
| 11 | Cross-check never stuck / input / world | `never-stuck`, `responds-to-input`, `world-moves` |
| 12 | Escape on Contracts / Hunt / Crew (expect gaps) | `escape-closes-overlay` **known-fail** |
| 13 | Leave Contracts open → Sail → assert overlay | `contracts-close-on-undock` **known-fail** |
| 14 | Open two desks without closing | `one-docked-overlay` **known-fail** |

### Scoring rules

- **PASS / FAIL** on product-gate ids (17).
- Soft pair: report PASS/FAIL but mark **SOFT** — do not treat as ship-blocker until Fix A.
- Known-fail trio: report actual behavior; mark **KNOWN-FAIL** — expected red until docked-consistency.
- Hunt missing on seed 1 Grain Road: `encounter-interrupt` = **N/A / not-fail**; note if you used Hunt or `encounter_seed4` alt.

---

## 4. Track B — Dry harness only (no GPU)

On Robot (or box with Node):

```text
node E:\AI\ai-playtest\dist\cli.js check <path-to>\playtest-portlight-v0.playtest.json
```

Expect **ok**. Do **not** `run`. Paste check output into the report.

Optional: confirm JSON still lists 17 criteria ids matching `playtest-criteria-draft.md`.

---

## 5. Track C — Lit Ollama seat (only if GPU free)

1. In Coord/GT chat (or note in report): **GPU clear for playtest** + model tag (e.g. `llama3.1:8b`).
2. Fill only `seats[]` in the config (keep criteria ids).
3. `ai-playtest check` → must ok.
4. Short `ai-playtest run` (prefer `--serial` when rpc exists — today still **echo fixture**, so this validates seat plumbing, not Portlight feel).
5. Revoke with “GPU busy / stop seats” when done.
6. **Never** OpenRouter.

Until rpc bridge lands, Track C does **not** replace Track A for product judgment.

---

## 6. Criteria checklist (copy into report)

### Product-gate (hard)

- [ ] `responds-to-input`
- [ ] `world-moves`
- [ ] `never-stuck`
- [ ] `identity-clear`
- [ ] `contracts-findable`
- [ ] `maiden-sail-progress`
- [ ] `contract-complete-path`
- [ ] `encounter-interrupt`
- [ ] `victory-receipt-payoff`
- [ ] `journal-closure`
- [ ] `day-report-clarity`
- [ ] `hire-findable`
- [ ] *(plus any other LOCKED core ids listed in criteria draft — score all 15+2)*

### Soft until Fix A

- [ ] `first-goal-after-begin` — expect missing breadcrumb today
- [ ] `next-verb-after-accept` — expect missing breadcrumb today

### Known-fail until docked-consistency

- [ ] `escape-closes-overlay` — Day's report Escape may work; other desks often Close-only
- [ ] `contracts-close-on-undock`
- [ ] `one-docked-overlay`

### Pending stubs (do not hard-fail product yet)

- [ ] `contract-strip-glance` — DRAFT for PR #48; score only if #48 already on your tip
- Pier-after-Fix-A (time-to-Contracts/Sail + misclicks) — only after Fix A merges

Full pass/fail text: `playtest-criteria-draft.md`.

---

## 7. Do not touch

- OpenRouter / any cloud LLM seat for this consumer
- Renaming LOCKED criteria ids
- Shipping rpc bridge PR without Coord/Mike go
- Changing catalog seed 1 (Fix B)
- Treating soft or known-fail reds as unexpected regressions

---

## 8. After you return

Coordinator will:
1. Fold your report into GT harness status
2. Open Fix A / docked-consistency / #48 follow-ups from real fails
3. Flip Trigger A/B only when those PRs merge (not from this session alone)

---

**One-liner:** Play tip `cc2c50e4` Godot as Ada (Track A), score the 17 product-gate + soft + known-fail ids, write `playtest-<label>-REPORT.md`, bring it back. Optional Track B `check` only; Track C needs GPU clear and still won’t drive Godot until rpc.
