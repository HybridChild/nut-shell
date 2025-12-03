//! Shared library code for native platform examples
//!
//! This module contains common implementations used across multiple
//! native examples to reduce code duplication.

pub mod access_level;
#[cfg(feature = "authentication")]
pub mod credentials;
pub mod io;

// Re-export commonly used types for convenience
pub use access_level::ExampleAccessLevel;
#[cfg(feature = "authentication")]
pub use credentials::{ExampleCredentialProvider, create_example_provider};
pub use io::{RawModeGuard, StdioCharIo};
