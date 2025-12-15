//! Main app for the RFID CLI

use crate::cli::menu::{Menu, MenuOption};
use api::api::error::RfidError;
use api::api::uhf_rfid_api::UhfRfidApi;
use api::rfid_device::usb_device::UsbDevice;
use protocl::types::{DeviceAction, MemoryBank};
use std::io::{self, Write};

pub fn run_interactive_app() -> Result<(), RfidError> {
    let mut menu = Menu::new();

    println!("UHF RFID Programmer - Interactive Mode");
    println!("Connecting to RFID device...");

    // Create a new RFID device with debug mode-turn-off for cleaner output.
    let mut device = UsbDevice::new()?;
    println!("Connected to {} successfully!", device.get_info());

    let mut running = true;
    while running {
        menu.display();
        let choice = menu.get_choice();

        match choice {
            MenuOption::Inventory => handle_inventory(&device, &menu)?,
            MenuOption::ReadTag => handle_read_tag(&device, &menu)?,
            MenuOption::WriteTag => handle_write_tag(&device, &menu)?,
            MenuOption::LockTag => handle_lock_tag(&device, &menu)?,
            MenuOption::SetPassword => handle_set_password(&device, &menu)?,
            MenuOption::AdvancedOptions => unsafe {
                std::ptr::write(&raw mut menu.in_advanced_menu, true);
            },
            MenuOption::ShowDeviceInfo => handle_device_info(&device, &menu),
            MenuOption::ManualCommand => handle_manual_command(&device, &menu)?,
            MenuOption::DeviceAction => handle_device_action(&device, &menu)?,
            MenuOption::RunTest => handle_run_test(&device, &menu)?,
            MenuOption::BackToMain => unsafe {
                std::ptr::write(&raw mut menu.in_advanced_menu, false);
            },
            MenuOption::Quit => {
                running = false;
                println!("\nDisconnecting...");
                let _ = device.disconnect();
                println!("Disconnected. Goodbye!");
            }
        }
    }

    Ok(())
}

fn handle_inventory(device: &UsbDevice, _menu: &Menu) -> Result<(), RfidError> {
    println!("\nPerforming inventory (scanning for tags)...");
    match UhfRfidApi::inventory(device) {
        Ok(tags) => {
            if tags.is_empty() {
                println!("No tags found.");
            } else {
                println!("Found {} tags:", tags.len());
                for (i, tag) in tags.iter().enumerate() {
                    println!("  {}: {}", i + 1, tag);
                }
            }
            Menu::prompt_to_continue();
            Ok(())
        }
        Err(e) => {
            println!("Inventory failed: {e}");
            Err(e)
        }
    }
}

fn handle_read_tag(device: &UsbDevice, _menu: &Menu) -> Result<(), RfidError> {
    // First, do an inventory to check if tags are in range.
    println!("\nChecking for tags in range...");
    match UhfRfidApi::inventory(device) {
        Err(e) => Err(e),
        Ok(tags) => {
            if tags.is_empty() {
                println!("No tags found in range. Please place a tag near the reader.");
                Menu::prompt_to_continue();
                return Ok(());
            }

            println!("Found {} tags in range.", tags.len());

            // Prompt for memory bank, address, and word count
            let bank = Menu::prompt_for_memory_bank();
            let address = Menu::prompt_for_address();
            let word_count = Menu::prompt_for_word_count();

            println!("\nReading {word_count} words from {bank:?} memory at address {address}...");
            match UhfRfidApi::read(device, bank, address, word_count) {
                Ok(data) => {
                    println!("Read successful!");
                    println!("Data (hex): {}", UhfRfidApi::hex_to_ascii(&data));

                    // If it's EPC or TID, try to display as ASCII too
                    if bank == MemoryBank::Epc || bank == MemoryBank::Tid {
                        print!("Data (ASCII): ");
                        for b in &data {
                            if b.is_ascii() && !b.is_ascii_control() {
                                print!("{}", *b as char);
                            } else {
                                print!(".");
                            }
                        }
                        println!();
                    }
                    Menu::prompt_to_continue();
                    Ok(())
                }
                Err(e) => {
                    println!("Read failed: {e}");
                    Err(e)
                }
            }
        }
    }
}

fn handle_write_tag(device: &UsbDevice, _menu: &Menu) -> Result<(), RfidError> {
    // First, do an inventory to check if tags are in range
    println!("\nChecking for tags in range...");
    let tags = UhfRfidApi::inventory(device)?;

    if tags.is_empty() {
        println!("No tags found in range. Please place a tag near the reader.");
        Menu::prompt_to_continue();
        return Ok(());
    }

    println!("Found {} tags in range.", tags.len());
    if tags.len() > 1 {
        println!("Warning: Multiple tags detected. Write may affect all tags in range.");
        print("Continue? (y/n): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Operation cancelled.");
            Menu::prompt_to_continue();
            return Ok(());
        }
    }

    // Prompt for a memory bank, address, and data
    let bank = Menu::prompt_for_memory_bank();
    let address = Menu::prompt_for_address();
    let data = Menu::prompt_for_hex_data();

    println!(
        "\nWriting {} bytes to {:?} memory at address {}...",
        data.len(),
        bank,
        address
    );
    match UhfRfidApi::write(device, bank, address, &data) {
        Ok(()) => println!("Write successful!"),
        Err(e) => println!("Write failed: {e}"),
    }

    Menu::prompt_to_continue();
    Ok(())
}

fn handle_lock_tag(device: &UsbDevice, _menu: &Menu) -> Result<(), RfidError> {
    // First, do an inventory to check if tags are in range
    println!("\nChecking for tags in range...");
    match UhfRfidApi::inventory(device) {
        Err(e) => Err(e),
        Ok(tags) => {
            if tags.is_empty() {
                println!("No tags found in range. Please place a tag near the reader.");
                Menu::prompt_to_continue();
                return Ok(());
            }

            println!("Found {} tags in range.", tags.len());
            if tags.len() > 1 {
                println!("Warning: Multiple tags detected. Lock may affect all tags in range.");
                print("Continue? (y/n): ");
                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to read input");
                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("Operation cancelled.");
                    Menu::prompt_to_continue();
                    return Ok(());
                }
            }

            // Warning about lock operations
            println!("\n⚠️  WARNING: Lock operations can be PERMANENT depending on settings! ⚠️");
            println!("Some lock actions cannot be reversed. Proceed with caution.");
            print("Continue? (y/n): ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read input");
            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Operation cancelled.");
                Menu::prompt_to_continue();
                return Ok(());
            }

            // Prompt for memory bank and lock action
            let bank = Menu::prompt_for_lockable_memory_bank();
            let action = Menu::prompt_for_lock_action();

            println!("\nLocking {bank:?} memory with {action:?} action...");
            match UhfRfidApi::lock_memory_bank(device, bank, action) {
                Ok(()) => {
                    println!("Lock operation successful!");
                    Menu::prompt_to_continue();
                    Ok(())
                }
                Err(e) => {
                    println!("Lock operation failed: {e}");
                    Err(e)
                }
            }
        }
    }
}

fn handle_set_password(device: &UsbDevice, _menu: &Menu) -> Result<(), RfidError> {
    // First, do an inventory to check if tags are in range
    println!("\nChecking for tags in range...");
    match UhfRfidApi::inventory(device) {
        Err(e) => Err(e),
        Ok(tags) => {
            if tags.is_empty() {
                println!("No tags found in range. Please place a tag near the reader.");
                Menu::prompt_to_continue();
                return Ok(());
            }

            println!("Found {} tags in range.", tags.len());
            if tags.len() > 1 {
                println!(
                    "Warning: Multiple tags detected. Password will be set on all tags in range."
                );
                print("Continue? (y/n): ");
                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to read input");
                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("Operation cancelled.");
                    Menu::prompt_to_continue();
                    return Ok(());
                }
            }

            // Prompt for password
            let password = Menu::prompt_for_password();

            println!("\nSetting access password to {password:08X}...");
            match UhfRfidApi::set_access_password(device, password) {
                Ok(()) => {
                    println!("Password set successfully!");
                    println!("⚠️  IMPORTANT: Make sure to write down this password! ⚠️");
                    println!("If you lose this password, you may not be able to access your tag.");
                    Menu::prompt_to_continue();
                    Ok(())
                }
                Err(e) => {
                    println!("Failed to set password: {e}");
                    Err(e)
                }
            }
        }
    }
}

fn handle_device_info(device: &UsbDevice, _menu: &Menu) {
    println!("\nDevice Information:");
    println!("--------------------");
    println!("Device: {}", device.get_info());
    println!(
        "Connection: Interface {}, IN endpoint 0x{:02X}, OUT endpoint 0x{:02X}",
        device.get_interface(),
        device.get_in_endpoint(),
        device.get_out_endpoint()
    );
    println!("Is connected: {}", device.is_connected());

    // Try to print device capabilities or version if available
    println!("\nChecking device capabilities...");
    // This would depend on your specific device's capabilities
    // Implement if your device supports this

    Menu::prompt_to_continue();
}

fn handle_manual_command(device: &UsbDevice, _menu: &Menu) -> Result<(), RfidError> {
    println!("\nManual Command Mode");
    println!("------------------");
    println!("This mode allows sending raw commands to the device.");
    println!("Commands should be entered as hex bytes (e.g., 01020304)");

    // Warning about manual commands
    println!("\n⚠️  WARNING: Manual commands can damage your device or tags! ⚠️");
    println!("Only use this mode if you know exactly what you're doing.");
    print("Continue? (y/n): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    if !input.trim().eq_ignore_ascii_case("y") {
        println!("Operation cancelled.");
        Menu::prompt_to_continue();
        return Ok(());
    }

    // Get the command data
    let data = Menu::prompt_for_hex_data();

    println!("\nSending command: {}", UhfRfidApi::hex_to_ascii(&data));
    match device.write(&data) {
        Ok(bytes_written) => {
            println!("Sent {bytes_written} bytes. Waiting for response...");

            // Read response
            let mut buffer = [0u8; 256]; // Adjust buffer size as needed
            match device.read(&mut buffer) {
                Ok(bytes_read) if bytes_read > 0 => {
                    let response = &buffer[0..bytes_read];
                    println!("Received {bytes_read} bytes.");
                    println!("Response (hex): {}", UhfRfidApi::hex_to_ascii(response));

                    // Try to display as ASCII if possible
                    print!("Response (ASCII): ");
                    for b in response {
                        if b.is_ascii() && !b.is_ascii_control() {
                            print!("{}", *b as char);
                        } else {
                            print!(".");
                        }
                    }
                    println!();
                }
                Ok(_) => println!("No response received (timeout)."),
                Err(e) => println!("Error reading response: {e}"),
            }
        }
        Err(e) => println!("Error sending command: {e}"),
    }

    Menu::prompt_to_continue();
    Ok(())
}

fn handle_device_action(device: &UsbDevice, _menu: &Menu) -> Result<(), RfidError> {
    println!("\nDevice Action (LED/Beep)");
    println!("Enter actions separated by '+', options: beep, red, green, yellow");
    print("Actions: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    let actions_str = input.trim();

    let mut actions = Vec::new();
    for tok in actions_str.split('+') {
        match tok.trim().to_lowercase().as_str() {
            "beep" => actions.push(DeviceAction::Beep),
            "red" => actions.push(DeviceAction::RedLed),
            "green" => actions.push(DeviceAction::GreenLed),
            "yellow" => actions.push(DeviceAction::YellowLed),
            "" => {}
            other => {
                println!("Unknown action: {other}");
                Menu::prompt_to_continue();
                return Ok(());
            }
        }
    }
    if actions.is_empty() {
        println!("No actions specified.");
        Menu::prompt_to_continue();
        return Ok(());
    }

    print("Duration (deciseconds, default 50): ");
    io::stdout().flush()?;
    input.clear();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    let time = input.trim().parse::<u8>().unwrap_or(50);

    match UhfRfidApi::device_action(device, &actions, time) {
        Ok(()) => println!("Action performed successfully!"),
        Err(e) => println!("Action failed: {e}"),
    }
    Menu::prompt_to_continue();
    Ok(())
}

fn handle_run_test(device: &UsbDevice, _menu: &Menu) -> Result<(), RfidError> {
    println!("\nRunning basic UHF test...");
    // EPC
    println!("\nReading EPC memory bank:");
    let epc = UhfRfidApi::read(device, MemoryBank::Epc, 0, 8)?;
    println!("EPC: {}", UhfRfidApi::hex_to_ascii(&epc));
    // TID
    println!("\nReading TID memory bank:");
    let tid = UhfRfidApi::read(device, MemoryBank::Tid, 0, 8)?;
    println!("TID: {}", UhfRfidApi::hex_to_ascii(&tid));
    // USER
    println!("\nReading USER memory bank:");
    let user = UhfRfidApi::read(device, MemoryBank::User, 0, 8)?;
    println!("USER: {}", UhfRfidApi::hex_to_ascii(&user));
    // Reserved
    println!("\nReading Reserved memory bank:");
    let res = UhfRfidApi::read(device, MemoryBank::Reserved, 0, 8)?;
    println!("Reserved: {}", UhfRfidApi::hex_to_ascii(&res));

    // Action
    println!("\nPerforming beep + green LED action...");
    UhfRfidApi::device_action(device, &[DeviceAction::Beep, DeviceAction::GreenLed], 50)?;
    println!("Test completed successfully.");
    Menu::prompt_to_continue();
    Ok(())
}

// Helper function to avoid having to unwrap every time
fn print(text: &str) {
    print!("{text}");
    io::stdout().flush().unwrap();
}
