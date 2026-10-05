# Playtest runbook stub — Mike (Phase 0)

**Seat:** Game Tester · **Consumer:** portlight-bounty · **Date:** 2026-10-05  
**Tip pin:** `cc2c50e4` · **Config:** `playtest-portlight-v0.playtest.json`  
**Status:** Seats **dark** until you clear GPU (or RunPod later). Soft: **no local model `run`** until that clear.

This is the one-pager for lighting seats. Longer detail lives in `playtest-seats-README.md` and `playtest-harness-harden-cc2c50e4.md`.

---

## 1. Do not touch

| Forbidden | Why |
|-----------|-----|
| **OpenRouter** (any seat / key / provider) | Policy forever for this consumer — too expensive |
| `ai-playtest run` before GPU clear | Would hit the placeholder Ollama seat |
| Renaming LOCKED criteria ids | Breaks diffs; needs GD co-sign |
| Shipping rpc bridge / impl PR | Hold until you or Coordinator say go |
| Changing catalog seed 1 | Fix B — Grain Road duel-free by design |

Fixture `check` and outbox docs edits are fine. Robot plumbing fixture stays green.

---

## 2. How to clear GPU (local seats)

When the local GPU is free for a short playtest label:

1. Tell **Game Tester** (or Coordinator) in chat: **“GPU clear for playtest”** + the **Ollama model tag** to use (e.g. `llama3.1:8b`).
2. Game Tester fills only `seats[]` in `playtest-portlight-v0.playtest.json` (and updates the persona line to name the cleared host). Criteria ids stay put.
3. On Robot: `node E:\AI\ai-playtest\dist\cli.js check playtest\…` (or the outbox copy path Game Tester names) — must be **ok**.
4. Then a **short** `ai-playtest run` is allowed for that label. Prefer **`--serial`** once rpc exists.
5. Revoke anytime: “GPU busy / stop seats” → seats go dark again; no further `run`.

Until step 1, treat `pending-local` as schema documentation only.

---

## 3. Which seats light (when cleared)

| Host | What lights | When |
|------|-------------|------|
| **Local Ollama** | One `provider: ollama` seat, model tag you name | After your GPU clear |
| **RunPod** (later) | Same criteria; swap endpoint / model only | When you open RunPod for playtest |
| **OpenRouter** | Nothing | Never |
| **Fixture echo (stdio)** | No model — plumbing only | Always (check / dry harness) |
| **rpc / Godot** | Interactive first-voyage seats | After Builder bridge lands + you allow | 

Today the game command is still the **echo fixture**. Interactive voyage needs the **rpc bridge** (brief + choice ids locked; impl PR still hold). CLI `portlight script` is batch JSON — not a REPL.

---

## 4. What a first lit run should cover

- Short label only (fixture or rpc when ready) — not a marathon.
- Criteria: LOCKED 15 + DRAFT `day-report-clarity` / `hire-findable` (scored when present; product-gate after GD co-sign).
- Soft (not product-fail until Fix A): `first-goal-after-begin`, `next-verb-after-accept`.
- Known-fail until docked-consistency: `escape-closes-overlay`, `contracts-close-on-undock`, `one-docked-overlay`.
- Encounter: prefer **Hunt target** on seed 1; Grain Road skip ≠ fail; `encounter_seed4` alt OK.

Land reports under `outbox-PB-001/playtest-<label>-REPORT.md` — no full transcripts in chat.

---

## 5. Still waiting (not your GPU)

| Item | Owner |
|------|--------|
| Fix A chart-hints PR | Builder / Coord (quota) — then Trigger A harden |
| Docked-consistency PR | Builder / Coord — then Trigger B flip |
| GD co-sign DRAFT ids | Game Designer (routed) |
| rpc bridge impl | Builder — hold |

---

**One line for Coord:** Mike runbook stub at `playtest-mike-runbook.md` — seats stay dark until GPU clear phrase + model tag; OpenRouter off.
