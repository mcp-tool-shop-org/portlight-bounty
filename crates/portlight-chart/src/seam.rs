//! Harbour seam plate. Not a harbour layout and not the first playable.
//!
//! Every tile is placed by [`grid_to_screen`](crate::grid_to_screen) for a
//! 256×128 cell, then the footprint bottom, then the −48 sea datum. Water
//! covers the viewport at both capture zooms so no open edge is on screen.
//! The pier plate is the exact corner set facing the quay, and that cell is
//! water. Pilings sit on a different open water cell.
//!
//! Behind the join quay a quay deck shows flat cells. A kerb of
//! `quay_1111` blocks runs along row `1` (cols `-7..=-1`, beside the join
//! quay) and along col `0` (rows `-8..=0`). Behind it every cell with col
//! `-7..=-1` and row `-8..=0` is flat deck with no block, drawing the locked
//! `quay_flag_*` plates the same way a block top does. The deck
//! runs past the widest capture view, so no flat back edge meets open water
//! on camera, and the far kerb ends are off camera too. The camera still
//! frames the quay, pier, and pilings only, so the deck adds pixels without
//! moving the rest of the plate.

use crate::harbour::{
    build_harbour, build_harbour_with_deck, harbour_anchor, harbour_prop_tile, harbour_water_cells,
    harbour_water_tile, harbour_work_tile, seam_view, HarbourFault, HarbourTile, WorkKind,
};

/// Columns of the seam flat deck, inclusive. Col `0` is the kerb.
pub const SEAM_DECK_COLS: (i32, i32) = (-7, -1);

/// Rows of the seam flat deck, inclusive. Row `1` is the kerb.
pub const SEAM_DECK_ROWS: (i32, i32) = (-8, 0);

/// Quay blocks that form the kerb in front of the flat deck: row `1` beside
/// the join quay `(0, 1)`, then col `0`. Each is the locked `quay_1111`
/// block with its flag on top, like the join quay.
pub fn seam_deck_blocks() -> Vec<(i32, i32)> {
    let (col_min, col_max) = SEAM_DECK_COLS;
    let (row_min, row_max) = SEAM_DECK_ROWS;
    let mut blocks: Vec<(i32, i32)> = (col_min..=col_max).map(|col| (col, 1)).collect();
    blocks.extend((row_min..=row_max).map(|row| (0, row)));
    blocks
}

/// Flat quay deck cells with no raised block. Both front
/// neighbours of each are a quay block or another deck cell, and each back
/// edge is another deck cell or off camera.
pub fn seam_flat_deck() -> Vec<(i32, i32)> {
    let (col_min, col_max) = SEAM_DECK_COLS;
    let (row_min, row_max) = SEAM_DECK_ROWS;
    let mut deck = Vec::new();
    for row in row_min..=row_max {
        for col in col_min..=col_max {
            deck.push((col, row));
        }
    }
    deck
}

/// Zoom-1 review crop of the flat deck and its kerb, in view pixels
/// `(x, y, w, h)`. [`flat_deck_crop`] maps this same world window to other
/// zooms.
pub const FLAT_DECK_CROP_Z100: (i32, i32, i32, i32) = (224, 72, 432, 352);

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

/// Zoom-1 review crop of the quay block, in view pixels `(x, y, w, h)`.
/// [`quay_paving_crop`] maps this same world window to other zooms.
pub const QUAY_PAVING_CROP_Z100: (i32, i32, i32, i32) = (340, 160, 360, 340);

/// View crop of the quay close-up at `zoom`. Zoom `1` is
/// [`QUAY_PAVING_CROP_Z100`]. Zoom `0.72` is the committed
/// `quay-paving-z072.png` box `(424, 216, 259, 245)`.
pub fn quay_paving_crop(zoom: f32) -> (i32, i32, i32, i32) {
    crop_at_zoom(QUAY_PAVING_CROP_Z100, zoom)
}

/// View crop of the flat deck close-up at `zoom`. Zoom `1` is
/// [`FLAT_DECK_CROP_Z100`]; other zooms show the same world window.
pub fn flat_deck_crop(zoom: f32) -> (i32, i32, i32, i32) {
    crop_at_zoom(FLAT_DECK_CROP_Z100, zoom)
}

/// A zoom-1 view box mapped to the same world window at `zoom`.
fn crop_at_zoom(z100: (i32, i32, i32, i32), zoom: f32) -> (i32, i32, i32, i32) {
    let (x, y, w, h) = z100;
    let (cx, cy) = seam_camera_center();
    let world = |vx: i32, vy: i32| -> (f32, f32) {
        (
            (vx as f32 - 640.0) + cx as f32,
            (vy as f32 - 360.0) + cy as f32,
        )
    };
    let (wx, wy) = world(x, y);
    let (wr, wb) = world(x + w, y + h);
    let view = |wx: f32, wy: f32| -> (f32, f32) {
        (
            (wx - cx as f32) * zoom + 640.0,
            (wy - cy as f32) * zoom + 360.0,
        )
    };
    let (vx, vy) = view(wx, wy);
    let (vr, vb) = view(wr, wb);
    (
        vx.round() as i32,
        vy.round() as i32,
        (vr - vx).round() as i32,
        (vb - vy).round() as i32,
    )
}

/// Centre of the quay, pier, and pilings, not of the padded water or the
/// flat deck. Expanding water or the deck must not move the camera.
pub fn seam_camera_center() -> (i32, i32) {
    let (vx, vy) = seam_interior_vertex();
    let mut rect = crate::project::ScreenRect::from_point(vx, vy);
    for tile in seam_focus_works() {
        let left = tile.screen_x - tile.anchor_x;
        let top = tile.screen_y - tile.anchor_y;
        rect.include(left, top);
        rect.include(left + tile.canvas_w - 1, top + tile.canvas_h - 1);
    }
    ((rect.min_x + rect.max_x) / 2, (rect.min_y + rect.max_y) / 2)
}

/// The camera's works, the kerb blocks in front of the flat deck, then deck
/// props.
///
/// The kerb already holds the back quay `(0,0)` and side kerb `(-1,1)`, so
/// each block is placed once. Pier root stays water cell `(0,2)`. The pier head gets bollard+torch;
/// the join, back, and side quays each get one cargo prop.
fn seam_works() -> Vec<HarbourTile> {
    let quay_join = (0, 1);
    let quay_back = (0, 0);
    let quay_side = (-1, 1);
    let pier = (0, 2);
    let mut works = seam_focus_works();
    for (col, row) in seam_deck_blocks() {
        works.push(harbour_work_tile(
            col,
            row,
            WorkKind::Quay,
            "res://assets/landing/structures/quay_1111/beauty.png",
        ));
    }
    works.extend([
        // Pier head: bollard + torch (the one density exception).
        harbour_prop_tile(pier.0, pier.1, prop_path("bollard_1x1")),
        harbour_prop_tile(pier.0, pier.1, prop_path("torch_1x1")),
        // One prop per quay cell, near the water kerb cells.
        harbour_prop_tile(quay_join.0, quay_join.1, prop_path("barrel_1x1")),
        harbour_prop_tile(quay_back.0, quay_back.1, prop_path("crate_1x1")),
        harbour_prop_tile(quay_side.0, quay_side.1, prop_path("cart_1x1")),
    ]);
    works
}

/// Quay at `(0, 1)`, pier root at `(0, 2)`, pilings on open water `(2, 0)`.
fn seam_focus_works() -> Vec<HarbourTile> {
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

/// Water past both capture zooms, then the works and the flat deck in draw
/// order.
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
    // The deck's open back edges must sit outside the widest capture view.
    build_harbour_with_deck(water, seam_works(), &seam_flat_deck(), Some(&view))
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

/// Shipped Phase-2 will-use prop plates (ai-rpg-stage). Paths match the
/// landing `props/<id>/beauty.png` convention used by `pier_pilings_1x1`.
fn prop_path(id: &str) -> &'static str {
    match id {
        "bollard_1x1" => "res://assets/landing/props/bollard_1x1/beauty.png",
        "torch_1x1" => "res://assets/landing/props/torch_1x1/beauty.png",
        "barrel_1x1" => "res://assets/landing/props/barrel_1x1/beauty.png",
        "crate_1x1" => "res://assets/landing/props/crate_1x1/beauty.png",
        "cart_1x1" => "res://assets/landing/props/cart_1x1/beauty.png",
        "well_1x1" => "res://assets/landing/props/well_1x1/beauty.png",
        _ => panic!("unknown harbour prop id {id}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harbour::{
        flat_deck_back_edges, harbour_water_covers, validate_harbour, HarbourFault, HarbourLayer,
        WorkKind,
    };
    use crate::project::ScreenRect;
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
            placer.contains("host.add_child(&land)"),
            "paving is a child of the quay block and draws after it"
        );
        assert!(
            placer.contains("HarbourLayer::Prop"),
            "deck props parent under the pier or quay host"
        );
        assert!(
            scene.contains("quay-paving-z100.png"),
            "the seam capture writes the zoom-1 quay close-up"
        );
        assert!(
            scene.contains("quay-paving-z072.png"),
            "the seam capture writes the zoom-0.72 quay close-up"
        );
        assert_eq!(quay_paving_crop(1.0), QUAY_PAVING_CROP_Z100);
        assert_eq!(quay_paving_crop(0.72), (424, 216, 259, 245));
        assert!(
            scene.contains("flat-deck-z100.png"),
            "the seam capture writes the zoom-1 flat deck close-up"
        );
        assert!(
            scene.contains("flat-deck-z072.png"),
            "the seam capture writes the zoom-0.72 flat deck close-up"
        );
        assert!(
            scene.contains("flat_deck_crop("),
            "the flat deck close-up uses the shared crop"
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
            let screen_y = if matches!(tile.layer, HarbourLayer::Land | HarbourLayer::Prop) {
                // Land offset 0. Sea datum stays on water and works.
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
            .find(|tile| tile.path.contains("quay_1111") && (tile.col, tile.row) == (0, 1))
            .expect("join quay");
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
            .position(|tile| tile.layer == HarbourLayer::Land && (tile.col, tile.row) == (0, 1))
            .expect("paving");
        let quay_list_at = tiles
            .iter()
            .position(|tile| tile.kind == Some(WorkKind::Quay) && (tile.col, tile.row) == (0, 1))
            .expect("quay block");
        let deck_prop_at = tiles
            .iter()
            .position(|tile| tile.kind == Some(WorkKind::Prop) && (tile.col, tile.row) == (0, 1))
            .expect("join deck prop");
        let pier_at_join = tiles
            .iter()
            .position(|tile| tile.kind == Some(WorkKind::Pier))
            .expect("pier");
        assert!(
            quay_list_at < paving_at && paving_at < deck_prop_at,
            "join cell draw order is quay block, then paving, then its deck prop"
        );
        // Pier is on a front cell, so it may sort after join-cell paving too.
        assert!(paving_at < pier_at_join || pier_at_join != paving_at);
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
        // Prop map: pier head bollard+torch; one cargo prop per quay cell.
        let props: Vec<_> = tiles
            .iter()
            .filter(|tile| tile.kind == Some(WorkKind::Prop))
            .map(|tile| ((tile.col, tile.row), tile.path))
            .collect();
        assert!(props.contains(&((0, 2), "res://assets/landing/props/bollard_1x1/beauty.png")));
        assert!(props.contains(&((0, 2), "res://assets/landing/props/torch_1x1/beauty.png")));
        assert!(props.contains(&((0, 1), "res://assets/landing/props/barrel_1x1/beauty.png")));
        assert!(props.contains(&((0, 0), "res://assets/landing/props/crate_1x1/beauty.png")));
        assert!(props.contains(&((-1, 1), "res://assets/landing/props/cart_1x1/beauty.png")));
        assert_eq!(props.len(), 5);
        // Join quay plus the 16-block kerb. The back `(0,0)`
        // and side `(-1,1)` quays are kerb cells and are placed once.
        assert_eq!(
            tiles
                .iter()
                .filter(|tile| tile.kind == Some(WorkKind::Quay))
                .count(),
            1 + seam_deck_blocks().len(),
            "join quay + kerb"
        );
        for cell in [(0, 1), (0, 0), (-1, 1)] {
            assert_eq!(
                tiles
                    .iter()
                    .filter(|tile| tile.kind == Some(WorkKind::Quay) && (tile.col, tile.row) == cell)
                    .count(),
                1,
                "one quay block on {cell:?}"
            );
        }
        let works: Vec<_> = tiles
            .iter()
            .filter(|tile| tile.layer == HarbourLayer::Work)
            .collect();
        let quay_at = works
            .iter()
            .position(|tile| tile.kind == Some(WorkKind::Quay) && (tile.col, tile.row) == (0, 1))
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
    fn seam_flat_deck_cells_carry_the_hashed_flag() {
        let tiles = harbour_seam().expect("seam layout");
        // Cell -> flag pins. A hash change or a moved cell fails here.
        let pins = [
            ((-2, 0), "quay_flag_a", true),
            ((-1, 0), "quay_flag_b", true),
            ((0, 0), "quay_flag_a", false),
            ((-1, 1), "quay_flag_a", false),
            ((-2, 1), "quay_flag_a", false),
            ((0, 1), "quay_flag_c", false),
        ];
        for ((col, row), id, flat) in pins {
            let land: Vec<_> = tiles
                .iter()
                .enumerate()
                .filter(|(_, tile)| {
                    tile.layer == HarbourLayer::Land && (tile.col, tile.row) == (col, row)
                })
                .collect();
            assert_eq!(land.len(), 1, "one flag on ({col}, {row})");
            let (at, tile) = land[0];
            assert!(
                tile.path.ends_with(&format!("ground/{id}.png")),
                "({col}, {row})"
            );
            let want =
                crate::harbour::harbour_quay_paving_tile(col, row).expect("quay paving tile");
            assert_eq!(
                (
                    tile.path,
                    tile.screen_x,
                    tile.screen_y,
                    tile.anchor_x,
                    tile.anchor_y
                ),
                (want.path, want.screen_x, want.screen_y, 128, 127)
            );
            assert_eq!((tile.canvas_w, tile.canvas_h), (256, 128));
            let block = tiles
                .iter()
                .position(|t| t.kind == Some(WorkKind::Quay) && (t.col, t.row) == (col, row));
            if flat {
                assert!(block.is_none(), "flat deck ({col}, {row}) has no block");
                assert!(
                    tiles
                        .iter()
                        .all(|t| t.kind.is_none() || (t.col, t.row) != (col, row)),
                    "flat deck ({col}, {row}) holds no work"
                );
                for front in [(col + 1, row), (col, row + 1)] {
                    let quay_front = tiles
                        .iter()
                        .any(|t| t.kind == Some(WorkKind::Quay) && (t.col, t.row) == front);
                    assert!(
                        quay_front || seam_flat_deck().contains(&front),
                        "({col}, {row}) front {front:?} is deck height"
                    );
                    let front_at = tiles
                        .iter()
                        .position(|t| t.layer != HarbourLayer::Water && (t.col, t.row) == front)
                        .expect("front tile");
                    assert!(
                        at < front_at,
                        "front of ({col}, {row}) draws over the deck edge"
                    );
                }
                for (i, t) in tiles.iter().enumerate() {
                    if t.kind.is_some() && t.depth() < col + row {
                        assert!(
                            i < at,
                            "back work {:?} draws before deck ({col}, {row})",
                            (t.col, t.row)
                        );
                    }
                    if t.kind.is_some() && t.depth() >= col + row {
                        assert!(
                            i > at,
                            "work {:?} draws after deck ({col}, {row})",
                            (t.col, t.row)
                        );
                    }
                }
            } else {
                assert_eq!(
                    block.map(|b| b + 1),
                    Some(at),
                    "block ({col}, {row}) then its flag"
                );
            }
        }
        assert_eq!(
            tiles
                .iter()
                .filter(|t| t.layer == HarbourLayer::Land)
                .count(),
            seam_deck_blocks().len() + 1 + seam_flat_deck().len(),
            "kerb and join block tops plus every flat deck cell"
        );
        assert!(tiles.iter().all(|t| !t.path.contains("quay_flat")));

        // The deck does not move the camera or the quay close-up.
        assert_eq!(seam_camera_center(), (0, 80));
        assert_eq!(flat_deck_crop(1.0), FLAT_DECK_CROP_Z100);
        assert_eq!(flat_deck_crop(0.72), (340, 153, 311, 253));
        let (cx, cy) = seam_camera_center();
        for zoom in [1.0_f32, 0.72] {
            let (x, y, w, h) = flat_deck_crop(zoom);
            assert!(
                x >= 0 && y >= 0 && x + w <= 1280 && y + h <= 720,
                "zoom {zoom}"
            );
            for (col, row) in [(-1, 0), (-2, 0)] {
                let tile =
                    crate::harbour::harbour_quay_paving_tile(col, row).expect("quay paving tile");
                let view = |wx: i32, wy: i32| {
                    (
                        ((wx - cx) as f32 * zoom + 640.0).round() as i32,
                        ((wy - cy) as f32 * zoom + 360.0).round() as i32,
                    )
                };
                let (l, t) = view(tile.screen_x - tile.anchor_x, tile.screen_y - tile.anchor_y);
                let (r, b) = view(
                    tile.screen_x - tile.anchor_x + tile.canvas_w,
                    tile.screen_y - tile.anchor_y + tile.canvas_h,
                );
                assert!(
                    l >= x && t >= y && r <= x + w && b <= y + h,
                    "flat deck ({col}, {row}) inside the zoom {zoom} close-up"
                );
            }
        }
    }

    #[test]
    fn seam_flat_deck_has_no_open_back_edge_on_camera() {
        // Every flat deck cell is placed like a block top, its
        // fronts are deck height, and each back edge is deck height or lies
        // wholly outside the widest capture view (so outside z1 and both
        // close-ups too).
        let tiles = harbour_seam().expect("seam layout");
        let deck = seam_flat_deck();
        let blocks = seam_deck_blocks();
        assert_eq!(deck.len(), 63);
        assert_eq!(blocks.len(), 16);
        assert_eq!(blocks.iter().filter(|cell| deck.contains(cell)).count(), 0);
        let quay_cells: Vec<(i32, i32)> = tiles
            .iter()
            .filter(|t| t.kind == Some(WorkKind::Quay))
            .map(|t| (t.col, t.row))
            .collect();
        assert_eq!(
            quay_cells.len(),
            blocks.len() + 1,
            "kerb plus the join quay"
        );
        let deck_height = |cell: &(i32, i32)| quay_cells.contains(cell) || deck.contains(cell);
        let center = seam_camera_center();
        let wide = seam_view(center, 0.72);
        let tight = seam_view(center, 1.0);
        let world_crop = |(x, y, w, h): (i32, i32, i32, i32)| ScreenRect {
            min_x: x - 640 + center.0,
            min_y: y - 360 + center.1,
            max_x: x + w - 640 + center.0,
            max_y: y + h - 360 + center.1,
        };
        let frames = [
            wide,
            tight,
            world_crop(FLAT_DECK_CROP_Z100),
            world_crop(QUAY_PAVING_CROP_Z100),
        ];
        let overlap = |a: &ScreenRect, b: &ScreenRect| {
            a.min_x <= b.max_x && b.min_x <= a.max_x && a.min_y <= b.max_y && b.min_y <= a.max_y
        };
        let mut open_off_camera = 0;
        for &(col, row) in &deck {
            let land: Vec<_> = tiles
                .iter()
                .filter(|t| t.layer == HarbourLayer::Land && (t.col, t.row) == (col, row))
                .collect();
            assert_eq!(land.len(), 1, "one flag on flat ({col}, {row})");
            let want =
                crate::harbour::harbour_quay_paving_tile(col, row).expect("quay paving tile");
            assert_eq!(
                (land[0].path, land[0].screen_x, land[0].screen_y),
                (want.path, want.screen_x, want.screen_y)
            );
            assert!(
                !quay_cells.contains(&(col, row)),
                "no block on flat ({col}, {row})"
            );
            assert!(deck_height(&(col + 1, row)) && deck_height(&(col, row + 1)));
            for (neighbour, edge) in flat_deck_back_edges(col, row) {
                if deck_height(&neighbour) {
                    continue;
                }
                open_off_camera += 1;
                for frame in &frames {
                    assert!(
                        !overlap(&edge, frame),
                        "flat ({col}, {row}) back edge toward {neighbour:?} is on camera"
                    );
                }
            }
        }
        // Outer deck rows and cols are open, all off camera.
        assert_eq!(open_off_camera, 7 + 9);

        // The kerb ends are off camera too, so no deck edge or block end shows.
        for (col, row) in [(SEAM_DECK_COLS.0, 1), (0, SEAM_DECK_ROWS.0)] {
            let block = tiles
                .iter()
                .find(|t| t.kind == Some(WorkKind::Quay) && (t.col, t.row) == (col, row))
                .expect("kerb end");
            let plate = ScreenRect {
                min_x: block.screen_x - block.anchor_x,
                min_y: block.screen_y - block.anchor_y,
                max_x: block.screen_x - block.anchor_x + block.canvas_w - 1,
                max_y: block.screen_y - block.anchor_y + block.canvas_h - 1,
            };
            assert!(
                !overlap(&plate, &wide),
                "kerb end ({col}, {row}) is on camera"
            );
        }

        // Without the off-camera allowance the same deck fails the production
        // rule on exactly its open outer cells.
        let strict = build_harbour_with_deck(Vec::new(), seam_works(), &deck, None).unwrap_err();
        assert_eq!(
            strict.len(),
            7 + 9 - 1,
            "the far corner is one cell with two open edges"
        );
        assert!(strict
            .iter()
            .all(|fault| matches!(fault, HarbourFault::FlatDeckOpenBack { .. })));
        // A shorter deck shows its back edge on camera and is refused.
        let short: Vec<_> = deck.iter().copied().filter(|&(_, row)| row >= -2).collect();
        let refused = build_harbour_with_deck(Vec::new(), seam_works(), &short, Some(&wide))
            .expect_err("short deck");
        assert!(refused
            .iter()
            .any(|fault| matches!(fault, HarbourFault::FlatDeckOpenBack { .. })));
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
