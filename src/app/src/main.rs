#![warn(clippy::pedantic)]

mod cli;
mod tui;

use crate::cli::{commands::Cli, run_cli};
use crate::tui::run_tui;
use api::api::error::RfidError;
use api::platform;
use clap::Parser;
use std::process;

fn main() -> Result<(), RfidError> {
    // Check for USB permissions before trying to connect
    if let Err(e) = platform::check_usb_permissions() {
        eprintln!("USB permission check failed: {e}");
        platform::print_permission_instructions();
        process::exit(1);
    }

    // Parse command line arguments to check for CLI mode
    let cli = Cli::parse();

    // Run either CLI or TUI based on arguments

    if cli.cli || cli.command.is_some() {
        run_cli()
    } else {
        run_tui()
    }
}
