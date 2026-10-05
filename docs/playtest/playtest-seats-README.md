# Playtest seats — Portlight Phase 0 sidecar

Consumer: `portlight-bounty` · Config: `playtest-portlight-v0.playtest.json` · Date: 2026-10-05

## When seats may be filled

| Gate | Allowed? |
|------|----------|
| Phase 0 plumbing / `ai-playtest check` | Placeholder seat only — **do not `run`** |
| Mike clears local GPU | Fill `provider: ollama` + model tag Mike names |
| RunPod up (later) | Swap seat endpoint / model; same criteria ids |
| OpenRouter | **Forbidden forever** for this consumer |

Until a gate above lights seats, treat the `pending-local` ollama seat as documentation that the schema accepts a seat — not permission to call a model.

## How to swap provider / model without rewriting criteria

1. Edit only the `seats[]` entry (and optionally `persona` reminder text).
2. Leave `criteria[].id` and `criteria[].check` untouched — ids are the contract with Game Designer.
3. Keep `game` pointing at the fixture echo until a real Portlight stdio/rpc adapter exists:
   - Robot path: `E:\AI\ai-playtest\test\fixtures\echo-game.mjs`
   - Portlight CLI today is **batch `script`**, not an interactive REPL; interactive voyage → rpc/Godot later.
4. After a seat swap, re-run `ai-playtest check` on Robot before any `run`.

## Persona reminder (in config)

The config `persona` field states DO NOT RUN until Mike clears GPU or RunPod. Do not strip that wording when swapping models later — update it to name the cleared host instead.

## Related

- Criteria draft (GD co-sign): `playtest-criteria-draft.md`
- Harness status: `playtest-harness-status.md`
- Seat brief: `seat-brief-game-tester.md`
