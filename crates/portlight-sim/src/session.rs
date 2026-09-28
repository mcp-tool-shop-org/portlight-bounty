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
//! buy, sell (including trade reputation and the receipt ledger), depart,
//! advance, and dock work. Advance ticks reputation, ticks markets while in port (without
//! seasonal `current_day`), sails one day at sea, records inspection and
//! arrival reputation, reprices with the captain's modifiers, and records a
//! victory path when its requirements are all met.
//!
//! Not included: contract offers, infrastructure purchases, credit draws,
//! insurance policies, sea-culture enrichment, narrative, and milestone
//! evaluation. Those books stay empty, which is what a new Python game has
//! until those systems run. Contracts are not ported, so no victory path can
//! complete during play: [`Session::victory`] is display-only unless something
//! writes completed contracts through [`Session::books_mut`].
//!
//! A pending pirate duel still freezes [`Session::advance`] until
//! [`Session::duel`] or [`Session::resolve_pending_duel`] clears it. That is
//! `portlight duel` and `GameSession._resolve_pending_duel`. Negotiate, flee,
//! naval combat, boarding, and `engine/combat.py` are not on this type.
//! `advance` does not auto-resolve, matching `auto_resolve_duels = False`.

use crate::campaign::{self, HouseBooks, VictoryPathStatus};
use crate::content::{self, PricingDef};
use crate::duel::{self, DuelOutcome};
use crate::economy::{self, recalculate_prices, TradeReceipt};
use crate::error::SimError;
use crate::model::{Officer, VoyageStatus, World};
use crate::pyrand::PyRandom;
use crate::reputation::{self, record_trade_outcome};
use crate::ship::wage_bill;
use crate::util::py_trunc;
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
    ) -> Result<Self, SimError> {
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

    pub fn buy(&mut self, good_id: &str, qty: i64) -> Result<TradeReceipt, SimError> {
        let Some(port_id) = current_port_id(&self.world).map(str::to_string) else {
            return Err(SimError::NotDocked);
        };
        let seq = self.trade_seq;
        let receipt = {
            let world = &mut self.world;
            let port = world
                .ports
                .iter_mut()
                .find(|port| port.id == port_id)
                .ok_or(SimError::NotDocked)?;
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

    pub fn sell(&mut self, good_id: &str, qty: i64) -> Result<TradeReceipt, SimError> {
        let Some(port_id) = current_port_id(&self.world).map(str::to_string) else {
            return Err(SimError::NotDocked);
        };
        let (flood_before, stock_target, region) = {
            let port = self.world.port(&port_id).ok_or(SimError::NotDocked)?;
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
                .ok_or(SimError::NotDocked)?;
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

    pub fn depart(&mut self, destination_id: &str) -> Result<(), SimError> {
        voyage::depart(&mut self.world, destination_id, false)
    }

    /// Hire crew at the current port. `role` defaults to `"sailor"` when empty.
    ///
    /// Sailors cost `port.crew_cost`. Specialists cost `wage * 10` and receive
    /// a region-flavored name and trait, matching `GameSession.hire_crew`.
    pub fn hire_crew(&mut self, count: i64, role: &str) -> Result<(), SimError> {
        let role = if role.is_empty() { "sailor" } else { role };
        let port_id = current_port_id(&self.world)
            .map(str::to_string)
            .ok_or(SimError::MustBeDockedToHire)?;
        let (crew_cost, region) = {
            let port = self
                .world
                .port(&port_id)
                .ok_or(SimError::MustBeDockedToHire)?;
            (port.crew_cost, port.region.clone())
        };
        let ship = self.world.captain.ship.as_ref().ok_or(SimError::NoShip)?;
        let space = ship.crew_max - ship.crew;
        if space <= 0 {
            return Err(SimError::CrewFull);
        }
        let spec = role_spec(&role.to_lowercase())
            .ok_or_else(|| SimError::UnknownRole(role.to_string()))?;
        let mut count = count;
        if let Some(max) = spec.max_per_ship {
            let current = role_count(ship, spec.role);
            let avail = max - current;
            if avail <= 0 {
                return Err(SimError::RoleMaximum {
                    name: spec.name.to_string(),
                    max,
                });
            }
            count = count.min(avail);
        }
        count = count.min(space);
        let cost_per = if spec.role == "sailor" {
            crew_cost
        } else {
            spec.wage * 10
        };
        let cost = count * cost_per;
        if cost > self.world.captain.silver {
            return Err(SimError::NeedCrewSilver {
                cost,
                count,
                name: spec.name.to_string(),
                each: cost_per,
                have: self.world.captain.silver,
            });
        }
        self.world.captain.silver -= cost;
        let ship = self.world.captain.ship.as_mut().ok_or(SimError::NoShip)?;
        let current = role_count(ship, spec.role);
        set_role_count(ship, spec.role, current + count);
        ship.sync_crew();
        if spec.role != "sailor" {
            for _ in 0..count {
                ship.officers.push(Officer {
                    name: officer_name(&region, &mut self.rng),
                    role: spec.role.to_string(),
                    origin_port: port_id.clone(),
                    trait_name: officer_trait(&mut self.rng),
                });
            }
        }
        Ok(())
    }

    /// Buy `days` of provisions at the current port.
    ///
    /// Cost per day is `max(1, int(provision_cost * service_modifier))`.
    pub fn provision(&mut self, days: i64) -> Result<(), SimError> {
        if days <= 0 {
            return Err(SimError::QuantityMustBeAPositiveNumber);
        }
        let port_id = current_port_id(&self.world)
            .map(str::to_string)
            .ok_or(SimError::MustBeDockedToProvision)?;
        let provision_cost = self
            .world
            .port(&port_id)
            .ok_or(SimError::MustBeDockedToProvision)?
            .provision_cost;
        let mult = reputation::service_modifier(&self.world.captain.standing, &port_id);
        let per_day = 1.max(py_trunc(provision_cost as f64 * mult));
        let cost = days * per_day;
        if cost > self.world.captain.silver {
            return Err(SimError::NeedProvisions {
                cost,
                days,
                per_day,
                have: self.world.captain.silver,
            });
        }
        self.world.captain.silver -= cost;
        self.world.captain.provisions += days;
        Ok(())
    }

    /// Work the docks for a day. Returns silver earned (3 to 5).
    ///
    /// This is `GameSession.work`: one `randint(3, 5)` on the session RNG,
    /// then `captain.day` is copied onto `world.day`. Markets, provisions,
    /// wages, and reputation do not tick.
    pub fn work(&mut self) -> Result<i64, SimError> {
        if current_port_id(&self.world).is_none() {
            return Err(SimError::MustBeDockedToWork);
        }
        let earned = economy::work_docks(&mut self.world.captain, &mut self.rng);
        self.world.day = self.world.captain.day;
        Ok(earned)
    }

    /// Fight the pending pirate with the given stances (`portlight duel`).
    ///
    /// Clears the challenge and applies `silver_delta`. `standing_delta` is
    /// returned and not written onto reputation, matching the Python CLI.
    pub fn duel(&mut self, stances: &[String]) -> Result<DuelOutcome, SimError> {
        if self.world.pending_duel.is_none() {
            return Err(SimError::NoPendingDuel);
        }
        let mut parsed = Vec::with_capacity(stances.len());
        for stance in stances {
            let stance = stance.trim().to_lowercase();
            if !matches!(stance.as_str(), "thrust" | "slash" | "parry") {
                return Err(SimError::InvalidStance(stance));
            }
            parsed.push(stance);
        }
        if parsed.len() < 3 {
            return Err(SimError::TooFewStances);
        }
        self.finish_duel(parsed)
    }

    /// `GameSession._resolve_pending_duel`: five random stances, then the same
    /// silver and clear as [`Session::duel`].
    pub fn resolve_pending_duel(&mut self) -> Result<DuelOutcome, SimError> {
        if self.world.pending_duel.is_none() {
            return Err(SimError::NoPendingDuel);
        }
        let stances = duel::auto_stances(&mut self.rng);
        self.finish_duel(stances)
    }

    fn finish_duel(&mut self, stances: Vec<String>) -> Result<DuelOutcome, SimError> {
        let pending = self
            .world
            .pending_duel
            .clone()
            .ok_or(SimError::NoPendingDuel)?;
        let crew = self
            .world
            .captain
            .ship
            .as_ref()
            .map(|ship| ship.crew)
            .unwrap_or(5);
        let outcome = duel::resolve_duel(
            &stances,
            &pending.captain_id,
            &pending.captain_name,
            &pending.personality,
            pending.strength,
            &mut self.rng,
            crew,
        );
        self.world.captain.silver = 0.max(self.world.captain.silver + outcome.silver_delta);
        self.world.pending_duel = None;
        Ok(outcome)
    }

    /// One session day. In port this ticks markets. At sea this sails.
    ///
    /// A pending duel returns no events and does not spend the day. Call
    /// [`Session::duel`] or [`Session::resolve_pending_duel`] first.
    pub fn advance(&mut self) -> Result<Turn, SimError> {
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

struct RoleSpec {
    role: &'static str,
    name: &'static str,
    wage: i64,
    max_per_ship: Option<i64>,
}

fn role_spec(role: &str) -> Option<RoleSpec> {
    Some(match role {
        "sailor" => RoleSpec {
            role: "sailor",
            name: "Sailor",
            wage: 1,
            max_per_ship: None,
        },
        "gunner" => RoleSpec {
            role: "gunner",
            name: "Gunner",
            wage: 2,
            max_per_ship: Some(3),
        },
        "navigator" => RoleSpec {
            role: "navigator",
            name: "Navigator",
            wage: 3,
            max_per_ship: Some(1),
        },
        "surgeon" => RoleSpec {
            role: "surgeon",
            name: "Surgeon",
            wage: 3,
            max_per_ship: Some(1),
        },
        "marine" => RoleSpec {
            role: "marine",
            name: "Marine",
            wage: 2,
            max_per_ship: Some(4),
        },
        "quartermaster" => RoleSpec {
            role: "quartermaster",
            name: "Quartermaster",
            wage: 2,
            max_per_ship: Some(1),
        },
        _ => return None,
    })
}

fn role_count(ship: &crate::model::Ship, role: &str) -> i64 {
    match role {
        "sailor" => ship.sailors,
        "gunner" => ship.gunners,
        "navigator" => ship.navigators,
        "surgeon" => ship.surgeons,
        "marine" => ship.marines,
        "quartermaster" => ship.quartermasters,
        _ => 0,
    }
}

fn set_role_count(ship: &mut crate::model::Ship, role: &str, count: i64) {
    let count = count.max(0);
    match role {
        "sailor" => ship.sailors = count,
        "gunner" => ship.gunners = count,
        "navigator" => ship.navigators = count,
        "surgeon" => ship.surgeons = count,
        "marine" => ship.marines = count,
        "quartermaster" => ship.quartermasters = count,
        _ => {}
    }
}

const OFFICER_NAMES: &[(&str, &[&str])] = &[
    (
        "Mediterranean",
        &[
            "Marco",
            "Sophia",
            "Nikolaos",
            "Fatima",
            "Lorenzo",
            "Valentina",
            "Dimitri",
            "Leila",
            "Antonio",
            "Isadora",
        ],
    ),
    (
        "North Atlantic",
        &[
            "William",
            "Margaret",
            "Henrik",
            "Brigitte",
            "Duncan",
            "Eleanor",
            "Gunnar",
            "Astrid",
            "Thomas",
            "Catherine",
        ],
    ),
    (
        "West Africa",
        &[
            "Kwame", "Aminata", "Kofi", "Adaeze", "Sekou", "Mariam", "Ousmane", "Aisha", "Yusuf",
            "Zara",
        ],
    ),
    (
        "East Indies",
        &[
            "Rajan", "Mei Lin", "Arjun", "Suki", "Bao", "Padma", "Kenji", "Lien", "Haruki",
            "Kamala",
        ],
    ),
    (
        "South Seas",
        &[
            "Tane", "Moana", "Rangi", "Leilani", "Makoa", "Aroha", "Koa", "Nalani", "Ioane", "Mele",
        ],
    ),
];

const OFFICER_TRAITS: &[&str] = &[
    "loyal",
    "cautious",
    "bold",
    "superstitious",
    "sharp-eyed",
    "steady",
    "hot-tempered",
    "quiet",
    "gregarious",
    "shrewd",
    "resourceful",
    "fearless",
    "meticulous",
    "jovial",
    "stoic",
];

fn officer_name(region: &str, rng: &mut PyRandom) -> String {
    let pool = OFFICER_NAMES
        .iter()
        .find(|(name, _)| *name == region)
        .map(|(_, pool)| *pool)
        .unwrap_or(OFFICER_NAMES[0].1);
    pool[rng.choice_index(pool.len())].to_string()
}

fn officer_trait(rng: &mut PyRandom) -> String {
    OFFICER_TRAITS[rng.choice_index(OFFICER_TRAITS.len())].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provision_uses_the_service_modifier() {
        let mut session = Session::new("Ada", "merchant", 1, Some("silva_bay")).unwrap();
        session.provision(4).unwrap();
        assert_eq!(session.world.captain.silver, 550 - 8);
        assert_eq!(session.world.captain.provisions, 34);

        session.world.captain.standing.set_port("silva_bay", 30);
        let before = session.world.captain.silver;
        session.provision(4).unwrap();
        assert_eq!(session.world.captain.silver, before - 4);

        assert!(
            (reputation::service_modifier(&session.world.captain.standing, "silva_bay") - 0.8)
                .abs()
                < 1e-9
        );
        session.world.captain.standing.set_port("silva_bay", 15);
        assert!(
            (reputation::service_modifier(&session.world.captain.standing, "silva_bay") - 0.9)
                .abs()
                < 1e-9
        );
        session.world.captain.standing.set_port("silva_bay", 5);
        assert!(
            (reputation::service_modifier(&session.world.captain.standing, "silva_bay") - 0.95)
                .abs()
                < 1e-9
        );
        session.world.captain.standing.set_port("silva_bay", 4);
        assert!(
            (reputation::service_modifier(&session.world.captain.standing, "silva_bay") - 1.0)
                .abs()
                < 1e-9
        );
    }
}
