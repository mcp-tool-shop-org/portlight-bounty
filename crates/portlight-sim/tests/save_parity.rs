//! Save/load checked against the Python loader's snapshot.

use std::fs;
use std::path::PathBuf;
use std::process;

use portlight_sim::campaign::{
    ActiveLicense, BrokerSite, CompletedContract, CreditBook, VictoryRecord, WarehouseSite,
};
use portlight_sim::model::VoyageStatus;
use portlight_sim::snapshot;
use portlight_sim::Session;
use serde_json::Value;

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

fn snap(session: &Session) -> Value {
    serde_json::to_value(snapshot::capture(
        session.world(),
        session.trade_seq(),
        session.books(),
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
        books.warehouses.push(WarehouseSite {
            port_id: "porto_novo".to_string(),
            active: true,
        });
        books.brokers.push(BrokerSite {
            region: "East Indies".to_string(),
            tier: "local".to_string(),
            active: true,
        });
        books.licenses.push(ActiveLicense {
            license_id: "ei_access_charter".to_string(),
            active: true,
        });
        books.policies = 1;
        books.credit = Some(CreditBook {
            total_borrowed: 500,
            defaults: 0,
            active: true,
            total_repaid: 0,
        });
        books.completed_paths.push(VictoryRecord {
            path_id: "lawful_house".to_string(),
            completion_day: 4,
            summary: "Recorded early".to_string(),
            is_first: true,
        });
    }
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
    assert_eq!(loaded.world().routes[0].lore, "");
    assert_eq!(loaded.world().routes[0].lore_name, "");
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
    assert!(loaded
        .world()
        .routes
        .iter()
        .all(|route| route.lore.is_empty()));

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
    fs::write(path.join("default.json"), b"{\"version\": 13}").unwrap();
    let err = Session::load(&dir, "default").unwrap_err();
    assert_eq!(
        err.to_string(),
        "Save file version 13 is newer than supported version 12. Update Portlight to load this save."
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
