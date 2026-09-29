#!/usr/bin/env python3
"""Drive a real GameSession for the verbs it implements.

Exit 2 means the script uses a verb this runner does not send through
GameSession. The parity check skips that script. Exit 0 prints the same
snapshot shape as tools/parity/oracle.py for the session that remains.
"""

from __future__ import annotations

import json
import os
import shlex
import shutil
import sys
import tempfile
from pathlib import Path

from portlight.app.session import GameSession
from portlight.content.infrastructure import (
    BrokerTier,
    CreditTier,
    WarehouseTier,
    get_broker_spec,
    get_credit_spec,
    get_license_spec,
    get_policy_spec,
    get_tier_spec,
)
from portlight.engine.infrastructure import emergency_loan

from oracle import history_from_world, snapshot

# Verbs whose mutation goes through GameSession (or emergency_loan, which the
# session does not wrap). Anything else is skipped rather than reimplemented.
SESSION_VERBS = {
    "new",
    "buy",
    "sell",
    "depart",
    "advance",
    "hire",
    "provision",
    "work",
    "repair",
    "buy_ship",
    "upgrade",
    "buy_infrastructure",
    "deposit",
    "withdraw",
    "take_credit",
    "repay_credit",
    "buy_insurance",
    "accept_contract",
    "abandon_contract",
    "rename_ship",
    "dock_current_ship",
    "board_fleet_ship",
    "sell_fleet_ship",
    "fire",
    "transfer",
    "hunt",
    "save",
    "load",
}


class Skip(Exception):
    def __init__(self, verb: str) -> None:
        super().__init__(verb)
        self.verb = verb


class Stop(Exception):
    def __init__(self, message: str) -> None:
        super().__init__(message)
        self.message = message


def tokens_of(line: str) -> list[str]:
    return shlex.split(line)


def parse_int(token: str) -> int:
    try:
        return int(token)
    except ValueError as exc:
        raise Stop(f"Invalid number: {token}") from exc


def require(ok: bool, message: str) -> None:
    if not ok:
        raise Stop(message)


def check_result(result) -> None:
    if isinstance(result, str):
        raise Stop(result)


def credit_rank(tier: str) -> int:
    return {"merchant_line": 1, "house_credit": 2, "premier_commercial": 3}.get(tier, 0)


def do_new(session: GameSession, tokens: list[str]) -> None:
    require(len(tokens) >= 4, "Usage: new <captain_type> <name> <seed> [port]")
    seed = parse_int(tokens[3])
    port = tokens[4] if len(tokens) > 4 else None
    session.new(tokens[2], starting_port=port, captain_type=tokens[1], seed=seed)


def do_buy_infrastructure(session: GameSession, tokens: list[str]) -> None:
    usage = (
        "Usage: buy_infrastructure warehouse <tier> | broker <region> <tier> "
        "| license <id> | dry_dock [ship]"
    )
    require(len(tokens) >= 2, usage)
    kind = tokens[1]
    args = tokens[2:]
    if kind == "warehouse":
        require(len(args) >= 1, usage)
        try:
            spec = get_tier_spec(WarehouseTier(args[0]))
        except ValueError as exc:
            raise Stop(f"Unknown warehouse tier: {args[0]}") from exc
        check_result(session.lease_warehouse_cmd(spec))
    elif kind == "broker":
        require(len(args) >= 2, usage)
        try:
            spec = get_broker_spec(args[0], BrokerTier(args[1]))
        except ValueError as exc:
            raise Stop(f"Unknown broker: {args[0]} {args[1]}") from exc
        if spec is None:
            raise Stop(f"Unknown broker: {args[0]} {args[1]}")
        check_result(session.open_broker_cmd(args[0], spec))
    elif kind == "license":
        require(len(args) >= 1, usage)
        spec = get_license_spec(args[0])
        if spec is None:
            raise Stop(f"Unknown license: {args[0]}")
        check_result(session.purchase_license_cmd(spec))
    elif kind == "dry_dock":
        check_result(session.dry_dock(args[0] if args else None))
    else:
        raise Stop(usage)


def do_take_credit(session: GameSession, tokens: list[str]) -> None:
    require(len(tokens) == 3, "Usage: take_credit <tier> <amount>")
    tier = tokens[1]
    amount = parse_int(tokens[2])
    if tier == "emergency":
        result = emergency_loan(session.captain, amount)
        check_result(result)
        session._save()
        return
    try:
        spec = get_credit_spec(CreditTier(tier))
    except ValueError as exc:
        raise Stop(f"Unknown credit tier: {tier}") from exc
    if spec is None:
        raise Stop(f"Unknown credit tier: {tier}")
    credit = session.infra.credit
    if credit is None:
        current_rank, active, current_name = 0, False, "none"
    else:
        current_rank = credit_rank(credit.tier.value if hasattr(credit.tier, "value") else str(credit.tier))
        active = credit.active
        current_name = credit.tier.value if hasattr(credit.tier, "value") else str(credit.tier)
    requested = credit_rank(tier)
    if active and current_rank > requested:
        raise Stop(f"Already have {current_name} or better")
    if not active or current_rank < requested:
        check_result(session.open_credit_cmd(spec))
    elif amount == 0:
        raise Stop(f"Already have {current_name} or better")
    if amount > 0:
        check_result(session.draw_credit_cmd(amount))


def do_buy_insurance(session: GameSession, tokens: list[str]) -> None:
    require(2 <= len(tokens) <= 5, "Usage: buy_insurance <policy_id> [target_id] [origin] [destination]")
    spec = get_policy_spec(tokens[1])
    if spec is None:
        raise Stop(f"Unknown policy: {tokens[1]}")
    target = tokens[2] if len(tokens) > 2 else ""
    origin = tokens[3] if len(tokens) > 3 else ""
    destination = tokens[4] if len(tokens) > 4 else ""
    check_result(session.purchase_policy_cmd(spec, target, origin, destination))


def dispatch(session: GameSession, tokens: list[str]) -> None:
    cmd = tokens[0] if tokens else ""
    if cmd not in SESSION_VERBS:
        raise Skip(cmd or "(blank)")
    if cmd == "new":
        do_new(session, tokens)
    elif cmd == "buy":
        require(len(tokens) == 3, "Usage: buy <good> <qty>")
        check_result(session.buy(tokens[1], parse_int(tokens[2])))
    elif cmd == "sell":
        require(len(tokens) == 3, "Usage: sell <good> <qty>")
        check_result(session.sell(tokens[1], parse_int(tokens[2])))
    elif cmd == "depart":
        require(len(tokens) == 2, "Usage: depart <port_id>")
        check_result(session.sail(tokens[1]))
    elif cmd == "advance":
        require(len(tokens) == 1, "Usage: advance")
        session.advance()
    elif cmd == "hire":
        require(2 <= len(tokens) <= 3, "Usage: hire <count> [role]")
        role = tokens[2] if len(tokens) == 3 else "sailor"
        check_result(session.hire_crew(parse_int(tokens[1]), role))
    elif cmd == "provision":
        require(len(tokens) == 2, "Usage: provision <days>")
        check_result(session.provision(parse_int(tokens[1])))
    elif cmd == "work":
        require(len(tokens) == 1, "Usage: work")
        check_result(session.work())
    elif cmd == "repair":
        require(len(tokens) <= 2, "Usage: repair [points]")
        amount = parse_int(tokens[1]) if len(tokens) == 2 else None
        check_result(session.repair(amount))
    elif cmd == "buy_ship":
        require(len(tokens) == 2, "Usage: buy_ship <template_id>")
        check_result(session.buy_ship(tokens[1]))
    elif cmd == "upgrade":
        require(len(tokens) == 2, "Usage: upgrade <upgrade_id>")
        check_result(session.install_upgrade(tokens[1]))
    elif cmd == "buy_infrastructure":
        do_buy_infrastructure(session, tokens)
    elif cmd == "deposit":
        require(len(tokens) == 3, "Usage: deposit <good> <qty>")
        check_result(session.deposit_cmd(tokens[1], parse_int(tokens[2])))
    elif cmd == "withdraw":
        require(3 <= len(tokens) <= 4, "Usage: withdraw <good> <qty> [source_port]")
        source = tokens[3] if len(tokens) == 4 else None
        check_result(session.withdraw_cmd(tokens[1], parse_int(tokens[2]), source))
    elif cmd == "take_credit":
        do_take_credit(session, tokens)
    elif cmd == "repay_credit":
        require(len(tokens) == 2, "Usage: repay_credit <amount>")
        check_result(session.repay_credit_cmd(parse_int(tokens[1])))
    elif cmd == "buy_insurance":
        do_buy_insurance(session, tokens)
    elif cmd == "accept_contract":
        require(len(tokens) == 2, "Usage: accept_contract <offer_id>")
        check_result(session.accept_contract(tokens[1]))
    elif cmd == "abandon_contract":
        require(len(tokens) == 2, "Usage: abandon_contract <offer_id>")
        check_result(session.abandon_contract_cmd(tokens[1]))
    elif cmd == "rename_ship":
        require(2 <= len(tokens) <= 3, "Usage: rename_ship <new_name> [ship]")
        ship = tokens[2] if len(tokens) == 3 else None
        check_result(session.rename_ship(tokens[1], ship))
    elif cmd == "dock_current_ship":
        require(len(tokens) == 1, "Usage: dock_current_ship")
        check_result(session.dock_current_ship())
    elif cmd == "board_fleet_ship":
        require(len(tokens) == 2, "Usage: board_fleet_ship <ship>")
        check_result(session.board_fleet_ship(tokens[1]))
    elif cmd == "sell_fleet_ship":
        require(len(tokens) == 2, "Usage: sell_fleet_ship <ship>")
        check_result(session.sell_fleet_ship(tokens[1]))
    elif cmd == "fire":
        require(2 <= len(tokens) <= 3, "Usage: fire <count> [role]")
        role = tokens[2] if len(tokens) == 3 else "sailor"
        check_result(session.fire_crew(parse_int(tokens[1]), role))
    elif cmd == "transfer":
        require(len(tokens) == 5, "Usage: transfer <good> <qty> <from> <to>")
        check_result(session.transfer_fleet_cargo(tokens[1], parse_int(tokens[2]), tokens[3], tokens[4]))
    elif cmd == "hunt":
        require(len(tokens) == 1, "Usage: hunt")
        result = session.hunt()
        if isinstance(result, str):
            raise Stop(result)
    elif cmd == "save":
        require(len(tokens) == 2, "Usage: save <slot>")
        root = os.environ.get("PORTLIGHT_SAVE_ROOT")
        require(bool(root), "PORTLIGHT_SAVE_ROOT is not set")
        # GameSession._save writes self.slot after every mutation. Park later
        # autosaves on a scratch slot so they do not overwrite this one.
        session.base_path = Path(root)
        session.slot = tokens[1]
        session._save()
        session.slot = "_session"
    elif cmd == "load":
        require(len(tokens) == 2, "Usage: load <slot>")
        root = os.environ.get("PORTLIGHT_SAVE_ROOT")
        require(bool(root), "PORTLIGHT_SAVE_ROOT is not set")
        session.base_path = Path(root)
        session.slot = tokens[1]
        if not session.load():
            raise Stop(f"No save in slot {tokens[1]}")
        session.slot = "_session"


def blank_log(raw: str, error: str | None) -> dict:
    entry = {"command": raw, "error": error}
    return entry


def run(script: str, base: Path) -> dict:
    session = GameSession(base_path=base)
    log: list[dict] = []
    for line in script.splitlines():
        raw = line.strip()
        if not raw or raw.startswith("#"):
            continue
        tokens = tokens_of(raw)
        try:
            dispatch(session, tokens)
        except Skip:
            raise
        except Stop as exc:
            log.append(blank_log(raw, exc.message))
            break
        else:
            log.append(blank_log(raw, None))
    if session.world is None:
        state = {
            "world": None,
            "trade_seq": 0,
            "ledger": session.ledger,
            "board": session.board,
            "infra": session.infra,
            "campaign": session.campaign,
            "narrative": session.narrative,
        }
        return snapshot(state, log)
    state = {
        "world": session.world,
        "trade_seq": session._trade_seq,
        "ledger": session.ledger,
        "board": session.board,
        "infra": session.infra,
        "campaign": session.campaign,
        "narrative": session.narrative,
        "history": history_from_world(session.world),
    }
    return snapshot(state, log)


def main() -> int:
    if len(sys.argv) != 2:
        print("Usage: session_runner.py <script>", file=sys.stderr)
        return 2
    with open(sys.argv[1], encoding="utf-8") as fh:
        script = fh.read()
    for line in script.splitlines():
        raw = line.strip()
        if not raw or raw.startswith("#"):
            continue
        verb = shlex.split(raw)[0]
        if verb not in SESSION_VERBS:
            print(f"skip: GameSession runner has no {verb}", file=sys.stderr)
            return 2
    base = Path(tempfile.mkdtemp(prefix="portlight-session-"))
    try:
        try:
            snap = run(script, base)
        except Skip as exc:
            print(f"skip: GameSession runner has no {exc.verb}", file=sys.stderr)
            return 2
        json.dump(snap, sys.stdout, indent=2)
        sys.stdout.write("\n")
        return 0
    finally:
        shutil.rmtree(base, ignore_errors=True)


if __name__ == "__main__":
    raise SystemExit(main())
