//! The tuned overlay: allow-list, drift guard and exact reach.

use portlight_sim::content::{content, overlay_active};
use portlight_sim::tuned::{
    apply, base_value, diff_paths, overlay_value, rows, tuned_value, BIAS_CAPTAINS, TUNED_CAPTAINS,
};
use serde_json::{json, Value};
use std::panic::catch_unwind;

/// Turns `captains[id=x].pricing.y` into `.captains[3].pricing.y` for diffing.
fn index_path(base: &Value, path: &str) -> String {
    let (list, rest) = path.split_once("[id=").expect("bracket");
    let (id, tail) = rest.split_once(']').expect("close");
    let items = base[list].as_array().expect("list");
    let at = items
        .iter()
        .position(|item| item["id"] == id)
        .expect("id in base");
    format!(".{list}[{at}]{tail}")
}

fn overlay_of(rows: Value) -> Value {
    json!({ "note": "test", "rows": rows })
}

fn rejects(overlay: Value, needle: &str) {
    let base = base_value();
    let err = catch_unwind(|| apply(&base, &overlay)).expect_err("overlay must panic");
    let text = err
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| err.downcast_ref::<&str>().map(|s| s.to_string()))
        .unwrap_or_default();
    assert!(
        text.contains(needle),
        "panic text {text:?} lacks {needle:?}"
    );
}

#[test]
fn embedded_overlay_validates_and_changes_only_its_rows() {
    let base = base_value();
    let tuned = tuned_value();
    let parsed = rows(&overlay_value());
    assert!(!parsed.is_empty());
    let mut got = Vec::new();
    diff_paths(&base, &tuned, "", &mut got);
    got.sort();
    let mut want: Vec<String> = parsed.iter().map(|r| index_path(&base, &r.path)).collect();
    want.sort();
    assert_eq!(
        got, want,
        "content differs from the extract outside the rows"
    );
    for row in &parsed {
        let mut here = Vec::new();
        diff_paths(&base, &tuned, "", &mut here);
        assert!(here.contains(&index_path(&base, &row.path)), "{}", row.path);
    }
}

#[test]
fn overlay_reaches_only_the_allow_list() {
    for row in rows(&overlay_value()) {
        let path = row.path.as_str();
        if let Some(rest) = path.strip_prefix("captains[id=") {
            let id = rest.split(']').next().unwrap();
            assert!(TUNED_CAPTAINS.contains(&id), "{path}");
            assert!(
                [
                    "pricing.buy_price_mult",
                    "pricing.sell_price_mult",
                    "pricing.port_fee_mult",
                    "voyage.provision_burn"
                ]
                .iter()
                .any(|f| path.ends_with(f)),
                "{path}"
            );
        } else {
            assert!(path.starts_with("contracts[id="), "{path}");
            assert!(path.ends_with(".captain_bias"), "{path}");
            let added = &row.tuned.as_array().unwrap()[row.base.as_array().unwrap().len()..];
            assert!(added
                .iter()
                .all(|v| BIAS_CAPTAINS.contains(&v.as_str().unwrap())));
        }
    }
}

#[test]
fn silver_and_every_other_path_is_refused() {
    for path in [
        "captains[id=dockhand].starting_silver",
        "captains[id=dockhand].pricing.luxury_sell_bonus",
        "captains[id=dockhand].voyage.speed_bonus",
        "captains[id=dockhand].inspection.seizure_risk",
        "captains[id=merchant].pricing.buy_price_mult",
        "captains[id=navigator].pricing.buy_price_mult",
        "captains.dockhand.pricing.buy_price_mult",
        "ships[id=cutter].hold",
        "goods[id=grain].base_price",
        "contracts[id=proc_grain_feed].reward_per_unit",
        "captains[*].pricing.buy_price_mult",
    ] {
        let overlay = overlay_of(json!([{"path": path, "base": 1.0, "tuned": 1.1}]));
        let base = base_value();
        assert!(
            catch_unwind(|| apply(&base, &overlay)).is_err(),
            "{path} must be refused"
        );
    }
}

#[test]
fn a_wrong_base_value_is_refused() {
    rejects(
        overlay_of(json!([{
            "path": "captains[id=dockhand].pricing.buy_price_mult",
            "base": 1.5,
            "tuned": 1.1
        }])),
        "base does not match",
    );
}

#[test]
fn extreme_missing_or_unchanged_values_are_refused() {
    let path = "captains[id=dockhand].pricing.buy_price_mult";
    rejects(
        overlay_of(json!([{"path": path, "base": 1.0, "tuned": 0.1}])),
        "outside",
    );
    rejects(
        overlay_of(json!([{"path": path, "base": 1.0, "tuned": 9.0}])),
        "outside",
    );
    rejects(
        overlay_of(json!([{"path": path, "base": 1.0, "tuned": 1.0}])),
        "changes nothing",
    );
    rejects(
        overlay_of(json!([{"path": path, "base": 1.0, "tuned": "1.1"}])),
        "not a number",
    );
    rejects(
        overlay_of(json!([{"path": path, "base": 1.0}])),
        "without tuned",
    );
    rejects(
        overlay_of(json!([
            {"path": path, "base": 1.0, "tuned": 1.1},
            {"path": path, "base": 1.0, "tuned": 1.2}
        ])),
        "duplicate",
    );
    rejects(
        overlay_of(json!([{"path": path, "base": 1.0, "tuned": 1.1, "extra": 1}])),
        "unknown row key",
    );
    rejects(json!({"rows": [], "silver": 1}), "unknown top-level key");
}

#[test]
fn captain_bias_is_append_only() {
    let path = "contracts[id=proc_grain_feed].captain_bias";
    let base = base_value();
    let current = base["contracts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == "proc_grain_feed")
        .unwrap()["captain_bias"]
        .clone();
    let mut appended = current.as_array().unwrap().clone();
    appended.push(json!("dockhand"));
    let ok = overlay_of(json!([{"path": path, "base": current, "tuned": appended}]));
    let tuned = apply(&base, &ok);
    assert!(tuned["contracts"]
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c["id"] == "proc_grain_feed"
            && c["captain_bias"].as_array().unwrap().last().unwrap() == "dockhand"));
    // Removing, reordering, replacing, adding a disallowed captain or a
    // duplicate are all refused.
    rejects(
        overlay_of(json!([{"path": path, "base": current, "tuned": []}])),
        "append-only",
    );
    rejects(
        overlay_of(json!([{"path": path, "base": current, "tuned": ["dockhand"]}])),
        "append-only",
    );
    rejects(
        overlay_of(json!([{"path": path, "base": current, "tuned": ["navigator", "dockhand"]}])),
        "append-only",
    );
    let mut with_smuggler = current.as_array().unwrap().clone();
    with_smuggler.push(json!("smuggler"));
    rejects(
        overlay_of(json!([{"path": path, "base": current, "tuned": with_smuggler}])),
        "not allowed",
    );
    let mut twice = current.as_array().unwrap().clone();
    twice.push(json!("dockhand"));
    twice.push(json!("dockhand"));
    rejects(
        overlay_of(json!([{"path": path, "base": current, "tuned": twice}])),
        "already present",
    );
    rejects(
        overlay_of(json!([{"path": path, "base": ["wrong"], "tuned": ["wrong", "dockhand"]}])),
        "base does not match",
    );
}

#[test]
fn content_follows_the_feature() {
    let want = if cfg!(feature = "python-base") {
        base_value()
    } else {
        tuned_value()
    };
    assert_eq!(overlay_active(), !cfg!(feature = "python-base"));
    let dock = content().captain("dockhand").expect("dockhand");
    let want_buy = want["captains"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == "dockhand")
        .unwrap()["pricing"]["buy_price_mult"]
        .as_f64()
        .unwrap();
    assert_eq!(dock.pricing.buy_price_mult, want_buy);
    // Starting silver is never overlaid.
    for captain in &content().captains {
        let base = base_value();
        let silver = base["captains"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["id"] == captain.id.as_str())
            .unwrap()["starting_silver"]
            .as_i64()
            .unwrap();
        assert_eq!(captain.starting_silver, silver, "{}", captain.id);
    }
}
