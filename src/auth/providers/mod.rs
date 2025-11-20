//! Credential provider implementations.
//!
//! Provides various credential storage backends for authentication.
//!
//! ## Available Providers
//!
//! - `ConstCredentialProvider` - Hardcoded credentials (testing/examples only)
//! - `buildtime` - Build-time environment variables (planned for production)

pub mod buildtime;
pub mod const_provider;

pub use const_provider::ConstCredentialProvider;
