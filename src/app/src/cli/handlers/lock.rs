use crate::cli::commands::LockArgs;
use api::api::error::RfidError;
use api::api::uhf_rfid_api::UhfRfidApi;
use api::rfid_device::usb_device::UsbDevice;
use colorful::{Color, Colorful};
use std::io;
use std::io::Write;

pub fn handle(device: &UsbDevice, args: &LockArgs) -> Result<(), RfidError> {
    // First, do an inventory to check if tags are in range
    println!("{}", "Checking for tags in range...".color(Color::Cyan));
    match UhfRfidApi::inventory(device) {
        Err(e) => Err(e),
        Ok(tags) => {
            if tags.is_empty() {
                println!(
                    "{}",
                    "No tags found in range. Please place a tag near the reader."
                        .color(Color::Yellow)
                );
                return Ok(());
            }
            println!(
                "{} {} {}",
                "Found".color(Color::Green),
                tags.len().to_string().color(Color::Green).bold(),
                "tags in range.".color(Color::Green)
            );
            if tags.len() > 1 && !args.force {
                println!(
                    "{}",
                    "Warning: Multiple tags detected. Lock may affect all tags in range."
                        .color(Color::Yellow)
                        .bold()
                );
                print!("{}", "Continue? (y/n): ".color(Color::Yellow));
                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to read input");
                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("{}", "Operation cancelled.".color(Color::Yellow));
                    return Ok(());
                }
            }
            // Warning about lock operations
            if !args.force {
                println!(
                    "{}",
                    "⚠️  WARNING: Lock operations can be PERMANENT depending on settings! ⚠️"
                        .color(Color::Red3a)
                        .bold()
                );
                println!(
                    "{}",
                    "Some lock actions cannot be reversed. Proceed with caution."
                        .color(Color::Yellow)
                );
                print!("{}", "Continue? (y/n): ".color(Color::Yellow));
                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to read input");
                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("{}", "Operation cancelled.".color(Color::Yellow));
                    return Ok(());
                }
            }
            println!(
                "{} {} {} {} {}",
                "Locking".color(Color::Cyan),
                format!("{:?}", args.bank).color(Color::White).bold(),
                "memory with".color(Color::Cyan),
                format!("{:?}", args.action).color(Color::White).bold(),
                "action...".color(Color::Cyan)
            );
            UhfRfidApi::lock_memory_bank(device, args.bank, args.action)?;
            println!(
                "{}",
                "Lock operation successful!".color(Color::Green).bold()
            );
            Ok(())
        }
    }
}
