//! Chart placeholder ids. Real plates replace the file; the id stays.
//!
//! First-playable art is chart water, a code-drawn port marker, and the sloop
//! frames plus wake. Harbour water, quay, and pier are not drawn here. Their
//! datum (−48) is recorded in `godot/assets/catalog/locked-ids.csv` so those
//! tiles can drop in later. Ship silhouettes are placeholders: the spec holds
//! the real frames.

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

const PLACEHOLDER: &str = "PLACEHOLDER flat colour; replace this file by id";

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

pub static ASSETS: &[Asset] = &[
    plate(
        CHART_WATER_A,
        "placeholders/chart_water_a.png",
        128,
        64,
        CHART_WATER_ANCHOR.0,
        CHART_WATER_ANCHOR.1,
        AssetFamily::ChartWater,
    ),
    plate(
        CHART_WATER_B,
        "placeholders/chart_water_b.png",
        128,
        64,
        CHART_WATER_ANCHOR.0,
        CHART_WATER_ANCHOR.1,
        AssetFamily::ChartWater,
    ),
    plate(
        CHART_WATER_C,
        "placeholders/chart_water_c.png",
        128,
        64,
        CHART_WATER_ANCHOR.0,
        CHART_WATER_ANCHOR.1,
        AssetFamily::ChartWater,
    ),
    plate(
        PORT_MARKER,
        "placeholders/chart_port_marker.png",
        128,
        128,
        PORT_ANCHOR.0,
        PORT_ANCHOR.1,
        AssetFamily::Port,
    ),
    plate(
        "ship_sloop_f0",
        "placeholders/ship_sloop_f0.png",
        SLOOP_CANVAS.0,
        SLOOP_CANVAS.1,
        SLOOP_ANCHOR.0,
        SLOOP_ANCHOR.1,
        AssetFamily::Ship,
    ),
    plate(
        "ship_sloop_f1",
        "placeholders/ship_sloop_f1.png",
        SLOOP_CANVAS.0,
        SLOOP_CANVAS.1,
        SLOOP_ANCHOR.0,
        SLOOP_ANCHOR.1,
        AssetFamily::Ship,
    ),
    plate(
        "ship_sloop_f2",
        "placeholders/ship_sloop_f2.png",
        SLOOP_CANVAS.0,
        SLOOP_CANVAS.1,
        SLOOP_ANCHOR.0,
        SLOOP_ANCHOR.1,
        AssetFamily::Ship,
    ),
    plate(
        "ship_sloop_f3",
        "placeholders/ship_sloop_f3.png",
        SLOOP_CANVAS.0,
        SLOOP_CANVAS.1,
        SLOOP_ANCHOR.0,
        SLOOP_ANCHOR.1,
        AssetFamily::Ship,
    ),
    plate(
        "ship_sloop_f4",
        "placeholders/ship_sloop_f4.png",
        SLOOP_CANVAS.0,
        SLOOP_CANVAS.1,
        SLOOP_ANCHOR.0,
        SLOOP_ANCHOR.1,
        AssetFamily::Ship,
    ),
    plate(
        "ship_sloop_f5",
        "placeholders/ship_sloop_f5.png",
        SLOOP_CANVAS.0,
        SLOOP_CANVAS.1,
        SLOOP_ANCHOR.0,
        SLOOP_ANCHOR.1,
        AssetFamily::Ship,
    ),
    plate(
        "ship_sloop_f6",
        "placeholders/ship_sloop_f6.png",
        SLOOP_CANVAS.0,
        SLOOP_CANVAS.1,
        SLOOP_ANCHOR.0,
        SLOOP_ANCHOR.1,
        AssetFamily::Ship,
    ),
    plate(
        "ship_sloop_f7",
        "placeholders/ship_sloop_f7.png",
        SLOOP_CANVAS.0,
        SLOOP_CANVAS.1,
        SLOOP_ANCHOR.0,
        SLOOP_ANCHOR.1,
        AssetFamily::Ship,
    ),
    plate(
        SLOOP_WAKE,
        "placeholders/ship_sloop_wake.png",
        SLOOP_CANVAS.0,
        SLOOP_CANVAS.1,
        SLOOP_ANCHOR.0,
        SLOOP_ANCHOR.1,
        AssetFamily::Wake,
    ),
];

pub fn asset(id: &str) -> Option<&'static Asset> {
    ASSETS.iter().find(|asset| asset.id == id)
}

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
    asset(id).expect("sloop facing")
}

pub fn chart_water_id(u: i32, v: i32) -> &'static str {
    match (u.wrapping_mul(3).wrapping_add(v.wrapping_mul(5))).rem_euclid(3) {
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
        let wake = asset(SLOOP_WAKE).unwrap();
        assert_eq!((wake.anchor_x, wake.anchor_y), SLOOP_ANCHOR);
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
}
