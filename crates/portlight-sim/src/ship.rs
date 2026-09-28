//! Ship stat resolution and crew effects used by trade and voyage.
//!
//! Installed upgrades add to the stored hull, speed, cannons, and the rest.
//! Callers that pass a ship into naval math use [`resolved_ship`], which is a
//! copy. Hull damage stays on the real ship.

use crate::content;
use crate::model::Ship;
use crate::pyrand::PyRandom;
use crate::util::py_trunc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CrewRole {
    Sailor,
    Gunner,
    Navigator,
    Surgeon,
    Marine,
    Quartermaster,
}

fn upgrade_defs(ship: &Ship) -> Vec<&'static content::UpgradeDef> {
    let catalog = content::content();
    ship.upgrades
        .iter()
        .filter_map(|installed| catalog.upgrade(&installed.upgrade_id))
        .collect()
}

pub fn resolve_speed(ship: &Ship) -> f64 {
    let mut bonus = 0.0;
    let mut penalty = 0.0;
    for upgrade in upgrade_defs(ship) {
        bonus += upgrade.speed_bonus;
        penalty += upgrade.speed_penalty;
    }
    (ship.speed + bonus - penalty).max(0.5)
}

pub fn resolve_hull_max(ship: &Ship) -> i64 {
    let bonus: i64 = upgrade_defs(ship)
        .iter()
        .map(|upgrade| upgrade.hull_max_bonus)
        .sum();
    ship.hull_max + bonus
}

pub fn resolve_cargo_capacity(ship: &Ship) -> i64 {
    let bonus: i64 = upgrade_defs(ship)
        .iter()
        .map(|upgrade| upgrade.cargo_bonus)
        .sum();
    ship.cargo_capacity + bonus
}

pub fn resolve_cannons(ship: &Ship) -> i64 {
    let bonus: i64 = upgrade_defs(ship)
        .iter()
        .map(|upgrade| upgrade.cannon_bonus)
        .sum();
    ship.cannons + bonus
}

pub fn resolve_maneuver(ship: &Ship) -> f64 {
    let bonus: f64 = upgrade_defs(ship)
        .iter()
        .map(|upgrade| upgrade.maneuver_bonus)
        .sum();
    (ship.maneuver + bonus).clamp(0.0, 1.0)
}

pub fn resolve_storm_resist(ship: &Ship) -> f64 {
    let base = content::content()
        .ship(&ship.template_id)
        .map(|s| s.storm_resist)
        .unwrap_or(0.0);
    let bonus: f64 = upgrade_defs(ship)
        .iter()
        .map(|upgrade| upgrade.storm_resist_bonus)
        .sum();
    (base + bonus).min(0.9)
}

pub fn resolve_crew_max(ship: &Ship) -> i64 {
    let bonus: i64 = upgrade_defs(ship)
        .iter()
        .map(|upgrade| upgrade.crew_max_bonus)
        .sum();
    ship.crew_max + bonus
}

pub fn has_special(ship: &Ship, special: &str) -> bool {
    upgrade_defs(ship)
        .iter()
        .any(|upgrade| upgrade.special == special)
}

/// Copy with effective stats. The stored hull, speed, and cannons stay put.
pub fn resolved_ship(ship: &Ship) -> Ship {
    let mut copy = ship.clone();
    copy.hull_max = resolve_hull_max(ship);
    copy.cargo_capacity = resolve_cargo_capacity(ship);
    copy.speed = resolve_speed(ship);
    copy.crew_max = resolve_crew_max(ship);
    copy.cannons = resolve_cannons(ship);
    copy.maneuver = resolve_maneuver(ship);
    copy
}

pub fn template_daily_wage(ship: &Ship) -> i64 {
    content::content()
        .ship(&ship.template_id)
        .map(|s| s.daily_wage)
        .unwrap_or(1)
}

pub fn template_crew_min(ship: &Ship) -> i64 {
    content::content()
        .ship(&ship.template_id)
        .map(|s| s.crew_min)
        .unwrap_or(1)
}

pub fn navigator_speed_bonus(ship: &Ship) -> f64 {
    if ship.navigators >= 1 {
        0.5
    } else {
        0.0
    }
}

pub fn compute_daily_wages(ship: &Ship) -> i64 {
    let wage = template_daily_wage(ship);
    let base = ship.sailors * wage.max(1)
        + ship.gunners * wage.max(2)
        + ship.navigators * wage.max(3)
        + ship.surgeons * wage.max(3)
        + ship.marines * wage.max(2)
        + ship.quartermasters * wage.max(2);
    if ship.quartermasters >= 1 {
        py_trunc(base as f64 * 0.90)
    } else {
        base
    }
}

pub fn wage_bill(ship: &Ship) -> i64 {
    if ship.roster_total() > 0 {
        compute_daily_wages(ship)
    } else {
        template_daily_wage(ship) * ship.crew
    }
}

pub fn morale_speed_modifier(morale: i64) -> f64 {
    if morale > 70 {
        1.10
    } else if morale < 30 {
        0.80
    } else {
        1.0
    }
}

pub fn tick_morale_at_sea(
    ship: &Ship,
    wages_paid: bool,
    provisions_ok: bool,
    days_since_port: i64,
) -> i64 {
    let mut delta = if wages_paid { 1 } else { -5 };
    if !provisions_ok {
        delta -= 3;
    }
    if days_since_port > 10 {
        delta -= 1;
    }
    (ship.morale + delta).clamp(0, 100)
}

pub fn tick_morale_at_port(ship: &Ship, has_officers_cabin: bool) -> i64 {
    let bonus = if has_officers_cabin { 10 } else { 5 };
    (ship.morale + bonus).clamp(0, 100)
}

fn role_count(ship: &Ship, role: CrewRole) -> i64 {
    match role {
        CrewRole::Sailor => ship.sailors,
        CrewRole::Gunner => ship.gunners,
        CrewRole::Navigator => ship.navigators,
        CrewRole::Surgeon => ship.surgeons,
        CrewRole::Marine => ship.marines,
        CrewRole::Quartermaster => ship.quartermasters,
    }
}

fn set_role(ship: &mut Ship, role: CrewRole, count: i64) {
    let count = count.max(0);
    match role {
        CrewRole::Sailor => ship.sailors = count,
        CrewRole::Gunner => ship.gunners = count,
        CrewRole::Navigator => ship.navigators = count,
        CrewRole::Surgeon => ship.surgeons = count,
        CrewRole::Marine => ship.marines = count,
        CrewRole::Quartermaster => ship.quartermasters = count,
    }
}

fn select_casualty(ship: &Ship, context: &str, rng: &mut PyRandom) -> Option<CrewRole> {
    let mut roles = Vec::new();
    let mut weights = Vec::new();
    let candidates = [
        (CrewRole::Sailor, 3.0),
        (CrewRole::Gunner, 1.5),
        (CrewRole::Navigator, 0.5),
        (CrewRole::Surgeon, 0.5),
        (CrewRole::Marine, 2.0),
        (CrewRole::Quartermaster, 0.5),
    ];
    for (role, weight) in candidates {
        if role_count(ship, role) > 0 {
            let mut weight = weight;
            if context == "boarding" && role == CrewRole::Marine {
                weight *= 2.0;
            }
            if context == "storm" && role == CrewRole::Sailor {
                weight *= 2.0;
            }
            roles.push(role);
            weights.push(weight);
        }
    }
    if roles.is_empty() {
        return None;
    }
    Some(roles[rng.choices_weighted(&weights)])
}

pub fn apply_crew_delta(ship: &mut Ship, delta: i64, context: &str, rng: &mut PyRandom) -> i64 {
    if ship.roster_total() == 0 && ship.crew > 0 {
        ship.sailors = ship.crew;
    }
    if delta == 0 {
        ship.sync_crew();
        return 0;
    }
    if delta > 0 {
        ship.sailors += delta;
        ship.sync_crew();
        return delta;
    }
    let mut removed = 0;
    for _ in 0..(-delta) {
        let Some(role) = select_casualty(ship, context, rng) else {
            break;
        };
        set_role(ship, role, role_count(ship, role) - 1);
        removed += 1;
    }
    ship.sync_crew();
    -removed
}
