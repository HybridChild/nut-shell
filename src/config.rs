//! Configuration traits and implementations for buffer sizing.
//!
//! The `ShellConfig` trait allows compile-time configuration of buffer sizes
//! and capacity limits without runtime overhead.

/// Shell configuration trait defining buffer sizes and capacity limits.
///
/// All values are const (zero runtime cost). Implementations define buffer sizes
/// for input, paths, arguments, prompts, responses, and history.
///
/// # ⚠️ Current Limitation: Values Not Yet Configurable
///
/// **IMPORTANT:** Due to Rust's current const generics limitations, changing these
/// configuration values will NOT affect the actual buffer sizes used internally.
/// All internal buffers are currently hardcoded to the values from `DefaultConfig`:
///
/// - `MAX_INPUT`: 128 (hardcoded)
/// - `MAX_PATH_DEPTH`: 8 (hardcoded)
/// - `MAX_ARGS`: 16 (hardcoded)
/// - `MAX_PROMPT`: 128 (hardcoded, derived from MAX_INPUT)
/// - `MAX_RESPONSE`: 256 (hardcoded)
/// - `HISTORY_SIZE`: 10 (hardcoded)
///
/// These constants are **only used for message configuration** at present.
/// The trait exists to establish the API contract for when const generics
/// stabilize, at which point all TODO comments throughout the codebase
/// will be addressed to make these values fully functional.
///
/// **You CAN customize:** The message strings (`MSG_*` constants) are fully
/// functional and can be customized in your `ShellConfig` implementation.
///
/// See [TYPE_REFERENCE.md](../docs/TYPE_REFERENCE.md) "Configuration" section.
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

    /// Welcome message when authentication is enabled (default: "Welcome! Please log in.\r\n")
    const MSG_WELCOME_AUTH: &'static str;

    /// Welcome message when authentication is disabled (default: "Welcome to nut-shell!\r\n")
    const MSG_WELCOME_NO_AUTH: &'static str;

    /// Login prompt (default: "Login (username:password): ")
    const MSG_LOGIN_PROMPT: &'static str;

    /// Login success message (default: "Login successful!\r\n")
    const MSG_LOGIN_SUCCESS: &'static str;

    /// Login failed message (default: "Login failed. Try again.\r\n")
    const MSG_LOGIN_FAILED: &'static str;

    /// Logout message (default: "Logged out.\r\n")
    const MSG_LOGOUT: &'static str;

    /// Invalid login format message (default: "Invalid format. Use username:password\r\n")
    const MSG_INVALID_LOGIN_FORMAT: &'static str;
}

/// Default configuration for typical embedded systems.
///
/// Balanced buffer sizes suitable for most applications:
/// - MAX_INPUT: 128 bytes
/// - MAX_PATH_DEPTH: 8 levels
/// - MAX_ARGS: 16 arguments
/// - MAX_PROMPT: 64 bytes
/// - MAX_RESPONSE: 256 bytes
/// - HISTORY_SIZE: 10 commands
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DefaultConfig;

impl ShellConfig for DefaultConfig {
    const MAX_INPUT: usize = 128;
    const MAX_PATH_DEPTH: usize = 8;
    const MAX_ARGS: usize = 16;
    const MAX_PROMPT: usize = 64;
    const MAX_RESPONSE: usize = 256;
    const HISTORY_SIZE: usize = 10;

    const MSG_WELCOME_AUTH: &'static str = "Welcome! Please log in.\r\n";
    const MSG_WELCOME_NO_AUTH: &'static str = "Welcome to nut-shell!\r\n";
    const MSG_LOGIN_PROMPT: &'static str = "Login (username:password): ";
    const MSG_LOGIN_SUCCESS: &'static str = "Login successful!\r\n";
    const MSG_LOGIN_FAILED: &'static str = "Login failed. Try again.\r\n";
    const MSG_LOGOUT: &'static str = "Logged out.\r\n";
    const MSG_INVALID_LOGIN_FORMAT: &'static str = "Invalid format. Use username:password\r\n";
}

/// Minimal configuration for resource-constrained systems.
///
/// Reduced buffer sizes for memory-limited devices:
/// - MAX_INPUT: 64 bytes
/// - MAX_PATH_DEPTH: 4 levels
/// - MAX_ARGS: 8 arguments
/// - MAX_PROMPT: 32 bytes
/// - MAX_RESPONSE: 128 bytes
/// - HISTORY_SIZE: 5 commands
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MinimalConfig;

impl ShellConfig for MinimalConfig {
    const MAX_INPUT: usize = 64;
    const MAX_PATH_DEPTH: usize = 4;
    const MAX_ARGS: usize = 8;
    const MAX_PROMPT: usize = 32;
    const MAX_RESPONSE: usize = 128;
    const HISTORY_SIZE: usize = 5;

    const MSG_WELCOME_AUTH: &'static str = "Welcome! Please log in.\r\n";
    const MSG_WELCOME_NO_AUTH: &'static str = "Welcome to nut-shell!\r\n";
    const MSG_LOGIN_PROMPT: &'static str = "Login (username:password): ";
    const MSG_LOGIN_SUCCESS: &'static str = "Login successful!\r\n";
    const MSG_LOGIN_FAILED: &'static str = "Login failed. Try again.\r\n";
    const MSG_LOGOUT: &'static str = "Logged out.\r\n";
    const MSG_INVALID_LOGIN_FORMAT: &'static str = "Invalid format. Use username:password\r\n";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        assert_eq!(DefaultConfig::MAX_INPUT, 128);
        assert_eq!(DefaultConfig::MAX_PATH_DEPTH, 8);
        assert_eq!(DefaultConfig::MAX_ARGS, 16);
        assert_eq!(DefaultConfig::MAX_PROMPT, 64);
        assert_eq!(DefaultConfig::MAX_RESPONSE, 256);
        assert_eq!(DefaultConfig::HISTORY_SIZE, 10);
    }

    #[test]
    fn test_minimal_config() {
        assert_eq!(MinimalConfig::MAX_INPUT, 64);
        assert_eq!(MinimalConfig::MAX_PATH_DEPTH, 4);
        assert_eq!(MinimalConfig::MAX_ARGS, 8);
        assert_eq!(MinimalConfig::MAX_PROMPT, 32);
        assert_eq!(MinimalConfig::MAX_RESPONSE, 128);
        assert_eq!(MinimalConfig::HISTORY_SIZE, 5);
    }

    #[test]
    fn test_default_config_messages() {
        assert_eq!(
            DefaultConfig::MSG_WELCOME_AUTH,
            "Welcome! Please log in.\r\n"
        );
        assert_eq!(
            DefaultConfig::MSG_WELCOME_NO_AUTH,
            "Welcome to nut-shell!\r\n"
        );
        assert_eq!(DefaultConfig::MSG_LOGIN_PROMPT, "Login (username:password): ");
        assert_eq!(DefaultConfig::MSG_LOGIN_SUCCESS, "Login successful!\r\n");
        assert_eq!(DefaultConfig::MSG_LOGIN_FAILED, "Login failed. Try again.\r\n");
        assert_eq!(DefaultConfig::MSG_LOGOUT, "Logged out.\r\n");
        assert_eq!(
            DefaultConfig::MSG_INVALID_LOGIN_FORMAT,
            "Invalid format. Use username:password\r\n"
        );
    }

    #[test]
    fn test_minimal_config_messages() {
        // MinimalConfig uses same messages as DefaultConfig
        assert_eq!(
            MinimalConfig::MSG_WELCOME_AUTH,
            "Welcome! Please log in.\r\n"
        );
        assert_eq!(
            MinimalConfig::MSG_WELCOME_NO_AUTH,
            "Welcome to nut-shell!\r\n"
        );
        assert_eq!(MinimalConfig::MSG_LOGIN_PROMPT, "Login (username:password): ");
        assert_eq!(MinimalConfig::MSG_LOGIN_SUCCESS, "Login successful!\r\n");
        assert_eq!(MinimalConfig::MSG_LOGIN_FAILED, "Login failed. Try again.\r\n");
        assert_eq!(MinimalConfig::MSG_LOGOUT, "Logged out.\r\n");
        assert_eq!(
            MinimalConfig::MSG_INVALID_LOGIN_FORMAT,
            "Invalid format. Use username:password\r\n"
        );
    }

    #[test]
    fn test_custom_config_messages() {
        // Test that users can customize messages
        struct CustomConfig;

        impl ShellConfig for CustomConfig {
            const MAX_INPUT: usize = 128;
            const MAX_PATH_DEPTH: usize = 8;
            const MAX_ARGS: usize = 16;
            const MAX_PROMPT: usize = 64;
            const MAX_RESPONSE: usize = 256;
            const HISTORY_SIZE: usize = 10;

            const MSG_WELCOME_AUTH: &'static str = "🔐 Custom System - Auth Required\r\n";
            const MSG_WELCOME_NO_AUTH: &'static str = "🚀 Custom System Ready\r\n";
            const MSG_LOGIN_PROMPT: &'static str = "Enter credentials (user:pass): ";
            const MSG_LOGIN_SUCCESS: &'static str = "✓ Access granted\r\n";
            const MSG_LOGIN_FAILED: &'static str = "✗ Access denied\r\n";
            const MSG_LOGOUT: &'static str = "Session ended\r\n";
            const MSG_INVALID_LOGIN_FORMAT: &'static str = "Format error\r\n";
        }

        // Verify custom messages
        assert_eq!(
            CustomConfig::MSG_WELCOME_AUTH,
            "🔐 Custom System - Auth Required\r\n"
        );
        assert_eq!(CustomConfig::MSG_WELCOME_NO_AUTH, "🚀 Custom System Ready\r\n");
        assert_eq!(CustomConfig::MSG_LOGIN_PROMPT, "Enter credentials (user:pass): ");
        assert_eq!(CustomConfig::MSG_LOGIN_SUCCESS, "✓ Access granted\r\n");
        assert_eq!(CustomConfig::MSG_LOGIN_FAILED, "✗ Access denied\r\n");
        assert_eq!(CustomConfig::MSG_LOGOUT, "Session ended\r\n");
        assert_eq!(CustomConfig::MSG_INVALID_LOGIN_FORMAT, "Format error\r\n");
    }

    #[test]
    fn test_messages_are_const() {
        // Verify that messages are compile-time constants (can be used in const context)
        const _WELCOME: &str = DefaultConfig::MSG_WELCOME_AUTH;
        const _LOGIN: &str = DefaultConfig::MSG_LOGIN_PROMPT;
        const _SUCCESS: &str = DefaultConfig::MSG_LOGIN_SUCCESS;
        const _FAILED: &str = DefaultConfig::MSG_LOGIN_FAILED;
        const _LOGOUT: &str = DefaultConfig::MSG_LOGOUT;
        const _FORMAT: &str = DefaultConfig::MSG_INVALID_LOGIN_FORMAT;
    }
}
