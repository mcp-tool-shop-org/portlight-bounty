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

const MANIFEST_SHA256: &str = "562bb5e7a5fbb9fc9d5319754ece3ea45b2e9ea4d2a4940b4280cb2c446da8f8";

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
    assert_eq!(manifest["version"], "0.4.1");
    assert_eq!(manifest["count"], 73);
    assert_eq!(manifest["counts"]["ships"], 36);
    assert_eq!(manifest["counts"]["harbour"], 30);
    assert_eq!(manifest["counts"]["chart"], 4);
    assert_eq!(manifest["counts"]["quay_flag"], 3);
    let path_convention = manifest["path_convention"]["entry.path"]
        .as_str()
        .expect("path_convention");
    assert!(path_convention.contains("relative"), "{path_convention}");
    assert!(manifest["path_convention"]["provenance.*"]
        .as_str()
        .expect("provenance convention")
        .contains("relative"));
    assert!(manifest["import_policy"]["rule"]
        .as_str()
        .expect("import_policy")
        .contains("Rev 4 R11"));

    let entries = manifest["entries"].as_array().expect("entries");
    assert_eq!(entries.len(), 73);
    let mut listed = BTreeSet::new();
    let mut class_counts = [0u32; 4];
    let mut harbour = 0u32;
    let mut chart = 0u32;
    let mut quay_flag = 0u32;
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
        let source = entry["provenance"]["source_path"]
            .as_str()
            .unwrap_or_else(|| panic!("{rel} source_path"));
        let sidecar = entry["provenance"]["render_sidecar"]
            .as_str()
            .unwrap_or_else(|| panic!("{rel} render_sidecar"));
        for provenance_path in [source, sidecar] {
            assert!(
                !provenance_path.starts_with('/') && !provenance_path.contains(".."),
                "{rel} provenance is not repo-relative: {provenance_path}"
            );
        }
        let import = &entry["import"];
        assert_eq!(import["fix_alpha_border"], true, "{rel}");
        assert_eq!(import["premult_alpha"], false, "{rel}");
        assert_eq!(import["compress"], "Lossless", "{rel}");
        match entry["group"].as_str() {
            Some("ships") => {
                assert_eq!(import["mipmaps"], false, "{rel}");
                assert!(entry["subgroup"].is_null(), "{rel}");
                match entry["class"].as_str() {
                    Some("ship_sloop") => class_counts[0] += 1,
                    Some("ship_cutter") => class_counts[1] += 1,
                    Some("ship_brigantine") => class_counts[2] += 1,
                    Some("ship_galleon") => class_counts[3] += 1,
                    Some(other) => panic!("unexpected ship class {other}"),
                    None => panic!("{rel} ship has no class"),
                }
            }
            Some("harbour") => {
                harbour += 1;
                assert_eq!(import["mipmaps"], false, "{rel}");
                assert!(entry["class"].is_null(), "{rel}");
                match entry["subgroup"].as_str() {
                    Some("water" | "quay" | "pier" | "pilings") => {}
                    other => panic!("{rel} harbour subgroup {other:?}"),
                }
            }
            Some("chart") => {
                chart += 1;
                match entry["subgroup"].as_str() {
                    Some("water") => assert_eq!(import["mipmaps"], true, "{rel}"),
                    Some("marker") => assert_eq!(import["mipmaps"], false, "{rel}"),
                    other => panic!("{rel} chart subgroup {other:?}"),
                }
            }
            Some("quay_flag") => {
                quay_flag += 1;
                assert_eq!(import["mipmaps"], false, "{rel}");
                assert!(entry["class"].is_null(), "{rel}");
                assert_eq!(entry["subgroup"].as_str(), Some("flag"), "{rel}");
            }
            other => panic!("{rel} unexpected group {other:?}"),
        }
    }
    assert_eq!(class_counts, [9, 9, 9, 9], "four classes, nine plates each");
    assert_eq!(harbour, 30);
    assert_eq!(chart, 4);
    assert_eq!(quay_flag, 3);

    let mut pngs = Vec::new();
    collect_files(&dir, ".png", &mut pngs);
    let on_disk: BTreeSet<String> = pngs.iter().map(|path| rel_to(&dir, path)).collect();
    assert_eq!(on_disk.len(), 73, "committed PNG count");
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

/// asset-spec Rev 4, R11. Each `.import` is checked against that plate's
/// `import` object in MANIFEST.json. Nearest filtering is the project canvas
/// filter, not a per-import key. `size_limit: null` is Godot's `0` (no limit).
#[test]
fn plate_imports_follow_the_manifest_and_hdr_2d_is_off() {
    let godot = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../godot");
    let manifest: Value = serde_json::from_slice(
        &fs::read(landing_dir().join("MANIFEST.json")).expect("MANIFEST.json"),
    )
    .expect("manifest json");
    let entries = manifest["entries"].as_array().expect("entries");
    assert_eq!(entries.len(), 73);

    let mut mipmaps_on = Vec::new();
    for entry in entries {
        let rel = entry["path"].as_str().expect("path");
        let import_path = landing_dir().join(format!("{rel}.import"));
        let text = fs::read_to_string(&import_path)
            .unwrap_or_else(|err| panic!("{}: {err}", import_path.display()));
        let spec = &entry["import"];
        let mipmaps = if spec["mipmaps"].as_bool().expect("mipmaps") {
            mipmaps_on.push(entry["id"].as_str().unwrap().to_string());
            "mipmaps/generate=true"
        } else {
            "mipmaps/generate=false"
        };
        let size_limit = match &spec["size_limit"] {
            Value::Null => 0,
            Value::Number(number) => number.as_i64().expect("size_limit"),
            other => panic!("{rel} size_limit {other}"),
        };
        assert_eq!(spec["compress"], "Lossless", "{rel}");
        assert_eq!(spec["fix_alpha_border"], true, "{rel}");
        assert_eq!(spec["premult_alpha"], false, "{rel}");
        for line in [
            "compress/mode=0",
            mipmaps,
            "process/fix_alpha_border=true",
            "process/premult_alpha=false",
            "process/hdr_as_srgb=false",
        ] {
            assert!(
                text.lines().any(|existing| existing == line),
                "{} missing `{line}`",
                import_path.display()
            );
        }
        assert!(
            text.lines()
                .any(|existing| existing == format!("process/size_limit={size_limit}")),
            "{} size_limit",
            import_path.display()
        );
        assert!(
            text.contains("\"vram_texture\": false"),
            "{} enables vram_texture",
            import_path.display()
        );
    }
    mipmaps_on.sort();
    assert_eq!(
        mipmaps_on,
        ["chart_water_a", "chart_water_b", "chart_water_c"]
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
