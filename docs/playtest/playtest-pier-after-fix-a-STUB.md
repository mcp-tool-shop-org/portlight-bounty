# Playtest stub — pier after Fix A (time-to-first + misclicks)

**Status: BLOCKED until Fix A merges** · Seat: Game Tester · Date: 2026-10-05  
**Tip baseline (pre-Fix A):** `cc2c50e4` · Consumer: portlight-bounty  
**Soft:** no model run / no live scoring until Mike GPU clear (`playtest-mike-runbook.md`). OpenRouter off.  
**Trigger:** Coordinator routes this when Fix A lands — then un-BLOCK, fill numbers, land as `playtest-pier-after-fix-a.md` (drop STUB).

## Why this note exists

GD playtest hook (`design-call-pier-density.md`): after Fix A merges, first-10 run should note **time-to-first Contracts or Sail press** and **any pier misclicks**. That evidence decides whether pier-density **T2** (fold Hire / Provisions into Crew) proceeds.

Also hardens soft criteria after Trigger A (`playtest-friction-harden.md`): `first-goal-after-begin`, `next-verb-after-accept`.

## Not yet

| Hold | Until |
|------|--------|
| Live scoring / product-fail on soft pair | Fix A merge + Trigger A harden |
| Model seat `run` | Mike GPU clear |
| T2 Hire fold | This note’s evidence + fresh GD call |
| Docked-consistency flips | Separate Trigger B |

---


## Fix A player acceptance (pre-filled from Practitioner)

Source: `fix-a-acceptance-player.md`. On un-BLOCK after merge, verify these **before** scoring soft→hard:

- ≤1 active muted/cream chart breadcrumb; never blocks input; dismisses on step success
- Hint 1 after Begin → Contracts; hint 2 after Accept → sail/sell/Complete; hint 3 after first victory → Journal
- Non-goals (not Fix A fails): Harbour/Hire/Shipyard/Crew/Hunt as breadcrumbs; Deliver button; pier reflow; Escape/undock

Then record time-to-first + misclicks below for pier-density T2 evidence.

## Script (fill after Fix A — Ada seed 1)

**Build:** tip at/after Fix A merge sha ______. Interactive Godot (or rpc when bridge exists).

| # | Do | Record |
|---|-----|--------|
| 1 | New game → Merchant Ada → Begin | Wall-clock start `T0` |
| 2 | Note Fix A hint 1 visible? (`Open Contracts…`) | Y/N · copy exact |
| 3 | First intentional **Contracts** open **or** **Sail** press | `T1` · which verb · seconds `T1−T0` |
| 4 | Path to that press: any **misclick** on pier? | Desk hit · recovered? · seconds lost |
| 5 | Accept a starter contract → Close | Hint 2 visible? |
| 6 | First Sail toward Al-Manar (or Market buy first if natural) | Extra misclicks? |

**Misclick taxonomy (tick what happened):**

- [ ] Hire (one-shot) instead of Contracts / Sail  
- [ ] Provisions +5  
- [ ] Work / Shipyard / Harbour / Crew / Hunt  
- [ ] Market when aiming Contracts  
- [ ] Journal (top row) when aiming pier  
- [ ] Other: _______________

**Feel (one line each):** saw · confused · wanted next.

---

## Metrics to land (post-merge fill-in)

| Metric | Value | Notes |
|--------|-------|-------|
| Tip / Fix A PR sha | | |
| Time to first Contracts **or** Sail (`T1−T0`) | ___ s | Prefer wall-clock; say if approximate |
| First successful verb | Contracts / Sail | |
| Pier misclick count (first 10 min / until first Sail) | ___ | List desks |
| Fix A hint 1 helped? | Y / N / unclear | |
| Soft criteria after Trigger A | `first-goal-after-begin` · `next-verb-after-accept` | pass / fail / soft-only |

## Evidence → T2

Hand Coordinator + GD:

- If first Contracts/Sail is **fast** and misclicks are rare → T2 stays deferred.  
- If pier still reads as “which door is the voyage?” with Hire/Provisions traps → cite this note for T2 re-eval (Godot-only layout; `hire-findable` would need rewrite + fresh GD co-sign).

Do **not** implement T2 from this stub.

---

## Related

- Fix A brief: `playtest-first-voyage.md` §5  
- Pier density call: `design-call-pier-density.md`  
- Friction harden Trigger A: `playtest-friction-harden.md`
- Fix A player acceptance (Practitioner): `fix-a-acceptance-player.md` — use on soft→hard flip  
- Criteria: `playtest-criteria-draft.md` (soft pair until Fix A)

**Coord ping when unblocked:** Pier-after-Fix-A stub ready to fill — awaiting Fix A merge sha.
