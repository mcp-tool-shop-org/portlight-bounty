//! Canonical state view compared against the Python oracle.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::campaign::{self, HouseBooks, VictoryPathStatus};
use crate::content;
use crate::economy::TradeReceipt;
use crate::model::{Captain, Standing, Voyage, World};
use crate::voyage::VoyageEvent;

#[derive(Debug, Clone, Serialize)]
pub struct Snapshot {
    pub seed: i128,
    pub day: i64,
    pub trade_seq: u64,
    pub captain: CaptainSnap,
    pub voyage: VoyageSnap,
    pub pending_duel: Option<DuelSnap>,
    pub ports: Vec<PortSnap>,
    pub victory: Vec<VictoryPathStatus>,
    pub log: Vec<LogEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CaptainSnap {
    pub name: String,
    pub captain_type: String,
    pub silver: i64,
    pub provisions: i64,
    pub day: i64,
    pub wanted_level: i64,
    pub cargo: Vec<CargoSnap>,
    pub ship: Option<ShipSnap>,
    pub standing: StandingSnap,
}

#[derive(Debug, Clone, Serialize)]
pub struct CargoSnap {
    pub good_id: String,
    pub quantity: i64,
    pub cost_basis: i64,
    pub acquired_port: String,
    pub acquired_region: String,
    pub acquired_day: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ShipSnap {
    pub template_id: String,
    pub name: String,
    pub hull: i64,
    pub hull_max: i64,
    pub cargo_capacity: i64,
    pub speed: f64,
    pub crew: i64,
    pub crew_max: i64,
    pub morale: i64,
    pub cannons: i64,
    pub sailors: i64,
    pub gunners: i64,
    pub navigators: i64,
    pub surgeons: i64,
    pub marines: i64,
    pub quartermasters: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct StandingSnap {
    pub regional: BTreeMap<String, i64>,
    pub heat: BTreeMap<String, i64>,
    pub commercial_trust: i64,
    pub port_standing: Vec<PortStandingSnap>,
    pub underworld: BTreeMap<String, i64>,
    pub incidents: Vec<IncidentSnap>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PortStandingSnap {
    pub port_id: String,
    pub value: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct IncidentSnap {
    pub day: i64,
    pub port_id: String,
    pub region: String,
    pub incident_type: String,
    pub description: String,
    pub heat_delta: i64,
    pub standing_delta: i64,
    pub trust_delta: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct VoyageSnap {
    pub origin_id: String,
    pub destination_id: String,
    pub distance: i64,
    pub progress: i64,
    pub days_elapsed: i64,
    pub status: String,
    pub recent_events: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DuelSnap {
    pub captain_id: String,
    pub captain_name: String,
    pub faction_id: String,
    pub personality: String,
    pub strength: i64,
    pub region: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PortSnap {
    pub id: String,
    pub market: Vec<SlotSnap>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SlotSnap {
    pub good_id: String,
    pub stock: i64,
    pub buy_price: i64,
    pub sell_price: i64,
    pub flood_penalty: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReceiptSnap {
    pub receipt_id: String,
    pub action: String,
    pub good_id: String,
    pub quantity: i64,
    pub unit_price: i64,
    pub total_price: i64,
    pub stock_before: i64,
    pub stock_after: i64,
    pub day: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CargoLossSnap {
    pub good_id: String,
    pub quantity: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct EventSnap {
    pub event_type: String,
    pub message: String,
    pub hull_delta: i64,
    pub provision_delta: i64,
    pub silver_delta: i64,
    pub crew_delta: i64,
    pub speed_modifier: f64,
    pub cargo_lost: Vec<CargoLossSnap>,
    pub flavor: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    pub command: String,
    pub error: Option<String>,
    pub receipt: Option<ReceiptSnap>,
    pub events: Vec<EventSnap>,
    pub shocks: Vec<String>,
}

impl LogEntry {
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            error: None,
            receipt: None,
            events: Vec::new(),
            shocks: Vec::new(),
        }
    }
}

pub fn from_receipt(receipt: &TradeReceipt) -> ReceiptSnap {
    ReceiptSnap {
        receipt_id: receipt.receipt_id.clone(),
        action: receipt.action.to_string(),
        good_id: receipt.good_id.clone(),
        quantity: receipt.quantity,
        unit_price: receipt.unit_price,
        total_price: receipt.total_price,
        stock_before: receipt.stock_before,
        stock_after: receipt.stock_after,
        day: receipt.day,
    }
}

pub fn from_event(event: &VoyageEvent) -> EventSnap {
    EventSnap {
        event_type: event.event_type.as_str().to_string(),
        message: event.message.clone(),
        hull_delta: event.hull_delta,
        provision_delta: event.provision_delta,
        silver_delta: event.silver_delta,
        crew_delta: event.crew_delta,
        speed_modifier: event.speed_modifier,
        cargo_lost: event
            .cargo_lost
            .iter()
            .map(|(good_id, quantity)| CargoLossSnap {
                good_id: good_id.clone(),
                quantity: *quantity,
            })
            .collect(),
        flavor: event.flavor.clone(),
    }
}

fn standing_snap(standing: &Standing) -> StandingSnap {
    let mut regional = BTreeMap::new();
    let mut heat = BTreeMap::new();
    for (i, region) in content::REGIONS.iter().enumerate() {
        regional.insert((*region).to_string(), standing.regional[i]);
        heat.insert((*region).to_string(), standing.heat[i]);
    }
    let mut underworld = BTreeMap::new();
    for (k, v) in &standing.underworld {
        underworld.insert(k.clone(), *v);
    }
    StandingSnap {
        regional,
        heat,
        commercial_trust: standing.commercial_trust,
        port_standing: standing
            .port_standing
            .iter()
            .map(|(port_id, value)| PortStandingSnap {
                port_id: port_id.clone(),
                value: *value,
            })
            .collect(),
        underworld,
        incidents: standing
            .incidents
            .iter()
            .map(|inc| IncidentSnap {
                day: inc.day,
                port_id: inc.port_id.clone(),
                region: inc.region.clone(),
                incident_type: inc.incident_type.clone(),
                description: inc.description.clone(),
                heat_delta: inc.heat_delta,
                standing_delta: inc.standing_delta,
                trust_delta: inc.trust_delta,
            })
            .collect(),
    }
}

fn captain_snap(captain: &Captain) -> CaptainSnap {
    CaptainSnap {
        name: captain.name.clone(),
        captain_type: captain.captain_type.clone(),
        silver: captain.silver,
        provisions: captain.provisions,
        day: captain.day,
        wanted_level: captain.wanted_level,
        cargo: captain
            .cargo
            .iter()
            .map(|c| CargoSnap {
                good_id: c.good_id.clone(),
                quantity: c.quantity,
                cost_basis: c.cost_basis,
                acquired_port: c.acquired_port.clone(),
                acquired_region: c.acquired_region.clone(),
                acquired_day: c.acquired_day,
            })
            .collect(),
        ship: captain.ship.as_ref().map(|ship| ShipSnap {
            template_id: ship.template_id.clone(),
            name: ship.name.clone(),
            hull: ship.hull,
            hull_max: ship.hull_max,
            cargo_capacity: ship.cargo_capacity,
            speed: ship.speed,
            crew: ship.crew,
            crew_max: ship.crew_max,
            morale: ship.morale,
            cannons: ship.cannons,
            sailors: ship.sailors,
            gunners: ship.gunners,
            navigators: ship.navigators,
            surgeons: ship.surgeons,
            marines: ship.marines,
            quartermasters: ship.quartermasters,
        }),
        standing: standing_snap(&captain.standing),
    }
}

fn voyage_snap(voyage: &Voyage) -> VoyageSnap {
    VoyageSnap {
        origin_id: voyage.origin_id.clone(),
        destination_id: voyage.destination_id.clone(),
        distance: voyage.distance,
        progress: voyage.progress,
        days_elapsed: voyage.days_elapsed,
        status: voyage.status.as_str().to_string(),
        recent_events: voyage.recent_events.clone(),
    }
}

pub fn capture(world: &World, trade_seq: u64, books: &HouseBooks, log: Vec<LogEntry>) -> Snapshot {
    Snapshot {
        seed: world.seed,
        day: world.day,
        trade_seq,
        captain: captain_snap(&world.captain),
        voyage: voyage_snap(&world.voyage),
        pending_duel: world.pending_duel.as_ref().map(|d| DuelSnap {
            captain_id: d.captain_id.clone(),
            captain_name: d.captain_name.clone(),
            faction_id: d.faction_id.clone(),
            personality: d.personality.clone(),
            strength: d.strength,
            region: d.region.clone(),
        }),
        ports: world
            .ports
            .iter()
            .map(|port| PortSnap {
                id: port.id.clone(),
                market: port
                    .market
                    .iter()
                    .map(|slot| SlotSnap {
                        good_id: slot.good_id.clone(),
                        stock: slot.stock_current,
                        buy_price: slot.buy_price,
                        sell_price: slot.sell_price,
                        flood_penalty: slot.flood_penalty,
                    })
                    .collect(),
            })
            .collect(),
        victory: campaign::compute_victory_progress(world, books),
        log,
    }
}

pub fn empty(log: Vec<LogEntry>) -> Snapshot {
    Snapshot {
        seed: 0,
        day: 0,
        trade_seq: 0,
        captain: CaptainSnap {
            name: String::new(),
            captain_type: String::new(),
            silver: 0,
            provisions: 0,
            day: 0,
            wanted_level: 0,
            cargo: Vec::new(),
            ship: None,
            standing: StandingSnap {
                regional: BTreeMap::new(),
                heat: BTreeMap::new(),
                commercial_trust: 0,
                port_standing: Vec::new(),
                underworld: BTreeMap::new(),
                incidents: Vec::new(),
            },
        },
        voyage: VoyageSnap {
            origin_id: String::new(),
            destination_id: String::new(),
            distance: 0,
            progress: 0,
            days_elapsed: 0,
            status: String::new(),
            recent_events: Vec::new(),
        },
        pending_duel: None,
        ports: Vec::new(),
        victory: Vec::new(),
        log,
    }
}
