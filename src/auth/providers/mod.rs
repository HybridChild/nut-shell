//! Credential provider implementations for authentication.
//!
//! `ConstCredentialProvider` for testing (hardcoded credentials).

pub mod const_provider;

/// Testing/demo provider with hardcoded credentials (not for production).
pub use const_provider::ConstCredentialProvider;
