#!/usr/bin/env python3
"""Roster and custom-captain options, checked the way victory_cases.py is.

Python's `CAPTAIN_ORDER` and the catalogs `validate_spec` reads are the
expected lists. Regenerate with:

    PYTHONPATH=/path/to/portlight/src python3 tools/parity/roster_options.py

`--check` compares that output to `parity/golden/roster_options.json` and
does not write it. The Rust test reads the same file.
"""

from __future__ import annotations

import argparse
import json
import os
import sys

from portlight.content.factions import FACTIONS
from portlight.content.port_institutions import ALL_NPCS
from portlight.content.port_politics import TRADE_BLOCS
from portlight.content.ports import PORTS
from portlight.content.ships import SHIPS
from portlight.engine.captain_identity import CAPTAIN_ORDER, CAPTAIN_TEMPLATES
from portlight.engine.custom_captain import REGION_STARTING_SILVER
from portlight.engine.models import ReputationState


ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.path.join(ROOT, "parity", "golden", "roster_options.json")

# Counts the Rust catalog must match, in this order.
EXPECTED_COUNTS = {
    "captains": 9,
    "regions": 5,
    "ports": 20,
    "blocs": 7,
    "factions": 4,
    "mentors": 134,
}


def payload() -> dict:
    # Region order is the custom-captain builder's map. The standing model
    # uses the same insertion order; an unordered validate_spec set does not.
    regions = list(REGION_STARTING_SILVER)
    standing = list(ReputationState().regional_standing)
    if regions != standing:
        raise SystemExit(f"region order drifted: {regions} vs {standing}")
    captains = []
    for captain_type in CAPTAIN_ORDER:
        template = CAPTAIN_TEMPLATES[captain_type]
        port = PORTS[template.home_port_id]
        ship = SHIPS[template.starting_ship_id]
        captains.append(
            {
                "id": captain_type.value,
                "name": template.name,
                "title": template.title,
                "home_port_id": template.home_port_id,
                "home_port_name": port.name,
                "starting_ship_id": template.starting_ship_id,
                "starting_ship_class": ship.ship_class.value,
                "starting_silver": template.starting_silver,
            }
        )
    body = {
        "captains": captains,
        "regions": regions,
        "ports": [
            {"id": port_id, "name": port.name, "region": port.region}
            for port_id, port in PORTS.items()
        ],
        "blocs": [
            {"id": bloc_id, "name": bloc.name} for bloc_id, bloc in TRADE_BLOCS.items()
        ],
        "factions": [
            {"id": faction_id, "name": faction.name}
            for faction_id, faction in FACTIONS.items()
        ],
        "mentors": [
            {"id": npc_id, "name": npc.name, "port_id": npc.port_id}
            for npc_id, npc in ALL_NPCS.items()
        ],
    }
    for key, count in EXPECTED_COUNTS.items():
        if len(body[key]) != count:
            raise SystemExit(f"{key}: expected {count}, got {len(body[key])}")
    return body


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--check",
        action="store_true",
        help="compare with the committed golden and do not write it",
    )
    args = parser.parse_args()
    text = json.dumps(payload(), indent=2) + "\n"
    if args.check:
        with open(OUT, encoding="utf-8") as fh:
            existing = fh.read()
        if existing != text:
            print(f"{OUT} drifted from the live Python catalogs", file=sys.stderr)
            return 1
        print(
            f"{OUT} matches Python "
            f"({EXPECTED_COUNTS['captains']} captains, "
            f"{EXPECTED_COUNTS['regions']} regions, "
            f"{EXPECTED_COUNTS['ports']} ports, "
            f"{EXPECTED_COUNTS['blocs']} blocs, "
            f"{EXPECTED_COUNTS['factions']} factions, "
            f"{EXPECTED_COUNTS['mentors']} mentors)"
        )
        return 0
    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    with open(OUT, "w", encoding="utf-8") as fh:
        fh.write(text)
    print(f"wrote {OUT}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
