//! Godot 4.7 view. The simulation stays in `portlight-sim`; this crate only
//! draws [`portlight_chart::ChartModel`] and forwards button presses.

mod chart_canvas;
mod game;

use godot::prelude::*;

struct PortlightExtension;

// Safety: godot-rust requires an unsafe ExtensionLibrary impl. The macro
// emits the GDExtension entry point; this crate does not call the FFI itself.
#[gdextension]
unsafe impl ExtensionLibrary for PortlightExtension {}
