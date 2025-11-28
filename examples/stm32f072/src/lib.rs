//! Shared library code for STM32 examples
//!
//! This module contains common implementations used across multiple
//! STM32 examples to reduce code duplication.

#![no_std]

pub mod access_level;
pub mod credentials;
pub mod hw_commands;
pub mod system_commands;

// Re-export commonly used types for convenience
pub use access_level::Stm32AccessLevel;

#[cfg(feature = "authentication")]
pub use credentials::Stm32CredentialProvider;

// Re-export command initialization functions
pub use system_commands::init_boot_time;
