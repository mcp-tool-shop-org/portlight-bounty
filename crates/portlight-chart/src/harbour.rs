//! Harbour placement shared by every harbour scene, including the seam plate.
//!
//! Water is one layer and does not Y-sort. Raised works (quay, pier, pilings,
//! and later buildings) sort by the footprint-bottom anchor. On this grid
//! that anchor's screen Y is `col + row`, so a front cell sorts after a back
//! cell. The same cell breaks the tie by kind: pilings, then pier, then quay.

use crate::cover::{cells_covering, view_world_rect, WATER_COVER_PAD};
use crate::project::{grid_to_screen, ScreenRect, HARBOUR_CELL_H, HARBOUR_CELL_W, WATER_DATUM_Y};

/// Water plate centre pixel, above the footprint-bottom anchor `(128, 127)`.
const WATER_CENTER_ABOVE_ANCHOR: i32 = 127 - 64;
#[cfg(test)]
const WATER_HALF_W: f32 = 128.0;
#[cfg(test)]
const WATER_HALF_H: f32 = 64.0;

/// What a raised plate is. The rank is the tie-break inside one cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkKind {
    Pilings,
    Pier,
    Quay,
}

impl WorkKind {
    /// Lower draws first. Pilings stay under a pier or quay that shares the cell.
    pub fn tie_rank(self) -> u8 {
        match self {
            Self::Pilings => 0,
            Self::Pier => 1,
            Self::Quay => 2,
        }
    }
}

/// A layout the builder refuses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HarbourFault {
    /// Pilings under a pier deck never show.
    PilingsOnPier { col: i32, row: i32 },
    /// A pier root has to be water, not the quay cell.
    PierOnQuay { col: i32, row: i32 },
}

impl std::fmt::Display for HarbourFault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PilingsOnPier { col, row } => {
                write!(f, "pilings on a pier cell ({col}, {row})")
            }
            Self::PierOnQuay { col, row } => write!(f, "pier on a quay cell ({col}, {row})"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HarbourLayer {
    Water,
    Work,
}

/// One placed plate. `screen` is where the texture anchor lands.
#[derive(Clone, Copy, Debug)]
pub struct HarbourTile {
    pub col: i32,
    pub row: i32,
    pub path: &'static str,
    pub anchor_x: i32,
    pub anchor_y: i32,
    pub canvas_w: i32,
    pub canvas_h: i32,
    pub screen_x: i32,
    pub screen_y: i32,
    pub layer: HarbourLayer,
    pub kind: Option<WorkKind>,
}

impl HarbourTile {
    /// Footprint depth. Larger is closer to the camera (screen down).
    pub fn depth(self) -> i32 {
        self.col + self.row
    }
}

/// Anchor pixel of a harbour cell: grid centre, half a cell down to the
/// footprint bottom, then the sea datum. Godot Y grows down, so datum −48
/// is a `+48` screen shift. Chart water does not use this.
pub fn harbour_anchor(col: i32, row: i32) -> (i32, i32) {
    let (sx, sy) = grid_to_screen(col, row, HARBOUR_CELL_W, HARBOUR_CELL_H);
    (sx, sy + HARBOUR_CELL_H / 2 - WATER_DATUM_Y)
}

/// Screen centre of the water diamond. This is the plate centre, not the
/// grid centre: the datum shifts the sprite, and the anchor is the bottom tip.
#[cfg(test)]
fn harbour_diamond_center(col: i32, row: i32) -> (f32, f32) {
    let (sx, sy) = harbour_anchor(col, row);
    (sx as f32, sy as f32 - WATER_CENTER_ABOVE_ANCHOR as f32)
}

/// Fractional `(col, row)` whose diamond centre is the screen point.
pub fn harbour_cell_f(x: f32, y: f32) -> (f32, f32) {
    let dy = (HARBOUR_CELL_H / 2 - WATER_DATUM_Y - WATER_CENTER_ABOVE_ANCHOR) as f32;
    let d = x / (HARBOUR_CELL_W as f32 / 2.0);
    let s = (y - dy) / (HARBOUR_CELL_H as f32 / 2.0);
    ((s + d) / 2.0, (s - d) / 2.0)
}

#[cfg(test)]
fn harbour_cell_at(x: f32, y: f32) -> (i32, i32) {
    let (col, row) = harbour_cell_f(x, y);
    (col.round() as i32, row.round() as i32)
}

pub fn harbour_water_tile(col: i32, row: i32) -> HarbourTile {
    let path = match crate::assets::water_variant(col, row) {
        0 => "res://assets/landing/ground/water_a.png",
        1 => "res://assets/landing/ground/water_b.png",
        _ => "res://assets/landing/ground/water_c.png",
    };
    let (screen_x, screen_y) = harbour_anchor(col, row);
    HarbourTile {
        col,
        row,
        path,
        anchor_x: 128,
        anchor_y: 127,
        canvas_w: 256,
        canvas_h: 128,
        screen_x,
        screen_y,
        layer: HarbourLayer::Water,
        kind: None,
    }
}

pub fn harbour_work_tile(col: i32, row: i32, kind: WorkKind, path: &'static str) -> HarbourTile {
    let (screen_x, screen_y) = harbour_anchor(col, row);
    HarbourTile {
        col,
        row,
        path,
        anchor_x: 128,
        anchor_y: 255,
        canvas_w: 256,
        canvas_h: 256,
        screen_x,
        screen_y,
        layer: HarbourLayer::Work,
        kind: Some(kind),
    }
}

/// Reject pilings co-placed with a pier, and a pier co-placed with a quay.
pub fn validate_harbour(works: &[HarbourTile]) -> Result<(), Vec<HarbourFault>> {
    let mut faults = Vec::new();
    let mut cells: Vec<((i32, i32), Vec<WorkKind>)> = Vec::new();
    for work in works {
        let Some(kind) = work.kind else {
            continue;
        };
        if let Some((_, kinds)) = cells
            .iter_mut()
            .find(|(cell, _)| *cell == (work.col, work.row))
        {
            kinds.push(kind);
        } else {
            cells.push(((work.col, work.row), vec![kind]));
        }
    }
    for ((col, row), kinds) in cells {
        let pier = kinds.contains(&WorkKind::Pier);
        let pilings = kinds.contains(&WorkKind::Pilings);
        let quay = kinds.contains(&WorkKind::Quay);
        if pier && pilings {
            faults.push(HarbourFault::PilingsOnPier { col, row });
        }
        if pier && quay {
            faults.push(HarbourFault::PierOnQuay { col, row });
        }
    }
    if faults.is_empty() {
        Ok(())
    } else {
        Err(faults)
    }
}

/// Back to front. `sort_by` is stable, so equal depth and kind keep input order.
pub fn sort_works(works: &mut [HarbourTile]) {
    works.sort_by(|a, b| {
        a.depth().cmp(&b.depth()).then_with(|| {
            a.kind
                .map(WorkKind::tie_rank)
                .cmp(&b.kind.map(WorkKind::tie_rank))
        })
    });
}

/// Water first, then works in draw order. Refuses an illegal work layout.
pub fn build_harbour(
    water: Vec<HarbourTile>,
    mut works: Vec<HarbourTile>,
) -> Result<Vec<HarbourTile>, Vec<HarbourFault>> {
    validate_harbour(&works)?;
    sort_works(&mut works);
    let mut tiles = water;
    tiles.extend(works);
    Ok(tiles)
}

/// Water cells whose diamonds cover `view` past the tip feather.
pub fn harbour_water_cells(view: &ScreenRect) -> Vec<(i32, i32)> {
    cells_covering(view, WATER_COVER_PAD, harbour_cell_f)
}

#[cfg(test)]
pub fn harbour_water_covers(cells: &[(i32, i32)], view: &ScreenRect) -> bool {
    use crate::cover::{grid_covers_view, TIP_FEATHER_PX};
    grid_covers_view(
        cells,
        view,
        WATER_HALF_W,
        WATER_HALF_H,
        TIP_FEATHER_PX,
        harbour_diamond_center,
        harbour_cell_at,
    )
}

/// The seam camera shows this view at `zoom`. Water for the plate uses the
/// wider of the two capture zooms.
pub fn seam_view(center: (i32, i32), zoom: f32) -> ScreenRect {
    view_world_rect(center.0 as f32, center.1 as f32, zoom, 1280.0, 720.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quay(col: i32, row: i32) -> HarbourTile {
        harbour_work_tile(col, row, WorkKind::Quay, "res://quay")
    }

    fn pier(col: i32, row: i32) -> HarbourTile {
        harbour_work_tile(col, row, WorkKind::Pier, "res://pier")
    }

    fn pilings(col: i32, row: i32) -> HarbourTile {
        harbour_work_tile(col, row, WorkKind::Pilings, "res://pilings")
    }

    #[test]
    fn a_front_cell_sorts_after_a_back_cell() {
        // Depth sort is col+row. An older plate listed the quay last and
        // gave it a higher z, which drew the back wall over the pier.
        let front = pier(0, 2);
        let back = quay(0, 1);
        assert!(front.depth() > back.depth());
        assert!(front.screen_y > back.screen_y);
        let built = build_harbour(Vec::new(), vec![front, back]).expect("legal");
        let works: Vec<_> = built
            .iter()
            .filter(|tile| tile.layer == HarbourLayer::Work)
            .collect();
        assert_eq!(works[0].kind, Some(WorkKind::Quay));
        assert_eq!(works[1].kind, Some(WorkKind::Pier));
        assert!(works[0].depth() < works[1].depth());
    }

    #[test]
    fn the_same_cell_draws_pilings_before_a_quay() {
        let built = build_harbour(Vec::new(), vec![quay(3, 3), pilings(3, 3)]).expect("legal");
        let works: Vec<_> = built
            .iter()
            .filter(|tile| tile.layer == HarbourLayer::Work)
            .collect();
        assert_eq!(works[0].kind, Some(WorkKind::Pilings));
        assert_eq!(works[1].kind, Some(WorkKind::Quay));
        assert_eq!(works[0].depth(), works[1].depth());
    }

    #[test]
    fn equal_depth_keeps_input_order() {
        let built = build_harbour(Vec::new(), vec![quay(1, 2), quay(0, 3)]).expect("legal");
        let works: Vec<_> = built
            .iter()
            .filter(|tile| tile.layer == HarbourLayer::Work)
            .collect();
        assert_eq!((works[0].col, works[0].row), (1, 2));
        assert_eq!((works[1].col, works[1].row), (0, 3));
    }

    #[test]
    fn water_tiles_follow_the_cell_hash() {
        let mut seen = [false; 3];
        let mut differs_from_stripe = false;
        for row in 0..5 {
            for col in 0..5 {
                let tile = harbour_water_tile(col, row);
                let variant = crate::assets::water_variant(col, row);
                let expect = match variant {
                    0 => "water_a.png",
                    1 => "water_b.png",
                    _ => "water_c.png",
                };
                assert!(
                    tile.path.ends_with(expect),
                    "({}, {}) -> {} wanted {expect}",
                    tile.col,
                    tile.row,
                    tile.path
                );
                seen[variant as usize] = true;
                let stripe = match (col + row).rem_euclid(3) {
                    0 => "water_a.png",
                    1 => "water_b.png",
                    _ => "water_c.png",
                };
                differs_from_stripe |= expect != stripe;
            }
        }
        assert_eq!(seen, [true, true, true]);
        assert!(differs_from_stripe);
    }

    #[test]
    fn validator_rejects_pilings_on_a_pier_and_a_pier_on_a_quay() {
        let on_pier = validate_harbour(&[pilings(0, 2), pier(0, 2)]).unwrap_err();
        assert_eq!(
            on_pier,
            vec![HarbourFault::PilingsOnPier { col: 0, row: 2 }]
        );
        let on_quay = validate_harbour(&[pier(0, 1), quay(0, 1)]).unwrap_err();
        assert_eq!(on_quay, vec![HarbourFault::PierOnQuay { col: 0, row: 1 }]);
        assert!(validate_harbour(&[pilings(2, 0), pier(0, 2), quay(0, 1)]).is_ok());
        assert_eq!(on_pier[0].to_string(), "pilings on a pier cell (0, 2)");
        assert_eq!(on_quay[0].to_string(), "pier on a quay cell (0, 1)");
    }
}
