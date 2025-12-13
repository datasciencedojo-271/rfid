use api::api::error::RfidError;
use api::api::uhf_rfid_api::UhfRfidApi;
use api::rfid_device::usb_device::UsbDevice;
use colorful::{Color, Colorful};
use std::println;

pub fn handle(device: &UsbDevice) -> Result<(), RfidError> {
    println!(
        "{}",
        "Performing inventory (scanning for tags)...".color(Color::Cyan)
    );
    match UhfRfidApi::inventory(device) {
        Ok(tags) => {
            if tags.is_empty() {
                println!("{}", "No tags found.".color(Color::Yellow));
            } else {
                println!(
                    "{} {}{}",
                    "Found".color(Color::Green),
                    tags.len().to_string().color(Color::Green).bold(),
                    " tags:".color(Color::Green)
                );

                for (i, tag) in tags.iter().enumerate() {
                    println!("  {}: {}", (i + 1).to_string().color(Color::Blue), tag);
                }
            }
            Ok(())
        }
        Err(e) => {
            println!("{}", "Error performing inventory.".color(Color::Red));
            Err(e)
        }
    }
}
