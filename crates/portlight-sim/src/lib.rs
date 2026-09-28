//! Portlight simulation core.
//!
//! Rules and world state live here with no terminal, ratatui, or graphics
//! dependency, so a later dimetric front end can sit on the same crate.
//!
//! The first slice covers content, prices, trade, reputation side effects of
//! trade and travel, and one day of voyaging. See `docs/PORTING-PLAN.md`.

pub mod content;
pub mod economy;
pub mod model;
pub mod pyrand;
pub mod reputation;
pub mod script;
pub mod ship;
pub mod snapshot;
pub mod util;
pub mod voyage;
pub mod world;

pub use script::run_script;
pub use snapshot::Snapshot;
pub use world::new_game;
