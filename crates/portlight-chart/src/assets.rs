//! Chart plate ids. Real art replaces the file; the id stays.
//!
//! Chart water, the port marker, and the ship frames are the approved
//! landing plates (`godot/assets/landing/`, MANIFEST v0.3.0). Ship canvas
//! and anchor come from that manifest. The port marker anchor is the
//! sidecar value recorded there, `(64, 95)`. Harbour water, quay, pier,
//! and pilings are in the same bundle. Their datum (−48) is recorded in
//! `godot/assets/catalog/locked-ids.csv`.

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

use portlight_sim::content;

use crate::project::Facing;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetFamily {
    ChartWater,
    Port,
    Ship,
    Wake,
}

impl AssetFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ChartWater => "chart_water",
            Self::Port => "port",
            Self::Ship => "ship",
            Self::Wake => "wake",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Asset {
    pub id: &'static str,
    pub file: &'static str,
    pub canvas_w: i32,
    pub canvas_h: i32,
    /// Pixel that lands on the placement point.
    pub anchor_x: i32,
    pub anchor_y: i32,
    pub family: AssetFamily,
    pub note: &'static str,
}

impl Asset {
    pub fn res_path(self) -> String {
        format!("res://assets/{}", self.file)
    }
}

pub const PLACEHOLDER: &str = "PLACEHOLDER flat colour; replace this file by id";
const APPROVED: &str = "approved landing plate; do not regenerate";

pub const CHART_WATER_A: &str = "chart_water_a";
pub const CHART_WATER_B: &str = "chart_water_b";
pub const CHART_WATER_C: &str = "chart_water_c";
pub const PORT_MARKER: &str = "chart_port_marker";
pub const SLOOP_WAKE: &str = "ship_sloop_wake";

/// Sloop hull length on screen. The wake ellipse is 0.9 times this.
pub const SLOOP_HULL_PX: i32 = 40;
pub const SLOOP_ANCHOR: (i32, i32) = (32, 50);
pub const SLOOP_CANVAS: (i32, i32) = (64, 64);

/// Port marker anchor: footprint centre of the 128×64 diamond on a 128×128 canvas.
pub const PORT_ANCHOR: (i32, i32) = (64, 95);

/// Chart water anchor: footprint bottom vertex.
pub const CHART_WATER_ANCHOR: (i32, i32) = (64, 63);

const fn plate(
    id: &'static str,
    file: &'static str,
    canvas_w: i32,
    canvas_h: i32,
    anchor_x: i32,
    anchor_y: i32,
    family: AssetFamily,
) -> Asset {
    Asset {
        id,
        file,
        canvas_w,
        canvas_h,
        anchor_x,
        anchor_y,
        family,
        note: PLACEHOLDER,
    }
}

const fn approved(asset: Asset) -> Asset {
    Asset {
        note: APPROVED,
        ..asset
    }
}

pub static ASSETS: &[Asset] = &[
    approved(plate(
        CHART_WATER_A,
        "landing/chart/chart_water_a.png",
        128,
        64,
        CHART_WATER_ANCHOR.0,
        CHART_WATER_ANCHOR.1,
        AssetFamily::ChartWater,
    )),
    approved(plate(
        CHART_WATER_B,
        "landing/chart/chart_water_b.png",
        128,
        64,
        CHART_WATER_ANCHOR.0,
        CHART_WATER_ANCHOR.1,
        AssetFamily::ChartWater,
    )),
    approved(plate(
        CHART_WATER_C,
        "landing/chart/chart_water_c.png",
        128,
        64,
        CHART_WATER_ANCHOR.0,
        CHART_WATER_ANCHOR.1,
        AssetFamily::ChartWater,
    )),
    approved(plate(
        PORT_MARKER,
        "landing/chart/chart_port_marker.png",
        128,
        128,
        PORT_ANCHOR.0,
        PORT_ANCHOR.1,
        AssetFamily::Port,
    )),
    approved(plate(
        "ship_sloop_f0",
        "landing/chart/ships/ship_sloop/ship_sloop_f0.png",
        SLOOP_CANVAS.0,
        SLOOP_CANVAS.1,
        SLOOP_ANCHOR.0,
        SLOOP_ANCHOR.1,
        AssetFamily::Ship,
    )),
    approved(plate(
        "ship_sloop_f1",
        "landing/chart/ships/ship_sloop/ship_sloop_f1.png",
        SLOOP_CANVAS.0,
        SLOOP_CANVAS.1,
        SLOOP_ANCHOR.0,
        SLOOP_ANCHOR.1,
        AssetFamily::Ship,
    )),
    approved(plate(
        "ship_sloop_f2",
        "landing/chart/ships/ship_sloop/ship_sloop_f2.png",
        SLOOP_CANVAS.0,
        SLOOP_CANVAS.1,
        SLOOP_ANCHOR.0,
        SLOOP_ANCHOR.1,
        AssetFamily::Ship,
    )),
    approved(plate(
        "ship_sloop_f3",
        "landing/chart/ships/ship_sloop/ship_sloop_f3.png",
        SLOOP_CANVAS.0,
        SLOOP_CANVAS.1,
        SLOOP_ANCHOR.0,
        SLOOP_ANCHOR.1,
        AssetFamily::Ship,
    )),
    approved(plate(
        "ship_sloop_f4",
        "landing/chart/ships/ship_sloop/ship_sloop_f4.png",
        SLOOP_CANVAS.0,
        SLOOP_CANVAS.1,
        SLOOP_ANCHOR.0,
        SLOOP_ANCHOR.1,
        AssetFamily::Ship,
    )),
    approved(plate(
        "ship_sloop_f5",
        "landing/chart/ships/ship_sloop/ship_sloop_f5.png",
        SLOOP_CANVAS.0,
        SLOOP_CANVAS.1,
        SLOOP_ANCHOR.0,
        SLOOP_ANCHOR.1,
        AssetFamily::Ship,
    )),
    approved(plate(
        "ship_sloop_f6",
        "landing/chart/ships/ship_sloop/ship_sloop_f6.png",
        SLOOP_CANVAS.0,
        SLOOP_CANVAS.1,
        SLOOP_ANCHOR.0,
        SLOOP_ANCHOR.1,
        AssetFamily::Ship,
    )),
    approved(plate(
        "ship_sloop_f7",
        "landing/chart/ships/ship_sloop/ship_sloop_f7.png",
        SLOOP_CANVAS.0,
        SLOOP_CANVAS.1,
        SLOOP_ANCHOR.0,
        SLOOP_ANCHOR.1,
        AssetFamily::Ship,
    )),
    approved(plate(
        SLOOP_WAKE,
        "landing/chart/ships/ship_sloop/ship_sloop_wake.png",
        SLOOP_CANVAS.0,
        SLOOP_CANVAS.1,
        SLOOP_ANCHOR.0,
        SLOOP_ANCHOR.1,
        AssetFamily::Wake,
    )),
];

/// Landing manifest embedded so ship canvas and anchor are not copied
/// into constants. The sloop rows in [`ASSETS`] stay as they are for the
/// placeholder catalog; drawing reads this file.
const LANDING_MANIFEST: &str = include_str!("../../../godot/assets/landing/MANIFEST.json");

pub fn asset(id: &str) -> Option<&'static Asset> {
    ASSETS
        .iter()
        .chain(manifest_ship_plates().iter())
        .find(|asset| asset.id == id)
}

/// Sloop plate from the static catalog. Placeholder generation still uses this.
pub fn ship_asset(facing: Facing) -> &'static Asset {
    let id = match facing {
        Facing::F0 => "ship_sloop_f0",
        Facing::F1 => "ship_sloop_f1",
        Facing::F2 => "ship_sloop_f2",
        Facing::F3 => "ship_sloop_f3",
        Facing::F4 => "ship_sloop_f4",
        Facing::F5 => "ship_sloop_f5",
        Facing::F6 => "ship_sloop_f6",
        Facing::F7 => "ship_sloop_f7",
    };
    ASSETS
        .iter()
        .find(|asset| asset.id == id)
        .expect("sloop facing")
}

/// Hull and wake for one template. Canvas and anchor are the manifest
/// values for the plate that is drawn. `class_name` is the catalog class,
/// including `man_of_war` when the plate is a galleon.
#[derive(Debug, Clone, Copy)]
pub struct ShipDraw {
    pub hull: &'static Asset,
    pub wake: &'static Asset,
    pub class_name: &'static str,
}

pub fn ship_draw(template_id: &str, facing: Facing) -> ShipDraw {
    let Some(template) = content::content().ship(template_id) else {
        log_plate_note_once(&format!(
            "warning: unknown ship class {template_id} drawn with sloop plates"
        ));
        return sloop_draw(template_id, facing);
    };
    if template.ship_class == "man_of_war" {
        log_plate_note_once(&format!(
            "placeholder: {template_id} drawn with galleon plates"
        ));
    }
    // Catalog class stays on the draw. Plate choice is separate.
    plates_for_class(&template.ship_class, facing)
}

fn sloop_draw(class_name: &str, facing: Facing) -> ShipDraw {
    ShipDraw {
        hull: manifest_plate("sloop", facing.asset_suffix()),
        wake: manifest_plate("sloop", "wake"),
        class_name: static_class(class_name),
    }
}

/// Plates for a class id. `man_of_war` uses the galleon plates at the galleon
/// anchor. Any other unknown id uses the sloop and logs a warning once.
pub fn plates_for_class(class_id: &str, facing: Facing) -> ShipDraw {
    match class_id {
        "sloop" | "cutter" | "brigantine" | "galleon" => ShipDraw {
            hull: manifest_plate(class_id, facing.asset_suffix()),
            wake: manifest_plate(class_id, "wake"),
            class_name: static_class(class_id),
        },
        "man_of_war" => ShipDraw {
            hull: manifest_plate("galleon", facing.asset_suffix()),
            wake: manifest_plate("galleon", "wake"),
            class_name: "man_of_war",
        },
        other => {
            log_plate_note_once(&format!(
                "warning: unknown ship class {other} drawn with sloop plates"
            ));
            sloop_draw(other, facing)
        }
    }
}

fn static_class(class: &str) -> &'static str {
    match class {
        "sloop" => "sloop",
        "cutter" => "cutter",
        "brigantine" => "brigantine",
        "galleon" => "galleon",
        "man_of_war" => "man_of_war",
        other => leak_once(other),
    }
}

fn leak_once(text: &str) -> &'static str {
    static KEPT: Mutex<BTreeMap<String, &'static str>> = Mutex::new(BTreeMap::new());
    let mut kept = KEPT.lock().expect("class names");
    if let Some(existing) = kept.get(text) {
        return existing;
    }
    let leaked: &'static str = Box::leak(text.to_string().into_boxed_str());
    kept.insert(text.to_string(), leaked);
    leaked
}

fn manifest_plate(class: &str, frame: &str) -> &'static Asset {
    let id = format!("ship_{class}_{frame}");
    manifest_ship_plates()
        .iter()
        .find(|plate| plate.id == id)
        .unwrap_or_else(|| panic!("MANIFEST has no ship plate {id}"))
}

fn manifest_ship_plates() -> &'static [Asset] {
    static PLATES: OnceLock<Vec<Asset>> = OnceLock::new();
    PLATES.get_or_init(load_manifest_ships).as_slice()
}

fn load_manifest_ships() -> Vec<Asset> {
    let manifest: serde_json::Value =
        serde_json::from_str(LANDING_MANIFEST).expect("embedded landing MANIFEST");
    let entries = manifest["entries"].as_array().expect("MANIFEST entries");
    let mut plates = Vec::new();
    for entry in entries {
        if entry["group"] != "ships" {
            continue;
        }
        let id = entry["id"].as_str().expect("ship id");
        let rel = entry["path"].as_str().expect("ship path");
        let canvas = entry["canvas"].as_array().expect("canvas");
        let anchor = entry["anchor"].as_array().expect("anchor");
        let frame = entry["frame"].as_str().unwrap_or("");
        let family = if frame == "wake" {
            AssetFamily::Wake
        } else {
            AssetFamily::Ship
        };
        plates.push(Asset {
            id: leak_str(id.to_string()),
            file: leak_str(format!("landing/{rel}")),
            canvas_w: json_i32(&canvas[0], id, "canvas w"),
            canvas_h: json_i32(&canvas[1], id, "canvas h"),
            anchor_x: json_i32(&anchor[0], id, "anchor x"),
            anchor_y: json_i32(&anchor[1], id, "anchor y"),
            family,
            note: APPROVED,
        });
    }
    plates
}

fn json_i32(value: &serde_json::Value, id: &str, field: &str) -> i32 {
    value
        .as_i64()
        .unwrap_or_else(|| panic!("{id} {field}"))
        .try_into()
        .unwrap_or_else(|_| panic!("{id} {field}"))
}

fn leak_str(text: String) -> &'static str {
    Box::leak(text.into_boxed_str())
}

static PLATE_NOTE_LOGS: Mutex<BTreeMap<String, u32>> = Mutex::new(BTreeMap::new());

fn log_plate_note_once(line: &str) {
    let mut logs = PLATE_NOTE_LOGS.lock().expect("plate notes");
    let count = logs.entry(line.to_string()).or_insert(0);
    if *count == 0 {
        eprintln!("{line}");
    }
    *count = 1;
}

#[cfg(test)]
fn plate_note_emissions(line: &str) -> u32 {
    PLATE_NOTE_LOGS
        .lock()
        .expect("plate notes")
        .get(line)
        .copied()
        .unwrap_or(0)
}

/// Deterministic water-variant index in `0..3` for one cell.
///
/// The same `(col, row)` always selects the same plate. The mix is not
/// `(col + row) % 3`.
pub fn water_variant(col: i32, row: i32) -> u8 {
    let mut hash = (col as u32).wrapping_mul(0x9E37_79B1);
    hash ^= (row as u32).wrapping_mul(0x85EB_CA6B);
    hash ^= hash >> 16;
    hash = hash.wrapping_mul(0xC2B2_AE35);
    hash ^= hash >> 16;
    (hash % 3) as u8
}

pub fn chart_water_id(u: i32, v: i32) -> &'static str {
    match water_variant(u, v) {
        0 => CHART_WATER_A,
        1 => CHART_WATER_B,
        _ => CHART_WATER_C,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chart_placeholders_match_the_locked_canvases() {
        let water = asset(CHART_WATER_A).unwrap();
        assert_eq!((water.canvas_w, water.canvas_h), (128, 64));
        assert_eq!((water.anchor_x, water.anchor_y), CHART_WATER_ANCHOR);
        let marker = asset(PORT_MARKER).unwrap();
        assert_eq!((marker.canvas_w, marker.canvas_h), (128, 128));
        assert_eq!((marker.anchor_x, marker.anchor_y), PORT_ANCHOR);
        let ship = ship_asset(Facing::F0);
        assert_eq!((ship.canvas_w, ship.canvas_h), SLOOP_CANVAS);
        assert_eq!((ship.anchor_x, ship.anchor_y), SLOOP_ANCHOR);
        assert!(ship.file.starts_with("landing/"));
        assert_ne!(ship.note, PLACEHOLDER);
        assert_eq!(water.note, APPROVED);
        assert!(water.file.starts_with("landing/chart/"));
        assert!(marker.file.starts_with("landing/chart/"));
        let wake = asset(SLOOP_WAKE).unwrap();
        assert_eq!((wake.anchor_x, wake.anchor_y), SLOOP_ANCHOR);
        assert!(wake.file.ends_with("ship_sloop_wake.png"));
    }

    #[test]
    fn ids_are_unique_and_ships_cover_eight_facings() {
        let mut ids = std::collections::BTreeSet::new();
        for asset in ASSETS {
            assert!(ids.insert(asset.id), "duplicate {}", asset.id);
        }
        for index in 0..8 {
            let facing = Facing::from_index(index);
            assert_eq!(ship_asset(facing).family, AssetFamily::Ship);
            assert!(ship_asset(facing).id.ends_with(facing.asset_suffix()));
        }
    }

    #[test]
    fn locked_catalog_keeps_harbour_datum_and_chart_ids() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../godot/assets/catalog/locked-ids.csv");
        let text = std::fs::read_to_string(path).expect("locked-ids.csv");
        assert!(text.contains("chart_water_a"));
        assert!(text.contains("chart_port_marker"));
        assert!(text.contains("ship_sloop_f0..f7"));
        assert!(text.contains("\"water_a\""));
        assert!(text.contains("\"0,48\""));
        assert!(text.contains("quay_1111"));
        assert!(text.contains("pier_pilings_1x1"));
        assert!(!text.to_ascii_lowercase().contains("character"));
    }

    #[test]
    fn every_template_picks_its_manifest_plate_and_anchor() {
        // Spec numbers, checked against the embedded MANIFEST below.
        // royal_man_of_war has no plate; it draws the galleon at the galleon anchor.
        let expected = [
            ("coastal_sloop", "sloop", "ship_sloop", 64, 64, 32, 50),
            ("swift_cutter", "cutter", "ship_cutter", 72, 80, 36, 64),
            (
                "trade_brigantine",
                "brigantine",
                "ship_brigantine",
                96,
                96,
                48,
                76,
            ),
            (
                "merchant_galleon",
                "galleon",
                "ship_galleon",
                112,
                112,
                56,
                88,
            ),
            (
                "royal_man_of_war",
                "man_of_war",
                "ship_galleon",
                112,
                112,
                56,
                88,
            ),
        ];
        let catalog = content::content();
        assert_eq!(catalog.ships.len(), expected.len());
        for (template_id, class_name, plate_class, canvas_w, canvas_h, anchor_x, anchor_y) in
            expected
        {
            let template = catalog.ship(template_id).expect(template_id);
            assert_eq!(template.ship_class, class_name);
            let drawn = ship_draw(template_id, Facing::F3);
            assert_eq!(drawn.class_name, class_name, "{template_id}");
            assert_eq!(drawn.hull.id, format!("{plate_class}_f3"));
            assert_eq!(drawn.wake.id, format!("{plate_class}_wake"));
            assert_eq!(
                (
                    drawn.hull.canvas_w,
                    drawn.hull.canvas_h,
                    drawn.hull.anchor_x,
                    drawn.hull.anchor_y
                ),
                (canvas_w, canvas_h, anchor_x, anchor_y),
                "{template_id}"
            );
            assert_eq!(
                (drawn.wake.anchor_x, drawn.wake.anchor_y),
                (anchor_x, anchor_y)
            );
            let from_manifest = manifest_geom(drawn.hull.id);
            assert_eq!(
                from_manifest,
                (canvas_w, canvas_h, anchor_x, anchor_y),
                "MANIFEST disagrees with the plate spec for {}",
                drawn.hull.id
            );
            assert_eq!(
                (
                    drawn.hull.canvas_w,
                    drawn.hull.canvas_h,
                    drawn.hull.anchor_x,
                    drawn.hull.anchor_y
                ),
                from_manifest
            );
        }
    }

    #[test]
    fn man_of_war_fallback_is_logged_once_and_keeps_the_class_name() {
        let line = "placeholder: royal_man_of_war drawn with galleon plates";
        let first = ship_draw("royal_man_of_war", Facing::F7);
        let second = ship_draw("royal_man_of_war", Facing::F0);
        assert_eq!(first.class_name, "man_of_war");
        assert_eq!(first.hull.id, "ship_galleon_f7");
        assert_eq!(second.hull.id, "ship_galleon_f0");
        assert_eq!((first.hull.anchor_x, first.hull.anchor_y), (56, 88));
        assert_eq!(plate_note_emissions(line), 1);
        let by_class = plates_for_class("man_of_war", Facing::F4);
        assert_eq!(by_class.class_name, "man_of_war");
        assert_eq!(by_class.hull.id, "ship_galleon_f4");
        assert_eq!((by_class.hull.anchor_x, by_class.hull.anchor_y), (56, 88));
    }

    #[test]
    fn unknown_class_id_falls_back_to_the_sloop() {
        let line = "warning: unknown ship class not_a_ship drawn with sloop plates";
        let drawn = ship_draw("not_a_ship", Facing::F4);
        assert_eq!(drawn.class_name, "not_a_ship");
        assert_eq!(drawn.hull.id, "ship_sloop_f4");
        assert_eq!(drawn.wake.id, "ship_sloop_wake");
        assert_eq!((drawn.hull.anchor_x, drawn.hull.anchor_y), (32, 50));
        assert_eq!((drawn.hull.canvas_w, drawn.hull.canvas_h), (64, 64));
        let again = ship_draw("not_a_ship", Facing::F4);
        assert_eq!(again.hull.id, drawn.hull.id);
        assert_eq!(plate_note_emissions(line), 1);

        let class_line = "warning: unknown ship class skiff drawn with sloop plates";
        let by_class = plates_for_class("skiff", Facing::F2);
        assert_eq!(by_class.class_name, "skiff");
        assert_eq!(by_class.hull.id, "ship_sloop_f2");
        assert_eq!((by_class.hull.anchor_x, by_class.hull.anchor_y), (32, 50));
        let _ = plates_for_class("skiff", Facing::F6);
        assert_eq!(plate_note_emissions(class_line), 1);
    }

    #[test]
    fn water_variant_is_stable_and_uses_every_variant() {
        let mut seen = [false; 3];
        let mut differs_from_stripe = false;
        for row in -6..10 {
            for col in -6..10 {
                let variant = water_variant(col, row);
                assert_eq!(water_variant(col, row), variant);
                assert!(variant < 3);
                seen[variant as usize] = true;
                let stripe = (col + row).rem_euclid(3) as u8;
                if variant != stripe {
                    differs_from_stripe = true;
                }
                assert_eq!(
                    chart_water_id(col, row),
                    match variant {
                        0 => CHART_WATER_A,
                        1 => CHART_WATER_B,
                        _ => CHART_WATER_C,
                    }
                );
            }
        }
        assert_eq!(seen, [true, true, true]);
        assert!(
            differs_from_stripe,
            "water hash still matches (col + row) % 3"
        );
    }

    fn manifest_geom(id: &str) -> (i32, i32, i32, i32) {
        let manifest: serde_json::Value = serde_json::from_str(LANDING_MANIFEST).expect("manifest");
        let entry = manifest["entries"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["id"] == id)
            .unwrap_or_else(|| panic!("no {id}"));
        (
            entry["canvas"][0].as_i64().unwrap() as i32,
            entry["canvas"][1].as_i64().unwrap() as i32,
            entry["anchor"][0].as_i64().unwrap() as i32,
            entry["anchor"][1].as_i64().unwrap() as i32,
        )
    }
}
