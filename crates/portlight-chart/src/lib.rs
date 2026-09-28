//! Dimetric chart for the Portlight simulation.
//!
//! Chart coordinates stay in the sim (`Port.map_x` / `map_y` on the 50×36
//! grid). This crate only rotates them into display space. It reads
//! [`portlight_sim::sail_lanes`] for the overlay and does not decide which
//! lanes are legal.

mod assets;
mod placeholder;
mod project;
mod view;

pub use assets::{asset, ship_asset, Asset, AssetFamily, ASSETS};
pub use placeholder::write_asset_files;
pub use project::{
    chart_to_screen, chart_to_screen_f, facing_from_chart_delta, facing_from_display_delta,
    frame_to_view, sprite_origin, water_sprite_origin, Facing, Frame, ScreenRect, CELL_HEIGHT,
    CELL_WIDTH, SIT_X, SIT_Y, WATER_DATUM_Y,
};
pub use view::{
    project_chart, ActiveLeg, ChartLane, ChartModel, ChartPort, Rgba, ShipMarker, CHART_VIEW_H,
    CHART_VIEW_W, FIRST_PLAYABLE_CAPTAIN, FIRST_PLAYABLE_NAME, FIRST_PLAYABLE_SEED, MEDITERRANEAN,
    SHIP_FOOTPRINT_CELLS,
};
