//! # nut-shell
//!
//! A lightweight command-line interface library for embedded systems.
//!
//! **nut-shell** provides a flexible, zero-allocation CLI framework designed for `no_std`
//! environments. It features:
//!
//! - **Static allocation**: Everything lives in ROM, zero heap usage
//! - **Const initialization**: Command trees defined at compile time
//! - **Optional features**: Authentication, tab completion, command history can be toggled
//! - **Flexible I/O**: Platform-agnostic character I/O trait
//! - **Access control**: Hierarchical permissions with generic access levels
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use nut_shell::{Shell, Directory, Node, CommandMeta, CommandHandler, Response, DefaultConfig};
//!
//! // 1. Define command tree (const, lives in ROM)
//! const ROOT: Directory<MyAccessLevel> = Directory {
//!     name: "/",
//!     children: &[/* commands */],
//!     access_level: MyAccessLevel::Guest,
//! };
//!
//! // 2. Implement command handlers
//! struct MyHandlers;
//! impl CommandHandler<DefaultConfig> for MyHandlers {
//!     fn execute_sync(&self, name: &str, args: &[&str]) -> Result<Response<DefaultConfig>, CliError> {
//!         // Command implementation
//!     }
//! }
//!
//! // 3. Create shell and process input
//! let mut shell = Shell::new(&ROOT, MyHandlers, io);
//! shell.activate()?;
//!
//! loop {
//!     if let Some(c) = io.get_char()? {
//!         shell.process_char(c)?;
//!     }
//! }
//! ```
//!
//! ## Features
//!
//! - `authentication` - User login/logout, password hashing, credential providers
//! - `completion` - Tab completion for commands and paths
//! - `history` - Command history with up/down arrow navigation
//! - `async` - Async command execution support
//!
//! The library also provides a `#[derive(AccessLevel)]` macro that's always available.
//!
//! ## no_std Support
//!
//! This library is `no_std` by default. Enable the `std` feature for testing only.

#![no_std]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![allow(clippy::result_large_err)]

extern crate heapless;

// Optional dependencies (feature-gated)
#[cfg(feature = "authentication")]
extern crate sha2;

#[cfg(feature = "authentication")]
extern crate subtle;

// Re-export derive macro (always available)
pub use nut_shell_macros::AccessLevel;

// ============================================================================
// Module Declarations
// ============================================================================

// Phase 2: I/O & Access Control Foundation
pub mod config;
pub mod io;

// Authentication module (always present, but with different contents based on features)
pub mod auth;

// Phase 2: Error handling
pub mod error;

// Phase 3: Tree data model
pub mod tree;

// Phase 5: Response types
pub mod response;

// Phase 6+: Shell orchestration
pub mod shell;

// ============================================================================
// Re-exports - Public API
// ============================================================================

// Core I/O
pub use io::CharIo;

// Configuration
pub use config::{DefaultConfig, MinimalConfig, ShellConfig};

// Error types
pub use error::CliError;

// Tree types (Phase 3)
pub use tree::{CommandKind, CommandMeta, Directory, Node};

// Access control (always available, even without authentication feature)
pub use auth::{AccessLevel, User};

// Response types (Phase 5)
pub use response::Response;

// Shell types (Phase 6+)
pub use shell::handlers::CommandHandler;
pub use shell::{CliState, HistoryDirection, Request, Shell};

// Optional feature re-exports (authentication-only types)
#[cfg(feature = "authentication")]
pub use auth::{ConstCredentialProvider, CredentialProvider, PasswordHasher, Sha256Hasher};

// ============================================================================
// Library Metadata
// ============================================================================

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library name
pub const NAME: &str = env!("CARGO_PKG_NAME");

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    // No tests needed - all public APIs tested in their respective modules
}
