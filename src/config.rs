//! Configuration traits and implementations for buffer sizing.
//!
//! The `ShellConfig` trait allows compile-time configuration of buffer sizes
//! and capacity limits without runtime overhead.

/// Shell configuration trait defining buffer sizes and capacity limits.
///
/// All values are const (zero runtime cost). Due to Rust's const generics limitations
/// (const trait bounds not yet stable), internal buffers use hardcoded sizes from
/// `DefaultConfig` rather than `C::MAX_INPUT`, etc. The trait establishes the API
/// contract for when const generics stabilize.
///
/// **Currently customizable:** `MSG_*` strings only.
/// **Not yet customizable:** Buffer size constants (hardcoded to `DefaultConfig` values).
pub trait ShellConfig {
    /// Maximum input buffer size (default: 128)
    const MAX_INPUT: usize;

    /// Maximum path depth (default: 8)
    const MAX_PATH_DEPTH: usize;

    /// Maximum number of command arguments (default: 16)
    const MAX_ARGS: usize;

    /// Maximum prompt length (default: 64)
    const MAX_PROMPT: usize;

    /// Maximum response message length (default: 256)
    const MAX_RESPONSE: usize;

    /// Command history size (default: 10)
    const HISTORY_SIZE: usize;

    // Message constants for user-visible strings
    // All stored in ROM, zero runtime cost

    /// Welcome message shown on activation
    const MSG_WELCOME: &'static str;

    /// Login prompt
    const MSG_LOGIN_PROMPT: &'static str;

    /// Login success message
    const MSG_LOGIN_SUCCESS: &'static str;

    /// Login failed message
    const MSG_LOGIN_FAILED: &'static str;

    /// Logout message
    const MSG_LOGOUT: &'static str;

    /// Invalid login format message
    const MSG_INVALID_LOGIN_FORMAT: &'static str;
}

/// Default configuration for typical embedded systems.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DefaultConfig;

impl ShellConfig for DefaultConfig {
    const MAX_INPUT: usize = 128;
    const MAX_PATH_DEPTH: usize = 8;
    const MAX_ARGS: usize = 16;
    const MAX_PROMPT: usize = 64;
    const MAX_RESPONSE: usize = 256;

    #[cfg(feature = "history")]
    const HISTORY_SIZE: usize = 10;

    #[cfg(not(feature = "history"))]
    const HISTORY_SIZE: usize = 0;

    const MSG_WELCOME: &'static str = "Welcome to nut-shell! Type '?' for help.";
    const MSG_LOGIN_PROMPT: &'static str = "Login> ";
    const MSG_LOGIN_SUCCESS: &'static str = "Logged in.";
    const MSG_LOGIN_FAILED: &'static str = "Login failed. Try again.";
    const MSG_LOGOUT: &'static str = "Logged out.";
    const MSG_INVALID_LOGIN_FORMAT: &'static str = "Invalid format. Use <username>:<password>";
}

/// Minimal configuration for resource-constrained systems.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MinimalConfig;

impl ShellConfig for MinimalConfig {
    const MAX_INPUT: usize = 64;
    const MAX_PATH_DEPTH: usize = 4;
    const MAX_ARGS: usize = 8;
    const MAX_PROMPT: usize = 32;
    const MAX_RESPONSE: usize = 128;

    #[cfg(feature = "history")]
    const HISTORY_SIZE: usize = 4;

    #[cfg(not(feature = "history"))]
    const HISTORY_SIZE: usize = 0;

    const MSG_WELCOME: &'static str = "Welcome!";
    const MSG_LOGIN_PROMPT: &'static str = "Login> ";
    const MSG_LOGIN_SUCCESS: &'static str = "Logged in";
    const MSG_LOGIN_FAILED: &'static str = "Login failed";
    const MSG_LOGOUT: &'static str = "Logged out";
    const MSG_INVALID_LOGIN_FORMAT: &'static str = "Invalid format. Use name:password";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_messages_are_const() {
        // Verify that messages are compile-time constants (can be used in const context)
        const _WELCOME: &str = DefaultConfig::MSG_WELCOME;
        const _LOGIN: &str = DefaultConfig::MSG_LOGIN_PROMPT;
        const _SUCCESS: &str = DefaultConfig::MSG_LOGIN_SUCCESS;
        const _FAILED: &str = DefaultConfig::MSG_LOGIN_FAILED;
        const _LOGOUT: &str = DefaultConfig::MSG_LOGOUT;
        const _FORMAT: &str = DefaultConfig::MSG_INVALID_LOGIN_FORMAT;
    }
}
