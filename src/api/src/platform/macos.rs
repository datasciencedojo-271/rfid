/// Print macOS-specific instructions for granting USB permissions
pub fn print_permission_instructions() {
    println!(
        "macOS: If you encounter permission issues, ensure the app has Full Disk Access or run from Terminal with appropriate privileges."
    );
    println!("Try running: `sudo dscl . -append /Groups/_usbmuxd GroupMembership $(whoami)`");
    println!("Then restart, and retry");
}
