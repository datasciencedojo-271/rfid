//! Terminal User Interface for the RFID application

use std::io::stdout;
use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{event, ExecutableCommand};
use ratatui::prelude::*;

use api::api::error::RfidError;
use api::api::uhf_rfid_api::UhfRfidApi;
use api::rfid_device::usb_device::UsbDevice;

mod app;
mod components;

pub use app::{App, AppState, MenuItem, PendingConfirm, TagData};
use protocl::types::{DeviceAction, MemoryBank};

pub fn run_tui() -> Result<(), RfidError> {
    // Set up terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // Create app state
    let app = App::new();

    // Run the application
    let result = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    // Handle result
    if let Err(err) = result {
        println!("Error: {err}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<(), RfidError> {
    // Try to connect to a device
    match UsbDevice::new() {
        Ok(device) => {
            app.device = Some(device);
            app.status_message =
                format!("Connected to {}", app.device.as_ref().unwrap().get_info());

            // Get initial inventory
            update_inventory(&mut app);
        }
        Err(e) => {
            app.status_message = format!("Failed to connect to device: {e}");
        }
    }

    let tick_rate = Duration::from_millis(100);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| components::main_ui::draw(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::ZERO);

        if event::poll(timeout)?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Esc => {
                    app.state = AppState::Main;
                }
                KeyCode::Char('q') => {
                    if app.state == AppState::Main {
                        if let Some(ref mut device) = app.device {
                            let _ = device.disconnect();
                        }
                        return Ok(());
                    }
                    app.state = AppState::Main;
                }
                KeyCode::Char('i') => {
                    if app.state == AppState::Main {
                        update_inventory(&mut app);
                    }
                }
                KeyCode::Char('r') => {
                    if app.state == AppState::Main {
                        app.state = AppState::Read;
                        app.active_input_field = 0;
                    } else if app.state == AppState::Read {
                        handle_read(&mut app);
                    }
                }
                KeyCode::Char('w') => {
                    if app.state == AppState::Main {
                        app.state = AppState::Write;
                        app.active_input_field = 0;
                    } else if app.state == AppState::Write {
                        handle_write(&mut app);
                    }
                }
                KeyCode::Char('l') => {
                    if app.state == AppState::Main {
                        app.state = AppState::Lock;
                        app.active_input_field = 0;
                    } else if app.state == AppState::Lock {
                        handle_lock(&mut app);
                    }
                }
                KeyCode::Char('p') => {
                    if app.state == AppState::Main {
                        app.state = AppState::Password;
                    } else if app.state == AppState::Password {
                        handle_password(&mut app);
                    }
                }
                KeyCode::Char('a') => {
                    if app.state == AppState::Main {
                        app.state = AppState::Action;
                        app.active_input_field = 0;
                    }
                }
                KeyCode::Char('m') => {
                    if app.state == AppState::Main {
                        app.state = AppState::Raw;
                        app.active_input_field = 0;
                    } else if app.state == AppState::Raw
                        && let Err(e) = handle_raw(&mut app)
                    {
                        app.status_message = format!("Raw command failed: {e}");
                    }
                }
                KeyCode::Char('t') => {
                    if app.state == AppState::Main {
                        app.state = AppState::Test;
                    } else if app.state == AppState::Test
                        && let Err(e) = handle_test(&mut app)
                    {
                        app.status_message = format!("Test failed: {e}");
                    }
                }
                KeyCode::Char(' ') => {
                    // Toggle action flags based on the selected field
                    match app.active_input_field {
                        0 => app.action_beep = !app.action_beep,
                        1 => app.action_red = !app.action_red,
                        2 => app.action_green = !app.action_green,
                        3 => app.action_yellow = !app.action_yellow,
                        _ => {}
                    }
                }
                KeyCode::Tab => {
                    app.next_input_field();
                }
                KeyCode::BackTab => {
                    app.prev_input_field();
                }
                KeyCode::Backspace => {
                    app.input_backspace();
                }
                KeyCode::Char(c) => {
                    app.input_char(c);
                }
                KeyCode::Enter => {
                    if app.state == AppState::Main {
                        // Handle menu selection based on the selected item
                        match app.selected_menu_item {
                            MenuItem::Inventory => update_inventory(&mut app),
                            // Initialize focus to the bank field when entering a Read form
                            MenuItem::ReadTag => {
                                app.state = AppState::Read;
                                app.active_input_field = 0;
                            }
                            MenuItem::WriteTag => {
                                app.state = AppState::Write;
                                app.active_input_field = 0;
                            }
                            MenuItem::LockTag => {
                                app.state = AppState::Lock;
                                app.active_input_field = 0;
                            }
                            MenuItem::SetPassword => app.state = AppState::Password,
                            MenuItem::DeviceAction => {
                                app.state = AppState::Action;
                                app.active_input_field = 0;
                            }
                            MenuItem::RawCommand => {
                                app.state = AppState::Raw;
                            }
                            MenuItem::RunTest => {
                                app.state = AppState::Test;
                            }
                            MenuItem::Quit => {
                                // Disconnect device before quitting
                                if let Some(ref mut device) = app.device {
                                    let _ = device.disconnect();
                                }
                                return Ok(());
                            }
                        }
                    } else {
                        match app.state {
                            AppState::Read => handle_read(&mut app),
                            AppState::Write => handle_write(&mut app),
                            AppState::Lock => handle_lock(&mut app),
                            AppState::Password => handle_password(&mut app),
                            AppState::Action => {
                                if let Err(e) = handle_action(&mut app) {
                                    app.status_message = format!("Action failed: {e}");
                                }
                            }
                            AppState::Raw => {
                                if let Err(e) = handle_raw(&mut app) {
                                    app.status_message = format!("Raw command failed: {e}");
                                }
                            }
                            AppState::Test => {
                                if let Err(e) = handle_test(&mut app) {
                                    app.status_message = format!("Test failed: {e}");
                                }
                            }
                            _ => {}
                        }
                    }
                }
                KeyCode::Up => {
                    if app.state == AppState::Main {
                        app.previous_menu_item();
                    } else if app.state == AppState::Action {
                        // When in action state, cycle through the action flags
                        if app.active_input_field > 0 {
                            app.active_input_field -= 1;
                        }
                    }
                }
                KeyCode::Down => {
                    if app.state == AppState::Main {
                        app.next_menu_item();
                    } else if app.state == AppState::Action {
                        // When in action state, cycle through the action flags
                        if app.active_input_field < 3 {
                            // Since we have 4 actions (0-3)
                            app.active_input_field += 1;
                        }
                    } else if app.state == AppState::Read {
                        // No vertical navigation inside the read bank row
                    }
                }
                KeyCode::Right => {
                    // Cycle options with Right arrow when the focus is on a selectable row
                    if app.active_input_field == 0 {
                        match app.state {
                            AppState::Read => app.cycle_read_bank(),
                            AppState::Write => app.cycle_write_bank(),
                            AppState::Lock => app.cycle_lock_bank(),
                            _ => {}
                        }
                    } else if app.state == AppState::Lock && app.active_input_field == 1 {
                        // Cycle lock action when focused
                        app.cycle_lock_action();
                    }
                }
                KeyCode::Left => {
                    // Cycle options backwards with Left arrow
                    if app.active_input_field == 0 {
                        match app.state {
                            AppState::Read => app.prev_read_bank(),
                            AppState::Write => app.prev_write_bank(),
                            AppState::Lock => app.prev_lock_bank(),
                            _ => {}
                        }
                    } else if app.state == AppState::Lock && app.active_input_field == 1 {
                        // Cycle lock action backwards when focused
                        app.prev_lock_action();
                    }
                }
                _ => {}
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
    }
}

fn update_inventory(app: &mut App) {
    app.status_message = "Scanning for tags...".to_owned();
    if let Some(ref device) = app.device {
        match UhfRfidApi::inventory(device) {
            Ok(tags) => {
                app.tags = tags
                    .iter()
                    .map(|tag| TagData {
                        epc: tag.to_string(),
                        data: Vec::new(),
                        data_bank: None,
                    })
                    .collect();

                if app.tags.is_empty() {
                    app.status_message = "No tags found.".to_owned();
                } else {
                    app.status_message = format!("Found {} tags.", app.tags.len());
                }
            }
            Err(e) => {
                app.status_message = format!("Inventory failed: {e}");
            }
        }
    } else {
        app.status_message = "No device connected.".to_owned();
    }
}

fn handle_read(app: &mut App) {
    app.status_message = "Reading tag...".to_owned();

    if let Some(ref device) = app.device {
        // Ensure at least one tag is present so the device has a target
        match UhfRfidApi::inventory(device) {
            Ok(tags) => {
                if tags.is_empty() {
                    app.status_message =
                        "No tags in range. Please place a tag near the reader.".to_owned();
                    return;
                }
            }
            Err(e) => {
                app.status_message = format!("Inventory failed: {e}");
                // Don't clear previous results; allow the user to see the last successful read
                return;
            }
        }

        let address = app.read_address.parse::<u32>().unwrap_or(0);
        let word_count = app.read_word_count.parse::<u32>().unwrap_or(4);

        // Check if we're reading all banks or a specific bank
        if app.read_all_banks() {
            // Read all memory banks
            let banks = [MemoryBank::Epc, MemoryBank::Tid, MemoryBank::User];
            let mut combined_data = Vec::new();
            let mut success = true;
            let mut error_msg = String::new();

            // Clear previous per-bank results before a new all-banks read
            app.result_epc.clear();
            app.result_tid.clear();
            app.result_user.clear();

            for bank in &banks {
                match UhfRfidApi::read(device, *bank, address, word_count) {
                    Ok(data) => {
                        // Save per-bank results and reuse stored vec to avoid extra cloning
                        let stored = match bank {
                            MemoryBank::Epc => &mut app.result_epc,
                            MemoryBank::Tid => &mut app.result_tid,
                            MemoryBank::User => &mut app.result_user,
                            MemoryBank::Reserved => continue,
                        };
                        *stored = data;

                        // Store data for EPC bank in the first tag (for a quick view)
                        if *bank == MemoryBank::Epc
                            && let Some(tag) = app.tags.first_mut()
                        {
                            tag.data = stored.clone();
                            tag.data_bank = Some(*bank);
                        }
                        combined_data.extend_from_slice(stored);
                    }
                    Err(e) => {
                        success = false;
                        error_msg = e.to_string();
                        break;
                    }
                }
            }

            if success {
                let total = combined_data.len();
                app.status_message =
                    format!("Read all banks successfully (total {total} byte(s)).");
                app.result_data = combined_data;
                // Mark that effectively all banks were read
                app.clear_last_read_flags();
                app.mark_last_read(MemoryBank::Reserved);
            } else {
                app.status_message = format!("Read failed: {error_msg}");
            }
        } else {
            // Read a specific bank
            let bank = app.read_bank;
            match UhfRfidApi::read(device, bank, address, word_count) {
                Ok(data) => {
                    app.status_message = format!("Read {bank:?} successful!");
                    app.result_data = data.clone();

                    // Update per-bank buffers: set the selected bank buffer, clear others
                    match bank {
                        MemoryBank::Epc => {
                            app.result_epc = data.clone();
                            app.result_tid.clear();
                            app.result_user.clear();
                        }
                        MemoryBank::Tid => {
                            app.result_tid = data.clone();
                            app.result_epc.clear();
                            app.result_user.clear();
                        }
                        MemoryBank::User => {
                            app.result_user = data.clone();
                            app.result_epc.clear();
                            app.result_tid.clear();
                        }
                        MemoryBank::Reserved => {}
                    }

                    // Mark last-read flags: only this bank is from the last operation
                    app.clear_last_read_flags();
                    app.mark_last_read(bank);

                    // Update tag data if it matches one of our tags
                    if let Some(tag) = app.tags.first_mut() {
                        tag.data = data;
                        tag.data_bank = Some(bank);
                    }
                }
                Err(e) => {
                    app.status_message = format!("Read failed: {e}");
                }
            }
        }
    } else {
        app.status_message = "No device connected.".to_owned();
    }
}

fn handle_write(app: &mut App) {
    let Some(device) = app.device.as_ref() else {
        app.status_message = "No device connected.".to_owned();
        return;
    };

    // Confirm multi-tag and risk before proceeding
    match UhfRfidApi::inventory(device) {
        Ok(tags) => {
            if tags.is_empty() {
                app.status_message =
                    "No tags in range. Please place a tag near the reader.".to_owned();
                return;
            }
            if tags.len() > 1 && app.pending_confirm != Some(PendingConfirm::Write) {
                app.pending_confirm = Some(PendingConfirm::Write);
                app.status_message =
                    "Warning: Multiple tags detected. Press Enter again to confirm write."
                        .to_owned();
                return;
            }
        }
        Err(e) => {
            app.status_message = format!("Inventory failed: {e}");
            return;
        }
    }

    app.status_message = "Writing to tag...".to_owned();

    let bank = app.write_bank;
    let address = app.write_address.parse::<u32>().unwrap_or(0);

    // Parse hex data
    if let Ok(data) = UhfRfidApi::ascii_to_hex(&app.write_data) {
        match UhfRfidApi::write(device, bank, address, &data) {
            Ok(()) => {
                app.status_message = "Write successful!".to_owned();
                app.pending_confirm = None;
                // Update inventory to reflect changes
                update_inventory(app);
            }
            Err(e) => {
                app.status_message = format!("Write failed: {e}");
                app.pending_confirm = None;
            }
        }
    } else {
        app.status_message = "Invalid hex data format.".to_owned();
        app.pending_confirm = None;
    }
}

fn handle_lock(app: &mut App) {
    let Some(device) = app.device.as_ref() else {
        app.status_message = "No device connected.".to_owned();
        return;
    };

    // Always warn about permanence and multi-tag situations
    match UhfRfidApi::inventory(device) {
        Ok(tags) => {
            if tags.is_empty() {
                app.status_message =
                    "No tags in range. Please place a tag near the reader.".to_owned();
                return;
            }
            if app.pending_confirm != Some(PendingConfirm::Lock) {
                let warn = if tags.len() > 1 {
                    "WARNING: Lock operations can be permanent! Press Enter again to confirm. Multiple tags detected; operation may affect all.".to_owned()
                } else {
                    "WARNING: Lock operations can be permanent! Press Enter again to confirm."
                        .to_owned()
                };
                app.pending_confirm = Some(PendingConfirm::Lock);
                app.status_message = warn;
                return;
            }
        }
        Err(e) => {
            app.status_message = format!("Inventory failed: {e}");
            return;
        }
    }

    app.status_message = "Locking tag...".to_owned();
    match UhfRfidApi::lock_memory_bank(device, app.lock_bank, app.lock_action) {
        Ok(()) => {
            app.status_message = "Lock operation successful!".to_owned();
            app.pending_confirm = None;
        }
        Err(e) => {
            app.status_message = format!("Lock operation failed: {e}");
            app.pending_confirm = None;
        }
    }
}

fn handle_password(app: &mut App) {
    if app.tags.is_empty() {
        app.status_message = "No tags in range. Please place a tag near the reader.".to_owned();
        return;
    }

    app.status_message = "Setting password...".to_owned();

    if let Some(device) = app.device.as_ref() {
        // Parse password
        match u32::from_str_radix(&app.password, 16) {
            Ok(password) => match UhfRfidApi::set_access_password(device, password) {
                Ok(()) => {
                    app.status_message = "Password set successfully!".to_owned();
                }
                Err(e) => {
                    app.status_message = format!("Failed to set password: {e}");
                }
            },
            Err(_) => {
                app.status_message =
                    "Invalid password format. Use hex format (e.g. 12345678).".to_owned();
            }
        }
    } else {
        app.status_message = "No device connected.".to_owned();
    }
}

fn handle_action(app: &mut App) -> Result<(), RfidError> {
    let Some(device) = app.device.as_ref() else {
        app.status_message = "No device connected".to_owned();
        return Ok(());
    };

    let mut actions = Vec::new();
    if app.action_beep {
        actions.push(DeviceAction::Beep);
    }
    if app.action_red {
        actions.push(DeviceAction::RedLed);
    }
    if app.action_green {
        actions.push(DeviceAction::GreenLed);
    }
    if app.action_yellow {
        actions.push(DeviceAction::YellowLed);
    }

    if actions.is_empty() {
        app.status_message = "No actions selected".to_owned();
        return Ok(());
    }

    let time: u8 = if let Ok(t) = app.action_time.parse() {
        t
    } else {
        app.status_message = "Invalid time value".to_owned();
        return Ok(());
    };

    // Perform the actions
    UhfRfidApi::device_action(device, &actions, time)?;
    app.status_message = "Device actions performed successfully".to_owned();
    app.state = AppState::Main;

    Ok(())
}

fn handle_raw(app: &mut App) -> Result<(), RfidError> {
    let Some(device) = app.device.as_ref() else {
        app.status_message = "No device connected.".to_owned();
        return Ok(());
    };

    // validate input
    let data = if let Ok(d) = UhfRfidApi::ascii_to_hex(&app.raw_input) {
        d
    } else {
        app.status_message = "Invalid hex string (must be even length and 0-9A-F).".to_owned();
        return Ok(());
    };

    if app.pending_confirm != Some(PendingConfirm::Raw) {
        app.pending_confirm = Some(PendingConfirm::Raw);
        app.status_message = "Manual command is risky. Press Enter again to send.".to_owned();
        return Ok(());
    }

    // send
    let written = device.write(&data)?;
    app.status_message = format!("Sent {written} bytes. Waiting for response...");

    let mut buffer = [0u8; 256];
    match device.read(&mut buffer) {
        Ok(bytes_read) if bytes_read > 0 => {
            app.raw_response = buffer[..bytes_read].to_vec();
            app.status_message = format!("Received {bytes_read} bytes.");
        }
        Ok(_) => {
            app.raw_response.clear();
            app.status_message = "No response received (timeout).".to_owned();
        }
        Err(e) => {
            app.raw_response.clear();
            app.status_message = format!("Error reading response: {e}");
        }
    }

    app.pending_confirm = None;
    Ok(())
}

fn handle_test(app: &mut App) -> Result<(), RfidError> {
    let Some(device) = app.device.as_ref() else {
        app.status_message = "No device connected.".to_owned();
        return Ok(());
    };
    // Basic test similar to CLI
    UhfRfidApi::read(device, MemoryBank::Epc, 0, 8)?;
    UhfRfidApi::read(device, MemoryBank::Tid, 0, 8)?;
    UhfRfidApi::read(device, MemoryBank::User, 0, 8)?;
    UhfRfidApi::read(device, MemoryBank::Reserved, 0, 4)?;
    UhfRfidApi::device_action(device, &[DeviceAction::Beep, DeviceAction::GreenLed], 50)?;
    app.status_message = "Test sequence completed successfully.".to_owned();
    Ok(())
}
