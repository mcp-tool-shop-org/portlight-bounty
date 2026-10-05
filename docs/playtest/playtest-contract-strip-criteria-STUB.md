# Playtest criteria — contract strip (PR #48)

**Status: ARMED awaiting #48 merge** (GD PASS no edits)  
Seat: Game Tester · Date: 2026-10-05 · Tip baseline: `cc2c50e4` (pin until merge)  
PR: [#48](https://github.com/mcp-tool-shop-org/portlight-bounty/pull/48) still OPEN (`mergedAt` null) · Chart HUD contract deadline/progress strip  
Brief: `design-brief-contract-strip.md` (APPROVED)  
**GD cosign:** `design-signoff-pr48-contract-strip.md` — "PASS (product-gate once #48 merges)"; stub matches; no edits required.  
**Folded:** id `contract-strip-glance` in `playtest-criteria-draft.md` + `playtest-portlight-v0.playtest.json` as **ARMED — product-gate on #48 merge** (17 live product-gate; **not** live 18 until merge).  
**Soft:** seats dark; no model run until Mike GPU clear. OpenRouter off.

Do **not** count as live product-gate or product-fail until #48 merges.

---

## Id (GD co-signed — no rename)

### `contract-strip-glance`

| | |
|---|---|
| **Intent** | Player can glance active contract progress + time left on the chart without opening Contracts. |
| **Spine?** | Softens friction #3 (sell-vs-Complete / next verb) as persistent glance; does **not** replace Fix A breadcrumbs or Day's report. |
| **Phase now** | **ARMED** awaiting #48 merge (tip pin `cc2c50e4`) |
| **Phase after merge** | product-gate tip (18th live id) |

### Pass (armed; product-gate on #48 merge)

1. **Hidden** when no active contracts (fresh new game — no empty placeholder).
2. After Accept → strip **visible** on chart; ASCII ` - ` separators; never `due soon` (reuse Day's report `deadline_timing`).
3. Cap **2** segments + `+N more`; nearest deadline first.
4. **Urgency colour:** DUE/warning for due today / 1 day left / overdue; else GOLD; `+N more` MUTED (existing tokens only).
5. Single-segment may show destination `- to {Port}` when it fits; multi-segment drops dest.
6. **Docked click** → opens Contracts (and closes Day's report if open — one-overlay hygiene).
7. **At sea:** strip stays visible; **click is no-op** (does not open Contracts).
8. **`chart-1280.png` unchanged** (byte-identical) — new-game has no actives so strip hidden.
9. Does not wrap `port_row`, shift lanes/market/log, or host/clear Fix A hints.

### Fail signals

- Empty placeholder with zero actives  
- `due soon` wording  
- Remote/broker price leakage (strip is contracts only)  
- Click at sea opens Contracts  
- Docked click does nothing when actives exist  
- chart-1280 / port_row wrap / panel VBox shift when strip appears  
- Strip replaces Day's report or Fix A notice/log  

### Smoke / evidence (#48)

- `--contract-strip-screen` → `portlight contract-strip smoke ok`  
- Frames: `contract-strip-active.png`, `contract-strip-urgent.png`  
- Unit: sort/cap/copy/empty-hide; Complete short cue unit-only (Session auto-settles)

---

## Related LOCKED / tip ids (no rename)

| Id | Interaction with strip |
|----|------------------------|
| `next-verb-after-accept` | Strip helps glance progress; Fix A hint 2 still owns “named next verb” until/after soft→hard |
| `contract-complete-path` | Strip may show progress toward Complete; delivery still sell-at-dest |
| `day-report-clarity` | Separate surface — daily delta vs persistent glance; do not merge criteria |
| `one-docked-overlay` | Docked strip click → Contracts; closes Day's report peers per #48 smoke |
| `contracts-findable` | Strip is not a pier door; findability of Contracts desk stays separate |

## Out of scope (do not score here)

Deliver button · Fix A implementation · Escape/undock · sim/goldens/plates · inventing Session fields

---

## Flip checklist (Game Tester — after #48 merges)

1. Record merge sha / PR number here.
2. Drop ARMED hedges in criteria draft + config check prefix ("ARMED pending PR #48 merge — not product-fail until merge:").
3. Bump live product-gate count **17 → 18**.
4. Rename this file drop `STUB` → `playtest-contract-strip-criteria.md`.
5. Refresh harness status; ping Coordinator. Still no `ai-playtest run` until Mike GPU clear.

## Related

- Brief: `design-brief-contract-strip.md`  
- GD signoff: `design-signoff-pr48-contract-strip.md`  
- PR: https://github.com/mcp-tool-shop-org/portlight-bounty/pull/48  
- Criteria draft: `playtest-criteria-draft.md`  
- Config: `playtest-portlight-v0.playtest.json`  
- Mike runbook: `playtest-mike-runbook.md`
