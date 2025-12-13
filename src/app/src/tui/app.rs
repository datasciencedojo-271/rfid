//! Application state for the TUI

use api::rfid_device::usb_device::UsbDevice;
use protocl::types::{LockAction, LockableMemoryBank, MemoryBank};
use strum::{EnumIter, IntoEnumIterator};

/// Application state
pub struct App {
    pub state: AppState,
    pub status_message: String,
    pub device: Option<UsbDevice>,
    pub tags: Vec<TagData>,
    pub selected_menu_item: MenuItem,
    pub active_input_field: usize,
    pub result_data: Vec<u8>,
    // Per-bank results (for All-banks read rendering)
    pub result_epc: Vec<u8>,
    pub result_tid: Vec<u8>,
    pub result_user: Vec<u8>,

    // Track which banks were read in the last read operation
    pub last_read_epc: bool,
    pub last_read_tid: bool,
    pub last_read_user: bool,

    // Read form fields
    pub read_bank: MemoryBank,
    pub read_address: String,
    pub read_word_count: String,

    // Write form fields
    pub write_bank: MemoryBank,
    pub write_address: String,
    pub write_data: String,

    // Lock form fields
    pub lock_bank: LockableMemoryBank,
    pub lock_action: LockAction,

    // Password form field
    pub password: String,

    // Device Actions
    pub action_beep: bool,
    pub action_red: bool,
    pub action_green: bool,
    pub action_yellow: bool,
    pub action_time: String,

    // Raw command form fields
    pub raw_input: String,
    pub raw_response: Vec<u8>,

    // Confirmation gate for risky operations
    pub pending_confirm: Option<PendingConfirm>,
}

/// Tag data structure
pub struct TagData {
    pub epc: String,
    pub data: Vec<u8>,
    pub data_bank: Option<MemoryBank>,
}

/// Application state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    Main,
    Read,
    Write,
    Lock,
    Password,
    Action,
    Raw,
    Test,
}

/// Menu items
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
pub enum MenuItem {
    Inventory,
    ReadTag,
    WriteTag,
    LockTag,
    SetPassword,
    DeviceAction,
    RawCommand,
    RunTest,
    Quit,
}

impl MenuItem {
    pub fn to_string(&self) -> &'static str {
        match self {
            MenuItem::Inventory => "[i] Inventory (Scan for Tags)",
            MenuItem::ReadTag => "[r] Read Tag",
            MenuItem::WriteTag => "[w] Write Tag",
            MenuItem::LockTag => "[l] Lock Tag",
            MenuItem::SetPassword => "[p] Set Password",
            MenuItem::DeviceAction => "[a] Device Action",
            MenuItem::RawCommand => "[m] Manual Raw Command",
            MenuItem::RunTest => "[t] Run Test",
            MenuItem::Quit => "[q] Quit",
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::Main,
            status_message: "Starting...".to_string(),
            device: None,
            tags: Vec::new(),
            selected_menu_item: MenuItem::Inventory,
            active_input_field: 0,
            result_data: Vec::new(),
            result_epc: Vec::new(),
            result_tid: Vec::new(),
            result_user: Vec::new(),

            last_read_epc: false,
            last_read_tid: false,
            last_read_user: false,

            read_bank: MemoryBank::Epc,
            read_address: "0".to_string(),
            read_word_count: "4".to_string(),

            write_bank: MemoryBank::Epc,
            write_address: "0".to_string(),
            write_data: String::new(),

            lock_bank: LockableMemoryBank::Epc,
            lock_action: LockAction::SecureWriteable,

            // Default password
            password: String::new(),

            action_beep: false,
            action_red: false,
            action_green: false,
            action_yellow: false,
            action_time: "50".to_string(),

            raw_input: String::new(),
            raw_response: Vec::new(),

            pending_confirm: None,
        }
    }

    pub fn on_tick(&mut self) {
        // Update state on timer tick
        // This could include checking device connection, updating inventory, etc.
    }

    pub fn next_menu_item(&mut self) {
        let items: Vec<MenuItem> = MenuItem::iter().collect();
        let current_pos = items
            .iter()
            .position(|&item| item == self.selected_menu_item)
            .unwrap_or(0);
        let next_pos = (current_pos + 1) % items.len();
        self.selected_menu_item = items[next_pos];
    }

    pub fn previous_menu_item(&mut self) {
        let items: Vec<MenuItem> = MenuItem::iter().collect();
        let current_pos = items
            .iter()
            .position(|&item| item == self.selected_menu_item)
            .unwrap_or(0);
        let next_pos = if current_pos == 0 {
            items.len() - 1
        } else {
            current_pos - 1
        };
        self.selected_menu_item = items[next_pos];
    }

    pub fn next_input_field(&mut self) {
        match self.state {
            AppState::Read => {
                self.active_input_field = (self.active_input_field + 1) % 3;
            }
            AppState::Write => {
                self.active_input_field = (self.active_input_field + 1) % 3;
            }
            AppState::Lock => {
                self.active_input_field = (self.active_input_field + 1) % 2;
            }
            AppState::Password => {
                // Only one field
            }
            AppState::Raw => {
                // Only one field
            }
            _ => {}
        }
    }

    pub fn prev_input_field(&mut self) {
        match self.state {
            AppState::Read => {
                self.active_input_field = if self.active_input_field == 0 {
                    2
                } else {
                    self.active_input_field - 1
                };
            }
            AppState::Write => {
                self.active_input_field = if self.active_input_field == 0 {
                    2
                } else {
                    self.active_input_field - 1
                };
            }
            AppState::Lock => {
                self.active_input_field = usize::from(self.active_input_field == 0);
            }
            AppState::Password => {
                // Only one field
            }
            AppState::Raw => {
                // Only one field
            }
            _ => {}
        }
    }

    pub fn input_char(&mut self, c: char) {
        // Only allow valid characters for each field
        match self.state {
            AppState::Read => {
                match self.active_input_field {
                    0 => { /* Bank selection is handled separately */ }
                    1 => {
                        // Address - only digits
                        if c.is_ascii_digit() {
                            self.read_address.push(c);
                        }
                    }
                    2 => {
                        // Word count - only digits
                        if c.is_ascii_digit() {
                            self.read_word_count.push(c);
                        }
                    }
                    _ => {}
                }
            }
            AppState::Write => {
                match self.active_input_field {
                    0 => { /* Bank selection is handled separately */ }
                    1 => {
                        // Address - only digits
                        if c.is_ascii_digit() {
                            self.write_address.push(c);
                        }
                    }
                    2 => {
                        // Data - hex digits
                        if c.is_ascii_hexdigit() {
                            self.write_data.push(c);
                        }
                    }
                    _ => {}
                }
            }
            AppState::Lock => {
                // Selection-based fields, no character input
            }
            AppState::Password => {
                // Password - hex digits
                if c.is_ascii_hexdigit() {
                    self.password.push(c);
                }
            }
            AppState::Raw => {
                // Raw input - hex digits and spaces allowed (we'll ignore spaces)
                if c.is_ascii_hexdigit() {
                    self.raw_input.push(c);
                }
            }
            _ => {}
        }
    }

    pub fn input_backspace(&mut self) {
        match self.state {
            AppState::Read => match self.active_input_field {
                1 => {
                    self.read_address.pop();
                }
                2 => {
                    self.read_word_count.pop();
                }
                _ => {}
            },
            AppState::Write => match self.active_input_field {
                1 => {
                    self.write_address.pop();
                }
                2 => {
                    self.write_data.pop();
                }
                _ => {}
            },
            AppState::Password => {
                self.password.pop();
            }
            AppState::Raw => {
                self.raw_input.pop();
            }
            _ => {}
        }
    }

    // Cycle through memory bank options
    pub fn cycle_read_bank(&mut self) {
        self.read_bank = match self.read_bank {
            MemoryBank::Reserved => MemoryBank::Epc,
            MemoryBank::Epc => MemoryBank::Tid,
            MemoryBank::Tid => MemoryBank::User,
            MemoryBank::User => MemoryBank::Reserved,
        };
    }

    // Cycle backwards through memory bank options
    pub fn prev_read_bank(&mut self) {
        self.read_bank = match self.read_bank {
            MemoryBank::Reserved => MemoryBank::User,
            MemoryBank::Epc => MemoryBank::Reserved,
            MemoryBank::Tid => MemoryBank::Epc,
            MemoryBank::User => MemoryBank::Tid,
        };
    }

    // Check if we should read all memory banks
    pub fn read_all_banks(&self) -> bool {
        self.read_bank == MemoryBank::Reserved
    }

    // Helpers to reset/mark last-read flags
    pub fn clear_last_read_flags(&mut self) {
        self.last_read_epc = false;
        self.last_read_tid = false;
        self.last_read_user = false;
    }
    pub fn mark_last_read(&mut self, bank: MemoryBank) {
        match bank {
            MemoryBank::Epc => self.last_read_epc = true,
            MemoryBank::Tid => self.last_read_tid = true,
            MemoryBank::User => self.last_read_user = true,
            MemoryBank::Reserved => {
                // Treat as all
                self.last_read_epc = true;
                self.last_read_tid = true;
                self.last_read_user = true;
            }
        }
    }

    pub fn cycle_write_bank(&mut self) {
        self.write_bank = match self.write_bank {
            MemoryBank::Epc => MemoryBank::Tid,
            MemoryBank::Tid => MemoryBank::User,
            MemoryBank::User => MemoryBank::Epc,
            _ => MemoryBank::Epc,
        };
    }

    // Cycle backwards through write memory bank options
    pub fn prev_write_bank(&mut self) {
        self.write_bank = match self.write_bank {
            MemoryBank::Epc => MemoryBank::User,
            MemoryBank::Tid => MemoryBank::Epc,
            MemoryBank::User => MemoryBank::Tid,
            _ => MemoryBank::Epc,
        };
    }

    pub fn cycle_lock_bank(&mut self) {
        self.lock_bank = match self.lock_bank {
            LockableMemoryBank::KillPassword => LockableMemoryBank::AccessPassword,
            LockableMemoryBank::AccessPassword => LockableMemoryBank::Epc,
            LockableMemoryBank::Epc => LockableMemoryBank::Tid,
            LockableMemoryBank::Tid => LockableMemoryBank::User,
            LockableMemoryBank::User => LockableMemoryBank::KillPassword,
        };
    }

    pub fn prev_lock_bank(&mut self) {
        self.lock_bank = match self.lock_bank {
            LockableMemoryBank::KillPassword => LockableMemoryBank::User,
            LockableMemoryBank::AccessPassword => LockableMemoryBank::KillPassword,
            LockableMemoryBank::Epc => LockableMemoryBank::AccessPassword,
            LockableMemoryBank::Tid => LockableMemoryBank::Epc,
            LockableMemoryBank::User => LockableMemoryBank::Tid,
        };
    }

    pub fn cycle_lock_action(&mut self) {
        self.lock_action = match self.lock_action {
            LockAction::Writeable => LockAction::PermanentlyWriteable,
            LockAction::PermanentlyWriteable => LockAction::SecureWriteable,
            LockAction::SecureWriteable => LockAction::NotWriteable,
            LockAction::NotWriteable => LockAction::Writeable,
        };
    }

    pub fn prev_lock_action(&mut self) {
        self.lock_action = match self.lock_action {
            LockAction::Writeable => LockAction::NotWriteable,
            LockAction::PermanentlyWriteable => LockAction::Writeable,
            LockAction::SecureWriteable => LockAction::PermanentlyWriteable,
            LockAction::NotWriteable => LockAction::SecureWriteable,
        };
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PendingConfirm {
    Write,
    Lock,
    Raw,
}
