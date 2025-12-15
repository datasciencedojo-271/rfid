use api::api::error::RfidError;
use api::api::uhf_rfid_api::UhfRfidApi;
use api::rfid_device::usb_device::UsbDevice;
use colorful::{Color, Colorful};
use protocl::types::{DeviceAction, MemoryBank};

pub fn handle(device: &UsbDevice) -> Result<(), RfidError> {
    println!("{}", "Running basic UHF test...".color(Color::Cyan));
    // Read EPC
    println!("\n{}", "Reading EPC memory bank:".color(Color::Yellow));
    let epc_data = UhfRfidApi::read(device, MemoryBank::Epc, 0, 8)?;
    let epc_hex = UhfRfidApi::hex_to_ascii(&epc_data);
    println!("EPC Data: {}", epc_hex.color(Color::Green));
    // Read TID
    println!("\n{}", "Reading TID memory bank:".color(Color::Yellow));
    let tid_data = UhfRfidApi::read(device, MemoryBank::Tid, 0, 8)?;
    let tid_hex = UhfRfidApi::hex_to_ascii(&tid_data);
    println!("TID Data: {}", tid_hex.color(Color::Green));
    // Read USER
    println!("\n{}", "Reading USER memory bank:".color(Color::Yellow));
    let user_data = UhfRfidApi::read(device, MemoryBank::User, 0, 8)?;
    let user_hex = UhfRfidApi::hex_to_ascii(&user_data);
    println!("USER Data: {}", user_hex.color(Color::Green));
    // Read Reserved
    println!("\n{}", "Reading Reserved memory bank:".color(Color::Yellow));
    let reserved_data = UhfRfidApi::read(device, MemoryBank::Reserved, 0, 8)?;
    let reserved_hex = UhfRfidApi::hex_to_ascii(&reserved_data);
    println!("Reserved Data: {}", reserved_hex.color(Color::Green));
    // Perform beep + green LED action
    println!(
        "\n{}",
        "Performing beep + green LED action...".color(Color::Yellow)
    );
    UhfRfidApi::device_action(
        device,
        &[DeviceAction::Beep, DeviceAction::GreenLed],
        50, // 500ms
    )?;
    println!("{}", "Action performed successfully!".color(Color::Green));
    Ok(())
}
