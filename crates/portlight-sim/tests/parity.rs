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
fn encounter_goldens_cover_flee_bands() {
    let root = parity_root();
    for name in [
        "encounter_negotiate",
        "encounter_flee",
        "naval_combat",
        "boarding",
    ] {
        assert!(
            root.join("scripts").join(format!("{name}.txt")).is_file(),
            "missing script {name}"
        );
        assert!(
            root.join("golden").join(format!("{name}.json")).is_file(),
            "missing golden {name}"
        );
    }
    let flee: Value = serde_json::from_str(
        &fs::read_to_string(root.join("golden/encounter_flee.json")).expect("flee golden"),
    )
    .expect("flee json");
    let mut low = false;
    let mut mid = false;
    let mut high = false;
    for log in flee["log"].as_array().expect("log") {
        let Some(strength) = log
            .get("encounter")
            .and_then(|item| item.get("enemy_strength"))
            .and_then(|item| item.as_i64())
        else {
            continue;
        };
        if strength <= 3 {
            low = true;
        } else if strength <= 6 {
            mid = true;
        } else {
            high = true;
        }
    }
    assert!(low, "flee golden has no strength <= 3");
    assert!(mid, "flee golden has no strength 4-6");
    assert!(high, "flee golden has no strength >= 7");
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

/// Area 3 scripts. `golden_scripts_match_python` also runs every file in
/// `parity/scripts`; this test names the four that must exist.
const AREA3_SCRIPTS: &[&str] = &[
    "train_crew",
    "recruit_companion",
    "skill_spend",
    "milestone_reached",
];

#[test]
fn area3_goldens_cover_training_recruiting_skill_and_milestone() {
    let root = parity_root();
    for stem in AREA3_SCRIPTS {
        let script_path = root.join("scripts").join(format!("{stem}.txt"));
        let golden_path = root.join("golden").join(format!("{stem}.json"));
        let script = fs::read_to_string(&script_path)
            .unwrap_or_else(|_| panic!("missing {}", script_path.display()));
        let golden: Value = serde_json::from_str(
            &fs::read_to_string(&golden_path)
                .unwrap_or_else(|_| panic!("missing {}", golden_path.display())),
        )
        .expect("golden json");
        let got = serde_json::to_value(run_script(&script)).expect("snapshot");
        close(&golden, &got, stem);
    }
    let train = load_golden("train_crew");
    assert!(train["captain"]["learned_styles"]
        .as_array()
        .unwrap()
        .iter()
        .any(|style| style == "la_destreza"));
    let recruit = load_golden("recruit_companion");
    assert_eq!(
        recruit["captain"]["companions"][0]["companion_id"],
        "red_tomas"
    );
    let skill = load_golden("skill_spend");
    assert_eq!(skill["captain"]["skills"][0]["id"], "blacksmith");
    assert_eq!(skill["captain"]["skills"][0]["level"], 1);
    let milestone = load_golden("milestone_reached");
    let ids: Vec<_> = milestone["milestones"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|item| item["milestone_id"].as_str())
        .collect();
    assert!(ids.contains(&"foothold_standing_established"));
    assert!(ids.contains(&"lawful_credible_trust"));
    let families: Vec<_> = milestone["milestones"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|item| item["family"].as_str())
        .collect();
    assert!(families.contains(&"regional_foothold"));
    assert!(families.contains(&"lawful_house"));
    assert!(!families.contains(&"commercial_empire"));
    assert!(!ids.contains(&"commercial_finance"));
}

#[test]
fn sea_captain_agency_golden_records_the_ambush() {
    let root = parity_root();
    let script = fs::read_to_string(root.join("scripts/sea_captain_agency.txt")).unwrap();
    let golden = load_golden("sea_captain_agency");
    let got = serde_json::to_value(run_script(&script)).expect("snapshot");
    close(&golden, &got, "sea_captain_agency");
    let calls: Vec<_> = golden["log"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|entry| entry["command"] == "agency")
        .collect();
    assert_eq!(calls.len(), 3);
    assert_eq!(calls[0]["agency"]["notices"].as_array().unwrap().len(), 0);
    assert_eq!(calls[1]["agency"]["ambush"], false);
    assert_eq!(calls[2]["agency"]["ambush"], true);
    assert_eq!(
        calls[2]["agency"]["encounter"]["enemy_captain_id"],
        "the_butcher"
    );
    assert_eq!(calls[2]["agency"]["encounter"]["phase"], "naval");
    assert_eq!(golden["pending_duel"]["captain_id"], "the_butcher");
    assert_eq!(golden["day"], 4);
}

fn load_golden(stem: &str) -> Value {
    let path = parity_root().join("golden").join(format!("{stem}.json"));
    serde_json::from_str(&fs::read_to_string(path).expect("golden")).expect("json")
}

#[test]
fn infrastructure_goldens_cover_warehouse_broker_credit_and_insurance() {
    let warehouse = load_golden("buy_warehouse");
    let wh = &warehouse["infrastructure"]["warehouses"][0];
    assert_eq!(wh["tier"], "depot");
    assert_eq!(wh["port_id"], "porto_novo");
    assert_eq!(wh["upkeep_paid_through"], 4);
    assert_eq!(wh["active"], true);
    let broker = load_golden("buy_broker");
    assert_eq!(
        broker["infrastructure"]["brokers"][0]["region"],
        "Mediterranean"
    );
    assert_eq!(broker["infrastructure"]["brokers"][0]["tier"], "local");
    let credit = load_golden("take_credit");
    assert_eq!(credit["infrastructure"]["credit"]["total_borrowed"], 100);
    assert!(
        credit["infrastructure"]["credit"]["total_repaid"]
            .as_i64()
            .unwrap_or(0)
            > 0
    );
    let insurance = load_golden("buy_insurance");
    assert_eq!(
        insurance["infrastructure"]["policies"][0]["spec_id"],
        "hull_basic"
    );
    assert_eq!(insurance["infrastructure"]["policies"][0]["family"], "hull");
}
