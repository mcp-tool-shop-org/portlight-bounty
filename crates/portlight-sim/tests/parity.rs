//! Compares scripted runs with golden snapshots produced by the Python oracle.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use portlight_sim::run_script;
use serde_json::Value;

fn parity_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../parity")
}

fn close(left: &Value, right: &Value, path: &str, errors: &mut Vec<String>) {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => {
            let af = a.as_f64().expect("left number");
            let bf = b.as_f64().expect("right number");
            let diff = (af - bf).abs();
            if !(diff <= 1e-6 || diff <= 1e-9 * af.abs().max(bf.abs())) {
                errors.push(format!("{path}: {af} != {bf}"));
            }
        }
        (Value::String(a), Value::String(b)) => {
            if a != b {
                errors.push(format!("{path}: {a:?} != {b:?}"));
            }
        }
        (Value::Bool(a), Value::Bool(b)) => {
            if a != b {
                errors.push(format!("{path}: {a} != {b}"));
            }
        }
        (Value::Null, Value::Null) => {}
        (Value::Array(a), Value::Array(b)) => {
            let shared = a.len().min(b.len());
            for i in 0..shared {
                close(&a[i], &b[i], &format!("{path}[{i}]"), errors);
            }
            if a.len() > b.len() {
                for (i, value) in a.iter().enumerate().skip(shared) {
                    errors.push(format!("{path}[{i}]: {value} != <missing>"));
                }
            } else if b.len() > a.len() {
                for (i, value) in b.iter().enumerate().skip(shared) {
                    errors.push(format!("{path}[{i}]: <missing> != {value}"));
                }
            }
        }
        (Value::Object(a), Value::Object(b)) => {
            if a.len() != b.len() {
                errors.push(format!("{path}: keys differ"));
                return;
            }
            for (key, value) in a {
                let Some(other) = b.get(key) else {
                    errors.push(format!("{path}: missing {key}"));
                    return;
                };
                close(value, other, &format!("{path}.{key}"), errors);
            }
        }
        _ => errors.push(format!("{path}: type mismatch {left} vs {right}")),
    }
}

fn path_matches(path: &str, pattern: &str) -> bool {
    // Exact leaf, or that leaf plus trailing `[n]` indexes only.
    // `$.board.offers` does not match `$.board.offers[0].id`.
    if path == pattern {
        return true;
    }
    let Some(rest) = path.strip_prefix(pattern) else {
        return false;
    };
    if !rest.starts_with('[') {
        return false;
    }
    let bytes = rest.as_bytes();
    let mut index = 0;
    let mut saw = false;
    while index < bytes.len() {
        if bytes[index] != b'[' {
            return false;
        }
        index += 1;
        let start = index;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            index += 1;
        }
        if index == start || index >= bytes.len() || bytes[index] != b']' {
            return false;
        }
        index += 1;
        saw = true;
    }
    saw
}

fn error_path(line: &str) -> &str {
    line.split(": ").next().unwrap_or(line)
}

/// Serializes the narrow snapshot. Clearing the variable is paired with the
/// lock so one test cannot widen another test's golden JSON.
static WIDE_ENV: Mutex<()> = Mutex::new(());

fn narrow_value(script: &str) -> Value {
    std::env::remove_var("PORTLIGHT_WIDE_SNAPSHOT");
    serde_json::to_value(run_script(script)).expect("snapshot")
}

fn script_value(script: &str) -> Value {
    let _guard = WIDE_ENV.lock().unwrap_or_else(|err| err.into_inner());
    narrow_value(script)
}

struct Pin {
    path: String,
    python: Value,
    rust: Value,
}

fn oracle_pins(root: &Path, script: &str) -> Vec<Pin> {
    let text = fs::read_to_string(root.join("expected_divergences.json")).unwrap_or_default();
    if text.is_empty() {
        return Vec::new();
    }
    let data: Value = serde_json::from_str(&text).expect("expected_divergences.json");
    let mut pins = Vec::new();
    for entry in data
        .get("entries")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if entry.get("script").and_then(Value::as_str) != Some(script) {
            continue;
        }
        if entry.get("check").and_then(Value::as_str) != Some("oracle") {
            continue;
        }
        for spec in entry
            .get("paths")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let path = spec
                .get("path")
                .and_then(Value::as_str)
                .expect("allowlist path")
                .to_string();
            pins.push(Pin {
                path,
                python: spec.get("python").cloned().expect("python pin"),
                rust: spec.get("rust").cloned().expect("rust pin"),
            });
        }
    }
    pins
}

fn pattern_error(path: &str) -> Option<String> {
    if path.contains('*') {
        Some(format!("allowlist path contains a wildcard: {path}"))
    } else {
        None
    }
}

fn lookup<'a>(value: &'a Value, path: &str) -> Result<Option<&'a Value>, String> {
    if let Some(message) = pattern_error(path) {
        return Err(message);
    }
    if path == "$" {
        return Ok(Some(value));
    }
    let mut cur = value;
    let mut rest = path
        .strip_prefix('$')
        .ok_or_else(|| format!("allowlist path contains a wildcard: {path}"))?;
    while !rest.is_empty() {
        if let Some(stripped) = rest.strip_prefix('.') {
            let end = stripped.find(['.', '[']).unwrap_or(stripped.len());
            let key = &stripped[..end];
            cur = match cur.get(key) {
                Some(next) => next,
                None => return Ok(None),
            };
            rest = &stripped[end..];
        } else if let Some(stripped) = rest.strip_prefix('[') {
            let end = stripped
                .find(']')
                .ok_or_else(|| format!("allowlist path contains a wildcard: {path}"))?;
            let index: usize = stripped[..end]
                .parse()
                .map_err(|_| format!("allowlist path contains a wildcard: {path}"))?;
            cur = match cur.get(index) {
                Some(next) => next,
                None => return Ok(None),
            };
            rest = &stripped[end + 1..];
        } else {
            return Err(format!("allowlist path contains a wildcard: {path}"));
        }
    }
    Ok(Some(cur))
}

fn is_missing_pin(value: &Value) -> bool {
    value.as_object().is_some_and(|map| {
        map.len() == 1 && map.get("$missing").and_then(Value::as_bool) == Some(true)
    })
}

fn values_pinned(actual: Option<&Value>, expected: &Value) -> bool {
    if is_missing_pin(expected) {
        return actual.is_none();
    }
    let Some(actual) = actual else {
        return false;
    };
    let mut errors = Vec::new();
    close(actual, expected, "$", &mut errors);
    errors.is_empty()
}

fn pattern_names_container(left: &Value, right: &Value, pattern: &str) -> Result<bool, String> {
    for snap in [left, right] {
        if matches!(
            lookup(snap, pattern)?,
            Some(Value::Array(_)) | Some(Value::Object(_))
        ) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn assert_close(left: &Value, right: &Value, path: &str) {
    let mut errors = Vec::new();
    close(left, right, path, &mut errors);
    assert!(errors.is_empty(), "{path}: {errors:?}");
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
        let mut golden: Value = serde_json::from_str(&golden_text).expect("golden json");
        let python_golden = golden.get("_source").and_then(Value::as_str) == Some("python-oracle");
        if let Value::Object(map) = &mut golden {
            map.retain(|key, _| !key.starts_with('_'));
        }
        let got = script_value(&script);
        let mut errors = Vec::new();
        close(&golden, &got, "$", &mut errors);
        if python_golden {
            let pins = oracle_pins(&root, &format!("{stem}.txt"));
            for pin in &pins {
                if let Some(message) = pattern_error(&pin.path) {
                    panic!("{stem}: {message}");
                }
                assert!(
                    !pattern_names_container(&golden, &got, &pin.path)
                        .unwrap_or_else(|message| panic!("{stem}: {message}")),
                    "{stem} allowlist path names a list or object: {}",
                    pin.path
                );
            }
            let unexpected: Vec<_> = errors
                .iter()
                .filter(|line| {
                    !pins
                        .iter()
                        .any(|pin| path_matches(error_path(line), &pin.path))
                })
                .cloned()
                .collect();
            assert!(
                unexpected.is_empty(),
                "{stem} python golden differs outside the allowlist: {unexpected:?}"
            );
            let missing: Vec<_> = pins
                .iter()
                .filter(|pin| {
                    !errors
                        .iter()
                        .any(|line| path_matches(error_path(line), &pin.path))
                })
                .map(|pin| pin.path.clone())
                .collect();
            assert!(
                missing.is_empty(),
                "{stem} python golden no longer differs on {missing:?}"
            );
            for pin in &pins {
                assert!(
                    values_pinned(
                        lookup(&golden, &pin.path)
                            .unwrap_or_else(|message| panic!("{stem}: {message}")),
                        &pin.python
                    ),
                    "{stem} {} python value is not the pin",
                    pin.path
                );
                assert!(
                    values_pinned(
                        lookup(&got, &pin.path)
                            .unwrap_or_else(|message| panic!("{stem}: {message}")),
                        &pin.rust
                    ),
                    "{stem} {} rust value is not the pin",
                    pin.path
                );
            }
        } else {
            assert!(errors.is_empty(), "{stem}: {errors:?}");
        }
    }
}

#[test]
fn container_pattern_does_not_hide_an_offer_mutation() {
    let left = serde_json::json!({"board": {"offers": [{"id": "kept", "reward_silver": 10}]}});
    let mut right = left.clone();
    right["board"]["offers"][0]["id"] = serde_json::json!("mutated");
    right["board"]["offers"][0]["reward_silver"] = serde_json::json!(99);
    let mut errors = Vec::new();
    close(&left, &right, "$", &mut errors);
    let pattern = "$.board.offers";
    assert!(
        pattern_names_container(&left, &right, pattern).expect("pattern"),
        "offers is a list and must be rejected"
    );
    let hidden: Vec<_> = errors
        .iter()
        .filter(|line| path_matches(error_path(line), pattern))
        .cloned()
        .collect();
    assert!(
        hidden.is_empty(),
        "container pattern swallowed the mutation: {hidden:?}"
    );
    assert!(
        errors
            .iter()
            .any(|line| error_path(line) == "$.board.offers[0].id"),
        "{errors:?}"
    );
}

#[test]
fn enemy_crew_pin_rejects_a_different_rust_value() {
    let root = parity_root();
    let pins = oracle_pins(&root, "capture_prize.txt");
    let crew = pins
        .iter()
        .find(|pin| pin.path.ends_with("enemy_crew"))
        .expect("enemy_crew pin");
    assert_eq!(crew.python, serde_json::json!(5));
    assert_eq!(crew.rust, serde_json::json!(0));
    let python = serde_json::json!({"log": [{"encounter": {"enemy_crew": 5}}]});
    let mut rust = python.clone();
    rust["log"][0]["encounter"]["enemy_crew"] = serde_json::json!(99);
    let path = "$.log[0].encounter.enemy_crew";
    assert!(values_pinned(
        lookup(&python, path).expect("path"),
        &crew.python
    ));
    assert!(
        !values_pinned(lookup(&rust, path).expect("path"), &crew.rust),
        "enemy_crew 99 must not satisfy the pinned rust value"
    );
}

#[test]
fn roundtrip_dropped_offer_is_not_allowlisted() {
    let offer = || serde_json::json!({"id": "kept", "reward_silver": 10});
    let python = serde_json::json!({
        "board": {"offers": [offer(), offer(), offer(), offer(), offer()]}
    });
    let mut rust = python.clone();
    rust["board"]["offers"]
        .as_array_mut()
        .expect("offers")
        .pop();
    let mut errors = Vec::new();
    close(&rust, &python, "$", &mut errors);
    let pattern = "$.board.offers";
    assert!(
        pattern_names_container(&python, &rust, pattern).expect("pattern"),
        "offers is a list and must be rejected"
    );
    assert!(
        errors
            .iter()
            .any(|line| error_path(line) == "$.board.offers[4]"),
        "{errors:?}"
    );
    assert!(
        errors
            .iter()
            .any(|line| path_matches(error_path(line), pattern)),
        "trailing index would hide the dropped offer: {errors:?}"
    );
    let bogus = serde_json::json!({"id": "bogus"});
    assert!(
        !values_pinned(lookup(&rust, "$.board.offers[4]").expect("path"), &bogus),
        "a bogus pin must not match the dropped offer"
    );
    assert!(lookup(&python, "$.board.offers[4]")
        .expect("path")
        .is_some());
}

#[test]
fn wildcard_pattern_is_rejected() {
    let err = lookup(
        &serde_json::json!({"board": {"offers": [{"id": "kept"}]}}),
        "$.board.offers[*].id",
    )
    .expect_err("wildcard must be an error");
    assert!(err.contains("wildcard"), "{err}");
    assert!(err.contains("[*]"), "{err}");
    let container = pattern_names_container(
        &serde_json::json!({"board": {"offers": []}}),
        &serde_json::json!({"board": {"offers": []}}),
        "$.board.offers[*]",
    );
    assert!(container.is_err(), "{container:?}");
    assert!(container.unwrap_err().contains("wildcard"));
}

#[test]
fn golden_compare_ignores_wide_snapshot_env() {
    let _guard = WIDE_ENV.lock().unwrap_or_else(|err| err.into_inner());
    std::env::set_var("PORTLIGHT_WIDE_SNAPSHOT", "1");
    let value = narrow_value("new merchant Ada 1\n");
    assert!(value.get("board").is_none(), "{value}");
    assert!(value.get("ledger").is_none(), "{value}");
    assert!(std::env::var("PORTLIGHT_WIDE_SNAPSHOT").is_err());
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
        // Catalog and save-slot fixtures, not script snapshots.
        if entry.file_name() == "roster_options.json" || entry.file_name() == "save_slots.json" {
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
        let got = script_value(&script);
        assert_close(&golden, &got, stem);
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
    let got = script_value(&script);
    assert_close(&golden, &got, "sea_captain_agency");
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

#[test]
fn hunting_and_bounty_goldens_cover_yields_refusals_and_the_hunter() {
    let success = load_golden("hunt_port_success");
    assert_eq!(success["log"][1]["hunt"]["success"], true);
    assert_eq!(success["log"][1]["hunt"]["location"], "port");
    assert_eq!(success["captain"]["cargo"][0]["good_id"], "pelts");

    let miss = load_golden("hunt_port_fail");
    assert_eq!(miss["log"][1]["hunt"]["success"], false);
    assert_eq!(miss["captain"]["cargo"].as_array().unwrap().len(), 0);

    let sea = load_golden("hunt_sea_success");
    assert_eq!(sea["log"][2]["hunt"]["location"], "sea");
    assert_eq!(sea["log"][2]["hunt"]["success"], true);

    let sea_miss = load_golden("hunt_sea_fail");
    assert_eq!(sea_miss["log"][2]["hunt"]["success"], false);
    assert_eq!(sea_miss["log"][2]["hunt"]["danger_text"], "");

    let morale = load_golden("hunt_sea_morale");
    let last = morale["log"].as_array().unwrap().last().unwrap();
    assert_eq!(
        last["error"],
        "Crew morale too low for hunting at sea (need 20+)."
    );

    let claim = load_golden("bounty_claim");
    let claim_log = claim["log"].as_array().unwrap();
    assert_eq!(claim_log[claim_log.len() - 2]["bounty"]["reward"], 120);
    assert_eq!(
        claim_log[claim_log.len() - 2]["bounty"]["target_id"],
        "raj_the_quiet"
    );
    assert_eq!(
        claim["log"].as_array().unwrap().last().unwrap()["error"],
        "Bounty already claimed"
    );

    let hunter = load_golden("bounty_hunter_voyage");
    assert_eq!(hunter["captain"]["wanted_level"], 3);
    let flavors: Vec<_> = hunter["log"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|entry| entry["events"].as_array().into_iter().flatten())
        .map(|event| event["flavor"].as_str().unwrap_or(""))
        .collect();
    assert!(
        flavors
            .iter()
            .any(|flavor| flavor.starts_with("Bounty hunter")),
        "bounty hunter event missing: {flavors:?}"
    );
    assert_eq!(hunter["pending_duel"]["captain_id"], "iron_hound");

    let sailed = load_golden("bounty_claim_sail");
    assert!(sailed["pending_duel"].is_null());
    assert_eq!(sailed["voyage"]["status"], "at_sea");
    assert!(sailed["log"]
        .as_array()
        .unwrap()
        .iter()
        .all(|entry| entry.get("duel").is_none() || entry["duel"].is_null()));
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
