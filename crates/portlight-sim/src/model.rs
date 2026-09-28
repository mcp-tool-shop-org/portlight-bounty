//! Runtime game state. Plain data, no rendering.

use serde::Serialize;

use crate::content::{self, upgrade_slots, CaptainDef, MarketSlotDef, PortDef, RouteDef, ShipDef};

pub const CONTRABAND: [&str; 3] = ["opium", "black_powder", "stolen_cargo"];

/// Portlight's port grid width (`GAME_W` in the Python world map).
///
/// Stored `map_x` values run from 0 through this width, matching the
/// print-and-play board comment.
pub const MAP_GRID_WIDTH: i64 = 50;

/// Portlight's port grid height (`GAME_H` in the Python world map).
///
/// Stored `map_y` values run from 0 through this height.
pub const MAP_GRID_HEIGHT: i64 = 36;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VoyageStatus {
    InPort,
    AtSea,
    Arrived,
}

impl VoyageStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InPort => "in_port",
            Self::AtSea => "at_sea",
            Self::Arrived => "arrived",
        }
    }
}

#[derive(Debug, Clone)]
pub struct MarketSlot {
    pub good_id: String,
    pub stock_current: i64,
    pub stock_target: i64,
    pub restock_rate: f64,
    pub local_affinity: f64,
    pub spread: f64,
    pub buy_price: i64,
    pub sell_price: i64,
    pub flood_penalty: f64,
}

impl From<&MarketSlotDef> for MarketSlot {
    fn from(s: &MarketSlotDef) -> Self {
        Self {
            good_id: s.good_id.clone(),
            stock_current: s.stock_current,
            stock_target: s.stock_target,
            restock_rate: s.restock_rate,
            local_affinity: s.local_affinity,
            spread: s.spread,
            buy_price: 0,
            sell_price: 0,
            flood_penalty: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Port {
    pub id: String,
    pub name: String,
    pub description: String,
    pub region: String,
    pub features: Vec<String>,
    pub market: Vec<MarketSlot>,
    pub port_fee: i64,
    pub provision_cost: i64,
    pub repair_cost: i64,
    pub crew_cost: i64,
    pub map_x: i64,
    pub map_y: i64,
}

impl From<&PortDef> for Port {
    fn from(p: &PortDef) -> Self {
        Self {
            id: p.id.clone(),
            name: p.name.clone(),
            description: p.description.clone(),
            region: p.region.clone(),
            features: p.features.clone(),
            market: p.market.iter().map(MarketSlot::from).collect(),
            port_fee: p.port_fee,
            provision_cost: p.provision_cost,
            repair_cost: p.repair_cost,
            crew_cost: p.crew_cost,
            map_x: p.map_x,
            map_y: p.map_y,
        }
    }
}

impl Port {
    pub fn has_feature(&self, feature: &str) -> bool {
        self.features.iter().any(|f| f == feature)
    }

    pub fn slot_mut(&mut self, good_id: &str) -> Option<&mut MarketSlot> {
        self.market.iter_mut().find(|s| s.good_id == good_id)
    }

    pub fn slot(&self, good_id: &str) -> Option<&MarketSlot> {
        self.market.iter().find(|s| s.good_id == good_id)
    }
}

#[derive(Debug, Clone)]
pub struct Route {
    pub port_a: String,
    pub port_b: String,
    pub distance: i64,
    pub danger: f64,
    pub min_ship_class: String,
    pub lore_name: String,
    pub lore: String,
}

impl From<&RouteDef> for Route {
    fn from(r: &RouteDef) -> Self {
        Self {
            port_a: r.port_a.clone(),
            port_b: r.port_b.clone(),
            distance: r.distance,
            danger: r.danger,
            min_ship_class: r.min_ship_class.clone(),
            lore_name: r.lore_name.clone(),
            lore: r.lore.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ship {
    pub template_id: String,
    pub name: String,
    pub hull: i64,
    pub hull_max: i64,
    pub cargo_capacity: i64,
    pub speed: f64,
    pub crew: i64,
    pub crew_max: i64,
    pub cannons: i64,
    pub maneuver: f64,
    pub upgrade_slots: i64,
    pub morale: i64,
    pub sailors: i64,
    pub gunners: i64,
    pub navigators: i64,
    pub surgeons: i64,
    pub marines: i64,
    pub quartermasters: i64,
    pub officers: Vec<Officer>,
}

#[derive(Debug, Clone)]
pub struct Officer {
    pub name: String,
    pub role: String,
    pub origin_port: String,
    pub trait_name: String,
}

impl Ship {
    pub fn from_template(template: &ShipDef) -> Self {
        Self {
            template_id: template.id.clone(),
            name: template.name.clone(),
            hull: template.hull_max,
            hull_max: template.hull_max,
            cargo_capacity: template.cargo_capacity,
            speed: template.speed,
            crew: template.crew_min,
            crew_max: template.crew_max,
            cannons: template.cannons,
            maneuver: template.maneuver,
            upgrade_slots: upgrade_slots(&template.ship_class),
            morale: 50,
            sailors: template.crew_min,
            gunners: 0,
            navigators: 0,
            surgeons: 0,
            marines: 0,
            quartermasters: 0,
            officers: Vec::new(),
        }
    }

    pub fn roster_total(&self) -> i64 {
        self.sailors
            + self.gunners
            + self.navigators
            + self.surgeons
            + self.marines
            + self.quartermasters
    }

    pub fn sync_crew(&mut self) {
        self.crew = self.roster_total();
    }
}

#[derive(Debug, Clone)]
pub struct CargoItem {
    pub good_id: String,
    pub quantity: i64,
    pub cost_basis: i64,
    pub acquired_port: String,
    pub acquired_region: String,
    pub acquired_day: i64,
}

#[derive(Debug, Clone)]
pub struct Incident {
    pub day: i64,
    pub port_id: String,
    pub region: String,
    pub incident_type: String,
    pub description: String,
    pub heat_delta: i64,
    pub standing_delta: i64,
    pub trust_delta: i64,
}

#[derive(Debug, Clone)]
pub struct Standing {
    /// Parallel to [`content::REGIONS`].
    pub regional: [i64; 5],
    pub heat: [i64; 5],
    pub commercial_trust: i64,
    /// Insertion order matches Python dict insertion.
    pub port_standing: Vec<(String, i64)>,
    pub underworld: Vec<(String, i64)>,
    pub incidents: Vec<Incident>,
}

impl Standing {
    pub fn from_captain(captain: &CaptainDef) -> Self {
        let r = &captain.reputation;
        Self {
            regional: [
                r.mediterranean,
                r.north_atlantic,
                r.west_africa,
                r.east_indies,
                r.south_seas,
            ],
            heat: [r.customs_heat; 5],
            commercial_trust: r.commercial_trust,
            port_standing: Vec::new(),
            underworld: r.underworld.iter().map(|(k, v)| (k.clone(), *v)).collect(),
            incidents: Vec::new(),
        }
    }

    pub fn region_index(region: &str) -> Option<usize> {
        content::REGIONS.iter().position(|r| *r == region)
    }

    pub fn heat_of(&self, region: &str) -> i64 {
        Self::region_index(region)
            .map(|i| self.heat[i])
            .unwrap_or(0)
    }

    pub fn regional_of(&self, region: &str) -> i64 {
        Self::region_index(region)
            .map(|i| self.regional[i])
            .unwrap_or(0)
    }

    pub fn set_heat(&mut self, region: &str, value: i64) {
        if let Some(i) = Self::region_index(region) {
            self.heat[i] = value;
        }
    }

    pub fn set_regional(&mut self, region: &str, value: i64) {
        if let Some(i) = Self::region_index(region) {
            self.regional[i] = value;
        }
    }

    pub fn port_value(&self, port_id: &str) -> Option<i64> {
        self.port_standing
            .iter()
            .find(|(id, _)| id == port_id)
            .map(|(_, v)| *v)
    }

    pub fn set_port(&mut self, port_id: &str, value: i64) {
        if let Some(slot) = self.port_standing.iter_mut().find(|(id, _)| id == port_id) {
            slot.1 = value;
        } else {
            self.port_standing.push((port_id.to_string(), value));
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeferredFee {
    pub fee_type: String,
    pub amount: i64,
    pub day: i64,
}

/// A weapon the captain is carrying. Catalog stats stay on the content defs.
#[derive(Debug, Clone)]
pub struct Weapon {
    pub id: String,
    pub name: String,
    /// `melee`, `firearm`, `thrown`, or `mechanical`.
    pub kind: String,
    pub quality: String,
    pub ammo: i64,
}

/// Armor the captain is wearing.
#[derive(Debug, Clone)]
pub struct Armor {
    pub id: String,
    pub name: String,
    pub armor_type: String,
    pub damage_reduction: i64,
    pub dodge_penalty: i64,
    pub stamina_penalty: i64,
    pub quality: String,
}

/// A fighting style the captain has learned.
#[derive(Debug, Clone)]
pub struct FightingStyle {
    pub id: String,
    pub name: String,
    pub region: String,
}

/// One finished meeting with a named pirate, kept on the captain.
#[derive(Debug, Clone)]
pub struct PirateEncounterRecord {
    pub captain_id: String,
    pub faction_id: String,
    pub day: i64,
    pub outcome: String,
    pub region: String,
}

/// A prize or docked hull besides the flagship.
#[derive(Debug, Clone)]
pub struct OwnedShip {
    pub ship: Ship,
    pub docked_port_id: String,
}

/// A learned captain skill. `level` is 1–3; untrained skills are absent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skill {
    pub id: String,
    pub level: i64,
}

/// One companion traveling with the captain. Morale starts at 70.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Companion {
    pub companion_id: String,
    pub role_id: String,
    pub morale: i64,
    pub joined_day: i64,
    pub personality: String,
}

/// The captain's companion party. Python stores this as `captain.party`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Party {
    pub companions: Vec<Companion>,
    pub max_size: i64,
    pub departed: Vec<String>,
}

impl Default for Party {
    fn default() -> Self {
        Self {
            companions: Vec::new(),
            max_size: 2,
            departed: Vec::new(),
        }
    }
}

/// Four-axis opinion a pirate captain holds toward the player.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CaptainRelationship {
    pub respect: i64,
    pub fear: i64,
    pub grudge: i64,
    pub familiarity: i64,
}

/// One remembered meeting with a pirate captain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncounterMemory {
    pub day: i64,
    pub region: String,
    pub outcome: String,
    pub player_spared: bool,
    pub player_used_firearm: bool,
    pub crew_killed: i64,
    pub respect_delta: i64,
    pub fear_delta: i64,
    pub grudge_delta: i64,
    pub familiarity_delta: i64,
}

/// Persistent memory for one pirate captain (`engine/captain_memory.py`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptainMemory {
    pub captain_id: String,
    pub encounters: Vec<EncounterMemory>,
    pub relationship: CaptainRelationship,
    pub last_seen_day: i64,
    pub last_seen_region: String,
    pub times_spared: i64,
    pub times_defeated_by_player: i64,
    pub times_defeated_player: i64,
    pub player_sank_their_ship: bool,
}

impl CaptainMemory {
    pub fn new(captain_id: impl Into<String>) -> Self {
        Self {
            captain_id: captain_id.into(),
            encounters: Vec::new(),
            relationship: CaptainRelationship::default(),
            last_seen_day: 0,
            last_seen_region: String::new(),
            times_spared: 0,
            times_defeated_by_player: 0,
            times_defeated_player: 0,
            player_sank_their_ship: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Captain {
    pub name: String,
    pub captain_type: String,
    pub silver: i64,
    pub ship: Option<Ship>,
    pub cargo: Vec<CargoItem>,
    pub provisions: i64,
    pub day: i64,
    pub standing: Standing,
    pub wanted_level: i64,
    pub active_bounties: Vec<String>,
    pub deferred_fees: Vec<DeferredFee>,
    pub melee: Option<Weapon>,
    pub firearm: Option<Weapon>,
    pub mechanical: Option<Weapon>,
    pub throwing: Vec<Weapon>,
    pub armor: Option<Armor>,
    pub styles: Vec<FightingStyle>,
    pub active_style: Option<String>,
    pub duels_won: i64,
    pub duels_lost: i64,
    pub encounters: Vec<PirateEncounterRecord>,
    pub fleet: Vec<OwnedShip>,
    pub naval_victories: i64,
    pub naval_defeats: i64,
    /// Fighting styles learned from regional masters, in learn order.
    pub learned_styles: Vec<String>,
    /// Captain skills, in the order they were first learned.
    pub skills: Vec<Skill>,
    pub party: Party,
}

#[derive(Debug, Clone)]
pub struct Voyage {
    pub origin_id: String,
    pub destination_id: String,
    pub distance: i64,
    pub progress: i64,
    pub days_elapsed: i64,
    pub status: VoyageStatus,
    pub recent_events: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PendingDuel {
    pub captain_id: String,
    pub captain_name: String,
    pub faction_id: String,
    pub personality: String,
    pub strength: i64,
    pub region: String,
}

#[derive(Debug, Clone)]
pub struct World {
    pub captain: Captain,
    pub ports: Vec<Port>,
    pub routes: Vec<Route>,
    pub voyage: Voyage,
    pub day: i64,
    /// `random.Random` seed. Any `i128`, matching CPython's absolute-value
    /// seeding for that range. See [`crate::pyrand::PyRandom::from_seed`].
    pub seed: i128,
    pub pending_duel: Option<PendingDuel>,
    /// Pirate-captain memories. Empty until an encounter is recorded.
    pub captain_memories: Vec<CaptainMemory>,
}

impl World {
    pub fn port(&self, id: &str) -> Option<&Port> {
        self.ports.iter().find(|p| p.id == id)
    }

    pub fn port_mut(&mut self, id: &str) -> Option<&mut Port> {
        self.ports.iter_mut().find(|p| p.id == id)
    }

    pub fn find_route(&self, origin_id: &str, dest_id: &str) -> Option<&Route> {
        self.routes.iter().find(|r| {
            (r.port_a == origin_id && r.port_b == dest_id)
                || (r.port_a == dest_id && r.port_b == origin_id)
        })
    }
}

/// A live offer on the contract board. Python `ContractOffer`.
#[derive(Debug, Clone, PartialEq)]
pub struct Contract {
    pub id: String,
    pub template_id: String,
    pub family: String,
    pub title: String,
    pub description: String,
    pub issuer_port_id: String,
    pub destination_port_id: String,
    pub good_id: String,
    pub quantity: i64,
    pub created_day: i64,
    pub deadline_day: i64,
    pub reward_silver: i64,
    pub bonus_reward: i64,
    pub required_trust_tier: String,
    pub required_standing: i64,
    pub heat_ceiling: Option<i64>,
    pub inspection_modifier: f64,
    pub source_region: Option<String>,
    pub source_port: Option<String>,
    pub offer_reason: String,
    pub tags: Vec<String>,
    pub acceptance_window: i64,
}

/// An accepted obligation. Python `ActiveContract`.
#[derive(Debug, Clone, PartialEq)]
pub struct ActiveContract {
    pub offer_id: String,
    pub template_id: String,
    pub family: String,
    pub title: String,
    pub accepted_day: i64,
    pub deadline_day: i64,
    pub destination_port_id: String,
    pub good_id: String,
    pub required_quantity: i64,
    pub delivered_quantity: i64,
    pub reward_silver: i64,
    pub bonus_reward: i64,
    pub source_region: Option<String>,
    pub source_port: Option<String>,
    pub inspection_modifier: f64,
    pub status: String,
}

/// Resolution of a completed, expired, or abandoned contract.
///
/// `good_id` through `reward_silver` are copied off the obligation at
/// resolution so a caller can log the contract after it leaves the board.
/// Python's `ContractOutcome` does not carry those fields.
#[derive(Debug, Clone, PartialEq)]
pub struct ContractOutcome {
    pub contract_id: String,
    pub outcome_type: String,
    pub silver_delta: i64,
    pub trust_delta: i64,
    pub standing_delta: i64,
    pub heat_delta: i64,
    pub completion_day: i64,
    pub summary: String,
    pub family: String,
    pub good_id: String,
    pub required_quantity: i64,
    pub delivered_quantity: i64,
    pub destination_port_id: String,
    pub deadline_day: i64,
    pub reward_silver: i64,
}

/// One row of `Captain.breach_records`.
#[derive(Debug, Clone, PartialEq)]
pub struct BreachRecord {
    pub contract_id: String,
    pub day: i64,
    pub port_id: String,
    pub family: String,
}

/// Offers, accepted work, and resolved outcomes for one session.
#[derive(Debug, Clone)]
pub struct ContractBoard {
    pub offers: Vec<Contract>,
    pub active: Vec<ActiveContract>,
    pub completed: Vec<ContractOutcome>,
    pub last_refresh_day: i64,
    pub max_offers: i64,
    pub breaches: Vec<BreachRecord>,
}

impl Default for ContractBoard {
    fn default() -> Self {
        Self {
            offers: Vec::new(),
            active: Vec::new(),
            completed: Vec::new(),
            last_refresh_day: 0,
            max_offers: 5,
            breaches: Vec::new(),
        }
    }
}
