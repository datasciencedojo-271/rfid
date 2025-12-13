use crate::cli::commands::RawCommandArgs;
use crate::cli::handlers::utils;
use api::api::error::RfidError;
use api::api::uhf_rfid_api::UhfRfidApi;
use api::rfid_device::usb_device::UsbDevice;
use colorful::{Color, Colorful};
use std::io;
use std::io::Write;

pub fn handle(device: &UsbDevice, args: &RawCommandArgs) -> Result<(), RfidError> {
    if !args.force {
        println!("{}", "\nManual Command Mode".color(Color::Cyan).bold());
        println!("{}", "------------------".color(Color::Cyan));
        println!(
            "{}",
            "This mode allows sending raw commands to the device.".color(Color::White)
        );
        // Warning about manual commands
        println!(
            "{}",
            "\n⚠️  WARNING: Manual commands can damage your device or tags! ⚠️"
                .color(Color::Red3b)
                .bold()
        );
        println!(
            "{}",
            "Only use this mode if you know exactly what you're doing.".color(Color::Yellow)
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
        "{} {}",
        "Sending command:".color(Color::Cyan),
        UhfRfidApi::hex_to_ascii(&args.data)
    );
    let bytes_written = device.write(&args.data)?;
    println!(
        "{} {}. {}",
        "Sent".color(Color::Green),
        bytes_written.to_string().color(Color::Green).bold(),
        "bytes. Waiting for response...".color(Color::Green)
    );
    // Read response
    let mut buffer = [0u8; 256]; // Adjust buffer size as needed
    match device.read(&mut buffer) {
        Ok(bytes_read) if bytes_read > 0 => {
            let response = &buffer[0..bytes_read];
            println!(
                "{} {}.",
                "Received".color(Color::Green),
                bytes_read.to_string().color(Color::Green).bold()
            );
            println!(
                "{} {}",
                "Response (hex):".color(Color::Cyan),
                UhfRfidApi::hex_to_ascii(response)
            );
            // Try to display as ASCII if possible
            utils::print_as_ascii("Response (ASCII):", &Vec::from(response));
        }
        Ok(_) => println!("{}", "No response received (timeout).".color(Color::Yellow)),
        Err(e) => println!(
            "{} {}",
            "Error reading response:".color(Color::Red),
            e.to_string().color(Color::Red).bold()
        ),
    }
    Ok(())
}
