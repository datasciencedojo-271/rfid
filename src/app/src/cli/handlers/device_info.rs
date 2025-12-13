use api::api::error::RfidError;
use api::rfid_device::usb_device::UsbDevice;
use colorful::{Color, Colorful};

pub fn handle(device: &UsbDevice) -> Result<(), RfidError> {
    println!("{}", "Device Information:".color(Color::Cyan).bold());
    println!("{}", "--------------------".color(Color::Cyan));
    let info = device.get_info();
    println!(
        "{} {} ({:04x}:{:04x})",
        "Device:".color(Color::Green),
        info.product,
        info.vendor_id,
        info.product_id
    );
    println!(
        "{} {}",
        "Manufacturer:".color(Color::Green),
        info.manufacturer
    );
    println!(
        "{} {}",
        "Serial Number:".color(Color::Green),
        info.serial_number
    );
    println!(
        "{} {}{} {} {}{} {} {}{}",
        "Connection:".color(Color::Green),
        "Interface ".color(Color::White),
        device
            .get_interface()
            .to_string()
            .color(Color::White)
            .bold(),
        ",".color(Color::White),
        "IN endpoint 0x".color(Color::White),
        format!("{:02X}", device.get_in_endpoint())
            .color(Color::White)
            .bold(),
        ",".color(Color::White),
        "OUT endpoint 0x".color(Color::White),
        format!("{:02X}", device.get_out_endpoint())
            .color(Color::White)
            .bold()
    );
    println!(
        "{} {}",
        "Is connected:".color(Color::Green),
        if device.is_connected() {
            "Yes".color(Color::Green).bold()
        } else {
            "No".color(Color::Red).bold()
        }
    );
    // Try to print device capabilities or version if available
    println!("{}", "\nChecking device capabilities...".color(Color::Cyan));
    // This would depend on your specific device's capabilities
    // Implement if your device supports this
    Ok(())
}
