//! Platform-specific code for USB access

use crate::api::error::UsbError;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
/// macOS-specific USB permission helpers
pub mod macos;

/// Check if the current user has enough permissions to access USB devices
///
/// # Errors
/// Returns a platform-specific USB error if permissions are insufficient (Linux only).
pub fn check_usb_permissions() -> Result<(), UsbError> {
    #[cfg(target_os = "linux")]
    return linux::check_usb_permissions();

    #[cfg(target_os = "windows")]
    return Ok(());

    #[cfg(target_os = "macos")]
    return Ok(());

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    return Ok(());
}

/// Print instructions for fixing USB permission issues
pub fn print_permission_instructions() {
    #[cfg(target_os = "linux")]
    linux::print_permission_instructions();

    #[cfg(target_os = "windows")]
    windows::print_permission_instructions();

    #[cfg(target_os = "macos")]
    macos::print_permission_instructions();

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    println!("For your platform, you may need to run this program with administrator privileges.");
}
