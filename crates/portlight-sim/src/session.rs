//! Stepwise game session for a front end.
//!
//! A Godot/gdext node should keep one [`Session`] and call [`Session::buy`],
//! [`Session::sell`], [`Session::depart`], and [`Session::advance`] one turn
//! at a time. Read [`Session::world`] for ports (including `map_x` / `map_y`),
//! the ship, and the voyage. Read [`Session::sail_lanes`] for the picker and
//! [`Session::victory`] for the four victory paths.
//!
//! `run_script` is a thin wrapper over these methods, so the parity harness
//! covers the same path. There is no save/load yet; do not invent a format
//! in the UI crate.
//!
//! This is the slice of `GameSession` the port compares with Python: new game,
//! buy, sell (including trade reputation and the receipt ledger), depart, and
//! advance. Advance ticks reputation, ticks markets while in port (without
//! seasonal `current_day`), sails one day at sea, records inspection and
//! arrival reputation, reprices with the captain's modifiers, and records a
//! victory path when its requirements are all met.
//!
//! Not included: contract offers, infrastructure purchases, credit draws,
//! insurance policies, sea-culture enrichment, narrative, milestone
//! evaluation, and auto-resolved duels. Those books stay empty, which is what
//! a new Python game has until those systems run.

use crate::campaign::{self, HouseBooks, VictoryPathStatus};
use crate::content::{self, PricingDef};
use crate::economy::{self, recalculate_prices, TradeReceipt};
use crate::model::{VoyageStatus, World};
use crate::pyrand::PyRandom;
use crate::reputation::{self, record_trade_outcome};
use crate::ship::wage_bill;
use crate::voyage::{self, sail_lanes, SailLane, VoyageEvent};
use crate::world::new_game;

/// One turn of [`Session::advance`].
#[derive(Debug, Clone)]
pub struct Turn {
    pub events: Vec<VoyageEvent>,
    pub shocks: Vec<String>,
}

/// One playable game.
#[derive(Debug, Clone)]
pub struct Session {
    world: World,
    rng: PyRandom,
    trade_seq: u64,
    books: HouseBooks,
}

impl Session {
    /// Start a game. `seed` matches `random.Random(seed)` for every `i128`.
    ///
    /// `starting_port` overrides the captain's home port. `None` uses the
    /// archetype home.
    pub fn new(
        captain_name: &str,
        captain_type: &str,
        seed: i128,
        starting_port: Option<&str>,
    ) -> Result<Self, String> {
        let world = new_game(captain_name, captain_type, seed, starting_port)?;
        let rng = PyRandom::from_seed(world.seed);
        Ok(Self {
            world,
            rng,
            trade_seq: 0,
            books: HouseBooks::default(),
        })
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn trade_seq(&self) -> u64 {
        self.trade_seq
    }

    /// Ledger and the contract/infrastructure records victory reads.
    pub fn books(&self) -> &HouseBooks {
        &self.books
    }

    /// Mutable books for systems that grant contracts, licenses, warehouses,
    /// brokers, policies, or credit. Trade receipts already update the ledger.
    pub fn books_mut(&mut self) -> &mut HouseBooks {
        &mut self.books
    }

    pub fn sail_lanes(&self) -> Vec<SailLane> {
        sail_lanes(&self.world)
    }

    /// The four victory paths, highest candidate strength first.
    pub fn victory(&self) -> Vec<VictoryPathStatus> {
        campaign::compute_victory_progress(&self.world, &self.books)
    }

    pub fn buy(&mut self, good_id: &str, qty: i64) -> Result<TradeReceipt, String> {
        let Some(port_id) = current_port_id(&self.world).map(str::to_string) else {
            return Err("Not docked at a port".to_string());
        };
        let seq = self.trade_seq;
        let receipt = {
            let world = &mut self.world;
            let port = world
                .ports
                .iter_mut()
                .find(|port| port.id == port_id)
                .ok_or_else(|| "Not docked at a port".to_string())?;
            economy::execute_buy(&mut world.captain, port, good_id, qty, seq)?
        };
        self.trade_seq += 1;
        self.books.note_receipt(receipt.action, receipt.total_price);
        let pricing = pricing(&self.world).cloned();
        if let Some(port) = self.world.port_mut(&port_id) {
            recalculate_prices(port, pricing.as_ref());
        }
        Ok(receipt)
    }

    pub fn sell(&mut self, good_id: &str, qty: i64) -> Result<TradeReceipt, String> {
        let Some(port_id) = current_port_id(&self.world).map(str::to_string) else {
            return Err("Not docked at a port".to_string());
        };
        let (flood_before, stock_target, region) = {
            let port = self
                .world
                .port(&port_id)
                .ok_or_else(|| "Not docked at a port".to_string())?;
            let slot = port.slot(good_id);
            (
                slot.map(|slot| slot.flood_penalty).unwrap_or(0.0),
                slot.map(|slot| slot.stock_target).unwrap_or(50),
                port.region.clone(),
            )
        };
        let seq = self.trade_seq;
        let receipt = {
            let world = &mut self.world;
            let port = world
                .ports
                .iter_mut()
                .find(|port| port.id == port_id)
                .ok_or_else(|| "Not docked at a port".to_string())?;
            economy::execute_sell(&mut world.captain, port, good_id, qty, seq)?
        };
        self.trade_seq += 1;
        self.books.note_receipt(receipt.action, receipt.total_price);
        let cost_basis =
            economy::estimate_cost_basis(&self.world.captain, good_id, receipt.quantity);
        let margin_pct = if cost_basis > 0 {
            (receipt.total_price - cost_basis) as f64 / cost_basis.max(1) as f64 * 100.0
        } else {
            50.0
        };
        let category = content::content()
            .good(good_id)
            .map(|good| good.category.as_str())
            .unwrap_or("commodity");
        record_trade_outcome(
            &mut self.world.captain.standing,
            &self.world.captain.captain_type,
            self.world.day,
            &port_id,
            &region,
            good_id,
            category,
            receipt.quantity,
            margin_pct,
            stock_target,
            flood_before,
        );
        let pricing = pricing(&self.world).cloned();
        if let Some(port) = self.world.port_mut(&port_id) {
            recalculate_prices(port, pricing.as_ref());
        }
        Ok(receipt)
    }

    pub fn depart(&mut self, destination_id: &str) -> Result<(), String> {
        voyage::depart(&mut self.world, destination_id, false)
    }

    /// One session day. In port this ticks markets. At sea this sails.
    pub fn advance(&mut self) -> Result<Turn, String> {
        reputation::tick_reputation(&mut self.world.captain.standing);
        let turn = if self.world.voyage.status != VoyageStatus::AtSea {
            let shocks = economy::tick_markets(&mut self.world.ports, 1, &mut self.rng, 0);
            self.world.day += 1;
            self.world.captain.day += 1;
            if self.world.captain.provisions > 0 {
                self.world.captain.provisions -= 1;
            }
            if let Some(ship) = self.world.captain.ship.as_ref() {
                let wage = wage_bill(ship);
                if wage > 0 && self.world.captain.silver >= wage {
                    self.world.captain.silver -= wage;
                }
            }
            Turn {
                events: Vec::new(),
                shocks,
            }
        } else {
            let events = voyage::advance_day(&mut self.world, &mut self.rng)?;
            record_sea_consequences(&mut self.world, &events);
            if self.world.voyage.status == VoyageStatus::Arrived {
                let _ = voyage::arrive(&mut self.world);
                if let Some(port) = self.world.port(&self.world.voyage.destination_id) {
                    let port_id = port.id.clone();
                    let region = port.region.clone();
                    reputation::record_port_arrival(
                        &mut self.world.captain.standing,
                        &port_id,
                        &region,
                    );
                }
            }
            Turn {
                events,
                shocks: Vec::new(),
            }
        };
        reprice_all(&mut self.world);
        let newly = campaign::evaluate_victory_closure(&self.world, &self.books);
        self.books.completed_paths.extend(newly);
        Ok(turn)
    }
}

fn pricing(world: &World) -> Option<&PricingDef> {
    content::content()
        .captain(&world.captain.captain_type)
        .map(|captain| &captain.pricing)
}

fn current_port_id(world: &World) -> Option<&str> {
    if world.voyage.status == VoyageStatus::InPort {
        Some(world.voyage.destination_id.as_str())
    } else {
        None
    }
}

fn record_sea_consequences(world: &mut World, events: &[VoyageEvent]) {
    let region = world
        .port(&world.voyage.destination_id)
        .map(|port| port.region.clone())
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
