# playtest/

OpenRouter is off. Seats stay dark. Do not `ai-playtest run` until the GPU is cleared for playtest or a RunPod seat is named. A scripted probe is not a product score.

- `_plumbing-fixture.playtest.json` — echo fixture. `ai-playtest check` was green on 2026-10-05.
- `portlight-godot.playtest.json` — rpc driver, headless Godot, the locked criteria from the v0 list (15 core + 2 tip, plus the soft pair and the known-fail trio inside that 17, plus `contract-strip-glance` still ARMED). Choice ids other than the six tip rows are a local scheme.
- `probe-bridge.mjs` — no model. Hello, a second client refused, a bad line answered, Title, captains, the name Ada, Merchant (Ada docked, Hire offered). Then Contracts open and close, the board cards read, the cheapest run the market stocks accepted (seed 1: grain x23 to Corsair's Rest), the order bought with a margin, Hire, a sail to the destination while the strip counts down, the Arrival card read from `state.day_report` on the docking day before Escape dismisses it, and the sale there. The sale is the delivery: the contract leaves Active, silver goes up, and the strip clears. Complete is not offered (R10), so pressing it is the offered-check. The strip's own id `chart.contract_strip` opens Contracts. Each lane's offered label reads `Sail - {Port}`. If the port has a shipyard, `line` types `Gull` into the rename field and `shipyard.rename` is offered only after that. Journal open (read from `state.journal`) and close, then reset to Title. Under an encounter, the probe asserts no `chart.*` or `pier.*` id is offered. Under an open desk, it records other desks' ids as information only.

## Observation

`state` has `screen`, `captain`, `day`, `silver`, `docked`, `place`, `open`, `hire_on_screen`, plus three contract fields, the Day's report and the Journal, read from what is drawn:

- `contract_strip` — the chart strip, segments joined with ` | `. Empty when the strip is hidden.
- `contract_board` — Contracts board cards, `{id, title, detail, meta, actions}`. Empty unless the Contracts desk is open.
- `contracts_active` — Contracts active rows, same shape; `actions` holds the row's Complete and Abandon ids. Empty unless the desk is open.
- `day_report` — the Day's report card, `{title, eyebrow, sections: [{id, title, lines}], footer}`. `footer` holds the Captain's week rows when they show. `null` when no Day's report is showing.
- `journal` — the Journal, `{title, notice, sections: [{id, title, lines}]}`. A section's `lines` are its labels and beat buttons in draw order. `null` unless the Journal is open.

`text` carries the same facts before `Actions:`: a `Contract strip:` line when the strip shows, `Board:` / `Active:` lists while the desk is open, and `Day's report: {title}` with each section's `{Title}:` / `- {line}` (then `Footer:`) while the card shows, and `Journal: {title}` with each section while the Journal is open.

## Offered choices

- **One blocking overlay.** While an encounter, Hunt, Crew, Contracts, Shipyard, Harbour, Journal, or a new-game page is open, `actions.options` lists only that overlay's buttons. The encounter comes first. Departure check and Day's report are non-blocking, so they do not narrow the list. A `choose` for anything else answers `not on screen`.
- **Labels.** `label` is the button text, except `chart.sail.<port_id>`, which reads `Sail - {Port}`. The button on screen still says `Sail`. `chart.contract_strip` (label `Contract strip`) is the strip on the chart. It opens Contracts when docked and does nothing at sea.
- **Insurance ids.** Empty segments are dropped. The forms are `harbour.prepare.insurance.{policy}`, `.{policy}.{target}`, or `.{policy}.{origin}.{destination}`. Any other mix keeps all four segments. For one release, the old form with trailing empty segments still parses.
- **`line`.** Types into the text field that is showing: the Shipyard rename field while the Shipyard is open, else the new-game name.

Check the Godot config (no seat is started):

```
node ../ai-playtest/dist/cli.js check playtest/portlight-godot.playtest.json
```

Run that from the repo root after `ai-playtest` has been built. The probe, also from the repo root:

```
cargo build -p portlight-godot
godot --headless --path godot --import --quit
node playtest/probe-bridge.mjs
```

Reports name the editor. CI pins Godot 4.7.2.

The consult for the next harden pass is `docs/playtest/playtest-bridge-consult-brief.md`.
