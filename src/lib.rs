mod adapters;
mod application;
mod cli;
mod generator;
mod headless;
mod helpers;
mod merge;
mod models;
mod ports;
mod state;
mod templates;
mod theme;
mod tui;
mod utils;

pub use cli::Cli;
pub use headless::run_cli;

use anyhow::Result;

/// Starts Banna's terminal user interface.
pub fn run(cli: &Cli) -> Result<()> {
    tui::run_tui(cli)
}
