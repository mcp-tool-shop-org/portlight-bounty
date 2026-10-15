//! The Rust-tuned overlay on the extracted Python content.
//!
//! `data/content.json` is the unmodified extract of the Python game. The tuned
//! overlay (`data/tuned_overlay.json`) lists every deliberate Rust-side
//! deviation as one row: `{path, base, tuned}`. The loader checks each row
//! against a closed allow-list and against the extract, so the overlay cannot
//! drift or reach anything it does not name. Anything else panics.
//!
//! Allowed rows:
//! - `captains[id=<id>].pricing.<field>` for `buy_price_mult`,
//!   `sell_price_mult` and `port_fee_mult`, and
//!   `captains[id=<id>].voyage.provision_burn`, on the captains in
//!   [`TUNED_CAPTAINS`]. Starting silver, the luxury bonus, holds, ships and
//!   inspection are not reachable.
//! - `contracts[id=<template>].captain_bias`, append-only: `tuned` must be
//!   `base` plus new ids from [`BIAS_CAPTAINS`].
//!
//! The `python-base` cargo feature skips the overlay at compile time, so a
//! build with it behaves as the extract does.

use std::collections::BTreeSet;

use serde_json::{Number, Value};

/// The tuned overlay, embedded at compile time.
pub const OVERLAY_JSON: &str = include_str!("../data/tuned_overlay.json");

/// The unmodified extract of the Python game.
pub const BASE_JSON: &str = include_str!("../data/content.json");

/// Captains whose pricing or burn the overlay may change.
pub const TUNED_CAPTAINS: [&str; 5] = [
    "bounty_hunter",
    "dockhand",
    "privateer",
    "scholar",
    "smuggler",
];

/// Captains the overlay may append to a contract template's `captain_bias`.
pub const BIAS_CAPTAINS: [&str; 3] = ["dockhand", "merchant_prince", "privateer"];

const PRICING_FIELDS: [&str; 3] = ["buy_price_mult", "sell_price_mult", "port_fee_mult"];
const VOYAGE_FIELDS: [&str; 1] = ["provision_burn"];

/// Multipliers outside this range are not a balance lever.
const MULT_MIN: f64 = 0.5;
const MULT_MAX: f64 = 1.5;

/// One validated overlay row.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub path: String,
    pub base: Value,
    pub tuned: Value,
}

enum Target<'a> {
    /// `captains[id=..].<section>.<field>`
    Captain {
        id: &'a str,
        section: &'static str,
        field: &'a str,
    },
    /// `contracts[id=..].captain_bias`
    Bias { template: &'a str },
}

fn bracket_id<'a>(segment: &'a str, head: &str) -> Option<&'a str> {
    let id = segment
        .strip_prefix(head)?
        .strip_prefix("[id=")?
        .strip_suffix(']')?;
    let ok = !id.is_empty()
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
    ok.then_some(id)
}

fn parse_path(path: &str) -> Target<'_> {
    let parts: Vec<&str> = path.split('.').collect();
    match parts.as_slice() {
        [head, section, field] => {
            let id = bracket_id(head, "captains")
                .unwrap_or_else(|| panic!("tuned overlay: path not allowed: {path}"));
            assert!(
                TUNED_CAPTAINS.contains(&id),
                "tuned overlay: captain not allowed: {path}"
            );
            let section = match *section {
                "pricing" => {
                    assert!(
                        PRICING_FIELDS.contains(field),
                        "tuned overlay: pricing field not allowed: {path}"
                    );
                    "pricing"
                }
                "voyage" => {
                    assert!(
                        VOYAGE_FIELDS.contains(field),
                        "tuned overlay: voyage field not allowed: {path}"
                    );
                    "voyage"
                }
                _ => panic!("tuned overlay: path not allowed: {path}"),
            };
            Target::Captain { id, section, field }
        }
        [head, field] => {
            let template = bracket_id(head, "contracts")
                .unwrap_or_else(|| panic!("tuned overlay: path not allowed: {path}"));
            assert!(
                *field == "captain_bias",
                "tuned overlay: contract field not allowed: {path}"
            );
            Target::Bias { template }
        }
        _ => panic!("tuned overlay: path not allowed: {path}"),
    }
}

fn find_mut<'a>(base: &'a mut Value, list: &str, id: &str, path: &str) -> &'a mut Value {
    base.get_mut(list)
        .and_then(Value::as_array_mut)
        .and_then(|items| {
            items
                .iter_mut()
                .find(|item| item.get("id").and_then(Value::as_str) == Some(id))
        })
        .unwrap_or_else(|| panic!("tuned overlay: no such id in base content: {path}"))
}

fn number(value: &Value, what: &str, path: &str) -> f64 {
    let n = value
        .as_f64()
        .unwrap_or_else(|| panic!("tuned overlay: {what} is not a number: {path}"));
    assert!(n.is_finite(), "tuned overlay: {what} is not finite: {path}");
    n
}

/// Parses the overlay and checks its shape. Panics on anything outside it.
pub fn rows(overlay: &Value) -> Vec<Row> {
    let object = overlay
        .as_object()
        .expect("tuned overlay: top level must be an object");
    for key in object.keys() {
        assert!(
            key == "note" || key == "rows",
            "tuned overlay: unknown top-level key {key}"
        );
    }
    let list = object
        .get("rows")
        .and_then(Value::as_array)
        .expect("tuned overlay: missing rows");
    let mut seen = BTreeSet::new();
    let mut out = Vec::with_capacity(list.len());
    for row in list {
        let map = row
            .as_object()
            .expect("tuned overlay: row must be an object");
        for key in map.keys() {
            assert!(
                matches!(key.as_str(), "path" | "base" | "tuned"),
                "tuned overlay: unknown row key {key}"
            );
        }
        let path = map
            .get("path")
            .and_then(Value::as_str)
            .expect("tuned overlay: row without path");
        let base = map.get("base").expect("tuned overlay: row without base");
        let tuned = map.get("tuned").expect("tuned overlay: row without tuned");
        assert!(
            seen.insert(path.to_string()),
            "tuned overlay: duplicate row {path}"
        );
        match parse_path(path) {
            Target::Captain { .. } => {
                let b = number(base, "base", path);
                let t = number(tuned, "tuned", path);
                assert!(
                    (MULT_MIN..=MULT_MAX).contains(&t),
                    "tuned overlay: tuned value outside {MULT_MIN}..={MULT_MAX}: {path}"
                );
                assert!(b != t, "tuned overlay: row changes nothing: {path}");
            }
            Target::Bias { .. } => {
                let b = base
                    .as_array()
                    .unwrap_or_else(|| panic!("tuned overlay: base is not a list: {path}"));
                let t = tuned
                    .as_array()
                    .unwrap_or_else(|| panic!("tuned overlay: tuned is not a list: {path}"));
                assert!(
                    t.len() > b.len() && t[..b.len()] == b[..],
                    "tuned overlay: captain_bias is append-only: {path}"
                );
                let mut known: BTreeSet<&str> = BTreeSet::new();
                for item in b {
                    known.insert(item.as_str().unwrap_or_default());
                }
                for item in &t[b.len()..] {
                    let id = item
                        .as_str()
                        .unwrap_or_else(|| panic!("tuned overlay: bias id not a string: {path}"));
                    assert!(
                        BIAS_CAPTAINS.contains(&id),
                        "tuned overlay: bias captain not allowed: {path} {id}"
                    );
                    assert!(
                        known.insert(id),
                        "tuned overlay: bias id already present: {path} {id}"
                    );
                }
            }
        }
        out.push(Row {
            path: path.to_string(),
            base: base.clone(),
            tuned: tuned.clone(),
        });
    }
    out
}

/// Applies the overlay to the parsed extract. Every row's `base` must equal
/// the extract's value; the result differs from the extract only at the rows.
pub fn apply(base: &Value, overlay: &Value) -> Value {
    let mut out = base.clone();
    for row in rows(overlay) {
        let slot: &mut Value = match parse_path(&row.path) {
            Target::Captain { id, section, field } => {
                let captain = find_mut(&mut out, "captains", id, &row.path);
                captain
                    .get_mut(section)
                    .and_then(|s| s.get_mut(field))
                    .unwrap_or_else(|| panic!("tuned overlay: no such field: {}", row.path))
            }
            Target::Bias { template } => {
                let contract = find_mut(&mut out, "contracts", template, &row.path);
                contract
                    .get_mut("captain_bias")
                    .unwrap_or_else(|| panic!("tuned overlay: no such field: {}", row.path))
            }
        };
        if row.base.is_array() {
            assert!(
                *slot == row.base,
                "tuned overlay: base does not match content.json: {}",
                row.path
            );
            *slot = row.tuned.clone();
        } else {
            let current = number(slot, "content value", &row.path);
            assert!(
                current == number(&row.base, "base", &row.path),
                "tuned overlay: base does not match content.json: {}",
                row.path
            );
            let tuned = number(&row.tuned, "tuned", &row.path);
            *slot = Value::Number(Number::from_f64(tuned).expect("finite"));
        }
    }
    out
}

/// The extract as parsed JSON.
pub fn base_value() -> Value {
    serde_json::from_str(BASE_JSON).expect("embedded Portlight content JSON")
}

/// The overlay as parsed JSON.
pub fn overlay_value() -> Value {
    serde_json::from_str(OVERLAY_JSON).expect("embedded tuned overlay JSON")
}

/// The extract with the embedded overlay applied.
pub fn tuned_value() -> Value {
    apply(&base_value(), &overlay_value())
}

/// Paths where two content values differ. A list that differs is reported as
/// one path, so an append-only list is a single leaf.
pub fn diff_paths(a: &Value, b: &Value, path: &str, out: &mut Vec<String>) {
    match (a, b) {
        (Value::Object(x), Value::Object(y)) => {
            for (key, value) in x {
                match y.get(key) {
                    Some(other) => diff_paths(value, other, &format!("{path}.{key}"), out),
                    None => out.push(format!("{path}.{key}")),
                }
            }
            for key in y.keys().filter(|k| !x.contains_key(*k)) {
                out.push(format!("{path}.{key}"));
            }
        }
        (Value::Array(x), Value::Array(y)) if x.len() == y.len() => {
            for (i, (left, right)) in x.iter().zip(y).enumerate() {
                diff_paths(left, right, &format!("{path}[{i}]"), out);
            }
        }
        _ => {
            if a != b {
                out.push(path.to_string());
            }
        }
    }
}
