//! Stable art ids. Final tiles replace the file at the same id.
//!
//! Sea, water, quay, and pier share the water datum (`datum_y = -48`).
//! The ship glyphs use the same canvas sit-point so the keel rests on the
//! cell anchor; they are not water-family tiles.

use crate::project::{Facing, CELL_WIDTH, SIT_X, SIT_Y, WATER_DATUM_Y};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetFamily {
    Sea,
    Water,
    Quay,
    Pier,
    Lane,
    Port,
    Ship,
}

impl AssetFamily {
    pub fn is_water_datum(self) -> bool {
        matches!(self, Self::Sea | Self::Water | Self::Quay | Self::Pier)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sea => "sea",
            Self::Water => "water",
            Self::Quay => "quay",
            Self::Pier => "pier",
            Self::Lane => "lane",
            Self::Port => "port",
            Self::Ship => "ship",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Asset {
    pub id: &'static str,
    pub file: &'static str,
    pub canvas_w: i32,
    pub canvas_h: i32,
    pub sit_x: i32,
    pub sit_y: i32,
    /// Screen Y of the sprite origin relative to the cell anchor.
    pub datum_y: i32,
    pub family: AssetFamily,
    pub note: &'static str,
}

impl Asset {
    pub fn res_path(self) -> String {
        format!("res://assets/{}", self.file)
    }
}

const PLACEHOLDER: &str = "PLACEHOLDER flat colour; replace this file by id";

/// Placeholder canvas. The 128×64 diamond is centered on the sit-point,
/// with transparent padding so the bottom vertex is inside the image.
const CANVAS_H: i32 = 96;

const fn water(id: &'static str, file: &'static str, family: AssetFamily) -> Asset {
    Asset {
        id,
        file,
        canvas_w: CELL_WIDTH,
        canvas_h: CANVAS_H,
        sit_x: SIT_X,
        sit_y: SIT_Y,
        datum_y: WATER_DATUM_Y,
        family,
        note: PLACEHOLDER,
    }
}

const fn mark(id: &'static str, file: &'static str, family: AssetFamily) -> Asset {
    Asset {
        id,
        file,
        canvas_w: CELL_WIDTH,
        canvas_h: CANVAS_H,
        sit_x: SIT_X,
        sit_y: SIT_Y,
        datum_y: -SIT_Y,
        family,
        note: PLACEHOLDER,
    }
}

pub const TILE_SEA: &str = "chart.tile.sea";
pub const TILE_WATER: &str = "chart.tile.water";
pub const TILE_QUAY: &str = "chart.tile.quay";
pub const TILE_PIER: &str = "chart.tile.pier";
pub const LANE_OPEN: &str = "chart.lane.open";
pub const LANE_WARNING: &str = "chart.lane.warning";
pub const LANE_BLOCKED: &str = "chart.lane.blocked";
pub const LANE_UNDERWAY: &str = "chart.lane.underway";
pub const PORT_MARKER: &str = "chart.port.marker";

pub static ASSETS: &[Asset] = &[
    water(TILE_SEA, "placeholders/tile_sea.png", AssetFamily::Sea),
    water(
        TILE_WATER,
        "placeholders/tile_water.png",
        AssetFamily::Water,
    ),
    water(TILE_QUAY, "placeholders/tile_quay.png", AssetFamily::Quay),
    water(TILE_PIER, "placeholders/tile_pier.png", AssetFamily::Pier),
    mark(LANE_OPEN, "placeholders/lane_open.png", AssetFamily::Lane),
    mark(
        LANE_WARNING,
        "placeholders/lane_warning.png",
        AssetFamily::Lane,
    ),
    mark(
        LANE_BLOCKED,
        "placeholders/lane_blocked.png",
        AssetFamily::Lane,
    ),
    mark(
        LANE_UNDERWAY,
        "placeholders/lane_underway.png",
        AssetFamily::Lane,
    ),
    mark(
        PORT_MARKER,
        "placeholders/port_marker.png",
        AssetFamily::Port,
    ),
    mark(
        "chart.ship.sloop.e",
        "placeholders/ship_sloop_e.png",
        AssetFamily::Ship,
    ),
    mark(
        "chart.ship.sloop.se",
        "placeholders/ship_sloop_se.png",
        AssetFamily::Ship,
    ),
    mark(
        "chart.ship.sloop.s",
        "placeholders/ship_sloop_s.png",
        AssetFamily::Ship,
    ),
    mark(
        "chart.ship.sloop.sw",
        "placeholders/ship_sloop_sw.png",
        AssetFamily::Ship,
    ),
    mark(
        "chart.ship.sloop.w",
        "placeholders/ship_sloop_w.png",
        AssetFamily::Ship,
    ),
    mark(
        "chart.ship.sloop.nw",
        "placeholders/ship_sloop_nw.png",
        AssetFamily::Ship,
    ),
    mark(
        "chart.ship.sloop.n",
        "placeholders/ship_sloop_n.png",
        AssetFamily::Ship,
    ),
    mark(
        "chart.ship.sloop.ne",
        "placeholders/ship_sloop_ne.png",
        AssetFamily::Ship,
    ),
];

pub fn asset(id: &str) -> Option<&'static Asset> {
    ASSETS.iter().find(|asset| asset.id == id)
}

pub fn ship_asset(facing: Facing) -> &'static Asset {
    let id = match facing {
        Facing::E => "chart.ship.sloop.e",
        Facing::Se => "chart.ship.sloop.se",
        Facing::S => "chart.ship.sloop.s",
        Facing::Sw => "chart.ship.sloop.sw",
        Facing::W => "chart.ship.sloop.w",
        Facing::Nw => "chart.ship.sloop.nw",
        Facing::N => "chart.ship.sloop.n",
        Facing::Ne => "chart.ship.sloop.ne",
    };
    asset(id).expect("ship facing asset")
}

pub fn lane_asset(suitability: portlight_sim::LaneSuitability) -> &'static str {
    use portlight_sim::LaneSuitability::{Blocked, Ok, Warning};
    match suitability {
        Ok => LANE_OPEN,
        Warning => LANE_WARNING,
        Blocked => LANE_BLOCKED,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn water_family_sits_at_minus_48() {
        let mut families = 0;
        for asset in ASSETS {
            if asset.family.is_water_datum() {
                families += 1;
                assert_eq!(asset.datum_y, -48, "{}", asset.id);
                assert_eq!(asset.sit_y, 48, "{}", asset.id);
                assert_eq!(asset.canvas_w, 128, "{}", asset.id);
            }
        }
        assert_eq!(families, 4);
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
        }
    }
}
