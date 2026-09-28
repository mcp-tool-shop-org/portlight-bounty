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
from portlight.content.campaign import MILESTONE_BY_ID, MILESTONE_SPECS
from portlight.engine.campaign import (
    CampaignState,
    SessionSnapshot,
    compute_victory_progress,
    evaluate_milestones,
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
    milestone_newly = evaluate_milestones(MILESTONE_SPECS, session_snapshot(state))
    if milestone_newly:
        state["campaign"].completed.extend(milestone_newly)
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
        state["encounter"] = None
        state["player_combat"] = None
        state["opponent_combat"] = None
        state["history"] = fresh_history()
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
    elif cmd == "encounter":
        if len(tokens) < 2 or len(tokens) > 3:
            raise ScriptError("Usage: encounter <negotiate|flee|fight> [captain_id|strength:N]")
        captain_id, band = parse_encounter_target(tokens[2] if len(tokens) > 2 else None)
        do_encounter(state, tokens[1], captain_id, band, entry)
    elif cmd == "naval":
        if len(tokens) != 2:
            raise ScriptError("Usage: naval <action>")
        do_naval(state, tokens[1], entry)
    elif cmd == "board":
        if len(tokens) != 1:
            raise ScriptError("Usage: board")
        do_board(state, entry)
    elif cmd == "fight":
        if len(tokens) != 2:
            raise ScriptError("Usage: fight <action>")
        do_fight(state, tokens[1], entry)
    elif cmd == "capture":
        if len(tokens) != 2:
            raise ScriptError("Usage: capture <crew>")
        try:
            crew = int(tokens[1])
        except ValueError as exc:
            raise ScriptError(f"Invalid number: {tokens[1]}") from exc
        do_capture(state, crew, entry)
    elif cmd == "train":
        if len(tokens) != 2:
            raise ScriptError("Usage: train <style_id>")
        do_train(state, tokens[1])
    elif cmd == "recruit":
        if len(tokens) != 2:
            raise ScriptError("Usage: recruit <companion_id>")
        do_recruit(state, tokens[1])
    elif cmd == "skill":
        if len(tokens) != 2:
            raise ScriptError("Usage: skill <skill_id>")
        do_skill(state, tokens[1])
    elif cmd == "remember":
        if len(tokens) != 3:
            raise ScriptError("Usage: remember <captain_id> <outcome>")
        do_remember(state, tokens[1], tokens[2])
    elif cmd == "agency":
        if len(tokens) != 1:
            raise ScriptError("Usage: agency")
        do_agency(state, entry)
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


def fresh_history() -> dict:
    return {
        "encounters": [],
        "duels_won": 0,
        "duels_lost": 0,
        "naval_victories": 0,
        "naval_defeats": 0,
        "fleet": [],
    }


def parse_encounter_target(token):
    if token is None:
        return None, None
    if token.startswith("strength:"):
        rest = token.split(":", 1)[1]
        try:
            return None, int(rest)
        except ValueError as exc:
            raise ScriptError(f"Invalid number: {rest}") from exc
    return token, None


def remember(state, enc, outcome: str) -> None:
    world = state["world"]
    state["history"]["encounters"].append({
        "captain_id": enc.enemy_captain_id,
        "faction_id": enc.enemy_faction_id,
        "day": world.day,
        "outcome": outcome,
        "region": enc.enemy_region,
    })


def clear_encounter(state) -> None:
    state["encounter"] = None
    state["player_combat"] = None
    state["opponent_combat"] = None


def ensure_encounter(state, captain_id, band) -> None:
    enc = state.get("encounter")
    if enc is not None and enc.phase not in ("", "resolved"):
        return
    if captain_id is not None:
        from portlight.content.factions import PIRATE_CAPTAINS
        if captain_id not in PIRATE_CAPTAINS:
            raise ScriptError(f"Unknown pirate captain: {captain_id}")
    world = state["world"]
    rng = state["rng"]
    if band is not None:
        from portlight.engine.models import EncounterState as Enc
        from portlight.engine.naval import generate_enemy_ship
        dest = world.ports.get(world.voyage.destination_id)
        region = dest.region if dest else "Mediterranean"
        enemy = generate_enemy_ship(f"Band {band}", band, rng)
        enc = Enc(
            enemy_captain_id=f"band-{band}",
            enemy_captain_name=f"Band {band}",
            enemy_faction_id="",
            enemy_personality="balanced",
            enemy_strength=band,
            enemy_region=region,
            enemy_ship_hull=enemy.hull,
            enemy_ship_hull_max=enemy.hull_max,
            enemy_ship_cannons=enemy.cannons,
            enemy_ship_maneuver=enemy.maneuver,
            enemy_ship_speed=enemy.speed,
            enemy_ship_crew=enemy.crew,
            enemy_ship_crew_max=enemy.crew_max,
            phase="approach",
            boarding_progress=0,
            boarding_threshold=3,
        )
    else:
        from portlight.engine.encounter import create_encounter
        enc = create_encounter(
            world.ports, world.voyage.destination_id, rng, captain_id,
        )
        if enc is None:
            raise ScriptError("No pirate captain in this region.")
    state["encounter"] = enc
    state["player_combat"] = None
    state["opponent_combat"] = None


def encounter_view(enc, ship, **kw) -> dict:
    base = {
        "kind": "",
        "phase": enc.phase if enc is not None else "",
        "message": "",
        "choice": "",
        "success": False,
        "escaped": False,
        "hull_damage": 0,
        "enemy_captain_id": enc.enemy_captain_id if enc is not None else "",
        "enemy_captain_name": enc.enemy_captain_name if enc is not None else "",
        "enemy_strength": enc.enemy_strength if enc is not None else 0,
        "turn": 0,
        "player_action": "",
        "enemy_action": "",
        "player_hull_delta": 0,
        "enemy_hull_delta": 0,
        "player_crew_delta": 0,
        "enemy_crew_delta": 0,
        "boarding_progress": enc.boarding_progress if enc is not None else 0,
        "boarding_threshold": enc.boarding_threshold if enc is not None else 0,
        "enemy_sunk": False,
        "player_sunk": False,
        "boarding_triggered": False,
        "flavor": "",
        "player_hull": ship.hull if ship is not None else 0,
        "enemy_hull": enc.enemy_ship_hull if enc is not None else 0,
        "player_crew": ship.crew if ship is not None else 0,
        "enemy_crew": enc.enemy_ship_crew if enc is not None else 0,
        "player_crew_lost": 0,
        "enemy_crew_lost": 0,
        "player_advantage": False,
        "damage_to_opponent": 0,
        "damage_to_player": 0,
        "player_hp": 0,
        "opponent_hp": 0,
        "player_stamina_delta": 0,
        "opponent_stamina_delta": 0,
        "player_won": False,
        "draw": False,
        "injury": "",
        "opponent_injury": "",
        "style_effect": "",
        "prize_ok": False,
        "prize_reason": "",
    }
    base.update(kw)
    return base


def do_encounter(state, choice, captain_id, band, entry) -> None:
    from portlight.engine.encounter import begin_fight, resolve_flee, resolve_negotiate

    choice = choice.strip().lower()
    if choice not in ("negotiate", "flee", "fight"):
        raise ScriptError("Choose: negotiate, flee, or fight")
    enc = state.get("encounter")
    if enc is not None and enc.phase not in ("", "resolved"):
        if enc.phase != "approach":
            raise ScriptError(
                "No active encounter. Encounters happen during pirate encounters at sea."
            )
    else:
        ensure_encounter(state, captain_id, band)
    world = state["world"]
    ship = world.captain.ship
    if ship is None:
        raise ScriptError("No ship")
    enc = state["encounter"]
    rng = state["rng"]
    if choice == "negotiate":
        success, message = resolve_negotiate(
            enc, world.captain.standing.underworld_standing,
            world.captain.captain_type, rng,
        )
        if success:
            outcome = "alliance" if "ally" in message else "trade"
            remember(state, enc, outcome)
            entry["encounter"] = encounter_view(
                enc, ship, kind="choice", choice="negotiate", success=True, message=message,
            )
            clear_encounter(state)
        else:
            message = message + "\n" + begin_fight(enc, ship)
            entry["encounter"] = encounter_view(
                enc, ship, kind="choice", choice="negotiate", success=False, message=message,
            )
    elif choice == "flee":
        escaped, damage, message = resolve_flee(enc, ship, rng)
        if damage > 0:
            ship.hull = max(0, ship.hull - damage)
        if escaped:
            remember(state, enc, "fled")
            entry["encounter"] = encounter_view(
                enc, ship, kind="choice", choice="flee", success=True, escaped=True,
                hull_damage=damage, message=message,
            )
            clear_encounter(state)
        else:
            message = message + "\n" + begin_fight(enc, ship)
            entry["encounter"] = encounter_view(
                enc, ship, kind="choice", choice="flee", success=False, escaped=False,
                hull_damage=damage, message=message,
            )
    else:
        message = begin_fight(enc, ship)
        entry["encounter"] = encounter_view(
            enc, ship, kind="choice", choice="fight", success=True, message=message,
        )


def do_naval(state, action, entry) -> None:
    from portlight.app.session import apply_crew_casualties
    from portlight.engine.encounter import resolve_naval_turn
    from portlight.engine.models import EnemyShip
    from portlight.engine.naval import attempt_flee, get_valid_actions

    enc = state.get("encounter")
    if enc is None or enc.phase != "naval":
        raise ScriptError("Not in naval combat.")
    world = state["world"]
    ship = world.captain.ship
    if ship is None:
        raise ScriptError("No ship")
    action = action.strip().lower()
    valid = get_valid_actions(ship.cannons)
    if action not in valid:
        raise ScriptError("Invalid action. Available: " + ", ".join(valid))
    rng = state["rng"]
    if action == "flee":
        enemy = EnemyShip(
            name=f"{enc.enemy_captain_name}'s Ship",
            hull=enc.enemy_ship_hull, hull_max=enc.enemy_ship_hull_max,
            cannons=enc.enemy_ship_cannons, maneuver=enc.enemy_ship_maneuver,
            speed=enc.enemy_ship_speed, crew=enc.enemy_ship_crew,
            crew_max=enc.enemy_ship_crew_max,
        )
        escaped, damage = attempt_flee(ship, enemy, rng)
        ship.hull = max(0, ship.hull - damage)
        enc.naval_turns += 1
        message = "You break away!" if escaped else f"Flee failed! Their broadside rakes you for {damage} hull damage."
        if escaped and damage > 0:
            message += f" A parting shot catches your hull for {damage} damage."
        player_sunk = ship.hull <= 0
        crew_gone = ship.crew <= 0
        view = encounter_view(
            enc, ship, kind="naval", choice="flee", success=escaped, escaped=escaped,
            hull_damage=damage, message=message, turn=enc.naval_turns, player_sunk=player_sunk,
        )
        if escaped:
            remember(state, enc, "fled")
            clear_encounter(state)
            view["phase"] = "resolved"
        elif player_sunk or crew_gone:
            if crew_gone and not player_sunk:
                _plunder(world.captain)
                view["message"] = message + " No crew left to sail."
            state["history"]["naval_defeats"] += 1
            clear_encounter(state)
            view["phase"] = "resolved"
        entry["encounter"] = view
        return
    result = resolve_naval_turn(enc, action, ship, rng)
    ship.hull = max(0, ship.hull + result["player_hull_delta"])
    crew_lost = max(0, -int(result.get("player_crew_delta") or 0))
    apply_crew_casualties(ship, crew_lost)
    enemy_sunk = enc.enemy_ship_hull <= 0
    boarding_triggered = enc.boarding_progress >= enc.boarding_threshold
    player_sunk = ship.hull <= 0
    crew_gone = ship.crew <= 0
    prize_ok = False
    prize_reason = ""
    if enemy_sunk:
        from portlight.engine.encounter import can_capture_prize as gate
        from portlight.engine.models import max_fleet_size
        trust = world.captain.standing.commercial_trust
        prize_ok, prize_reason = gate(world.captain, enc, max_fleet_size(trust))
    view = encounter_view(
        enc, ship, kind="naval", turn=result["turn"], player_action=result["player_action"],
        enemy_action=result["enemy_action"], player_hull_delta=result["player_hull_delta"],
        enemy_hull_delta=result["enemy_hull_delta"], player_crew_delta=result["player_crew_delta"],
        enemy_crew_delta=result["enemy_crew_delta"], boarding_progress=result["boarding_progress"],
        flavor=result["flavor"], enemy_sunk=enemy_sunk, player_sunk=player_sunk,
        boarding_triggered=boarding_triggered, prize_ok=prize_ok, prize_reason=prize_reason or "",
    )
    if enemy_sunk:
        state["history"]["naval_victories"] += 1
        if prize_ok:
            enc.phase = "capture_available"
            view["phase"] = "capture_available"
        else:
            remember(state, enc, "attack")
            clear_encounter(state)
            view["phase"] = "resolved"
    elif player_sunk or (crew_gone and enc.phase != "boarding"):
        if crew_gone and not player_sunk:
            _plunder(world.captain)
        state["history"]["naval_defeats"] += 1
        clear_encounter(state)
        view["phase"] = "resolved"
    entry["encounter"] = view


def do_board(state, entry) -> None:
    from portlight.app.session import apply_crew_casualties
    from portlight.engine.encounter import resolve_boarding_phase

    enc = state.get("encounter")
    if enc is None or enc.phase != "boarding":
        raise ScriptError("Not boarding.")
    world = state["world"]
    ship = world.captain.ship
    crew = ship.crew if ship is not None else 0
    outcome = resolve_boarding_phase(enc, crew, state["rng"])
    if ship is not None:
        apply_crew_casualties(ship, outcome["player_crew_lost"])
    state["player_combat"] = None
    state["opponent_combat"] = None
    entry["encounter"] = encounter_view(
        enc, ship, kind="board",
        player_crew_lost=outcome["player_crew_lost"],
        enemy_crew_lost=outcome["enemy_crew_lost"],
        player_advantage=outcome["player_advantage"],
        flavor=outcome["flavor"], message=outcome["flavor"],
    )


def do_fight(state, action, entry) -> None:
    from portlight.engine.combat import get_available_actions
    from portlight.engine.encounter import create_duel_combatants, resolve_duel_turn

    enc = state.get("encounter")
    if enc is None or enc.phase != "duel":
        raise ScriptError("Not in personal combat.")
    action = action.strip().lower()
    world = state["world"]
    if state.get("player_combat") is None or state.get("opponent_combat") is None:
        crew = world.captain.ship.crew if world.captain.ship else 5
        state["player_combat"], state["opponent_combat"] = create_duel_combatants(
            enc, crew, world.captain.active_style, [], None, 0, 0,
        )
    player = state["player_combat"]
    opponent = state["opponent_combat"]
    valid = get_available_actions(player)
    if action not in valid:
        raise ScriptError("Invalid action. Available: " + ", ".join(valid))
    result = resolve_duel_turn(enc, action, player, opponent, state["rng"])
    player_hp = player.hp
    opponent_hp = opponent.hp
    finished = enc.phase == "resolved"
    player_won = finished and opponent_hp <= 0 and player_hp > 0
    draw = finished and player_hp <= 0 and opponent_hp <= 0
    ship = world.captain.ship
    view = encounter_view(
        enc, ship, kind="fight", turn=result.turn, player_action=result.player_action,
        enemy_action=result.opponent_action, damage_to_opponent=result.damage_to_opponent,
        damage_to_player=result.damage_to_player,
        player_stamina_delta=result.player_stamina_delta,
        opponent_stamina_delta=result.opponent_stamina_delta,
        flavor=result.flavor, message=result.flavor,
        player_hp=player_hp, opponent_hp=opponent_hp,
        player_won=player_won, draw=draw,
        injury=result.injury_inflicted or "",
        opponent_injury=result.opponent_injury or "",
        style_effect=result.style_effect or "",
    )
    if finished:
        if player_won or draw:
            state["history"]["duels_won"] += 1
        else:
            state["history"]["duels_lost"] += 1
            loss = 15 + enc.enemy_strength * 3
            world.captain.silver = max(0, world.captain.silver - loss)
        outcome = "duel_win" if player_won else ("duel_draw" if draw else "duel_loss")
        remember(state, enc, outcome)
        clear_encounter(state)
        view["phase"] = "resolved"
    entry["encounter"] = view


def do_capture(state, crew_to_prize, entry) -> None:
    enc = state.get("encounter")
    if enc is None or enc.phase != "capture_available":
        raise ScriptError("Cannot capture: No ship available to capture.")
    world = state["world"]
    ship = world.captain.ship
    if crew_to_prize <= 0:
        entry["encounter"] = encounter_view(
            enc, ship, kind="capture", phase="resolved", message="You let the prize go under.",
        )
        clear_encounter(state)
        return
    from portlight.engine.encounter import can_capture_prize, capture_prize, prize_template_id
    from portlight.engine.models import max_fleet_size
    from portlight.content.ships import SHIPS
    trust = world.captain.standing.commercial_trust
    ok, reason = can_capture_prize(world.captain, enc, max_fleet_size(trust))
    if not ok:
        raise ScriptError(f"Cannot capture: {reason}")
    prize_tid = prize_template_id(enc.enemy_strength)
    prize_min = SHIPS[prize_tid].crew_min if prize_tid in SHIPS else 3
    current = SHIPS.get(ship.template_id) if ship is not None else None
    current_min = current.crew_min if current else 3
    if crew_to_prize < prize_min:
        raise ScriptError(f"Cannot capture: Need at least {prize_min} crew for the prize ship.")
    have = ship.crew if ship is not None else 0
    leftover = have - crew_to_prize
    if leftover < current_min:
        raise ScriptError(
            f"Cannot capture: Would leave your flagship with {leftover} crew (need {current_min})."
        )
    owned = capture_prize(world.captain, enc, crew_to_prize, state["rng"])
    owned.docked_port_id = world.voyage.destination_id
    world.captain.fleet.append(owned)
    state["history"]["fleet"].append({
        "template_id": owned.ship.template_id,
        "name": owned.ship.name,
        "hull": owned.ship.hull,
        "hull_max": owned.ship.hull_max,
        "crew": owned.ship.crew,
        "docked_port_id": owned.docked_port_id,
    })
    remember(state, enc, "attack")
    name = owned.ship.name
    entry["encounter"] = encounter_view(
        enc, world.captain.ship, kind="capture", phase="resolved",
        message=f"Prize captured! {name} added to your fleet.",
        prize_ok=True, prize_reason=name,
        player_crew=world.captain.ship.crew if world.captain.ship else 0,
    )
    clear_encounter(state)


def _plunder(captain) -> None:
    for item in captain.cargo:
        item.quantity = max(0, item.quantity - item.quantity // 2)
    captain.cargo = [item for item in captain.cargo if item.quantity > 0]
    captain.silver -= captain.silver // 4


def split_stances(tokens: list[str]) -> list[str]:
    stances = []
    for token in tokens:
        for part in token.split(","):
            part = part.strip()
            if part:
                stances.append(part)
    return stances


def do_train(state, style_id: str) -> None:
    """`portlight train`: learn a style, then advance training_days."""
    from portlight.content.fighting_styles import FIGHTING_STYLES
    from portlight.engine.training import can_learn_style, learn_style

    world = state["world"]
    port = current_port(world)
    if port is None:
        raise ScriptError("Must be docked at a port to train.")
    error = can_learn_style(
        world.captain.learned_styles, set(), world.captain.silver, port.id, style_id,
    )
    if error:
        raise ScriptError(error)
    world.captain.learned_styles, world.captain.silver = learn_style(
        world.captain.learned_styles, world.captain.silver, style_id,
    )
    for _ in range(FIGHTING_STYLES[style_id].training_days):
        do_advance(state, {})


def do_skill(state, skill_id: str) -> None:
    """`portlight learn-skill`: pay the next level, then advance training days."""
    from portlight.content.skills import SKILLS
    from portlight.engine.skill_engine import can_learn_skill, learn_skill

    world = state["world"]
    port = current_port(world)
    if port is None:
        raise ScriptError("Must be docked to learn skills.")
    if skill_id not in SKILLS:
        lowered = skill_id.lower()
        for sid, spec in SKILLS.items():
            if spec.name.lower() == lowered:
                skill_id = sid
                break
    error = can_learn_skill(world.captain.skills, world.captain.silver, port.id, skill_id)
    if error:
        raise ScriptError(error)
    world.captain.skills, world.captain.silver, days = learn_skill(
        world.captain.skills, world.captain.silver, skill_id,
    )
    for _ in range(days):
        do_advance(state, {})


def _party_from(raw):
    from portlight.engine.companion_engine import CompanionState, PartyState

    if isinstance(raw, PartyState):
        return raw
    companions = [
        CompanionState(
            companion_id=item["companion_id"],
            role_id=item["role_id"],
            morale=item.get("morale", 70),
            joined_day=item.get("joined_day", 0),
            personality=item.get("personality", "pragmatic"),
        )
        for item in raw.get("companions", [])
    ]
    return PartyState(
        companions=companions,
        max_size=raw.get("max_size", 2),
        departed=list(raw.get("departed", [])),
    )


def _party_dict(party) -> dict:
    return {
        "companions": [
            {
                "companion_id": member.companion_id,
                "role_id": member.role_id,
                "morale": member.morale,
                "joined_day": member.joined_day,
                "personality": member.personality,
            }
            for member in party.companions
        ],
        "max_size": party.max_size,
        "departed": list(party.departed),
    }


def do_remember(state, captain_id: str, outcome: str) -> None:
    """Seed one captain memory. Encounter resolution does this in the real game."""
    from portlight.content.factions import PIRATE_CAPTAINS
    from portlight.engine.captain_memory import get_or_create_memory, record_encounter

    if captain_id not in PIRATE_CAPTAINS:
        raise ScriptError(f"Unknown pirate captain: {captain_id}")
    world = state["world"]
    port = current_port(world)
    region = port.region if port is not None else "Mediterranean"
    if world.voyage is not None and port is None:
        dest = world.ports.get(world.voyage.destination_id)
        region = dest.region if dest is not None else "Mediterranean"
    memory = get_or_create_memory(world.pirates.captain_memories, captain_id)
    record_encounter(memory, world.day, region, outcome, False, False, 0)


def do_agency(state, entry: dict) -> None:
    """`GameSession.tick_sea_captain_agency`.

    Calls `create_encounter` with no target, then overwrites identity. The
    snapshot records `pending_duel` and the encounter fields. Rust also stores
    that encounter on the session so the v12 `pirate_state` blob can persist it.
    """
    from portlight.content.factions import PIRATE_CAPTAINS
    from portlight.engine.captain_memory import tick_captain_agency
    from portlight.engine.encounter import create_encounter
    from portlight.engine.models import PendingDuel

    world = state["world"]
    if world.voyage is None or world.voyage.status != VoyageStatus.AT_SEA:
        entry["agency"] = {"ambush": False, "encounter": None, "notices": []}
        return
    dest = world.ports.get(world.voyage.destination_id)
    region = dest.region if dest is not None else "Mediterranean"
    actions = tick_captain_agency(
        world.pirates.captain_memories,
        region,
        world.captain.silver,
        world.day,
        state["rng"],
    )
    notices = []
    encounter = None
    ambush = False
    for action in actions:
        notices.append({"effect_type": action.effect_type, "message": action.message})
        if action.effect_type == "silver":
            world.captain.silver += action.effect_value
        elif action.effect_type == "encounter" and encounter is None:
            dest_id = world.voyage.destination_id or "porto_novo"
            enc = create_encounter(world.ports, dest_id, state["rng"])
            if not enc:
                continue
            cap = PIRATE_CAPTAINS.get(action.captain_id)
            enc.enemy_captain_id = action.captain_id
            enc.enemy_captain_name = action.captain_name
            if cap is not None:
                enc.enemy_faction_id = cap.faction_id
                enc.enemy_personality = cap.personality
                enc.enemy_strength = cap.strength
            enc.enemy_region = region
            if action.verb == "ambush":
                enc.phase = "naval"
                ambush = True
            world.pirates.pending_duel = PendingDuel(
                captain_id=enc.enemy_captain_id,
                captain_name=enc.enemy_captain_name,
                faction_id=enc.enemy_faction_id,
                personality=enc.enemy_personality,
                strength=enc.enemy_strength,
                region=enc.enemy_region,
            )
            encounter = {
                "enemy_captain_id": enc.enemy_captain_id,
                "enemy_captain_name": enc.enemy_captain_name,
                "enemy_faction_id": enc.enemy_faction_id,
                "enemy_personality": enc.enemy_personality,
                "enemy_strength": enc.enemy_strength,
                "enemy_region": enc.enemy_region,
                "enemy_ship_hull": enc.enemy_ship_hull,
                "enemy_ship_hull_max": enc.enemy_ship_hull_max,
                "enemy_ship_cannons": enc.enemy_ship_cannons,
                "enemy_ship_maneuver": enc.enemy_ship_maneuver,
                "enemy_ship_speed": enc.enemy_ship_speed,
                "enemy_ship_crew": enc.enemy_ship_crew,
                "enemy_ship_crew_max": enc.enemy_ship_crew_max,
                "phase": enc.phase,
                "boarding_progress": enc.boarding_progress,
                "boarding_threshold": enc.boarding_threshold,
                "naval_turns": enc.naval_turns,
                "duel_turns": enc.duel_turns,
            }
            break
    entry["agency"] = {"ambush": ambush, "encounter": encounter, "notices": notices}


def do_recruit(state, companion_id: str) -> None:
    """`portlight recruit`: charge hire cost, then add the companion."""
    from portlight.content.companions import COMPANIONS
    from portlight.engine.companion_engine import can_recruit, recruit

    world = state["world"]
    port = current_port(world)
    if port is None:
        raise ScriptError("Must be docked to recruit companions.")
    party = _party_from(world.captain.party)
    error = can_recruit(
        party,
        companion_id,
        world.captain.silver,
        world.captain.standing.regional_standing,
        port.id,
    )
    if error:
        raise ScriptError(error)
    world.captain.silver -= COMPANIONS[companion_id].hire_cost
    recruit(party, companion_id, world.day)
    world.captain.party = _party_dict(party)


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
    captain_view = {
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
    }
    history = state.get("history") or fresh_history()
    if history["encounters"]:
        captain_view["encounters"] = history["encounters"]
    if history["duels_won"]:
        captain_view["duels_won"] = history["duels_won"]
    if history["duels_lost"]:
        captain_view["duels_lost"] = history["duels_lost"]
    if history["naval_victories"]:
        captain_view["naval_victories"] = history["naval_victories"]
    if history["naval_defeats"]:
        captain_view["naval_defeats"] = history["naval_defeats"]
    if history["fleet"]:
        captain_view["fleet"] = history["fleet"]
    snap = {
        "seed": world.seed,
        "day": world.day,
        "trade_seq": state["trade_seq"],
        "captain": captain_view,
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
    captain = snap["captain"]
    if world.captain.skills:
        captain["skills"] = [
            {"id": skill_id, "level": level}
            for skill_id, level in world.captain.skills.items()
        ]
    if world.captain.learned_styles:
        captain["learned_styles"] = list(world.captain.learned_styles)
    party = world.captain.party if isinstance(world.captain.party, dict) else _party_dict(world.captain.party)
    if party.get("companions"):
        captain["companions"] = list(party["companions"])
    milestones = []
    for completion in state["campaign"].completed:
        spec = MILESTONE_BY_ID.get(completion.milestone_id)
        milestones.append({
            "milestone_id": completion.milestone_id,
            "completed_day": completion.completed_day,
            "evidence": completion.evidence,
            "family": spec.family.value if spec else "",
        })
    if milestones:
        logged = snap.pop("log")
        snap["milestones"] = milestones
        snap["log"] = logged
    return snap


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
