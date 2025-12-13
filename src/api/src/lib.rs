//! High-level API crate providing device access and platform helpers.
//! Re-exports API modules, device wrappers, and platform-specific utilities.

/// Core API modules (errors, helpers, high-level operations)
pub mod api;

/// USB device wrapper and helpers
pub mod rfid_device;

/// Platform-specific code (permissions/messages)
pub mod platform;
