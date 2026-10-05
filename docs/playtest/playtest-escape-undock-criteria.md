# PB-001 — Playtest flip: Escape / undock / one-overlay

Seat: **Practitioner** (draft) · For: **Game Tester** · Date: 2026-10-05  
Tip baseline: **`cc2c50e4`** (Day's report + Hunt + Crew + Hire polish LIVE)  
Trigger: flip **after docked-consistency PR merges** (not before)  
Ids: already LOCKED in `playtest-criteria-draft.md` — **expand membership only; do not add new ids**  
Status: **known-fail / pending-design-PR** until that merge · Soft: **Fix A still first** if one quota slot (`design-call-friction-route.md`)

GD tip amend (accepted on design note): **Crew clears on undock** (docked-only); Contracts-open-at-sail smoke harden is **required**. Folded here for Tester.

Sources: `design-note-escape-contracts-undock.md` §5; `design-call-docked-consistency.md` (APPROVED); `land-docked-consistency.md`; `playtest-friction-harden.md` Trigger B; `playtest-criteria-draft.md` (LOCKED trio).

---

## 1. Purpose

Hand Game Tester a concrete **before → after** checklist for the three docked-consistency criteria when the land PR merges. Tip `cc2c50e4` already has Hunt, Crew, and Day's report — membership below replaces the older "Contracts/Shipyard/Harbour/Journal only" wording in criteria pass lines.

**Do not** flip soft→hard or known-fail→live from this note alone. **No code. No GPU run** until Mike clears (OpenRouter forbidden).

---

## 2. Sequencing

| Order | Merge | Tester action |
|---|---|---|
| 1 | **Fix A** (chart breadcrumbs) — soft, still preferred first | Harden `first-goal-after-begin` + `next-verb-after-accept` per `playtest-friction-harden.md` Trigger A |
| 2 | **Docked-consistency** (rebased onto Fix A when possible) | Flip the **three ids below** known-fail → **live** |

If Coord lands docked-consistency before Fix A: still flip these three; note Fix A soft criteria remain soft until Fix A merges.

---

## 3. Criteria expansions (same ids)

### `escape-closes-overlay` — C1

| | |
|---|---|
| **Phase now** | `pending-design-PR` / **known-fail** on tip `cc2c50e4` |
| **Phase after land** | **live** (scripted or live-gpu when allowed) |
| **Intent** | Escape / `ui_cancel` = visible Close (confirm-first); encounter Escape only when Leave/Back visible |
| **Membership (tip `cc2c50e4`+)** | Contracts · Shipyard · Harbour · Journal · **Crew** · **Hunt** · **Day's report** · encounter Leave when Back visible |
| **Before (tip today)** | Day's report closes on Escape; other desks Close-button only → **fail** if scored hard |
| **Pass after land** | Escape closes each member overlay; pending confirm → first Escape cancels confirm only; mid-encounter Escape inert; Leave visible → Escape leaves |
| **Fail signals** | Escape no-ops on open desk; Escape confirms a buy/abandon/claim; Escape skips naval/Spare choice; Day's report still special-cased while desks ignore Escape |
| **Check steps** | (1) Open each desk → Escape. (2) Arm abandon / harbour confirm / Hunt claim confirm → Escape once (confirm gone, overlay stays) → Escape again (closes). (3) Next day → Day's report → Escape. (4) Encounter mid-resolve → Escape inert; Leave visible → Escape leaves. |
| **Harden note** | Update criteria Pass line to list Hunt/Crew/Day's report; cite docked-consistency PR sha. Land-PR smoke **must** assert Escape closes each desk including Hunt and Crew (GD required). |

### `contracts-close-on-undock` — C2

| | |
|---|---|
| **Phase now** | `pending-design-PR` / **known-fail** |
| **Phase after land** | **live** |
| **Intent** | Pier Contracts overlay cannot remain open at sea |
| **Membership** | Force-close on undock: **Contracts** + **Crew** (docked-only; GD amend 2026-10-05). **Journal** and **Hunt** stay openable at sea (intentional). Shipyard / Harbour already undock-hide — regression-watch only |
| **Before** | Leave Contracts open → Sail → overlay can stay (`sync_contracts` ignores `docked_id`) — smoke closes first so CI misses it |
| **Pass after land** | Sail or Next day away from port with Contracts left open → overlay gone; button/port_row hygiene unchanged |
| **Fail signals** | Contracts root still visible / `contracts_open` true while undocked |
| **Check steps** | (1) Docked → open Contracts → **do not Close** → Sail Al-Manar → Next day until at sea → assert overlay hidden + `contracts_open == false`. (2) Same with **Crew** left open → sail → Crew overlay gone / `crew_open` cleared (not stale at next pier). (3) Control: Journal still opens at sea. (4) Control: Hunt still opens at sea. (5) Day's report may stay across undock (sea-capable per GD) — not a C2 fail. |
| **Harden note** | Land-PR smoke **must** sail with Contracts (and Crew) left open and assert overlay hidden — required (GD), not optional. Current tip smoke closes first and only checks the button. |

### `one-docked-overlay` — M1

| | |
|---|---|
| **Phase now** | `pending-design-PR` / **known-fail** |
| **Phase after land** | **live** |
| **Intent** | One ink desk at a time; encounter wins |
| **Membership (mutual exclusion set)** | Contracts · Shipyard · Harbour · Journal · **Crew** · **Hunt** · **Day's report** (opening a desk clears peers). **Market** = in-panel toggle — **not** in set |
| **Before** | Opens set own flag only (Day's report is closed by peers today; desks can still stack) |
| **Pass after land** | Opening any member clears other member flags; opening encounter clears all members |
| **Fail signals** | Two ink desks visible; docked desk under encounter |
| **Check steps** | (1) Open Contracts → open Hunt → only Hunt visible. (2) Open Crew → open Journal → only Journal. (3) Open Harbour → start encounter (Hunt target or smoke) → desks cleared. (4) Market toggle does not force-close desks / is not required to clear on desk open |
| **Harden note** | Criteria Pass line must drop "other three" wording — set is six desks + Day's report |

---

## 4. Flip checklist (Game Tester — after merge)

When docked-consistency lands on `main`:

1. Record merge sha / PR number on this file and in `playtest-harness-status.md`.  
2. Edit `playtest-criteria-draft.md` — trio phase `pending-design-PR` → **live**; paste membership lines from §3; keep ids unchanged.  
3. Update `playtest-portlight-v0.playtest.json` (or successor) check text if it still says four desks only.  
4. Graduate optional Escape / `ui.close` in `playtest-rpc-choice-ids.md` to OK if that bridge tracks Escape.  
5. Ping Coordinator: docked-consistency criteria live (still no `ai-playtest run` unless Mike GPU cleared).  
6. Do **not** invent Deliver / seed picker / Harbour gating checks.

Mirror of `playtest-friction-harden.md` Trigger B — this file is the **membership expansion** Trigger B should apply on tip ≥`cc2c50e4`.

---

## 5. Out of scope

- New criterion ids (LOCKED count stands)  
- Fix A soft→hard (separate Trigger A)  
- N1 strength format  
- Forage≠fight / Claim-miss copy (fun notes, not this flip)  
- Flipping receipt-tone or blocked-Sail runners-up to Critical  

---

## 6. Cosign / handoff

| Seat | Ask |
|---|---|
| **Game Tester** | Arm Trigger B with §3 membership; keep known-fail until land |
| **Game Designer** | Cosign already on design note (Crew undock + required smoke). This file mirrors that for Tester. |
| **Coordinator** | Route; remind Fix A first if one quota slot |

Practitioner: no code · no harness edit from this seat · tip cites in `design-note-escape-contracts-undock.md`.

✅
