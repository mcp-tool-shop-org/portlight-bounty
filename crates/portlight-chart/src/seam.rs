//! Harbour seam plate. Not a harbour layout and not the first playable.
//!
//! Every tile is placed by [`grid_to_screen`](crate::grid_to_screen) for a
//! 256×128 cell, then the footprint bottom, then the −48 sea datum. Water is
//! a 3×3 block so four cells meet at an interior vertex. The pier plate is
//! the exact corner set facing the quay, and that cell is water.

use crate::project::{grid_to_screen, HARBOUR_CELL_H, HARBOUR_CELL_W, WATER_DATUM_Y};

/// One corner of a harbour diamond, named in screen space.
///
/// `+col` is screen down-right and `+row` is screen down-left, so the corner
/// toward a neighbour is UL (−col), UR (−row), DR (+col), or DL (+row).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DiamondCorner {
    Ul,
    Ur,
    Dr,
    Dl,
}

impl DiamondCorner {
    fn token(self) -> &'static str {
        match self {
            Self::Ul => "UL",
            Self::Ur => "UR",
            Self::Dr => "DR",
            Self::Dl => "DL",
        }
    }
}

/// Plate id for exactly this corner set, in UL, UR, DR, DL order.
pub fn pier_plate_id(corners: &[DiamondCorner]) -> String {
    let mut id = String::from("pier");
    for corner in [
        DiamondCorner::Ul,
        DiamondCorner::Ur,
        DiamondCorner::Dr,
        DiamondCorner::Dl,
    ] {
        if corners.contains(&corner) {
            id.push('_');
            id.push_str(corner.token());
        }
    }
    id
}

/// Corner of `(from_col, from_row)` that faces an edge-adjacent cell.
pub fn corner_toward(from_col: i32, from_row: i32, to_col: i32, to_row: i32) -> DiamondCorner {
    match (to_col - from_col, to_row - from_row) {
        (-1, 0) => DiamondCorner::Ul,
        (0, -1) => DiamondCorner::Ur,
        (1, 0) => DiamondCorner::Dr,
        (0, 1) => DiamondCorner::Dl,
        _ => panic!("corner_toward expects an edge neighbour"),
    }
}

/// Anchor pixel of a harbour cell: grid centre, half a cell down to the
/// footprint bottom, then the sea datum. Godot Y grows down, so datum −48
/// is a `+48` screen shift. Chart water does not use this.
pub fn harbour_anchor(col: i32, row: i32) -> (i32, i32) {
    let (sx, sy) = grid_to_screen(col, row, HARBOUR_CELL_W, HARBOUR_CELL_H);
    (sx, sy + HARBOUR_CELL_H / 2 - WATER_DATUM_Y)
}

/// Bottom tip of cell `(1, 1)`. Cells `(1,1)`, `(2,1)`, `(1,2)`, and `(2,2)`
/// meet there. Those four are water, so the vertex is an interior seam.
pub fn seam_interior_vertex() -> (i32, i32) {
    harbour_anchor(1, 1)
}

/// One placed plate. `screen` is where the texture anchor lands.
#[derive(Clone, Copy, Debug)]
pub struct SeamTile {
    pub col: i32,
    pub row: i32,
    pub path: &'static str,
    pub anchor_x: i32,
    pub anchor_y: i32,
    pub canvas_w: i32,
    pub canvas_h: i32,
    pub z: i32,
    pub screen_x: i32,
    pub screen_y: i32,
}

/// Centre of the sprite bounds, so a zoomed camera frames the whole plate.
pub fn seam_camera_center() -> (i32, i32) {
    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;
    for tile in harbour_seam() {
        let left = tile.screen_x - tile.anchor_x;
        let top = tile.screen_y - tile.anchor_y;
        min_x = min_x.min(left);
        min_y = min_y.min(top);
        max_x = max_x.max(left + tile.canvas_w);
        max_y = max_y.max(top + tile.canvas_h);
    }
    ((min_x + max_x) / 2, (min_y + max_y) / 2)
}

/// 3×3 water, `quay_1111` at `(0, 1)`, pier root and pilings at `(0, 2)`.
pub fn harbour_seam() -> Vec<SeamTile> {
    let mut tiles = Vec::with_capacity(12);
    for row in 0..3 {
        for col in 0..3 {
            tiles.push(water(col, row));
        }
    }
    // The pier root is the water cell beside the quay, not the quay cell.
    // Only the corner that faces the quay is set.
    let quay = (0, 1);
    let pier = (0, 2);
    let corner = corner_toward(pier.0, pier.1, quay.0, quay.1);
    tiles.push(works(
        pier.0,
        pier.1,
        "res://assets/landing/props/pier_pilings_1x1/beauty.png",
        1,
    ));
    tiles.push(works(
        pier.0,
        pier.1,
        pier_path(&pier_plate_id(&[corner])),
        2,
    ));
    tiles.push(works(
        quay.0,
        quay.1,
        "res://assets/landing/structures/quay_1111/beauty.png",
        3,
    ));
    tiles
}

fn water(col: i32, row: i32) -> SeamTile {
    let path = match (col + row).rem_euclid(3) {
        0 => "res://assets/landing/ground/water_a.png",
        1 => "res://assets/landing/ground/water_b.png",
        _ => "res://assets/landing/ground/water_c.png",
    };
    let (screen_x, screen_y) = harbour_anchor(col, row);
    SeamTile {
        col,
        row,
        path,
        anchor_x: 128,
        anchor_y: 127,
        canvas_w: 256,
        canvas_h: 128,
        z: 0,
        screen_x,
        screen_y,
    }
}

fn works(col: i32, row: i32, path: &'static str, z: i32) -> SeamTile {
    let (screen_x, screen_y) = harbour_anchor(col, row);
    SeamTile {
        col,
        row,
        path,
        anchor_x: 128,
        anchor_y: 255,
        canvas_w: 256,
        canvas_h: 256,
        z,
        screen_x,
        screen_y,
    }
}

fn pier_path(id: &str) -> &'static str {
    match id {
        "pier_UL" => "res://assets/landing/structures/pier_UL/beauty.png",
        "pier_UR" => "res://assets/landing/structures/pier_UR/beauty.png",
        "pier_DR" => "res://assets/landing/structures/pier_DR/beauty.png",
        "pier_DL" => "res://assets/landing/structures/pier_DL/beauty.png",
        _ => panic!("strict corner set has no single-corner plate {id}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::{grid_to_screen, water_cell_center, CELL_HEIGHT, CELL_WIDTH};

    #[test]
    fn seam_scene_uses_the_shared_projection() {
        let placement = include_str!("seam.rs");
        let scene = include_str!("../../portlight-godot/src/seam.rs");
        assert!(
            placement.contains("grid_to_screen("),
            "harbour placement must call grid_to_screen"
        );
        assert!(
            scene.contains("harbour_seam("),
            "the seam scene must place the chart crate's tiles"
        );
        assert!(
            !scene.contains("STEP_U"),
            "the seam scene must not hand-place a step"
        );
        assert!(!scene.contains("(col - row) * 128"));

        for tile in harbour_seam() {
            let (sx, sy) = grid_to_screen(tile.col, tile.row, HARBOUR_CELL_W, HARBOUR_CELL_H);
            assert_eq!(
                sx,
                (tile.col - tile.row) * 128,
                "col {} row {}",
                tile.col,
                tile.row
            );
            assert_eq!(sy, (tile.col + tile.row) * 64);
            assert_eq!(tile.screen_x, sx);
            assert_eq!(
                tile.screen_y,
                sy + HARBOUR_CELL_H / 2 - WATER_DATUM_Y,
                "datum applies to {}",
                tile.path
            );
        }

        let (cx, cy) = grid_to_screen(4, 2, CELL_WIDTH, CELL_HEIGHT);
        assert_eq!(water_cell_center(4, 2), (cx as f32, cy as f32));
        assert_eq!(cx, (4 - 2) * 64);
        assert_eq!(cy, (4 + 2) * 32);
    }

    #[test]
    fn seam_is_a_3x3_with_a_water_pier_root() {
        let tiles = harbour_seam();
        let water: Vec<_> = tiles.iter().filter(|tile| tile.z == 0).collect();
        assert_eq!(water.len(), 9);
        for row in 0..3 {
            for col in 0..3 {
                assert!(
                    water.iter().any(|tile| tile.col == col && tile.row == row),
                    "missing water {col},{row}"
                );
            }
        }
        let quay = tiles
            .iter()
            .find(|tile| tile.path.contains("quay_1111"))
            .expect("quay");
        let pier = tiles
            .iter()
            .find(|tile| tile.path.contains("structures/pier_"))
            .expect("pier");
        let pilings = tiles
            .iter()
            .find(|tile| tile.path.contains("pier_pilings"))
            .expect("pilings");
        assert!(water
            .iter()
            .any(|tile| tile.col == pier.col && tile.row == pier.row));
        assert!(quay.col != pier.col || quay.row != pier.row);
        assert_eq!((pier.col - quay.col).abs() + (pier.row - quay.row).abs(), 1);
        let facing = corner_toward(pier.col, pier.row, quay.col, quay.row);
        let plate = pier_plate_id(&[facing]);
        assert_eq!(pier.path, pier_path(&plate));
        assert_eq!(plate, "pier_UR");
        assert!(!pier.path.contains("UL_UR"));
        assert_eq!((pilings.col, pilings.row), (pier.col, pier.row));
        for (col, row) in [(1, 1), (2, 1), (1, 2), (2, 2)] {
            assert!(
                tiles
                    .iter()
                    .filter(|tile| tile.col == col && tile.row == row)
                    .all(|tile| tile.z == 0),
                "interior cell {col},{row} must stay water"
            );
        }
        assert_eq!(seam_interior_vertex(), harbour_anchor(1, 1));
    }
}
