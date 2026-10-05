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

pub mod bounty;
pub mod campaign;
pub mod combat;
pub mod companion;
pub mod consequences;
pub mod content;
pub mod contracts;
pub mod cross_port_networks;
pub mod culture;
pub mod custom_captain;
pub mod duel;
pub mod economy;
pub mod encounter;
pub mod error;
pub mod fleet;
pub mod hunting;
pub mod infrastructure;
pub mod injuries;
pub mod loot;
pub mod memory;
pub mod merchant;
pub mod model;
pub mod narrative;
pub mod naval;
pub mod port_arrival_engine;
pub mod pyrand;
pub mod reputation;
pub mod save;
pub mod script;
pub mod sea_culture;
pub mod session;
pub mod ship;
pub mod skills;
pub mod snapshot;
pub mod training;
pub mod util;
pub mod voyage;
pub mod weapon_provenance;
pub mod weapon_quality;
pub mod world;

pub use campaign::{
    compute_victory_progress, HouseBooks, MilestoneFamily, VictoryPathStatus,
    MILESTONE_FAMILY_COMMERCIAL_FINANCE, PATH_COMMERCIAL_EMPIRE,
};
pub use custom_captain::{
    build_custom_template, validate_spec, CustomCaptainSpec, CustomCaptainTemplate,
};
pub use duel::{DuelOutcome, DuelRound};
pub use error::SimError;
pub use model::{MAP_GRID_HEIGHT, MAP_GRID_WIDTH};
pub use script::{load_snapshot, run_and_save, run_script};
pub use session::{Session, Turn, VictoryReceipt};
pub use snapshot::Snapshot;
pub use voyage::{
    estimate_sail_days, sail_lanes, EventType, LaneSuitability, SailLane, VoyageEvent,
};
pub use world::new_game;
