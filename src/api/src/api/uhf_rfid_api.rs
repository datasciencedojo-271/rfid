use crate::api::error::RfidError;
use crate::api::lock_pattern_builder::LockPatternBuilder;
use crate::rfid_device::usb_device::UsbDevice;
use protocl::interface::Interface;
use protocl::types::{
    DeviceAction, InventoryResult, LockAction, LockableMemoryBank, MemoryBank, PasswordLockAction,
};

/// High-level UHF RFID operations built on top of the protocol layer.
pub struct UhfRfidApi {}

impl UhfRfidApi {
    fn get_interface(usb_device: &UsbDevice) -> Result<Interface, RfidError> {
        if usb_device.is_connected() {
            Ok(Interface { debug_mode: false })
        } else {
            Err(RfidError::NotConnected)
        }
    }

    /// Control device indicators (LED and beep)
    /// The time parameter is in deciseconds (10 ms units)
    /// Control device indicators (LED and beep)
    /// The time parameter is in deciseconds (10 ms units)
    ///
    /// # Errors
    /// Returns an error if the device is not connected or USB communication fails.
    pub fn device_action(
        usb_device: &UsbDevice,
        actions: &[DeviceAction],
        time: u8,
    ) -> Result<(), RfidError> {
        let interface = Self::get_interface(usb_device)?;
        let action_byte = DeviceAction::combine(actions);
        if !usb_device.is_connected() {
            return Err(RfidError::NotConnected);
        }
        // Pass the UsbDevice itself (implements UsbIo) to the protocol interface
        interface.action(usb_device, action_byte, time)?;
        Ok(())
    }

    /// Read data from a memory bank
    /// Read data from a memory bank
    ///
    /// # Errors
    /// Returns an error if the device is not connected or USB communication fails.
    pub fn read(
        usb_device: &UsbDevice,
        bank: MemoryBank,
        address: u32,
        word_count: u32,
    ) -> Result<Vec<u8>, RfidError> {
        let interface = Self::get_interface(usb_device)?;
        let address = u8::try_from(address)
            .map_err(|_| RfidError::Protocol("Address out of range".to_owned()))?;
        let word_count = u8::try_from(word_count)
            .map_err(|_| RfidError::Protocol("Word count out of range".to_owned()))?;
        if !usb_device.is_connected() {
            return Err(RfidError::NotConnected);
        }
        let data = interface.read(usb_device, bank, address, word_count)?;
        Ok(data)
    }

    /// Write data to a memory bank
    /// Write data to a memory bank
    ///
    /// # Errors
    /// Returns an error if parameters are invalid, the device is not connected, or USB communication fails.
    pub fn write(
        usb_device: &UsbDevice,
        bank: MemoryBank,
        address: u32,
        data: &[u8],
    ) -> Result<(), RfidError> {
        if !data.len().is_multiple_of(4) {
            return Err(RfidError::Protocol(
                "Data length must be a multiple of 4".to_string(),
            ));
        }
        let interface = Self::get_interface(usb_device)?;
        if !usb_device.is_connected() {
            return Err(RfidError::NotConnected);
        }
        let address = u8::try_from(address)
            .map_err(|_| RfidError::Protocol("Address out of range".to_owned()))?;
        interface.write(usb_device, bank, address, data)?;
        Ok(())
    }

    /// Lock a memory bank with the specified action
    /// Lock a memory bank with the specified action
    ///
    /// # Errors
    /// Returns an error if the device is not connected or USB communication fails.
    pub fn lock_memory_bank(
        usb_device: &UsbDevice,
        bank: LockableMemoryBank,
        action: LockAction,
    ) -> Result<(), RfidError> {
        Self::lock_memory_raw(
            usb_device,
            LockPatternBuilder::memory_bank(bank, action, true),
        )
    }

    /// Lock memory using a raw lock pattern string
    /// Lock memory using a raw lock pattern string
    ///
    /// # Errors
    /// Returns an error if the device is not connected or USB communication fails.
    pub fn lock_memory_raw(usb_device: &UsbDevice, pattern: u16) -> Result<(), RfidError> {
        let interface = Self::get_interface(usb_device)?;
        let pattern_str = format!("{pattern:04X}");
        let pattern_bytes: [u8; 6] = pattern_str
            .as_bytes()
            .try_into()
            .map_err(|_| RfidError::Protocol("Invalid lock pattern".to_string()))?;
        if !usb_device.is_connected() {
            return Err(RfidError::NotConnected);
        }
        interface.lock_memory(usb_device, &pattern_bytes)?;
        Ok(())
    }

    /// Perform an inventory operation to find tags
    /// Perform an inventory operation to find tags
    ///
    /// # Errors
    /// Returns an error if the device is not connected or USB communication fails.
    pub fn inventory(usb_device: &UsbDevice) -> Result<Vec<InventoryResult>, RfidError> {
        let interface = Self::get_interface(usb_device)?;
        if !usb_device.is_connected() {
            return Err(RfidError::NotConnected);
        }
        let epc_list = interface.get_epc_list_raw(usb_device)?;
        let read_count = u8::try_from(epc_list.len())
            .map_err(|_| RfidError::Protocol("Too many tags in one inventory".to_owned()))?;
        let tags = epc_list
            .into_iter()
            .map(|epc| InventoryResult { epc, read_count })
            .collect();
        Ok(tags)
    }

    /// Set the access password for secured operations
    /// Set the access password for secured operations
    ///
    /// # Errors
    /// Returns an error if the device is not connected or USB communication fails.
    pub fn set_access_password(usb_device: &UsbDevice, password: u32) -> Result<(), RfidError> {
        let interface = Self::get_interface(usb_device)?;
        let password_bytes = password.to_be_bytes();
        let full_password: [u8; 8] = [
            password_bytes[0],
            password_bytes[1],
            password_bytes[2],
            password_bytes[3],
            0,
            0,
            0,
            0, // Second password set to zeros
        ];
        if !usb_device.is_connected() {
            return Err(RfidError::NotConnected);
        }
        interface.set_access_password(usb_device, &full_password)?;
        Ok(())
    }

    /// Lock a password bank with the specified action
    /// Lock a password bank with the specified action
    ///
    /// # Errors
    /// Returns an error if the bank is not a password bank or USB communication fails.
    pub fn lock_password(
        usb_device: &UsbDevice,
        bank: LockableMemoryBank,
        action: PasswordLockAction,
    ) -> Result<(), RfidError> {
        if bank != LockableMemoryBank::AccessPassword && bank != LockableMemoryBank::KillPassword {
            return Err(RfidError::Protocol(
                "Only Access and Kill passwords can be locked with this method".to_string(),
            ));
        }
        Self::lock_memory_raw(usb_device, LockPatternBuilder::password(bank, action, true))
    }

    /// Utility function to convert ASCII hex string to bytes
    /// Utility function to convert ASCII hex string to bytes
    ///
    /// # Errors
    /// Returns an error if the string contains non-hex characters or has odd length.
    pub fn ascii_to_hex(hex_string: &str) -> Result<Vec<u8>, RfidError> {
        if !hex_string.len().is_multiple_of(2) {
            return Err(RfidError::Protocol("Invalid hex string".to_string()));
        }
        let mut result = Vec::with_capacity(hex_string.len() / 2);
        let bytes = hex_string.as_bytes();
        for i in (0..bytes.len()).step_by(2) {
            let a = bytes[i] as char;
            let b = bytes[i + 1] as char;
            let byte = u8::from_str_radix(&format!("{a}{b}"), 16)
                .map_err(|_| RfidError::Protocol("Invalid hex string".to_string()))?;
            result.push(byte);
        }
        Ok(result)
    }

    /// Utility function to convert bytes to ASCII hex string
    #[must_use]
    pub fn hex_to_ascii(bytes: &[u8]) -> String {
        let mut s = String::with_capacity(bytes.len() * 2);
        for b in bytes {
            use std::fmt::Write as _;
            let _ = write!(&mut s, "{b:02X}");
        }
        s
    }

    /// Utility function to convert a byte to its ASCII hex representation
    #[must_use]
    pub fn byte_to_ascii_hex(byte: u8) -> (u8, u8) {
        let high = byte >> 4;
        let low = byte & 0x0F;
        let high_char = if high >= 0x0A {
            high + 55 // 'A'-'F'
        } else {
            high + 48 // '0'-'9'
        };
        let low_char = if low >= 0x0A {
            low + 55 // 'A'-'F'
        } else {
            low + 48 // '0'-'9'
        };
        (high_char, low_char)
    }
}
