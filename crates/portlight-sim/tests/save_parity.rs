//! Save/load checked against the Python loader's snapshot.

use std::fs;
use std::path::PathBuf;
use std::process;
use std::sync::Mutex;

use portlight_sim::campaign::{CompletedContract, VictoryRecord};
use portlight_sim::model::{
    ActivePolicy, BrokerOffice, CreditState, InfrastructureRecord, OwnedLicense, VoyageStatus,
    WarehouseLease,
};
use portlight_sim::snapshot;
use portlight_sim::Session;
use serde_json::Value;

fn content_route(a: &str, b: &str) -> Option<&'static portlight_sim::content::RouteDef> {
    portlight_sim::content::content()
        .routes
        .iter()
        .find(|def| (def.port_a == a && def.port_b == b) || (def.port_a == b && def.port_b == a))
}

/// Every loaded route carries the same lane name and lore as the session
/// that was saved.
fn assert_lane_names_match(saved: &Session, loaded: &Session) {
    assert_eq!(saved.world().routes.len(), loaded.world().routes.len());
    for (before, after) in saved.world().routes.iter().zip(&loaded.world().routes) {
        assert_eq!(before.port_a, after.port_a);
        assert_eq!(before.port_b, after.port_b);
        assert_eq!(
            before.lore_name, after.lore_name,
            "{}-{}",
            before.port_a, before.port_b
        );
        assert_eq!(
            before.lore, after.lore,
            "{}-{}",
            before.port_a, before.port_b
        );
    }
}

fn parity_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../parity")
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("portlight-{name}-{}", process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn close(left: &Value, right: &Value, path: &str) {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => {
            let af = a.as_f64().expect("left number");
            let bf = b.as_f64().expect("right number");
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
            for (i, (x, y)) in a.iter().zip(b).enumerate() {
                close(x, y, &format!("{path}[{i}]"));
            }
        }
        (Value::Object(a), Value::Object(b)) => {
            assert_eq!(
                a.len(),
                b.len(),
                "{path} keys {:?} vs {:?}",
                a.keys().collect::<Vec<_>>(),
                b.keys().collect::<Vec<_>>()
            );
            for (key, value) in a {
                let Some(other) = b.get(key) else {
                    panic!("{path} missing {key}");
                };
                close(value, other, &format!("{path}.{key}"));
            }
        }
        _ => panic!("{path}: type mismatch {left} vs {right}"),
    }
}

static SNAP_ENV: Mutex<()> = Mutex::new(());

fn snap(session: &Session) -> Value {
    let _guard = SNAP_ENV.lock().unwrap_or_else(|err| err.into_inner());
    // Golden JSON is the narrow shape. A developer shell may have exported this.
    std::env::remove_var("PORTLIGHT_WIDE_SNAPSHOT");
    serde_json::to_value(snapshot::capture(
        session.world(),
        session.trade_seq(),
        session.books(),
        session.infrastructure(),
        session.narrative(),
        session.board(),
        session.receipts(),
        session.run_id(),
        Vec::new(),
    ))
    .unwrap()
}

fn played() -> Session {
    let mut session = Session::new("Ada", "merchant", 42, None).unwrap();
    session.buy("grain", 8).unwrap();
    session.sell("grain", 3).unwrap();
    session.hire_crew(1, "navigator").unwrap();
    session.provision(2).unwrap();
    session.depart("silva_bay").unwrap();
    for _ in 0..12 {
        if session.world().pending_duel.is_some()
            || session.world().voyage.status != VoyageStatus::AtSea
        {
            break;
        }
        session.advance().unwrap();
    }
    {
        let books = session.books_mut();
        books.completed_contracts.push(CompletedContract {
            outcome_type: "completed".to_string(),
            family: Some("luxury_discreet".to_string()),
            summary: "Silk moved quietly".to_string(),
        });
        books.completed_paths.push(VictoryRecord {
            path_id: "lawful_house".to_string(),
            completion_day: 4,
            summary: "Recorded early".to_string(),
            is_first: true,
        });
    }
    session.adopt_infrastructure(InfrastructureRecord {
        warehouses: vec![WarehouseLease {
            id: "b21e3e594ab0".to_string(),
            port_id: "porto_novo".to_string(),
            tier: "depot".to_string(),
            capacity: 20,
            lease_cost: 50,
            upkeep_per_day: 1,
            inventory: Vec::new(),
            opened_day: 1,
            upkeep_paid_through: 1,
            active: true,
        }],
        brokers: vec![BrokerOffice {
            region: "East Indies".to_string(),
            tier: "local".to_string(),
            opened_day: 1,
            upkeep_paid_through: 1,
            active: true,
        }],
        licenses: vec![OwnedLicense {
            license_id: "ei_access_charter".to_string(),
            purchased_day: 1,
            upkeep_paid_through: 1,
            active: true,
        }],
        policies: vec![ActivePolicy {
            id: "ec4d4839a293".to_string(),
            spec_id: "hull_basic".to_string(),
            family: "hull".to_string(),
            scope: "next_voyage".to_string(),
            purchased_day: 1,
            coverage_pct: 0.5,
            coverage_cap: 150,
            premium_paid: 40,
            target_id: String::new(),
            claims_made: 0,
            total_paid_out: 0,
            active: true,
            voyage_origin: String::new(),
            voyage_destination: String::new(),
        }],
        claims: Vec::new(),
        credit: Some(CreditState {
            tier: "house_credit".to_string(),
            credit_limit: 800,
            outstanding: 500,
            interest_accrued: 0,
            last_interest_day: 1,
            next_due_day: 11,
            defaults: 0,
            total_borrowed: 500,
            total_repaid: 0,
            active: true,
        }),
        bank: Vec::new(),
    });
    session
}

#[test]
fn v1_slot_matches_the_python_loader() {
    let root = parity_root();
    let loaded = Session::load(&root, "v1").unwrap().unwrap();
    let ship = loaded.world().captain.ship.as_ref().unwrap();
    assert_eq!(ship.morale, 50);
    assert_eq!(ship.sailors, 4);
    assert_eq!(ship.cannons, 0);
    assert!((ship.maneuver - 0.9).abs() < 1e-9);
    assert_eq!(ship.upgrade_slots, 2);
    assert_eq!(ship.officers[0].role, "sailor");
    assert_eq!(ship.officers[0].name, "Ned");
    assert_eq!(
        loaded.world().ports[0].features,
        vec!["shipyard".to_string()]
    );
    // Saves keep no lane names; the loader refills them from content by
    // port pair (the Python loader leaves them empty; see the parity note).
    let route = &loaded.world().routes[0];
    let def = content_route(&route.port_a, &route.port_b);
    assert_eq!(
        route.lore_name,
        def.map(|d| d.lore_name.clone()).unwrap_or_default()
    );
    assert_eq!(route.lore, def.map(|d| d.lore.clone()).unwrap_or_default());
    assert_eq!(loaded.world().captain.deferred_fees[0].fee_type, "port_fee");
    assert_eq!(loaded.world().captain.deferred_fees[0].amount, 12);
    assert_eq!(
        loaded.world().captain.active_bounties,
        vec!["red_hand".to_string()]
    );
    assert_eq!(loaded.trade_seq(), 0);
    let golden: Value =
        serde_json::from_str(&fs::read_to_string(root.join("golden/save_v1.json")).unwrap())
            .unwrap();
    close(&golden, &snap(&loaded), "save_v1");
}

#[test]
fn played_slot_matches_the_python_loader() {
    let mut session = played();
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/save-oracle");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    session.save(&dir, "played").unwrap();
    let loaded = Session::load(&dir, "played").unwrap().unwrap();

    assert_eq!(loaded.trade_seq(), session.trade_seq());
    assert_eq!(loaded.books().total_buys, session.books().total_buys);
    assert_eq!(loaded.books().total_sells, session.books().total_sells);
    assert_eq!(loaded.books().net_profit, session.books().net_profit);
    assert_eq!(loaded.books().trade_count, session.books().trade_count);
    assert_eq!(loaded.books().policies, 1);
    assert_eq!(loaded.books().completed_contracts.len(), 1);
    assert_eq!(
        loaded.books().completed_contracts[0].family.as_deref(),
        Some("luxury_discreet")
    );
    assert_eq!(loaded.books().warehouses[0].port_id, "porto_novo");
    assert_eq!(loaded.books().brokers[0].region, "East Indies");
    assert_eq!(loaded.books().brokers[0].tier, "local");
    assert_eq!(loaded.books().licenses[0].license_id, "ei_access_charter");
    assert_eq!(loaded.books().credit.as_ref().unwrap().total_borrowed, 500);
    assert_eq!(loaded.books().completed_paths[0].summary, "Recorded early");
    assert_eq!(
        loaded.world().pending_duel.is_some(),
        session.world().pending_duel.is_some()
    );
    assert_lane_names_match(&session, &loaded);

    let golden_path = parity_root().join("golden/save_roundtrip.json");
    let golden_text = fs::read_to_string(&golden_path).unwrap_or_else(|_| {
        panic!(
            "missing {} — run tools/parity/save_slot.py on {}",
            golden_path.display(),
            dir.display()
        )
    });
    let golden: Value = serde_json::from_str(&golden_text).unwrap();
    close(&golden, &snap(&loaded), "save_roundtrip");
}

#[test]
fn python_v12_career_keys_round_trip() {
    let root = parity_root();
    let python: Value =
        serde_json::from_str(&fs::read_to_string(root.join("saves/career_v12.json")).unwrap())
            .unwrap();
    let loaded = Session::load(&root, "career_v12").unwrap().unwrap();
    assert_eq!(
        loaded.world().captain.learned_styles,
        vec!["la_destreza".to_string()]
    );
    assert_eq!(loaded.world().captain.skills.len(), 1);
    assert_eq!(loaded.world().captain.skills[0].id, "blacksmith");
    assert_eq!(loaded.world().captain.skills[0].level, 1);
    assert_eq!(loaded.world().captain.party.max_size, 2);
    assert!(loaded.world().captain.party.departed.is_empty());
    assert_eq!(
        loaded.world().captain.party.companions[0].companion_id,
        "red_tomas"
    );
    assert_eq!(loaded.world().captain.party.companions[0].role_id, "marine");
    assert_eq!(loaded.world().captain.party.companions[0].morale, 70);
    assert_eq!(loaded.world().captain.party.companions[0].joined_day, 1);
    assert_eq!(
        loaded.world().captain.party.companions[0].personality,
        "pragmatic"
    );
    assert_eq!(
        loaded
            .books()
            .completed_milestones
            .iter()
            .map(|item| (
                item.milestone_id.as_str(),
                item.completed_day,
                item.evidence.as_str()
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                "foothold_standing_established",
                2,
                "Standing 10+ in Mediterranean",
            ),
            ("lawful_credible_trust", 2, "Trust tier: credible"),
        ]
    );
    assert_eq!(loaded.world().captain_memories.len(), 1);
    let memory = &loaded.world().captain_memories[0];
    assert_eq!(memory.captain_id, "the_butcher");
    assert_eq!(memory.relationship.respect, 5);
    assert_eq!(memory.relationship.fear, 20);
    assert_eq!(memory.relationship.grudge, 15);
    assert_eq!(memory.relationship.familiarity, 10);
    assert_eq!(memory.last_seen_day, 1);
    assert_eq!(memory.last_seen_region, "Mediterranean");
    assert_eq!(memory.times_defeated_by_player, 1);
    assert!(memory.player_sank_their_ship);
    assert_eq!(memory.encounters[0].outcome, "ship_sunk");
    assert_eq!(memory.encounters[0].respect_delta, 5);
    assert_eq!(memory.encounters[0].fear_delta, 20);

    let dir = scratch("career-v12");
    let mut loaded = loaded;
    loaded.save(&dir, "career_v12").unwrap();
    let rust: Value = serde_json::from_str(
        &fs::read_to_string(dir.join("saves").join("career_v12.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        rust["captain"]["learned_styles"],
        python["captain"]["learned_styles"]
    );
    assert_eq!(rust["captain"]["skills"], python["captain"]["skills"]);
    assert_eq!(rust["captain"]["party"], python["captain"]["party"]);
    assert_eq!(
        rust["campaign"]["completed"],
        python["campaign"]["completed"]
    );
    assert_eq!(
        rust["pirate_state"]["captain_memories"],
        python["pirate_state"]["captain_memories"]
    );
    let again = Session::load(&dir, "career_v12").unwrap().unwrap();
    assert_eq!(
        again.world().captain.learned_styles,
        loaded.world().captain.learned_styles
    );
    assert_eq!(again.world().captain.skills, loaded.world().captain.skills);
    assert_eq!(again.world().captain.party, loaded.world().captain.party);
    assert_eq!(
        again.books().completed_milestones,
        loaded.books().completed_milestones
    );
    assert_eq!(
        again.world().captain_memories,
        loaded.world().captain_memories
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn python_v12_save_restores_the_contract_board() {
    let root = parity_root();
    let loaded = Session::load(&root, "contract_v12").unwrap().unwrap();
    let board = loaded.board();
    assert_eq!(board.active.len(), 1);
    assert_eq!(board.active[0].offer_id, "63fc3f8be22a");
    assert_eq!(board.active[0].family, "smuggling");
    assert_eq!(board.active[0].good_id, "weapons");
    assert_eq!(board.active[0].required_quantity, 7);
    assert_eq!(board.active[0].status, "accepted");
    assert_eq!(board.active[0].destination_port_id, "corsairs_rest");
    assert_eq!(
        board
            .offers
            .iter()
            .map(|offer| offer.id.as_str())
            .collect::<Vec<_>>(),
        vec![
            "d2c430fdaf99",
            "71773aae754b",
            "dd38287c23aa",
            "0b22b57f35b1",
        ]
    );
    assert_eq!(board.last_refresh_day, 1);
    assert_eq!(board.max_offers, 5);
    assert_eq!(board.breaches.len(), 1);
    assert_eq!(board.breaches[0].contract_id, "63fc3f8be22a");
    assert_eq!(board.breaches[0].day, 2);
    assert_eq!(board.breaches[0].port_id, "porto_novo");
    assert_eq!(board.breaches[0].family, "smuggling");
    assert_eq!(loaded.world().captain.wanted_level, 1);
    assert_eq!(loaded.world().day, 1);
    assert_eq!(loaded.world().seed, 1);
}

#[test]
fn python_v12_area5_round_trips() {
    let root = parity_root();
    let fixture: Value =
        serde_json::from_str(&fs::read_to_string(root.join("saves/area5_v12.json")).unwrap())
            .unwrap();
    let loaded = Session::load(&root, "area5_v12").unwrap().unwrap();
    let captain = &loaded.world().captain;
    assert_eq!(captain.injuries.len(), 2);
    assert_eq!(captain.injuries[0].injury_id, "cut_hand");
    assert_eq!(captain.injuries[0].heal_remaining, Some(10));
    assert_eq!(captain.injuries[1].injury_id, "blinded_eye");
    assert_eq!(captain.injuries[1].heal_remaining, None);
    assert_eq!(captain.armor.as_ref().unwrap().id, "chain_shirt");
    assert_eq!(captain.armor.as_ref().unwrap().damage_reduction, 1);
    assert_eq!(captain.armor.as_ref().unwrap().dodge_penalty, 1);
    assert_eq!(captain.melee.as_ref().unwrap().id, "cutlass");
    assert_eq!(captain.quality_of("cutlass"), "fine");
    assert_eq!(captain.usage_of("cutlass"), 9);
    let provenance = captain
        .weapon_provenance
        .iter()
        .find(|(id, _)| id == "cutlass")
        .map(|(_, prov)| prov)
        .unwrap();
    assert_eq!(provenance.epithet.as_deref(), Some("Raj the Quiet's Bane"));
    assert_eq!(provenance.kills, 1);
    assert_eq!(captain.fleet.len(), 1);
    assert_eq!(captain.fleet[0].ship.template_id, "coastal_sloop");
    assert_eq!(captain.fleet[0].ship.hull, 40);
    assert_eq!(captain.fleet[0].docked_port_id, "porto_novo");
    assert_eq!(captain.fleet[0].cargo[0].good_id, "grain");
    assert_eq!(captain.fleet[0].cargo[0].quantity, 3);
    assert_eq!(captain.fleet[0].cargo[0].acquired_port, "loot");
    assert_eq!(
        captain.ship.as_ref().unwrap().upgrades[0].upgrade_id,
        "extra_gun_ports"
    );
    assert!(loaded.pending_victory());

    let dir = scratch("area5");
    let mut loaded = loaded;
    loaded.save(&dir, "area5_v12").unwrap();
    let again = Session::load(&dir, "area5_v12").unwrap().unwrap();
    assert!(again.pending_victory());
    assert_eq!(again.world().captain.injuries.len(), 2);
    let rust: Value = serde_json::from_str(
        &fs::read_to_string(dir.join("saves").join("area5_v12.json")).unwrap(),
    )
    .unwrap();
    for path in [
        "captain.injuries",
        "captain.combat_gear",
        "captain.fleet",
        "captain.ship.upgrades",
    ] {
        close(&dig(&fixture, path), &dig(&rust, path), path);
    }
    close(
        &fixture["pirate_state"]["encounter_state"]["pending_victory"],
        &rust["pirate_state"]["encounter_state"]["pending_victory"],
        "pending_victory",
    );
    let _ = fs::remove_dir_all(&dir);
}

fn dig(value: &Value, path: &str) -> Value {
    let mut cursor = value;
    for key in path.split('.') {
        cursor = &cursor[key];
    }
    cursor.clone()
}

#[test]
fn load_reseeds_with_seed_plus_day() {
    let mut session = Session::new("Ada", "merchant", 42, None).unwrap();
    let dir = scratch("reseed");
    session.save(&dir, "default").unwrap();
    let mut loaded = Session::load(&dir, "default").unwrap().unwrap();
    // random.Random(42 + 1).randint(3, 5) == 3
    assert_eq!(loaded.work().unwrap(), 3);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn missing_and_corrupt_slots_are_absent() {
    let dir = scratch("absent");
    assert!(Session::load(&dir, "default").unwrap().is_none());
    let path = dir.join("saves");
    fs::create_dir_all(&path).unwrap();
    fs::write(path.join("default.json"), b"{not json").unwrap();
    assert!(Session::load(&dir, "default").unwrap().is_none());
    fs::write(path.join("default.json"), b"{\"version\": 14}").unwrap();
    let err = Session::load(&dir, "default").unwrap_err();
    assert_eq!(
        err.to_string(),
        "Save file version 14 is newer than supported version 13. Update Portlight to load this save."
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn legacy_filename_moves_to_the_default_slot() {
    let dir = scratch("legacy");
    let saves = dir.join("saves");
    fs::create_dir_all(&saves).unwrap();
    let mut session = Session::new("Ada", "merchant", 7, None).unwrap();
    session.save(&dir, "default").unwrap();
    let text = fs::read_to_string(saves.join("default.json")).unwrap();
    fs::remove_file(saves.join("default.json")).unwrap();
    fs::write(saves.join("portlight_save.json"), text).unwrap();
    let loaded = Session::load(&dir, "default").unwrap().unwrap();
    assert_eq!(loaded.world().seed, 7);
    assert!(saves.join("default.json").exists());
    assert!(!saves.join("portlight_save.json").exists());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn python_v12_infrastructure_round_trips() {
    let root = parity_root();
    let python: Value = serde_json::from_str(
        &fs::read_to_string(root.join("saves/infrastructure_v12.json")).unwrap(),
    )
    .unwrap();
    let loaded = Session::load(&root, "infrastructure_v12").unwrap().unwrap();
    let infra = loaded.infrastructure();
    assert_eq!(infra.warehouses.len(), 1);
    assert_eq!(infra.warehouses[0].id, "b21e3e594ab0");
    assert_eq!(infra.warehouses[0].tier, "depot");
    assert_eq!(infra.warehouses[0].port_id, "porto_novo");
    assert_eq!(infra.warehouses[0].capacity, 20);
    assert_eq!(infra.warehouses[0].upkeep_paid_through, 1);
    assert_eq!(infra.brokers[0].region, "Mediterranean");
    assert_eq!(infra.brokers[0].tier, "local");
    assert_eq!(infra.policies[0].id, "ec4d4839a293");
    assert_eq!(infra.policies[0].spec_id, "hull_basic");
    assert!((infra.policies[0].coverage_pct - 0.5).abs() < 1e-9);
    assert_eq!(infra.claims.len(), 1);
    assert!(!infra.claims[0].denied);
    assert_eq!(infra.claims[0].payout, 40);
    let credit = infra.credit.as_ref().unwrap();
    assert_eq!(credit.tier, "merchant_line");
    assert_eq!(credit.credit_limit, 300);
    assert_eq!(credit.outstanding, 100);
    assert_eq!(credit.total_borrowed, 100);
    assert!(credit.active);
    assert_eq!(loaded.books().claims_paid, 1);
    assert_eq!(loaded.books().policies, 1);
    assert_eq!(loaded.books().credit.as_ref().unwrap().total_borrowed, 100);
    assert!(loaded.books().credit.as_ref().unwrap().active);

    let dir = scratch("infra-v12");
    let mut loaded = loaded;
    loaded.save(&dir, "infrastructure_v12").unwrap();
    let rust: Value = serde_json::from_str(
        &fs::read_to_string(dir.join("saves").join("infrastructure_v12.json")).unwrap(),
    )
    .unwrap();
    close(
        &python["infrastructure"],
        &rust["infrastructure"],
        "infrastructure",
    );
    let again = Session::load(&dir, "infrastructure_v12").unwrap().unwrap();
    assert_eq!(again.infrastructure().warehouses[0].id, "b21e3e594ab0");
    assert_eq!(again.books().claims_paid, 1);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn python_v12_area6_round_trips() {
    let root = parity_root();
    let fixture: Value =
        serde_json::from_str(&fs::read_to_string(root.join("saves/area6_v12.json")).unwrap())
            .unwrap();
    let loaded = Session::load(&root, "area6_v12").unwrap().unwrap();
    assert_eq!(loaded.world().captain.reputation, 3);
    assert_eq!(loaded.world().culture.visits("porto_novo"), 2);
    assert_eq!(loaded.world().culture.visits("silva_bay"), 1);
    assert_eq!(
        loaded.world().culture.regions_entered,
        ["Mediterranean".to_string(), "North Atlantic".to_string()]
    );
    assert_eq!(loaded.world().culture.cultural_encounters, 1);
    assert_eq!(loaded.world().culture.festivals_visited, 1);
    assert_eq!(loaded.world().culture.active_festivals.len(), 1);
    assert_eq!(
        loaded.world().culture.active_festivals[0].festival_id,
        "harvest_of_plenty"
    );
    assert_eq!(
        loaded.narrative().fired,
        ["first_trade".to_string(), "first_profit".to_string()]
    );
    assert_eq!(loaded.narrative().journal[1].region, "");
    assert_eq!(loaded.world().nemesis_id.as_deref(), Some("the_butcher"));

    let dir = scratch("area6");
    let mut loaded = loaded;
    loaded.save(&dir, "area6_v12").unwrap();
    let rust: Value = serde_json::from_str(
        &fs::read_to_string(dir.join("saves").join("area6_v12.json")).unwrap(),
    )
    .unwrap();
    for path in [
        "captain.reputation",
        "cultural_state",
        "narrative",
        "pirate_state.nemesis_id",
    ] {
        close(&dig(&fixture, path), &dig(&rust, path), path);
    }
    let again = Session::load(&dir, "area6_v12").unwrap().unwrap();
    assert_eq!(again.world().captain.reputation, 3);
    assert_eq!(again.narrative().journal[0].port_id, "porto_novo");
    assert_eq!(again.world().nemesis_id.as_deref(), Some("the_butcher"));
    let _ = fs::remove_dir_all(&dir);
}

/// Python `GameSession.new` wrote this slot for Sable Quinn (custom, seed 1).
///
/// A reloaded custom captain reverts to merchant pricing in both games. The
/// v12 file does not store the built template. Python's
/// `GameSession.captain_template` hits `KeyError` on `CaptainType.CUSTOM` and
/// returns the merchant archetype. Rust `captain_template` does the same when
/// `world.custom_captain` is absent. Porto Novo porcelain is 176/151 in the
/// file (new-game prices, no captain modifier). Seven trade points would sell
/// it at 194. After load both games reprice with the merchant modifiers:
/// buy 162, sell 159.
#[test]
fn python_custom_captain_v12_loads_name_type_silver_and_day() {
    let root = parity_root();
    let file: Value = serde_json::from_str(
        &fs::read_to_string(root.join("saves/custom_captain_v12.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(file["version"], 12);
    let on_disk = file["ports"]["porto_novo"]["market"]
        .as_array()
        .unwrap()
        .iter()
        .find(|slot| slot["good_id"] == "porcelain")
        .unwrap();
    assert_eq!(on_disk["buy_price"], 176);
    assert_eq!(on_disk["sell_price"], 151);

    let loaded = Session::load(&root, "custom_captain_v12").unwrap().unwrap();
    let captain = &loaded.world().captain;
    assert_eq!(captain.name, "Sable Quinn");
    assert_eq!(captain.captain_type, "custom");
    assert_eq!(captain.silver, 500);
    assert_eq!(loaded.world().day, 1);
    assert_eq!(captain.day, 1);
    assert!(loaded.world().custom_captain.is_none());
    let port = loaded
        .world()
        .ports
        .iter()
        .find(|port| port.id == "porto_novo")
        .unwrap();
    let porcelain = port
        .market
        .iter()
        .find(|slot| slot.good_id == "porcelain")
        .unwrap();
    assert_eq!(porcelain.buy_price, 162);
    assert_eq!(porcelain.sell_price, 159);
}

/// Rust wrote this v13 slot (seed 1, 300 silver banked at Porto Novo). It is
/// not a Python golden: the Python game has no bank.
#[test]
fn rust_v13_bank_round_trips() {
    let root = parity_root();
    let file: Value =
        serde_json::from_str(&fs::read_to_string(root.join("saves/bank_v13.json")).unwrap())
            .unwrap();
    assert_eq!(file["version"], 13);
    let mut loaded = Session::load(&root, "bank_v13").unwrap().unwrap();
    assert_eq!(loaded.bank_balances(), [("porto_novo".to_string(), 300)]);
    assert_eq!(loaded.world().captain.silver, 250);
    let dir = scratch("bank-v13");
    loaded.save(&dir, "bank_v13").unwrap();
    let again: Value =
        serde_json::from_str(&fs::read_to_string(dir.join("saves").join("bank_v13.json")).unwrap())
            .unwrap();
    assert_eq!(again["version"], 13);
    assert_eq!(
        again["infrastructure"]["bank"],
        file["infrastructure"]["bank"]
    );
    let _ = fs::remove_dir_all(&dir);
}

/// Written at commit e1ca59b33292bb8dc14701f304e403e7a51fd5a8 (the #85
/// merge, the last main before counting houses existed): the first by
/// `portlight script` from `PRE_BANK_SCRIPT`, the second by `Session::load`
/// of the first and `Session::save` again (a load reprices the markets, so the two differ). A game with no
/// account must still write both byte for byte.
///
/// Both files stay frozen, not regenerated: they are the proof that a save
/// written before counting houses loads and re-saves unchanged, and a file
/// written by today's code could not prove that. A saved offer keeps its
/// stored reason, so the load and re-save half is still exact. Since then the
/// sim words the shortage offer reason differently ("Shortage at {port} -
/// urgent demand for {good name}"); a fresh game writes that one value
/// anew, so the fresh-write half swaps it into the frozen bytes and compares
/// everything else exactly.
const PRE_BANK_SCRIPT: &str = "new merchant Ada 1\nbuy grain 5\nbuy_infrastructure warehouse depot\ndeposit grain 2\nbuy_insurance hull_basic\n";

/// The one value a fresh game now writes differently from the frozen file.
const OLD_SHORTAGE_REASON: &str = "Shortage at Corsair's Rest \u{2014} urgent demand for grain";
const NEW_SHORTAGE_REASON: &str = "Shortage at Corsair's Rest - urgent demand for Grain";

#[test]
fn pre_bank_v12_save_is_byte_identical() {
    let root = parity_root();
    let fixture = fs::read(root.join("saves/pre_bank_v12.json")).unwrap();
    let resaved_fixture = fs::read(root.join("saves/pre_bank_v12_resaved.json")).unwrap();
    let dir = scratch("pre-bank-v12");
    portlight_sim::script::run_and_save(PRE_BANK_SCRIPT, &dir, "pre_bank_v12").unwrap();
    let written = fs::read(dir.join("saves").join("pre_bank_v12.json")).unwrap();
    let frozen = String::from_utf8(fixture.clone()).unwrap();
    assert_eq!(frozen.matches(OLD_SHORTAGE_REASON).count(), 1);
    let fresh = frozen.replace(OLD_SHORTAGE_REASON, NEW_SHORTAGE_REASON);
    assert!(
        written == fresh.as_bytes(),
        "a new game without an account wrote different bytes"
    );
    let mut loaded = Session::load(&root, "pre_bank_v12").unwrap().unwrap();
    assert_eq!(loaded.banked_total(), 0);
    loaded.save(&dir, "pre_bank_v12_resaved").unwrap();
    let resaved = fs::read(dir.join("saves").join("pre_bank_v12_resaved.json")).unwrap();
    assert!(
        resaved == resaved_fixture,
        "load and re-save wrote different bytes"
    );
    let _ = fs::remove_dir_all(&dir);
}

/// Lane names survive a save and load, and a loaded game's lane voyage
/// draws from that lane's own encounter table, not the region fallback.
#[test]
fn lane_names_and_lane_encounters_survive_save_and_load() {
    let mut fresh = Session::new("Ada", "merchant", 1, None).unwrap();
    let named = fresh
        .world()
        .routes
        .iter()
        .filter(|route| !route.lore_name.is_empty())
        .count();
    assert_eq!(named, 15);
    let dir = scratch("lane-names");
    fresh.save(&dir, "lanes").unwrap();
    let mut loaded = Session::load(&dir, "lanes").unwrap().unwrap();
    assert_lane_names_match(&fresh, &loaded);

    // Porto Novo to Silva Bay is The Timber Run.
    let lane = loaded
        .world()
        .find_route("porto_novo", "silva_bay")
        .map(|route| route.lore_name.clone())
        .unwrap();
    assert_eq!(lane, "The Timber Run");
    let catalog = portlight_sim::content::content();
    let lane_texts: Vec<&str> = catalog
        .route_encounters(&lane)
        .unwrap()
        .encounters
        .iter()
        .map(|e| e.text.as_str())
        .collect();
    let mut seen_lane = 0;
    let mut here = "porto_novo".to_string();
    for _ in 0..40 {
        if seen_lane > 0 {
            break;
        }
        let there = if here == "porto_novo" {
            "silva_bay"
        } else {
            "porto_novo"
        };
        loaded.provision(10).ok();
        if loaded.depart(there).is_err() {
            break;
        }
        for _ in 0..20 {
            if loaded.world().voyage.status != VoyageStatus::AtSea {
                break;
            }
            if loaded.world().pending_duel.is_some() {
                break;
            }
            let Ok(turn) = loaded.advance() else { break };
            seen_lane += turn
                .events
                .iter()
                .filter(|event| lane_texts.contains(&event.message.as_str()))
                .count();
        }
        if loaded.world().pending_duel.is_some()
            || loaded.world().voyage.status == VoyageStatus::AtSea
        {
            break;
        }
        here = there.to_string();
    }
    assert!(seen_lane > 0, "no Timber Run encounter after a load");
    let _ = fs::remove_dir_all(&dir);
}
