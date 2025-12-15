//! Menu system for the RFID CLI application

use api::api::uhf_rfid_api::UhfRfidApi;
use protocl::types::{LockAction, LockableMemoryBank, MemoryBank};
use std::fmt;
use std::io::{self, Write};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MenuOption {
    // Main menu options
    Inventory,
    ReadTag,
    WriteTag,
    LockTag,
    SetPassword,
    AdvancedOptions,
    Quit,

    // Advanced menu options
    ShowDeviceInfo,
    ManualCommand,
    DeviceAction,
    RunTest,
    BackToMain,
}

impl MenuOption {
    pub fn from_index(index: usize, is_advanced_menu: bool) -> Option<Self> {
        if is_advanced_menu {
            match index {
                1 => Some(Self::ShowDeviceInfo),
                2 => Some(Self::ManualCommand),
                3 => Some(Self::DeviceAction),
                4 => Some(Self::RunTest),
                5 => Some(Self::BackToMain),
                _ => None,
            }
        } else {
            match index {
                1 => Some(Self::Inventory),
                2 => Some(Self::ReadTag),
                3 => Some(Self::WriteTag),
                4 => Some(Self::LockTag),
                5 => Some(Self::SetPassword),
                6 => Some(Self::AdvancedOptions),
                7 => Some(Self::Quit),
                _ => None,
            }
        }
    }

    pub fn string_value(self) -> &'static str {
        match self {
            // Main menu options
            Self::Inventory => "Inventory (scan for tags)",
            Self::ReadTag => "Read tag memory",
            Self::WriteTag => "Write to tag memory",
            Self::LockTag => "Lock tag memory",
            Self::SetPassword => "Set access password",
            Self::AdvancedOptions => "Advanced options",
            Self::Quit => "Quit",

            // Advanced menu options
            Self::ShowDeviceInfo => "Show device information",
            Self::ManualCommand => "Send manual command",
            Self::DeviceAction => "Device action (LED/beep)",
            Self::RunTest => "Run basic test",
            Self::BackToMain => "Back to main menu",
        }
    }
}

pub enum MemoryBankOption {
    Reserved,
    Epc,
    Tid,
    User,
}

impl fmt::Display for MemoryBankOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reserved => write!(f, "Reserved"),
            Self::Epc => write!(f, "EPC"),
            Self::Tid => write!(f, "TID"),
            Self::User => write!(f, "User"),
        }
    }
}

impl MemoryBankOption {
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            1 => Some(Self::Reserved),
            2 => Some(Self::Epc),
            3 => Some(Self::Tid),
            4 => Some(Self::User),
            _ => None,
        }
    }

    pub fn to_protocol_bank(&self) -> MemoryBank {
        match self {
            Self::Reserved => MemoryBank::Reserved,
            Self::Epc => MemoryBank::Epc,
            Self::Tid => MemoryBank::Tid,
            Self::User => MemoryBank::User,
        }
    }
}

pub enum LockActionOption {
    Writeable,
    PermanentlyWriteable,
    SecureWriteable,
    NotWriteable,
}

impl fmt::Display for LockActionOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Writeable => write!(f, "Writeable (no protection)"),
            Self::PermanentlyWriteable => write!(f, "Permanently Writeable"),
            Self::SecureWriteable => write!(f, "Secure Writeable (requires password)"),
            Self::NotWriteable => write!(f, "Not Writeable (locked)"),
        }
    }
}

impl LockActionOption {
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            1 => Some(Self::Writeable),
            2 => Some(Self::PermanentlyWriteable),
            3 => Some(Self::SecureWriteable),
            4 => Some(Self::NotWriteable),
            _ => None,
        }
    }

    pub fn to_protocol_action(&self) -> LockAction {
        match self {
            Self::Writeable => LockAction::Writeable,
            Self::PermanentlyWriteable => LockAction::PermanentlyWriteable,
            Self::SecureWriteable => LockAction::SecureWriteable,
            Self::NotWriteable => LockAction::NotWriteable,
        }
    }
}

pub struct Menu {
    pub in_advanced_menu: bool,
}

impl Menu {
    pub fn new() -> Self {
        Self {
            in_advanced_menu: false,
        }
    }

    pub fn display(&self) {
        Self::clear_screen();
        if self.in_advanced_menu {
            Self::display_advanced_menu();
        } else {
            Self::display_main_menu();
        }
    }

    fn display_main_menu() {
        println!("\n┌──────────────────────────────────┐");
        println!("│    UHF RFID Programmer - Menu    │");
        println!("└──────────────────────────────────┘\n");
        println!("1. {}", MenuOption::Inventory.string_value());
        println!("2. {}", MenuOption::ReadTag.string_value());
        println!("3. {}", MenuOption::WriteTag.string_value());
        println!("4. {}", MenuOption::LockTag.string_value());
        println!("5. {}", MenuOption::SetPassword.string_value());
        println!("6. {}", MenuOption::AdvancedOptions.string_value());
        println!("7. {}", MenuOption::Quit.string_value());
    }

    fn display_advanced_menu() {
        println!("\n┌──────────────────────────────────┐");
        println!("│  UHF RFID Programmer - Advanced  │");
        println!("└──────────────────────────────────┘\n");
        println!("1. {}", MenuOption::ShowDeviceInfo.string_value());
        println!("2. {}", MenuOption::ManualCommand.string_value());
        println!("3. {}", MenuOption::DeviceAction.string_value());
        println!("4. {}", MenuOption::RunTest.string_value());
        println!("5. {}", MenuOption::BackToMain.string_value());
    }

    pub fn get_choice(&self) -> MenuOption {
        loop {
            print!("\nEnter your choice: ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read input");

            match input.trim().parse::<usize>() {
                Ok(choice) => {
                    if let Some(option) = MenuOption::from_index(choice, self.in_advanced_menu) {
                        return option;
                    }
                    println!("Invalid choice. Please try again.");
                }
                Err(_) => println!("Invalid input. Please enter a number."),
            }
        }
    }

    pub fn prompt_for_memory_bank() -> MemoryBank {
        println!("\nSelect memory bank:");
        println!("1. Reserved (passwords)");
        println!("2. EPC");
        println!("3. TID");
        println!("4. User");

        loop {
            print("Enter your choice: ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read input");

            match input.trim().parse::<usize>() {
                Ok(choice) => {
                    if let Some(bank) = MemoryBankOption::from_index(choice) {
                        return bank.to_protocol_bank();
                    }
                    println!("Invalid choice. Please try again.");
                }
                Err(_) => println!("Invalid input. Please enter a number."),
            }
        }
    }

    pub fn prompt_for_lockable_memory_bank() -> LockableMemoryBank {
        println!("\nSelect memory bank to lock:");
        println!("1. Kill Password");
        println!("2. Access Password");
        println!("3. EPC");
        println!("4. TID");
        println!("5. User");

        loop {
            print("Enter your choice: ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read input");

            match input.trim().parse::<usize>() {
                Ok(choice) => match choice {
                    1 => return LockableMemoryBank::KillPassword,
                    2 => return LockableMemoryBank::AccessPassword,
                    3 => return LockableMemoryBank::Epc,
                    4 => return LockableMemoryBank::Tid,
                    5 => return LockableMemoryBank::User,
                    _ => println!("Invalid choice. Please try again."),
                },
                Err(_) => println!("Invalid input. Please enter a number."),
            }
        }
    }

    pub fn prompt_for_lock_action() -> LockAction {
        println!("\nSelect lock action:");
        println!("1. Writeable (no protection)");
        println!("2. Permanently Writeable");
        println!("3. Secure Writeable (requires password)");
        println!("4. Not Writeable (locked)");

        loop {
            print("Enter your choice: ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read input");

            match input.trim().parse::<usize>() {
                Ok(choice) => {
                    if let Some(action) = LockActionOption::from_index(choice) {
                        return action.to_protocol_action();
                    }
                    println!("Invalid choice. Please try again.");
                }
                Err(_) => println!("Invalid input. Please enter a number."),
            }
        }
    }

    pub fn prompt_for_address() -> u32 {
        loop {
            print("Enter address (decimal): ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read input");

            match input.trim().parse::<u32>() {
                Ok(address) => return address,
                Err(_) => println!("Invalid input. Please enter a valid number."),
            }
        }
    }

    pub fn prompt_for_word_count() -> u32 {
        loop {
            print("Enter word count (1 word = 2 bytes): ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read input");

            match input.trim().parse::<u32>() {
                Ok(count) if count > 0 => return count,
                Ok(_) => println!("Word count must be greater than 0."),
                Err(_) => println!("Invalid input. Please enter a valid number."),
            }
        }
    }

    pub fn prompt_for_hex_data() -> Vec<u8> {
        loop {
            print("Enter hex data (e.g., 01020304): ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read input");

            let hex_str = input.trim();
            if !hex_str.len().is_multiple_of(2) {
                println!("Hex data must have an even number of characters.");
                continue;
            }

            // Try to parse the hex string
            match UhfRfidApi::ascii_to_hex(hex_str) {
                Ok(data) => return data,
                Err(_) => println!("Invalid hex data. Use only 0-9 and A-F characters."),
            }
        }
    }

    pub fn prompt_for_password() -> u32 {
        loop {
            print("Enter password (hexadecimal, 8 characters): ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read input");

            let hex_str = input.trim();
            if hex_str.len() != 8 {
                println!("Password must be exactly 8 hex characters (32 bits).");
                continue;
            }

            // Try to parse the hex string
            match u32::from_str_radix(hex_str, 16) {
                Ok(password) => return password,
                Err(_) => println!("Invalid hex data. Use only 0-9 and A-F characters."),
            }
        }
    }

    pub fn prompt_to_continue() {
        println!("\nPress Enter to continue...");
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
    }

    fn clear_screen() {
        // Print ANSI escape code to clear the screen
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().unwrap();
    }
}

// Helper function to avoid having to unwrap every time
fn print(text: &str) {
    print!("{text}");
    io::stdout().flush().unwrap();
}
