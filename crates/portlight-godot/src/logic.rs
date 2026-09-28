//! Decisions the Godot view can make without a display server.
//!
//! Projection and facing stay in `portlight-chart`. This module is the view's
//! use of that math, plus the button rules that sit beside `Session`.

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
}
