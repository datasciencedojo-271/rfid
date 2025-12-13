//! Linux-specific USB access helpers

use crate::rfid_device::usb_device::UsbError;
use std::fs;
use std::path::Path;

/// Check if the current user has enough permissions to access USB devices
pub fn check_usb_permissions() -> Result<(), UsbError> {
    // Check if the user is root
    if unsafe { libc::geteuid() } == 0 {
        return Ok(());
    }

    // Check for membership in relevant groups
    let groups = ["plugdev", "dialout", "uucp"];
    for group in groups {
        if is_user_in_group(group) {
            return Ok(());
        }
    }

    // Check for udev rules
    if has_rfid_udev_rules() {
        return Ok(());
    }

    Err(UsbError::PermissionDenied)
}

/// Check if the user is in a specified group
fn is_user_in_group(group_name: &str) -> bool {
    use std::process::Command;

    // Try to use the "groups" command to check group membership
    let output = Command::new("groups").output().ok();

    if let Some(output) = output {
        if output.status.success() {
            let groups = String::from_utf8_lossy(&output.stdout);
            return groups.contains(group_name);
        }
    }

    false
}

/// Check if there are udev rules for RFID readers
fn has_rfid_udev_rules() -> bool {
    let udev_dirs = [
        "/etc/udev/rules.d",
        "/lib/udev/rules.d",
        "/usr/lib/udev/rules.d",
    ];

    for dir in &udev_dirs {
        if !Path::new(dir).exists() {
            continue;
        }

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.filter_map(Result::ok) {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    // Look for rules that might match our device
                    if content.contains("0e6a") && content.contains("0317") {
                        return true;
                    }

                    // More generic checks
                    if content.contains("SUBSYSTEM==\"usb\"")
                        && (content.contains("MODE=\"0666\"")
                            || content.contains("GROUP=\"plugdev\""))
                    {
                        return true;
                    }
                }
            }
        }
    }

    false
}

/// Print instructions for fixing USB permission issues on Linux
pub fn print_permission_instructions() {
    println!(
        "\n\nUSB PERMISSION ERROR: Cannot access the USB device due to insufficient permissions.\n"
    );
    println!("To fix this issue, you can:\n");
    println!("1. Create a udev rule (recommended permanent solution):");
    println!(
        "   sudo sh -c 'echo SUBSYSTEM==\"usb\", ATTR{{idVendor}}==\"0e6a\", ATTR{{idProduct}}==\"0317\", MODE=\"0666\" > /etc/udev/rules.d/50-rfid-reader.rules'"
    );
    println!("   sudo udevadm control --reload-rules");
    println!("   sudo udevadm trigger");
    println!("   (Then unplug and replug your device)\n");
    println!("2. Run this program with sudo (temporary solution):\n");
    println!(
        "   sudo {}",
        std::env::args()
            .next()
            .unwrap_or_else(|| "./rfid_program".to_string())
    );
    println!("\n3. Add your user to the 'plugdev' group (if it exists):\n");
    println!("   sudo usermod -aG plugdev $USER");
    println!("   (Log out and log back in for this to take effect)\n");
}
