//! Scripted session over the engine.
//!
//! This is the slice of `portlight.app.session.GameSession` that stage 1
//! compares with Python: new game, buy, sell (including trade reputation),
//! depart, and advance. Advance ticks reputation, ticks markets while in port
//! (without seasonal `current_day`, matching the session call), sails one day
//! at sea, records inspection and arrival reputation, and reprices with the
//! captain's modifiers.
//!
//! Not included: contracts, infrastructure, credit, sea-culture enrichment,
//! narrative, campaign milestones, insurance, and auto-resolved duels.

use crate::content::{self, PricingDef};
use crate::economy::{self, recalculate_prices};
use crate::model::{VoyageStatus, World};
use crate::pyrand::PyRandom;
use crate::reputation::{self, record_trade_outcome};
use crate::ship::wage_bill;
use crate::snapshot::{self, LogEntry, Snapshot};
use crate::voyage::{self, VoyageEvent};
use crate::world::new_game;

struct Game {
    world: World,
    rng: PyRandom,
    trade_seq: u64,
}

pub fn run_script(script: &str) -> Snapshot {
    let mut game: Option<Game> = None;
    let mut log = Vec::new();
    for line in script.lines() {
        let raw = line.trim();
        if raw.is_empty() || raw.starts_with('#') {
            continue;
        }
        let tokens = tokenize(raw);
        let mut entry = LogEntry::new(raw);
        match dispatch(&mut game, &tokens, &mut entry) {
            Ok(()) => log.push(entry),
            Err(err) => {
                entry.error = Some(err);
                log.push(entry);
                break;
            }
        }
    }
    match game {
        Some(game) => snapshot::capture(&game.world, game.trade_seq, log),
        None => snapshot::empty(log),
    }
}

fn dispatch(
    game: &mut Option<Game>,
    tokens: &[String],
    entry: &mut LogEntry,
) -> Result<(), String> {
    let cmd = tokens.first().map(String::as_str).unwrap_or("");
    match cmd {
        "new" => {
            if tokens.len() < 4 {
                return Err("Usage: new <captain_type> <name> <seed> [port]".to_string());
            }
            let captain_type = &tokens[1];
            let name = &tokens[2];
            let seed: i64 = tokens[3]
                .parse()
                .map_err(|_| format!("Invalid number: {}", tokens[3]))?;
            let port = tokens
                .get(3)
                .and_then(|_| tokens.get(4))
                .map(String::as_str);
            let world = new_game(name, captain_type, seed, port)?;
            let rng = PyRandom::from_seed(world.seed);
            *game = Some(Game {
                world,
                rng,
                trade_seq: 0,
            });
            Ok(())
        }
        "buy" => {
            let game = game.as_mut().ok_or_else(|| "No active game".to_string())?;
            if tokens.len() != 3 {
                return Err("Usage: buy <good> <qty>".to_string());
            }
            let qty = parse_qty(&tokens[2])?;
            buy(game, &tokens[1], qty, entry)
        }
        "sell" => {
            let game = game.as_mut().ok_or_else(|| "No active game".to_string())?;
            if tokens.len() != 3 {
                return Err("Usage: sell <good> <qty>".to_string());
            }
            let qty = parse_qty(&tokens[2])?;
            sell(game, &tokens[1], qty, entry)
        }
        "depart" => {
            let game = game.as_mut().ok_or_else(|| "No active game".to_string())?;
            if tokens.len() != 2 {
                return Err("Usage: depart <port_id>".to_string());
            }
            voyage::depart(&mut game.world, &tokens[1], false)
        }
        "advance" => {
            let game = game.as_mut().ok_or_else(|| "No active game".to_string())?;
            advance(game, entry);
            Ok(())
        }
        other => Err(format!("Unknown command: {other}")),
    }
}

fn parse_qty(token: &str) -> Result<i64, String> {
    token
        .parse()
        .map_err(|_| format!("Invalid number: {token}"))
}

fn pricing(world: &World) -> Option<&PricingDef> {
    content::content()
        .captain(&world.captain.captain_type)
        .map(|c| &c.pricing)
}

fn current_port_id(world: &World) -> Option<&str> {
    if world.voyage.status == VoyageStatus::InPort {
        Some(world.voyage.destination_id.as_str())
    } else {
        None
    }
}

fn buy(game: &mut Game, good_id: &str, qty: i64, entry: &mut LogEntry) -> Result<(), String> {
    let Some(port_id) = current_port_id(&game.world).map(str::to_string) else {
        return Err("Not docked at a port".to_string());
    };
    let seq = game.trade_seq;
    let receipt = {
        let world = &mut game.world;
        let port = world
            .ports
            .iter_mut()
            .find(|p| p.id == port_id)
            .ok_or_else(|| "Not docked at a port".to_string())?;
        economy::execute_buy(&mut world.captain, port, good_id, qty, seq)?
    };
    game.trade_seq += 1;
    let pricing = pricing(&game.world).cloned();
    if let Some(port) = game.world.port_mut(&port_id) {
        recalculate_prices(port, pricing.as_ref());
    }
    entry.receipt = Some(snapshot::from_receipt(&receipt));
    Ok(())
}

fn sell(game: &mut Game, good_id: &str, qty: i64, entry: &mut LogEntry) -> Result<(), String> {
    let Some(port_id) = current_port_id(&game.world).map(str::to_string) else {
        return Err("Not docked at a port".to_string());
    };
    let (flood_before, stock_target, region) = {
        let port = game
            .world
            .port(&port_id)
            .ok_or_else(|| "Not docked at a port".to_string())?;
        let slot = port.slot(good_id);
        (
            slot.map(|s| s.flood_penalty).unwrap_or(0.0),
            slot.map(|s| s.stock_target).unwrap_or(50),
            port.region.clone(),
        )
    };
    let seq = game.trade_seq;
    let receipt = {
        let world = &mut game.world;
        let port = world
            .ports
            .iter_mut()
            .find(|p| p.id == port_id)
            .ok_or_else(|| "Not docked at a port".to_string())?;
        economy::execute_sell(&mut world.captain, port, good_id, qty, seq)?
    };
    game.trade_seq += 1;
    let cost_basis = economy::estimate_cost_basis(&game.world.captain, good_id, receipt.quantity);
    let margin_pct = if cost_basis > 0 {
        (receipt.total_price - cost_basis) as f64 / cost_basis.max(1) as f64 * 100.0
    } else {
        50.0
    };
    let category = content::content()
        .good(good_id)
        .map(|g| g.category.as_str())
        .unwrap_or("commodity");
    record_trade_outcome(
        &mut game.world.captain.standing,
        &game.world.captain.captain_type,
        game.world.day,
        &port_id,
        &region,
        good_id,
        category,
        receipt.quantity,
        margin_pct,
        stock_target,
        flood_before,
    );
    let pricing = pricing(&game.world).cloned();
    if let Some(port) = game.world.port_mut(&port_id) {
        recalculate_prices(port, pricing.as_ref());
    }
    entry.receipt = Some(snapshot::from_receipt(&receipt));
    Ok(())
}

fn advance(game: &mut Game, entry: &mut LogEntry) {
    reputation::tick_reputation(&mut game.world.captain.standing);
    if game.world.voyage.status != VoyageStatus::AtSea {
        let shocks = economy::tick_markets(&mut game.world.ports, 1, &mut game.rng, 0);
        game.world.day += 1;
        game.world.captain.day += 1;
        if game.world.captain.provisions > 0 {
            game.world.captain.provisions -= 1;
        }
        if let Some(ship) = game.world.captain.ship.as_ref() {
            let wage = wage_bill(ship);
            if wage > 0 && game.world.captain.silver >= wage {
                game.world.captain.silver -= wage;
            }
        }
        entry.shocks = shocks;
    } else {
        let events = voyage::advance_day(&mut game.world, &mut game.rng);
        record_sea_consequences(&mut game.world, &events);
        if game.world.voyage.status == VoyageStatus::Arrived {
            let _ = voyage::arrive(&mut game.world);
            if let Some(port) = game.world.port(&game.world.voyage.destination_id) {
                let port_id = port.id.clone();
                let region = port.region.clone();
                reputation::record_port_arrival(
                    &mut game.world.captain.standing,
                    &port_id,
                    &region,
                );
            }
        }
        entry.events = events.iter().map(snapshot::from_event).collect();
    }
    reprice_all(&mut game.world);
}

fn record_sea_consequences(world: &mut World, events: &[VoyageEvent]) {
    let region = world
        .port(&world.voyage.destination_id)
        .map(|p| p.region.clone())
        .unwrap_or_else(|| "Mediterranean".to_string());
    let port_id = world.voyage.origin_id.clone();
    let day = world.day;
    for event in events {
        if event.event_type == voyage::EventType::Inspection {
            let seized = !event.cargo_lost.is_empty();
            reputation::record_inspection_outcome(
                &mut world.captain.standing,
                day,
                &port_id,
                &region,
                event.silver_delta.abs(),
                seized,
            );
        }
    }
}

fn reprice_all(world: &mut World) {
    let pricing = pricing(world).cloned();
    for port in &mut world.ports {
        recalculate_prices(port, pricing.as_ref());
    }
}

fn tokenize(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    for ch in line.chars() {
        if ch == '"' {
            quoted = !quoted;
            continue;
        }
        if ch.is_whitespace() && !quoted {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
        } else {
            cur.push(ch);
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_command_stops_the_script() {
        let snap = run_script("new merchant Ada 1\nfly\nbuy grain 1\n");
        assert_eq!(snap.log.len(), 2);
        assert_eq!(snap.log[1].error.as_deref(), Some("Unknown command: fly"));
        assert_eq!(snap.captain.silver, 550);
    }
}
