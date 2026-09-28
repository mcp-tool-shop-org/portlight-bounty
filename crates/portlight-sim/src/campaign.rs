//! The four victory paths from `portlight.engine.campaign`.
//!
//! Path id `commercial_empire` and milestone family `commercial_finance` are
//! different strings in Python. This module keeps both. `evaluate_milestones`
//! records completions under the family id and does not rename the path.
//!
//! Candidate strength uses Python `round(strength, 1)`, then clamps at 0.

use serde::Serialize;

use crate::content::{self, class_rank};
use crate::model::World;
use crate::util::py_round_places;

pub const PATH_LAWFUL_HOUSE: &str = "lawful_house";
pub const PATH_SHADOW_NETWORK: &str = "shadow_network";
pub const PATH_OCEANIC_REACH: &str = "oceanic_reach";
/// Victory-path id. Not the milestone family.
pub const PATH_COMMERCIAL_EMPIRE: &str = "commercial_empire";
/// `MilestoneFamily.COMMERCIAL_FINANCE`. Not a victory-path id.
pub const MILESTONE_FAMILY_COMMERCIAL_FINANCE: &str = "commercial_finance";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MilestoneFamily {
    RegionalFoothold,
    LawfulHouse,
    ShadowNetwork,
    OceanicReach,
    CommercialFinance,
    IntegratedHouse,
}

impl MilestoneFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RegionalFoothold => "regional_foothold",
            Self::LawfulHouse => "lawful_house",
            Self::ShadowNetwork => "shadow_network",
            Self::OceanicReach => "oceanic_reach",
            Self::CommercialFinance => MILESTONE_FAMILY_COMMERCIAL_FINANCE,
            Self::IntegratedHouse => "integrated_house",
        }
    }
}

/// Families that feed each career-profile tag. `commercial_finance` is the
/// family string; the victory path that uses the same theme is
/// [`PATH_COMMERCIAL_EMPIRE`].
pub const PROFILE_MILESTONE_FAMILIES: &[(&str, &[&str])] = &[
    ("Lawful House", &["lawful_house", "regional_foothold"]),
    ("Shadow Operator", &["shadow_network"]),
    ("Oceanic Carrier", &["oceanic_reach"]),
    (
        "Contract Specialist",
        &["regional_foothold", "integrated_house"],
    ),
    (
        "Infrastructure Builder",
        &["integrated_house", "commercial_finance"],
    ),
    ("Leveraged Trader", &["commercial_finance"]),
    ("Risk-Managed Merchant", &["commercial_finance"]),
];

#[derive(Debug, Clone)]
pub struct CompletedContract {
    pub outcome_type: String,
    /// `None` matches a Python contract whose `family` is missing.
    pub family: Option<String>,
    pub summary: String,
}

#[derive(Debug, Clone)]
pub struct WarehouseSite {
    pub port_id: String,
    pub active: bool,
}

#[derive(Debug, Clone)]
pub struct BrokerSite {
    pub region: String,
    pub active: bool,
    /// `"none"` is `BrokerTier.NONE` and does not count.
    pub tier: String,
}

#[derive(Debug, Clone)]
pub struct ActiveLicense {
    pub license_id: String,
    pub active: bool,
}

#[derive(Debug, Clone)]
pub struct CreditBook {
    pub total_borrowed: i64,
    pub defaults: i64,
    /// `CreditState.active`. Absent credit is `None`, not an inactive book.
    pub active: bool,
    pub total_repaid: i64,
}

#[derive(Debug, Clone)]
pub struct VictoryRecord {
    pub path_id: String,
    pub completion_day: i64,
    pub summary: String,
    pub is_first: bool,
}

/// Ledger, contracts, and infrastructure the victory evaluators read.
///
/// Buy and sell update the ledger. The contract, license, warehouse, broker,
/// insurance, and credit systems are not ported, so those lists stay empty
/// until something records them here.
#[derive(Debug, Clone, Default)]
pub struct HouseBooks {
    pub total_buys: i64,
    pub total_sells: i64,
    pub net_profit: i64,
    pub trade_count: i64,
    pub completed_contracts: Vec<CompletedContract>,
    pub warehouses: Vec<WarehouseSite>,
    pub brokers: Vec<BrokerSite>,
    pub licenses: Vec<ActiveLicense>,
    pub policies: i64,
    /// Paid, non-denied insurance claims. Area 4 writes this.
    pub claims_paid: i64,
    pub credit: Option<CreditBook>,
    pub completed_paths: Vec<VictoryRecord>,
    pub completed_milestones: Vec<MilestoneCompletion>,
}

/// A milestone that `evaluate_milestones` just recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MilestoneCompletion {
    pub milestone_id: String,
    pub completed_day: i64,
    pub evidence: String,
}

impl HouseBooks {
    pub fn note_receipt(&mut self, action: &str, total_price: i64) {
        if action == "buy" {
            self.total_buys += total_price;
        } else {
            self.total_sells += total_price;
        }
        self.net_profit = self.total_sells - self.total_buys;
        self.trade_count += 1;
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct VictoryRequirement {
    pub description: String,
    pub status: String,
    pub detail: String,
    pub action: String,
}

impl VictoryRequirement {
    fn met(&self) -> bool {
        self.status == "met"
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct VictoryPathStatus {
    pub path_id: String,
    pub name: String,
    pub candidate_strength: f64,
    pub completion_day: i64,
    pub completion_summary: String,
    pub requirements: Vec<VictoryRequirement>,
}

impl VictoryPathStatus {
    fn is_complete(&self) -> bool {
        self.requirements.iter().all(VictoryRequirement::met)
    }
}

fn req(
    description: impl Into<String>,
    met: bool,
    detail: impl Into<String>,
    action: impl Into<String>,
    blocker: bool,
) -> VictoryRequirement {
    let status = if met {
        "met"
    } else if blocker {
        "blocked"
    } else {
        "missing"
    };
    VictoryRequirement {
        description: description.into(),
        status: status.to_string(),
        detail: detail.into(),
        action: action.into(),
    }
}

fn trust_tier(trust: i64) -> &'static str {
    if trust >= 40 {
        "trusted"
    } else if trust >= 25 {
        "reliable"
    } else if trust >= 10 {
        "credible"
    } else if trust >= 1 {
        "new"
    } else {
        "unproven"
    }
}

fn trust_rank(tier: &str) -> i64 {
    match tier {
        "unproven" => 0,
        "new" => 1,
        "credible" => 2,
        "reliable" => 3,
        "trusted" => 4,
        _ => 0,
    }
}

fn regions_at(world: &World, min_standing: i64) -> Vec<&'static str> {
    content::REGIONS
        .iter()
        .copied()
        .enumerate()
        .filter(|(index, _)| world.captain.standing.regional[*index] >= min_standing)
        .map(|(_, region)| region)
        .collect()
}

fn max_heat(world: &World) -> i64 {
    world
        .captain
        .standing
        .heat
        .iter()
        .copied()
        .max()
        .unwrap_or(0)
}

fn ship_class(world: &World) -> String {
    world
        .captain
        .ship
        .as_ref()
        .and_then(|ship| content::content().ship(&ship.template_id))
        .map(|ship| ship.ship_class.clone())
        .unwrap_or_else(|| "sloop".to_string())
}

fn completed_contracts(books: &HouseBooks) -> usize {
    books
        .completed_contracts
        .iter()
        .filter(|contract| {
            matches!(
                contract.outcome_type.as_str(),
                "completed" | "completed_bonus"
            )
        })
        .count()
}

fn discreet_completions(books: &HouseBooks) -> usize {
    books
        .completed_contracts
        .iter()
        .filter(|contract| {
            if !matches!(
                contract.outcome_type.as_str(),
                "completed" | "completed_bonus"
            ) {
                return false;
            }
            match contract.family.as_deref() {
                Some("luxury_discreet") => true,
                None => {
                    let summary = contract.summary.to_lowercase();
                    summary.contains("luxury") || summary.contains("discreet")
                }
                Some(_) => false,
            }
        })
        .count()
}

fn seizure_count(world: &World) -> usize {
    world
        .captain
        .standing
        .incidents
        .iter()
        .filter(|incident| incident.description.to_lowercase().contains("seized"))
        .count()
}

fn active_licenses(books: &HouseBooks) -> usize {
    books
        .licenses
        .iter()
        .filter(|license| license.active)
        .count()
}

fn has_license(books: &HouseBooks, license_id: &str) -> bool {
    books
        .licenses
        .iter()
        .any(|license| license.active && license.license_id == license_id)
}

fn broker_regions(books: &HouseBooks) -> Vec<String> {
    let mut regions: Vec<String> = books
        .brokers
        .iter()
        .filter(|broker| broker.active && broker.tier != "none")
        .map(|broker| broker.region.clone())
        .collect();
    regions.sort();
    regions.dedup();
    regions
}

fn warehouse_regions(world: &World, books: &HouseBooks) -> Vec<String> {
    let mut regions: Vec<String> = books
        .warehouses
        .iter()
        .filter(|site| site.active)
        .filter_map(|site| world.port(&site.port_id).map(|port| port.region.clone()))
        .collect();
    regions.sort();
    regions.dedup();
    regions
}

fn infra_regions(world: &World, books: &HouseBooks) -> Vec<String> {
    let mut regions = warehouse_regions(world, books);
    for region in broker_regions(books) {
        if !regions.contains(&region) {
            regions.push(region);
        }
    }
    regions.sort();
    regions
}

fn scored(strength: f64) -> f64 {
    py_round_places(strength, 1).max(0.0)
}

fn met_ratio(reqs: &[VictoryRequirement]) -> f64 {
    let met = reqs.iter().filter(|req| req.met()).count();
    met as f64 / reqs.len() as f64 * 100.0
}

fn completion_summary(path_id: &str) -> &'static str {
    match path_id {
        PATH_LAWFUL_HOUSE => {
            "Your company earned trust across multiple regions, secured premium charters, and scaled lawful commerce without surrendering discipline to heat."
        }
        PATH_SHADOW_NETWORK => {
            "Your operation survived scrutiny, moved sensitive luxury cargo profitably, and built a resilient gray-market network under pressure."
        }
        PATH_OCEANIC_REACH => {
            "Your house established East Indies access, commercialized long-haul routes, and proved that distant trade could be run at serious scale."
        }
        PATH_COMMERCIAL_EMPIRE => {
            "You built an integrated trade concern with infrastructure, access, finance, and multi-region business power beyond a single ship or route."
        }
        _ => "",
    }
}

fn evaluate_lawful(world: &World, books: &HouseBooks) -> VictoryPathStatus {
    let tier = trust_tier(world.captain.standing.commercial_trust);
    let rank = trust_rank(tier);
    let max_h = max_heat(world);
    let contracts = completed_contracts(books);
    let regions_15 = regions_at(world, 15);
    let lic_count = active_licenses(books);
    let regions_listed = if regions_15.is_empty() {
        "none".to_string()
    } else {
        regions_15.join(", ")
    };
    let breadth_met = lic_count >= 2 || regions_15.len() >= 2;
    let breadth_action = if lic_count < 2 && regions_15.len() < 2 {
        format!(
            "Acquire {} more license(s) or build standing in another region",
            2 - lic_count
        )
    } else {
        String::new()
    };
    let contract_action = if contracts < 8 {
        format!("Complete {} more contracts", 8 - contracts)
    } else {
        String::new()
    };
    let silver_action = if world.captain.silver < 2000 {
        format!("Earn {} more silver", 2000 - world.captain.silver)
    } else {
        String::new()
    };
    let reqs = vec![
        req(
            "Trusted commercial standing",
            rank >= 4,
            format!("Currently: {tier}"),
            format!("Reach trusted trust tier (currently {tier})"),
            false,
        ),
        req(
            "High Reputation Commercial Charter",
            has_license(books, "high_rep_charter"),
            "",
            "Acquire the High Reputation Commercial Charter",
            false,
        ),
        req(
            "Regional breadth (2+ licenses or standing 15+ in 2 regions)",
            breadth_met,
            format!("Licenses: {lic_count}, regions at 15+: {regions_listed}"),
            breadth_action,
            false,
        ),
        req(
            "8+ contracts completed",
            contracts >= 8,
            format!("Completed: {contracts}"),
            contract_action,
            false,
        ),
        req(
            "Max heat <= 5",
            max_h <= 5,
            format!("Max heat: {max_h}"),
            format!("Reduce customs heat from {max_h} to 5 or below"),
            max_h > 5,
        ),
        req(
            "2000+ silver",
            world.captain.silver >= 2000,
            format!("Silver: {}", world.captain.silver),
            silver_action,
            false,
        ),
    ];
    let mut strength = met_ratio(&reqs);
    strength += (rank - 2).max(0) as f64 * 5.0;
    if regions_at(world, 10).len() >= 2 {
        strength += 8.0;
    }
    if max_h <= 3 {
        strength += 10.0;
    }
    strength += seizure_count(world) as f64 * -15.0;
    if max_h > 5 {
        strength += (max_h - 5) as f64 * -3.0;
    }
    if books
        .credit
        .as_ref()
        .is_some_and(|credit| credit.defaults > 0)
    {
        strength += -20.0;
    }
    VictoryPathStatus {
        path_id: PATH_LAWFUL_HOUSE.to_string(),
        name: "Lawful Trade House".to_string(),
        requirements: reqs,
        candidate_strength: scored(strength),
        completion_day: 0,
        completion_summary: String::new(),
    }
}

fn evaluate_shadow(world: &World, books: &HouseBooks) -> VictoryPathStatus {
    let max_h = max_heat(world);
    let profit = books.net_profit;
    let trades = books.trade_count;
    let discreet = discreet_completions(books);
    let seizures = seizure_count(world);
    let discreet_action = if discreet < 2 {
        format!("Complete {} more discreet luxury deliveries", 2 - discreet)
    } else {
        String::new()
    };
    let silver_action = if world.captain.silver < 1500 {
        format!("Accumulate {} more silver", 1500 - world.captain.silver)
    } else {
        String::new()
    };
    let reqs = vec![
        req(
            "2+ discreet luxury completions",
            discreet >= 2,
            format!("Discreet completions: {discreet}"),
            discreet_action,
            false,
        ),
        req(
            "Operated under meaningful heat (>= 10)",
            max_h >= 10,
            format!("Max heat: {max_h}"),
            format!(
                "Shadow commerce requires operating under customs pressure (heat {max_h}, need 10+)"
            ),
            max_h < 10 && profit > 1000,
        ),
        req(
            "Heat manageable (<= 40)",
            max_h <= 40,
            format!("Max heat: {max_h}"),
            format!("Reduce heat from {max_h} — network collapses above 40"),
            max_h > 40,
        ),
        req(
            "Profitable under pressure (profit >= 2000)",
            profit >= 2000 && max_h >= 10,
            format!("Profit: {profit}, heat: {max_h}"),
            "Build profitability while maintaining shadow operations",
            false,
        ),
        req(
            "1500+ silver on hand",
            world.captain.silver >= 1500,
            format!("Silver: {}", world.captain.silver),
            silver_action,
            false,
        ),
        req(
            "Trade volume under heat (8+ trades)",
            trades >= 8 && max_h >= 10,
            format!("Trades: {trades}, max heat: {max_h}"),
            "Complete more trades while operating under customs pressure",
            false,
        ),
    ];
    let mut strength = met_ratio(&reqs);
    strength += discreet as f64 * 6.0;
    if profit >= 2000 && max_h >= 10 {
        strength += 10.0;
    }
    if seizures > 0 && world.captain.silver >= 200 {
        strength += 8.0;
    }
    if max_h < 3 {
        strength += -20.0;
    }
    if world.captain.silver < 100 {
        strength += -25.0;
    }
    VictoryPathStatus {
        path_id: PATH_SHADOW_NETWORK.to_string(),
        name: "Shadow Network".to_string(),
        requirements: reqs,
        candidate_strength: scored(strength),
        completion_day: 0,
        completion_summary: String::new(),
    }
}

fn evaluate_oceanic(world: &World, books: &HouseBooks) -> VictoryPathStatus {
    let ei_standing = world.captain.standing.regional_of("East Indies");
    let contracts = completed_contracts(books);
    let ship = ship_class(world);
    let has_ei_charter = has_license(books, "ei_access_charter");
    let brokers = broker_regions(books);
    let warehouses = warehouse_regions(world, books);
    let ei_broker = brokers.iter().any(|region| region == "East Indies");
    let ei_warehouse = warehouses.iter().any(|region| region == "East Indies");
    let ship_ok = class_rank(&ship) >= class_rank("brigantine");
    let contract_action = if contracts < 5 {
        format!("Complete {} more contracts", 5 - contracts)
    } else {
        String::new()
    };
    let silver_action = if world.captain.silver < 2000 {
        format!("Earn {} more silver", 2000 - world.captain.silver)
    } else {
        String::new()
    };
    let reqs = vec![
        req(
            "East Indies Access Charter",
            has_ei_charter,
            "",
            "Acquire the East Indies Access Charter",
            false,
        ),
        req(
            "East Indies commercial foothold (broker or warehouse)",
            ei_broker || ei_warehouse,
            format!(
                "EI broker: {}, EI warehouse: {}",
                if ei_broker { "yes" } else { "no" },
                if ei_warehouse { "yes" } else { "no" }
            ),
            "Open a broker office or warehouse in the East Indies",
            false,
        ),
        req(
            "East Indies standing >= 15",
            ei_standing >= 15,
            format!("EI standing: {ei_standing}"),
            format!("Build East Indies standing from {ei_standing} to 15"),
            false,
        ),
        req(
            "Long-haul ship capability (Brigantine or Galleon)",
            ship_ok,
            format!("Ship: {ship}"),
            "Upgrade to a Brigantine or Galleon for long-haul routes",
            false,
        ),
        req(
            "5+ contracts completed",
            contracts >= 5,
            format!("Completed: {contracts}"),
            contract_action,
            false,
        ),
        req(
            "2000+ silver",
            world.captain.silver >= 2000,
            format!("Silver: {}", world.captain.silver),
            silver_action,
            false,
        ),
    ];
    let mut strength = met_ratio(&reqs);
    strength += ei_standing as f64 * 2.0;
    if ship == "galleon" {
        strength += 15.0;
    }
    if ei_broker && ei_warehouse {
        strength += 10.0;
    }
    if ei_standing == 0 && !has_ei_charter {
        strength += -15.0;
    }
    VictoryPathStatus {
        path_id: PATH_OCEANIC_REACH.to_string(),
        name: "Oceanic Reach".to_string(),
        requirements: reqs,
        candidate_strength: scored(strength),
        completion_day: 0,
        completion_summary: String::new(),
    }
}

fn evaluate_empire(world: &World, books: &HouseBooks) -> VictoryPathStatus {
    let tier = trust_tier(world.captain.standing.commercial_trust);
    let rank = trust_rank(tier);
    let regions = infra_regions(world, books);
    let contracts = completed_contracts(books);
    let lic_count = active_licenses(books);
    let credit_used = books
        .credit
        .as_ref()
        .is_some_and(|credit| credit.total_borrowed > 0);
    let insurance_used = books.policies >= 1;
    let finance_ok = credit_used && insurance_used;
    let listed = if regions.is_empty() {
        "none".to_string()
    } else {
        regions.join(", ")
    };
    let infra_action = if regions.len() < 3 {
        format!(
            "Expand infrastructure to {} more region(s)",
            3 - regions.len()
        )
    } else {
        String::new()
    };
    let contract_action = if contracts < 10 {
        format!("Complete {} more contracts", 10 - contracts)
    } else {
        String::new()
    };
    let silver_action = if world.captain.silver < 3000 {
        format!("Earn {} more silver", 3000 - world.captain.silver)
    } else {
        String::new()
    };
    let license_action = if lic_count < 3 {
        format!("Acquire {} more license(s)", 3 - lic_count)
    } else {
        String::new()
    };
    let finance_action = format!(
        "{}{}{}",
        if insurance_used { "" } else { "Use insurance" },
        if !insurance_used && !credit_used {
            " and "
        } else {
            ""
        },
        if credit_used { "" } else { "Draw on credit" },
    );
    let reqs = vec![
        req(
            "Infrastructure in 3 regions",
            regions.len() >= 3,
            format!("Regions: {listed}"),
            infra_action,
            false,
        ),
        req(
            "Reliable+ trust",
            rank >= 3,
            format!("Trust: {tier}"),
            format!("Build trust to reliable tier (currently {tier})"),
            false,
        ),
        req(
            "Insurance and credit both used successfully",
            finance_ok,
            format!(
                "Insurance: {}, Credit: {}",
                if insurance_used { "yes" } else { "no" },
                if credit_used { "yes" } else { "no" }
            ),
            finance_action,
            false,
        ),
        req(
            "10+ contracts completed",
            contracts >= 10,
            format!("Completed: {contracts}"),
            contract_action,
            false,
        ),
        req(
            "3000+ silver",
            world.captain.silver >= 3000,
            format!("Silver: {}", world.captain.silver),
            silver_action,
            false,
        ),
        req(
            "3+ active licenses",
            lic_count >= 3,
            format!("Active: {lic_count}"),
            license_action,
            false,
        ),
    ];
    let mut strength = met_ratio(&reqs);
    strength += regions.len() as f64 * 5.0;
    if finance_ok {
        strength += 10.0;
    }
    if contracts >= 10 {
        strength += 8.0;
    }
    if regions.len() <= 1 {
        strength += -10.0;
    }
    if books
        .credit
        .as_ref()
        .is_some_and(|credit| credit.defaults > 0)
    {
        strength += -15.0;
    }
    VictoryPathStatus {
        path_id: PATH_COMMERCIAL_EMPIRE.to_string(),
        name: "Commercial Empire".to_string(),
        requirements: reqs,
        candidate_strength: scored(strength),
        completion_day: 0,
        completion_summary: String::new(),
    }
}

fn attach_records(books: &HouseBooks, mut path: VictoryPathStatus) -> VictoryPathStatus {
    if let Some(record) = books
        .completed_paths
        .iter()
        .find(|record| record.path_id == path.path_id)
    {
        path.completion_day = record.completion_day;
        path.completion_summary = record.summary.clone();
    }
    if path.is_complete() && path.completion_summary.is_empty() {
        path.completion_summary = completion_summary(&path.path_id).to_string();
    }
    path
}

/// Evaluate all four paths. Sorted by candidate strength, highest first.
/// Ties keep lawful, shadow, oceanic, empire order.
pub fn compute_victory_progress(world: &World, books: &HouseBooks) -> Vec<VictoryPathStatus> {
    let mut paths = vec![
        attach_records(books, evaluate_lawful(world, books)),
        attach_records(books, evaluate_shadow(world, books)),
        attach_records(books, evaluate_oceanic(world, books)),
        attach_records(books, evaluate_empire(world, books)),
    ];
    paths.sort_by(|left, right| right.candidate_strength.total_cmp(&left.candidate_strength));
    paths
}

/// Paths that just became complete and are not already recorded.
pub fn evaluate_victory_closure(world: &World, books: &HouseBooks) -> Vec<VictoryRecord> {
    let already: Vec<&str> = books
        .completed_paths
        .iter()
        .map(|record| record.path_id.as_str())
        .collect();
    let is_first = already.is_empty();
    let mut newly = Vec::new();
    for path in [
        evaluate_lawful(world, books),
        evaluate_shadow(world, books),
        evaluate_oceanic(world, books),
        evaluate_empire(world, books),
    ] {
        if already.iter().any(|id| *id == path.path_id) {
            continue;
        }
        if path.is_complete() {
            newly.push(VictoryRecord {
                path_id: path.path_id.clone(),
                completion_day: world.day,
                summary: completion_summary(&path.path_id).to_string(),
                is_first: is_first && newly.is_empty(),
            });
        }
    }
    newly
}

fn min_heat(world: &World) -> i64 {
    world
        .captain
        .standing
        .heat
        .iter()
        .copied()
        .min()
        .unwrap_or(0)
}

fn active_warehouse_count(books: &HouseBooks) -> usize {
    books.warehouses.iter().filter(|site| site.active).count()
}

fn active_broker_count(books: &HouseBooks) -> usize {
    books
        .brokers
        .iter()
        .filter(|broker| broker.active && broker.tier != "none")
        .count()
}

fn inspection_seizure(world: &World) -> bool {
    world.captain.standing.incidents.iter().any(|incident| {
        incident.incident_type == "inspection"
            && incident.description.to_lowercase().contains("seized")
    })
}

fn eval_milestone(evaluator: &str, world: &World, books: &HouseBooks) -> (bool, String) {
    let tier = trust_tier(world.captain.standing.commercial_trust);
    let rank = trust_rank(tier);
    let heat = max_heat(world);
    let class_name = ship_class(world);
    match evaluator {
        "first_warehouse" => first_active_warehouse(books)
            .map(|site| (true, format!("Warehouse at {}", site.port_id)))
            .unwrap_or((false, String::new())),
        "first_broker" => first_active_broker(books)
            .map(|broker| (true, format!("Broker in {}", broker.region)))
            .unwrap_or((false, String::new())),
        "standing_one_region" => {
            let regions = regions_at(world, 10);
            regions
                .first()
                .map(|region| (true, format!("Standing 10+ in {region}")))
                .unwrap_or((false, String::new()))
        }
        "strong_standing_one_region" => {
            let regions = regions_at(world, 25);
            regions
                .first()
                .map(|region| (true, format!("Standing 25+ in {region}")))
                .unwrap_or((false, String::new()))
        }
        "presence_two_regions" => {
            let regions = regions_at(world, 5);
            if regions.len() >= 2 {
                (
                    true,
                    format!("Established in {}, {}", regions[0], regions[1]),
                )
            } else {
                (false, String::new())
            }
        }
        "sustained_three_regions" => {
            let regions = regions_at(world, 10);
            if regions.len() >= 3 {
                (true, "Standing 10+ in all three regions".to_string())
            } else {
                (false, String::new())
            }
        }
        "credible_trust" => {
            if rank >= 2 {
                (true, format!("Trust tier: {tier}"))
            } else {
                (false, String::new())
            }
        }
        "reliable_trust" => {
            if rank >= 3 {
                (true, format!("Trust tier: {tier}"))
            } else {
                (false, String::new())
            }
        }
        "regional_charter" => {
            for license_id in [
                "med_trade_charter",
                "wa_commerce_permit",
                "ei_access_charter",
            ] {
                if has_license(books, license_id) {
                    return (true, format!("License: {license_id}"));
                }
            }
            (false, String::new())
        }
        "high_rep_charter" => {
            if has_license(books, "high_rep_charter") {
                (
                    true,
                    "High Reputation Commercial Charter acquired".to_string(),
                )
            } else {
                (false, String::new())
            }
        }
        "lawful_contracts_completed" => {
            let count = completed_contracts(books);
            if count >= 5 {
                (true, format!("{count} contracts completed successfully"))
            } else {
                (false, String::new())
            }
        }
        "low_heat_scaling" => {
            if rank >= 3 && heat <= 5 {
                (true, format!("Reliable trust with max heat {heat}"))
            } else {
                (false, String::new())
            }
        }
        "first_discreet_success" => {
            if discreet_completions(books) >= 1 {
                (true, "First discreet luxury delivery".to_string())
            } else {
                (false, String::new())
            }
        }
        "elevated_heat_sustained" => {
            if heat >= 15 && world.captain.silver >= 500 {
                (
                    true,
                    format!(
                        "Operating at heat {heat} with {} silver",
                        world.captain.silver
                    ),
                )
            } else {
                (false, String::new())
            }
        }
        "shadow_profitability" => {
            if books.net_profit >= 2000 && heat >= 10 {
                (
                    true,
                    format!("Profit {} with heat {heat}", books.net_profit),
                )
            } else {
                (false, String::new())
            }
        }
        "seizure_recovery" => {
            if inspection_seizure(world) && world.captain.silver >= 300 {
                (true, "Recovered from cargo seizure".to_string())
            } else {
                (false, String::new())
            }
        }
        "ei_access" => {
            if has_license(books, "ei_access_charter") {
                (true, "East Indies Access Charter acquired".to_string())
            } else {
                (false, String::new())
            }
        }
        "ei_broker" => {
            if broker_regions(books)
                .iter()
                .any(|region| region == "East Indies")
            {
                (true, "Broker office in East Indies".to_string())
            } else {
                (false, String::new())
            }
        }
        "galleon_deployed" => {
            if matches!(class_name.as_str(), "galleon" | "man_of_war") {
                (true, format!("Operating a {class_name}"))
            } else {
                (false, String::new())
            }
        }
        "ei_standing" => {
            let standing = world.captain.standing.regional_of("East Indies");
            if standing >= 15 {
                (true, format!("East Indies standing: {standing}"))
            } else {
                (false, String::new())
            }
        }
        "first_insurance_success" => {
            if books.claims_paid >= 1 {
                (true, format!("{} insurance claims paid", books.claims_paid))
            } else {
                (false, String::new())
            }
        }
        "credit_opened" => match &books.credit {
            Some(credit) if credit.total_borrowed > 0 => (
                true,
                format!("Credit used, {} total borrowed", credit.total_borrowed),
            ),
            _ => (false, String::new()),
        },
        "credit_clean" => match &books.credit {
            Some(credit)
                if credit.active && credit.defaults == 0 && credit.total_borrowed >= 200 =>
            {
                (
                    true,
                    format!("Borrowed {} with no defaults", credit.total_borrowed),
                )
            }
            _ => (false, String::new()),
        },
        "leveraged_expansion" => {
            let warehouses = active_warehouse_count(books);
            let brokers = active_broker_count(books);
            match &books.credit {
                Some(credit)
                    if credit.total_borrowed >= 300
                        && credit.defaults == 0
                        && warehouses + brokers >= 3 =>
                {
                    (
                        true,
                        format!(
                            "Borrowed {}, {warehouses} warehouses + {brokers} brokers",
                            credit.total_borrowed
                        ),
                    )
                }
                _ => (false, String::new()),
            }
        }
        "multi_region_infra" => {
            let regions = infra_regions(world, books);
            if regions.len() >= 2 {
                (true, format!("Infrastructure in {}", regions.join(", ")))
            } else {
                (false, String::new())
            }
        }
        "full_spectrum" => {
            let warehouse = active_warehouse_count(books) >= 1;
            let brokers = active_broker_count(books) >= 1;
            let licenses = active_licenses(books) >= 1;
            let insured = books.policies >= 1;
            let credit = books
                .credit
                .as_ref()
                .is_some_and(|credit| credit.total_borrowed > 0);
            let met = [warehouse, brokers, licenses, insured, credit]
                .into_iter()
                .filter(|flag| *flag)
                .count();
            if met >= 4 {
                let mut parts = Vec::new();
                if warehouse {
                    parts.push("warehouse");
                }
                if brokers {
                    parts.push("broker");
                }
                if licenses {
                    parts.push("license");
                }
                if insured {
                    parts.push("insurance");
                }
                if credit {
                    parts.push("credit");
                }
                (true, format!("Using {}", parts.join(", ")))
            } else {
                (false, String::new())
            }
        }
        "major_contracts_multi_region" => {
            let contracts = completed_contracts(books);
            let regions = regions_at(world, 10);
            if contracts >= 5 && regions.len() >= 2 {
                (
                    true,
                    format!("{contracts} contracts, standing in {}", regions.join(", ")),
                )
            } else {
                (false, String::new())
            }
        }
        "brigantine_acquired" => {
            if matches!(
                class_name.as_str(),
                "cutter" | "brigantine" | "galleon" | "man_of_war"
            ) {
                (true, format!("Operating a {class_name}"))
            } else {
                (false, String::new())
            }
        }
        _ => (false, String::new()),
    }
}

fn first_active_warehouse(books: &HouseBooks) -> Option<&WarehouseSite> {
    books.warehouses.iter().find(|site| site.active)
}

fn first_active_broker(books: &HouseBooks) -> Option<&BrokerSite> {
    books
        .brokers
        .iter()
        .find(|broker| broker.active && broker.tier != "none")
}

/// Newly completed milestones, in catalog order. Already-recorded ids are skipped.
pub fn evaluate_milestones(world: &World, books: &HouseBooks) -> Vec<MilestoneCompletion> {
    let mut already: Vec<String> = books
        .completed_milestones
        .iter()
        .map(|completion| completion.milestone_id.clone())
        .collect();
    let mut newly = Vec::new();
    for spec in &content::content().campaign.milestones {
        if already.iter().any(|id| id == &spec.id) {
            continue;
        }
        let (met, evidence) = eval_milestone(&spec.evaluator, world, books);
        if met {
            newly.push(MilestoneCompletion {
                milestone_id: spec.id.clone(),
                completed_day: world.day,
                evidence,
            });
            already.push(spec.id.clone());
        }
    }
    newly
}

const MILESTONE_WEIGHT: f64 = 8.0;
const RECENT_MILESTONE_BONUS: f64 = 5.0;
const RECENT_WINDOW_DAYS: i64 = 20;
const LIFETIME_WEIGHT: f64 = 0.6;
const RECENT_WEIGHT: f64 = 0.4;
const SECONDARY_THRESHOLD: f64 = 15.0;
const EMERGING_MIN_RECENT: f64 = 12.0;

const PROFILE_TAGS: [&str; 7] = [
    "Lawful House",
    "Shadow Operator",
    "Oceanic Carrier",
    "Contract Specialist",
    "Infrastructure Builder",
    "Leveraged Trader",
    "Risk-Managed Merchant",
];

#[derive(Debug, Clone, PartialEq)]
pub struct CareerProfileTag {
    pub tag: String,
    pub lifetime_score: f64,
    pub recent_score: f64,
    pub combined_score: f64,
    pub confidence: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CareerProfile {
    pub primary: Option<CareerProfileTag>,
    pub secondaries: Vec<CareerProfileTag>,
    pub emerging: Option<CareerProfileTag>,
    pub all_tags: Vec<CareerProfileTag>,
}

fn family_tags(family: &str) -> Vec<&'static str> {
    PROFILE_MILESTONE_FAMILIES
        .iter()
        .filter(|(_, families)| families.contains(&family))
        .map(|(tag, _)| *tag)
        .collect()
}

fn milestone_scores(world: &World, books: &HouseBooks) -> Vec<(f64, f64)> {
    let mut lifetime = [0.0; 7];
    let mut recent = [0.0; 7];
    for completion in &books.completed_milestones {
        let Some(spec) = content::content().milestone(&completion.milestone_id) else {
            continue;
        };
        for tag in family_tags(&spec.family) {
            let Some(index) = PROFILE_TAGS.iter().position(|name| *name == tag) else {
                continue;
            };
            lifetime[index] += MILESTONE_WEIGHT;
            if world.day - completion.completed_day <= RECENT_WINDOW_DAYS {
                recent[index] += RECENT_MILESTONE_BONUS;
            }
        }
    }
    lifetime.into_iter().zip(recent).collect()
}

fn base_scores(world: &World, books: &HouseBooks) -> Vec<(f64, Vec<String>)> {
    let tier = trust_tier(world.captain.standing.commercial_trust);
    let rank = trust_rank(tier);
    let heat = max_heat(world);
    let contracts = completed_contracts(books) as i64;
    let mut scores = Vec::with_capacity(7);

    let mut lawful = rank as f64 * 10.0;
    let mut lawful_ev = Vec::new();
    if rank >= 3 {
        lawful_ev.push(format!("trust: {tier}"));
    }
    if min_heat(world) <= 3 {
        lawful += 15.0;
        lawful_ev.push("low heat".to_string());
    }
    for license_id in [
        "med_trade_charter",
        "wa_commerce_permit",
        "ei_access_charter",
        "high_rep_charter",
    ] {
        if has_license(books, license_id) {
            lawful += 10.0;
            lawful_ev.push(license_id.to_string());
        }
    }
    lawful += (contracts * 3).min(30) as f64;
    if contracts >= 3 {
        lawful_ev.push(format!("{contracts} contracts completed"));
    }
    scores.push((lawful, lawful_ev));

    let mut shadow = 0.0;
    let mut shadow_ev = Vec::new();
    if heat >= 10 {
        shadow += (heat * 2).min(40) as f64;
        shadow_ev.push(format!("max heat: {heat}"));
    }
    if world.captain.captain_type == "smuggler" {
        shadow += 15.0;
        shadow_ev.push("smuggler captain".to_string());
    }
    let seizures = seizure_count(world) as i64;
    if seizures > 0 && world.captain.silver >= 200 {
        shadow += (seizures * 10) as f64;
        shadow_ev.push(format!("survived {seizures} seizures"));
    }
    if books.net_profit > 1500 && heat >= 10 {
        shadow += 20.0;
        shadow_ev.push("profitable under heat".to_string());
    }
    scores.push((shadow, shadow_ev));

    let mut oceanic = 0.0;
    let mut oceanic_ev = Vec::new();
    let ei_standing = world.captain.standing.regional_of("East Indies");
    if ei_standing >= 5 {
        oceanic += (ei_standing * 2) as f64;
        oceanic_ev.push(format!("East Indies standing: {ei_standing}"));
    }
    if has_license(books, "ei_access_charter") {
        oceanic += 20.0;
        oceanic_ev.push("EI access charter".to_string());
    }
    if broker_regions(books)
        .iter()
        .any(|region| region == "East Indies")
    {
        oceanic += 15.0;
        oceanic_ev.push("EI broker".to_string());
    }
    match ship_class(world).as_str() {
        "galleon" => {
            oceanic += 25.0;
            oceanic_ev.push("galleon operator".to_string());
        }
        "brigantine" => {
            oceanic += 10.0;
            oceanic_ev.push("brigantine capable".to_string());
        }
        _ => {}
    }
    scores.push((oceanic, oceanic_ev));

    let mut contract = (contracts * 5).min(50) as f64;
    let mut contract_ev = Vec::new();
    if contracts >= 3 {
        contract_ev.push(format!("{contracts} contracts delivered"));
    }
    let bonus_count = books
        .completed_contracts
        .iter()
        .filter(|contract| contract.outcome_type == "completed_bonus")
        .count() as i64;
    if bonus_count > 0 {
        contract += (bonus_count * 8) as f64;
        contract_ev.push(format!("{bonus_count} early bonuses"));
    }
    scores.push((contract, contract_ev));

    let warehouses = active_warehouse_count(books) as i64;
    let brokers = active_broker_count(books) as i64;
    let licenses = active_licenses(books) as i64;
    let mut infra = (warehouses * 10 + brokers * 12 + licenses * 15) as f64;
    let mut infra_ev = Vec::new();
    if warehouses >= 2 {
        infra_ev.push(format!("{warehouses} warehouses"));
    }
    if brokers >= 2 {
        infra_ev.push(format!("{brokers} broker offices"));
    }
    if licenses >= 2 {
        infra_ev.push(format!("{licenses} licenses"));
    }
    let regions = infra_regions(world, books);
    if regions.len() >= 2 {
        infra += 15.0;
        infra_ev.push(format!("presence in {} regions", regions.len()));
    }
    scores.push((infra, infra_ev));

    let mut leverage = 0.0;
    let mut leverage_ev = Vec::new();
    if let Some(credit) = &books.credit {
        if credit.total_borrowed > 0 {
            leverage += (credit.total_borrowed / 10).min(30) as f64;
            leverage_ev.push(format!("borrowed {}", credit.total_borrowed));
            if credit.defaults == 0 {
                leverage += 20.0;
                leverage_ev.push("no defaults".to_string());
            }
            if (credit.total_repaid as f64) > credit.total_borrowed as f64 * 0.5 {
                leverage += 15.0;
                leverage_ev.push("repaying responsibly".to_string());
            }
        }
    }
    scores.push((leverage, leverage_ev));

    let mut risk = 0.0;
    let mut risk_ev = Vec::new();
    if books.policies > 0 {
        risk += (books.policies * 8) as f64;
        risk_ev.push(format!("{} policies purchased", books.policies));
    }
    if books.claims_paid > 0 {
        risk += (books.claims_paid * 12) as f64;
        risk_ev.push(format!("{} claims paid", books.claims_paid));
    }
    if books.policies >= 3 && rank >= 2 {
        risk += 15.0;
        risk_ev.push("systematic insurance user".to_string());
    }
    scores.push((risk, risk_ev));
    scores
}

fn confidence_of(combined: f64) -> &'static str {
    for (level, threshold) in [("Defining", 60.0), ("Strong", 35.0), ("Moderate", 15.0)] {
        if combined >= threshold {
            return level;
        }
    }
    "Forming"
}

/// Weighted career profile. Milestone families come from
/// [`PROFILE_MILESTONE_FAMILIES`]; `commercial_finance` is not `commercial_empire`.
pub fn compute_career_profile(world: &World, books: &HouseBooks) -> CareerProfile {
    let bases = base_scores(world, books);
    let milestones = milestone_scores(world, books);
    let mut all_tags = Vec::with_capacity(PROFILE_TAGS.len());
    for (index, tag) in PROFILE_TAGS.iter().enumerate() {
        let (base, evidence) = &bases[index];
        let (ms_lifetime, ms_recent) = milestones[index];
        let lifetime = base + ms_lifetime;
        let recent = base * 0.5 + ms_recent;
        let combined = lifetime * LIFETIME_WEIGHT + recent * RECENT_WEIGHT;
        all_tags.push(CareerProfileTag {
            tag: (*tag).to_string(),
            lifetime_score: py_round_places(lifetime, 1),
            recent_score: py_round_places(recent, 1),
            combined_score: py_round_places(combined, 1),
            confidence: confidence_of(combined).to_string(),
            evidence: evidence.clone(),
        });
    }
    all_tags.sort_by(|left, right| right.combined_score.total_cmp(&left.combined_score));
    let primary_index = if all_tags.first().is_some_and(|tag| tag.combined_score > 0.0) {
        Some(0)
    } else {
        None
    };
    let mut secondary_indexes = Vec::new();
    for (index, tag) in all_tags.iter().enumerate().skip(1).take(2) {
        if tag.combined_score >= SECONDARY_THRESHOLD {
            secondary_indexes.push(index);
        }
    }
    let emerging_index = all_tags.iter().enumerate().find_map(|(index, tag)| {
        if Some(index) == primary_index || secondary_indexes.contains(&index) {
            return None;
        }
        if tag.recent_score >= EMERGING_MIN_RECENT {
            Some(index)
        } else {
            None
        }
    });
    CareerProfile {
        primary: primary_index.map(|index| all_tags[index].clone()),
        secondaries: secondary_indexes
            .into_iter()
            .map(|index| all_tags[index].clone())
            .collect(),
        emerging: emerging_index.map(|index| all_tags[index].clone()),
        all_tags,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Incident, World};
    use crate::world::new_game;
    use serde_json::Value;

    fn apply(input: &Value) -> (World, HouseBooks) {
        let mut world = new_game("Ada", "merchant", 1, None).expect("game");
        if let Some(silver) = input.get("silver").and_then(Value::as_i64) {
            world.captain.silver = silver;
        }
        if let Some(trust) = input.get("trust").and_then(Value::as_i64) {
            world.captain.standing.commercial_trust = trust;
        }
        if let Some(day) = input.get("day").and_then(Value::as_i64) {
            world.day = day;
        }
        if let Some(regional) = input.get("regional").and_then(Value::as_object) {
            for (region, value) in regional {
                world
                    .captain
                    .standing
                    .set_regional(region, value.as_i64().unwrap_or(0));
            }
        }
        if let Some(heat) = input.get("heat").and_then(Value::as_object) {
            for (region, value) in heat {
                world
                    .captain
                    .standing
                    .set_heat(region, value.as_i64().unwrap_or(0));
            }
        }
        if let Some(template) = input.get("ship_template").and_then(Value::as_str) {
            if let Some(ship) = world.captain.ship.as_mut() {
                ship.template_id = template.to_string();
            }
        }
        if let Some(incidents) = input.get("incidents").and_then(Value::as_array) {
            for description in incidents {
                world.captain.standing.incidents.insert(
                    0,
                    Incident {
                        day: 1,
                        port_id: "porto_novo".to_string(),
                        region: "Mediterranean".to_string(),
                        incident_type: "inspection".to_string(),
                        description: description.as_str().unwrap_or("").to_string(),
                        heat_delta: 0,
                        standing_delta: 0,
                        trust_delta: 0,
                    },
                );
            }
        }
        let mut books = HouseBooks {
            net_profit: input.get("net_profit").and_then(Value::as_i64).unwrap_or(0),
            trade_count: input
                .get("trade_count")
                .and_then(Value::as_i64)
                .unwrap_or(0),
            policies: input.get("policies").and_then(Value::as_i64).unwrap_or(0),
            ..HouseBooks::default()
        };
        if let Some(contracts) = input.get("contracts").and_then(Value::as_array) {
            for contract in contracts {
                books.completed_contracts.push(CompletedContract {
                    outcome_type: contract["outcome_type"].as_str().unwrap_or("").to_string(),
                    family: match contract.get("family") {
                        None | Some(Value::Null) => None,
                        Some(value) => value.as_str().map(str::to_string),
                    },
                    summary: contract
                        .get("summary")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string(),
                });
            }
        }
        if let Some(warehouses) = input.get("warehouses").and_then(Value::as_array) {
            for site in warehouses {
                let (port_id, active) = if let Some(port_id) = site.as_str() {
                    (port_id.to_string(), true)
                } else {
                    (
                        site["port_id"].as_str().unwrap_or("").to_string(),
                        site.get("active").and_then(Value::as_bool).unwrap_or(true),
                    )
                };
                books.warehouses.push(WarehouseSite { port_id, active });
            }
        }
        if let Some(brokers) = input.get("brokers").and_then(Value::as_array) {
            for broker in brokers {
                books.brokers.push(BrokerSite {
                    region: broker["region"].as_str().unwrap_or("").to_string(),
                    tier: broker["tier"].as_str().unwrap_or("none").to_string(),
                    active: broker
                        .get("active")
                        .and_then(Value::as_bool)
                        .unwrap_or(true),
                });
            }
        }
        if let Some(licenses) = input.get("licenses").and_then(Value::as_array) {
            for license in licenses {
                books.licenses.push(ActiveLicense {
                    license_id: license.as_str().unwrap_or("").to_string(),
                    active: true,
                });
            }
        }
        if let Some(credit) = input.get("credit").filter(|value| !value.is_null()) {
            books.credit = Some(CreditBook {
                total_borrowed: credit["total_borrowed"].as_i64().unwrap_or(0),
                defaults: credit["defaults"].as_i64().unwrap_or(0),
                active: credit
                    .get("active")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                total_repaid: credit
                    .get("total_repaid")
                    .and_then(Value::as_i64)
                    .unwrap_or(0),
            });
        }
        if let Some(records) = input.get("completed_paths").and_then(Value::as_array) {
            for record in records {
                books.completed_paths.push(VictoryRecord {
                    path_id: record["path_id"].as_str().unwrap_or("").to_string(),
                    completion_day: record["completion_day"].as_i64().unwrap_or(0),
                    summary: record["summary"].as_str().unwrap_or("").to_string(),
                    is_first: record
                        .get("is_first")
                        .and_then(Value::as_bool)
                        .unwrap_or(false),
                });
            }
        }
        (world, books)
    }

    fn close(left: &Value, right: &Value, path: &str) {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => {
                let af = a.as_f64().unwrap();
                let bf = b.as_f64().unwrap();
                let diff = (af - bf).abs();
                assert!(
                    diff <= 1e-6 || diff <= 1e-9 * af.abs().max(bf.abs()),
                    "{path}: {af} != {bf}"
                );
            }
            (Value::String(a), Value::String(b)) => assert_eq!(a, b, "{path}"),
            (Value::Bool(a), Value::Bool(b)) => assert_eq!(a, b, "{path}"),
            (Value::Null, Value::Null) => {}
            (Value::Array(a), Value::Array(b)) => {
                assert_eq!(a.len(), b.len(), "{path} length");
                for (index, (item, other)) in a.iter().zip(b).enumerate() {
                    close(item, other, &format!("{path}[{index}]"));
                }
            }
            (Value::Object(a), Value::Object(b)) => {
                assert_eq!(a.len(), b.len(), "{path} keys");
                for (key, value) in a {
                    close(
                        value,
                        b.get(key).unwrap_or(&Value::Null),
                        &format!("{path}.{key}"),
                    );
                }
            }
            _ => panic!("{path}: type mismatch {left} vs {right}"),
        }
    }

    #[test]
    fn commercial_ids_stay_distinct() {
        assert_ne!(PATH_COMMERCIAL_EMPIRE, MILESTONE_FAMILY_COMMERCIAL_FINANCE);
        assert_eq!(
            MilestoneFamily::CommercialFinance.as_str(),
            "commercial_finance"
        );
        let families: Vec<&str> = PROFILE_MILESTONE_FAMILIES
            .iter()
            .flat_map(|(_, families)| families.iter().copied())
            .collect();
        assert!(families.contains(&"commercial_finance"));
        assert!(!families.contains(&"commercial_empire"));
        let world = new_game("Ada", "merchant", 1, None).expect("game");
        let paths = compute_victory_progress(&world, &HouseBooks::default());
        let ids: Vec<_> = paths.iter().map(|path| path.path_id.as_str()).collect();
        assert!(ids.contains(&"commercial_empire"));
        assert!(!ids.iter().any(|id| *id == "commercial_finance"));
        assert_eq!(
            paths
                .iter()
                .find(|path| path.path_id == "lawful_house")
                .map(|path| path.candidate_strength),
            Some(26.7)
        );
    }

    #[test]
    fn victory_fixtures_match_python() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../parity/golden/victory_cases.json");
        let text = std::fs::read_to_string(&path).unwrap_or_else(|_| {
            panic!(
                "missing {}; run tools/parity/victory_cases.py",
                path.display()
            )
        });
        let cases: Vec<Value> = serde_json::from_str(&text).expect("victory cases");
        assert!(cases.len() >= 8, "fixture set is too small");
        for case in cases {
            let name = case["name"].as_str().unwrap_or("case");
            let (world, books) = apply(&case["input"]);
            let got = serde_json::to_value(compute_victory_progress(&world, &books)).expect("json");
            close(&case["paths"], &got, name);
        }
    }

    #[test]
    fn profile_families_keep_commercial_finance_off_the_path_id() {
        let catalog = crate::content::content();
        assert_eq!(
            catalog.campaign.profile_milestone_families.len(),
            PROFILE_MILESTONE_FAMILIES.len()
        );
        for (loaded, (tag, families)) in catalog
            .campaign
            .profile_milestone_families
            .iter()
            .zip(PROFILE_MILESTONE_FAMILIES.iter())
        {
            assert_eq!(loaded.tag, *tag);
            assert_eq!(loaded.families, *families);
        }
        let joined = PROFILE_MILESTONE_FAMILIES
            .iter()
            .flat_map(|(_, families)| families.iter().copied())
            .collect::<Vec<_>>();
        assert!(joined.contains(&"commercial_finance"));
        assert!(!joined.contains(&PATH_COMMERCIAL_EMPIRE));
    }

    #[test]
    fn merchant_advance_reaches_standing_and_credible_trust() {
        let mut world = new_game("Ada", "merchant", 1, None).unwrap();
        world.day = 2;
        let books = HouseBooks::default();
        let newly = evaluate_milestones(&world, &books);
        let ids: Vec<_> = newly
            .iter()
            .map(|item| item.milestone_id.as_str())
            .collect();
        assert_eq!(
            ids,
            vec!["foothold_standing_established", "lawful_credible_trust"]
        );
        assert_eq!(newly[0].evidence, "Standing 10+ in Mediterranean");
        assert_eq!(newly[0].completed_day, 2);
        assert_eq!(newly[1].evidence, "Trust tier: credible");
        let mut books = books;
        books.completed_milestones = newly;
        assert!(evaluate_milestones(&world, &books).is_empty());
        let profile = compute_career_profile(&world, &books);
        assert_eq!(profile.primary.as_ref().unwrap().tag, "Lawful House");
        assert_eq!(profile.primary.as_ref().unwrap().confidence, "Strong");
        assert_eq!(profile.primary.as_ref().unwrap().combined_score, 41.6);
        assert_eq!(profile.all_tags[1].tag, "Contract Specialist");
        assert_eq!(profile.all_tags[1].combined_score, 6.8);
        assert!(profile.secondaries.is_empty());
        assert!(profile.emerging.is_none());
    }

    #[test]
    fn commercial_finance_milestones_do_not_rename_the_empire_path() {
        let world = new_game("Ada", "merchant", 1, None).unwrap();
        let mut books = HouseBooks {
            credit: Some(CreditBook {
                total_borrowed: 300,
                defaults: 0,
                active: true,
                total_repaid: 0,
            }),
            ..HouseBooks::default()
        };
        let newly = evaluate_milestones(&world, &books);
        assert!(newly
            .iter()
            .any(|item| item.milestone_id == "finance_credit_opened"));
        assert!(!newly
            .iter()
            .any(|item| item.milestone_id == "commercial_empire"));
        books.completed_milestones = newly
            .into_iter()
            .filter(|item| item.milestone_id == "finance_credit_opened")
            .collect();
        let profile = compute_career_profile(&world, &books);
        let leveraged = profile
            .all_tags
            .iter()
            .find(|tag| tag.tag == "Leveraged Trader")
            .unwrap();
        assert!(leveraged.lifetime_score >= 8.0);
        let paths = compute_victory_progress(&world, &books);
        assert!(paths
            .iter()
            .any(|path| path.path_id == PATH_COMMERCIAL_EMPIRE));
        assert!(!paths
            .iter()
            .any(|path| path.path_id == "commercial_finance"));
    }

    #[test]
    fn a_cutter_reaches_the_ship_upgrade_milestone() {
        let world = new_game("Ada", "bounty_hunter", 1, None).unwrap();
        let newly = evaluate_milestones(&world, &HouseBooks::default());
        assert_eq!(newly[0].milestone_id, "integrated_brigantine");
        assert_eq!(newly[0].evidence, "Operating a cutter");
    }
}
