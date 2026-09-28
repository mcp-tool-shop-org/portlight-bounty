#!/usr/bin/env python3
"""Regenerate crates/portlight-sim/data/content.json from the Python game.

Usage (from the portlight-bounty repo root):

    PYTHONPATH=/path/to/portlight/src python3 tools/extract_content.py

The Python checkout this port was built against is commit 9b02494.
"""

from __future__ import annotations

import json
import os
import sys

from portlight.content.contracts import TEMPLATES
from portlight.content.factions import FACTIONS, PIRATE_CAPTAINS
from portlight.content.goods import GOODS
from portlight.content.ports import PORTS
from portlight.content.routes import ROUTES
from portlight.content.seasons import SEASONAL_PROFILES
from portlight.content.ships import SHIPS
from portlight.engine.captain_identity import CAPTAIN_TEMPLATES


def main() -> None:
    data = {
        "source_commit": "9b02494cca9cd8f58c4531c41f75be02c9720e43",
        "goods": [
            {
                "id": g.id,
                "name": g.name,
                "category": g.category.value,
                "base_price": g.base_price,
                "weight_per_unit": g.weight_per_unit,
            }
            for g in GOODS.values()
        ],
        "ships": [
            {
                "id": s.id,
                "name": s.name,
                "ship_class": s.ship_class.value,
                "cargo_capacity": s.cargo_capacity,
                "speed": s.speed,
                "hull_max": s.hull_max,
                "crew_min": s.crew_min,
                "crew_max": s.crew_max,
                "price": s.price,
                "daily_wage": s.daily_wage,
                "storm_resist": s.storm_resist,
                "cannons": s.cannons,
                "maneuver": s.maneuver,
            }
            for s in SHIPS.values()
        ],
        "ports": [
            {
                "id": p.id,
                "name": p.name,
                "description": p.description,
                "region": p.region,
                "features": [f.value for f in p.features],
                "market": [
                    {
                        "good_id": s.good_id,
                        "stock_current": s.stock_current,
                        "stock_target": s.stock_target,
                        "restock_rate": s.restock_rate,
                        "local_affinity": s.local_affinity,
                        "spread": s.spread,
                    }
                    for s in p.market
                ],
                "port_fee": p.port_fee,
                "provision_cost": p.provision_cost,
                "repair_cost": p.repair_cost,
                "crew_cost": p.crew_cost,
                "map_x": p.map_x,
                "map_y": p.map_y,
            }
            for p in PORTS.values()
        ],
        "routes": [
            {
                "port_a": r.port_a,
                "port_b": r.port_b,
                "distance": r.distance,
                "danger": r.danger,
                "min_ship_class": r.min_ship_class,
                "lore_name": r.lore_name,
                "lore": r.lore,
            }
            for r in ROUTES
        ],
        "captains": [
            {
                "id": c.id.value,
                "name": c.name,
                "title": c.title,
                "home_region": c.home_region,
                "home_port_id": c.home_port_id,
                "starting_silver": c.starting_silver,
                "starting_ship_id": c.starting_ship_id,
                "starting_provisions": c.starting_provisions,
                "pricing": {
                    "buy_price_mult": c.pricing.buy_price_mult,
                    "sell_price_mult": c.pricing.sell_price_mult,
                    "luxury_sell_bonus": c.pricing.luxury_sell_bonus,
                    "port_fee_mult": c.pricing.port_fee_mult,
                },
                "voyage": {
                    "provision_burn": c.voyage.provision_burn,
                    "speed_bonus": c.voyage.speed_bonus,
                    "storm_resist_bonus": c.voyage.storm_resist_bonus,
                    "cargo_damage_mult": c.voyage.cargo_damage_mult,
                },
                "inspection": {
                    "inspection_chance_mult": c.inspection.inspection_chance_mult,
                    "seizure_risk": c.inspection.seizure_risk,
                    "fine_mult": c.inspection.fine_mult,
                },
                "reputation": {
                    "commercial_trust": c.reputation_seed.commercial_trust,
                    "customs_heat": c.reputation_seed.customs_heat,
                    "mediterranean": c.reputation_seed.mediterranean,
                    "north_atlantic": c.reputation_seed.north_atlantic,
                    "west_africa": c.reputation_seed.west_africa,
                    "east_indies": c.reputation_seed.east_indies,
                    "south_seas": c.reputation_seed.south_seas,
                    "underworld": dict(c.reputation_seed.underworld or {}),
                },
            }
            for c in CAPTAIN_TEMPLATES.values()
        ],
        "seasons": [
            {
                "season": p.season.value,
                "region": p.region,
                "danger_mult": p.danger_mult,
                "speed_mult": p.speed_mult,
                "market_effects": dict(p.market_effects),
            }
            for p in SEASONAL_PROFILES.values()
        ],
        "factions": [
            {
                "id": f.id,
                "name": f.name,
                "territory_regions": list(f.territory_regions),
            }
            for f in FACTIONS.values()
        ],
        "pirate_captains": [
            {
                "id": c.id,
                "name": c.name,
                "faction_id": c.faction_id,
                "personality": c.personality,
                "strength": c.strength,
            }
            for c in PIRATE_CAPTAINS.values()
        ],
        "contracts": [
            {
                "id": t.id,
                "family": t.family.value,
                "title_pattern": t.title_pattern,
                "description": t.description,
                "goods_pool": list(t.goods_pool),
                "quantity_min": t.quantity_min,
                "quantity_max": t.quantity_max,
                "reward_per_unit": t.reward_per_unit,
                "bonus_reward": t.bonus_reward,
                "deadline_days": t.deadline_days,
                "trust_requirement": t.trust_requirement,
                "standing_requirement": t.standing_requirement,
                "heat_ceiling": t.heat_ceiling,
                "inspection_modifier": t.inspection_modifier,
                "source_region": t.source_region,
                "source_port": t.source_port,
                "destination_regions": list(t.destination_regions),
                "captain_bias": list(t.captain_bias),
                "tags": list(t.tags),
                "cultural_flavor": t.cultural_flavor,
            }
            for t in TEMPLATES
        ],
    }
    root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    path = os.path.join(root, "crates", "portlight-sim", "data", "content.json")
    with open(path, "w", encoding="utf-8") as fh:
        json.dump(data, fh, indent=2)
        fh.write("\n")
    print(f"wrote {path}")


if __name__ == "__main__":
    sys.exit(main())
