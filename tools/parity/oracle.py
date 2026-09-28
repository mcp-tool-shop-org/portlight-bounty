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
from portlight.content.infrastructure import (
    BrokerTier,
    CreditTier,
    WarehouseTier,
    available_tiers,
    get_broker_spec,
    get_credit_spec,
    get_license_spec,
    get_policy_spec,
    get_tier_spec,
)
from portlight.engine.models import PortFeature, VoyageStatus
from portlight.engine.infrastructure import (
    InfrastructureState,
    deposit_cargo,
    draw_credit,
    emergency_loan,
    expire_voyage_policies,
    lease_warehouse,
    open_broker_office,
    open_credit_line,
    purchase_license,
    purchase_policy,
    repay_credit,
    resolve_claim,
    tick_credit,
    tick_infrastructure,
    withdraw_cargo,
)
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
    for outcome in outcomes:
        if outcome.outcome_type in ("expired", "abandoned"):
            loss_value = abs(outcome.trust_delta) * 50 + abs(outcome.standing_delta) * 30
            resolve_claim(
                state["infra"],
                world.captain,
                "contract_failure",
                loss_value,
                world.day,
                contract_id=outcome.contract_id,
            )
    notes = apply_upkeep(state)
    heal_injuries(state)
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
            from portlight.engine.fleet import fleet_daily_wages
            wage += fleet_daily_wages(world.captain)
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
            settle_event_insurance(state, event, world.voyage.destination_id)
        if world.voyage.status == VoyageStatus.ARRIVED:
            arrive(world)
            notes.extend(expire_voyage_policies(state["infra"]))
            port = world.ports.get(world.voyage.destination_id)
            if port is not None:
                record_port_arrival(world.captain.standing, world.day, port.id, port.region)
                refresh_board(state, port)
        entry["events"] = [event_view(event) for event in events]
    if notes:
        entry["notes"] = notes
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
        state["pending_victory"] = False
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
    elif cmd == "buy_infrastructure":
        if len(tokens) < 2:
            raise ScriptError(
                "Usage: buy_infrastructure warehouse <tier> | broker <region> <tier> | license <id> | dry_dock [ship]"
            )
        do_buy_infrastructure(state, tokens[1], tokens[2:])
    elif cmd == "take_credit":
        if len(tokens) != 3:
            raise ScriptError("Usage: take_credit <tier> <amount>")
        try:
            amount = int(tokens[2])
        except ValueError as exc:
            raise ScriptError(f"Invalid number: {tokens[2]}") from exc
        do_take_credit(state, tokens[1], amount)
    elif cmd == "buy_insurance":
        if len(tokens) < 2 or len(tokens) > 5:
            raise ScriptError("Usage: buy_insurance <policy_id> [target_id] [origin] [destination]")
        target = tokens[2] if len(tokens) > 2 else ""
        origin = tokens[3] if len(tokens) > 3 else ""
        destination = tokens[4] if len(tokens) > 4 else ""
        do_buy_insurance(state, tokens[1], target, origin, destination)
    elif cmd == "deposit":
        if len(tokens) != 3:
            raise ScriptError("Usage: deposit <good> <qty>")
        try:
            qty = int(tokens[2])
        except ValueError as exc:
            raise ScriptError(f"Invalid number: {tokens[2]}") from exc
        do_deposit(state, tokens[1], qty)
    elif cmd == "withdraw":
        if len(tokens) < 3 or len(tokens) > 4:
            raise ScriptError("Usage: withdraw <good> <qty> [source_port]")
        try:
            qty = int(tokens[2])
        except ValueError as exc:
            raise ScriptError(f"Invalid number: {tokens[2]}") from exc
        source = tokens[3] if len(tokens) > 3 else None
        do_withdraw(state, tokens[1], qty, source)
    elif cmd == "repay_credit":
        if len(tokens) != 2:
            raise ScriptError("Usage: repay_credit <amount>")
        try:
            amount = int(tokens[1])
        except ValueError as exc:
            raise ScriptError(f"Invalid number: {tokens[1]}") from exc
        do_repay_credit(state, amount)
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
    elif cmd == "spare":
        if len(tokens) != 1:
            raise ScriptError("Usage: spare")
        do_spare(state, True)
    elif cmd == "take_all":
        if len(tokens) != 1:
            raise ScriptError("Usage: take_all")
        do_spare(state, False)
    elif cmd == "gear":
        if len(tokens) != 2:
            raise ScriptError("Usage: gear <id>")
        do_gear(state, tokens[1])
    elif cmd == "buy_ship":
        if len(tokens) != 2:
            raise ScriptError("Usage: buy_ship <template_id>")
        do_buy_ship(state, tokens[1])
    elif cmd == "upgrade":
        if len(tokens) != 2:
            raise ScriptError("Usage: upgrade <upgrade_id>")
        do_install_upgrade(state, tokens[1])
    elif cmd == "form_convoy":
        if len(tokens) != 1:
            raise ScriptError("Usage: form_convoy")
        do_form_convoy(state)
    elif cmd == "repair_fleet":
        if len(tokens) != 1:
            raise ScriptError("Usage: repair_fleet")
        do_repair_fleet(state)
    elif cmd == "transfer":
        if len(tokens) != 5:
            raise ScriptError("Usage: transfer <good> <qty> <from> <to>")
        try:
            qty = int(tokens[2])
        except ValueError as exc:
            raise ScriptError(f"Invalid number: {tokens[2]}") from exc
        do_transfer(state, tokens[1], qty, tokens[3], tokens[4])
    elif cmd == "maintain":
        if len(tokens) != 2:
            raise ScriptError("Usage: maintain <weapon_id>")
        do_maintain(state, tokens[1])
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
    state["pending_victory"] = False


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
    state["pending_victory"] = False


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
    if not base.get("player_stamina"):
        base.pop("player_stamina", None)
    if not base.get("player_stamina_max"):
        base.pop("player_stamina_max", None)
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
    combat = resolved_player_ship(ship)
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
            message = message + "\n" + begin_fight(enc, combat)
            entry["encounter"] = encounter_view(
                enc, ship, kind="choice", choice="negotiate", success=False, message=message,
            )
    elif choice == "flee":
        escaped, damage, message = resolve_flee(enc, combat, rng)
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
            message = message + "\n" + begin_fight(enc, combat)
            entry["encounter"] = encounter_view(
                enc, ship, kind="choice", choice="flee", success=False, escaped=False,
                hull_damage=damage, message=message,
            )
    else:
        message = begin_fight(enc, combat)
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
    from portlight.content.upgrades import UPGRADES
    from portlight.engine.ship_stats import resolve_cannons
    valid = get_valid_actions(resolve_cannons(ship, UPGRADES))
    if action not in valid:
        raise ScriptError("Invalid action. Available: " + ", ".join(valid))
    rng = state["rng"]
    combat = resolved_player_ship(ship)
    if action == "flee":
        enemy = EnemyShip(
            name=f"{enc.enemy_captain_name}'s Ship",
            hull=enc.enemy_ship_hull, hull_max=enc.enemy_ship_hull_max,
            cannons=enc.enemy_ship_cannons, maneuver=enc.enemy_ship_maneuver,
            speed=enc.enemy_ship_speed, crew=enc.enemy_ship_crew,
            crew_max=enc.enemy_ship_crew_max,
        )
        escaped, damage = attempt_flee(combat, enemy, rng)
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
    result = resolve_naval_turn(enc, action, combat, rng)
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
    from portlight.engine.encounter import resolve_duel_turn

    enc = state.get("encounter")
    if enc is None or enc.phase != "duel":
        raise ScriptError("Not in personal combat.")
    action = action.strip().lower()
    world = state["world"]
    ensure_combatants(state)
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
        player_stamina=player.stamina,
        player_stamina_max=player.stamina_max,
    )
    if finished:
        finish_personal_fight(state, result, player_won, draw)
        if not player_won:
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


def resolved_player_ship(ship):
    """Upgrade-resolved copy. Hull changes stay on the real ship."""
    from portlight.content.upgrades import UPGRADES
    from portlight.engine.ship_stats import resolved_ship
    return resolved_ship(ship, UPGRADES)


def heal_injuries(state) -> None:
    """GameSession.advance: in port, or at sea with a surgeon's bay."""
    world = state["world"]
    if not world.captain.injuries:
        return
    in_port = world.voyage.status != VoyageStatus.AT_SEA
    ship = world.captain.ship
    bay = False
    if ship is not None:
        bay = any(getattr(inst, "upgrade_id", "") == "surgeons_bay" for inst in ship.upgrades)
    if not (in_port or bay):
        return
    from portlight.engine.injuries import heal_injury_tick
    medicines = any(item.good_id == "medicines" for item in world.captain.cargo)
    world.captain.injuries = heal_injury_tick(
        world.captain.injuries, days=1, in_port=True, has_medicines=medicines,
    )


def ensure_combatants(state) -> None:
    """CLI fight path: armor DR and dodge penalty after create_duel_combatants."""
    if state.get("player_combat") is not None and state.get("opponent_combat") is not None:
        return
    from portlight.content.armor import ARMOR
    from portlight.engine.encounter import create_duel_combatants
    world = state["world"]
    enc = state["encounter"]
    gear = world.captain.combat_gear
    throwing = gear.throwing_weapons or {}
    total_throwing = sum(throwing.values())
    tw_ids = []
    for weapon_id, count in throwing.items():
        tw_ids.extend([weapon_id] * count)
    melee_q = gear.weapon_quality.get(gear.melee_weapon, "standard") if gear.melee_weapon else "standard"
    ranged_q = gear.weapon_quality.get(gear.firearm, "standard") if gear.firearm else "standard"
    injury_ids = [injury.injury_id for injury in world.captain.injuries]
    crew = world.captain.ship.crew if world.captain.ship else 5
    player, opponent = create_duel_combatants(
        enc, crew, world.captain.active_style, injury_ids,
        gear.firearm, gear.firearm_ammo, total_throwing,
        gear.mechanical_weapon, gear.mechanical_ammo,
    )
    player.throwing_weapon_ids = tw_ids
    player.melee_weapon_id = gear.melee_weapon
    player.melee_quality = melee_q
    player.ranged_quality = ranged_q
    if gear.armor:
        armor_def = ARMOR.get(gear.armor)
        if armor_def:
            player.armor_dr = armor_def.damage_reduction
            player.dodge_stamina_penalty = armor_def.dodge_penalty
    state["player_combat"] = player
    state["opponent_combat"] = opponent


def finish_personal_fight(state, result, player_won: bool, draw: bool) -> None:
    from portlight.engine.injuries import create_injury
    from portlight.engine.skill_engine import get_degrade_threshold_bonus, get_skill_level
    from portlight.engine.weapon_quality import tick_weapon_degradation
    world = state["world"]
    gear = world.captain.combat_gear
    if result.injury_inflicted:
        world.captain.injuries.append(create_injury(result.injury_inflicted, world.day))
    bonus = get_degrade_threshold_bonus(get_skill_level(world.captain.skills, "blacksmith"))
    if gear.melee_weapon:
        tick_weapon_degradation(
            gear.weapon_quality, gear.weapon_usage, gear.melee_weapon, "melee", 1, bonus,
        )
    if gear.armor:
        tick_weapon_degradation(
            gear.weapon_quality, gear.weapon_usage, gear.armor, "armor", 1, bonus,
        )
    sync_combat_ammo(state)
    if player_won:
        state["pending_victory"] = True
    elif draw:
        state["history"]["duels_won"] += 1
        clear_encounter(state)
    else:
        enc = state.get("encounter")
        strength = enc.enemy_strength if enc is not None else 0
        loss = 15 + strength * 3
        world.captain.silver = max(0, world.captain.silver - loss)
        state["history"]["duels_lost"] += 1
        clear_encounter(state)


def sync_combat_ammo(state) -> None:
    player = state.get("player_combat")
    if player is None:
        return
    gear = state["world"].captain.combat_gear
    gear.firearm_ammo = player.ammo
    gear.mechanical_ammo = player.mechanical_ammo
    if not gear.throwing_weapons:
        return
    total = sum(gear.throwing_weapons.values())
    spent = total - player.throwing_weapons
    for weapon_id in list(gear.throwing_weapons):
        if spent <= 0:
            break
        take = min(spent, gear.throwing_weapons[weapon_id])
        gear.throwing_weapons[weapon_id] -= take
        spent -= take
    gear.throwing_weapons = {key: qty for key, qty in gear.throwing_weapons.items() if qty > 0}


def do_spare(state, spared: bool) -> None:
    enc = state.get("encounter")
    if not state.get("pending_victory") or enc is None:
        if enc is not None and enc.phase == "capture_available":
            raise ScriptError("Prize waiting. Use portlight capture <crew> (or 0 to decline).")
        if spared:
            raise ScriptError("No defeated opponent to spare. Win a duel first.")
        raise ScriptError("No defeated opponent. Win a duel first.")
    from portlight.engine.loot import apply_loot, roll_loot
    from portlight.engine.underworld import record_duel_outcome
    from portlight.engine.weapon_provenance import create_provenance, record_kill
    world = state["world"]
    gear = world.captain.combat_gear
    silver_gain = 20 + enc.enemy_strength * (3 if spared else 7)
    world.captain.silver += silver_gain
    state["history"]["duels_won"] += 1
    from portlight.engine.captain_memory import get_or_create_memory, record_encounter
    from portlight.engine.companion_engine import apply_morale_trigger, check_departures
    memory = get_or_create_memory(world.pirates.captain_memories, enc.enemy_captain_id)
    crew_killed = max(0, enc.enemy_ship_crew_max - enc.enemy_ship_crew)
    record_encounter(
        memory, world.day, enc.enemy_region, "player_won",
        player_spared=spared, player_used_firearm=False, crew_killed=crew_killed,
    )
    record_duel_outcome(
        world.captain.standing.underworld_standing,
        enc.enemy_faction_id,
        True,
        spared,
    )
    if gear.melee_weapon:
        prov = gear.weapon_provenance.get(gear.melee_weapon)
        if prov is None:
            prov = create_provenance(gear.melee_weapon)
            gear.weapon_provenance[gear.melee_weapon] = prov
        record_kill(prov, enc.enemy_captain_id, enc.enemy_captain_name)
    if not spared:
        drops = roll_loot(enc.enemy_strength, enc.enemy_captain_id, state["rng"], 2)
        apply_loot(world.captain, drops)
    trigger = "spared_enemy" if spared else "took_all"
    party = _party_from(world.captain.party)
    apply_morale_trigger(party, trigger)
    check_departures(party)
    world.captain.party = _party_dict(party)
    clear_encounter(state)


def do_buy_ship(state, ship_id: str) -> None:
    from portlight.app.session import _trim_cargo_to_capacity
    from portlight.content.ships import SHIPS, create_ship_from_template
    from portlight.engine.models import OwnedShip, PortFeature, max_fleet_size
    world = state["world"]
    port = current_port(world)
    if port is None:
        raise ScriptError("Must be docked")
    if PortFeature.SHIPYARD not in port.features:
        raise ScriptError(f"{port.name} has no shipyard")
    template = SHIPS.get(ship_id)
    if template is None:
        raise ScriptError(f"Unknown ship: {ship_id}")
    ship = world.captain.ship
    if ship is None:
        raise ScriptError("No ship")
    if template.id == ship.template_id:
        raise ScriptError("You already have this ship")
    if template.price > world.captain.silver:
        raise ScriptError(f"Need {template.price} silver, have {world.captain.silver}")
    trust = world.captain.standing.commercial_trust
    fleet_limit = max_fleet_size(trust)
    fleet_count = len(world.captain.fleet) + 1
    if fleet_count < fleet_limit:
        world.captain.fleet.append(OwnedShip(ship=ship, docked_port_id=port.id))
    else:
        old = SHIPS.get(ship.template_id)
        if old is not None:
            world.captain.silver += int(old.price * 0.4)
    world.captain.silver -= template.price
    world.captain.ship = create_ship_from_template(template)
    _trim_cargo_to_capacity(world.captain.cargo, template.cargo_capacity)


def do_install_upgrade(state, upgrade_id: str) -> None:
    from portlight.content.upgrades import UPGRADES
    from portlight.engine.models import InstalledUpgrade, PortFeature
    world = state["world"]
    port = current_port(world)
    if port is None:
        raise ScriptError("Must be docked")
    if PortFeature.SHIPYARD not in port.features:
        raise ScriptError(f"{port.name} has no shipyard")
    template = UPGRADES.get(upgrade_id)
    if template is None:
        raise ScriptError(f"Unknown upgrade: {upgrade_id}")
    ship = world.captain.ship
    if ship is None:
        raise ScriptError("No ship")
    if len(ship.upgrades) >= ship.upgrade_slots:
        raise ScriptError(
            f"No upgrade slots remaining ({ship.upgrade_slots}/{ship.upgrade_slots} used)"
        )
    if template.price > world.captain.silver:
        raise ScriptError(f"Need {template.price} silver, have {world.captain.silver}")
    world.captain.silver -= template.price
    ship.upgrades.append(InstalledUpgrade(upgrade_id=upgrade_id, installed_day=world.day))


def do_form_convoy(state) -> None:
    world = state["world"]
    port = current_port(world)
    if port is None:
        raise ScriptError("Must be docked")
    for owned in world.captain.fleet:
        if owned.docked_port_id == port.id:
            owned.docked_port_id = ""


def do_repair_fleet(state) -> None:
    """GameSession.repair with no amount: the flagship only.

    fleet.py, dry_dock, and arrival do not patch escort hull. dry_dock restores
    template hull_max and is not this command.
    """
    world = state["world"]
    port = current_port(world)
    if port is None:
        raise ScriptError("Must be docked to repair")
    ship = world.captain.ship
    if ship is None:
        raise ScriptError("No ship")
    damage = ship.hull_max - ship.hull
    if damage == 0:
        raise ScriptError("Ship is already in perfect condition")
    cost_per = max(1, int(port.repair_cost * get_service_modifier(world.captain.standing, port.id)))
    amount = damage
    cost = amount * cost_per
    if cost > world.captain.silver:
        affordable = world.captain.silver // cost_per if cost_per > 0 else 0
        if affordable == 0:
            raise ScriptError("Can't afford any repairs")
        amount = affordable
        cost = amount * cost_per
    world.captain.silver -= cost
    ship.hull += amount


def do_transfer(state, good_id: str, qty: int, src: str, dst: str) -> None:
    from portlight.engine.fleet import transfer_cargo
    world = state["world"]
    port = current_port(world)
    if port is None:
        raise ScriptError("Must be docked")
    err = transfer_cargo(world.captain, good_id, qty, src, dst, port.id)
    if err:
        raise ScriptError(err)


def do_gear(state, gear_id: str) -> None:
    """Buy one item from the port's merchant via `buy_from_merchant`."""
    from portlight.content.merchants import get_merchants_at_port
    from portlight.engine.merchant import buy_from_merchant, get_merchant_inventory
    world = state["world"]
    port = current_port(world)
    if port is None:
        raise ScriptError("Must be docked")
    merchants = get_merchants_at_port(port.id)
    if not merchants:
        raise ScriptError("Unknown merchant")
    chosen = None
    for merchant in merchants:
        inventory = get_merchant_inventory(merchant, port.region)
        if any(item["item_id"] == gear_id for item in inventory):
            chosen = merchant
            break
    if chosen is None:
        chosen = merchants[0]
    result = buy_from_merchant(world.captain, chosen.id, gear_id, 1, port.region)
    if isinstance(result, str):
        raise ScriptError(result)


def do_maintain(state, weapon_id: str) -> None:
    """CLI `maintain`: blacksmith discount, then reset usage."""
    from portlight.engine.skill_engine import apply_maintenance_discount, get_skill_level
    from portlight.engine.weapon_quality import get_maintenance_cost
    world = state["world"]
    if current_port(world) is None:
        raise ScriptError("Must be docked")
    gear = world.captain.combat_gear
    level = get_skill_level(world.captain.skills, "blacksmith")
    base = get_maintenance_cost(weapon_id, gear.weapon_quality)
    cost = apply_maintenance_discount(base, level)
    if world.captain.silver < cost:
        raise ScriptError(
            f"Maintenance costs {cost} silver. You have {world.captain.silver}."
        )
    gear.weapon_usage[weapon_id] = 0
    world.captain.silver -= cost


def cargo_snap(item) -> dict:
    return {
        "good_id": item.good_id,
        "quantity": item.quantity,
        "cost_basis": item.cost_basis,
        "acquired_port": item.acquired_port,
        "acquired_region": item.acquired_region,
        "acquired_day": item.acquired_day,
    }


def fleet_snap(owned) -> dict:
    row = {
        "template_id": owned.ship.template_id,
        "name": owned.ship.name,
        "hull": owned.ship.hull,
        "hull_max": owned.ship.hull_max,
        "crew": owned.ship.crew,
        "docked_port_id": owned.docked_port_id,
    }
    if owned.cargo:
        row["cargo"] = [cargo_snap(item) for item in owned.cargo]
    return row


def provenance_snap(prov) -> dict:
    return {
        "weapon_id": prov.weapon_id,
        "acquired_port": prov.acquired_port,
        "acquired_day": prov.acquired_day,
        "acquired_region": prov.acquired_region,
        "kills": prov.kills,
        "named_kills": list(prov.named_kills),
        "epithet": prov.epithet,
        "custom_name": prov.custom_name,
        "times_recognized": prov.times_recognized,
    }


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
    outcome_str = "duel_win" if result.player_won else ("duel_draw" if result.draw else "duel_loss")
    state["history"]["encounters"].append({
        "captain_id": pending.captain_id,
        "faction_id": pending.faction_id,
        "day": world.day,
        "outcome": outcome_str,
        "region": pending.region,
    })
    if result.player_won:
        state["history"]["duels_won"] += 1
    elif not result.draw:
        state["history"]["duels_lost"] += 1
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
        if ship.upgrades:
            ship_view["upgrades"] = [
                {"upgrade_id": inst.upgrade_id, "installed_day": inst.installed_day}
                for inst in ship.upgrades
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
    if world.captain.fleet:
        captain_view["fleet"] = [fleet_snap(owned) for owned in world.captain.fleet]
    if world.captain.injuries:
        captain_view["injuries"] = [
            {
                "injury_id": injury.injury_id,
                "acquired_day": injury.acquired_day,
                "heal_remaining": injury.heal_remaining,
                "treated": injury.treated,
            }
            for injury in world.captain.injuries
        ]
    gear = world.captain.combat_gear
    if gear.armor:
        captain_view["armor"] = gear.armor
    if gear.melee_weapon:
        captain_view["melee_weapon"] = gear.melee_weapon
    if gear.firearm:
        captain_view["firearm"] = gear.firearm
    if gear.weapon_quality:
        captain_view["weapon_quality"] = dict(gear.weapon_quality)
    if gear.weapon_usage:
        captain_view["weapon_usage"] = dict(gear.weapon_usage)
    if gear.weapon_provenance:
        captain_view["weapon_provenance"] = {
            key: provenance_snap(prov)
            for key, prov in gear.weapon_provenance.items()
        }
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
    infra = _infra_view(state["infra"])
    if infra is not None:
        logged = snap.pop("log")
        snap["infrastructure"] = infra
        snap["log"] = logged
    if milestones:
        logged = snap.pop("log")
        snap["milestones"] = milestones
        snap["log"] = logged
    return snap


def credit_rank(tier: str) -> int:
    return {
        "merchant_line": 1,
        "house_credit": 2,
        "premier_commercial": 3,
    }.get(tier, 0)


def insurance_region(world) -> str:
    if world.voyage.status == VoyageStatus.AT_SEA:
        dest = world.ports.get(world.voyage.destination_id)
        return dest.region if dest else "Mediterranean"
    port = current_port(world)
    return port.region if port else "Mediterranean"


def apply_upkeep(state) -> list[str]:
    """Infrastructure and credit ticks, before the in-port or at-sea split."""
    world = state["world"]
    infra = state["infra"]
    notes: list[str] = []
    for message in tick_infrastructure(infra, world.captain, world.day):
        notes.append(message)
        if "seized" in message.lower():
            resolve_claim(infra, world.captain, "cargo_damage", 100, world.day)
    for message in tick_credit(infra, world.captain, world.day):
        notes.append(message)
        if "DEFAULT" in message:
            world.captain.standing.commercial_trust = max(
                0, world.captain.standing.commercial_trust - 15
            )
    return notes


def settle_event_insurance(state, event, destination: str) -> None:
    world = state["world"]
    incident = event.event_type.value
    if event.hull_delta < 0:
        resolve_claim(
            state["infra"],
            world.captain,
            incident,
            abs(event.hull_delta) * 3,
            world.day,
            voyage_destination=destination,
        )
    if event.cargo_lost:
        for good_id, qty in event.cargo_lost.items():
            good = GOODS.get(good_id)
            if good is None:
                continue
            category = good.category.value if hasattr(good.category, "value") else str(good.category)
            resolve_claim(
                state["infra"],
                world.captain,
                incident,
                good.base_price * qty,
                world.day,
                cargo_category=category,
                voyage_destination=destination,
            )


def _engine_result(result) -> None:
    if isinstance(result, str):
        raise ScriptError(result)


def do_buy_infrastructure(state, kind: str, args: list[str]) -> None:
    if kind == "warehouse":
        if not args:
            raise ScriptError(
                "Usage: buy_infrastructure warehouse <tier> | broker <region> <tier> | license <id> | dry_dock [ship]"
            )
        do_lease_warehouse(state, args[0])
    elif kind == "broker":
        if len(args) < 2:
            raise ScriptError(
                "Usage: buy_infrastructure warehouse <tier> | broker <region> <tier> | license <id> | dry_dock [ship]"
            )
        do_open_broker(state, args[0], args[1])
    elif kind == "license":
        if not args:
            raise ScriptError(
                "Usage: buy_infrastructure warehouse <tier> | broker <region> <tier> | license <id> | dry_dock [ship]"
            )
        do_purchase_license(state, args[0])
    elif kind == "dry_dock":
        do_dry_dock(state, args[0] if args else None)
    else:
        raise ScriptError(
            "Usage: buy_infrastructure warehouse <tier> | broker <region> <tier> | license <id> | dry_dock [ship]"
        )


def do_lease_warehouse(state, tier: str) -> None:
    world = state["world"]
    port = current_port(world)
    if port is None:
        raise ScriptError("Must be docked to lease a warehouse")
    try:
        spec = get_tier_spec(WarehouseTier(tier))
    except ValueError as exc:
        raise ScriptError(f"Unknown warehouse tier: {tier}") from exc
    offered = {item.tier.value for item in available_tiers(port.id)}
    if tier not in offered:
        raise ScriptError(f"{tier} warehouse is not available at {port.name}")
    _engine_result(lease_warehouse(state["infra"], world.captain, port.id, spec, world.day))


def do_open_broker(state, region: str, tier: str) -> None:
    try:
        spec = get_broker_spec(region, BrokerTier(tier))
    except ValueError as exc:
        raise ScriptError(f"Unknown broker: {region} {tier}") from exc
    if spec is None:
        raise ScriptError(f"Unknown broker: {region} {tier}")
    _engine_result(open_broker_office(state["infra"], state["world"].captain, region, spec, state["world"].day))


def do_purchase_license(state, license_id: str) -> None:
    spec = get_license_spec(license_id)
    if spec is None:
        raise ScriptError(f"Unknown license: {license_id}")
    world = state["world"]
    _engine_result(purchase_license(state["infra"], world.captain, spec, world.captain.standing, world.day))


def _dry_dock_ship(world, ship, port, missing: str) -> None:
    """`GameSession._do_dry_dock`. `missing` is the flagship's existing sentence."""
    template = SHIPS.get(ship.template_id)
    if template is None:
        raise ScriptError(missing)
    degradation = template.hull_max - ship.hull_max
    if degradation <= 0:
        raise ScriptError("Ship hull is not degraded")
    service = get_service_modifier(world.captain.standing, port.id)
    cost_per = max(1, int(port.repair_cost * service * 5))
    cost = degradation * cost_per
    if cost > world.captain.silver:
        raise ScriptError(
            f"Need {cost} silver for dry dock ({degradation} points at {cost_per}/point), have {world.captain.silver}"
        )
    world.captain.silver -= cost
    ship.hull_max = template.hull_max
    ship.hull = min(ship.hull + degradation, ship.hull_max)


def do_dry_dock(state, ship_name: str | None) -> None:
    world = state["world"]
    port = current_port(world)
    if port is None:
        raise ScriptError("Must be docked")
    if PortFeature.SHIPYARD not in port.features:
        raise ScriptError(f"{port.name} has no shipyard")
    if ship_name:
        wanted = ship_name.lower()
        for owned in world.captain.fleet:
            if owned.docked_port_id == port.id and (
                owned.ship.name.lower() == wanted
                or owned.ship.template_id.lower() == wanted
            ):
                _dry_dock_ship(world, owned.ship, port, "Unknown ship template")
                return
        raise ScriptError(f"No ship named '{ship_name}' docked at this port")
    ship = world.captain.ship
    if ship is None:
        raise ScriptError("No ship")
    _dry_dock_ship(world, ship, port, f"Unknown ship: {ship.template_id}")


def do_take_credit(state, tier: str, amount: int) -> None:
    world = state["world"]
    if tier == "emergency":
        _engine_result(emergency_loan(world.captain, amount))
        return
    try:
        spec = get_credit_spec(CreditTier(tier))
    except ValueError as exc:
        raise ScriptError(f"Unknown credit tier: {tier}") from exc
    if spec is None:
        raise ScriptError(f"Unknown credit tier: {tier}")
    credit = state["infra"].credit
    if credit is None:
        current_rank, active, current_name = 0, False, "none"
    else:
        current_rank = credit_rank(credit.tier.value)
        active = credit.active
        current_name = credit.tier.value
    requested = credit_rank(tier)
    if active and current_rank > requested:
        raise ScriptError(f"Already have {current_name} or better")
    if not active or current_rank < requested:
        err = open_credit_line(state["infra"], spec, world.captain.standing, world.day)
        if err:
            raise ScriptError(err)
    elif amount == 0:
        raise ScriptError(f"Already have {current_name} or better")
    if amount > 0:
        err = draw_credit(state["infra"], world.captain, amount)
        if err:
            raise ScriptError(err)


def do_buy_insurance(state, policy_id: str, target: str, origin: str, destination: str) -> None:
    spec = get_policy_spec(policy_id)
    if spec is None:
        raise ScriptError(f"Unknown policy: {policy_id}")
    world = state["world"]
    region = insurance_region(world)
    heat = world.captain.standing.customs_heat.get(region, 0)
    _engine_result(purchase_policy(
        state["infra"],
        world.captain,
        spec,
        world.day,
        heat=heat,
        target_id=target,
        voyage_origin=origin,
        voyage_destination=destination,
    ))


def do_deposit(state, good_id: str, qty: int) -> None:
    world = state["world"]
    port = current_port(world)
    if port is None:
        raise ScriptError("Must be docked to deposit")
    _engine_result(deposit_cargo(state["infra"], port.id, world.captain, good_id, qty, world.day))


def do_withdraw(state, good_id: str, qty: int, source: str | None) -> None:
    world = state["world"]
    port = current_port(world)
    if port is None:
        raise ScriptError("Must be docked to withdraw")
    _engine_result(withdraw_cargo(state["infra"], port.id, world.captain, good_id, qty, source))


def do_repay_credit(state, amount: int) -> None:
    err = repay_credit(state["infra"], state["world"].captain, amount)
    if err:
        raise ScriptError(err)


def _credit_visible(credit) -> bool:
    if credit is None:
        return False
    return bool(
        credit.active
        or credit.outstanding
        or credit.interest_accrued
        or credit.defaults
        or credit.total_borrowed
        or credit.total_repaid
    )


def _infra_view(infra) -> dict | None:
    if not (
        infra.warehouses
        or infra.brokers
        or infra.licenses
        or infra.policies
        or infra.claims
        or _credit_visible(infra.credit)
    ):
        return None
    view = {
        "warehouses": [
            {
                "id": lease.id,
                "port_id": lease.port_id,
                "tier": lease.tier.value,
                "capacity": lease.capacity,
                "lease_cost": lease.lease_cost,
                "upkeep_per_day": lease.upkeep_per_day,
                "inventory": [
                    {
                        "good_id": lot.good_id,
                        "quantity": lot.quantity,
                        "acquired_port": lot.acquired_port,
                        "acquired_region": lot.acquired_region,
                        "acquired_day": lot.acquired_day,
                        "deposited_day": lot.deposited_day,
                    }
                    for lot in lease.inventory
                ],
                "opened_day": lease.opened_day,
                "upkeep_paid_through": lease.upkeep_paid_through,
                "active": lease.active,
            }
            for lease in infra.warehouses
        ],
        "brokers": [
            {
                "region": broker.region,
                "tier": broker.tier.value,
                "opened_day": broker.opened_day,
                "upkeep_paid_through": broker.upkeep_paid_through,
                "active": broker.active,
            }
            for broker in infra.brokers
        ],
        "licenses": [
            {
                "license_id": lic.license_id,
                "purchased_day": lic.purchased_day,
                "upkeep_paid_through": lic.upkeep_paid_through,
                "active": lic.active,
            }
            for lic in infra.licenses
        ],
        "policies": [
            {
                "id": policy.id,
                "spec_id": policy.spec_id,
                "family": policy.family.value,
                "scope": policy.scope.value,
                "purchased_day": policy.purchased_day,
                "coverage_pct": policy.coverage_pct,
                "coverage_cap": policy.coverage_cap,
                "premium_paid": policy.premium_paid,
                "target_id": policy.target_id,
                "claims_made": policy.claims_made,
                "total_paid_out": policy.total_paid_out,
                "active": policy.active,
                "voyage_origin": policy.voyage_origin,
                "voyage_destination": policy.voyage_destination,
            }
            for policy in infra.policies
        ],
        "claims": [
            {
                "policy_id": claim.policy_id,
                "day": claim.day,
                "incident_type": claim.incident_type,
                "loss_value": claim.loss_value,
                "payout": claim.payout,
                "denied": claim.denied,
                "denial_reason": claim.denial_reason,
            }
            for claim in infra.claims
        ],
    }
    if _credit_visible(infra.credit):
        credit = infra.credit
        view["credit"] = {
            "tier": credit.tier.value,
            "credit_limit": credit.credit_limit,
            "outstanding": credit.outstanding,
            "interest_accrued": credit.interest_accrued,
            "last_interest_day": credit.last_interest_day,
            "next_due_day": credit.next_due_day,
            "defaults": credit.defaults,
            "total_borrowed": credit.total_borrowed,
            "total_repaid": credit.total_repaid,
            "active": credit.active,
        }
    return view


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
