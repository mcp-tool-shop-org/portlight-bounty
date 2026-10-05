# playtest/

OpenRouter is off. Seats stay dark. Do not `ai-playtest run` until the GPU is cleared for playtest or a RunPod seat is named. A scripted probe is not a product score.

- `_plumbing-fixture.playtest.json` — echo fixture. `ai-playtest check` was green on 2026-10-05.
- `portlight-godot.playtest.json` — rpc driver, headless Godot, the locked criteria from the v0 list (15 core + 2 tip, plus the soft pair and the known-fail trio inside that 17, plus `contract-strip-glance` still ARMED). Choice ids other than the six tip rows are a local scheme.
- `probe-bridge.mjs` — no model. Hello, Title, captains, the name Ada, Merchant, then Hire on the docked chart.

Check the Godot config (no seat is started):

```
node ../ai-playtest/dist/cli.js check playtest/portlight-godot.playtest.json
```

Run that from the repo root after `ai-playtest` has been built. The probe, also from the repo root:

```
node playtest/probe-bridge.mjs
```

Reports name the editor. This machine's `godot` is 4.7 stable, not the CI 4.7.2 pin.
