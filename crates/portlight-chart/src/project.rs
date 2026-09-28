//! Dimetric 2:1 projection and ship facing.
//!
//! Chart cell `(x, y)` is rotated into display axes, then scaled:
//!
//! ```text
//! u = x - y
//! v = x + y
//! screen_x = u * (cell_width / 2)     // 64
//! screen_y = v * (cell_height / 2)    // 32
//! ```
//!
//! `u` is the horizontal display axis and `v` is the vertical one, before
//! the 2:1 pixel scale. Ship facing is the screen-space angle of `(u, v)`,
//! split into eight 45° sectors. Simulation coordinates are not modified.

/// Chart cell width in pixels. The ground diamond is twice as wide as it is tall.
pub const CELL_WIDTH: i32 = 128;
/// Chart cell height in pixels (dimetric 2:1).
pub const CELL_HEIGHT: i32 = 64;

/// Art-director datum for sea, water, quay, and pier.
///
/// The sprite origin is this many screen pixels from the cell anchor
/// (Godot Y grows downward, so the origin sits above the anchor). The
/// pixel at local `(SIT_X, SIT_Y)` then lands on the anchor.
pub const WATER_DATUM_Y: i32 = -48;

/// Local X of the sit-point inside a placeholder canvas.
pub const SIT_X: i32 = CELL_WIDTH / 2;
/// Local Y of the sit-point. Equals `-WATER_DATUM_Y`.
pub const SIT_Y: i32 = -WATER_DATUM_Y;

/// Eight headings in screen space. Index 0 is east, then clockwise,
/// because display Y grows downward.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Facing {
    E = 0,
    Se = 1,
    S = 2,
    Sw = 3,
    W = 4,
    Nw = 5,
    N = 6,
    Ne = 7,
}

impl Facing {
    pub fn index(self) -> i32 {
        self as i32
    }

    pub fn from_index(index: i32) -> Self {
        match index.rem_euclid(8) {
            0 => Self::E,
            1 => Self::Se,
            2 => Self::S,
            3 => Self::Sw,
            4 => Self::W,
            5 => Self::Nw,
            6 => Self::N,
            _ => Self::Ne,
        }
    }

    pub fn asset_suffix(self) -> &'static str {
        match self {
            Self::E => "e",
            Self::Se => "se",
            Self::S => "s",
            Self::Sw => "sw",
            Self::W => "w",
            Self::Nw => "nw",
            Self::N => "n",
            Self::Ne => "ne",
        }
    }
}

/// Rotate a chart cell into unscaled display axes `(u, v)`.
pub fn rotate_chart(x: i64, y: i64) -> (i64, i64) {
    (x - y, x + y)
}

/// Screen position of a chart cell's anchor (center of the ground diamond).
pub fn chart_to_screen(x: i64, y: i64) -> (i32, i32) {
    let (u, v) = rotate_chart(x, y);
    (
        (u * i64::from(CELL_WIDTH / 2)) as i32,
        (v * i64::from(CELL_HEIGHT / 2)) as i32,
    )
}

/// Same projection for a point that falls between cells (ship along a lane).
pub fn chart_to_screen_f(x: f64, y: f64) -> (f32, f32) {
    let u = x - y;
    let v = x + y;
    (
        (u * f64::from(CELL_WIDTH / 2)) as f32,
        (v * f64::from(CELL_HEIGHT / 2)) as f32,
    )
}

/// Top-left of a sprite whose sit-point is `(sit_x, sit_y)` in the canvas.
pub fn sprite_origin(anchor_x: i32, anchor_y: i32, sit_x: i32, sit_y: i32) -> (i32, i32) {
    (anchor_x - sit_x, anchor_y - sit_y)
}

/// Top-left of a sea, water, quay, or pier sprite. The sit-point is 48px
/// below the origin, which is the art-director datum of -48.
pub fn water_sprite_origin(anchor_x: i32, anchor_y: i32) -> (i32, i32) {
    sprite_origin(anchor_x, anchor_y, SIT_X, SIT_Y)
}

/// Facing for a step in chart cells, measured after the dimetric rotation.
pub fn facing_from_chart_delta(dx: i64, dy: i64) -> Facing {
    facing_from_display_delta((dx - dy) as f64, (dx + dy) as f64)
}

/// Facing for a step already in rotated display axes `(u, v)`.
///
/// Pixel deltas are `(u * 64, v * 32)`. The angle is `atan2` in that
/// Y-down screen space, divided into eight sectors centered on the
/// cardinal and diagonal directions. A zero step faces east.
pub fn facing_from_display_delta(u: f64, v: f64) -> Facing {
    if u == 0.0 && v == 0.0 {
        return Facing::E;
    }
    let screen_dx = u * f64::from(CELL_WIDTH / 2);
    let screen_dy = v * f64::from(CELL_HEIGHT / 2);
    let angle = screen_dy.atan2(screen_dx);
    let sector = std::f64::consts::FRAC_PI_4;
    let index = (angle / sector).round() as i32;
    Facing::from_index(index)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projection_is_dimetric_two_to_one() {
        assert_eq!(chart_to_screen(0, 0), (0, 0));
        // +chart x runs down-right: one cell is (64, 32), a 2:1 step.
        assert_eq!(chart_to_screen(1, 0), (64, 32));
        // +chart y runs down-left.
        assert_eq!(chart_to_screen(0, 1), (-64, 32));
        assert_eq!(chart_to_screen(1, 1), (0, 64));
        assert_eq!(CELL_WIDTH, CELL_HEIGHT * 2);
    }

    #[test]
    fn rotation_round_trips() {
        for x in 0..50 {
            for y in 0..36 {
                let (u, v) = rotate_chart(x, y);
                assert_eq!((u + v) / 2, x);
                assert_eq!((v - u) / 2, y);
            }
        }
    }

    #[test]
    fn water_datum_places_sit_point_on_the_anchor() {
        let anchor = chart_to_screen(18, 8);
        let origin = water_sprite_origin(anchor.0, anchor.1);
        assert_eq!(origin.1 - anchor.1, WATER_DATUM_Y);
        assert_eq!(origin.0 + SIT_X, anchor.0);
        assert_eq!(origin.1 + SIT_Y, anchor.1);
        assert_eq!(WATER_DATUM_Y, -48);
    }

    #[test]
    fn eight_screen_directions_map_to_eight_facings() {
        for index in 0..8 {
            let angle = index as f64 * std::f64::consts::FRAC_PI_4;
            // Invert the pixel scale so the screen angle is exactly `angle`.
            let u = angle.cos() / f64::from(CELL_WIDTH / 2);
            let v = angle.sin() / f64::from(CELL_HEIGHT / 2);
            assert_eq!(
                facing_from_display_delta(u, v),
                Facing::from_index(index),
                "sector {index}"
            );
        }
    }

    #[test]
    fn chart_axes_face_along_the_diamond_edges() {
        // +chart x is the down-right edge. atan2(32, 64) is about 26.6°,
        // which is the south-east sector (east ends at 22.5°).
        assert_eq!(facing_from_chart_delta(1, 0), Facing::Se);
        // +chart y is the down-left edge.
        assert_eq!(facing_from_chart_delta(0, 1), Facing::Sw);
        assert_eq!(facing_from_chart_delta(-1, 0), Facing::Nw);
        assert_eq!(facing_from_chart_delta(0, -1), Facing::Ne);
        assert_eq!(facing_from_chart_delta(0, 0), Facing::E);
    }

    #[test]
    fn sector_boundary_rounds_half_away_from_east() {
        // Exactly 22.5° is halfway between east and south-east.
        let angle = std::f64::consts::FRAC_PI_8;
        let u = angle.cos() / f64::from(CELL_WIDTH / 2);
        let v = angle.sin() / f64::from(CELL_HEIGHT / 2);
        assert_eq!(facing_from_display_delta(u, v), Facing::Se);
        let just_inside = angle - 0.01;
        let u = just_inside.cos() / f64::from(CELL_WIDTH / 2);
        let v = just_inside.sin() / f64::from(CELL_HEIGHT / 2);
        assert_eq!(facing_from_display_delta(u, v), Facing::E);
    }

    #[test]
    fn pure_display_axes_are_the_cardinals() {
        assert_eq!(facing_from_display_delta(1.0, 0.0), Facing::E);
        assert_eq!(facing_from_display_delta(0.0, 1.0), Facing::S);
        assert_eq!(facing_from_display_delta(-1.0, 0.0), Facing::W);
        assert_eq!(facing_from_display_delta(0.0, -1.0), Facing::N);
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
}
