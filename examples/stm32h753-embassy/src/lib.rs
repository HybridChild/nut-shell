//! Shared library code for STM32H753ZI Embassy examples

#![no_std]

pub mod access_level;
pub mod credentials;
pub mod hw_commands;
pub mod system_commands;

pub use access_level::H753AccessLevel;

#[cfg(feature = "authentication")]
pub use credentials::{H753CredentialProvider, create_h753_provider};
