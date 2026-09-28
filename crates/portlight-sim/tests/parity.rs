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
