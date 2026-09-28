#!/usr/bin/env python3
"""Build victory-path fixtures and write the golden the Rust test checks.

The inputs below are the cases. Python's compute_victory_progress is the
expected output. Regenerate with:

    PYTHONPATH=/path/to/portlight/src python3 tools/parity/victory_cases.py
"""

from __future__ import annotations

import argparse
import json
import os
import sys

from portlight.content.world import new_game
from portlight.engine.campaign import (
    CampaignState,
    SessionSnapshot,
    VictoryCompletion,
    compute_victory_progress,
)
from portlight.engine.captain_identity import CaptainType
from portlight.engine.contracts import ContractBoard, ContractFamily, ContractOutcome
from portlight.engine.infrastructure import (
    BrokerOffice,
    BrokerTier,
    CreditState,
    InfrastructureState,
    OwnedLicense,
    WarehouseLease,
    WarehouseTier,
)
from portlight.engine.models import ReputationIncident
from portlight.receipts.models import ReceiptLedger


ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.path.join(ROOT, "parity", "golden", "victory_cases.json")


CASES = [
    {"name": "fresh_merchant", "input": {}},
    {
        "name": "lawful_complete",
        "input": {
            "silver": 2500,
            "trust": 40,
            "regional": {"Mediterranean": 16, "North Atlantic": 15},
            "heat": {"Mediterranean": 2},
            "licenses": ["high_rep_charter", "regional_charter"],
            "contracts": [{"outcome_type": "completed", "family": "procurement"}] * 8,
        },
    },
    {
        "name": "lawful_heat_blocked",
        "input": {
            "silver": 2500,
            "trust": 40,
            "heat": {"West Africa": 9},
            "licenses": ["high_rep_charter", "regional_charter"],
            "contracts": [{"outcome_type": "completed", "family": "procurement"}] * 8,
            "credit": {"total_borrowed": 0, "defaults": 1},
            "incidents": ["Cargo seized during inspection (fined 20 silver)"],
        },
    },
    {
        "name": "shadow_under_heat",
        "input": {
            "silver": 1800,
            "heat": {"Mediterranean": 12, "East Indies": 4},
            "net_profit": 2500,
            "trade_count": 9,
            "contracts": [
                {
                    "outcome_type": "completed",
                    "family": "luxury_discreet",
                    "summary": "quiet delivery",
                },
                {
                    "outcome_type": "completed_bonus",
                    "family": None,
                    "summary": "A discreet luxury run",
                },
                {"outcome_type": "expired", "family": "luxury_discreet", "summary": "late"},
            ],
            "incidents": ["Cargo seized during inspection"],
        },
    },
    {
        "name": "shadow_clean_profit_blocked",
        "input": {"silver": 80, "net_profit": 1500, "heat": {"Mediterranean": 0}},
    },
    {
        "name": "oceanic_galleon",
        "input": {
            "silver": 2200,
            "ship_template": "merchant_galleon",
            "regional": {"East Indies": 18},
            "licenses": ["ei_access_charter"],
            "warehouses": ["jade_port"],
            "brokers": [{"region": "East Indies", "tier": "local"}],
            "contracts": [{"outcome_type": "completed", "family": "circuit"}] * 5,
        },
    },
    {
        "name": "commercial_empire_complete",
        "input": {
            "silver": 4000,
            "trust": 30,
            "day": 40,
            "licenses": ["high_rep_charter", "ei_access_charter", "regional_charter"],
            "warehouses": ["porto_novo", "sun_harbor"],
            "brokers": [{"region": "East Indies", "tier": "established"}],
            "policies": 1,
            "credit": {"total_borrowed": 500, "defaults": 0},
            "contracts": [{"outcome_type": "completed", "family": "procurement"}] * 10,
        },
    },
    {
        "name": "recorded_path_keeps_its_summary",
        "input": {
            "completed_paths": [
                {
                    "path_id": "commercial_empire",
                    "completion_day": 12,
                    "summary": "recorded earlier",
                    "is_first": True,
                }
            ]
        },
    },
    {
        "name": "inactive_broker_and_none_tier_do_not_count",
        "input": {
            "brokers": [
                {"region": "East Indies", "tier": "none"},
                {"region": "South Seas", "tier": "local", "active": False},
            ],
            "warehouses": [{"port_id": "jade_port", "active": False}],
        },
    },
]


def apply(spec: dict):
    world = new_game("Ada", None, CaptainType.MERCHANT, seed=1)
    if "silver" in spec:
        world.captain.silver = spec["silver"]
    if "trust" in spec:
        world.captain.standing.commercial_trust = spec["trust"]
    if "day" in spec:
        world.day = spec["day"]
    for region, value in spec.get("regional", {}).items():
        world.captain.standing.regional_standing[region] = value
    for region, value in spec.get("heat", {}).items():
        world.captain.standing.customs_heat[region] = value
    if "ship_template" in spec and world.captain.ship is not None:
        world.captain.ship.template_id = spec["ship_template"]
    for description in spec.get("incidents", []):
        world.captain.standing.recent_incidents.insert(
            0,
            ReputationIncident(
                day=1,
                port_id="porto_novo",
                region="Mediterranean",
                incident_type="inspection",
                description=description,
            ),
        )
    board = ContractBoard()
    for index, contract in enumerate(spec.get("contracts", [])):
        family = contract.get("family")
        board.completed.append(
            ContractOutcome(
                contract_id=f"c{index}",
                outcome_type=contract["outcome_type"],
                silver_delta=0,
                trust_delta=0,
                standing_delta=0,
                heat_delta=0,
                completion_day=1,
                summary=contract.get("summary", ""),
                family=None if family is None else ContractFamily(family),
            )
        )
    infra = InfrastructureState()
    for port_id in spec.get("warehouses", []):
        if isinstance(port_id, str):
            active = True
            pid = port_id
        else:
            active = port_id.get("active", True)
            pid = port_id["port_id"]
        infra.warehouses.append(
            WarehouseLease(
                id=f"wh-{pid}",
                port_id=pid,
                tier=WarehouseTier.DEPOT,
                capacity=10,
                lease_cost=1,
                upkeep_per_day=1,
                active=active,
            )
        )
    for broker in spec.get("brokers", []):
        infra.brokers.append(
            BrokerOffice(
                region=broker["region"],
                tier=BrokerTier(broker["tier"]),
                active=broker.get("active", True),
            )
        )
    for license_id in spec.get("licenses", []):
        infra.licenses.append(OwnedLicense(license_id=license_id, purchased_day=1, active=True))
    infra.policies = [object() for _ in range(spec.get("policies", 0))]
    if "credit" in spec:
        credit = CreditState()
        credit.total_borrowed = spec["credit"]["total_borrowed"]
        credit.defaults = spec["credit"]["defaults"]
        credit.active = True
        infra.credit = credit
    ledger = ReceiptLedger()
    ledger.net_profit = spec.get("net_profit", 0)
    ledger.receipts = [object() for _ in range(spec.get("trade_count", 0))]
    campaign = CampaignState()
    for record in spec.get("completed_paths", []):
        campaign.completed_paths.append(
            VictoryCompletion(
                path_id=record["path_id"],
                completion_day=record["completion_day"],
                summary=record["summary"],
                is_first=record.get("is_first", False),
            )
        )
    return SessionSnapshot(
        captain=world.captain,
        world=world,
        board=board,
        infra=infra,
        ledger=ledger,
        campaign=campaign,
    )


def view(paths) -> list:
    return [
        {
            "path_id": path.path_id,
            "name": path.name,
            "candidate_strength": path.candidate_strength,
            "completion_day": path.completion_day,
            "completion_summary": path.completion_summary,
            "requirements": [
                {
                    "description": req.description,
                    "status": req.status.value,
                    "detail": req.detail,
                    "action": req.action,
                }
                for req in path.requirements
            ],
        }
        for path in paths
    ]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--check",
        action="store_true",
        help="compare with the committed golden and do not write it",
    )
    args = parser.parse_args()
    cases = []
    for case in CASES:
        snap = apply(case["input"])
        cases.append(
            {
                "name": case["name"],
                "input": case["input"],
                "paths": view(compute_victory_progress(snap)),
            }
        )
    payload = json.dumps(cases, indent=2) + "\n"
    ids = {path["path_id"] for case in cases for path in case["paths"]}
    if "commercial_finance" in ids or "commercial_empire" not in ids:
        print("victory path ids drifted", ids, file=sys.stderr)
        return 1
    if args.check:
        with open(OUT, encoding="utf-8") as fh:
            existing = fh.read()
        if existing != payload:
            print(f"{OUT} drifted from the live Python evaluator", file=sys.stderr)
            return 1
        print(f"{OUT} matches Python ({len(cases)} cases)")
        return 0
    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    with open(OUT, "w", encoding="utf-8") as fh:
        fh.write(payload)
    print(f"wrote {OUT} ({len(cases)} cases)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
