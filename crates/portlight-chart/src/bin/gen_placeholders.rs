//! Write placeholder chart tiles and `asset-list.csv`.
//!
//! ```text
//! cargo run -p portlight-chart --bin gen-placeholders -- godot/assets
//! ```

use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let dir = env::args()
        .nth(1)
        .unwrap_or_else(|| "godot/assets".to_string());
    match portlight_chart::write_asset_files(std::path::Path::new(&dir)) {
        Ok(()) => {
            println!("wrote placeholders to {dir}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(1)
        }
    }
}
