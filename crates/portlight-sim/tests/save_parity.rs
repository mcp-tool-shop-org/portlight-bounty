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
