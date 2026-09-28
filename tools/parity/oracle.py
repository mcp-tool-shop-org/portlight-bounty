#!/usr/bin/env python3
"""Run a Portlight action script through the Python engine and print state JSON.

This is the reference side of the parity harness. It calls the same engine
functions the Rust script runner calls — not the full GameSession — so
contracts, sea culture, insurance, and duel auto-resolution stay out of the
comparison. See docs/PORTING-PLAN.md.

Usage:

    PYTHONPATH=/path/to/portlight/src python3 tools/parity/oracle.py parity/scripts/trade_in_port.txt
"""

from __future__ import annotations

import json
import random
import shlex
import sys

from portlight.content.goods import GOODS
from portlight.content.ships import SHIPS
from portlight.content.world import new_game
from portlight.engine.captain_identity import CAPTAIN_TEMPLATES, CaptainType
from portlight.engine.economy import (
    execute_buy,
    execute_sell,
    recalculate_prices,
    tick_markets,
)
from portlight.engine.models import VoyageStatus
from portlight.engine.reputation import (
    record_inspection_outcome,
    record_port_arrival,
    record_trade_outcome,
    tick_reputation,
)
from portlight.engine.ship_stats import compute_daily_wages
from portlight.engine.voyage import advance_day, arrive, depart
from portlight.receipts.models import TradeReceipt


REGIONS = [
    "Mediterranean",
    "North Atlantic",
    "West Africa",
    "East Indies",
    "South Seas",
]


class ScriptError(Exception):
    pass


def estimate_cost_basis(captain, good_id: str, qty: int) -> int:
    for item in captain.cargo:
        if item.good_id == good_id and item.quantity > 0:
            avg = item.cost_basis / item.quantity
            return int(avg * qty)
    good = GOODS.get(good_id)
    return good.base_price * qty if good else qty * 10


def current_port(world):
    if world.voyage.status == VoyageStatus.IN_PORT:
        return world.ports.get(world.voyage.destination_id)
    return None


def pricing_of(world):
    try:
        return CAPTAIN_TEMPLATES[CaptainType(world.captain.captain_type)].pricing
    except (ValueError, KeyError):
        return None


def reprice_port(world, port) -> None:
    recalculate_prices(port, GOODS, pricing_of(world))


def reprice_all(world) -> None:
    pricing = pricing_of(world)
    for port in world.ports.values():
        recalculate_prices(port, GOODS, pricing)


def receipt_view(receipt: TradeReceipt) -> dict:
    return {
        "receipt_id": receipt.receipt_id,
        "action": receipt.action.value,
        "good_id": receipt.good_id,
        "quantity": receipt.quantity,
        "unit_price": receipt.unit_price,
        "total_price": receipt.total_price,
        "stock_before": receipt.stock_before,
        "stock_after": receipt.stock_after,
        "day": receipt.day,
    }


def event_view(event) -> dict:
    lost = []
    if event.cargo_lost:
        for good_id, qty in event.cargo_lost.items():
            lost.append({"good_id": good_id, "quantity": qty})
    return {
        "event_type": event.event_type.value,
        "message": event.message,
        "hull_delta": event.hull_delta,
        "provision_delta": event.provision_delta,
        "silver_delta": event.silver_delta,
        "crew_delta": event.crew_delta,
        "speed_modifier": event.speed_modifier,
        "cargo_lost": lost,
        "flavor": event.flavor or "",
    }


def blank_entry(command: str) -> dict:
    return {
        "command": command,
        "error": None,
        "receipt": None,
        "events": [],
        "shocks": [],
    }


def do_buy(state, good_id: str, qty: int, entry: dict) -> None:
    world = state["world"]
    port = current_port(world)
    if port is None:
        raise ScriptError("Not docked at a port")
    result = execute_buy(world.captain, port, good_id, qty, GOODS, state["trade_seq"])
    if isinstance(result, str):
        raise ScriptError(result)
    state["trade_seq"] += 1
    reprice_port(world, port)
    entry["receipt"] = receipt_view(result)


def do_sell(state, good_id: str, qty: int, entry: dict) -> None:
    world = state["world"]
    port = current_port(world)
    if port is None:
        raise ScriptError("Not docked at a port")
    slot = next((s for s in port.market if s.good_id == good_id), None)
    flood_before = slot.flood_penalty if slot else 0.0
    stock_target = slot.stock_target if slot else 50
    result = execute_sell(world.captain, port, good_id, qty, state["trade_seq"])
    if isinstance(result, str):
        raise ScriptError(result)
    state["trade_seq"] += 1
    cost_basis = estimate_cost_basis(world.captain, good_id, result.quantity)
    if cost_basis > 0:
        margin_pct = ((result.total_price - cost_basis) / max(cost_basis, 1)) * 100
    else:
        margin_pct = 50.0
    good = GOODS.get(good_id)
    category = good.category if good else None
    from portlight.engine.models import GoodCategory

    if category is None:
        category = GoodCategory.COMMODITY
    record_trade_outcome(
        world.captain.standing,
        world.captain.captain_type,
        world.day,
        port.id,
        port.region,
        good_id,
        category,
        result.quantity,
        margin_pct,
        stock_target,
        flood_before,
        is_sell=True,
    )
    reprice_port(world, port)
    entry["receipt"] = receipt_view(result)


def do_advance(state, entry: dict) -> None:
    world = state["world"]
    rng = state["rng"]
    tick_reputation(world.captain.standing)
    if world.voyage.status != VoyageStatus.AT_SEA:
        shocks = tick_markets(world.ports, days=1, rng=rng)
        world.day += 1
        world.captain.day += 1
        if world.captain.provisions > 0:
            world.captain.provisions -= 1
        ship = world.captain.ship
        if ship is not None:
            template = SHIPS.get(ship.template_id)
            daily = template.daily_wage if template else 1
            if ship.roster.total > 0:
                wage = compute_daily_wages(ship.roster, daily)
            else:
                wage = daily * ship.crew
            if wage > 0 and world.captain.silver >= wage:
                world.captain.silver -= wage
        entry["shocks"] = shocks
    else:
        events = advance_day(world, rng)
        dest = world.ports.get(world.voyage.destination_id)
        region = dest.region if dest else "Mediterranean"
        for event in events:
            if event.event_type.value == "inspection":
                seized = event.cargo_lost is not None and len(event.cargo_lost) > 0
                record_inspection_outcome(
                    world.captain.standing,
                    world.day,
                    world.voyage.origin_id,
                    region,
                    abs(event.silver_delta),
                    seized,
                )
        if world.voyage.status == VoyageStatus.ARRIVED:
            arrive(world)
            port = world.ports.get(world.voyage.destination_id)
            if port is not None:
                record_port_arrival(world.captain.standing, world.day, port.id, port.region)
        entry["events"] = [event_view(event) for event in events]
    reprice_all(world)


def dispatch(state, tokens: list[str], entry: dict) -> None:
    cmd = tokens[0]
    if cmd == "new":
        if len(tokens) < 4:
            raise ScriptError("Usage: new <captain_type> <name> <seed> [port]")
        try:
            seed = int(tokens[3])
        except ValueError as exc:
            raise ScriptError(f"Invalid number: {tokens[3]}") from exc
        port = tokens[4] if len(tokens) > 4 else None
        try:
            captain_type = CaptainType(tokens[1])
        except ValueError as exc:
            raise ScriptError(f"Unknown captain type: {tokens[1]}") from exc
        world = new_game(tokens[2], port, captain_type, seed=seed)
        state["world"] = world
        state["rng"] = random.Random(world.seed)
        state["trade_seq"] = 0
        return
    if state.get("world") is None:
        raise ScriptError("No active game")
    if cmd == "buy":
        if len(tokens) != 3:
            raise ScriptError("Usage: buy <good> <qty>")
        try:
            qty = int(tokens[2])
        except ValueError as exc:
            raise ScriptError(f"Invalid number: {tokens[2]}") from exc
        do_buy(state, tokens[1], qty, entry)
    elif cmd == "sell":
        if len(tokens) != 3:
            raise ScriptError("Usage: sell <good> <qty>")
        try:
            qty = int(tokens[2])
        except ValueError as exc:
            raise ScriptError(f"Invalid number: {tokens[2]}") from exc
        do_sell(state, tokens[1], qty, entry)
    elif cmd == "depart":
        if len(tokens) != 2:
            raise ScriptError("Usage: depart <port_id>")
        result = depart(state["world"], tokens[1], defer_fee=False)
        if isinstance(result, str):
            raise ScriptError(result)
    elif cmd == "advance":
        do_advance(state, entry)
    else:
        raise ScriptError(f"Unknown command: {cmd}")


def standing_view(standing) -> dict:
    return {
        "regional": {region: standing.regional_standing.get(region, 0) for region in REGIONS},
        "heat": {region: standing.customs_heat.get(region, 0) for region in REGIONS},
        "commercial_trust": standing.commercial_trust,
        "port_standing": [
            {"port_id": port_id, "value": value}
            for port_id, value in standing.port_standing.items()
        ],
        "underworld": dict(standing.underworld_standing),
        "incidents": [
            {
                "day": inc.day,
                "port_id": inc.port_id,
                "region": inc.region,
                "incident_type": inc.incident_type,
                "description": inc.description,
                "heat_delta": inc.heat_delta,
                "standing_delta": inc.standing_delta,
                "trust_delta": inc.trust_delta,
            }
            for inc in standing.recent_incidents
        ],
    }


def snapshot(state: dict, log: list[dict]) -> dict:
    world = state.get("world")
    if world is None:
        return {
            "seed": 0,
            "day": 0,
            "trade_seq": 0,
            "captain": {
                "name": "",
                "captain_type": "",
                "silver": 0,
                "provisions": 0,
                "day": 0,
                "wanted_level": 0,
                "cargo": [],
                "ship": None,
                "standing": {
                    "regional": {},
                    "heat": {},
                    "commercial_trust": 0,
                    "port_standing": [],
                    "underworld": {},
                    "incidents": [],
                },
            },
            "voyage": {
                "origin_id": "",
                "destination_id": "",
                "distance": 0,
                "progress": 0,
                "days_elapsed": 0,
                "status": "",
                "recent_events": [],
            },
            "pending_duel": None,
            "ports": [],
            "log": log,
        }
    ship = world.captain.ship
    ship_view = None
    if ship is not None:
        ship_view = {
            "template_id": ship.template_id,
            "name": ship.name,
            "hull": ship.hull,
            "hull_max": ship.hull_max,
            "cargo_capacity": ship.cargo_capacity,
            "speed": float(ship.speed),
            "crew": ship.crew,
            "crew_max": ship.crew_max,
            "morale": ship.morale,
            "cannons": ship.cannons,
            "sailors": ship.roster.sailors,
            "gunners": ship.roster.gunners,
            "navigators": ship.roster.navigators,
            "surgeons": ship.roster.surgeons,
            "marines": ship.roster.marines,
            "quartermasters": ship.roster.quartermasters,
        }
    pending = None
    duel = world.pirates.pending_duel
    if duel is not None:
        pending = {
            "captain_id": duel.captain_id,
            "captain_name": duel.captain_name,
            "faction_id": duel.faction_id,
            "personality": duel.personality,
            "strength": duel.strength,
            "region": duel.region,
        }
    return {
        "seed": world.seed,
        "day": world.day,
        "trade_seq": state["trade_seq"],
        "captain": {
            "name": world.captain.name,
            "captain_type": world.captain.captain_type,
            "silver": world.captain.silver,
            "provisions": world.captain.provisions,
            "day": world.captain.day,
            "wanted_level": world.captain.wanted_level,
            "cargo": [
                {
                    "good_id": item.good_id,
                    "quantity": item.quantity,
                    "cost_basis": item.cost_basis,
                    "acquired_port": item.acquired_port,
                    "acquired_region": item.acquired_region,
                    "acquired_day": item.acquired_day,
                }
                for item in world.captain.cargo
            ],
            "ship": ship_view,
            "standing": standing_view(world.captain.standing),
        },
        "voyage": {
            "origin_id": world.voyage.origin_id,
            "destination_id": world.voyage.destination_id,
            "distance": world.voyage.distance,
            "progress": world.voyage.progress,
            "days_elapsed": world.voyage.days_elapsed,
            "status": world.voyage.status.value,
            "recent_events": list(world.voyage.recent_events),
        },
        "pending_duel": pending,
        "ports": [
            {
                "id": port.id,
                "market": [
                    {
                        "good_id": slot.good_id,
                        "stock": slot.stock_current,
                        "buy_price": slot.buy_price,
                        "sell_price": slot.sell_price,
                        "flood_penalty": slot.flood_penalty,
                    }
                    for slot in port.market
                ],
            }
            for port in world.ports.values()
        ],
        "log": log,
    }


def run(script: str) -> dict:
    state: dict = {"world": None, "rng": None, "trade_seq": 0}
    log: list[dict] = []
    for line in script.splitlines():
        raw = line.strip()
        if not raw or raw.startswith("#"):
            continue
        tokens = shlex.split(raw)
        entry = blank_entry(raw)
        try:
            dispatch(state, tokens, entry)
        except ScriptError as exc:
            entry["error"] = str(exc)
            log.append(entry)
            break
        log.append(entry)
    return snapshot(state, log)


def main() -> int:
    if len(sys.argv) != 2:
        print("Usage: oracle.py <script>", file=sys.stderr)
        return 2
    with open(sys.argv[1], encoding="utf-8") as fh:
        script = fh.read()
    json.dump(run(script), sys.stdout, indent=2)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
