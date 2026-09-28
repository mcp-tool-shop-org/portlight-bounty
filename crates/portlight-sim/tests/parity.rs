//! Compares scripted runs with golden snapshots produced by the Python oracle.

use std::fs;
use std::path::PathBuf;

use portlight_sim::run_script;
use serde_json::Value;

fn parity_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../parity")
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

#[test]
fn golden_scripts_match_python() {
    let root = parity_root();
    let mut names: Vec<_> = fs::read_dir(root.join("scripts"))
        .expect("parity scripts")
        .map(|entry| entry.expect("dir entry").file_name())
        .filter(|name| name.to_string_lossy().ends_with(".txt"))
        .collect();
    names.sort();
    assert!(!names.is_empty(), "no parity scripts");
    for name in names {
        let stem = name.to_string_lossy();
        let stem = stem.trim_end_matches(".txt");
        let script = fs::read_to_string(root.join("scripts").join(&name)).expect("script");
        let golden_text = fs::read_to_string(root.join("golden").join(format!("{stem}.json")))
            .unwrap_or_else(|_| {
                panic!("missing golden for {stem}; run tools/parity/check.py --write-golden")
            });
        let golden: Value = serde_json::from_str(&golden_text).expect("golden json");
        let got = serde_json::to_value(run_script(&script)).expect("snapshot");
        close(&golden, &got, stem);
    }
}

#[test]
fn goldens_guard_the_checklist_paths() {
    let root = parity_root().join("golden");
    let mut events = std::collections::BTreeSet::new();
    let mut saw_inspection_incident = false;
    let mut saw_cargo_loss = false;
    let mut saw_crew_delta = false;
    let mut saw_contraband = false;
    let mut saw_crew_minimum = false;
    let mut saw_victory = false;
    let mut saw_hull_wear = false;
    let mut saw_duel_win = false;
    let mut saw_duel_draw = false;
    let mut saw_duel_loss = false;
    let mut saw_hire = false;
    let mut saw_provision = false;
    let mut saw_dock_work = false;
    for entry in fs::read_dir(&root).expect("golden dir") {
        let entry = entry.expect("entry");
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        if entry.file_name() == "victory_cases.json" {
            saw_victory = true;
            continue;
        }
        let value: Value =
            serde_json::from_str(&fs::read_to_string(entry.path()).expect("golden")).expect("json");
        if entry.file_name() == "hull_day20.json" {
            assert_eq!(
                value["captain"]["ship"]["hull_max"].as_i64(),
                Some(69),
                "day-20 hull wear"
            );
            assert_eq!(value["voyage"]["days_elapsed"].as_i64(), Some(20));
            saw_hull_wear = true;
        }
        if value
            .get("victory")
            .and_then(|item| item.as_array())
            .is_some_and(|paths| !paths.is_empty())
        {
            saw_victory = true;
            let ids: Vec<_> = value["victory"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|path| path.get("path_id").and_then(|id| id.as_str()))
                .collect();
            assert!(
                ids.contains(&"commercial_empire"),
                "{}",
                entry.path().display()
            );
            assert!(!ids.contains(&"commercial_finance"));
        }
        let empty = Vec::new();
        for log in value["log"].as_array().unwrap() {
            if let Some(error) = log.get("error").and_then(|item| item.as_str()) {
                if error.contains("won't touch") {
                    saw_contraband = true;
                }
                if error.contains("Need at least") && error.contains("crew") {
                    saw_crew_minimum = true;
                }
            }
            if log.get("command").and_then(|item| item.as_str()) == Some("hire 5") {
                saw_hire = true;
            }
            if log.get("command").and_then(|item| item.as_str()) == Some("provision 4") {
                saw_provision = true;
            }
            if let Some(earned) = log.get("earned").and_then(|item| item.as_i64()) {
                assert!((3..=5).contains(&earned), "dock work earned {earned}");
                saw_dock_work = true;
            }
            if let Some(duel) = log.get("duel").filter(|item| !item.is_null()) {
                if duel.get("player_won").and_then(|item| item.as_bool()) == Some(true) {
                    saw_duel_win = true;
                } else if duel.get("draw").and_then(|item| item.as_bool()) == Some(true) {
                    saw_duel_draw = true;
                } else {
                    saw_duel_loss = true;
                }
            }
            for event in log
                .get("events")
                .and_then(|item| item.as_array())
                .unwrap_or(&empty)
            {
                if let Some(kind) = event.get("event_type").and_then(|item| item.as_str()) {
                    events.insert(kind.to_string());
                }
                if event
                    .get("cargo_lost")
                    .and_then(|item| item.as_array())
                    .is_some_and(|lost| !lost.is_empty())
                {
                    saw_cargo_loss = true;
                }
                if event
                    .get("crew_delta")
                    .and_then(|item| item.as_i64())
                    .unwrap_or(0)
                    != 0
                {
                    saw_crew_delta = true;
                }
            }
        }
        for incident in value["captain"]["standing"]["incidents"]
            .as_array()
            .unwrap_or(&empty)
        {
            if incident.get("incident_type").and_then(|item| item.as_str()) == Some("inspection") {
                saw_inspection_incident = true;
            }
        }
    }
    let required = [
        "storm",
        "pirates",
        "inspection",
        "calm_seas",
        "favorable_wind",
        "provisions_spoiled",
        "cargo_damaged",
        "merchant_encounter",
        "flotsam",
        "nothing",
        "foreign_vessel",
        "cultural_waters",
        "sea_ceremony",
        "whale_sighting",
        "lighthouse",
        "musician_aboard",
        "drifting_offering",
        "star_navigation",
    ];
    for kind in required {
        assert!(events.contains(kind), "no golden contains {kind}");
    }
    assert!(
        saw_inspection_incident,
        "inspection reputation is not in a golden"
    );
    assert!(saw_cargo_loss, "cargo loss is not in a golden");
    assert!(saw_crew_delta, "crew casualty is not in a golden");
    assert!(saw_contraband, "contraband refusal is not in a golden");
    assert!(
        saw_crew_minimum,
        "crew minimum on depart is not in a golden"
    );
    assert!(saw_victory, "victory paths are not in a golden");
    assert!(saw_hull_wear, "day-20 hull wear is not in a golden");
    assert!(saw_duel_win, "a won duel is not in a golden");
    assert!(saw_duel_draw, "a drawn duel is not in a golden");
    assert!(saw_duel_loss, "a lost duel is not in a golden");
    assert!(saw_hire, "hiring crew is not in a golden");
    assert!(saw_provision, "buying provisions is not in a golden");
    assert!(saw_dock_work, "dock work is not in a golden");
    for name in [
        "contract_accept.json",
        "contract_arrival_rng.json",
        "contract_complete.json",
        "contract_expire.json",
    ] {
        assert!(
            root.join(name).is_file(),
            "missing golden {name}; run tools/parity/check.py --write-golden"
        );
    }
}
