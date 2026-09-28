//! Python `stress/invariants.py` checked after seeded session runs.
//!
//! The scenario ids, captain types, seeds, and full day counts come from
//! `stress/scenarios.py`. Each day uses `Session` (trade, sail, advance,
//! and the public credit, warehouse, insurance, contract, and license
//! calls). The Python policy bot and its direct field injection are not
//! reproduced. `reporting.py` is not ported.
//!
//! | Python invariant (`invariants.py`) | Rust test |
//! | --- | --- |
//! | `no_negative_silver` (line 33) | `invariant_no_negative_silver` |
//! | `no_negative_cargo` (line 43) | `invariant_no_negative_cargo` |
//! | `cargo_within_capacity` (line 59) | `invariant_cargo_within_capacity` |
//! | `market_stock_valid` (line 79) | `invariant_market_stock_valid` |
//! | `no_dual_contract_resolution` (line 100) | `invariant_no_dual_contract_resolution` |
//! | `delivered_within_required` (line 113) | `invariant_delivered_within_required` |
//! | `inactive_warehouse_empty` (line 134) | `invariant_inactive_warehouse_empty` |
//! | `warehouse_within_capacity` (line 151) | `invariant_warehouse_within_capacity` |
//! | `no_overclaimed_policy` (line 175) | `invariant_no_overclaimed_policy` |
//! | `no_credit_overdraw` (line 225) | `invariant_no_credit_overdraw` |
//! | `frozen_credit_no_draw` (line 248) | `invariant_frozen_credit_no_draw` |
//! | `completed_milestones_no_dupes` (line 276) | `invariant_completed_milestones_no_dupes` |
//! | `completed_paths_no_dupes` (line 294) | `invariant_completed_paths_no_dupes` |
//! | `first_path_stays_first` (line 311) | `invariant_first_path_stays_first` |
//! | `crew_matches_roster` (line 332) | `invariant_crew_matches_roster` |
//! | `fleet_within_trust_limit` (line 355) | `invariant_fleet_within_trust_limit` |
//! | `upgrades_within_slots` (line 369) | `invariant_upgrades_within_slots` |
//! | `no_negative_crew_roles` (line 390) | `invariant_no_negative_crew_roles` |
//! | `flagship_exists` (line 407) | `invariant_flagship_exists` |
//!
//! `short_stress_scenarios_hold_invariants` runs every scenario for a few
//! days and asserts the whole list. `full_stress_sweep_holds_invariants`
//! is the Python `max_days` sweep and is `#[ignore]`.

use portlight_sim::content;
use portlight_sim::model::{Ship, VoyageStatus};
use portlight_sim::naval::max_fleet_size;
use portlight_sim::ship::resolve_cargo_capacity;
use portlight_sim::Session;

#[derive(Clone, Debug)]
struct Failure {
    name: &'static str,
    message: String,
}

struct Scenario {
    id: &'static str,
    captain: &'static str,
    seed: i128,
    max_days: i64,
    reload: bool,
}

const SCENARIOS: &[Scenario] = &[
    Scenario {
        id: "debt_spiral",
        captain: "merchant",
        seed: 42,
        max_days: 40,
        reload: false,
    },
    Scenario {
        id: "warehouse_neglect",
        captain: "merchant",
        seed: 137,
        max_days: 30,
        reload: false,
    },
    Scenario {
        id: "insured_luxury_loss",
        captain: "navigator",
        seed: 999,
        max_days: 30,
        reload: false,
    },
    Scenario {
        id: "contract_expiry_under_pressure",
        captain: "merchant",
        seed: 256,
        max_days: 25,
        reload: false,
    },
    Scenario {
        id: "heat_license_conflict",
        captain: "smuggler",
        seed: 666,
        max_days: 35,
        reload: false,
    },
    Scenario {
        id: "legitimization_pivot",
        captain: "smuggler",
        seed: 512,
        max_days: 50,
        reload: false,
    },
    Scenario {
        id: "oceanic_overextension",
        captain: "navigator",
        seed: 777,
        max_days: 40,
        reload: false,
    },
    Scenario {
        id: "victory_then_stress",
        captain: "merchant",
        seed: 1234,
        max_days: 60,
        reload: false,
    },
    Scenario {
        id: "save_load_mid_crisis",
        captain: "merchant",
        seed: 5678,
        max_days: 30,
        reload: true,
    },
];

fn check_all(session: &Session) -> Vec<Failure> {
    let mut failures = Vec::new();
    for hit in [
        check_no_negative_silver(session),
        check_no_negative_cargo(session),
        check_cargo_within_capacity(session),
        check_market_stock_valid(session),
        check_no_dual_contract_resolution(session),
        check_delivered_within_required(session),
        check_inactive_warehouse_empty(session),
        check_warehouse_within_capacity(session),
        check_no_overclaimed_policy(session),
        check_no_credit_overdraw(session),
        check_frozen_credit_no_draw(session),
        check_completed_milestones_no_dupes(session),
        check_completed_paths_no_dupes(session),
        check_first_path_stays_first(session),
        check_crew_matches_roster(session),
        check_fleet_within_trust_limit(session),
        check_upgrades_within_slots(session),
        check_no_negative_crew_roles(session),
        check_flagship_exists(session),
    ]
    .into_iter()
    .flatten()
    {
        failures.push(hit);
    }
    failures
}

fn check_no_negative_silver(session: &Session) -> Option<Failure> {
    let silver = session.world().captain.silver;
    if silver < 0 {
        Some(failure("no_negative_silver", format!("Silver is {silver}")))
    } else {
        None
    }
}

fn check_no_negative_cargo(session: &Session) -> Option<Failure> {
    for item in &session.world().captain.cargo {
        if item.quantity < 0 {
            return Some(failure(
                "no_negative_cargo",
                format!("{} has quantity {}", item.good_id, item.quantity),
            ));
        }
    }
    None
}

fn check_cargo_within_capacity(session: &Session) -> Option<Failure> {
    let ship = session.world().captain.ship.as_ref()?;
    let used: i64 = session
        .world()
        .captain
        .cargo
        .iter()
        .map(|item| item.quantity)
        .sum();
    let cap = resolve_cargo_capacity(ship);
    if used > cap {
        Some(failure(
            "cargo_within_capacity",
            format!("Cargo {used} exceeds capacity {cap}"),
        ))
    } else {
        None
    }
}

fn check_market_stock_valid(session: &Session) -> Option<Failure> {
    for port in &session.world().ports {
        for slot in &port.market {
            if slot.stock_current < 0 {
                return Some(failure(
                    "market_stock_valid",
                    format!("{}/{} stock={}", port.id, slot.good_id, slot.stock_current),
                ));
            }
        }
    }
    None
}

fn check_no_dual_contract_resolution(session: &Session) -> Option<Failure> {
    let board = session.board();
    let mut overlap = Vec::new();
    for contract in &board.active {
        if board
            .completed
            .iter()
            .any(|done| done.contract_id == contract.offer_id)
        {
            overlap.push(contract.offer_id.clone());
        }
    }
    if overlap.is_empty() {
        None
    } else {
        Some(failure(
            "no_dual_contract_resolution",
            format!("Overlapping: {overlap:?}"),
        ))
    }
}

fn check_delivered_within_required(session: &Session) -> Option<Failure> {
    for contract in &session.board().active {
        if contract.delivered_quantity > contract.required_quantity {
            return Some(failure(
                "delivered_within_required",
                format!(
                    "{}: delivered {} > required {}",
                    contract.offer_id, contract.delivered_quantity, contract.required_quantity
                ),
            ));
        }
    }
    None
}

fn check_inactive_warehouse_empty(session: &Session) -> Option<Failure> {
    for warehouse in &session.infrastructure().warehouses {
        if !warehouse.active && !warehouse.inventory.is_empty() {
            return Some(failure(
                "inactive_warehouse_empty",
                format!(
                    "Warehouse {} inactive but has {} lots",
                    warehouse.id,
                    warehouse.inventory.len()
                ),
            ));
        }
    }
    None
}

fn check_warehouse_within_capacity(session: &Session) -> Option<Failure> {
    for warehouse in &session.infrastructure().warehouses {
        if !warehouse.active {
            continue;
        }
        let used: i64 = warehouse.inventory.iter().map(|lot| lot.quantity).sum();
        if used > warehouse.capacity {
            return Some(failure(
                "warehouse_within_capacity",
                format!(
                    "Warehouse {}: used {used} > capacity {}",
                    warehouse.id, warehouse.capacity
                ),
            ));
        }
    }
    None
}

fn check_no_overclaimed_policy(session: &Session) -> Option<Failure> {
    let infra = session.infrastructure();
    for policy in &infra.policies {
        if policy.total_paid_out > policy.coverage_cap {
            return Some(failure(
                "no_overclaimed_policy",
                format!(
                    "Policy {} ({}): total paid {} > cap {}",
                    policy.id, policy.spec_id, policy.total_paid_out, policy.coverage_cap
                ),
            ));
        }
    }
    for claim in &infra.claims {
        let cap = if let Some(policy) = infra
            .policies
            .iter()
            .find(|policy| policy.id == claim.policy_id)
        {
            policy.coverage_cap
        } else if let Some(spec) = content::content().policy(&claim.policy_id) {
            spec.coverage_cap
        } else {
            continue;
        };
        if claim.payout > cap {
            return Some(failure(
                "no_overclaimed_policy",
                format!(
                    "Claim {}: paid {} > cap {cap}",
                    claim.policy_id, claim.payout
                ),
            ));
        }
    }
    None
}

fn check_no_credit_overdraw(session: &Session) -> Option<Failure> {
    let credit = session.infrastructure().credit.as_ref()?;
    if !credit.active {
        return None;
    }
    if credit.outstanding > credit.credit_limit {
        Some(failure(
            "no_credit_overdraw",
            format!(
                "Outstanding {} > limit {}",
                credit.outstanding, credit.credit_limit
            ),
        ))
    } else {
        None
    }
}

fn check_frozen_credit_no_draw(session: &Session) -> Option<Failure> {
    let credit = session.infrastructure().credit.as_ref()?;
    if credit.defaults >= 3 && credit.active {
        Some(failure(
            "frozen_credit_no_draw",
            format!("Credit active despite {} defaults", credit.defaults),
        ))
    } else {
        None
    }
}

fn check_completed_milestones_no_dupes(session: &Session) -> Option<Failure> {
    let ids: Vec<&str> = session
        .books()
        .completed_milestones
        .iter()
        .map(|row| row.milestone_id.as_str())
        .collect();
    if ids.len() == ids.iter().collect::<std::collections::BTreeSet<_>>().len() {
        return None;
    }
    let dupes: Vec<&str> = ids
        .iter()
        .copied()
        .filter(|id| ids.iter().filter(|other| *other == id).count() > 1)
        .collect();
    Some(failure(
        "completed_milestones_no_dupes",
        format!("Duplicate milestones: {dupes:?}"),
    ))
}

fn check_completed_paths_no_dupes(session: &Session) -> Option<Failure> {
    let ids: Vec<&str> = session
        .books()
        .completed_paths
        .iter()
        .map(|row| row.path_id.as_str())
        .collect();
    if ids.len() == ids.iter().collect::<std::collections::BTreeSet<_>>().len() {
        None
    } else {
        Some(failure(
            "completed_paths_no_dupes",
            format!("Duplicate paths: {ids:?}"),
        ))
    }
}

fn check_first_path_stays_first(session: &Session) -> Option<Failure> {
    let firsts = session
        .books()
        .completed_paths
        .iter()
        .filter(|path| path.is_first)
        .count();
    if firsts > 1 {
        Some(failure(
            "first_path_stays_first",
            format!("{firsts} paths marked as first"),
        ))
    } else {
        None
    }
}

fn check_crew_matches_roster(session: &Session) -> Option<Failure> {
    let world = session.world();
    if let Some(ship) = world.captain.ship.as_ref() {
        if let Some(message) = roster_mismatch(ship, true) {
            return Some(failure("crew_matches_roster", message));
        }
    }
    for owned in &world.captain.fleet {
        if let Some(message) = roster_mismatch(&owned.ship, false) {
            return Some(failure("crew_matches_roster", message));
        }
    }
    None
}

fn roster_mismatch(ship: &Ship, flagship: bool) -> Option<String> {
    let total = ship.roster_total();
    if total > 0 && ship.crew != total {
        if flagship {
            Some(format!(
                "Flagship crew={} != roster.total={total}",
                ship.crew
            ))
        } else {
            Some(format!(
                "Fleet ship {} crew={} != roster.total={total}",
                ship.name, ship.crew
            ))
        }
    } else {
        None
    }
}

fn check_fleet_within_trust_limit(session: &Session) -> Option<Failure> {
    let world = session.world();
    let trust = world.captain.standing.commercial_trust;
    let limit = max_fleet_size(trust);
    let total = 1 + world.captain.fleet.len() as i64;
    if total > limit {
        Some(failure(
            "fleet_within_trust_limit",
            format!("Fleet {total} > limit {limit}"),
        ))
    } else {
        None
    }
}

fn check_upgrades_within_slots(session: &Session) -> Option<Failure> {
    let world = session.world();
    if let Some(ship) = world.captain.ship.as_ref() {
        if ship.upgrades.len() as i64 > ship.upgrade_slots {
            return Some(failure(
                "upgrades_within_slots",
                format!(
                    "Flagship has {} upgrades in {} slots",
                    ship.upgrades.len(),
                    ship.upgrade_slots
                ),
            ));
        }
    }
    for owned in &world.captain.fleet {
        if owned.ship.upgrades.len() as i64 > owned.ship.upgrade_slots {
            return Some(failure(
                "upgrades_within_slots",
                format!("Fleet ship {} exceeds upgrade slots", owned.ship.name),
            ));
        }
    }
    None
}

fn check_no_negative_crew_roles(session: &Session) -> Option<Failure> {
    let world = session.world();
    let mut ships = Vec::new();
    if let Some(ship) = world.captain.ship.as_ref() {
        ships.push(ship);
    }
    for owned in &world.captain.fleet {
        ships.push(&owned.ship);
    }
    for ship in ships {
        for (field, value) in [
            ("sailors", ship.sailors),
            ("gunners", ship.gunners),
            ("navigators", ship.navigators),
            ("surgeons", ship.surgeons),
            ("marines", ship.marines),
            ("quartermasters", ship.quartermasters),
        ] {
            if value < 0 {
                return Some(failure(
                    "no_negative_crew_roles",
                    format!("{} has {field}={value}", ship.name),
                ));
            }
        }
    }
    None
}

fn check_flagship_exists(session: &Session) -> Option<Failure> {
    if session.world().captain.ship.is_none() {
        Some(failure("flagship_exists", "No flagship"))
    } else {
        None
    }
}

fn failure(name: &'static str, message: impl Into<String>) -> Failure {
    Failure {
        name,
        message: message.into(),
    }
}

fn assert_holds(session: &Session, name: &str) {
    let hits: Vec<_> = check_all(session)
        .into_iter()
        .filter(|hit| hit.name == name)
        .collect();
    assert!(
        hits.is_empty(),
        "{name} failed: {}",
        hits.iter()
            .map(|hit| hit.message.as_str())
            .collect::<Vec<_>>()
            .join("; ")
    );
}

fn probe_session() -> Session {
    let mut session = Session::new("Ada", "merchant", 7, None).expect("game");
    let _ = session.buy("grain", 4);
    let _ = session.depart("silva_bay");
    let _ = session.advance();
    if session.world().pending_duel.is_some() {
        let _ = session.resolve_pending_duel();
    }
    let _ = session.advance();
    session
}

fn prepare(session: &mut Session, id: &str) {
    if matches!(
        id,
        "debt_spiral" | "oceanic_overextension" | "save_load_mid_crisis"
    ) {
        if let Some(limit) = content::content()
            .credit_tier("merchant_line")
            .map(|tier| tier.credit_limit)
        {
            let _ = session.take_credit("merchant_line", limit);
        }
    }
    if matches!(id, "warehouse_neglect" | "save_load_mid_crisis") {
        let _ = session.buy("grain", 4);
        let _ = session.buy_infrastructure("warehouse", &["depot"]);
        let _ = session.deposit_cargo("grain", 2);
    }
    if matches!(id, "insured_luxury_loss" | "save_load_mid_crisis") {
        let _ = session.buy_insurance("cargo_premium", "", "", "");
        let _ = session.buy("silk", 1);
    }
    if matches!(
        id,
        "contract_expiry_under_pressure" | "save_load_mid_crisis"
    ) {
        let offer_id = session
            .available_contracts()
            .first()
            .map(|offer| offer.id.clone());
        if let Some(offer_id) = offer_id {
            let _ = session.accept_contract(&offer_id);
        }
    }
    if id == "heat_license_conflict" {
        let _ = session.buy_infrastructure("license", &["med_trade_charter"]);
    }
}

fn act(session: &mut Session) {
    if session.world().voyage.status != VoyageStatus::InPort {
        return;
    }
    if session.world().captain.provisions < 8 {
        let _ = session.provision(8);
    }
    if let Some(good) = session
        .world()
        .captain
        .cargo
        .first()
        .map(|item| item.good_id.clone())
    {
        let _ = session.sell(&good, 1);
    }
    let silver = session.world().captain.silver;
    let port_id = session.world().voyage.destination_id.clone();
    let buy = session.world().port(&port_id).and_then(|port| {
        port.market
            .iter()
            .find(|slot| {
                slot.stock_current > 0 && slot.buy_price > 0 && slot.buy_price * 2 <= silver
            })
            .map(|slot| slot.good_id.clone())
    });
    if let Some(good) = buy {
        let _ = session.buy(&good, 2);
    }
    if session.world().day % 2 == 0 {
        let dest = next_port(session);
        if let Some(dest) = dest {
            let _ = session.depart(&dest);
        }
    }
}

fn next_port(session: &Session) -> Option<String> {
    let world = session.world();
    let here = world.voyage.destination_id.as_str();
    let mut neighbors = Vec::new();
    for route in &world.routes {
        if route.port_a == here {
            neighbors.push(route.port_b.clone());
        } else if route.port_b == here {
            neighbors.push(route.port_a.clone());
        }
    }
    if neighbors.is_empty() {
        return None;
    }
    let index = (world.day as usize).wrapping_add(world.seed as usize) % neighbors.len();
    Some(neighbors[index].clone())
}

fn drive(scenario: &Scenario, days: i64) {
    let mut session =
        Session::new("StressBot", scenario.captain, scenario.seed, None).expect(scenario.id);
    prepare(&mut session, scenario.id);
    let mut reloaded = false;
    let mut guard = 0;
    let max_guard = days.max(1) * 3;
    while session.world().day < days && guard < max_guard {
        guard += 1;
        let before = session.world().day;
        assert_clean(&session, scenario, before);
        act(&mut session);
        if session.world().pending_duel.is_some() {
            let _ = session.resolve_pending_duel();
        }
        let _ = session.advance();
        if session.world().pending_duel.is_some() {
            let _ = session.resolve_pending_duel();
        }
        if scenario.reload && !reloaded && session.world().day > before {
            reloaded = true;
            session = reload(session, scenario);
        }
        assert_clean(&session, scenario, session.world().day);
        let captain = &session.world().captain;
        if captain.silver <= 0 && captain.provisions <= 0 {
            break;
        }
    }
}

fn reload(session: Session, scenario: &Scenario) -> Session {
    let mut session = session;
    let dir = std::env::temp_dir().join(format!(
        "portlight-inv-{}-{}",
        scenario.id,
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("temp dir");
    session.save(&dir, "stress").expect("save");
    let loaded = Session::load(&dir, "stress").expect("load").expect("slot");
    let _ = std::fs::remove_dir_all(&dir);
    loaded
}

fn assert_clean(session: &Session, scenario: &Scenario, day: i64) {
    let failures = check_all(session);
    assert!(
        failures.is_empty(),
        "{} day {day} seed {}: {}",
        scenario.id,
        scenario.seed,
        failures
            .iter()
            .map(|hit| format!("{} ({})", hit.name, hit.message))
            .collect::<Vec<_>>()
            .join("; ")
    );
}

fn run_scenarios(days_of: impl Fn(&Scenario) -> i64) {
    for scenario in SCENARIOS {
        drive(scenario, days_of(scenario));
    }
}

macro_rules! invariant_test {
    ($name:ident, $invariant:literal) => {
        #[test]
        fn $name() {
            assert_holds(&probe_session(), $invariant);
        }
    };
}

invariant_test!(invariant_no_negative_silver, "no_negative_silver");
invariant_test!(invariant_no_negative_cargo, "no_negative_cargo");
invariant_test!(invariant_cargo_within_capacity, "cargo_within_capacity");
invariant_test!(invariant_market_stock_valid, "market_stock_valid");
invariant_test!(
    invariant_no_dual_contract_resolution,
    "no_dual_contract_resolution"
);
invariant_test!(
    invariant_delivered_within_required,
    "delivered_within_required"
);
invariant_test!(
    invariant_inactive_warehouse_empty,
    "inactive_warehouse_empty"
);
invariant_test!(
    invariant_warehouse_within_capacity,
    "warehouse_within_capacity"
);
invariant_test!(invariant_no_overclaimed_policy, "no_overclaimed_policy");
invariant_test!(invariant_no_credit_overdraw, "no_credit_overdraw");
invariant_test!(invariant_frozen_credit_no_draw, "frozen_credit_no_draw");
invariant_test!(
    invariant_completed_milestones_no_dupes,
    "completed_milestones_no_dupes"
);
invariant_test!(
    invariant_completed_paths_no_dupes,
    "completed_paths_no_dupes"
);
invariant_test!(invariant_first_path_stays_first, "first_path_stays_first");
invariant_test!(invariant_crew_matches_roster, "crew_matches_roster");
invariant_test!(
    invariant_fleet_within_trust_limit,
    "fleet_within_trust_limit"
);
invariant_test!(invariant_upgrades_within_slots, "upgrades_within_slots");
invariant_test!(invariant_no_negative_crew_roles, "no_negative_crew_roles");
invariant_test!(invariant_flagship_exists, "flagship_exists");

#[test]
fn short_stress_scenarios_hold_invariants() {
    run_scenarios(|_| 4);
}

#[test]
#[ignore]
fn full_stress_sweep_holds_invariants() {
    run_scenarios(|scenario| scenario.max_days);
}
