#!/usr/bin/env python3
"""Run a Portlight action script through the Python engine and print state JSON.

This is the reference side of the parity harness. It calls the same engine
functions the Rust script runner calls — not the full GameSession — so
sea culture and insurance stay out of the comparison. The new-game
contract board is drawn from Random(seed + 7919) and the session RNG is
restored. Arrival refreshes the board from the session stream.
`duel` and `resolve_duel` call `engine/duel.py`. See docs/PORTING-PLAN.md.

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
    work_docks,
)
from portlight.engine.models import VoyageStatus
from portlight.engine.reputation import (
    record_inspection_outcome,
    record_port_arrival,
    record_trade_outcome,
    tick_reputation,
)
from portlight.engine.ship_stats import compute_daily_wages
from portlight.engine.campaign import (
    CampaignState,
    SessionSnapshot,
    compute_victory_progress,
    evaluate_victory_closure,
)
from portlight.engine.contracts import (
    ContractBoard,
    accept_offer,
    check_delivery,
    generate_offers,
    resolve_completed,
    tick_contracts,
)
from portlight.engine.infrastructure import InfrastructureState
from portlight.engine.duel import resolve_duel
from portlight.engine.reputation import get_service_modifier
from portlight.engine.voyage import advance_day, arrive, depart
from portlight.receipts.models import ReceiptLedger, TradeReceipt


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


def family_name(value) -> str:
    if value is None:
        return ""
    return value.value if hasattr(value, "value") else str(value)


def contract_row(
    contract,
    outcome_type: str,
    silver_delta: int,
    trust_delta: int,
    standing_delta: int,
    heat_delta: int,
    summary: str,
) -> dict:
    return {
        "contract_id": contract.offer_id,
        "outcome_type": outcome_type,
        "family": family_name(contract.family),
        "good_id": contract.good_id,
        "quantity": contract.required_quantity,
        "delivered_quantity": contract.delivered_quantity,
        "destination_port_id": contract.destination_port_id,
        "deadline_day": contract.deadline_day,
        "reward_silver": contract.reward_silver,
        "silver_delta": silver_delta,
        "trust_delta": trust_delta,
        "standing_delta": standing_delta,
        "heat_delta": heat_delta,
        "summary": summary,
    }


def attach_contracts(entry: dict, rows: list[dict]) -> None:
    if rows:
        entry["contracts"] = rows


def refresh_board(state, port) -> None:
    """GameSession._refresh_board with empty infrastructure effects."""
    world = state["world"]
    board = state["board"]
    if board.last_refresh_day == world.day:
        return
    from portlight.content.contracts import TEMPLATES
    from portlight.engine.voyage import ship_class_rank

    ship = world.captain.ship
    rank = ship_class_rank(ship.template_id) if ship else 0
    board.offers = generate_offers(
        TEMPLATES,
        world,
        port,
        world.captain.standing,
        world.captain.captain_type,
        state["rng"],
        player_ship_rank=rank,
        max_offers=board.max_offers,
    )
    board.last_refresh_day = world.day


def refresh_with_board_rng(state, port) -> None:
    """Save the session RNG, draw Random(seed + 7919), then restore it."""
    saved = state["rng"]
    state["rng"] = random.Random(state["world"].seed + 7919)
    refresh_board(state, port)
    state["rng"] = saved


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
    state["ledger"].append(result)
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
    cargo_item = next((c for c in world.captain.cargo if c.good_id == good_id), None)
    source_port = cargo_item.acquired_port if cargo_item else ""
    source_region = cargo_item.acquired_region if cargo_item else ""
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
    credited = check_delivery(
        state["board"], port.id, good_id, result.quantity, source_port, source_region,
    )
    if credited:
        attach_contracts(entry, settle_fulfilled(state))
    reprice_port(world, port)
    state["ledger"].append(result)
    entry["receipt"] = receipt_view(result)


def session_snapshot(state):
    world = state["world"]
    return SessionSnapshot(
        captain=world.captain,
        world=world,
        board=state["board"],
        infra=state["infra"],
        ledger=state["ledger"],
        campaign=state["campaign"],
    )


def do_advance(state, entry: dict) -> None:
    world = state["world"]
    rng = state["rng"]
    tick_reputation(world.captain.standing)
    before = {c.offer_id: c for c in state["board"].active}
    outcomes = tick_contracts(state["board"], world.day)
    rows = []
    for outcome in outcomes:
        world.captain.silver += outcome.silver_delta
        rows.append(stash_resolution(
            state,
            before[outcome.contract_id],
            outcome,
        ))
    attach_contracts(entry, rows)
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
                refresh_board(state, port)
        entry["events"] = [event_view(event) for event in events]
    reprice_all(world)
    newly = evaluate_victory_closure(session_snapshot(state))
    if newly:
        state["campaign"].completed_paths.extend(newly)


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
        state["ledger"] = ReceiptLedger()
        state["board"] = ContractBoard()
        state["infra"] = InfrastructureState()
        state["campaign"] = CampaignState()
        port = current_port(world)
        if port is not None:
            refresh_with_board_rng(state, port)
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
    elif cmd == "hire":
        if len(tokens) < 2 or len(tokens) > 3:
            raise ScriptError("Usage: hire <count> [role]")
        try:
            count = int(tokens[1])
        except ValueError as exc:
            raise ScriptError(f"Invalid number: {tokens[1]}") from exc
        role = tokens[2] if len(tokens) > 2 else "sailor"
        do_hire(state, count, role)
    elif cmd == "provision":
        if len(tokens) != 2:
            raise ScriptError("Usage: provision <days>")
        try:
            days = int(tokens[1])
        except ValueError as exc:
            raise ScriptError(f"Invalid number: {tokens[1]}") from exc
        do_provision(state, days)
    elif cmd == "work":
        if len(tokens) != 1:
            raise ScriptError("Usage: work")
        do_work(state, entry)
    elif cmd == "duel":
        if len(tokens) < 2:
            raise ScriptError("Usage: duel <stance>[,<stance>...]")
        stances = split_stances(tokens[1:])
        do_duel(state, stances, entry)
    elif cmd == "resolve_duel":
        do_resolve_duel(state, entry)
    elif cmd == "accept_contract":
        if len(tokens) != 2:
            raise ScriptError("Usage: accept_contract <offer_id>")
        do_accept(state, tokens[1], entry)
    elif cmd == "complete_contract":
        if len(tokens) != 2:
            raise ScriptError("Usage: complete_contract <offer_id>")
        do_complete(state, tokens[1], entry)
    else:
        raise ScriptError(f"Unknown command: {cmd}")


def do_accept(state, offer_id: str, entry: dict) -> None:
    result = accept_offer(state["board"], offer_id, state["world"].day)
    if isinstance(result, str):
        raise ScriptError(result)
    attach_contracts(entry, [contract_row(
        result, "accepted", 0, 0, 0, 0, result.title,
    )])


def stash_resolution(state, contract, outcome) -> dict:
    row = contract_row(
        contract,
        outcome.outcome_type,
        outcome.silver_delta,
        outcome.trust_delta,
        outcome.standing_delta,
        outcome.heat_delta,
        outcome.summary,
    )
    state.setdefault("resolved_contracts", {})[contract.offer_id] = row
    return row


def settle_fulfilled(state) -> list[dict]:
    """resolve_completed, then silver and the house-books equivalent on the board."""
    board = state["board"]
    ready = [
        c for c in board.active
        if c.status == "accepted" and c.delivered_quantity >= c.required_quantity
    ]
    outcomes = resolve_completed(board, state["world"].day)
    rows = []
    for outcome in outcomes:
        state["world"].captain.silver += outcome.silver_delta
        contract = next(c for c in ready if c.offer_id == outcome.contract_id)
        rows.append(stash_resolution(state, contract, outcome))
    return rows


def do_complete(state, offer_id: str, entry: dict) -> None:
    """Same settlement as sell. A second call returns the recorded outcome."""
    board = state["board"]
    contract = next((c for c in board.active if c.offer_id == offer_id), None)
    if contract is None:
        row = state.get("resolved_contracts", {}).get(offer_id)
        if row is None:
            raise ScriptError("No active contract with that ID")
        attach_contracts(entry, [row])
        return
    if contract.status != "accepted":
        raise ScriptError("No active contract with that ID")
    if contract.delivered_quantity < contract.required_quantity:
        raise ScriptError("Contract is not yet fulfilled")
    rows = [row for row in settle_fulfilled(state) if row["contract_id"] == offer_id]
    if not rows:
        raise ScriptError("Contract is not yet fulfilled")
    attach_contracts(entry, rows)


def split_stances(tokens: list[str]) -> list[str]:
    stances = []
    for token in tokens:
        for part in token.split(","):
            part = part.strip()
            if part:
                stances.append(part)
    return stances


def do_hire(state, count: int, role: str) -> None:
    """GameSession.hire_crew, without the save side effect."""
    from portlight.content.crew_roles import ROLE_SPECS, get_role_count, set_role_count
    from portlight.content.officer_names import generate_officer_name, generate_officer_trait
    from portlight.content.upgrades import UPGRADES
    from portlight.engine.models import CrewRole, Officer
    from portlight.engine.ship_stats import resolve_crew_max

    world = state["world"]
    port = current_port(world)
    if port is None:
        raise ScriptError("Must be docked to hire crew")
    ship = world.captain.ship
    if ship is None:
        raise ScriptError("No ship")
    space = resolve_crew_max(ship, UPGRADES) - ship.crew
    if space <= 0:
        raise ScriptError("Crew is already full")
    try:
        crew_role = CrewRole(role.lower())
    except ValueError as exc:
        valid = ", ".join(r.value for r in CrewRole)
        raise ScriptError(f"Unknown role: {role}. Valid: {valid}") from exc
    spec = ROLE_SPECS[crew_role]
    if spec.max_per_ship is not None:
        current = get_role_count(ship.roster, crew_role)
        avail = spec.max_per_ship - current
        if avail <= 0:
            raise ScriptError(f"Already at maximum {spec.name}s ({spec.max_per_ship})")
        count = min(count, avail)
    count = min(count, space)
    if crew_role == CrewRole.SAILOR:
        cost_per = port.crew_cost
    else:
        cost_per = spec.wage * 10
    cost = count * cost_per
    if cost > world.captain.silver:
        raise ScriptError(
            f"Need {cost} silver for {count} {spec.name}(s) ({cost_per}/each), have {world.captain.silver}"
        )
    world.captain.silver -= cost
    current = get_role_count(ship.roster, crew_role)
    set_role_count(ship.roster, crew_role, current + count)
    ship.sync_crew()
    if crew_role != CrewRole.SAILOR:
        for _ in range(count):
            ship.officers.append(Officer(
                name=generate_officer_name(port.region, state["rng"]),
                role=crew_role,
                origin_port=port.id,
                trait=generate_officer_trait(state["rng"]),
            ))


def do_work(state, entry: dict) -> None:
    """GameSession.work: work_docks, then copy captain.day onto world.day."""
    world = state["world"]
    if current_port(world) is None:
        raise ScriptError("Must be docked to work the docks.")
    earned = work_docks(world.captain, state["rng"])
    world.day = world.captain.day
    entry["earned"] = earned


def do_provision(state, days: int) -> None:
    world = state["world"]
    if days <= 0:
        raise ScriptError("Quantity must be a positive number.")
    port = current_port(world)
    if port is None:
        raise ScriptError("Must be docked to provision")
    per_day = max(1, int(port.provision_cost * get_service_modifier(world.captain.standing, port.id)))
    cost = days * per_day
    if cost > world.captain.silver:
        raise ScriptError(
            f"Need {cost} silver for {days} days of provisions ({per_day}/day here), have {world.captain.silver}"
        )
    world.captain.silver -= cost
    world.captain.provisions += days


def _apply_duel(state, stances: list[str], entry: dict) -> None:
    world = state["world"]
    pending = world.pirates.pending_duel
    if pending is None:
        raise ScriptError(
            "No pirate has challenged you. Duels happen during pirate encounters at sea."
        )
    parsed = []
    for stance in stances:
        stance = stance.strip().lower()
        if stance not in {"thrust", "slash", "parry"}:
            raise ScriptError(f"Invalid stance: {stance}. Use: thrust, slash, parry")
        parsed.append(stance)
    if len(parsed) < 3:
        raise ScriptError("Provide at least 3 stances (e.g. thrust,parry,slash,thrust,parry)")
    crew = world.captain.ship.crew if world.captain.ship else 5
    result = resolve_duel(
        player_stances=parsed,
        opponent_id=pending.captain_id,
        opponent_name=pending.captain_name,
        opponent_personality=pending.personality,
        opponent_strength=pending.strength,
        rng=state["rng"],
        player_crew=crew,
    )
    world.captain.silver = max(0, world.captain.silver + result.silver_delta)
    world.pirates.pending_duel = None
    entry["duel"] = {
        "opponent_id": result.opponent_id,
        "opponent_name": result.opponent_name,
        "player_won": result.player_won,
        "draw": result.draw,
        "silver_delta": result.silver_delta,
        "standing_delta": result.standing_delta,
        "rounds": [
            {
                "player_stance": round_.player_stance,
                "opponent_stance": round_.opponent_stance,
                "damage_to_opponent": round_.damage_to_opponent,
                "damage_to_player": round_.damage_to_player,
                "flavor": round_.flavor,
            }
            for round_ in result.rounds
        ],
    }


def do_duel(state, stances: list[str], entry: dict) -> None:
    _apply_duel(state, stances, entry)


def do_resolve_duel(state, entry: dict) -> None:
    world = state["world"]
    if world.pirates.pending_duel is None:
        raise ScriptError(
            "No pirate has challenged you. Duels happen during pirate encounters at sea."
        )
    stances = [state["rng"].choice(["thrust", "slash", "parry"]) for _ in range(5)]
    _apply_duel(state, stances, entry)


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
            "victory": [],
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
        if ship.officers:
            ship_view["officers"] = [
                {
                    "name": officer.name,
                    "role": officer.role.value if hasattr(officer.role, "value") else str(officer.role),
                    "origin_port": officer.origin_port,
                    "trait": officer.trait,
                }
                for officer in ship.officers
            ]
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
        "victory": victory_view(compute_victory_progress(session_snapshot(state))),
        "log": log,
    }


def victory_view(paths) -> list:
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


def run(script: str) -> dict:
    state: dict = {
        "world": None,
        "rng": None,
        "trade_seq": 0,
        "ledger": ReceiptLedger(),
        "board": ContractBoard(),
        "infra": InfrastructureState(),
        "campaign": CampaignState(),
    }
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
