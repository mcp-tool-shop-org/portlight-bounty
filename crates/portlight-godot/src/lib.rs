//! Godot 4.7 view. The simulation stays in `portlight-sim`; this crate only
//! draws [`portlight_chart::ChartModel`] and forwards button presses.

mod chart_canvas;
mod contracts_screen;
mod encounter_screen;
mod game;
mod harbour;
mod harbour_screen;
mod journal_screen;
mod logic;
mod newgame_screen;
mod seam;
mod shipyard_screen;

use godot::prelude::*;

struct PortlightExtension;

// Safety: godot-rust requires an unsafe ExtensionLibrary impl. The macro
// emits the GDExtension entry point; this crate does not call the FFI itself.
#[gdextension]
unsafe impl ExtensionLibrary for PortlightExtension {}
