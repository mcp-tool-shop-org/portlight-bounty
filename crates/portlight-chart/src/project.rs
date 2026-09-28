//! Dimetric 2:1 projection and ship facing.
//!
//! Chart coordinates stay on the sim grid. Display applies the operator
//! rotation, then the 2:1 scale (`asset-spec` §2, which replaces the
//! unrotated A1 formula):
//!
//! ```text
//! u = (x + y) / √2
//! v = (y − x) / √2
//! screen_x = (u − v) · 64
//! screen_y = (u + v) · 32
//! ```
//!
//! That is `screen_x = x · 64√2` and `screen_y = y · 32√2`. Water tiles sit
//! on integer `(u, v)`. Ports and ships sit at the fractional `(u, v)` of
//! their chart point. Facing is bucketed in `(u, v)`, not in raw map units.

/// Chart cell width in pixels. The ground diamond is twice as wide as it is tall.
pub const CELL_WIDTH: i32 = 128;
/// Chart cell height in pixels (dimetric 2:1).
pub const CELL_HEIGHT: i32 = 64;
/// Harbour water cell. Twice the chart cell, still 2:1.
pub const HARBOUR_CELL_W: i32 = 256;
/// Harbour water cell height in pixels.
pub const HARBOUR_CELL_H: i32 = 128;

/// Harbour sea, water, quay, and pier sit at this screen offset.
///
/// Godot Y grows downward, so the sea layer is placed at `+48` to land on
/// datum −48. Chart water is a different scale and stays at layer offset 0.
pub const WATER_DATUM_Y: i32 = -48;

/// Docked ship, applied after projection. Midpoint of the anchor cell's
/// bottom-right edge. The contact Y is 16 px below the port, so the ship
/// sorts in front of that port.
pub const DOCKED_OFFSET_X: f32 = 32.0;
pub const DOCKED_OFFSET_Y: f32 = 16.0;

/// Extra degrees past a 22.5° bucket edge before the bow changes.
pub const FACING_HYSTERESIS_DEG: f64 = 5.0;

/// Eight headings in rotated display `(u, v)`.
///
/// Index 0 is +u (screen down-right). Yaw increases toward +v
/// (screen down-left). `F1` is screen down, the facing before the first voyage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Facing {
    F0 = 0,
    F1 = 1,
    F2 = 2,
    F3 = 3,
    F4 = 4,
    F5 = 5,
    F6 = 6,
    F7 = 7,
}

impl Facing {
    pub fn index(self) -> i32 {
        self as i32
    }

    pub fn from_index(index: i32) -> Self {
        match index.rem_euclid(8) {
            0 => Self::F0,
            1 => Self::F1,
            2 => Self::F2,
            3 => Self::F3,
            4 => Self::F4,
            5 => Self::F5,
            6 => Self::F6,
            _ => Self::F7,
        }
    }

    /// `f0`..`f7`, the frame order on one canvas and one anchor pixel.
    pub fn asset_suffix(self) -> &'static str {
        match self {
            Self::F0 => "f0",
            Self::F1 => "f1",
            Self::F2 => "f2",
            Self::F3 => "f3",
            Self::F4 => "f4",
            Self::F5 => "f5",
            Self::F6 => "f6",
            Self::F7 => "f7",
        }
    }
}

/// Rotate a chart point into display-cart `(u, v)`.
pub fn chart_to_uv(x: f64, y: f64) -> (f64, f64) {
    let s = std::f64::consts::SQRT_2;
    ((x + y) / s, (y - x) / s)
}

/// Inverse of [`chart_to_uv`].
pub fn uv_to_chart(u: f64, v: f64) -> (f64, f64) {
    let s = std::f64::consts::SQRT_2;
    ((u - v) / s, (u + v) / s)
}

/// Dimetric grid shared by the chart and the harbour.
///
/// `sx = (col − row) * cell_w / 2`, `sy = (col + row) * cell_h / 2`.
/// A 256×128 harbour cell is `(col − row) * 128`, `(col + row) * 64`.
/// A 128×64 chart cell is the same function at half scale. The value is the
/// cell centre. The footprint anchor sits half a cell below that.
pub fn grid_to_screen_f(col: f64, row: f64, cell_w: i32, cell_h: i32) -> (f64, f64) {
    (
        (col - row) * f64::from(cell_w / 2),
        (col + row) * f64::from(cell_h / 2),
    )
}

/// Integer form of [`grid_to_screen_f`].
pub fn grid_to_screen(col: i32, row: i32, cell_w: i32, cell_h: i32) -> (i32, i32) {
    let (x, y) = grid_to_screen_f(f64::from(col), f64::from(row), cell_w, cell_h);
    (x as i32, y as i32)
}

/// 2:1 scale of a display-cart point. Integer `(u, v)` is a water-cell centre.
pub fn uv_to_screen(u: f64, v: f64) -> (f64, f64) {
    grid_to_screen_f(u, v, CELL_WIDTH, CELL_HEIGHT)
}

/// Screen position of a chart point. Porto Novo `(18, 8)` is about `(1629.2, 362.0)`.
pub fn chart_to_screen_f(x: f64, y: f64) -> (f32, f32) {
    let (u, v) = chart_to_uv(x, y);
    let (sx, sy) = uv_to_screen(u, v);
    (sx as f32, sy as f32)
}

/// Rounded screen position, for camera bounds.
pub fn chart_to_screen(x: i64, y: i64) -> (i32, i32) {
    let (sx, sy) = chart_to_screen_f(x as f64, y as f64);
    (sx.round() as i32, sy.round() as i32)
}

/// Centre of the water cell at integer display-cart `(u, v)`.
pub fn water_cell_center(u: i32, v: i32) -> (f32, f32) {
    let (sx, sy) = grid_to_screen(u, v, CELL_WIDTH, CELL_HEIGHT);
    (sx as f32, sy as f32)
}

/// Footprint bottom vertex of that water cell (half a cell below the centre).
pub fn water_cell_bottom(u: i32, v: i32) -> (f32, f32) {
    let center = water_cell_center(u, v);
    (center.0, center.1 + (CELL_HEIGHT / 2) as f32)
}

/// Top-left of a sprite whose anchor pixel should land on `at`.
pub fn sprite_origin(at: (f32, f32), anchor_x: i32, anchor_y: i32) -> (f32, f32) {
    (at.0 - anchor_x as f32, at.1 - anchor_y as f32)
}

/// Screen step of a `(u, v)` delta, used to point a placeholder bow.
pub fn uv_to_screen_delta(du: f64, dv: f64) -> (f64, f64) {
    grid_to_screen_f(du, dv, CELL_WIDTH, CELL_HEIGHT)
}

/// Facing for a chart step, measured in rotated `(u, v)`.
///
/// A zero step keeps `previous`, or `F1` (screen down) when there is no
/// previous voyage. Bucket edges hold the previous facing for
/// [`FACING_HYSTERESIS_DEG`] past 22.5°.
pub fn facing_from_chart_delta(dx: f64, dy: f64, previous: Option<Facing>) -> Facing {
    let (du, dv) = chart_to_uv(dx, dy);
    // `chart_to_uv` of a delta is the delta of `chart_to_uv` (the map is linear).
    facing_from_uv(du, dv, previous)
}

/// `k = round(atan2(dv, du) / 45°) mod 8`, with hysteresis.
pub fn facing_from_uv(du: f64, dv: f64, previous: Option<Facing>) -> Facing {
    if du == 0.0 && dv == 0.0 {
        return previous.unwrap_or(Facing::F1);
    }
    let degrees = dv.atan2(du).to_degrees();
    let raw = Facing::from_index((degrees / 45.0).round() as i32);
    let Some(previous) = previous else {
        return raw;
    };
    let mut delta = degrees - f64::from(previous.index()) * 45.0;
    if delta > 180.0 {
        delta -= 360.0;
    } else if delta < -180.0 {
        delta += 360.0;
    }
    if delta.abs() <= 22.5 + FACING_HYSTERESIS_DEG {
        previous
    } else {
        raw
    }
}

/// Inclusive screen rectangle used to frame the camera.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenRect {
    pub min_x: i32,
    pub min_y: i32,
    pub max_x: i32,
    pub max_y: i32,
}

impl ScreenRect {
    pub fn from_point(x: i32, y: i32) -> Self {
        Self {
            min_x: x,
            min_y: y,
            max_x: x,
            max_y: y,
        }
    }

    pub fn include(&mut self, x: i32, y: i32) {
        self.min_x = self.min_x.min(x);
        self.min_y = self.min_y.min(y);
        self.max_x = self.max_x.max(x);
        self.max_y = self.max_y.max(y);
    }

    pub fn pad(mut self, px: i32, py: i32) -> Self {
        self.min_x -= px;
        self.min_y -= py;
        self.max_x += px;
        self.max_y += py;
        self
    }

    pub fn contains(self, x: i32, y: i32) -> bool {
        x >= self.min_x && x <= self.max_x && y >= self.min_y && y <= self.max_y
    }

    pub fn width(self) -> i32 {
        self.max_x - self.min_x
    }

    pub fn height(self) -> i32 {
        self.max_y - self.min_y
    }
}

/// Screen bounds of the 50×36 chart, from cell `(0, 0)` through `(49, 35)`.
pub fn chart_bounds() -> ScreenRect {
    let mut rect = ScreenRect::from_point(0, 0);
    for (x, y) in [(49, 0), (0, 35), (49, 35)] {
        let (sx, sy) = chart_to_screen(x, y);
        rect.include(sx, sy);
    }
    rect
}

/// Camera frame: center of [`ScreenRect`] and a uniform zoom that fits it
/// in a viewport. Zoom is clamped so a single port does not fill the window.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    pub center_x: f32,
    pub center_y: f32,
    pub zoom: f32,
}

pub fn frame_to_view(rect: ScreenRect, view_w: f32, view_h: f32) -> Frame {
    let span_w = (rect.width().max(1)) as f32;
    let span_h = (rect.height().max(1)) as f32;
    let zoom = (view_w / span_w).min(view_h / span_h).clamp(0.2, 2.0);
    Frame {
        center_x: (rect.min_x + rect.max_x) as f32 / 2.0,
        center_y: (rect.min_y + rect.max_y) as f32 / 2.0,
        zoom,
    }
}

/// At sea the camera center follows the ship at the region's zoom, and stays
/// inside the chart. Docked, the region frame is used as-is.
pub fn follow_ship(
    region: Frame,
    ship: (f32, f32),
    at_sea: bool,
    view_w: f32,
    view_h: f32,
) -> Frame {
    if !at_sea {
        return region;
    }
    let bounds = chart_bounds();
    let half_w = view_w / region.zoom / 2.0;
    let half_h = view_h / region.zoom / 2.0;
    let min_x = bounds.min_x as f32 + half_w;
    let max_x = bounds.max_x as f32 - half_w;
    let min_y = bounds.min_y as f32 + half_h;
    let max_y = bounds.max_y as f32 - half_h;
    let center_x = if min_x <= max_x {
        ship.0.clamp(min_x, max_x)
    } else {
        (bounds.min_x + bounds.max_x) as f32 / 2.0
    };
    let center_y = if min_y <= max_y {
        ship.1.clamp(min_y, max_y)
    } else {
        (bounds.min_y + bounds.max_y) as f32 / 2.0
    };
    Frame {
        center_x,
        center_y,
        zoom: region.zoom,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn porto_novo_uses_the_rotated_projection() {
        let (sx, sy) = chart_to_screen_f(18.0, 8.0);
        assert!((sx - 1629.2).abs() < 0.15, "{sx}");
        assert!((sy - 362.0).abs() < 0.15, "{sy}");
        assert_eq!(CELL_WIDTH, CELL_HEIGHT * 2);
    }

    #[test]
    fn projection_is_dimetric_two_to_one_in_uv() {
        let (sx, sy) = uv_to_screen(1.0, 0.0);
        assert!((sx - 64.0).abs() < 1e-9);
        assert!((sy - 32.0).abs() < 1e-9);
        let (sx, sy) = uv_to_screen(0.0, 1.0);
        assert!((sx - -64.0).abs() < 1e-9);
        assert!((sy - 32.0).abs() < 1e-9);
        let (sx, sy) = uv_to_screen(1.0, 1.0);
        assert!(sx.abs() < 1e-9);
        assert!((sy - 64.0).abs() < 1e-9);
    }

    #[test]
    fn rotation_round_trips_every_chart_cell() {
        for x in 0..50 {
            for y in 0..36 {
                let (u, v) = chart_to_uv(x as f64, y as f64);
                let (x2, y2) = uv_to_chart(u, v);
                assert!((x2 - x as f64).abs() < 1e-9);
                assert!((y2 - y as f64).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn harbour_datum_is_minus_48_and_chart_water_is_not_shifted() {
        assert_eq!(WATER_DATUM_Y, -48);
        // Chart water layer offset is 0. The +48 shift is the harbour sea plane.
        assert_eq!(-WATER_DATUM_Y, 48);
    }

    #[test]
    fn grain_road_faces_bucket_seven() {
        let (du, dv) = chart_to_uv(6.0, -2.0);
        assert!((du - 2.828).abs() < 0.01, "{du}");
        assert!((dv - -5.657).abs() < 0.01, "{dv}");
        assert_eq!(facing_from_uv(du, dv, None), Facing::F7);
        let (sx, sy) = uv_to_screen_delta(du, dv);
        assert!((sx - 543.1).abs() < 0.2, "{sx}");
        assert!((sy - -90.5).abs() < 0.2, "{sy}");
        let angle = sy.atan2(sx).to_degrees();
        assert!((angle - -9.46).abs() < 0.15, "{angle}");
    }

    #[test]
    fn zero_step_faces_screen_down_until_a_voyage() {
        assert_eq!(facing_from_uv(0.0, 0.0, None), Facing::F1);
        assert_eq!(facing_from_uv(0.0, 0.0, Some(Facing::F7)), Facing::F7);
    }

    #[test]
    fn hysteresis_holds_five_degrees_past_the_bucket_edge() {
        // k=0 is centered at 0°. The edge is 22.5°. +5° keeps k=0 through 27°.
        let hold = 26.0_f64.to_radians();
        assert_eq!(
            facing_from_uv(hold.cos(), hold.sin(), Some(Facing::F0)),
            Facing::F0
        );
        let switch = 28.0_f64.to_radians();
        assert_eq!(
            facing_from_uv(switch.cos(), switch.sin(), Some(Facing::F0)),
            Facing::F1
        );
        let fresh = 10.0_f64.to_radians();
        assert_eq!(facing_from_uv(fresh.cos(), fresh.sin(), None), Facing::F0);
    }

    #[test]
    fn eight_uv_axes_cover_eight_facings() {
        for index in 0..8 {
            let angle = (index as f64) * 45.0_f64.to_radians();
            assert_eq!(
                facing_from_uv(angle.cos(), angle.sin(), None),
                Facing::from_index(index),
                "sector {index}"
            );
        }
    }

    #[test]
    fn frame_fits_the_viewport() {
        let rect = ScreenRect {
            min_x: 0,
            min_y: 0,
            max_x: 900,
            max_y: 300,
        };
        let frame = frame_to_view(rect, 900.0, 720.0);
        assert!((frame.zoom - 1.0).abs() < 1e-4);
        assert!((frame.center_x - 450.0).abs() < 1e-4);
        assert!((frame.center_y - 150.0).abs() < 1e-4);
    }

    #[test]
    fn follow_clamps_inside_the_chart() {
        let region = Frame {
            center_x: 0.0,
            center_y: 0.0,
            zoom: 1.0,
        };
        let frame = follow_ship(region, (10_000.0, 10_000.0), true, 900.0, 720.0);
        let bounds = chart_bounds();
        assert!(frame.center_x < bounds.max_x as f32);
        assert!(frame.center_y < bounds.max_y as f32);
        assert!((frame.zoom - 1.0).abs() < 1e-6);
    }
}
