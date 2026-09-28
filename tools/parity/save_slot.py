#!/usr/bin/env python3
"""Load one Portlight save slot the way GameSession.load does, and print the snapshot.

Usage:
    PYTHONPATH=/path/to/portlight/src python3 tools/parity/save_slot.py <base_path> [slot]

`base_path` is the directory that contains `saves/`. The default slot is `default`.
The snapshot is the one `tools/parity/oracle.py` builds, after migration, receipt-count
trade_seq, and captain-modifier repricing.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

from portlight.content.goods import GOODS
from portlight.engine.captain_identity import CAPTAIN_TEMPLATES, CaptainType
from portlight.engine.economy import recalculate_prices
from portlight.engine.save import load_game

from oracle import snapshot


def pricing_for(captain_type: str):
    try:
        return CAPTAIN_TEMPLATES[CaptainType(captain_type)].pricing
    except (ValueError, KeyError):
        return CAPTAIN_TEMPLATES[CaptainType.MERCHANT].pricing


def main() -> int:
    if len(sys.argv) not in (2, 3):
        print("Usage: save_slot.py <base_path> [slot]", file=sys.stderr)
        return 2
    base = Path(sys.argv[1])
    slot = sys.argv[2] if len(sys.argv) == 3 else "default"
    loaded = load_game(base, slot=slot)
    if loaded is None:
        print(f"No save in slot {slot}", file=sys.stderr)
        return 1
    world, ledger, board, infra, campaign, _narrative = loaded
    pricing = pricing_for(world.captain.captain_type)
    for port in world.ports.values():
        recalculate_prices(port, GOODS, pricing)
    state = {
        "world": world,
        "trade_seq": len(ledger.receipts),
        "ledger": ledger,
        "board": board,
        "infra": infra,
        "campaign": campaign,
    }
    json.dump(snapshot(state, []), sys.stdout, indent=2)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
