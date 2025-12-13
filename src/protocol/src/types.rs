//! Constants and types for UHF RFID communication protocol.
//! Contains memory bank enums, device actions, inventory results, and errors.
use std::fmt;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use thiserror::Error;

/// Memory locking actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockAction {
    /// Memory bank is writeable without a password
    Writeable,
    /// Memory bank is permanently writeable and can never be locked
    PermanentlyWriteable,
    /// Memory bank is writeable only with the correct ACCESS password
    SecureWriteable,
    /// Memory bank is not writeable (locked)
    NotWriteable,
}

/// Password locking actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasswordLockAction {
    /// Password is readable and writeable without a password
    ReadWriteable,
    /// Password is permanently readable and writeable and can never be locked
    PermanentlyReadWriteable,
    /// Password is readable and writeable only with the correct ACCESS password
    SecureReadWriteable,
    /// Password is not readable or writeable (locked)
    NotReadWriteable,
}

/// Memory bank selection for locking operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockableMemoryBank {
    /// User memory bank
    User,
    /// TID memory bank
    Tid,
    /// EPC memory bank
    Epc,
    /// Access password
    AccessPassword,
    /// Kill password
    KillPassword,
}

/// Memory Banks enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryBank {
    /// EPC memory bank
    Epc = 1,
    /// TID memory bank
    Tid = 2,
    /// User memory bank
    User = 3,
    /// Reserved memory bank
    Reserved = 4,
}

impl Hash for MemoryBank {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (*self as u8).hash(state);
    }
}

impl From<u8> for MemoryBank {
    fn from(value: u8) -> Self {
        match value {
            1 => MemoryBank::Epc,
            2 => MemoryBank::Tid,
            3 => MemoryBank::User,
            _ => MemoryBank::Reserved, // Default to Reserved for all other values
        }
    }
}

impl fmt::Display for MemoryBank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryBank::Reserved => write!(f, "Reserved"),
            MemoryBank::Epc => write!(f, "EPC"),
            MemoryBank::Tid => write!(f, "TID"),
            MemoryBank::User => write!(f, "User"),
        }
    }
}

impl MemoryBank {
    /// Convert the memory bank value to its ASCII digit representation.
    #[must_use]
    pub fn to_ascii(&self) -> u8 {
        (*self as u8) + 48 // Convert to ASCII digit
    }
}

impl DeviceAction {
    /// Combine multiple device actions into a single bitmask.
    #[must_use]
    pub fn combine(actions: &[DeviceAction]) -> u8 {
        actions.iter().fold(0u8, |acc, a| acc | (*a as u8))
    }
}

/// Device Action enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceAction {
    /// Trigger buzzer
    Beep = 0x01,
    /// Turn on red LED
    RedLed = 0x02,
    /// Turn on green LED
    GreenLed = 0x04,
    /// Turn on yellow LED
    YellowLed = 0x08,
}

/// Inventory Read results with the read count
pub struct InventoryResult {
    /// Electronic Product Code of the tag
    pub epc: String,
    /// Number of times the tag was read
    pub read_count: u8,
}

impl fmt::Display for InventoryResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (Read Count: {})", self.epc, self.read_count)
    }
}

/// Error types for UHF operations (backend-agnostic)
#[derive(Error, Debug)]
pub enum UhfError {
    /// RFID device could not be found
    #[error("Device not found")]
    DeviceNotFound,

    /// Invalid or stale device handle
    #[error("Invalid device handle")]
    InvalidHandle,

    /// Input parameter failed validation
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Communication layer error (e.g., USB I/O)
    #[error("Communication error: {0}")]
    Communication(String),

    /// Timed out waiting for a device response
    #[error("Timeout waiting for response")]
    Timeout,

    /// Device returned an invalid or unexpected response
    #[error("Invalid response from device")]
    InvalidResponse,
}

/// Minimal USB I/O abstraction to decouple protocol from a concrete backend (rusb/nusb)
pub trait UsbIo {
    /// Reads from device
    ///
    /// # Arguments
    ///
    /// * `endpoint`: device write endpoint address
    /// * `data`: data in bytes
    /// * `timeout`: timeout for the read operation
    ///
    /// returns: Result<usize, UhfError>
    ///
    fn read_bulk(
        &self,
        endpoint: u8,
        buf: &mut [u8],
        timeout: Duration,
    ) -> crate::interface::Result<usize>;

    /// Writes to device
    ///
    /// # Arguments
    ///
    /// * `endpoint`: device write endpoint address
    /// * `data`: data in bytes
    /// * `timeout`: timeout for the write operation
    ///
    /// returns: Result<usize, UhfError>
    fn write_bulk(
        &self,
        endpoint: u8,
        data: &[u8],
        timeout: Duration,
    ) -> crate::interface::Result<usize>;
}
