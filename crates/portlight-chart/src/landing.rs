//! Byte check for the approved landing bundle.
//!
//! `godot/assets/landing/MANIFEST.json` is generated. This test fails if a
//! committed PNG drifts from the hash in that file, or if the manifest
//! itself is edited.

use std::fs;
use std::path::Path;

use serde_json::Value;
use sha2::{Digest, Sha256};

const MANIFEST_SHA256: &str = "acfacd1d507d4a77c703a6e0b8578d40b746a8c6d944e52eac8ea5469424e7a2";

fn landing_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../godot/assets/landing")
}

fn sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

#[test]
fn manifest_hashes_match_the_committed_files() {
    let dir = landing_dir();
    let manifest_bytes = fs::read(dir.join("MANIFEST.json")).expect("MANIFEST.json");
    assert_eq!(sha256(&manifest_bytes), MANIFEST_SHA256);

    let manifest: Value = serde_json::from_slice(&manifest_bytes).expect("manifest json");
    assert_eq!(manifest["version"], "0.2.0");
    assert_eq!(manifest["count"], 66);
    assert_eq!(manifest["counts"]["ships"], 36);
    assert_eq!(manifest["counts"]["harbour"], 30);

    let entries = manifest["entries"].as_array().expect("entries");
    assert_eq!(entries.len(), 66);
    for entry in entries {
        let rel = entry["path"].as_str().expect("path");
        assert!(
            !rel.contains("..") && !rel.starts_with('/'),
            "manifest path escapes the bundle: {rel}"
        );
        let bytes = fs::read(dir.join(rel)).unwrap_or_else(|_| panic!("missing {rel}"));
        assert_eq!(
            bytes.len(),
            entry["bytes"].as_u64().expect("bytes") as usize,
            "{rel}"
        );
        assert_eq!(
            sha256(&bytes),
            entry["sha256"].as_str().expect("sha256"),
            "{rel}"
        );
        if entry["id"] == "ship_sloop_f0" || entry["id"] == "ship_sloop_wake" {
            assert_eq!(entry["canvas"], serde_json::json!([64, 64]));
            assert_eq!(entry["anchor"], serde_json::json!([32, 50]));
            assert_eq!(entry["LOA"]["px"], 40);
        }
    }

    for asset in crate::assets::ASSETS {
        if asset.note == crate::assets::PLACEHOLDER {
            assert!(asset.file.starts_with("placeholders/"), "{}", asset.id);
            continue;
        }
        let rel = asset
            .file
            .strip_prefix("landing/")
            .unwrap_or_else(|| panic!("{} is not under landing/", asset.id));
        let entry = entries
            .iter()
            .find(|entry| entry["path"] == rel)
            .unwrap_or_else(|| panic!("no manifest entry for {}", asset.id));
        assert_eq!(entry["canvas"][0], asset.canvas_w);
        assert_eq!(entry["canvas"][1], asset.canvas_h);
        assert_eq!(entry["anchor"][0], asset.anchor_x);
        assert_eq!(entry["anchor"][1], asset.anchor_y);
    }
}
