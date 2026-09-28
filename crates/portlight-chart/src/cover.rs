//! Water must run past the viewport. The plates feather 8 px at each tip, and
//! that fringe reads as a notch against the clear colour when an open edge
//! is on screen.

#[cfg(test)]
use std::collections::HashSet;

use crate::project::ScreenRect;

/// Designed tip feather on the water plates.
#[cfg(test)]
pub const TIP_FEATHER_PX: f32 = 8.0;

/// Extra cells past the view's corner cells. One cell already clears the
/// feather; the second keeps a sample-grid gap from landing on the fringe.
pub const WATER_COVER_PAD: i32 = 2;

#[cfg(test)]
const SAMPLE_STEP: i32 = 8;
/// Half the sample diagonal, rounded up, so a gap between samples still
/// clears [`TIP_FEATHER_PX`].
#[cfg(test)]
const SAMPLE_SLACK: f32 = 6.0;

/// World rectangle the camera shows. Zoom is the Godot `Camera2D` factor:
/// `0.72` shows more world than `1.0`.
pub fn view_world_rect(
    center_x: f32,
    center_y: f32,
    zoom: f32,
    view_w: f32,
    view_h: f32,
) -> ScreenRect {
    let half_w = view_w / zoom / 2.0;
    let half_h = view_h / zoom / 2.0;
    ScreenRect {
        min_x: (center_x - half_w).floor() as i32,
        min_y: (center_y - half_h).floor() as i32,
        max_x: (center_x + half_w).ceil() as i32,
        max_y: (center_y + half_h).ceil() as i32,
    }
}

/// Pixels of diamond left between `point` and the edge. Negative is outside.
#[cfg(test)]
pub fn diamond_clearance(center: (f32, f32), half_w: f32, half_h: f32, point: (f32, f32)) -> f32 {
    let dx = (point.0 - center.0).abs();
    let dy = (point.1 - center.1).abs();
    let depth = dx / half_w + dy / half_h;
    let grad = (1.0 / (half_w * half_w) + 1.0 / (half_h * half_h)).sqrt();
    (1.0 - depth) / grad
}

/// Every cell in the corner range of `view`, plus `pad` cells on each side.
/// `cell_at` returns fractional `(col, row)` of a screen point.
pub fn cells_covering(
    view: &ScreenRect,
    pad: i32,
    cell_at: impl Fn(f32, f32) -> (f32, f32),
) -> Vec<(i32, i32)> {
    let corners = [
        (view.min_x as f32, view.min_y as f32),
        (view.max_x as f32, view.min_y as f32),
        (view.min_x as f32, view.max_y as f32),
        (view.max_x as f32, view.max_y as f32),
    ];
    let mut min_c = i32::MAX;
    let mut max_c = i32::MIN;
    let mut min_r = i32::MAX;
    let mut max_r = i32::MIN;
    for (x, y) in corners {
        let (col, row) = cell_at(x, y);
        min_c = min_c.min(col.floor() as i32);
        max_c = max_c.max(col.ceil() as i32);
        min_r = min_r.min(row.floor() as i32);
        max_r = max_r.max(row.ceil() as i32);
    }
    min_c -= pad;
    max_c += pad;
    min_r -= pad;
    max_r += pad;
    let mut cells = Vec::with_capacity(((max_c - min_c + 1) * (max_r - min_r + 1)) as usize);
    for col in min_c..=max_c {
        for row in min_r..=max_r {
            cells.push((col, row));
        }
    }
    cells
}

/// `true` when `view` sits at least `feather` px inside the water, so an open
/// edge cannot cross the screen. Neighbouring diamonds share edges, so the
/// check inflates the view and asks that the larger rect still lie inside
/// some diamond. A point near one plate's tip is covered by the next plate.
#[cfg(test)]
pub fn grid_covers_view(
    cells: &[(i32, i32)],
    view: &ScreenRect,
    half_w: f32,
    half_h: f32,
    feather: f32,
    center_of: impl Fn(i32, i32) -> (f32, f32),
    cell_at: impl Fn(f32, f32) -> (i32, i32),
) -> bool {
    let set: HashSet<(i32, i32)> = cells.iter().copied().collect();
    let margin = (feather + SAMPLE_SLACK).ceil() as i32;
    let padded = view.pad(margin, margin);
    for y in axis_samples(padded.min_y, padded.max_y) {
        for x in axis_samples(padded.min_x, padded.max_x) {
            if !point_inside(
                &set, x as f32, y as f32, half_w, half_h, &center_of, &cell_at,
            ) {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
fn axis_samples(min: i32, max: i32) -> Vec<i32> {
    let mut samples = Vec::new();
    let mut cursor = min;
    while cursor < max {
        samples.push(cursor);
        cursor += SAMPLE_STEP;
    }
    samples.push(max);
    samples
}

#[cfg(test)]
fn point_inside(
    cells: &HashSet<(i32, i32)>,
    x: f32,
    y: f32,
    half_w: f32,
    half_h: f32,
    center_of: &impl Fn(i32, i32) -> (f32, f32),
    cell_at: &impl Fn(f32, f32) -> (i32, i32),
) -> bool {
    let (col, row) = cell_at(x, y);
    for d_col in -2..=2 {
        for d_row in -2..=2 {
            let cell = (col + d_col, row + d_row);
            if !cells.contains(&cell) {
                continue;
            }
            // On the shared edge of two plates the clearance is ~0. The
            // inflated view is what keeps the open outer edge off screen.
            if diamond_clearance(center_of(cell.0, cell.1), half_w, half_h, (x, y)) >= -0.5 {
                return true;
            }
        }
    }
    false
}
