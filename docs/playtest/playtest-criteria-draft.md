# Playtest criteria — first voyage (Phase 0)

**Status: LOCKED** (Game Designer PASS 2026-10-05 — rename `goal-stated` → `first-goal-after-begin` applied)  
Seat: Game Tester · Consumer: portlight-bounty · Date: 2026-10-05  
Config: `playtest-portlight-v0.playtest.json`  
Sources: `playtest-first-voyage.md` §2–3 feel checklist, `design-call-docked-consistency.md` (APPROVED, PR not merged), Practitioner `playtest-escape-undock-criteria.md` + `design-note-escape-contracts-undock.md` (Coord confirmed fold), tip `cc2c50e4` (Day's report #46 + Hire polish #47; Hunt #35 + Crew #36 LIVE)

Stable ids feed `ai-playtest score` later; renaming after Phase 1 breaks diffs. Core set of **15 ids is LOCKED** — do not rename without a fresh GD co-sign. Tip-alignment notes and tip product-gate ids below do **not** rename LOCKED ids. **15 LOCKED core + 2 tip product-gate live** (17 product-gate ids) **+ 1 ARMED** (`contract-strip-glance` — product-gate on #48 merge; not live 18 until merge).

---

## Tip alignment (`cc2c50e4`)

**Spine note (product tip):** tip `cc2c50e4` includes Day's report (#46), Hire label polish (#47: pier shows **`Hire`**, Action still `HireSailor`), Hunt LIVE (#35), Crew LIVE (#36).

| Tip surface | Playtest stance |
|-------------|-----------------|
| Day's report | Auto panel after Next day when notable; Close = `CloseDayReport`; Escape closes; other docked overlays close it; quiet days stay hidden. See product-gate `day-report-clarity`. |
| Hire | Pier label **`Hire`** (not "Hire sailor"). See product-gate `hire-findable`. |
| Hunt | Preferred **diegetic** encounter path on seed 1 (forage + bounty → Hunt target → encounter). Fix B / `encounter_seed4` remains a valid alt. |
| Crew | LIVE desk (`OpenCrew` / `CloseCrew`); curiosity for first-voyage spine; full docked-overlay member when open. |

**Known gaps (explicit — do not flip):**
- Fix A soft pair still soft-until-Fix-A: `first-goal-after-begin`, `next-verb-after-accept`
- Docked-consistency trio still known-fail / pending-design-PR: `escape-closes-overlay`, `contracts-close-on-undock`, `one-docked-overlay` — membership expanded to tip `cc2c50e4` per Practitioner `playtest-escape-undock-criteria.md` (Hunt·Crew·Day's report on Escape+M1; Contracts+Crew force-close on undock; Journal/Hunt/Day's report sea-capable). Stay known-fail until docked-consistency PR. Fix A first if one quota slot.

---

## Voyage step → criteria map

First-voyage spine (LIVE on tip ≥`cc2c50e4`):  
New game → Contracts → Harbour peek → Sail Al-Manar → Sell (the sale pays the contract) → Encounter (Hunt preferred on seed 1; Fix B alt) → Victory finish → Journal  
(+ Day's report auto-show on notable Next day; Hire optional curiosity)

| Voyage step | Criteria ids |
|-------------|--------------|
| New game → Begin | `identity-clear`, `first-goal-after-begin` |
| Contracts open / Accept | `contracts-findable`, `next-verb-after-accept` |
| Harbour peek | (curiosity only; no dedicated criterion) |
| Hire (optional) | `hire-findable` (product-gate tip; optional curiosity) |
| Sail / Next day → Al-Manar | `maiden-sail-progress`, `world-moves`, `responds-to-input` |
| Day's report (notable Next day) | `day-report-clarity` (product-gate tip) |
| Contract strip glance (PR #48) | `contract-strip-glance` (**ARMED** — product-gate on #48 merge; not live until merge) |
| Sell at destination (the sale pays the contract) | `contract-complete-path` |
| Encounter (Hunt preferred; Fix B alt) | `encounter-interrupt` |
| Victory finish (#41) | `victory-receipt-payoff` |
| Journal | `journal-closure` |
| Cross-cutting UX | `never-stuck`, `responds-to-input`, `world-moves` |
| Docked consistency (pending PR) | `escape-closes-overlay`, `contracts-close-on-undock`, `one-docked-overlay` |

---

## Soft-until-Fix-A policy

`first-goal-after-begin` and `next-verb-after-accept` are **scored and reported every run** but do **not** count as a product fail until the Fix A chart-hints PR merges. Once Fix A merges they become hard. **Still parked — keep soft.**

| Fix A hint | Criterion |
|------------|-----------|
| Hint 1 (after Begin) | `first-goal-after-begin` |
| Hint 2 (after Accept) | `next-verb-after-accept` |
| Hint 3 (victory → Journal) | `journal-closure` |

---

## Criteria (stable ids) — LOCKED core 15

### `responds-to-input`
- **Intent:** Game reacts to the player's action, not stuck repeating.
- **Pass:** Specific input produces a visible state or log change distinct from the prior screen.
- **Fail signals:** Same text loop; input ignored; no acknowledgement across turns.
- **Notes:** Scorable in fixture and live seats alike.
- **Phase:** plumbing

### `world-moves`
- **Intent:** World/state visibly advances across turns (day, dock, silver, log).
- **Pass:** At least one of day, docked port, silver, cargo, contract progress, or journal/log advances during the session.
- **Fail signals:** Static HUD and log for the whole run despite actions.
- **Notes:** First-voyage happy path should move day and dock on maiden sail.
- **Phase:** plumbing

### `never-stuck`
- **Intent:** No soft-lock; overlays close; chart reachable; Escape/Close when UI expects.
- **Pass:** Player can always return to chart or dismiss overlay without restart.
- **Fail signals:** Overlay with no Close; Escape dead when dismiss expected; unreachable chart.
- **Notes:** Overlaps docked-consistency C1; keep as cross-cutting soft-lock gate. Day's report never traps the loop: dismissible in one action; Sail/Next day immediately usable after dismiss.
- **Phase:** scripted

### `first-goal-after-begin`
- **Intent:** After Begin only, first chart makes one starting verb discernible (Fix A hint 1).
- **Pass:** After Begin, UI, notice, or log makes one starting verb clear: open Contracts, buy at Market, or sail.
- **Fail signals:** Pier density with no breadcrumb; player must already know the loop.
- **Notes:** Soft until Fix A chart-hints PR merges — scored every run but not a product-fail until then; then hard. Scope is Begin only (Accept covered by `next-verb-after-accept`). **KNOWN GAP — still soft.**
- **Phase:** live-gpu (soft until Fix A)

### `identity-clear`
- **Intent:** New game → docked identity + victory paths visible in under 30s.
- **Pass:** After Begin, chart shows docked port, day, silver, ship, and victory-path names within 30 seconds.
- **Fail signals:** Blank or delayed identity; victory paths missing from first chart.
- **Notes:** Feel checklist item 1; seed 1 Merchant Ada default.
- **Phase:** scripted

### `contracts-findable`
- **Intent:** Contracts on the pier without hunting forever.
- **Pass:** From docked pier, Contracts opens in one obvious click on `port_row`.
- **Fail signals:** Hidden behind unrelated desks; no pier entry; player wanders >~30s.
- **Notes:** Feel checklist item 2 (Contracts half); Harbour findability is curiosity, not a fail here.
- **Phase:** scripted

### `next-verb-after-accept`
- **Intent:** After Accept, next step obvious (sail / sell there - the sale pays the contract) — Fix A hint 2.
- **Pass:** Notice, log, or UI makes load → sail to dest → sell clear; the sale pays the contract.
- **Fail signals:** Accept succeeds but pier is silent; player hunts for a Deliver button with no cue.
- **Notes:** Soft until Fix A chart-hints PR merges — scored every run but not a product-fail until then; then hard. **KNOWN GAP — still soft.**
- **Phase:** live-gpu (soft until Fix A)

### `maiden-sail-progress`
- **Intent:** Sail / Next day to Al-Manar shows depart / progress / Docked.
- **Pass:** Depart line or equivalent, voyage progress across Next day, then Docked at Al-Manar.
- **Fail signals:** Sail no-ops; stuck at sea with no progress; dock never registers.
- **Notes:** Lane Sail or chart arm+second click both valid; blocked-lane Sail clickable is a soft UX note, not this fail. Notable Next day may auto-show Day's report (never traps the loop: dismissible in one action; Sail/Next day usable after dismiss).
- **Phase:** scripted

### `contract-complete-path`
- **Intent:** Selling the contract good at the destination settles it.
- **Pass:** Selling the contract good at the destination settles it: it leaves Active and silver rises; Complete is not needed.
- **Fail signals:** Contract goods sold at the destination but the contract stays in Active, or silver does not rise.
- **Notes:** Delivery is sell-at-dest by design; no separate Deliver button expected.
- **Phase:** scripted

### `encounter-interrupt`
- **Intent:** Encounter overlay interrupt feels intentional.
- **Pass:** Overlay opens with clear choices; resolve shows signed delta (#40); Spare/Take all reachable.
- **Fail signals:** Confusing entry; no Leave when expected; choices skipped by stray Escape (post-PR).
- **Notes:** **Preferred product path on tip `cc2c50e4`:** Hunt target on seed 1 (forage + bounty → Hunt → encounter). **Fix B alt:** seed 4 `--encounter-screen` / playtest-host `encounter_seed4` remains valid. Grain Road seed 1 may skip combat — **not a fail**. Do not change catalog seed 1.
- **Phase:** scripted (Hunt preferred; Fix B alt)

### `victory-receipt-payoff`
- **Intent:** #41 receipt feels like a reward, not a log dump.
- **Pass:** After Spare/Take all, receipt lines (silver, loot/untouched, standing, companion flavor) read as a payoff beat.
- **Fail signals:** Silent finish; lines blur into ordinary log with no reward read.
- **Notes:** Tip ≥`97a1c314`; qualitative until seats can judge tone.
- **Phase:** live-gpu

### `journal-closure`
- **Intent:** Journal victory section readable after a beat (covers Fix A hint 3: victory → Journal).
- **Pass:** After ≥1 voyage beat, Journal (top row) opens; Victory section shows requirements vs status.
- **Fail signals:** Empty forever after beats; Journal unreachable; victory section blank when paths exist.
- **Notes:** Feel checklist item 8; Journal stays usable at sea (intentional). Fix A hint 3 lands here.
- **Phase:** scripted

### `escape-closes-overlay`
- **Intent:** C1 — Escape/ui_cancel = Close (confirm-first; encounter Escape only when Leave visible).
- **Pass:** Escape matches Close on each member overlay: Contracts · Shipyard · Harbour · Journal · **Crew** · **Hunt** · **Day's report**; first Escape cancels confirm only (abandon / harbour / shipyard / Hunt claim); encounter Escape inert until Leave/Back visible (then Escape = Leave).
- **Fail signals:** Escape no-ops on open desk; Escape confirms a buy/abandon/claim; Escape skips naval/Spare choice; Day's report Escape works while other desks ignore Escape.
- **Notes:** Design call APPROVED; membership per Practitioner `playtest-escape-undock-criteria.md` + `design-note-escape-contracts-undock.md` (tip `cc2c50e4`). **known-fail / pending until docked-consistency PR merges** — do **not** flip live. Tip today: Day's report closes on Escape; other desks Close-button only. **KNOWN GAP.**
- **Phase:** pending-design-PR

### `contracts-close-on-undock`
- **Intent:** C2 — pier docked-only desks cannot remain open at sea.
- **Pass:** Sail or Next day away from port with Contracts left open → overlay gone / `contracts_open` false; same for **Crew** (docked-only; GD amend — Crew clears on undock). **Journal** and **Hunt** remain sea-capable (intentional — opening at sea is not a fail). Day's report may stay across undock (sea-capable per GD) — not a C2 fail. Shipyard / Harbour undock-hide = regression-watch.
- **Fail signals:** Contracts root still visible / `contracts_open` true while undocked; Crew overlay stale at sea / next pier.
- **Notes:** Design call APPROVED; membership per Practitioner `playtest-escape-undock-criteria.md` + `design-note-escape-contracts-undock.md`. Land-PR smoke **must** sail with Contracts (and Crew) left open and assert overlay hidden (required GD). **known-fail / pending until docked-consistency PR merges** — do **not** flip live. **KNOWN GAP.**
- **Phase:** pending-design-PR

### `one-docked-overlay`
- **Intent:** M1 — one ink desk at a time; encounter wins.
- **Pass:** Mutual-exclusion set (tip `cc2c50e4`+): Contracts · Shipyard · Harbour · Journal · **Crew** · **Hunt** · **Day's report**. Opening any member clears other member flags; opening encounter clears all members. **Market** = in-panel toggle — **not** in set.
- **Fail signals:** Two ink desks visible; docked desk under encounter.
- **Notes:** Design call APPROVED; membership per Practitioner `playtest-escape-undock-criteria.md` + `design-note-escape-contracts-undock.md` (drop old "other three" wording — set is six desks + Day's report). **known-fail / pending until docked-consistency PR merges** — do **not** flip live. Tip today: opens set own flag only; Day's report yields to peers but desks can still stack. **KNOWN GAP.**
- **Phase:** pending-design-PR

---

## Tip product-gate criteria (GD PASS WITH EDITS 2026-10-05; #48 ARMED)

**15 LOCKED core + these 2 tip product-gate live** = **17 product-gate ids** total. **+ 1 ARMED** (`contract-strip-glance` — product-gate on #48 merge; **not** counting as live 18 until merge). Co-sign live pair: `design-cosign-playtest-drafts-cc2c50e4.md`. Armed id GD PASS (no edits): `design-signoff-pr48-contract-strip.md`. Ids stay as named (no renames). Soft holds unchanged (Fix A soft; docked known-fail). Tip pin remains `cc2c50e4` until #48 merges.

### `day-report-clarity` — product-gate
- **Intent:** After Next day with notable content, Day's report card is readable and dismissible without trapping the voyage loop.
- **Pass:** After a notable Next day, the card shows only what has content, among: up to 3 deadlines with copy saying "days left" (never "due soon"); captain health (captain only); prices for the **docked port only**; a claimable bounty once and again on arrival days. Closes in one action (Close or Escape); any other docked desk opening closes it. **Never traps the loop:** dismissible in one action; after dismiss, Sail and Next day are immediately usable (covering the chart while open is by design). Quiet day = no auto panel; for health, only `Healed` and the 1-day heal milestone count as notable.
- **Fail signals:** Notable day with no readable card; Close dead; remote-port prices shown; crew health lines shown; "due soon" wording; more than 3 deadlines listed; quiet day forces a panel.
- **Notes:** Tip ≥`cc2c50e4` (#46). Aligns `design-brief-day-report.md` §13. Smoke: `--day-report-screen`; frames day-report-full / day-report-deadline. **GD PASS WITH EDITS 2026-10-05** citing `design-cosign-playtest-drafts-cc2c50e4.md` — now product-gate.
- **Phase:** live (scripted / rpc when bridge exists)

### `hire-findable` — product-gate
- **Intent:** Docked pier shows Hire; one-shot hire visibly works (or shows why not).
- **Pass:** Docked pier `port_row` shows `Hire`; one click = single `HireSailor` with visible crew count or gold change; when unaffordable → visible reason, never a silent no-op.
- **Fail signals:** Label still "Hire sailor"; Hire missing from pier; hire silent no-op when affordable; unaffordable click with no visible reason.
- **Notes:** Tip ≥`cc2c50e4` (#47 polish). Optional curiosity path — not first-voyage happy-path ordering. If pier-density T2 folds Hire into Crew (`design-call-pier-density.md`), rewrite id to "Hire findable in Crew" with a fresh GD co-sign. **GD PASS WITH EDITS 2026-10-05** citing `design-cosign-playtest-drafts-cc2c50e4.md` — now product-gate. Provisions clause dropped (out of scope).
- **Phase:** live (optional curiosity; scored every run)


### `contract-strip-glance` — **ARMED — product-gate on #48 merge**
- **Intent:** Player can glance active contract progress + time left on the chart without opening Contracts.
- **Pass:** Hidden when no active contracts (fresh new game — no empty placeholder). After Accept → strip visible on chart; ASCII ` - ` separators; never `due soon` (reuse Day's report `deadline_timing`). Cap **2** segments + `+N more` (MUTED); nearest deadline first. **Urgency colour:** DUE/warning for due today / 1 day left / overdue; else GOLD. Single-segment may show destination `- to {Port}` when it fits; multi-segment drops dest. **Docked click** → opens Contracts (closes Day's report if open — one-overlay hygiene). **At sea:** strip stays visible; **click is no-op**. **`chart-1280.png` unchanged** (byte-identical) — new-game has no actives so strip hidden. Does not wrap `port_row`, shift lanes/market/log, or host/clear Fix A hints.
- **Fail signals:** Empty placeholder with zero actives; `due soon` wording; remote/broker price leakage; click at sea opens Contracts; docked click does nothing when actives exist; chart-1280 / port_row wrap / panel VBox shift when strip appears; strip replaces Day's report or Fix A notice/log.
- **Notes:** **ARMED — product-gate on #48 merge** (PR still OPEN; tip pin `cc2c50e4` until merge). **Not** live product-gate / not the 18th until merge. **GD PASS (no edits)** citing `design-signoff-pr48-contract-strip.md` ("PASS (product-gate once #48 merges)"); stub `playtest-contract-strip-criteria-STUB.md` matches. Softens friction #3 glance only — does **not** replace Fix A breadcrumbs or Day's report. Smoke: `--contract-strip-screen`; frames contract-strip-active / urgent.
- **Phase:** armed-pending-PR #48 (product-gate on merge)

---

## Out of scope / not criteria

Do **not** add further ids for these until their briefs expand or GD expands scope:

- **Hunt deep tour** — board refresh / claim UX beyond the preferred encounter breadcrumb; encounter uses Hunt preferred + Fix B alt
- **Crew deep tour** — roster / train / recruit not on first-voyage spine (desk LIVE; curiosity only)
- **Seed picker** — catalog seed stays 1; no product seed UI from this playtest
- **Harbour finance curriculum** — peek-and-close only; not minute-one teaching
- **Characters / man-of-war** — HELD; do not invent playtest that requires them
- **Shipyard deep tour** — LIVE but optional curiosity, not spine
- **Strength format (N1)** — deferred in design call

---

## Co-sign — Game Designer (2026-10-05)

**PASS with one id edit.** 15 ids stay 15. Applied and **LOCKED**.

**Edit applied: `goal-stated` → `first-goal-after-begin`.**
- Reason: as drafted, `goal-stated` covered both Begin and Accept, which double-counted `next-verb-after-accept`. Split cleanly so each maps to one Fix A milestone.
- New scope: after Begin only. Pass when the first chart makes one starting verb discernible (open Contracts, buy at Market, or sail) from UI, notice, or log.
- Map table: New game → Begin = `identity-clear`, `first-goal-after-begin`. Contracts open / Accept = `contracts-findable`, `next-verb-after-accept` only.

**Locked as-is (14):** `responds-to-input`, `world-moves`, `never-stuck`, `identity-clear`, `contracts-findable`, `next-verb-after-accept`, `maiden-sail-progress`, `contract-complete-path`, `encounter-interrupt`, `victory-receipt-payoff`, `journal-closure`, `escape-closes-overlay`, `contracts-close-on-undock`, `one-docked-overlay`.

**Soft-until-Fix-A:** confirmed. `first-goal-after-begin` + `next-verb-after-accept` scored every run, not product-fail until Fix A merges; then hard. Hint 3 → `journal-closure`.

**Pending-design-PR:** confirmed keep. Docked-consistency trio stays known-fail for Phase 1 before-state.

**Out-of-scope list:** agreed.

**Game Designer: PASS → criteria LOCKED 2026-10-05**

### Tip harden note (Game Tester 2026-10-05, tip `cc2c50e4`)

LOCKED core 15 unchanged (no renames). Tip-alignment section + DRAFT optional ids `day-report-clarity` / `hire-findable` added for GD co-sign. `encounter-interrupt` notes updated (Hunt preferred). Docked trio pass/membership expanded per Practitioner `playtest-escape-undock-criteria.md` (Coord confirmed) but stay **known-fail**. Soft Fix A pair stays soft.

### Tip DRAFT co-sign (Game Tester 2026-10-05, tip `cc2c50e4`)

GD **PASS WITH EDITS** on `day-report-clarity` + `hire-findable` (`design-cosign-playtest-drafts-cc2c50e4.md`). Both moved out of DRAFT into tip product-gate; edits applied; no renames. **15 LOCKED core + 2 tip product-gate live (17).** Soft Fix A pair stays soft; docked trio stays known-fail.

### Contract-strip ARMED (Game Tester 2026-10-05, tip `cc2c50e4`, PR #48 OPEN)

GD **PASS (no edits)** on `contract-strip-glance` (`design-signoff-pr48-contract-strip.md` — "PASS (product-gate once #48 merges)"). Folded under tip product-gate as **ARMED — product-gate on #48 merge**; **not** live 18th until merge. Soft Fix A / docked known-fail / no model run unchanged.
