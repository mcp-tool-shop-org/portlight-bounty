//! Harbour placement shared by every harbour scene, including the seam plate.
//!
//! Water is one layer and does not Y-sort. Quay paving is land ground on the
//! cells that hold a quay block: it sits on layer Land at
//! offset `0,0` with y-sort origin `0`. That is the footprint bottom with no
//! sea datum, so the flag diamond registers on the quay block's top face.
//! Draw order on that cell is the block, then the paving, then any later
//! prop. Raised works (quay, pier, pilings, and later buildings) sort by
//! the footprint-bottom anchor. On this grid that anchor's screen Y is
//! `col + row`, so a front cell sorts after a back cell. The same cell breaks
//! the tie by kind: pilings, then pier, then quay, then props. Paving is
//! inserted after the last quay block of its cell, so a deck prop still draws
//! after the flag.
//!
//! Flat deck: a land cell at quay-top height with no
//! raised block draws the same `quay_flag_*` plate, from the same
//! [`harbour_quay_paving_tile`] (same Land values and variant hash). It is
//! only legal where both front neighbours (`+col` and `+row`) are a quay
//! block or another flat deck cell, so the diamond never hangs over water and
//! never needs a front face of its own. It holds no pilings, pier, or quay.
//!
//! The flag has no wall face, so a flat deck edge against open
//! water reads paper-thin at sea level. Each back neighbour (`-col`, `-row`)
//! must also be deck height, unless the caller passes a compare frame and
//! that back edge lies wholly outside it (off camera). There is no edge
//! plate; the fix is more deck or more `quay_*` kerb, never a new plate.
//! It sorts at its own footprint depth, ahead of every work at that depth or
//! deeper, so a back block's front face sits under the deck and every front
//! block and prop draws over it.

use crate::cover::{cells_covering, view_world_rect, WATER_COVER_PAD};
use crate::project::{grid_to_screen, ScreenRect, HARBOUR_CELL_H, HARBOUR_CELL_W, WATER_DATUM_Y};

/// Water plate centre pixel, above the footprint-bottom anchor `(128, 127)`.
const WATER_CENTER_ABOVE_ANCHOR: i32 = 127 - 64;
#[cfg(test)]
const WATER_HALF_W: f32 = 128.0;
#[cfg(test)]
const WATER_HALF_H: f32 = 64.0;

/// What a raised plate is. The rank is the tie-break inside one cell, among works.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkKind {
    Pilings,
    Pier,
    Quay,
    /// Deck dressing (bollard, torch, barrel, …). Draws after paving on the cell.
    Prop,
}

impl WorkKind {
    /// Lower draws first. Pilings stay under a pier or quay that shares the cell.
    /// Props are last so they sit above the flag and the structure top.
    pub fn tie_rank(self) -> u8 {
        match self {
            Self::Pilings => 0,
            Self::Pier => 1,
            Self::Quay => 2,
            Self::Prop => 3,
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
    /// More than one prop on a cell that is not a pier head, or a pier-head
    /// pair that is not bollard+torch (prop density cap).
    PropDensity { col: i32, row: i32, count: usize },
    /// A flat deck cell also holds pilings, a pier, or a quay block.
    FlatDeckOnWork { col: i32, row: i32 },
    /// A flat deck cell has a front neighbour (`+col` or `+row`) that is not
    /// deck height, so its flag would hang over water.
    FlatDeckOpenFront { col: i32, row: i32 },
    /// A flat deck cell has a back neighbour (`-col` or `-row`) that is not
    /// deck height, and that edge shows in the compare frame (no paper-thin
    /// deck rim against open water).
    FlatDeckOpenBack { col: i32, row: i32 },
    /// `quay_flag_*` is not placed as quay paving: layer Land, offset 0,0, y-sort 0.
    QuayFlagPlacement {
        id: &'static str,
        layer: &'static str,
        offset_x: i32,
        offset_y: i32,
        y_sort: i32,
    },
}

impl std::fmt::Display for HarbourFault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PilingsOnPier { col, row } => {
                write!(f, "pilings on a pier cell ({col}, {row})")
            }
            Self::PierOnQuay { col, row } => write!(f, "pier on a quay cell ({col}, {row})"),
            Self::PropDensity { col, row, count } => {
                write!(f, "prop density {count} on cell ({col}, {row})")
            }
            Self::FlatDeckOnWork { col, row } => {
                write!(f, "flat deck on a work cell ({col}, {row})")
            }
            Self::FlatDeckOpenFront { col, row } => {
                write!(f, "flat deck ({col}, {row}) has an open front over water")
            }
            Self::FlatDeckOpenBack { col, row } => write!(
                f,
                "flat deck ({col}, {row}) shows an open back edge against water"
            ),
            Self::QuayFlagPlacement {
                id,
                layer,
                offset_x,
                offset_y,
                y_sort,
            } => write!(
                f,
                "{id} must be layer Land, offset 0,0, y_sort 0 (quay paving rule); \
                 MANIFEST has layer {layer}, offset {offset_x},{offset_y}, y_sort {y_sort}"
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HarbourLayer {
    Water,
    /// Quay paving ground. Land anchor, drawn after the quay block, or
    /// on its own at footprint depth on a flat deck cell.
    Land,
    Work,
    /// Deck prop at land offset 0. Parented under the
    /// pier or quay on the cell so Y-sort matches the structure and the prop
    /// draws after paving.
    Prop,
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

/// Quay paving for one cell. The plate is the manifest `quay_flag_*` entry
/// for [`crate::assets::quay_flag_variant`]. Screen position is the footprint
/// bottom plus that entry's layer offset (the Land offset is `0,0`, so no sea datum).
/// Block tops and flat deck cells both use this one constructor.
///
/// A plate that is not layer Land at offset `0,0` with y-sort `0` returns
/// an error. That used to panic while the frame was drawn.
pub fn harbour_quay_paving_tile(col: i32, row: i32) -> Result<HarbourTile, HarbourFault> {
    let plate = crate::assets::quay_flag_plate(col, row);
    quay_flag_u4(plate)?;
    let (sx, sy) = grid_to_screen(col, row, HARBOUR_CELL_W, HARBOUR_CELL_H);
    Ok(HarbourTile {
        col,
        row,
        path: plate.res_path,
        anchor_x: plate.anchor_x,
        anchor_y: plate.anchor_y,
        canvas_w: plate.canvas_w,
        canvas_h: plate.canvas_h,
        screen_x: sx + plate.offset_x,
        screen_y: sy + HARBOUR_CELL_H / 2 + plate.offset_y,
        layer: HarbourLayer::Land,
        kind: None,
    })
}

/// Quay paving placement for one manifest plate. Anything else is a fault the seam
/// can print; it is not a panic.
fn quay_flag_u4(plate: &crate::assets::QuayFlagPlate) -> Result<(), HarbourFault> {
    if plate.layer == "Land" && plate.offset_x == 0 && plate.offset_y == 0 && plate.y_sort == 0 {
        return Ok(());
    }
    Err(HarbourFault::QuayFlagPlacement {
        id: plate.id,
        layer: plate.layer,
        offset_x: plate.offset_x,
        offset_y: plate.offset_y,
        y_sort: plate.y_sort,
    })
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

/// Deck prop at land offset 0 (no sea datum). Same foot anchor as SeaWorks
/// props (128, 255 on a 256×256 plate).
pub fn harbour_prop_tile(col: i32, row: i32, path: &'static str) -> HarbourTile {
    let (sx, sy) = grid_to_screen(col, row, HARBOUR_CELL_W, HARBOUR_CELL_H);
    HarbourTile {
        col,
        row,
        path,
        anchor_x: 128,
        anchor_y: 255,
        canvas_w: 256,
        canvas_h: 256,
        screen_x: sx,
        screen_y: sy + HARBOUR_CELL_H / 2,
        layer: HarbourLayer::Prop,
        kind: Some(WorkKind::Prop),
    }
}

/// Reject pilings co-placed with a pier, a pier co-placed with a quay, and
/// over-dense props (one prop per cell, or bollard+torch on a pier head).
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
    for ((col, row), kinds) in &cells {
        let pier = kinds.contains(&WorkKind::Pier);
        let pilings = kinds.contains(&WorkKind::Pilings);
        let quay = kinds.contains(&WorkKind::Quay);
        if pier && pilings {
            faults.push(HarbourFault::PilingsOnPier {
                col: *col,
                row: *row,
            });
        }
        if pier && quay {
            faults.push(HarbourFault::PierOnQuay {
                col: *col,
                row: *row,
            });
        }
    }
    // Prop density: paths on Prop tiles, one per cell unless pier head
    // bollard+torch.
    let mut prop_cells: Vec<((i32, i32), Vec<&'static str>)> = Vec::new();
    for work in works {
        if work.kind != Some(WorkKind::Prop) {
            continue;
        }
        let id = prop_id_from_path(work.path);
        if let Some((_, ids)) = prop_cells
            .iter_mut()
            .find(|(cell, _)| *cell == (work.col, work.row))
        {
            ids.push(id);
        } else {
            prop_cells.push(((work.col, work.row), vec![id]));
        }
    }
    for ((col, row), ids) in prop_cells {
        let pier = cells
            .iter()
            .find(|(cell, _)| *cell == (col, row))
            .map(|(_, kinds)| kinds.contains(&WorkKind::Pier))
            .unwrap_or(false);
        let ok = match (pier, ids.len()) {
            (_, 0 | 1) => true,
            (true, 2) => {
                let mut sorted = ids.clone();
                sorted.sort_unstable();
                sorted == ["bollard_1x1", "torch_1x1"]
            }
            _ => false,
        };
        if !ok {
            faults.push(HarbourFault::PropDensity {
                col,
                row,
                count: ids.len(),
            });
        }
    }
    if faults.is_empty() {
        Ok(())
    } else {
        Err(faults)
    }
}

/// Manifest id from a `…/props/<id>/beauty.png` path (or the last path segment).
fn prop_id_from_path(path: &'static str) -> &'static str {
    // res://assets/landing/props/bollard_1x1/beauty.png -> bollard_1x1
    let rest = path.rsplit_once("/props/").map(|(_, r)| r).unwrap_or(path);
    rest.split_once('/').map(|(id, _)| id).unwrap_or(rest)
}

/// Pixels kept clear around a flat deck back edge when it is judged off camera.
pub const FLAT_EDGE_PAD_PX: i32 = 2;

/// Screen box of the two back edges of a flat deck diamond, with the
/// neighbour each one faces. The land diamond spans `gy - 64 ..= gy + 64`
/// around the grid centre `(gx, gy)`; the Land anchor puts no sea datum on it.
pub fn flat_deck_back_edges(col: i32, row: i32) -> [((i32, i32), ScreenRect); 2] {
    let (gx, gy) = grid_to_screen(col, row, HARBOUR_CELL_W, HARBOUR_CELL_H);
    let half_w = HARBOUR_CELL_W / 2;
    let half_h = HARBOUR_CELL_H / 2;
    let edge = |x0: i32, x1: i32| {
        let mut rect = ScreenRect::from_point(x0, gy);
        rect.include(x1, gy - half_h);
        rect.pad(FLAT_EDGE_PAD_PX, FLAT_EDGE_PAD_PX)
    };
    [
        // Upper-left edge faces `-col`.
        ((col - 1, row), edge(gx - half_w, gx)),
        // Upper-right edge faces `-row`.
        ((col, row - 1), edge(gx, gx + half_w)),
    ]
}

fn rects_overlap(a: &ScreenRect, b: &ScreenRect) -> bool {
    a.min_x <= b.max_x && b.min_x <= a.max_x && a.min_y <= b.max_y && b.min_y <= a.max_y
}

/// Flat deck cells must not share a cell with pilings, a pier, or a quay block,
/// and each front neighbour (`+col`, `+row`) must be a quay block or deck.
/// Each back neighbour (`-col`, `-row`) must be deck height too, unless
/// `frame` is given and that edge lies wholly outside it.
/// With no frame every back edge counts, which is the production rule.
pub fn validate_flat_deck(
    works: &[HarbourTile],
    deck: &[(i32, i32)],
    frame: Option<&ScreenRect>,
) -> Result<(), Vec<HarbourFault>> {
    let mut faults = Vec::new();
    let quay_at = |col: i32, row: i32| {
        works
            .iter()
            .any(|work| work.kind == Some(WorkKind::Quay) && (work.col, work.row) == (col, row))
    };
    for (i, &(col, row)) in deck.iter().enumerate() {
        if deck[..i].contains(&(col, row)) {
            continue;
        }
        let on_work = works.iter().any(|work| {
            (work.col, work.row) == (col, row)
                && matches!(
                    work.kind,
                    Some(WorkKind::Pilings | WorkKind::Pier | WorkKind::Quay)
                )
        });
        if on_work {
            faults.push(HarbourFault::FlatDeckOnWork { col, row });
        }
        let deck_height = |c: i32, r: i32| quay_at(c, r) || deck.contains(&(c, r));
        if !deck_height(col + 1, row) || !deck_height(col, row + 1) {
            faults.push(HarbourFault::FlatDeckOpenFront { col, row });
        }
        let open_back = flat_deck_back_edges(col, row).iter().any(|((c, r), edge)| {
            !deck_height(*c, *r) && frame.is_none_or(|frame| rects_overlap(edge, frame))
        });
        if open_back {
            faults.push(HarbourFault::FlatDeckOpenBack { col, row });
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

/// Water, then each quay block, the paving on that cell, then later props.
/// Refuses an illegal work layout. No flat deck: see [`build_harbour_with_deck`].
pub fn build_harbour(
    water: Vec<HarbourTile>,
    works: Vec<HarbourTile>,
) -> Result<Vec<HarbourTile>, Vec<HarbourFault>> {
    build_harbour_with_deck(water, works, &[], None)
}

/// [`build_harbour`] plus flat deck cells. Each deck cell gets
/// the same `quay_flag_*` tile a block top would get, placed ahead of the
/// first work whose depth is the same or greater. A repeated deck cell is one
/// plate. Work faults and deck faults are reported together. `frame` is the
/// widest compare view, used only to let a back edge sit off camera; see
/// [`validate_flat_deck`].
pub fn build_harbour_with_deck(
    water: Vec<HarbourTile>,
    mut works: Vec<HarbourTile>,
    deck: &[(i32, i32)],
    frame: Option<&ScreenRect>,
) -> Result<Vec<HarbourTile>, Vec<HarbourFault>> {
    let mut faults = validate_harbour(&works).err().unwrap_or_default();
    faults.extend(
        validate_flat_deck(&works, deck, frame)
            .err()
            .unwrap_or_default(),
    );
    if !faults.is_empty() {
        return Err(faults);
    }
    sort_works(&mut works);
    let mut flat: Vec<(i32, i32)> = Vec::new();
    for cell in deck {
        if !flat.contains(cell) {
            flat.push(*cell);
        }
    }
    flat.sort_by_key(|(col, row)| col + row);
    let mut flat = flat.into_iter().peekable();
    let mut tiles = water;
    for (i, work) in works.iter().enumerate() {
        while let Some(&(col, row)) = flat.peek() {
            if col + row > work.depth() {
                break;
            }
            tiles.push(harbour_quay_paving_tile(col, row).map_err(|fault| vec![fault])?);
            flat.next();
        }
        tiles.push(*work);
        if work.kind != Some(WorkKind::Quay) {
            continue;
        }
        let again = works[i + 1..].iter().any(|other| {
            other.kind == Some(WorkKind::Quay) && other.col == work.col && other.row == work.row
        });
        if !again {
            tiles.push(harbour_quay_paving_tile(work.col, work.row).map_err(|fault| vec![fault])?);
        }
    }
    for (col, row) in flat {
        tiles.push(harbour_quay_paving_tile(col, row).map_err(|fault| vec![fault])?);
    }
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
    fn quay_paving_pins_cells_and_uses_every_flag() {
        // Literals pin the mix. A later edit that reshuffles the hash fails here.
        assert_eq!(crate::assets::quay_flag_variant(0, 1), 2);
        assert_eq!(crate::assets::quay_flag_variant(1, 0), 1);
        assert_eq!(crate::assets::quay_flag_variant(1, 2), 0);
        assert_eq!(crate::assets::quay_flag_variant(2, 3), 1);
        assert_eq!(crate::assets::quay_flag_variant(-1, 4), 1);
        let pinned_again = crate::assets::quay_flag_variant(0, 1);
        assert_eq!(pinned_again, 2);

        let mut seen = [false; 3];
        let mut differs_from_stripe = false;
        for row in 0..8 {
            let mut row_seen = [false; 3];
            for col in 0..8 {
                let variant = crate::assets::quay_flag_variant(col, row);
                assert!(variant < 3);
                seen[variant as usize] = true;
                row_seen[variant as usize] = true;
                differs_from_stripe |= variant != (col + row).rem_euclid(3) as u8;
                let tile =
                    harbour_quay_paving_tile(col, row).unwrap_or_else(|fault| panic!("{fault}"));
                let letter = match variant {
                    0 => "quay_flag_a.png",
                    1 => "quay_flag_b.png",
                    _ => "quay_flag_c.png",
                };
                assert!(
                    tile.path.ends_with(letter),
                    "({col}, {row}) -> {} wanted {letter}",
                    tile.path
                );
                assert_eq!(tile.layer, HarbourLayer::Land);
                assert!(tile.kind.is_none());
            }
            assert!(
                row_seen.iter().filter(|on| **on).count() >= 2,
                "row {row} is a single variant"
            );
        }
        assert_eq!(seen, [true, true, true]);
        assert!(differs_from_stripe);

        for start in 0..5 {
            for diagonal in [
                (0..8 - start)
                    .map(|step| (step, step + start))
                    .collect::<Vec<_>>(),
                (0..8 - start)
                    .map(|step| (step + start, step))
                    .collect::<Vec<_>>(),
            ] {
                let mut kinds = [false; 3];
                for (col, row) in diagonal {
                    kinds[crate::assets::quay_flag_variant(col, row) as usize] = true;
                }
                assert!(
                    kinds.iter().filter(|on| **on).count() >= 2,
                    "diagonal from {start} is a single variant"
                );
            }
        }
        let mut anti = [false; 3];
        for col in 0..8 {
            anti[crate::assets::quay_flag_variant(col, 7 - col) as usize] = true;
        }
        assert!(
            anti.iter().filter(|on| **on).count() >= 2,
            "anti-diagonal is a single variant"
        );

        let seam = harbour_quay_paving_tile(0, 1).unwrap_or_else(|fault| panic!("{fault}"));
        let plate = crate::assets::quay_flag_plate(0, 1);
        assert_eq!(plate.layer, "Land");
        assert_eq!((plate.offset_x, plate.offset_y), (0, 0));
        assert_eq!(plate.y_sort, 0);
        assert_eq!(seam.path, plate.res_path);
        assert!(seam.path.ends_with("ground/quay_flag_c.png"));
        assert!(!seam.path.contains("quay_1111"));
        let (sx, sy) = grid_to_screen(0, 1, HARBOUR_CELL_W, HARBOUR_CELL_H);
        assert_eq!(seam.screen_x, sx);
        assert_eq!(seam.screen_y, sy + HARBOUR_CELL_H / 2);
        assert_ne!(seam.screen_y, harbour_anchor(0, 1).1);
        assert_eq!(
            (seam.anchor_x, seam.anchor_y),
            (plate.anchor_x, plate.anchor_y)
        );
        assert_eq!((seam.canvas_w, seam.canvas_h), (256, 128));

        // One quay block, then a front prop. A second block on the same cell
        // stays in front of paving that was inserted before the block, so
        // `position` on the first block would still pass that revert.
        let built = build_harbour(Vec::new(), vec![quay(0, 1), pier(0, 2)]).expect("legal");
        let paving_at = built
            .iter()
            .position(|tile| tile.layer == HarbourLayer::Land)
            .expect("paving");
        let block_at = built
            .iter()
            .position(|tile| tile.kind == Some(WorkKind::Quay))
            .expect("block");
        let prop_at = built
            .iter()
            .position(|tile| tile.kind == Some(WorkKind::Pier))
            .expect("pier");
        assert_eq!(
            built
                .iter()
                .filter(|tile| tile.kind == Some(WorkKind::Quay))
                .count(),
            1,
            "the draw-order fixture has one quay block"
        );
        assert!(
            block_at < paving_at && paving_at < prop_at,
            "draw order is the block, then paving, then props"
        );

        let doubled = build_harbour(Vec::new(), vec![quay(0, 1), quay(0, 1)]).expect("legal");
        let paving: Vec<_> = doubled
            .iter()
            .filter(|tile| tile.layer == HarbourLayer::Land)
            .collect();
        assert_eq!(paving.len(), 1, "one plate per quay cell");
        assert_eq!((paving[0].col, paving[0].row), (0, 1));
        let last_block = doubled
            .iter()
            .rposition(|tile| tile.kind == Some(WorkKind::Quay))
            .expect("block");
        let doubled_paving = doubled
            .iter()
            .position(|tile| tile.layer == HarbourLayer::Land)
            .expect("paving");
        assert!(
            last_block < doubled_paving,
            "paving follows the last quay block on that cell"
        );
    }

    fn land_at(tiles: &[HarbourTile], col: i32, row: i32) -> Vec<usize> {
        tiles
            .iter()
            .enumerate()
            .filter(|(_, tile)| {
                tile.layer == HarbourLayer::Land && (tile.col, tile.row) == (col, row)
            })
            .map(|(i, _)| i)
            .collect()
    }

    fn work_at(tiles: &[HarbourTile], col: i32, row: i32, kind: WorkKind) -> usize {
        tiles
            .iter()
            .position(|tile| tile.kind == Some(kind) && (tile.col, tile.row) == (col, row))
            .unwrap_or_else(|| panic!("{kind:?} at ({col}, {row})"))
    }

    fn placement(tile: &HarbourTile) -> (i32, i32, &'static str, i32, i32, i32, i32, i32, i32) {
        (
            tile.col,
            tile.row,
            tile.path,
            tile.anchor_x,
            tile.anchor_y,
            tile.canvas_w,
            tile.canvas_h,
            tile.screen_x,
            tile.screen_y,
        )
    }

    /// Quay blocks on all four edge neighbours of a cell.
    fn ring(col: i32, row: i32) -> Vec<HarbourTile> {
        vec![
            quay(col + 1, row),
            quay(col, row + 1),
            quay(col - 1, row),
            quay(col, row - 1),
        ]
    }

    #[test]
    fn flat_deck_reuses_the_block_flag_tile() {
        // Same plate, same Land placement, same variant hash as a block top.
        let built =
            build_harbour_with_deck(Vec::new(), ring(0, 0), &[(0, 0)], None).expect("legal");
        let flat = land_at(&built, 0, 0);
        assert_eq!(flat.len(), 1, "one plate on the flat deck cell");
        let tile = built[flat[0]];
        let block_top = harbour_quay_paving_tile(0, 0).unwrap_or_else(|fault| panic!("{fault}"));
        assert_eq!(placement(&tile), placement(&block_top));
        assert_eq!(tile.layer, HarbourLayer::Land);
        assert!(tile.kind.is_none());
        let plate = crate::assets::quay_flag_plate(0, 0);
        assert_eq!(tile.path, plate.res_path);
        assert!(tile.path.ends_with("ground/quay_flag_a.png"));
        assert!(!tile.path.contains("quay_flat"));
        assert_eq!((tile.anchor_x, tile.anchor_y), (128, 127));
        assert_eq!((tile.canvas_w, tile.canvas_h), (256, 128));
        let (sx, sy) = grid_to_screen(0, 0, HARBOUR_CELL_W, HARBOUR_CELL_H);
        assert_eq!(
            (tile.screen_x, tile.screen_y),
            (sx, sy + HARBOUR_CELL_H / 2)
        );
        assert_ne!(
            tile.screen_y,
            harbour_anchor(0, 0).1,
            "no sea datum on land"
        );

        // Every cell of a kerbed deck patch picks the plate the hash picks.
        let mut works = Vec::new();
        let mut deck = Vec::new();
        for i in 0..6 {
            works.push(quay(i, 6));
            works.push(quay(6, i));
            works.push(quay(i, -1));
            works.push(quay(-1, i));
        }
        for row in 0..6 {
            for col in 0..6 {
                deck.push((col, row));
            }
        }
        let patch = build_harbour_with_deck(Vec::new(), works, &deck, None).expect("legal patch");
        let mut seen = [false; 3];
        for &(col, row) in &deck {
            let at = land_at(&patch, col, row);
            assert_eq!(at.len(), 1, "({col}, {row})");
            let want = harbour_quay_paving_tile(col, row).unwrap_or_else(|fault| panic!("{fault}"));
            assert_eq!(placement(&patch[at[0]]), placement(&want));
            seen[crate::assets::quay_flag_variant(col, row) as usize] = true;
        }
        assert_eq!(seen, [true, true, true], "flat deck shows a, b and c");
    }

    #[test]
    fn flat_deck_sorts_after_back_works_and_before_front_works() {
        let built = build_harbour_with_deck(
            Vec::new(),
            vec![
                pier(0, 2),
                quay(1, 0),
                pilings(1, -1),
                quay(0, 1),
                quay(-1, 0),
                quay(0, -1),
            ],
            &[(0, 0)],
            None,
        )
        .expect("legal");
        let flat = land_at(&built, 0, 0)[0];
        let back_block = work_at(&built, -1, 0, WorkKind::Quay);
        let back_paving = land_at(&built, -1, 0)[0];
        let side = work_at(&built, 1, -1, WorkKind::Pilings);
        let front_right = work_at(&built, 1, 0, WorkKind::Quay);
        let front_left = work_at(&built, 0, 1, WorkKind::Quay);
        let prop = work_at(&built, 0, 2, WorkKind::Pier);
        assert!(
            back_block < back_paving && back_paving < flat,
            "a back block's face sits under the deck"
        );
        assert!(flat < side, "same depth: the deck goes first");
        assert!(
            flat < front_right && flat < front_left,
            "front blocks draw over the deck edge"
        );
        assert!(flat < prop, "props draw above the deck flag");

        // Raised-block placement is unchanged: each block, then its paving.
        for (col, row) in [(-1, 0), (0, -1), (1, 0), (0, 1)] {
            let block = work_at(&built, col, row, WorkKind::Quay);
            let paving = land_at(&built, col, row);
            assert_eq!(
                paving,
                vec![block + 1],
                "paving follows block ({col}, {row})"
            );
        }
        assert_eq!(
            built
                .iter()
                .filter(|tile| tile.layer == HarbourLayer::Land)
                .count(),
            5,
            "four block tops and one flat deck"
        );
    }

    #[test]
    fn flat_deck_needs_deck_height_in_front_and_no_work() {
        let alone = build_harbour_with_deck(Vec::new(), Vec::new(), &[(0, 0)], None).unwrap_err();
        assert_eq!(
            alone,
            vec![
                HarbourFault::FlatDeckOpenFront { col: 0, row: 0 },
                HarbourFault::FlatDeckOpenBack { col: 0, row: 0 },
            ]
        );
        let backs = [quay(-1, 0), quay(0, -1)];
        let half =
            validate_flat_deck(&[&backs[..], &[quay(0, 1)]].concat(), &[(0, 0)], None).unwrap_err();
        assert_eq!(
            half,
            vec![HarbourFault::FlatDeckOpenFront { col: 0, row: 0 }]
        );
        let over_pier = validate_flat_deck(
            &[&backs[..], &[quay(1, 0), pier(0, 1)]].concat(),
            &[(0, 0)],
            None,
        )
        .unwrap_err();
        assert_eq!(
            over_pier,
            vec![HarbourFault::FlatDeckOpenFront { col: 0, row: 0 }]
        );
        // Deck behind deck, kerbed all round, is legal.
        assert!(validate_flat_deck(
            &[
                quay(2, 0),
                quay(0, 1),
                quay(1, 1),
                quay(-1, 0),
                quay(0, -1),
                quay(1, -1),
            ],
            &[(0, 0), (1, 0)],
            None,
        )
        .is_ok());

        for work in [quay(0, 0), pier(0, 0), pilings(0, 0)] {
            let mut works = ring(0, 0);
            works.push(work);
            let fault = validate_flat_deck(&works, &[(0, 0)], None).unwrap_err();
            assert_eq!(fault, vec![HarbourFault::FlatDeckOnWork { col: 0, row: 0 }]);
        }

        // Work faults come first, then deck faults, in one list.
        let both =
            build_harbour_with_deck(Vec::new(), vec![pilings(3, 3), pier(3, 3)], &[(0, 0)], None)
                .unwrap_err();
        assert_eq!(
            both,
            vec![
                HarbourFault::PilingsOnPier { col: 3, row: 3 },
                HarbourFault::FlatDeckOpenFront { col: 0, row: 0 },
                HarbourFault::FlatDeckOpenBack { col: 0, row: 0 },
            ]
        );
        assert_eq!(
            HarbourFault::FlatDeckOnWork { col: 0, row: 0 }.to_string(),
            "flat deck on a work cell (0, 0)"
        );
        assert_eq!(
            alone[0].to_string(),
            "flat deck (0, 0) has an open front over water"
        );
    }

    #[test]
    fn flat_deck_back_edge_is_deck_height_or_off_frame() {
        // No paper-thin deck rim against open water.
        let edges = flat_deck_back_edges(0, 0);
        assert_eq!(edges[0].0, (-1, 0), "upper-left edge faces -col");
        assert_eq!(edges[1].0, (0, -1), "upper-right edge faces -row");
        let pad = FLAT_EDGE_PAD_PX;
        let ul = edges[0].1;
        let ur = edges[1].1;
        assert_eq!(
            (ul.min_x, ul.min_y, ul.max_x, ul.max_y),
            (-128 - pad, -64 - pad, pad, pad)
        );
        assert_eq!(
            (ur.min_x, ur.min_y, ur.max_x, ur.max_y),
            (-pad, -64 - pad, 128 + pad, pad)
        );

        let fronts = vec![quay(1, 0), quay(0, 1)];
        let open = HarbourFault::FlatDeckOpenBack { col: 0, row: 0 };
        // Production rule: with no frame every back edge counts.
        assert_eq!(
            validate_flat_deck(&fronts, &[(0, 0)], None),
            Err(vec![open])
        );
        // One back kerbed is not enough.
        let one = [&fronts[..], &[quay(-1, 0)]].concat();
        assert_eq!(validate_flat_deck(&one, &[(0, 0)], None), Err(vec![open]));
        // Both backs at deck height: legal with or without a frame.
        assert!(validate_flat_deck(&ring(0, 0), &[(0, 0)], None).is_ok());

        let rect = |min_x: i32, min_y: i32, max_x: i32, max_y: i32| ScreenRect {
            min_x,
            min_y,
            max_x,
            max_y,
        };
        // A frame that sees the cell sees its open back edges.
        let wide = rect(-500, -500, 500, 500);
        assert_eq!(
            validate_flat_deck(&fronts, &[(0, 0)], Some(&wide)),
            Err(vec![open])
        );
        // A frame that touches only the upper-left edge box still fails.
        let corner = rect(-200, -40, -100, -20);
        assert_eq!(
            validate_flat_deck(&fronts, &[(0, 0)], Some(&corner)),
            Err(vec![open])
        );
        // A frame that sees only the lower half of the diamond, clear of
        // both back edges, passes: the open backs are off camera.
        let low = rect(-10, 20, 10, 60);
        assert!(validate_flat_deck(&fronts, &[(0, 0)], Some(&low)).is_ok());
        let far = rect(5000, 5000, 6000, 6000);
        assert!(validate_flat_deck(&fronts, &[(0, 0)], Some(&far)).is_ok());
        // Off camera never excuses a front edge.
        assert_eq!(
            validate_flat_deck(&[], &[(0, 0)], Some(&far)),
            Err(vec![HarbourFault::FlatDeckOpenFront { col: 0, row: 0 }])
        );
        assert_eq!(
            open.to_string(),
            "flat deck (0, 0) shows an open back edge against water"
        );
    }

    #[test]
    fn a_repeated_deck_cell_is_one_plate_and_no_deck_is_unchanged() {
        let works = vec![quay(1, 0), quay(0, 1), quay(-1, 0), quay(0, -1), pier(0, 2)];
        let twice = build_harbour_with_deck(Vec::new(), works.clone(), &[(0, 0), (0, 0)], None)
            .expect("legal");
        assert_eq!(land_at(&twice, 0, 0).len(), 1);
        let plain = build_harbour(Vec::new(), works.clone()).expect("legal");
        let empty = build_harbour_with_deck(Vec::new(), works, &[], None).expect("legal");
        assert_eq!(plain.len(), empty.len());
        for (a, b) in plain.iter().zip(&empty) {
            assert_eq!(placement(a), placement(b));
            assert_eq!((a.layer, a.kind), (b.layer, b.kind));
        }
        assert_eq!(twice.len(), plain.len() + 1);
    }

    #[test]
    fn a_quay_flag_off_u4_is_a_fault() {
        let plate = crate::assets::QuayFlagPlate {
            id: "quay_flag_a",
            res_path: "res://assets/landing/ground/quay_flag_a.png",
            anchor_x: 128,
            anchor_y: 127,
            canvas_w: 256,
            canvas_h: 128,
            layer: "Water",
            offset_x: 4,
            offset_y: -8,
            y_sort: 12,
        };
        let fault = quay_flag_u4(&plate).expect_err("off the quay paving rule");
        assert_eq!(
            fault.to_string(),
            "quay_flag_a must be layer Land, offset 0,0, y_sort 0 (quay paving rule); \
             MANIFEST has layer Water, offset 4,-8, y_sort 12"
        );
        assert!(quay_flag_u4(crate::assets::quay_flag_plate(1, 2)).is_ok());
        assert!(quay_flag_u4(crate::assets::quay_flag_plate(1, 0)).is_ok());
        assert!(quay_flag_u4(crate::assets::quay_flag_plate(0, 1)).is_ok());
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

    fn prop(col: i32, row: i32, path: &'static str) -> HarbourTile {
        harbour_prop_tile(col, row, path)
    }

    #[test]
    fn deck_props_use_land_offset_zero() {
        let tile = harbour_prop_tile(0, 2, "res://assets/landing/props/bollard_1x1/beauty.png");
        let (sx, sy) = grid_to_screen(0, 2, HARBOUR_CELL_W, HARBOUR_CELL_H);
        assert_eq!(tile.layer, HarbourLayer::Prop);
        assert_eq!(tile.kind, Some(WorkKind::Prop));
        assert_eq!(tile.screen_x, sx);
        assert_eq!(tile.screen_y, sy + HARBOUR_CELL_H / 2);
        assert_ne!(tile.screen_y, harbour_anchor(0, 2).1);
        assert_eq!((tile.anchor_x, tile.anchor_y), (128, 255));
    }

    #[test]
    fn build_order_is_block_paving_then_prop() {
        let built = build_harbour(
            Vec::new(),
            vec![
                quay(0, 1),
                prop(0, 1, "res://assets/landing/props/barrel_1x1/beauty.png"),
                pier(0, 2),
            ],
        )
        .expect("legal");
        let block_at = built
            .iter()
            .position(|tile| tile.kind == Some(WorkKind::Quay))
            .expect("block");
        let paving_at = built
            .iter()
            .position(|tile| tile.layer == HarbourLayer::Land)
            .expect("paving");
        let prop_at = built
            .iter()
            .position(|tile| tile.kind == Some(WorkKind::Prop))
            .expect("prop");
        assert!(block_at < paving_at && paving_at < prop_at);
    }

    #[test]
    fn pier_head_allows_bollard_and_torch_only() {
        let ok = validate_harbour(&[
            pier(0, 2),
            prop(0, 2, "res://assets/landing/props/bollard_1x1/beauty.png"),
            prop(0, 2, "res://assets/landing/props/torch_1x1/beauty.png"),
        ]);
        assert!(ok.is_ok());
        let too_many = validate_harbour(&[
            pier(0, 2),
            prop(0, 2, "res://assets/landing/props/bollard_1x1/beauty.png"),
            prop(0, 2, "res://assets/landing/props/torch_1x1/beauty.png"),
            prop(0, 2, "res://assets/landing/props/barrel_1x1/beauty.png"),
        ])
        .unwrap_err();
        assert_eq!(
            too_many,
            vec![HarbourFault::PropDensity {
                col: 0,
                row: 2,
                count: 3
            }]
        );
        let not_pair = validate_harbour(&[
            quay(0, 1),
            prop(0, 1, "res://assets/landing/props/barrel_1x1/beauty.png"),
            prop(0, 1, "res://assets/landing/props/crate_1x1/beauty.png"),
        ])
        .unwrap_err();
        assert_eq!(
            not_pair,
            vec![HarbourFault::PropDensity {
                col: 0,
                row: 1,
                count: 2
            }]
        );
    }
}
