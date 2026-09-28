//! Byte check for the approved landing bundle.
//!
//! `godot/assets/landing/MANIFEST.json` is generated. This test fails if a
//! committed PNG drifts from the hash in that file, or if the manifest
//! itself is edited.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

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

fn collect_files(dir: &Path, suffix: &str, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|err| panic!("read {}: {err}", dir.display()));
    let mut names: Vec<PathBuf> = entries
        .map(|entry| entry.expect("dir entry").path())
        .collect();
    names.sort();
    for path in names {
        if path.is_dir() {
            collect_files(&path, suffix, out);
        } else if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(suffix))
        {
            out.push(path);
        }
    }
}

fn rel_to(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or_else(|_| panic!("{} is outside {}", path.display(), root.display()))
        .to_str()
        .expect("utf-8 path")
        .replace('\\', "/")
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
    let mut listed = BTreeSet::new();
    let mut class_counts = [0u32; 4];
    let mut harbour = 0u32;
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
        assert!(
            listed.insert(rel.to_string()),
            "duplicate manifest path {rel}"
        );
        match entry["class"].as_str() {
            Some("ship_sloop") => class_counts[0] += 1,
            Some("ship_cutter") => class_counts[1] += 1,
            Some("ship_brigantine") => class_counts[2] += 1,
            Some("ship_galleon") => class_counts[3] += 1,
            Some(other) => panic!("unexpected ship class {other}"),
            None => {
                harbour += 1;
                assert!(
                    rel.starts_with("ground/")
                        || rel.starts_with("structures/")
                        || rel.starts_with("props/"),
                    "{rel} is not a harbour plate"
                );
            }
        }
    }
    assert_eq!(class_counts, [9, 9, 9, 9], "four classes, nine plates each");
    assert_eq!(harbour, 30);

    let mut pngs = Vec::new();
    collect_files(&dir, ".png", &mut pngs);
    let on_disk: BTreeSet<String> = pngs.iter().map(|path| rel_to(&dir, path)).collect();
    assert_eq!(on_disk.len(), 66, "committed PNG count");
    assert_eq!(on_disk, listed, "committed PNGs and MANIFEST paths differ");
    for png in &pngs {
        let import = PathBuf::from(format!("{}.import", png.display()));
        assert!(import.is_file(), "missing {}", import.display());
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

/// asset-spec Rev 3, R11. Nearest filtering is the project canvas filter, not a
/// per-import key. Fix Alpha Border is on for every plate. Mipmaps are off for
/// ship plates and harbour plates. Approved chart-water plates use mipmaps; the
/// files under `placeholders/chart_water_*.png` are not those plates, so they
/// stay off until the real water lands. When it does, this test should require
/// `mipmaps/generate=true` for those three paths.
const R11_LINES: &[&str] = &[
    "compress/mode=0",
    "process/fix_alpha_border=true",
    "process/premult_alpha=false",
    "process/size_limit=0",
    "process/hdr_as_srgb=false",
];

#[test]
fn plate_imports_follow_r11_and_hdr_2d_is_off() {
    let godot = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../godot");
    let mut landing = Vec::new();
    collect_files(&godot.join("assets/landing"), ".png.import", &mut landing);
    assert_eq!(landing.len(), 66, "one import per landing plate");
    let mut placeholders = Vec::new();
    collect_files(
        &godot.join("assets/placeholders"),
        ".png.import",
        &mut placeholders,
    );
    assert_eq!(placeholders.len(), 4);
    let mut chart_water_placeholders = 0;

    for path in landing.iter().chain(&placeholders) {
        let text =
            fs::read_to_string(path).unwrap_or_else(|err| panic!("{}: {err}", path.display()));
        for line in R11_LINES {
            assert!(
                text.lines().any(|existing| existing == *line),
                "{} missing `{line}`",
                path.display()
            );
        }
        assert!(
            text.contains("\"vram_texture\": false"),
            "{} enables vram_texture",
            path.display()
        );
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        let in_landing = path.components().any(|part| part.as_os_str() == "landing");
        let chart_water_placeholder = !in_landing && name.starts_with("chart_water_");
        let port_marker = !in_landing && name.starts_with("chart_port_marker");
        assert!(
            in_landing || chart_water_placeholder || port_marker,
            "unexpected plate {}",
            path.display()
        );
        // R11: mipmaps off for ship plates and harbour plates (everything under
        // landing/). The port marker is a placeholder and stays off too.
        // Approved chart water turns mipmaps on; these three placeholders are
        // not that art, so they stay off until those plates replace them.
        assert!(
            text.lines()
                .any(|existing| existing == "mipmaps/generate=false"),
            "{} mipmaps",
            path.display()
        );
        if chart_water_placeholder {
            chart_water_placeholders += 1;
        }
    }
    assert_eq!(
        chart_water_placeholders, 3,
        "chart_water_a/b/c stay placeholders with mipmaps off until approved water lands"
    );

    let project = fs::read_to_string(godot.join("project.godot")).expect("project.godot");
    assert!(
        project.lines().any(|line| line == "viewport/hdr_2d=false"),
        "hdr_2d must be set explicitly"
    );
    assert!(
        project
            .lines()
            .any(|line| line == "textures/canvas_textures/default_texture_filter=0"),
        "canvas filter must be Nearest (0)"
    );
}
