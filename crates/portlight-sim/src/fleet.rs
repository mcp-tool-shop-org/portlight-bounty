//! Fleet hulls, convoy formation, and cargo shared between ships.
//! Port of `engine/fleet.py`, plus the convoy steps in `engine/voyage.py`.

use crate::economy::{cargo_quantity, cargo_weight, consume_cargo_fifo, item_weight};
use crate::model::{Captain, CargoItem, FleetShip, Ship};
use crate::ship::{self, resolve_cargo_capacity, resolve_speed, resolve_storm_resist};
use crate::util::py_trunc;

/// Ships docked at `port_id` join the voyage. Empty `docked_port_id` means
/// in transit. Calling this twice does not undock anyone.
pub fn form_convoy(captain: &mut Captain, port_id: &str) {
    for owned in &mut captain.fleet {
        if owned.docked_port_id == port_id {
            owned.docked_port_id.clear();
        }
    }
}

/// Slowest in-transit escort, applied after the flagship's crew penalty and
/// before morale and the season. Escorts ignore crew and morale.
pub fn convoy_speed(captain: &Captain, flagship_speed: f64) -> f64 {
    let mut speed = flagship_speed;
    for owned in captain
        .fleet
        .iter()
        .filter(|owned| owned.docked_port_id.is_empty())
    {
        speed = speed.min(resolve_speed(&owned.ship));
    }
    speed
}

/// Storm or other negative route `hull_delta` hits every escort.
/// `dmg = max(1, int(abs(hull_delta) * (1 - their storm resist)))`.
pub fn damage_convoy(captain: &mut Captain, hull_delta: i64) {
    if hull_delta >= 0 {
        return;
    }
    let raw = hull_delta.abs();
    for owned in captain
        .fleet
        .iter_mut()
        .filter(|owned| owned.docked_port_id.is_empty())
    {
        let resist = resolve_storm_resist(&owned.ship);
        let dmg = 1.max(py_trunc(raw as f64 * (1.0 - resist)));
        owned.ship.hull = 0.max(owned.ship.hull - dmg);
    }
}

/// Hull-max wear on in-transit escorts. Floor 20, same as the flagship.
pub fn wear_convoy(captain: &mut Captain) {
    for owned in captain
        .fleet
        .iter_mut()
        .filter(|owned| owned.docked_port_id.is_empty())
    {
        if owned.ship.hull_max > 20 {
            owned.ship.hull_max -= 1;
            owned.ship.hull = owned.ship.hull.min(owned.ship.hull_max);
        }
    }
}

pub fn dock_convoy(captain: &mut Captain, destination_id: &str) {
    for owned in &mut captain.fleet {
        if owned.docked_port_id.is_empty() {
            owned.docked_port_id = destination_id.to_string();
        }
    }
}

pub fn fleet_daily_wages(captain: &Captain) -> i64 {
    captain
        .fleet
        .iter()
        .map(|owned| ship::template_daily_wage(&owned.ship) * owned.ship.crew)
        .sum()
}

pub fn dock_flagship(captain: &mut Captain, port_id: &str) -> Result<(), String> {
    if captain.ship.is_none() {
        return Err("No ship to dock".to_string());
    }
    let Some(idx) = captain
        .fleet
        .iter()
        .position(|owned| owned.docked_port_id == port_id)
    else {
        return Err("No other ship at this port to switch to".to_string());
    };
    swap_flagship(captain, idx, port_id);
    Ok(())
}

pub fn board_ship(captain: &mut Captain, ship_name: &str, port_id: &str) -> Result<(), String> {
    if captain.ship.is_none() {
        return Err("No active ship".to_string());
    }
    let needle = ship_name.to_lowercase();
    let Some(idx) = captain.fleet.iter().position(|owned| {
        owned.docked_port_id == port_id
            && (owned.ship.name.to_lowercase() == needle
                || owned.ship.template_id.to_lowercase() == needle)
    }) else {
        return Err(format!("No ship named '{ship_name}' docked at this port"));
    };
    swap_flagship(captain, idx, port_id);
    Ok(())
}

fn swap_flagship(captain: &mut Captain, idx: usize, port_id: &str) {
    let flagship = captain.ship.take().expect("flagship");
    let cargo = std::mem::take(&mut captain.cargo);
    let promoted = std::mem::replace(
        &mut captain.fleet[idx],
        FleetShip {
            ship: flagship,
            docked_port_id: port_id.to_string(),
            cargo,
        },
    );
    captain.ship = Some(promoted.ship);
    captain.cargo = promoted.cargo;
}

pub fn transfer_cargo(
    captain: &mut Captain,
    good_id: &str,
    qty: i64,
    from_ship_name: &str,
    to_ship_name: &str,
    port_id: &str,
) -> Result<(), String> {
    let src_flag = is_flagship(captain, from_ship_name);
    let dst_flag = is_flagship(captain, to_ship_name);
    if !src_flag && find_fleet(captain, from_ship_name, port_id).is_none() {
        return Err(format!("Ship '{from_ship_name}' not found at this port"));
    }
    if !dst_flag && find_fleet(captain, to_ship_name, port_id).is_none() {
        return Err(format!("Ship '{to_ship_name}' not found at this port"));
    }
    let avail = if src_flag {
        cargo_quantity(&captain.cargo, good_id)
    } else {
        let idx = find_fleet(captain, from_ship_name, port_id).unwrap();
        cargo_quantity(&captain.fleet[idx].cargo, good_id)
    };
    if avail < qty {
        return Err(format!(
            "Only {avail} units of {good_id} on {from_ship_name}"
        ));
    }
    let (dst_weight, dst_cap) = if dst_flag {
        let ship = captain.ship.as_ref().expect("flagship");
        (cargo_weight(&captain.cargo), resolve_cargo_capacity(ship))
    } else {
        let idx = find_fleet(captain, to_ship_name, port_id).unwrap();
        (
            cargo_weight(&captain.fleet[idx].cargo),
            resolve_cargo_capacity(&captain.fleet[idx].ship),
        )
    };
    let added = item_weight(good_id, qty);
    if dst_weight + added > dst_cap as f64 {
        let space = dst_cap as f64 - dst_weight;
        return Err(format!(
            "Only {} cargo space on {to_ship_name}",
            py_float(space)
        ));
    }
    let slices = if src_flag {
        consume_cargo_fifo(&mut captain.cargo, good_id, qty)
    } else {
        let idx = find_fleet(captain, from_ship_name, port_id).unwrap();
        consume_cargo_fifo(&mut captain.fleet[idx].cargo, good_id, qty)
    };
    let dst_cargo = if dst_flag {
        &mut captain.cargo
    } else {
        let idx = find_fleet(captain, to_ship_name, port_id).unwrap();
        &mut captain.fleet[idx].cargo
    };
    for slice in slices {
        if let Some(existing) = dst_cargo
            .iter_mut()
            .find(|item| item.good_id == good_id && item.acquired_port == slice.acquired_port)
        {
            existing.quantity += slice.quantity;
            existing.cost_basis += slice.cost_basis;
            existing.acquired_day = existing.acquired_day.max(slice.acquired_day);
        } else {
            dst_cargo.push(CargoItem {
                good_id: good_id.to_string(),
                quantity: slice.quantity,
                cost_basis: slice.cost_basis,
                acquired_port: slice.acquired_port,
                acquired_region: slice.acquired_region,
                acquired_day: slice.acquired_day,
            });
        }
    }
    Ok(())
}

pub fn sell_docked_ship(
    captain: &mut Captain,
    ship_name: &str,
    port_id: &str,
) -> Result<(i64, String), String> {
    let Some(idx) = find_fleet(captain, ship_name, port_id) else {
        return Err(format!("No ship named '{ship_name}' docked at this port"));
    };
    if !captain.fleet[idx].cargo.is_empty() {
        return Err("Ship has cargo — transfer it first".to_string());
    }
    let template = content_price(&captain.fleet[idx].ship);
    let hull = captain.fleet[idx].ship.hull;
    let hull_max = 1.max(captain.fleet[idx].ship.hull_max);
    let sale = py_trunc(template as f64 * 0.3 * (hull as f64 / hull_max as f64));
    let sold_name = captain.fleet[idx].ship.name.clone();
    captain.silver += sale;
    captain.fleet.remove(idx);
    Ok((sale, sold_name))
}

fn content_price(ship: &Ship) -> i64 {
    crate::content::content()
        .ship(&ship.template_id)
        .map(|template| template.price)
        .unwrap_or(0)
}

fn is_flagship(captain: &Captain, name: &str) -> bool {
    let needle = name.to_lowercase();
    captain.ship.as_ref().is_some_and(|ship| {
        ship.name.to_lowercase() == needle || ship.template_id.to_lowercase() == needle
    })
}

fn find_fleet(captain: &Captain, name: &str, port_id: &str) -> Option<usize> {
    let needle = name.to_lowercase();
    captain.fleet.iter().position(|owned| {
        owned.docked_port_id == port_id
            && (owned.ship.name.to_lowercase() == needle
                || owned.ship.template_id.to_lowercase() == needle)
    })
}

/// Python's default `str` for a float, enough for the cargo-space sentence.
fn py_float(value: f64) -> String {
    if value.fract() == 0.0 && value.is_finite() {
        format!("{value:.1}")
    } else {
        let text = format!("{value}");
        if text.contains('.') || text.contains('e') || text.contains('E') {
            text
        } else {
            format!("{text}.0")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Ship;
    use crate::world::new_game;

    fn escort(captain: &mut Captain, speed: f64, port: &str) {
        let mut ship = captain.ship.clone().unwrap();
        ship.template_id = "coastal_sloop".into();
        ship.name = "Escort".into();
        ship.speed = speed;
        ship.hull = 40;
        ship.hull_max = 60;
        captain.fleet.push(FleetShip {
            ship,
            docked_port_id: port.to_string(),
            cargo: Vec::new(),
        });
    }

    #[test]
    fn convoy_joins_only_ships_at_the_departure_port_and_limits_speed() {
        let mut world = new_game("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        escort(&mut world.captain, 4.0, "porto_novo");
        escort(&mut world.captain, 3.0, "silva_bay");
        form_convoy(&mut world.captain, "porto_novo");
        assert_eq!(world.captain.fleet[0].docked_port_id, "");
        assert_eq!(world.captain.fleet[1].docked_port_id, "silva_bay");
        form_convoy(&mut world.captain, "porto_novo");
        assert_eq!(world.captain.fleet[0].docked_port_id, "");
        assert!((convoy_speed(&world.captain, 9.0) - 4.0).abs() < 1e-9);
        damage_convoy(&mut world.captain, -10);
        assert_eq!(world.captain.fleet[0].ship.hull, 30);
        assert_eq!(world.captain.fleet[1].ship.hull, 40);
        dock_convoy(&mut world.captain, "silva_bay");
        assert_eq!(world.captain.fleet[0].docked_port_id, "silva_bay");
    }

    #[test]
    fn transfer_and_sell_match_python_sentences() {
        let mut world = new_game("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        world.captain.cargo.push(CargoItem {
            good_id: "grain".into(),
            quantity: 4,
            cost_basis: 20,
            acquired_port: "porto_novo".into(),
            acquired_region: "Mediterranean".into(),
            acquired_day: 1,
        });
        let mut other: Ship = world.captain.ship.clone().unwrap();
        other.name = "Hold".into();
        other.template_id = "coastal_sloop".into();
        world.captain.fleet.push(FleetShip {
            ship: other,
            docked_port_id: "porto_novo".into(),
            cargo: Vec::new(),
        });
        transfer_cargo(
            &mut world.captain,
            "grain",
            2,
            "coastal_sloop",
            "Hold",
            "porto_novo",
        )
        .unwrap();
        assert_eq!(world.captain.cargo[0].quantity, 2);
        assert_eq!(world.captain.fleet[0].cargo[0].quantity, 2);
        let err = sell_docked_ship(&mut world.captain, "Hold", "porto_novo").unwrap_err();
        assert_eq!(err, "Ship has cargo — transfer it first");
        world.captain.fleet[0].cargo.clear();
        let (silver, name) = sell_docked_ship(&mut world.captain, "Hold", "porto_novo").unwrap();
        assert_eq!(name, "Hold");
        assert_eq!(silver, 0);
        assert!(world.captain.fleet.is_empty());
    }

    #[test]
    fn wages_cover_every_fleet_hull_and_not_the_flagship() {
        let mut world = new_game("Ada", "merchant", 1, None).unwrap();
        assert_eq!(fleet_daily_wages(&world.captain), 0);
        escort(&mut world.captain, 8.0, "porto_novo");
        world.captain.fleet[0].ship.crew = 4;
        assert_eq!(fleet_daily_wages(&world.captain), 4);
    }
}
