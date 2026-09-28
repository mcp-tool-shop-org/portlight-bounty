//! Harbour seam plate. Not a harbour layout and not the first playable.
//!
//! Every tile is placed by [`grid_to_screen`](crate::grid_to_screen) for a
//! 256×128 cell, then the footprint bottom, then the −48 sea datum. Water
//! covers the viewport at both capture zooms so no open edge is on screen.
//! The pier plate is the exact corner set facing the quay, and that cell is
//! water. Pilings sit on a different open water cell.

use crate::harbour::{
    build_harbour, harbour_anchor, harbour_water_cells, harbour_water_tile, harbour_work_tile,
    seam_view, HarbourTile, WorkKind,
};

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

/// Bottom tip of cell `(1, 1)`. Cells `(1,1)`, `(2,1)`, `(1,2)`, and `(2,2)`
/// meet there. Those four are water, so the vertex is an interior seam.
pub fn seam_interior_vertex() -> (i32, i32) {
    harbour_anchor(1, 1)
}

/// Centre of the quay, pier, and pilings, not of the padded water.
/// Expanding water must not move the camera.
pub fn seam_camera_center() -> (i32, i32) {
    let (vx, vy) = seam_interior_vertex();
    let mut rect = crate::project::ScreenRect::from_point(vx, vy);
    for tile in seam_works() {
        let left = tile.screen_x - tile.anchor_x;
        let top = tile.screen_y - tile.anchor_y;
        rect.include(left, top);
        rect.include(left + tile.canvas_w - 1, top + tile.canvas_h - 1);
    }
    ((rect.min_x + rect.max_x) / 2, (rect.min_y + rect.max_y) / 2)
}

/// Quay at `(0, 1)`, pier root at `(0, 2)`, pilings on open water `(2, 0)`.
fn seam_works() -> Vec<HarbourTile> {
    let quay = (0, 1);
    let pier = (0, 2);
    let corner = corner_toward(pier.0, pier.1, quay.0, quay.1);
    vec![
        harbour_work_tile(
            2,
            0,
            WorkKind::Pilings,
            "res://assets/landing/props/pier_pilings_1x1/beauty.png",
        ),
        harbour_work_tile(
            pier.0,
            pier.1,
            WorkKind::Pier,
            pier_path(&pier_plate_id(&[corner])),
        ),
        harbour_work_tile(
            quay.0,
            quay.1,
            WorkKind::Quay,
            "res://assets/landing/structures/quay_1111/beauty.png",
        ),
    ]
}

/// Water past both capture zooms, then the three works in draw order.
pub fn harbour_seam() -> Vec<HarbourTile> {
    let center = seam_camera_center();
    // 0.72 shows more world than 1.0, so covering it covers the tighter frame.
    let view = seam_view(center, 0.72);
    let mut cells = harbour_water_cells(&view);
    for row in 0..3 {
        for col in 0..3 {
            if !cells.contains(&(col, row)) {
                cells.push((col, row));
            }
        }
    }
    let water = cells
        .into_iter()
        .map(|(col, row)| harbour_water_tile(col, row))
        .collect();
    build_harbour(water, seam_works()).expect("seam layout")
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
    use crate::harbour::{
        harbour_water_covers, validate_harbour, HarbourFault, HarbourLayer, WorkKind,
    };
    use crate::project::{
        grid_to_screen, water_cell_center, CELL_HEIGHT, CELL_WIDTH, HARBOUR_CELL_H, HARBOUR_CELL_W,
        WATER_DATUM_Y,
    };

    #[test]
    fn seam_scene_uses_the_shared_projection() {
        let placement = include_str!("harbour.rs");
        let scene = include_str!("../../portlight-godot/src/seam.rs");
        let placer = include_str!("../../portlight-godot/src/harbour.rs");
        assert!(
            placement.contains("grid_to_screen("),
            "harbour placement must call grid_to_screen"
        );
        assert!(
            scene.contains("harbour_seam("),
            "the seam scene must place the chart crate's tiles"
        );
        assert!(
            scene.contains("place_harbour("),
            "the seam scene must use the shared harbour placer"
        );
        assert!(
            placer.contains("set_y_sort_enabled(true)"),
            "works share one y-sort node"
        );
        assert!(
            !placer.contains("set_z_index(tile"),
            "works are not given a z per id"
        );
        assert!(
            !scene.contains("STEP_U"),
            "the seam scene must not hand-place a step"
        );
        assert!(!placer.contains("STEP_U"));
        assert!(!scene.contains("(col - row) * 128"));
        assert!(!placer.contains("(col - row) * 128"));

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
        assert!(validate_harbour(&tiles).is_ok());
        for row in 0..3 {
            for col in 0..3 {
                assert!(
                    tiles.iter().any(|tile| {
                        tile.layer == HarbourLayer::Water && tile.col == col && tile.row == row
                    }),
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
        assert!(tiles.iter().any(|tile| {
            tile.layer == HarbourLayer::Water && tile.col == pier.col && tile.row == pier.row
        }));
        assert!(quay.col != pier.col || quay.row != pier.row);
        assert_eq!((pier.col - quay.col).abs() + (pier.row - quay.row).abs(), 1);
        let facing = corner_toward(pier.col, pier.row, quay.col, quay.row);
        let plate = pier_plate_id(&[facing]);
        assert_eq!(pier.path, pier_path(&plate));
        assert_eq!(plate, "pier_UR");
        assert!(!pier.path.contains("UL_UR"));
        assert_eq!((pilings.col, pilings.row), (2, 0));
        assert!(pilings.col != pier.col || pilings.row != pier.row);
        let works: Vec<_> = tiles
            .iter()
            .filter(|tile| tile.layer == HarbourLayer::Work)
            .collect();
        let quay_at = works
            .iter()
            .position(|tile| tile.kind == Some(WorkKind::Quay))
            .expect("quay");
        let pier_at = works
            .iter()
            .position(|tile| tile.kind == Some(WorkKind::Pier))
            .expect("pier");
        assert!(
            quay_at < pier_at,
            "the front pier draws after the back quay"
        );
        for (col, row) in [(1, 1), (2, 1), (1, 2), (2, 2)] {
            assert!(
                tiles
                    .iter()
                    .filter(|tile| tile.col == col && tile.row == row)
                    .all(|tile| tile.layer == HarbourLayer::Water),
                "interior cell {col},{row} must stay water"
            );
        }
        assert_eq!(seam_interior_vertex(), harbour_anchor(1, 1));
        let center = seam_camera_center();
        let wide = seam_view(center, 0.72);
        let tight = seam_view(center, 1.0);
        let cells: Vec<_> = tiles
            .iter()
            .filter(|tile| tile.layer == HarbourLayer::Water)
            .map(|tile| (tile.col, tile.row))
            .collect();
        assert!(harbour_water_covers(&cells, &wide));
        assert!(harbour_water_covers(&cells, &tight));
    }

    #[test]
    fn seam_keeps_pilings_off_the_pier() {
        let tiles = harbour_seam();
        let err = validate_harbour(&tiles);
        assert_eq!(err, Ok(()));
        let bad = vec![
            harbour_work_tile(0, 2, WorkKind::Pilings, "res://pilings"),
            harbour_work_tile(0, 2, WorkKind::Pier, "res://pier"),
        ];
        assert_eq!(
            validate_harbour(&bad),
            Err(vec![HarbourFault::PilingsOnPier { col: 0, row: 2 }])
        );
    }
}
