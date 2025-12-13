use crate::cli::commands::PasswordArgs;
use api::api::error::RfidError;
use api::api::uhf_rfid_api::UhfRfidApi;
use api::rfid_device::usb_device::UsbDevice;
use colorful::{Color, Colorful};
use std::io;
use std::io::Write;

pub fn handle(device: &UsbDevice, args: &PasswordArgs) -> Result<(), RfidError> {
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
                    "Warning: Multiple tags detected. Password will be set on all tags in range."
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
            println!(
                "{} {:08X}{}",
                "Setting access password to ".color(Color::Cyan),
                args.password,
                "...".color(Color::Cyan)
            );
            if let Err(e) = UhfRfidApi::set_access_password(device, args.password) {
                Err(e)
            } else {
                println!(
                    "{}",
                    "Password set successfully!".color(Color::Green).bold()
                );
                println!(
                    "{}",
                    "⚠️  IMPORTANT: Make sure to write down this password! ⚠️"
                        .color(Color::Red3b)
                        .bold()
                );
                println!(
                    "{}",
                    "If you lose this password, you may not be able to access your tag."
                        .color(Color::Yellow)
                );
                Ok(())
            }
        }
    }
}
