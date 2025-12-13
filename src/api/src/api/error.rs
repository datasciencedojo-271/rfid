//! Error types used across the high-level RFID API.

#[allow(unused_imports)]
use protocl::interface::{PRODUCT_ID, VENDOR_ID};
use protocl::types::UhfError;
use std::io;
use thiserror::Error;

/// USB-level errors encountered while interacting with the device
#[derive(Error, Debug)]
pub enum UsbError {
    /// Backend USB error (backend-agnostic string)
    #[error("USB backend error: {0}")]
    Usb(String),

    /// Device with vendor/product IDs not found
    #[error("Device not found: VID={vid:04x}, PID={pid:04x}")]
    DeviceNotFound {
        /// Vendor ID (hex)
        vid: u16,
        /// Product ID (hex)
        pid: u16,
    },

    /// No suitable endpoint found on the device
    #[error("No suitable endpoint found")]
    EndpointNotFound,

    /// Operation timed out
    #[error("Transfer timeout")]
    Timeout,

    /// Access to the USB device was denied by the OS
    #[error("Permission denied accessing USB device. On Linux, try creating a udev rule.")]
    PermissionDenied,

    /// The device is busy or already claimed
    #[error("Device busy or already claimed")]
    DeviceBusy,

    /// Failed to claim the interface on the device
    #[error("Failed to claim interface: {0}")]
    InterfaceClaimFailed(String),

    /// The required USB transfer type is not supported
    #[error("Unsupported Transfer Type")]
    UnsupportedTransferType,

    /// Outgoing command exceeds supported size
    #[error("Command too large: {0} bytes")]
    CommandTooLarge(usize),
}

/// High-level application errors for RFID operations
#[derive(Error, Debug)]
pub enum RfidError {
    /// Wrapped I/O error
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Wrapped USB error
    #[error("USB error: {0}")]
    UsbError(#[from] UsbError),

    /// Error encountered while enumerating USB devices
    #[error("Error iterating USB devices")]
    DeviceEnumerationError(UsbError),

    /// Serial port error with a descriptive message
    #[error("Serial port error: {0}")]
    SerialPort(String),

    /// Operation attempted without an active device connection
    #[error("Device not connected")]
    NotConnected,

    /// No compatible devices were detected when scanning
    #[error("No compatible devices found")]
    NoDevicesFound,

    /// Protocol-level error with a descriptive message
    #[error("Protocol error: {0}")]
    Protocol(String),

    /// Device returned an invalid or unexpected response
    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    /// Device reported a non-zero status code
    #[error("Command failed with status: {0}")]
    CommandFailed(u8),

    /// Operation timed out
    #[error("Timeout waiting for response")]
    Timeout,

    /// Current platform does not support this operation
    #[error("Platform not supported: {0}")]
    PlatformNotSupported(String),

    /// Error encountered during packet fragmentation/assembly
    #[error("Packet fragmentation error: {0}")]
    FragmentationError(String),

    /// Response failed integrity or verification checks
    #[error("Response verification failed")]
    ResponseVerificationFailed,

    /// Wrapped low-level protocol error
    #[error("UhfError library error: {0}")]
    UhfError(#[from] UhfError),
}
