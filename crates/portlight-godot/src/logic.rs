//! Decisions the Godot view can make without a display server.
//!
//! Projection and facing stay in `portlight-chart`. This module is the view's
//! use of that math, plus the button rules that sit beside `Session`.
//! Sampling a captured image only reads pixels; it does not draw.

use godot::classes::Image;
use godot::prelude::*;

/// Playable window in `godot/project.godot`.
pub(crate) const WINDOW_W: f32 = 1280.0;
pub(crate) const WINDOW_H: f32 = 720.0;
/// Side panel. Wide enough for a lane label and a Sail button.
pub(crate) const PANEL_MIN_W: f32 = 420.0;
/// `HBoxContainer` separation set on the chart row.
pub(crate) const ROW_SEPARATION: i32 = 4;

/// Width left for the chart once the panel and the row gap are on screen.
pub(crate) fn chart_host_width() -> f32 {
    WINDOW_W - PANEL_MIN_W - ROW_SEPARATION as f32
}

pub(crate) fn layout_fits_window() -> bool {
    chart_host_width() > 0.0
        && chart_host_width() + PANEL_MIN_W + ROW_SEPARATION as f32 <= WINDOW_W
        && WINDOW_H == 720.0
}

/// The Duel button is available whenever a duel is pending. Stance count is
/// the sim's check (`Session::duel`), not a second gate in the view.
pub(crate) fn duel_button_enabled(pending: bool) -> bool {
    pending
}

/// A captured window is useless when one colour covers almost every sample.
/// The broken 1280×720 chart shot was the clear colour below a 28 px strip.
pub(crate) fn frame_mostly_flat(samples: &[[u8; 3]]) -> bool {
    dominant_color_fraction(samples) >= 0.80
}

/// `PORTLIGHT_SHOT` and the harbour seam share this rejection.
/// An empty image, the wrong size, or one colour over most of the frame fails.
pub(crate) fn capture_frame_rejected(
    width: i32,
    height: i32,
    expect_w: i32,
    expect_h: i32,
    samples: &[[u8; 3]],
) -> bool {
    samples.is_empty() || width != expect_w || height != expect_h || frame_mostly_flat(samples)
}

/// Process status for `harbour_seam.tscn`.
/// An illegal layout and a rejected frame both exit non-zero.
/// The scene must not write a PNG when `layout_ok` is false.
pub(crate) fn seam_exit_code(layout_ok: bool, frames_ok: bool) -> i32 {
    if layout_ok && frames_ok {
        0
    } else {
        1
    }
}

/// Every eighth pixel. `PORTLIGHT_SHOT` and the harbour seam both use this.
pub(crate) fn frame_samples(image: &Gd<Image>) -> Vec<[u8; 3]> {
    let width = image.get_width();
    let height = image.get_height();
    let mut samples = Vec::new();
    let step = 8;
    let mut y = 0;
    while y < height {
        let mut x = 0;
        while x < width {
            let color = image.get_pixel(x, y);
            samples.push([
                (color.r.clamp(0.0, 1.0) * 255.0).round() as u8,
                (color.g.clamp(0.0, 1.0) * 255.0).round() as u8,
                (color.b.clamp(0.0, 1.0) * 255.0).round() as u8,
            ]);
            x += step;
        }
        y += step;
    }
    samples
}

fn dominant_color_fraction(samples: &[[u8; 3]]) -> f32 {
    if samples.is_empty() {
        return 1.0;
    }
    let mut counts = std::collections::HashMap::<[u8; 3], usize>::new();
    let mut best = 0usize;
    for sample in samples {
        let count = counts.entry(*sample).or_insert(0);
        *count += 1;
        if *count > best {
            best = *count;
        }
    }
    best as f32 / samples.len() as f32
}

#[cfg(test)]
mod tests {
    use portlight_chart::{chart_to_screen_f, chart_to_uv, facing_from_uv, Facing};

    use super::*;

    #[test]
    fn porto_novo_uses_the_rotated_projection() {
        let (x, y) = chart_to_screen_f(18.0, 8.0);
        assert!((x - 1629.2).abs() < 0.2, "{x}");
        assert!((y - 362.0).abs() < 0.2, "{y}");
    }

    #[test]
    fn grain_road_selects_facing_f7() {
        let (u0, v0) = chart_to_uv(18.0, 8.0);
        let (u1, v1) = chart_to_uv(24.0, 6.0);
        assert_eq!(facing_from_uv(u1 - u0, v1 - v0, None), Facing::F7);
    }

    #[test]
    fn facing_holds_through_the_five_degree_edge() {
        let hold = 26.0_f64.to_radians();
        let switch = 28.0_f64.to_radians();
        assert_eq!(
            facing_from_uv(hold.cos(), hold.sin(), Some(Facing::F0)),
            Facing::F0
        );
        assert_eq!(
            facing_from_uv(switch.cos(), switch.sin(), Some(Facing::F0)),
            Facing::F1
        );
    }

    #[test]
    fn duel_button_ignores_stance_count() {
        assert!(!duel_button_enabled(false));
        assert!(duel_button_enabled(true));
    }

    #[test]
    fn the_panel_and_chart_fit_the_window() {
        let project = include_str!("../../../godot/project.godot");
        assert!(project.contains("window/size/viewport_width=1280"));
        assert!(project.contains("window/size/viewport_height=720"));
        assert!(layout_fits_window());
    }

    #[test]
    fn a_flat_capture_fails_and_a_chart_frame_does_not() {
        let clear = [13, 25, 41];
        assert!(frame_mostly_flat(&[clear; 100]));
        // The committed chart-1280.png was clear colour under a thin strip.
        let mut strip = vec![clear; 96];
        strip.extend([[70, 120, 150]; 4]);
        assert!(frame_mostly_flat(&strip));
        let mut chart = vec![clear; 40];
        chart.extend([[32, 78, 112]; 30]);
        chart.extend([[232, 196, 120]; 30]);
        assert!(!frame_mostly_flat(&chart));
    }

    #[test]
    fn an_illegal_layout_and_a_flat_seam_frame_exit_nonzero() {
        // The old capture wrote the clear colour and exited 0 when the
        // layout panic was swallowed. Both of those outcomes are failures.
        assert_eq!(seam_exit_code(false, true), 1);
        assert_eq!(seam_exit_code(true, false), 1);
        assert_eq!(seam_exit_code(false, false), 1);
        assert_eq!(seam_exit_code(true, true), 0);

        let clear = [13, 25, 41];
        assert!(capture_frame_rejected(1280, 720, 1280, 720, &[clear; 100]));
        assert!(capture_frame_rejected(0, 0, 1280, 720, &[]));
        let mut chart = vec![clear; 40];
        chart.extend([[32, 78, 112]; 30]);
        chart.extend([[232, 196, 120]; 30]);
        assert!(!capture_frame_rejected(1280, 720, 1280, 720, &chart));
        // A crop uses its own size. A flat crop is still rejected.
        assert!(capture_frame_rejected(192, 128, 192, 128, &[clear; 40]));
        assert!(!capture_frame_rejected(192, 128, 192, 128, &chart));
    }
}
