//! API surface for RFID operations (errors, helpers, and high-level UHF API).
//! Modules:
//! - `error`: error types used across the API
//! - `lock_pattern_builder`: helpers for lock pattern construction
//! - `uhf_rfid_api`: high-level operations over the low-level protocol
/// Error types used across the API
pub mod error;
/// Helpers for lock pattern construction
pub mod lock_pattern_builder;
/// High-level UHF RFID operations
pub mod uhf_rfid_api;
