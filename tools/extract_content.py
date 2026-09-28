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
from enum import Enum

from portlight.content.armor import ARMOR
from portlight.content.cross_port_networks import ALL_CROSS_PORT_RELATIONSHIPS
from portlight.content.contracts import TEMPLATES
from portlight.content.factions import FACTIONS, PIRATE_CAPTAINS
from portlight.content.fighting_styles import FIGHTING_STYLES
from portlight.content.goods import GOODS
from portlight.content.infrastructure import (
    BROKER_SPECS,
    CREDIT_TIERS,
    LICENSE_CATALOG,
    POLICY_CATALOG,
    PORT_WAREHOUSE_TIERS,
    WAREHOUSE_TIERS,
)
from portlight.content.melee_weapons import MELEE_WEAPONS
from portlight.content.port_institutions import PORT_INSTITUTIONAL_PROFILES
from portlight.content.port_institutions_east import EAST_PROFILES
from portlight.content.ports import PORTS
from portlight.content.ranged_weapons import RANGED_WEAPONS
from portlight.content.routes import ROUTES
from portlight.content.seasons import SEASONAL_PROFILES
from portlight.content.ships import SHIPS
from portlight.engine.captain_identity import CAPTAIN_TEMPLATES


def _plain(value):
    """JSON-ready view of a dataclass, enum, or container."""
    if isinstance(value, Enum):
        return value.value
    if hasattr(value, "__dataclass_fields__"):
        return {name: _plain(getattr(value, name)) for name in value.__dataclass_fields__}
    if isinstance(value, dict):
        return {
            _plain(key) if not isinstance(key, str) else key: _plain(item)
            for key, item in value.items()
        }
    if isinstance(value, (list, tuple)):
        return [_plain(item) for item in value]
    return value


def _infrastructure():
    brokers = []
    for (region, _tier), spec in BROKER_SPECS.items():
        row = _plain(spec)
        row["region"] = region
        brokers.append(row)
    return {
        "warehouse_tiers": [_plain(spec) for spec in WAREHOUSE_TIERS.values()],
        "port_warehouse_tiers": {
            port_id: [tier.value for tier in tiers]
            for port_id, tiers in PORT_WAREHOUSE_TIERS.items()
        },
        "brokers": brokers,
        "licenses": [_plain(spec) for spec in LICENSE_CATALOG.values()],
        "policies": [_plain(spec) for spec in POLICY_CATALOG.values()],
        "credit_tiers": [_plain(spec) for spec in CREDIT_TIERS.values()],
    }


def _profiles(profiles):
    return [_plain(profile) for profile in profiles.values()]


def _networks():
    return [_plain(rel) for rel in ALL_CROSS_PORT_RELATIONSHIPS]


def skills_payload() -> dict:
    from portlight.content.skills import BLACKSMITH_EFFECTS, SKILLS, SKILL_TRAINERS

    return {
        "skills": [
            {
                "id": skill.id,
                "name": skill.name,
                "description": skill.description,
                "training_port_feature": skill.training_port_feature,
                "max_level": skill.max_level,
                "levels": [
                    {
                        "level": level.level,
                        "name": level.name,
                        "silver_cost": level.silver_cost,
                        "training_days": level.training_days,
                        "description": level.description,
                    }
                    for level in skill.levels
                ],
            }
            for skill in SKILLS.values()
        ],
        "trainers": [
            {
                "id": trainer.id,
                "name": trainer.name,
                "skill_id": trainer.skill_id,
                "port_id": trainer.port_id,
                "max_teach_level": trainer.max_teach_level,
                "description": trainer.description,
                "dialog": trainer.dialog,
            }
            for trainer in SKILL_TRAINERS.values()
        ],
        "blacksmith_effects": [
            {"level": level, **effects}
            for level, effects in BLACKSMITH_EFFECTS.items()
        ],
    }


def companions_payload() -> dict:
    from portlight.content.companions import (
        COMPANIONS,
        MORALE_REACTIONS,
        PERSONALITY_MODIFIERS,
        ROLES,
    )

    return {
        "roles": [
            {
                "id": role.id,
                "name": role.name,
                "description": role.description,
                "combat_damage_bonus": role.combat_damage_bonus,
                "combat_interception_chance": role.combat_interception_chance,
                "speed_bonus": role.speed_bonus,
                "danger_reduction": role.danger_reduction,
                "heal_rate_bonus": role.heal_rate_bonus,
                "inspection_evasion": role.inspection_evasion,
                "trade_bonus": role.trade_bonus,
            }
            for role in ROLES.values()
        ],
        "companions": [
            {
                "id": comp.id,
                "name": comp.name,
                "role_id": comp.role_id,
                "home_port_id": comp.home_port_id,
                "region": comp.region,
                "description": comp.description,
                "personality": comp.personality,
                "hire_cost": comp.hire_cost,
                "required_standing": comp.required_standing,
                "greeting": comp.greeting,
                "hire_dialog": comp.hire_dialog,
                "loyalty_line": comp.loyalty_line,
                "warning_line": comp.warning_line,
                "departure_line": comp.departure_line,
            }
            for comp in COMPANIONS.values()
        ],
        "morale_reactions": [
            {"trigger": trigger, "deltas": dict(deltas)}
            for trigger, deltas in MORALE_REACTIONS.items()
        ],
        "personality_modifiers": [
            {"personality": personality, "deltas": dict(deltas)}
            for personality, deltas in PERSONALITY_MODIFIERS.items()
        ],
    }


def merchants_payload() -> list:
    from portlight.content.merchants import MERCHANTS

    return [
        {
            "id": merchant.id,
            "name": merchant.name,
            "port_id": merchant.port_id,
            "title": merchant.title,
            "personality": merchant.personality,
            "description": merchant.description,
            "greeting": merchant.greeting,
            "inventory_types": list(merchant.inventory_types),
            "price_markup": merchant.price_markup,
        }
        for merchant in MERCHANTS.values()
    ]


def officer_names_payload() -> dict:
    from portlight.content import officer_names

    return {
        "regions": [
            {"region": region, "names": list(pool)}
            for region, pool in officer_names._NAMES.items()
        ],
        "traits": list(officer_names._TRAITS),
    }


def style_masters_payload() -> list:
    from portlight.content.fighting_styles import STYLE_MASTERS

    return [
        {
            "id": master.id,
            "name": master.name,
            "style_id": master.style_id,
            "port_id": master.port_id,
            "description": master.description,
            "dialog": master.dialog,
        }
        for master in STYLE_MASTERS.values()
    ]


def campaign_payload() -> dict:
    from portlight.content.campaign import MILESTONE_SPECS, PROFILE_MILESTONE_FAMILIES

    return {
        "milestones": [
            {
                "id": spec.id,
                "name": spec.name,
                "family": spec.family.value,
                "description": spec.description,
                "evaluator": spec.evaluator,
            }
            for spec in MILESTONE_SPECS
        ],
        "profile_milestone_families": [
            {"tag": tag, "families": list(families)}
            for tag, families in PROFILE_MILESTONE_FAMILIES.items()
        ],
    }


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
                "smuggler_attitude": f.smuggler_attitude,
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
        "armor": [
            {
                "id": a.id,
                "name": a.name,
                "armor_type": a.armor_type,
                "damage_reduction": a.damage_reduction,
                "dodge_penalty": a.dodge_penalty,
                "stamina_penalty": a.stamina_penalty,
                "silver_cost": a.silver_cost,
                "available_regions": list(a.available_regions),
                "description": a.description,
            }
            for a in ARMOR.values()
        ],
        "melee_weapons": [
            {
                "id": w.id,
                "name": w.name,
                "weapon_class": w.weapon_class,
                "damage_bonus": w.damage_bonus,
                "thrust_bonus": w.thrust_bonus,
                "slash_bonus": w.slash_bonus,
                "silver_cost": w.silver_cost,
                "available_regions": list(w.available_regions),
                "description": w.description,
                "compatible_styles": list(w.compatible_styles),
                "speed_mod": w.speed_mod,
            }
            for w in MELEE_WEAPONS.values()
        ],
        "ranged_weapons": [
            {
                "id": w.id,
                "name": w.name,
                "weapon_type": w.weapon_type,
                "damage_min": w.damage_min,
                "damage_max": w.damage_max,
                "accuracy": w.accuracy,
                "reload_turns": w.reload_turns,
                "silver_cost": w.silver_cost,
                "ammo_per_purchase": w.ammo_per_purchase,
                "available_regions": list(w.available_regions),
                "description": w.description,
                "stun_turns": w.stun_turns,
                "loud": w.loud,
            }
            for w in RANGED_WEAPONS.values()
        ],
        "fighting_styles": [
            {
                "id": s.id,
                "name": s.name,
                "region": s.region,
                "description": s.description,
                "historical_note": s.historical_note,
                "passive_thrust_bonus": s.passive_thrust_bonus,
                "passive_slash_bonus": s.passive_slash_bonus,
                "passive_parry_bonus": s.passive_parry_bonus,
                "passive_hp_bonus": s.passive_hp_bonus,
                "passive_dodge_counter": s.passive_dodge_counter,
                "passive_ranged_accuracy": s.passive_ranged_accuracy,
                "passive_injury_bonus": s.passive_injury_bonus,
                "special_action": None
                if s.special_action is None
                else {
                    "id": s.special_action.id,
                    "name": s.special_action.name,
                    "action_type": s.special_action.action_type,
                    "stamina_cost": s.special_action.stamina_cost,
                    "beats": list(s.special_action.beats),
                    "loses_to": list(s.special_action.loses_to),
                    "damage_bonus": s.special_action.damage_bonus,
                    "flavor": s.special_action.flavor,
                    "cooldown": s.special_action.cooldown,
                },
                "training_port_ids": list(s.training_port_ids),
                "silver_cost": s.silver_cost,
                "training_days": s.training_days,
                "prerequisite_styles": s.prerequisite_styles,
                "required_body_parts": list(s.required_body_parts),
            }
            for s in FIGHTING_STYLES.values()
        ],
        "skills": skills_payload(),
        "companions": companions_payload(),
        "merchants": merchants_payload(),
        "officer_names": officer_names_payload(),
        "style_masters": style_masters_payload(),
        "campaign": campaign_payload(),
        "infrastructure": _infrastructure(),
        "port_institutions": _profiles(
            {
                port_id: profile
                for port_id, profile in PORT_INSTITUTIONAL_PROFILES.items()
                if port_id not in EAST_PROFILES
            }
        ),
        "port_institutions_east": _profiles(EAST_PROFILES),
        "cross_port_networks": _networks(),
    }
    root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    path = os.path.join(root, "crates", "portlight-sim", "data", "content.json")
    with open(path, "w", encoding="utf-8") as fh:
        json.dump(data, fh, indent=2)
        fh.write("\n")
    print(f"wrote {path}")


if __name__ == "__main__":
    sys.exit(main())
