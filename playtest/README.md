# playtest/

OpenRouter is off. Seats stay dark. Do not `ai-playtest run` until the GPU is cleared for playtest or a RunPod seat is named. A scripted probe is not a product score.

- `_plumbing-fixture.playtest.json` — echo fixture. `ai-playtest check` was green on 2026-10-05.
- `portlight-godot.playtest.json` — rpc driver, headless Godot, the locked criteria from the v0 list (15 core + 2 tip, plus the soft pair and the known-fail trio inside that 17, plus `contract-strip-glance` still ARMED). Choice ids other than the six tip rows are a local scheme.
- `probe-bridge.mjs` — no model. Hello, a second client refused, a bad line answered, Title, captains, the name Ada, Merchant (Ada docked, Hire offered). Then Contracts open and close, the board cards read, the cheapest run the market stocks accepted (seed 1: grain x23 to Corsair's Rest), the order bought with a margin, Hire, a sail to the destination while the strip counts down, the Arrival card read from `state.day_report` on the docking day before Escape dismisses it, and the sale there. The sale is the delivery: the contract leaves Active, silver goes up, and the strip clears. Complete is not offered (R10), so pressing it is the offered-check. Journal open and close, then reset to Title.

## Observation

`state` has `screen`, `captain`, `day`, `silver`, `docked`, `place`, `open`, `hire_on_screen`, plus three contract fields and the Day's report, read from what is drawn:

- `contract_strip` — the chart strip, segments joined with ` | `. Empty when the strip is hidden.
- `contract_board` — Contracts board cards, `{id, title, detail, meta, actions}`. Empty unless the Contracts desk is open.
- `contracts_active` — Contracts active rows, same shape; `actions` holds the row's Complete and Abandon ids. Empty unless the desk is open.
- `day_report` — the Day's report card, `{title, eyebrow, sections: [{id, title, lines}], footer}`. `footer` holds the Captain's week rows when they show. `null` when no Day's report is showing.

`text` carries the same facts before `Actions:`: a `Contract strip:` line when the strip shows, `Board:` / `Active:` lists while the desk is open, and `Day's report: {title}` with each section's `{Title}:` / `- {line}` (then `Footer:`) while the card shows.

Check the Godot config (no seat is started):

```
node ../ai-playtest/dist/cli.js check playtest/portlight-godot.playtest.json
```

Run that from the repo root after `ai-playtest` has been built. The probe, also from the repo root:

```
node playtest/probe-bridge.mjs
```

Reports name the editor. CI pins Godot 4.7.2.

The consult for the next harden pass is `docs/playtest/playtest-bridge-consult-brief.md`.
