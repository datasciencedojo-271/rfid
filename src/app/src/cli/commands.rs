//! Command definitions for the RFID CLI application

use api::api::uhf_rfid_api::UhfRfidApi;
use clap::{Args, Parser, Subcommand};
use protocl::types::{LockAction, LockableMemoryBank, MemoryBank};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct CliArguments {
    /// Enable debug output
    #[arg(short, long)]
    pub debug: bool,

    /// Use command-line mode instead of TUI
    #[arg(long)]
    pub cli: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Scan for RFID tags in range
    Inventory,

    /// Read data from a tag
    Read(ReadArgs),

    /// Write data to a tag
    Write(WriteArgs),

    /// Lock a memory bank on a tag
    Lock(LockArgs),

    /// Set an access password for a tag
    Password(PasswordArgs),

    /// Get device information
    DeviceInfo,

    /// Send a raw command to the device (advanced)
    RawCommand(RawCommandArgs),

    /// Run the application in legacy interactive menu mode
    Interactive,

    /// Control device actions (LED/beep)
    Action(ActionArgs),

    /// Run a basic test sequence
    Test,
}

#[derive(Args)]
pub struct ReadArgs {
    /// Memory bank to read from (reserved, epc, tid, user)
    #[arg(short, long, value_parser = parse_memory_bank)]
    pub bank: MemoryBank,

    /// Starting address for read operation
    #[arg(short, long, default_value = "0")]
    pub address: u32,

    /// Number of words to read (1 word = 2 bytes)
    #[arg(short, long, default_value = "4")]
    pub words: u32,
}

#[derive(Args)]
pub struct WriteArgs {
    /// Memory bank to write to (reserved, epc, tid, user)
    #[arg(short, long, value_parser = parse_memory_bank)]
    pub bank: MemoryBank,

    /// Starting address for write operation
    #[arg(short, long, default_value = "0")]
    pub address: u32,

    /// Data to write (hexadecimal string, e.g., 01020304)
    #[arg(short, long, value_parser = parse_hex_data)]
    pub data: Vec<u8>,
}

#[derive(Args)]
pub struct LockArgs {
    /// Memory bank to lock (`kill_password`, `access_password`, epc, tid, user)
    #[arg(short, long, value_parser = parse_lockable_memory_bank)]
    pub bank: LockableMemoryBank,

    /// Lock action to perform (writeable, permanent, secure, locked)
    #[arg(short, long, value_parser = parse_lock_action)]
    pub action: LockAction,

    /// Skip confirmation prompt (use with caution)
    #[arg(short, long)]
    pub force: bool,
}

#[derive(Args)]
pub struct PasswordArgs {
    /// Password value (hexadecimal, 8 characters, e.g., 12345678)
    #[arg(short, long, value_parser = parse_password)]
    pub password: u32,

    /// Skip confirmation prompt
    #[arg(short, long)]
    pub force: bool,
}

#[derive(Args)]
pub struct RawCommandArgs {
    /// Raw command data (hexadecimal string, e.g., 01020304)
    #[arg(short, long, value_parser = parse_hex_data)]
    pub data: Vec<u8>,

    /// Skip safety warning
    #[arg(short, long)]
    pub force: bool,
}

#[derive(Args, Debug)]
pub struct ActionArgs {
    /// Actions to perform (beep, red, green, yellow)
    #[arg(required = true)]
    pub actions: String,

    /// Duration in deciseconds (10ms units), default is 50 (500ms)
    #[arg(short, long, default_value = "50")]
    pub time: u8,
}

// Custom parser functions
fn parse_memory_bank(arg: &str) -> Result<MemoryBank, String> {
    use MemoryBank;
    match arg.to_lowercase().as_str() {
        "reserved" => Ok(MemoryBank::Reserved),
        "epc" => Ok(MemoryBank::Epc),
        "tid" => Ok(MemoryBank::Tid),
        "user" => Ok(MemoryBank::User),
        _ => Err(format!(
            "Invalid memory bank: {arg}. Use 'reserved', 'epc', 'tid', or 'user'"
        )),
    }
}

fn parse_lockable_memory_bank(arg: &str) -> Result<LockableMemoryBank, String> {
    use LockableMemoryBank;
    match arg.to_lowercase().as_str() {
        "kill_password" | "kill" => Ok(LockableMemoryBank::KillPassword),
        "access_password" | "access" => Ok(LockableMemoryBank::AccessPassword),
        "epc" => Ok(LockableMemoryBank::Epc),
        "tid" => Ok(LockableMemoryBank::Tid),
        "user" => Ok(LockableMemoryBank::User),
        _ => Err(format!(
            "Invalid lockable memory bank: {arg}. Use 'kill_password', 'access_password', 'epc', 'tid', or 'user'"
        )),
    }
}

fn parse_lock_action(arg: &str) -> Result<LockAction, String> {
    match arg.to_lowercase().as_str() {
        "writeable" | "normal" => Ok(LockAction::Writeable),
        "permanent" | "permanently_writeable" => Ok(LockAction::PermanentlyWriteable),
        "secure" | "secure_writeable" => Ok(LockAction::SecureWriteable),
        "locked" | "not_writeable" => Ok(LockAction::NotWriteable),
        _ => Err(format!(
            "Invalid lock action: {arg}. Use 'writeable', 'permanent', 'secure', or 'locked'"
        )),
    }
}

fn parse_hex_data(arg: &str) -> Result<Vec<u8>, String> {
    if !arg.len().is_multiple_of(2) {
        return Err("Hex data must have an even number of characters".to_string());
    }

    UhfRfidApi::ascii_to_hex(arg)
        .map_err(|_| "Invalid hex data. Use only 0-9 and A-F characters".to_string())
}

fn parse_password(arg: &str) -> Result<u32, String> {
    if arg.len() > 8 {
        return Err("Password must be at most 8 hex characters (32 bits)".to_string());
    }

    u32::from_str_radix(arg, 16)
        .map_err(|_| "Invalid hex data. Use only 0-9 and A-F characters".to_string())
}
