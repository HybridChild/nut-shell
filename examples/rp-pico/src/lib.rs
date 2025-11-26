//! Shared library code for RP2040 examples
//!
//! This module contains common implementations used across multiple
//! RP2040 examples to reduce code duplication.

#![no_std]

pub mod access_level;
#[cfg(feature = "authentication")]
pub mod credentials;
pub mod hw_commands;
pub mod system_commands;

// Re-export commonly used types for convenience
pub use access_level::PicoAccessLevel;
#[cfg(feature = "authentication")]
pub use credentials::PicoCredentialProvider;

// Re-export command initialization functions
pub use hw_commands::{init_chip_id, init_reset_reason};
pub use system_commands::init_boot_time;
