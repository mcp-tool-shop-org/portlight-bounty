//! Dimetric chart for the Portlight simulation.
//!
//! Chart coordinates stay in the sim (`Port.map_x` / `map_y` on the 50×36
//! grid). This crate only rotates them into display space. Lanes come from
//! [`portlight_sim::Session::sail_lanes`]. The chart does not decide which
//! lanes are legal and does not call the voyage helper underneath that method.

mod assets;
mod cover;
mod harbour;
#[cfg(test)]
mod landing;
mod placeholder;
mod project;
mod seam;
mod view;

pub use assets::{asset, ship_asset, Asset, AssetFamily, ASSETS};
pub use harbour::{harbour_anchor, HarbourLayer, HarbourTile, WorkKind};
pub use placeholder::write_asset_files;
pub use project::{
    chart_to_screen, chart_to_screen_f, chart_to_uv, facing_from_chart_delta, facing_from_uv,
    frame_to_view, grid_to_screen, grid_to_screen_f, sprite_origin, uv_to_chart, uv_to_screen,
    Facing, Frame, ScreenRect, CELL_HEIGHT, CELL_WIDTH, DOCKED_OFFSET_X, DOCKED_OFFSET_Y,
    HARBOUR_CELL_H, HARBOUR_CELL_W, WATER_DATUM_Y,
};
pub use seam::{harbour_seam, seam_camera_center, seam_interior_vertex};
pub use view::{
    docked_sloop_marker, hover_at, lane_inspect, press_port, project_chart, ActiveLeg, ChartLane,
    ChartModel, ChartPort, PortPress, Rgba, ShipMarker, WaterTile, CHART_VIEW_H, CHART_VIEW_W,
    DAY_TWEEN_SECS, FIRST_PLAYABLE_CAPTAIN, FIRST_PLAYABLE_NAME, FIRST_PLAYABLE_SEED,
    MEDITERRANEAN, SHIP_FOOTPRINT_CELLS,
};
