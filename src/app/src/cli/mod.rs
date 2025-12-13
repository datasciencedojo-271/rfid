//! CLI module for the RFID command-line application

use api::api::error::RfidError;
use api::rfid_device::usb_device::UsbDevice;
use clap::Parser;
use colorful::{Color, Colorful};
use commands::{Cli, Commands};
use std::process;

pub mod app;
pub mod commands;
mod handlers;
pub mod menu;

pub fn run_cli() -> Result<(), RfidError> {
    // Parse command line arguments
    let cli = Cli::parse();

    // Create a device with the appropriate debug setting
    let device = match UsbDevice::new() {
        Ok(device) => {
            println!(
                "{}",
                format!("Connected to {}", device.get_info()).color(Color::Green)
            );
            device
        }
        Err(e) => {
            eprintln!(
                "{}",
                format!("Failed to connect to RFID device: {e}").color(Color::Red)
            );
            process::exit(1);
        }
    };

    // If no command is specified, show help
    if cli.command.is_none() {
        println!(
            "{}",
            "No command specified. Use --help to see available commands.".color(Color::Yellow)
        );
        return Ok(());
    }

    match cli.command.as_ref().unwrap() {
        Commands::Interactive => {
            println!("{}", "Starting interactive mode...".color(Color::Cyan));
            app::run_interactive_app()
        }
        Commands::Inventory => handlers::inventory::handle(&device),
        Commands::Read(args) => handlers::read::handle(&device, args),
        Commands::Write(args) => handlers::write::handle(&device, args),
        Commands::Lock(args) => handlers::lock::handle(&device, args),
        Commands::Password(args) => handlers::password::handle(&device, args),
        Commands::DeviceInfo => handlers::device_info::handle(&device),
        Commands::RawCommand(args) => handlers::raw_command::handle(&device, args),
        Commands::Action(args) => handlers::device_action::handle(&device, args),
        Commands::Test => handlers::test::handle(&device),
    }
}
