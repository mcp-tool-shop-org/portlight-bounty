//! Harbour seam plate. Not a harbour layout and not the first playable.
//!
//! Every tile is placed by [`grid_to_screen`](crate::grid_to_screen) for a
//! 256×128 cell, then the footprint bottom, then the −48 sea datum. Water
//! covers the viewport at both capture zooms so no open edge is on screen.
//! The pier plate is the exact corner set facing the quay, and that cell is
//! water. Pilings sit on a different open water cell.

use crate::harbour::{
    build_harbour, harbour_anchor, harbour_water_cells, harbour_water_tile, harbour_work_tile,
    seam_view, HarbourFault, HarbourTile, WorkKind,
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

/// Pilings on the pier cell. [`build_harbour`] rejects this; nothing is drawn.
fn illegal_seam_plate() -> Result<Vec<HarbourTile>, Vec<HarbourFault>> {
    build_harbour(
        Vec::new(),
        vec![
            harbour_work_tile(
                0,
                2,
                WorkKind::Pilings,
                "res://assets/landing/props/pier_pilings_1x1/beauty.png",
            ),
            harbour_work_tile(
                0,
                2,
                WorkKind::Pier,
                "res://assets/landing/structures/pier_UR/beauty.png",
            ),
        ],
    )
}

/// A pier on the quay cell. [`build_harbour`] rejects this; nothing is drawn.
fn illegal_pier_on_quay_plate() -> Result<Vec<HarbourTile>, Vec<HarbourFault>> {
    build_harbour(
        Vec::new(),
        vec![
            harbour_work_tile(
                0,
                1,
                WorkKind::Pier,
                "res://assets/landing/structures/pier_UR/beauty.png",
            ),
            harbour_work_tile(
                0,
                1,
                WorkKind::Quay,
                "res://assets/landing/structures/quay_1111/beauty.png",
            ),
        ],
    )
}

/// Water past both capture zooms, then the three works in draw order.
///
/// An illegal work layout is [`Err`], not a panic. The seam capture checks
/// this before it writes a PNG and exits non-zero when it fails.
/// `PORTLIGHT_SEAM_ILLEGAL=1` is pilings on a pier cell.
/// `PORTLIGHT_SEAM_ILLEGAL=2` is a pier on a quay cell. Both return through
/// the same [`Err`] path. The legal plate is unchanged when the flag is unset.
pub fn harbour_seam() -> Result<Vec<HarbourTile>, Vec<HarbourFault>> {
    match std::env::var("PORTLIGHT_SEAM_ILLEGAL").ok().as_deref() {
        Some("1") => return illegal_seam_plate(),
        Some("2") => return illegal_pier_on_quay_plate(),
        _ => {}
    }
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
    build_harbour(water, seam_works())
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
            placer.contains("set_name(\"Land\")"),
            "quay paving uses the Land ground layer"
        );
        assert!(
            placer.contains("land.set_z_index(0)"),
            "paving keeps the block's z, so a higher z cannot cover front props"
        );
        assert!(
            placer.contains("quay.add_child"),
            "paving is a child of the quay block and draws after it"
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
        assert!(
            scene.contains("illegal layout"),
            "an illegal layout must be named and must not fall through to a blank PNG"
        );
        assert!(
            scene.contains("capture_frame_rejected"),
            "the seam capture must use the same blank-frame check as PORTLIGHT_SHOT"
        );
        let process = scene
            .split_once("fn process")
            .expect("process")
            .1
            .split_once("fn save_viewport")
            .expect("save")
            .0;
        let illegal_at = process
            .find("self.illegal")
            .expect("process checks the layout");
        let save_at = process.find("save_viewport").expect("process saves");
        assert!(
            illegal_at < save_at,
            "an illegal layout must exit before any seam PNG is written"
        );

        for tile in harbour_seam().expect("seam layout") {
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
            let ground_y = sy + HARBOUR_CELL_H / 2;
            let screen_y = if tile.layer == HarbourLayer::Land {
                // U.4 layer offset is 0,0. Sea datum stays on water and works.
                ground_y
            } else {
                ground_y - WATER_DATUM_Y
            };
            assert_eq!(tile.screen_y, screen_y, "placement for {}", tile.path);
        }

        let (cx, cy) = grid_to_screen(4, 2, CELL_WIDTH, CELL_HEIGHT);
        assert_eq!(water_cell_center(4, 2), (cx as f32, cy as f32));
        assert_eq!(cx, (4 - 2) * 64);
        assert_eq!(cy, (4 + 2) * 32);
    }

    #[test]
    fn seam_is_a_3x3_with_a_water_pier_root() {
        let tiles = harbour_seam().expect("seam layout");
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
        let paving = tiles
            .iter()
            .find(|tile| {
                tile.layer == HarbourLayer::Land && tile.col == quay.col && tile.row == quay.row
            })
            .expect("quay paving");
        assert!(paving.path.contains("quay_flag_"));
        assert!(paving.kind.is_none());
        let paving_at = tiles
            .iter()
            .position(|tile| tile.layer == HarbourLayer::Land)
            .expect("paving");
        let quay_list_at = tiles
            .iter()
            .position(|tile| tile.kind == Some(WorkKind::Quay))
            .expect("quay block");
        let prop_at = tiles
            .iter()
            .position(|tile| tile.kind == Some(WorkKind::Pier))
            .expect("pier");
        assert!(
            quay_list_at < paving_at && paving_at < prop_at,
            "draw order is quay block, then paving, then props"
        );
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
        let tiles = harbour_seam().expect("seam layout");
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

    #[test]
    fn the_illegal_flag_plate_is_pilings_on_the_pier_cell() {
        let body = include_str!("seam.rs")
            .split_once("pub fn harbour_seam()")
            .expect("harbour_seam")
            .1;
        let flag = body
            .find("PORTLIGHT_SEAM_ILLEGAL")
            .expect("flag is checked inside harbour_seam");
        let legal = body.find("seam_works()").expect("legal plate");
        assert!(
            flag < legal,
            "the flag returns the bad plate before the legal layout is built"
        );
        assert_eq!(
            illegal_seam_plate().expect_err("pilings on a pier"),
            vec![HarbourFault::PilingsOnPier { col: 0, row: 2 }]
        );
        assert_eq!(
            illegal_pier_on_quay_plate().expect_err("pier on a quay"),
            vec![HarbourFault::PierOnQuay { col: 0, row: 1 }]
        );
    }
}
