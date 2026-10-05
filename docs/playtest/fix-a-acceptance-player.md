# PB-001 — Fix A player-facing acceptance

Seat: **Practitioner** · For: **Coordinator / Game Tester / Game Designer** · Date: 2026-10-05  
Tip baseline: **`cc2c50e4`** · Fix A still **parked on quota** · **No code**  
Briefs: `playtest-first-voyage.md` §5 (APPROVED) · `design-call-friction-route.md` · `playtest-friction-harden.md` Trigger A · `playtest-criteria-draft.md`

---

## 1. What the player should notice (after Fix A lands)

One muted/cream chart breadcrumb at a time (≤1 active). ASCII. Never blocks input. Dismisses once that step succeeds.

| # | When it appears | What they read (binding intent; copy may tune in review) | What success feels like |
|---|---|---|---|
| **1 — First goal** | After **Begin** on a fresh voyage | `Open Contracts for a starter job.` | Dense pier still there, but one clear starting verb — open Contracts |
| **2 — Next verb** | After **first Accept** | `Load the goods, sail to the destination, then sell there.` | Sail → sell-at-dest → Complete path is named (softens Deliver confusion without a Deliver button) |
| **3 — Journal close** | After **first victory finish** | `Open Journal to see your victory paths.` | Combat payoff points at Journal (receipt already ships elsewhere) |

**Player-facing pass (seed 1, Merchant Ada):** Begin → hint 1 visible → Accept → hint 1 gone, hint 2 up → sail (hint 2 dismisses on success) → later first victory → hint 3 → Journal. Chart layout at 1280 unchanged.

---

## 2. Known non-goals (do not score as Fix A fails)

| Non-goal | Why |
|---|---|
| Harbour / Hire / Shipyard / Crew / Hunt as breadcrumb steps | Curiosity or other desks; Harbour stays peek-only |
| Deliver button or new Complete verb | Delivery stays sell-at-dest + Contracts Complete |
| Seed picker / Grain Road duel | Fix B tester note only; Hunt is combat breadcrumb |
| Pier `port_row` reflow or More menu | Pier T2 deferred; T3 rejected |
| Escape / undock / M1 | Separate docked-consistency package (behind Fix A) |
| Day's report / Forage helper | Separate queued Godot PRs |
| Art, plates, sim, goldens | Godot chart notice/log only |

---

## 3. Game Tester — soft → live flip (Trigger A)

**Today (pre-merge):** `first-goal-after-begin` + `next-verb-after-accept` = **soft** (scored, not product-fail). Hint 3 maps to existing hard-capable `journal-closure` (not in the soft pair).

**After Fix A PR merges** (`playtest-friction-harden.md` Trigger A):

| Criterion | Action |
|---|---|
| `first-goal-after-begin` | Soft → **hard** (maps hint 1) |
| `next-verb-after-accept` | Soft → **hard** (maps hint 2) |
| `contract-complete-path` | Note friction #3 softens via hint 2 copy; still no Deliver button |
| `journal-closure` | Already product-gate; confirm hint 3 dismiss path if not already covered |

**Dry-harness steps (still no GPU run until Mike clears):**  
1. Edit `playtest-criteria-draft.md` — drop soft-until-Fix-A on those two; cite Fix A PR sha/number.  
2. Update `playtest-portlight-v0.playtest.json` if soft wording remains.  
3. Refresh `playtest-harness-status.md`.  
4. Ping Coordinator: Fix A criteria hardened (still no run).

**Smoke / feel checks for the land PR (Builder + GT):** ≤1 active hint; after Accept+sail Accept hint gone; ASCII; no 1280 reflow; no Crew/Hunt/receipt scope creep.

---

## 4. Soft hold

Fix A remains **first** when quota opens; docked-consistency rebases after. Do not land from this seat.

✅
