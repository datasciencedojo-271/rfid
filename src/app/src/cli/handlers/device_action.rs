use crate::cli::commands::ActionArgs;
use api::api::error::RfidError;
use api::api::uhf_rfid_api::UhfRfidApi;
use api::rfid_device::usb_device::UsbDevice;
use colorful::{Color, Colorful};
use protocl::types::DeviceAction;

pub fn handle(device: &UsbDevice, args: &ActionArgs) -> Result<(), RfidError> {
    println!(
        "{}",
        format!(
            "Performing actions: {} for {} deciseconds",
            args.actions, args.time
        )
        .color(Color::Cyan)
    );
    let mut actions = Vec::new();
    // Parse action string
    for action in args.actions.split('+') {
        match action.trim().to_lowercase().as_str() {
            "beep" => actions.push(DeviceAction::Beep),
            "red" => actions.push(DeviceAction::RedLed),
            "green" => actions.push(DeviceAction::GreenLed),
            "yellow" => actions.push(DeviceAction::YellowLed),
            _ => {
                println!("{}", format!("Unknown action: {action}").color(Color::Red));
                return Ok(());
            }
        }
    }
    if actions.is_empty() {
        println!("{}", "No valid actions specified".color(Color::Red));
        return Ok(());
    }
    // Perform the actions
    match UhfRfidApi::device_action(device, &actions, args.time) {
        Ok(()) => {
            println!("{}", "Actions performed successfully!".color(Color::Green));
            Ok(())
        }
        Err(e) => Err(e),
    }
}
