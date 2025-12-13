use crate::cli::commands::ReadArgs;
use crate::cli::handlers::utils;
use api::api::error::RfidError;
use api::api::uhf_rfid_api::UhfRfidApi;
use api::rfid_device::usb_device::UsbDevice;
use colorful::{Color, Colorful};
use protocl::types::MemoryBank;

pub fn handle(device: &UsbDevice, args: &ReadArgs) -> Result<(), RfidError> {
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
            println!(
                "{} {} {} {} {} {}",
                "Reading".color(Color::Cyan),
                args.words.to_string().color(Color::White).bold(),
                "words from".color(Color::Cyan),
                format!("{:?}", args.bank).color(Color::White).bold(),
                "memory at address".color(Color::Cyan),
                args.address.to_string().color(Color::White).bold()
            );
            match UhfRfidApi::read(device, args.bank, args.address, args.words) {
                Err(e) => Err(e),
                Ok(data) => {
                    println!("{}", "Read successful!".color(Color::Green).bold());
                    println!(
                        "{} {}",
                        "Data (hex):".color(Color::Cyan),
                        UhfRfidApi::hex_to_ascii(&data)
                    );
                    // If it's EPC or TID, try to display as ASCII too
                    if args.bank == MemoryBank::Epc || args.bank == MemoryBank::Tid {
                        utils::print_as_ascii("Data (ASCII):", &data);
                    }
                    Ok(())
                }
            }
        }
    }
}
