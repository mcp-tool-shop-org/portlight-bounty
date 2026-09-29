#!/usr/bin/env python3
"""Write the custom-captain v12 fixture Rust loads, and check it.

`GameSession.new` registers the built template, starts seed 1, and saves.
The committed file is that JSON. `--check` writes it again in a temp
directory and requires the bytes to match, then loads the committed slot
with no custom template registered.

A reloaded custom captain reverts to merchant pricing in both games. The
save does not store the built template. `GameSession.captain_template`
raises KeyError for CaptainType.CUSTOM and returns the merchant archetype.
Rust `captain_template` does the same when `world.custom_captain` is
absent. Porto Novo porcelain is 176/151 in the file (new-game prices, no
captain modifier). Seven trade points would sell it at 194. After load
both games reprice with the merchant modifiers: buy 162, sell 159.

    PYTHONPATH=/path/to/portlight/src python3 tools/parity/custom_captain_save.py
    PYTHONPATH=/path/to/portlight/src python3 tools/parity/custom_captain_save.py --check
"""

from __future__ import annotations

import argparse
import json
import shutil
import sys
import tempfile
from pathlib import Path

from portlight.content.goods import GOODS
from portlight.content.world import new_game
from portlight.engine.captain_identity import CAPTAIN_TEMPLATES, CaptainType
from portlight.engine.custom_captain import (
    CustomCaptainSpec,
    build_custom_template,
    validate_spec,
)
from portlight.engine.economy import recalculate_prices
from portlight.app.session import GameSession


ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "parity" / "saves" / "custom_captain_v12.json"
SLOT = "custom_captain_v12"

# Porcelain at Porto Novo. On-disk prices have no captain modifier.
PORCELAIN_ON_DISK = (176, 151)
# Merchant modifiers applied by GameSession.load.
PORCELAIN_MERCHANT = (162, 159)
# The live custom template (7 trade points), which a reload does not use.
PORCELAIN_CUSTOM = (164, 194)


def spec() -> CustomCaptainSpec:
    return CustomCaptainSpec(
        name="Sable Quinn",
        title="Freelance Captain",
        home_port_id="porto_novo",
        home_region="Mediterranean",
        trade_points=7,
        sailing_points=1,
        shadow_points=1,
        reputation_points=1,
        bloc_alignment="exchange_alliance",
        faction_alignment="crimson_tide",
        mentor_npc_id="pn_marta",
        backstory="She wrote her own articles and left the quay at dusk.",
    )


def write_save(base: Path) -> bytes:
    built = spec()
    errors = validate_spec(built)
    if errors:
        raise SystemExit("\n".join(errors))
    CAPTAIN_TEMPLATES[CaptainType.CUSTOM] = build_custom_template(built)
    try:
        GameSession(base_path=base, slot=SLOT).new(
            built.name, captain_type="custom", seed=1
        )
        return (base / "saves" / f"{SLOT}.json").read_bytes()
    finally:
        CAPTAIN_TEMPLATES.pop(CaptainType.CUSTOM, None)


def porcelain(world) -> tuple[int, int]:
    for slot in world.ports["porto_novo"].market:
        if slot.good_id == "porcelain":
            return slot.buy_price, slot.sell_price
    raise SystemExit("porcelain missing at porto_novo")


def assert_loaded(base: Path) -> None:
    # No custom template is registered. Load must fall back to merchant.
    CAPTAIN_TEMPLATES.pop(CaptainType.CUSTOM, None)
    session = GameSession(base_path=base, slot=SLOT)
    if not session.load():
        raise SystemExit(f"Python could not load {base / 'saves' / (SLOT + '.json')}")
    captain = session.world.captain
    if captain.name != "Sable Quinn":
        raise SystemExit(f"name {captain.name!r}")
    if captain.captain_type != "custom":
        raise SystemExit(f"type {captain.captain_type!r}")
    if captain.silver != 500:
        raise SystemExit(f"silver {captain.silver}")
    if session.world.day != 1 or captain.day != 1:
        raise SystemExit(f"day world={session.world.day} captain={captain.day}")
    if porcelain(session.world) != PORCELAIN_MERCHANT:
        raise SystemExit(
            f"reloaded porcelain {porcelain(session.world)} is not merchant {PORCELAIN_MERCHANT}"
        )
    template = session.captain_template
    if template is None or template.id != CaptainType.MERCHANT:
        raise SystemExit(f"template did not fall back to merchant: {template}")


def assert_custom_differs() -> None:
    world = new_game("Sable Quinn", None, CaptainType.MERCHANT, seed=1)
    pricing = build_custom_template(spec()).pricing
    for port in world.ports.values():
        recalculate_prices(port, GOODS, pricing)
    if porcelain(world) != PORCELAIN_CUSTOM:
        raise SystemExit(f"custom porcelain drifted: {porcelain(world)}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--check",
        action="store_true",
        help="compare a fresh Python save to the committed fixture",
    )
    args = parser.parse_args()
    assert_custom_differs()
    if args.check:
        tmp = Path(tempfile.mkdtemp(prefix="portlight-custom-"))
        try:
            fresh = write_save(tmp)
        finally:
            shutil.rmtree(tmp, ignore_errors=True)
        committed = OUT.read_bytes()
        if fresh != committed:
            print(f"{OUT} drifted from GameSession.new", file=sys.stderr)
            return 1
        data = json.loads(committed.decode("utf-8"))
        on_disk = None
        for slot in data["ports"]["porto_novo"]["market"]:
            if slot["good_id"] == "porcelain":
                on_disk = (slot["buy_price"], slot["sell_price"])
        if on_disk != PORCELAIN_ON_DISK:
            print(f"on-disk porcelain {on_disk} != {PORCELAIN_ON_DISK}", file=sys.stderr)
            return 1
        assert_loaded(ROOT / "parity")
        print(
            f"{OUT} matches Python "
            "(Sable Quinn, custom, 500 silver, day 1; reload uses merchant pricing)"
        )
        return 0
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_bytes(write_save(ROOT / "parity"))
    print(f"wrote {OUT}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
