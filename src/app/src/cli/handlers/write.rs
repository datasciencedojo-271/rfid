use crate::cli::commands::WriteArgs;
use api::api::error::RfidError;
use api::api::uhf_rfid_api::UhfRfidApi;
use api::rfid_device::usb_device::UsbDevice;
use colorful::{Color, Colorful};
use std::io;
use std::io::Write;

pub fn handle(device: &UsbDevice, args: &WriteArgs) -> Result<(), RfidError> {
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
            if tags.len() > 1 {
                println!(
                    "{}",
                    "Warning: Multiple tags detected. Write may affect all tags in range."
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
                "{} {} {} {} {} {}",
                "Writing".color(Color::Cyan),
                args.data.len().to_string().color(Color::White).bold(),
                "bytes to".color(Color::Cyan),
                format!("{:?}", args.bank).color(Color::White).bold(),
                "memory at address".color(Color::Cyan),
                args.address.to_string().color(Color::White).bold()
            );
            if let Err(e) = UhfRfidApi::write(device, args.bank, args.address, &args.data) {
                Err(e)
            } else {
                println!("{}", "Write successful!".color(Color::Green).bold());
                Ok(())
            }
        }
    }
}
