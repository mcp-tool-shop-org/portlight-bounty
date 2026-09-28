//! Canonical state view compared against the Python oracle.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::campaign::{self, HouseBooks, VictoryPathStatus};
use crate::content;
use crate::duel::{DuelOutcome, DuelRound};
use crate::economy::TradeReceipt;
use crate::model::{
    ActiveContract, ActiveFestival, ActivePolicy, BrokerOffice, Captain, ContractOutcome,
    CreditState, CulturalState, InfrastructureRecord, InsuranceClaim, JournalEntry, NarrativeState,
    OwnedLicense, Standing, Voyage, WarehouseLease, World,
};
use crate::session::EncounterStep;
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub infrastructure: Option<InfraSnap>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub milestones: Vec<MilestoneSnap>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub culture: Option<CultureSnap>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub narrative: Option<NarrativeSnap>,
    pub log: Vec<LogEntry>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CultureSnap {
    pub port_visits: Vec<VisitSnap>,
    pub regions_entered: Vec<String>,
    pub cultural_encounters: i64,
    pub festivals_visited: i64,
    pub active_festivals: Vec<FestivalSnap>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct VisitSnap {
    pub port_id: String,
    pub visits: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FestivalSnap {
    pub festival_id: String,
    pub port_id: String,
    pub start_day: i64,
    pub end_day: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct NarrativeSnap {
    pub fired: Vec<String>,
    pub journal: Vec<JournalSnap>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct JournalSnap {
    pub beat_id: String,
    pub day: i64,
    pub port_id: String,
    pub region: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MilestoneSnap {
    pub milestone_id: String,
    pub completed_day: i64,
    pub evidence: String,
    pub family: String,
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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub encounters: Vec<EncounterRecordSnap>,
    #[serde(skip_serializing_if = "is_zero")]
    pub duels_won: i64,
    #[serde(skip_serializing_if = "is_zero")]
    pub duels_lost: i64,
    #[serde(skip_serializing_if = "is_zero")]
    pub naval_victories: i64,
    #[serde(skip_serializing_if = "is_zero")]
    pub naval_defeats: i64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub fleet: Vec<FleetSnap>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub skills: Vec<SkillSnap>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub learned_styles: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub companions: Vec<CompanionSnap>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub injuries: Vec<InjurySnap>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub armor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub melee_weapon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firearm: Option<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub weapon_quality: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub weapon_usage: BTreeMap<String, i64>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub weapon_provenance: BTreeMap<String, ProvenanceSnap>,
}

fn is_zero(value: &i64) -> bool {
    *value == 0
}

#[derive(Debug, Clone, Serialize)]
pub struct EncounterRecordSnap {
    pub captain_id: String,
    pub faction_id: String,
    pub day: i64,
    pub outcome: String,
    pub region: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FleetSnap {
    pub template_id: String,
    pub name: String,
    pub hull: i64,
    pub hull_max: i64,
    pub crew: i64,
    pub docked_port_id: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cargo: Vec<CargoSnap>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InjurySnap {
    pub injury_id: String,
    pub acquired_day: i64,
    pub heal_remaining: Option<i64>,
    pub treated: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProvenanceSnap {
    pub weapon_id: String,
    pub acquired_port: String,
    pub acquired_day: i64,
    pub acquired_region: String,
    pub kills: i64,
    pub named_kills: Vec<String>,
    pub epithet: Option<String>,
    pub custom_name: Option<String>,
    pub times_recognized: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpgradeSnap {
    pub upgrade_id: String,
    pub installed_day: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkillSnap {
    pub id: String,
    pub level: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompanionSnap {
    pub companion_id: String,
    pub role_id: String,
    pub morale: i64,
    pub joined_day: i64,
    pub personality: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EncounterLog {
    pub kind: String,
    pub phase: String,
    pub message: String,
    pub choice: String,
    pub success: bool,
    pub escaped: bool,
    pub hull_damage: i64,
    pub enemy_captain_id: String,
    pub enemy_captain_name: String,
    pub enemy_strength: i64,
    pub turn: i64,
    pub player_action: String,
    pub enemy_action: String,
    pub player_hull_delta: i64,
    pub enemy_hull_delta: i64,
    pub player_crew_delta: i64,
    pub enemy_crew_delta: i64,
    pub boarding_progress: i64,
    pub boarding_threshold: i64,
    pub enemy_sunk: bool,
    pub player_sunk: bool,
    pub boarding_triggered: bool,
    pub flavor: String,
    pub player_hull: i64,
    pub enemy_hull: i64,
    pub player_crew: i64,
    pub enemy_crew: i64,
    pub player_crew_lost: i64,
    pub enemy_crew_lost: i64,
    pub player_advantage: bool,
    pub damage_to_opponent: i64,
    pub damage_to_player: i64,
    pub player_hp: i64,
    pub opponent_hp: i64,
    pub player_stamina_delta: i64,
    pub opponent_stamina_delta: i64,
    pub player_won: bool,
    pub draw: bool,
    pub injury: String,
    pub opponent_injury: String,
    pub style_effect: String,
    pub prize_ok: bool,
    pub prize_reason: String,
    #[serde(skip_serializing_if = "is_zero")]
    pub player_stamina: i64,
    #[serde(skip_serializing_if = "is_zero")]
    pub player_stamina_max: i64,
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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub officers: Vec<OfficerSnap>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub upgrades: Vec<UpgradeSnap>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OfficerSnap {
    pub name: String,
    pub role: String,
    pub origin_port: String,
    #[serde(rename = "trait")]
    pub trait_name: String,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duel: Option<DuelLog>,
    /// Silver from `work`, when that command succeeded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub earned: Option<i64>,
    /// Contract accept, completion, or expiry produced by this command.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub contracts: Vec<ContractLog>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encounter: Option<EncounterLog>,
    /// `tick_sea_captain_agency` result, when that command ran.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agency: Option<AgencyLog>,
    /// Infrastructure and credit messages from this day.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
    /// `GameSession.hunt` result, when that command succeeded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hunt: Option<HuntLog>,
    /// Bounty list, accept, hunt, or claim, when that command succeeded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounty: Option<BountyLog>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HuntLog {
    pub success: bool,
    pub location: String,
    pub provisions_gained: i64,
    pub pelts_gained: i64,
    pub silver_gained: i64,
    pub morale_cost: i64,
    pub crew_lost: i64,
    pub hull_damage: i64,
    pub flavor: String,
    pub danger_text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BountyTargetSnap {
    pub captain_id: String,
    pub captain_name: String,
    pub faction_id: String,
    pub region: String,
    pub reward: i64,
    pub difficulty: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BountyLog {
    pub action: String,
    pub target_id: String,
    pub reward: i64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub targets: Vec<BountyTargetSnap>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContractLog {
    pub contract_id: String,
    pub outcome_type: String,
    pub family: String,
    pub good_id: String,
    pub quantity: i64,
    pub delivered_quantity: i64,
    pub destination_port_id: String,
    pub deadline_day: i64,
    pub reward_silver: i64,
    pub silver_delta: i64,
    pub trust_delta: i64,
    pub standing_delta: i64,
    pub heat_delta: i64,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DuelLog {
    pub opponent_id: String,
    pub opponent_name: String,
    pub player_won: bool,
    pub draw: bool,
    pub silver_delta: i64,
    pub standing_delta: i64,
    pub rounds: Vec<DuelRoundLog>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DuelRoundLog {
    pub player_stance: String,
    pub opponent_stance: String,
    pub damage_to_opponent: i64,
    pub damage_to_player: i64,
    pub flavor: String,
}

impl LogEntry {
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            error: None,
            receipt: None,
            events: Vec::new(),
            shocks: Vec::new(),
            duel: None,
            earned: None,
            contracts: Vec::new(),
            encounter: None,
            agency: None,
            notes: Vec::new(),
            hunt: None,
            bounty: None,
        }
    }
}

pub fn from_contract_outcome(outcome: &ContractOutcome) -> ContractLog {
    ContractLog {
        contract_id: outcome.contract_id.clone(),
        outcome_type: outcome.outcome_type.clone(),
        family: outcome.family.clone(),
        good_id: outcome.good_id.clone(),
        quantity: outcome.required_quantity,
        delivered_quantity: outcome.delivered_quantity,
        destination_port_id: outcome.destination_port_id.clone(),
        deadline_day: outcome.deadline_day,
        reward_silver: outcome.reward_silver,
        silver_delta: outcome.silver_delta,
        trust_delta: outcome.trust_delta,
        standing_delta: outcome.standing_delta,
        heat_delta: outcome.heat_delta,
        summary: outcome.summary.clone(),
    }
}

pub fn from_accepted(contract: &ActiveContract) -> ContractLog {
    ContractLog {
        contract_id: contract.offer_id.clone(),
        outcome_type: "accepted".to_string(),
        family: contract.family.clone(),
        good_id: contract.good_id.clone(),
        quantity: contract.required_quantity,
        delivered_quantity: contract.delivered_quantity,
        destination_port_id: contract.destination_port_id.clone(),
        deadline_day: contract.deadline_day,
        reward_silver: contract.reward_silver,
        silver_delta: 0,
        trust_delta: 0,
        standing_delta: 0,
        heat_delta: 0,
        summary: contract.title.clone(),
    }
}

pub fn from_encounter(step: &EncounterStep) -> EncounterLog {
    EncounterLog {
        kind: step.kind.clone(),
        phase: step.phase.clone(),
        message: step.message.clone(),
        choice: step.choice.clone(),
        success: step.success,
        escaped: step.escaped,
        hull_damage: step.hull_damage,
        enemy_captain_id: step.enemy_captain_id.clone(),
        enemy_captain_name: step.enemy_captain_name.clone(),
        enemy_strength: step.enemy_strength,
        turn: step.turn,
        player_action: step.player_action.clone(),
        enemy_action: step.enemy_action.clone(),
        player_hull_delta: step.player_hull_delta,
        enemy_hull_delta: step.enemy_hull_delta,
        player_crew_delta: step.player_crew_delta,
        enemy_crew_delta: step.enemy_crew_delta,
        boarding_progress: step.boarding_progress,
        boarding_threshold: step.boarding_threshold,
        enemy_sunk: step.enemy_sunk,
        player_sunk: step.player_sunk,
        boarding_triggered: step.boarding_triggered,
        flavor: step.flavor.clone(),
        player_hull: step.player_hull,
        enemy_hull: step.enemy_hull,
        player_crew: step.player_crew,
        enemy_crew: step.enemy_crew,
        player_crew_lost: step.player_crew_lost,
        enemy_crew_lost: step.enemy_crew_lost,
        player_advantage: step.player_advantage,
        damage_to_opponent: step.damage_to_opponent,
        damage_to_player: step.damage_to_player,
        player_hp: step.player_hp,
        opponent_hp: step.opponent_hp,
        player_stamina_delta: step.player_stamina_delta,
        opponent_stamina_delta: step.opponent_stamina_delta,
        player_won: step.player_won,
        draw: step.draw,
        injury: step.injury.clone(),
        opponent_injury: step.opponent_injury.clone(),
        style_effect: step.style_effect.clone(),
        prize_ok: step.prize_ok,
        prize_reason: step.prize_reason.clone(),
        player_stamina: step.player_stamina,
        player_stamina_max: step.player_stamina_max,
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AgencyLog {
    pub ambush: bool,
    pub encounter: Option<AgencyEncounterSnap>,
    pub notices: Vec<AgencyNoticeSnap>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgencyNoticeSnap {
    pub effect_type: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgencyEncounterSnap {
    pub enemy_captain_id: String,
    pub enemy_captain_name: String,
    pub enemy_faction_id: String,
    pub enemy_personality: String,
    pub enemy_strength: i64,
    pub enemy_region: String,
    pub enemy_ship_hull: i64,
    pub enemy_ship_hull_max: i64,
    pub enemy_ship_cannons: i64,
    pub enemy_ship_maneuver: f64,
    pub enemy_ship_speed: f64,
    pub enemy_ship_crew: i64,
    pub enemy_ship_crew_max: i64,
    pub phase: String,
    pub boarding_progress: i64,
    pub boarding_threshold: i64,
    pub naval_turns: i64,
    pub duel_turns: i64,
}

pub fn agency_log(
    ambush: bool,
    encounter: Option<&crate::encounter::EncounterState>,
    notices: &[(String, String)],
) -> AgencyLog {
    AgencyLog {
        ambush,
        encounter: encounter.map(|encounter| AgencyEncounterSnap {
            enemy_captain_id: encounter.enemy_captain_id.clone(),
            enemy_captain_name: encounter.enemy_captain_name.clone(),
            enemy_faction_id: encounter.enemy_faction_id.clone(),
            enemy_personality: encounter.enemy_personality.clone(),
            enemy_strength: encounter.enemy_strength,
            enemy_region: encounter.enemy_region.clone(),
            enemy_ship_hull: encounter.enemy_ship_hull,
            enemy_ship_hull_max: encounter.enemy_ship_hull_max,
            enemy_ship_cannons: encounter.enemy_ship_cannons,
            enemy_ship_maneuver: encounter.enemy_ship_maneuver,
            enemy_ship_speed: encounter.enemy_ship_speed,
            enemy_ship_crew: encounter.enemy_ship_crew,
            enemy_ship_crew_max: encounter.enemy_ship_crew_max,
            phase: encounter.phase.clone(),
            boarding_progress: encounter.boarding_progress,
            boarding_threshold: encounter.boarding_threshold,
            naval_turns: encounter.naval_turns,
            duel_turns: encounter.duel_turns,
        }),
        notices: notices
            .iter()
            .map(|(effect_type, message)| AgencyNoticeSnap {
                effect_type: effect_type.clone(),
                message: message.clone(),
            })
            .collect(),
    }
}

pub fn from_duel(outcome: &DuelOutcome) -> DuelLog {
    DuelLog {
        opponent_id: outcome.opponent_id.clone(),
        opponent_name: outcome.opponent_name.clone(),
        player_won: outcome.player_won,
        draw: outcome.draw,
        silver_delta: outcome.silver_delta,
        standing_delta: outcome.standing_delta,
        rounds: outcome.rounds.iter().map(from_round).collect(),
    }
}

fn from_round(round: &DuelRound) -> DuelRoundLog {
    DuelRoundLog {
        player_stance: round.player_stance.clone(),
        opponent_stance: round.opponent_stance.clone(),
        damage_to_opponent: round.damage_to_opponent,
        damage_to_player: round.damage_to_player,
        flavor: round.flavor.clone(),
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
            officers: ship
                .officers
                .iter()
                .map(|officer| OfficerSnap {
                    name: officer.name.clone(),
                    role: officer.role.clone(),
                    origin_port: officer.origin_port.clone(),
                    trait_name: officer.trait_name.clone(),
                })
                .collect(),
            upgrades: ship
                .upgrades
                .iter()
                .map(|upgrade| UpgradeSnap {
                    upgrade_id: upgrade.upgrade_id.clone(),
                    installed_day: upgrade.installed_day,
                })
                .collect(),
        }),
        standing: standing_snap(&captain.standing),
        encounters: captain
            .encounters
            .iter()
            .map(|record| EncounterRecordSnap {
                captain_id: record.captain_id.clone(),
                faction_id: record.faction_id.clone(),
                day: record.day,
                outcome: record.outcome.clone(),
                region: record.region.clone(),
            })
            .collect(),
        duels_won: captain.duels_won,
        duels_lost: captain.duels_lost,
        naval_victories: captain.naval_victories,
        naval_defeats: captain.naval_defeats,
        fleet: captain
            .fleet
            .iter()
            .map(|owned| FleetSnap {
                template_id: owned.ship.template_id.clone(),
                name: owned.ship.name.clone(),
                hull: owned.ship.hull,
                hull_max: owned.ship.hull_max,
                crew: owned.ship.crew,
                docked_port_id: owned.docked_port_id.clone(),
                cargo: owned
                    .cargo
                    .iter()
                    .map(|item| CargoSnap {
                        good_id: item.good_id.clone(),
                        quantity: item.quantity,
                        cost_basis: item.cost_basis,
                        acquired_port: item.acquired_port.clone(),
                        acquired_region: item.acquired_region.clone(),
                        acquired_day: item.acquired_day,
                    })
                    .collect(),
            })
            .collect(),
        injuries: captain
            .injuries
            .iter()
            .map(|injury| InjurySnap {
                injury_id: injury.injury_id.clone(),
                acquired_day: injury.acquired_day,
                heal_remaining: injury.heal_remaining,
                treated: injury.treated,
            })
            .collect(),
        armor: captain.armor.as_ref().map(|armor| armor.id.clone()),
        melee_weapon: captain.melee.as_ref().map(|weapon| weapon.id.clone()),
        firearm: captain.firearm.as_ref().map(|weapon| weapon.id.clone()),
        weapon_quality: captain.weapon_quality.iter().cloned().collect(),
        weapon_usage: captain.weapon_usage.iter().cloned().collect(),
        weapon_provenance: captain
            .weapon_provenance
            .iter()
            .map(|(id, prov)| {
                (
                    id.clone(),
                    ProvenanceSnap {
                        weapon_id: prov.weapon_id.clone(),
                        acquired_port: prov.acquired_port.clone(),
                        acquired_day: prov.acquired_day,
                        acquired_region: prov.acquired_region.clone(),
                        kills: prov.kills,
                        named_kills: prov.named_kills.clone(),
                        epithet: prov.epithet.clone(),
                        custom_name: prov.custom_name.clone(),
                        times_recognized: prov.times_recognized,
                    },
                )
            })
            .collect(),
        skills: captain
            .skills
            .iter()
            .map(|skill| SkillSnap {
                id: skill.id.clone(),
                level: skill.level,
            })
            .collect(),
        learned_styles: captain.learned_styles.clone(),
        companions: captain
            .party
            .companions
            .iter()
            .map(|member| CompanionSnap {
                companion_id: member.companion_id.clone(),
                role_id: member.role_id.clone(),
                morale: member.morale,
                joined_day: member.joined_day,
                personality: member.personality.clone(),
            })
            .collect(),
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

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct InfraSnap {
    pub warehouses: Vec<WarehouseSnap>,
    pub brokers: Vec<BrokerSnap>,
    pub licenses: Vec<LicenseSnap>,
    pub policies: Vec<PolicySnap>,
    pub claims: Vec<ClaimSnap>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credit: Option<CreditSnap>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct WarehouseSnap {
    pub id: String,
    pub port_id: String,
    pub tier: String,
    pub capacity: i64,
    pub lease_cost: i64,
    pub upkeep_per_day: i64,
    pub inventory: Vec<LotSnap>,
    pub opened_day: i64,
    pub upkeep_paid_through: i64,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LotSnap {
    pub good_id: String,
    pub quantity: i64,
    pub acquired_port: String,
    pub acquired_region: String,
    pub acquired_day: i64,
    pub deposited_day: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct BrokerSnap {
    pub region: String,
    pub tier: String,
    pub opened_day: i64,
    pub upkeep_paid_through: i64,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LicenseSnap {
    pub license_id: String,
    pub purchased_day: i64,
    pub upkeep_paid_through: i64,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PolicySnap {
    pub id: String,
    pub spec_id: String,
    pub family: String,
    pub scope: String,
    pub purchased_day: i64,
    pub coverage_pct: f64,
    pub coverage_cap: i64,
    pub premium_paid: i64,
    pub target_id: String,
    pub claims_made: i64,
    pub total_paid_out: i64,
    pub active: bool,
    pub voyage_origin: String,
    pub voyage_destination: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ClaimSnap {
    pub policy_id: String,
    pub day: i64,
    pub incident_type: String,
    pub loss_value: i64,
    pub payout: i64,
    pub denied: bool,
    pub denial_reason: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CreditSnap {
    pub tier: String,
    pub credit_limit: i64,
    pub outstanding: i64,
    pub interest_accrued: i64,
    pub last_interest_day: i64,
    pub next_due_day: i64,
    pub defaults: i64,
    pub total_borrowed: i64,
    pub total_repaid: i64,
    pub active: bool,
}

pub fn capture(
    world: &World,
    trade_seq: u64,
    books: &HouseBooks,
    infra: &InfrastructureRecord,
    narrative: &NarrativeState,
    log: Vec<LogEntry>,
) -> Snapshot {
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
        infrastructure: infra.is_visible().then(|| infra_snap(infra)),
        culture: culture_snap(&world.culture),
        narrative: narrative_snap(narrative),
        milestones: books
            .completed_milestones
            .iter()
            .map(|completion| MilestoneSnap {
                milestone_id: completion.milestone_id.clone(),
                completed_day: completion.completed_day,
                evidence: completion.evidence.clone(),
                family: content::content()
                    .milestone(&completion.milestone_id)
                    .map(|spec| spec.family.clone())
                    .unwrap_or_default(),
            })
            .collect(),
        log,
    }
}

fn culture_snap(state: &CulturalState) -> Option<CultureSnap> {
    let visible = !state.port_visits.is_empty()
        || !state.regions_entered.is_empty()
        || state.cultural_encounters != 0
        || state.festivals_visited != 0
        || !state.active_festivals.is_empty();
    if !visible {
        return None;
    }
    Some(CultureSnap {
        port_visits: state
            .port_visits
            .iter()
            .map(|(port_id, visits)| VisitSnap {
                port_id: port_id.clone(),
                visits: *visits,
            })
            .collect(),
        regions_entered: state.regions_entered.clone(),
        cultural_encounters: state.cultural_encounters,
        festivals_visited: state.festivals_visited,
        active_festivals: state.active_festivals.iter().map(festival_snap).collect(),
    })
}

fn festival_snap(fest: &ActiveFestival) -> FestivalSnap {
    FestivalSnap {
        festival_id: fest.festival_id.clone(),
        port_id: fest.port_id.clone(),
        start_day: fest.start_day,
        end_day: fest.end_day,
    }
}

fn narrative_snap(state: &NarrativeState) -> Option<NarrativeSnap> {
    if state.fired.is_empty() && state.journal.is_empty() {
        return None;
    }
    Some(NarrativeSnap {
        fired: state.fired.clone(),
        journal: state.journal.iter().map(journal_snap).collect(),
    })
}

fn journal_snap(entry: &JournalEntry) -> JournalSnap {
    JournalSnap {
        beat_id: entry.beat_id.clone(),
        day: entry.day,
        port_id: entry.port_id.clone(),
        region: entry.region.clone(),
    }
}

fn infra_snap(infra: &InfrastructureRecord) -> InfraSnap {
    InfraSnap {
        warehouses: infra.warehouses.iter().map(warehouse_snap).collect(),
        brokers: infra.brokers.iter().map(broker_snap).collect(),
        licenses: infra.licenses.iter().map(license_snap).collect(),
        policies: infra.policies.iter().map(policy_snap).collect(),
        claims: infra.claims.iter().map(claim_snap).collect(),
        credit: infra
            .credit
            .as_ref()
            .filter(credit_visible)
            .map(credit_snap),
    }
}

fn credit_visible(credit: &&CreditState) -> bool {
    credit.active
        || credit.outstanding != 0
        || credit.interest_accrued != 0
        || credit.defaults != 0
        || credit.total_borrowed != 0
        || credit.total_repaid != 0
}

fn warehouse_snap(lease: &WarehouseLease) -> WarehouseSnap {
    WarehouseSnap {
        id: lease.id.clone(),
        port_id: lease.port_id.clone(),
        tier: lease.tier.clone(),
        capacity: lease.capacity,
        lease_cost: lease.lease_cost,
        upkeep_per_day: lease.upkeep_per_day,
        inventory: lease.inventory.iter().map(lot_snap).collect(),
        opened_day: lease.opened_day,
        upkeep_paid_through: lease.upkeep_paid_through,
        active: lease.active,
    }
}

fn lot_snap(lot: &crate::model::StoredLot) -> LotSnap {
    LotSnap {
        good_id: lot.good_id.clone(),
        quantity: lot.quantity,
        acquired_port: lot.acquired_port.clone(),
        acquired_region: lot.acquired_region.clone(),
        acquired_day: lot.acquired_day,
        deposited_day: lot.deposited_day,
    }
}

fn broker_snap(broker: &BrokerOffice) -> BrokerSnap {
    BrokerSnap {
        region: broker.region.clone(),
        tier: broker.tier.clone(),
        opened_day: broker.opened_day,
        upkeep_paid_through: broker.upkeep_paid_through,
        active: broker.active,
    }
}

fn license_snap(license: &OwnedLicense) -> LicenseSnap {
    LicenseSnap {
        license_id: license.license_id.clone(),
        purchased_day: license.purchased_day,
        upkeep_paid_through: license.upkeep_paid_through,
        active: license.active,
    }
}

fn policy_snap(policy: &ActivePolicy) -> PolicySnap {
    PolicySnap {
        id: policy.id.clone(),
        spec_id: policy.spec_id.clone(),
        family: policy.family.clone(),
        scope: policy.scope.clone(),
        purchased_day: policy.purchased_day,
        coverage_pct: policy.coverage_pct,
        coverage_cap: policy.coverage_cap,
        premium_paid: policy.premium_paid,
        target_id: policy.target_id.clone(),
        claims_made: policy.claims_made,
        total_paid_out: policy.total_paid_out,
        active: policy.active,
        voyage_origin: policy.voyage_origin.clone(),
        voyage_destination: policy.voyage_destination.clone(),
    }
}

fn claim_snap(claim: &InsuranceClaim) -> ClaimSnap {
    ClaimSnap {
        policy_id: claim.policy_id.clone(),
        day: claim.day,
        incident_type: claim.incident_type.clone(),
        loss_value: claim.loss_value,
        payout: claim.payout,
        denied: claim.denied,
        denial_reason: claim.denial_reason.clone(),
    }
}

fn credit_snap(credit: &CreditState) -> CreditSnap {
    CreditSnap {
        tier: credit.tier.clone(),
        credit_limit: credit.credit_limit,
        outstanding: credit.outstanding,
        interest_accrued: credit.interest_accrued,
        last_interest_day: credit.last_interest_day,
        next_due_day: credit.next_due_day,
        defaults: credit.defaults,
        total_borrowed: credit.total_borrowed,
        total_repaid: credit.total_repaid,
        active: credit.active,
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
            encounters: Vec::new(),
            duels_won: 0,
            duels_lost: 0,
            naval_victories: 0,
            naval_defeats: 0,
            fleet: Vec::new(),
            skills: Vec::new(),
            learned_styles: Vec::new(),
            companions: Vec::new(),
            injuries: Vec::new(),
            armor: None,
            melee_weapon: None,
            firearm: None,
            weapon_quality: BTreeMap::new(),
            weapon_usage: BTreeMap::new(),
            weapon_provenance: BTreeMap::new(),
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
        infrastructure: None,
        milestones: Vec::new(),
        culture: None,
        narrative: None,
        log,
    }
}
