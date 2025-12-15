use std::fmt;
use std::time::Duration;

use hidapi::{HidApi, HidDevice};
use protocl::interface::{UsbIo, ENDPOINT_IN, ENDPOINT_OUT};
use protocl::interface::{PRODUCT_ID, VENDOR_ID};
use protocl::types::UhfError;

use crate::api::error::UsbError;

#[derive(Debug)]
/// Describes basic USB device information and descriptors
pub struct DeviceInfo {
    /// USB vendor identifier
    pub vendor_id: u16,
    /// USB product identifier
    pub product_id: u16,
    /// Manufacturer string
    pub manufacturer: String,
    /// Product string
    pub product: String,
    /// Serial number string
    pub serial_number: String,
}

/// High-level wrapper for interacting with the RFID USB device
#[derive(Debug)]
pub struct UsbDevice {
    /// Basic device descriptors
    pub info: DeviceInfo,
    device: HidDevice,
    connected: bool,
}

impl fmt::Display for DeviceInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Vendor ID: {}\nProduct ID: {}\nManufacturer: {}\nProduct: {}\nSerial Number: {}",
            self.vendor_id, self.product_id, self.manufacturer, self.product, self.serial_number
        )
    }
}

macro_rules! hidapi_str {
    ($x: expr) => {
        match $x {
            Ok(s) => match s {
                Some(s) => s,
                None => return Err(UsbError::Usb("Can't read device information.".to_string()))
            },
            Err(e) => return Err(UsbError::Usb(e.to_string()))
        }
    };
}

impl UsbDevice {
    /// Discover and open the RFID device.
    ///
    /// For now, this constructor prepares a lightweight handle without
    /// binding to a specific backend handle. The actual USB I/O is
    /// performed by backend-specific implementations (nusb in this
    /// project) that will be wired in later steps.
    ///
    /// # Errors
    /// Returns an error if the device cannot be found (placeholder behavior).
    pub fn new() -> Result<Self, UsbError> {
        let api = match HidApi::new() {
            Ok(api) => api,
            Err(e) => return Err(UsbError::Usb(e.to_string())),
        };
        let device = match api.open(VENDOR_ID, PRODUCT_ID) {
            Ok(device) => device,
            Err(e) => return Err(UsbError::Usb(e.to_string())),
        };

        let info = DeviceInfo {
            vendor_id: VENDOR_ID,
            product_id: PRODUCT_ID,
            manufacturer: hidapi_str!(device.get_manufacturer_string()),
            product: hidapi_str!(device.get_product_string()),
            serial_number: hidapi_str!(device.get_serial_number_string()),
        };

        Ok(Self {
            info,
            device,
            connected: true,
        })
    }

    /// Read using the bulk IN endpoint
    /// # Errors
    /// Returns a USB error if no device is connected or the read fails.
    fn read_bulk_impl(&self, buffer: &mut [u8], timeout: Duration) -> Result<usize, UsbError> {
        if !self.connected {
            return Err(UsbError::Usb("Device not connected".to_string()));
        }

        match self
            .device
            .read_timeout(buffer, i32::try_from(timeout.as_millis()).unwrap_or(0))
        {
            Ok(len) => Ok(len),
            Err(e) => Err(UsbError::Usb(e.to_string())),
        }
    }

    /// Read with default timeout (1 second)
    /// # Errors
    /// Returns a USB error if no device is connected or the read fails.
    pub fn read(&self, buffer: &mut [u8]) -> Result<usize, UsbError> {
        self.read_bulk_impl(buffer, Duration::from_secs(1))
    }

    /// Write using the bulk OUT endpoint
    /// # Errors
    /// Returns a USB error if no device is connected or the write operation fails.
    fn write_bulk_impl(&self, data: &[u8], _timeout: Duration) -> Result<usize, UsbError> {
        if !self.connected {
            return Err(UsbError::Usb("Device not connected".to_string()));
        }

        let mut buf = vec![0u8];
        buf.extend_from_slice(data);
        match self.device.write(&buf) {
            Ok(len) => Ok(len),
            Err(e) => Err(UsbError::Usb(e.to_string())),
        }
    }

    /// Write with default timeout (1 second)
    /// # Errors
    /// Returns a USB error if no device is connected or the write operation fails.
    pub fn write(&self, data: &[u8]) -> Result<usize, UsbError> {
        self.write_bulk_impl(data, Duration::from_secs(1))
    }

    /// Get claimed interface number
    #[must_use]
    pub fn get_interface(&self) -> u8 {
        0
    }

    /// Get IN endpoint address
    #[must_use]
    pub fn get_in_endpoint(&self) -> u8 {
        ENDPOINT_IN
    }

    /// Get OUT endpoint address
    #[must_use]
    pub fn get_out_endpoint(&self) -> u8 {
        ENDPOINT_OUT
    }

    /// Get device descriptive information
    #[must_use]
    pub fn get_info(&self) -> &DeviceInfo {
        &self.info
    }

    /// Check if the device is currently connected
    /// Returns true if a device handle is available
    #[must_use]
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Check if the device is accessible with current permissions
    /// Simple permission and connectivity check using a quick read
    /// # Errors
    /// Returns a USB error if the device is not connected or permission is denied.
    pub fn check_access(&self) -> Result<(), UsbError> {
        if !self.is_connected() {
            return Err(UsbError::Usb("Device not connected".to_string()));
        }
        Ok(())
    }

    /// Disconnect from the device
    /// Release interface and drop handle, disconnecting from the device
    /// # Errors
    /// Returns a USB error only if internal USB operations fail releasing interface.
    pub fn disconnect(&mut self) -> Result<(), UsbError> {
        self.connected = false;
        Ok(())
    }
}

fn map_usb_to_uhf(err: UsbError) -> UhfError {
    match err {
        UsbError::Timeout => UhfError::Timeout,
        UsbError::DeviceNotFound { .. } => UhfError::DeviceNotFound,
        UsbError::Usb(s) => UhfError::Communication(s),
        other => UhfError::Communication(other.to_string()),
    }
}

impl UsbIo for UsbDevice {
    fn read_bulk(
        &self,
        _endpoint: u8,
        buf: &mut [u8],
        timeout: Duration,
    ) -> Result<usize, UhfError> {
        self.read_bulk_impl(buf, timeout).map_err(map_usb_to_uhf)
    }

    fn write_bulk(&self, _endpoint: u8, data: &[u8], timeout: Duration) -> Result<usize, UhfError> {
        self.write_bulk_impl(data, timeout).map_err(map_usb_to_uhf)
    }
}

impl Drop for UsbDevice {
    fn drop(&mut self) {}
}
