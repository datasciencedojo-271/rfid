//!
//! This Rust library provides an interface to control the UHF-U6-CU-91 RFID reader
//! from `Fongwah Technology Co., Ltd.` It supports connecting via USB, reading/writing
//! tag memory (EPC, TID, USER, reserved), controlling buzzer/LEDs, setting access
//! passwords, locking memory, and scanning multiple EPCs.
//!
//! # Warning
//! Use with caution! Write operations can permanently alter or lock RFID tags,
//! risking data loss or tag unusability. Test with disposable tags first.
//!

pub use crate::types::{MemoryBank, UhfError, UsbIo};
use std::time::{Duration, Instant};

/// USB Device constants
/// Vendor ID for the RFID reader USB device (Fongwah Technology Co., Ltd)
pub const VENDOR_ID: u16 = 0x0e6a; // Fongwah Technology Co., Ltd
/// Product ID for the RFID reader USB device
pub const PRODUCT_ID: u16 = 0x0317; // RFID Reader
/// Default endpoint address for writing, use only as a fallback
pub const ENDPOINT_OUT: u8 = 0x03; // EP 3 OUT
/// Default endpoint address for reading, use only as a fallback
pub const ENDPOINT_IN: u8 = 0x82; // EP 2 IN

/// Convenient result alias for protocol operations
pub type Result<T> = std::result::Result<T, UhfError>;

/// UHF RFID Reader device handle
pub struct Interface {
    /// When enabled, prints protocol I/O for troubleshooting
    pub debug_mode: bool,
}

impl Interface {
    /// Enable or disable debug output
    pub fn set_debug_mode(&mut self, enabled: bool) {
        self.debug_mode = enabled;
    }

    /// Send a command to the device
    fn send_command(&self, device: &impl UsbIo, data: &[u8]) -> Result<()> {
        let mut bytes_written = 0;

        while bytes_written < data.len() {
            let end = (bytes_written + 64).min(data.len());
            let mut chunk = data[bytes_written..end].to_vec();
            chunk.resize(64, 0); // Pad to 64 bytes

            if data.len() > 64 && bytes_written + 64 < data.len() {
                chunk[0] = 0x82; // Continuation marker
            } else if data.len() > 64 {
                chunk[0] = 0x02; // End marker
            }

            let written = device.write_bulk(ENDPOINT_OUT, &chunk, Duration::from_secs(2))?;

            if self.debug_mode {
                println!(
                    "Sent command chunk: {} (Bytes written: {})",
                    hex::encode_upper(&chunk),
                    written
                );
            }

            bytes_written += 64;
        }

        Ok(())
    }

    /// Read a response from the device
    fn read_response(&self, device: &impl UsbIo, timeout: Duration) -> Result<Vec<u8>> {
        let mut response = Vec::new();
        let start_time = Instant::now();

        loop {
            let mut chunk = [0u8; 64];
            match device.read_bulk(ENDPOINT_IN, &mut chunk, timeout) {
                Ok(len) => {
                    response.extend_from_slice(&chunk[..len]);

                    if self.debug_mode {
                        println!(
                            "Received response chunk: {} (Length: {})",
                            hex::encode_upper(&chunk[..len]),
                            len
                        );
                    }

                    if chunk[0] != 63 || start_time.elapsed() > timeout {
                        break;
                    }
                    if len < 64 {
                        break;
                    }
                }
                Err(UhfError::Timeout) => {
                    if self.debug_mode {
                        println!("No data received within timeout period");
                    }
                    return Err(UhfError::Timeout);
                }
                Err(e) => return Err(e),
            }
        }

        if self.debug_mode && !response.is_empty() {
            println!(
                "Full response: {} (Length: {})",
                hex::encode_upper(&response),
                response.len()
            );
        }

        if response.is_empty() {
            Err(UhfError::InvalidResponse)
        } else {
            Ok(response)
        }
    }

    /// Convert value to hex character (ASCII)
    fn hex_value(value: u8) -> u8 {
        if value >= 10 {
            value + 55 // 'A'-'F'
        } else {
            value + 48 // '0'-'9'
        }
    }

    /// Read data from the UHF tag
    ///
    /// # Arguments
    /// * `memory_bank` - Memory bank type
    /// * `address` - Start address
    /// * `r_len` - Length of data to read (will get `r_len`*4 bytes)
    ///
    /// # Returns
    /// Vector containing the read data
    ///
    /// # Errors
    /// Returns an error if USB communication fails, times out, or the device response is invalid.
    pub fn read(
        &self,
        device: &impl UsbIo,
        memory_bank: MemoryBank,
        address: u8,
        r_len: u8,
    ) -> Result<Vec<u8>> {
        let mut command = vec![0u8; 256];
        command[1] = 2;
        command[2] = b'A';
        command[3] = b'R';
        command[4] = memory_bank.to_ascii();
        command[5] = b',';

        let command_length = if address >= 0x10 {
            command[6] = Self::hex_value(address >> 4);
            command[7] = Self::hex_value(address & 0xF);
            command[8] = b',';
            command[9] = if r_len >= 10 { r_len + 55 } else { r_len + 48 };
            command[0] = 9;
            10
        } else {
            command[6] = Self::hex_value(address);
            command[7] = b',';
            command[8] = if r_len >= 10 { r_len + 55 } else { r_len + 48 };
            command[0] = 8;
            9
        };

        self.send_command(device, &command[..command_length])?;
        let response = self.read_response(device, Duration::from_secs(2))?;

        if response.len() >= 6 && response[4] == b'R' {
            let data_length = (r_len as usize) * 4;
            let data = response[5..].to_vec();

            if self.debug_mode {
                println!(
                    "Read data: {}",
                    hex::encode(&data[..data_length.min(data.len())])
                );
            }

            Ok(data)
        } else {
            Err(UhfError::InvalidResponse)
        }
    }

    /// Write data to UHF tag
    ///
    /// # Arguments
    /// * `memory_bank` - Memory bank type
    /// * `address` - Start address
    /// * `data` - Data to write (length must be multiple of 4)
    ///
    /// # Errors
    /// Returns an error if parameters are invalid, USB communication fails, times out, or the device response is invalid.
    pub fn write(
        &self,
        device: &impl UsbIo,
        memory_bank: MemoryBank,
        address: u8,
        data: &[u8],
    ) -> Result<()> {
        if data.is_empty() || !data.len().is_multiple_of(4) {
            return Err(UhfError::InvalidParameter(
                "Data length must be a non-zero multiple of 4".to_string(),
            ));
        }

        let w_len = u8::try_from(data.len() / 4)
            .map_err(|_| UhfError::InvalidParameter("Data length too large".to_string()))?;
        let mut command = vec![0u8; 256];
        command[1] = 2;
        command[2] = b'A';
        command[3] = b'W';
        command[4] = memory_bank.to_ascii();
        command[5] = b',';

        let command_length = if address >= 0x10 {
            command[6] = Self::hex_value(address >> 4);
            command[7] = Self::hex_value(address & 0xF);
            command[8] = b',';
            command[9] = if w_len >= 10 { w_len + 55 } else { w_len + 48 };
            command[10] = b',';
            command[11..11 + data.len()].copy_from_slice(data);
            11 + data.len()
        } else {
            command[6] = Self::hex_value(address);
            command[7] = b',';
            command[8] = if w_len >= 10 { w_len + 55 } else { w_len + 48 };
            command[9] = b',';
            command[10..10 + data.len()].copy_from_slice(data);
            10 + data.len()
        };

        command[0] = u8::try_from(command_length - 1)
            .map_err(|_| UhfError::InvalidParameter("Command length too large".to_string()))?;

        self.send_command(device, &command[..command_length])?;
        let response = self.read_response(device, Duration::from_secs(2))?;

        if !response.is_empty() && response[0] == 8 {
            if self.debug_mode {
                println!("Write data: {} - Success", hex::encode(data));
            }
            Ok(())
        } else {
            Err(UhfError::InvalidResponse)
        }
    }

    /// Control buzzer and LED
    ///
    /// # Arguments
    /// * `action` - Action bitmask (use `ActionType` constants)
    /// * `time_units` - Duration in units of 10ms
    ///
    /// # Errors
    /// Returns an error if parameters are invalid, USB communication fails, times out, or the device response is invalid.
    pub fn action(&self, device: &impl UsbIo, action: u8, time_units: u8) -> Result<()> {
        if action == 0 || action > 15 {
            return Err(UhfError::InvalidParameter(
                "Invalid action value".to_string(),
            ));
        }

        let command = vec![4, 2, 145, action, time_units];
        self.send_command(device, &command)?;

        let response = self.read_response(device, Duration::from_secs(2))?;

        if response.len() >= 4
            && response[0] == 3
            && response[1] == 2
            && response[2] == 145
            && response[3] == 0
        {
            if self.debug_mode {
                println!(
                    "Action (beep/led) executed: action={}, time={}ms",
                    action,
                    time_units * 10
                );
            }
            Ok(())
        } else {
            Err(UhfError::InvalidResponse)
        }
    }

    /// Set access password for tag
    ///
    /// # Arguments
    /// * `password` - 8-byte password (two 4-byte integers)
    ///
    /// # Errors
    /// Returns an error if USB communication fails, times out, or the device response is invalid.
    pub fn set_access_password(&self, device: &impl UsbIo, password: &[u8; 8]) -> Result<()> {
        let mut command = vec![0u8; 12];
        command[0] = 11; // Length
        command[1] = 2; // Protocol marker
        command[2] = b'A';
        command[3] = b'P';
        command[4..8].copy_from_slice(&password[..4]);
        command[8..12].copy_from_slice(&password[4..]);

        self.send_command(device, &command)?;
        let response = self.read_response(device, Duration::from_secs(2))?;

        if response.len() >= 5
            && response[0] == 4
            && response[1] == 2
            && response[2] == 65
            && response[3] == 0
            && response[4] == 80
        {
            if self.debug_mode {
                println!("Set Access Password: {} - Success", hex::encode(password));
            }
            Ok(())
        } else {
            Err(UhfError::InvalidResponse)
        }
    }

    /// Lock memory bank
    ///
    /// # Arguments
    /// * `lock_setting` - 6-byte ASCII lock setting
    ///
    /// # Errors
    /// Returns an error if USB communication fails, times out, or the device response is invalid.
    pub fn lock_memory(&self, device: &impl UsbIo, lock_setting: &[u8; 6]) -> Result<()> {
        let mut command = vec![0u8; 11];
        command[0] = 10; // Length
        command[1] = 2; // Protocol marker
        command[2] = b'A';
        command[3] = b'L';
        command[4..10].copy_from_slice(lock_setting);

        self.send_command(device, &command)?;
        let response = self.read_response(device, Duration::from_secs(2))?;

        if response.len() >= 8
            && response[0] == 8
            && response[4] == 76
            && response[6] == 79
            && response[7] == 75
        {
            if self.debug_mode {
                println!("Lock Memory: {} - Success", hex::encode(lock_setting));
            }
            Ok(())
        } else {
            Err(UhfError::InvalidResponse)
        }
    }

    /// Get the raw EPC list (first 47 hex characters of valid responses)
    ///
    /// # Returns
    /// Vector of hex strings representing detected tags
    ///
    /// # Errors
    /// Returns an error if USB communication fails, times out, or the device response is invalid.
    pub fn get_epc_list_raw(&self, device: &impl UsbIo) -> Result<Vec<String>> {
        let mut responses = Vec::new();

        // Start inventory command
        let command = vec![3, 2, 0x55, 0x80];
        self.send_command(device, &command)?;

        // Read initial response
        let _ = self.read_response(device, Duration::from_secs(2))?;

        // Get up to 10 tag responses
        for _ in 0..10 {
            let followup = vec![3, 2, 0x55, 0x91];
            if self.send_command(device, &followup).is_err() {
                break;
            }

            match self.read_response(device, Duration::from_secs(2)) {
                Ok(resp) if resp.len() >= 5 => {
                    let hex_str = hex::encode(&resp);
                    let short_hex = if hex_str.len() >= 47 {
                        &hex_str[..47]
                    } else {
                        &hex_str
                    };

                    // // Skip invalid responses
                    if short_hex != "04025591000000000000000000000000000000000000000" {
                        responses.push(short_hex.to_string());
                    }
                }
                _ => break,
            }
        }

        Ok(responses)
    }
}
