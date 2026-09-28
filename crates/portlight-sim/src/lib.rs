//! Portlight simulation core.
//!
//! Rules and world state live here with no terminal, ratatui, or graphics
//! dependency, so a later dimetric front end can sit on the same crate.
//!
//! The first slice covers content, prices, trade, reputation side effects of
//! trade and travel, one day of voyaging, and the four victory paths.
//! See `docs/PORTING-PLAN.md`.
//!
//! Front ends drive [`Session`] one turn at a time. [`run_script`] calls that
//! same API.

pub mod campaign;
pub mod content;
pub mod contracts;
pub mod duel;
pub mod economy;
pub mod error;
pub mod model;
pub mod pyrand;
pub mod reputation;
pub mod save;
pub mod script;
pub mod session;
pub mod ship;
pub mod snapshot;
pub mod util;
pub mod voyage;
pub mod world;

pub use campaign::{
    compute_victory_progress, HouseBooks, MilestoneFamily, VictoryPathStatus,
    MILESTONE_FAMILY_COMMERCIAL_FINANCE, PATH_COMMERCIAL_EMPIRE,
};
pub use duel::{DuelOutcome, DuelRound};
pub use error::SimError;
pub use model::{MAP_GRID_HEIGHT, MAP_GRID_WIDTH};
pub use script::run_script;
pub use session::{Session, Turn};
pub use snapshot::Snapshot;
pub use voyage::{
    estimate_sail_days, sail_lanes, EventType, LaneSuitability, SailLane, VoyageEvent,
};
pub use world::new_game;
