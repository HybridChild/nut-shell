//! Error types for CLI operations.
//!
//! The `CliError` enum represents all possible error conditions during
//! command processing, with security-conscious error messages.

use core::fmt;

/// CLI error type.
///
/// Represents all possible error conditions during command processing.
/// Error messages are designed to be user-friendly while maintaining security
/// (e.g., `InvalidPath` for both non-existent and inaccessible paths).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliError {
    /// Command not found in tree
    CommandNotFound,

    /// Path doesn't exist OR user lacks access (intentionally ambiguous for security)
    ///
    /// SECURITY: Never reveal whether path exists vs. access denied
    InvalidPath,

    /// Wrong number of arguments
    InvalidArgumentCount {
        /// Minimum expected arguments
        expected_min: usize,
        /// Maximum expected arguments
        expected_max: usize,
        /// Number of arguments received
        received: usize,
    },

    /// Invalid argument format/type (e.g., expected integer, got string)
    InvalidArgumentFormat {
        /// Which argument (0-indexed)
        arg_index: usize,
        /// What was expected (e.g., "integer", "IP address")
        expected: heapless::String<32>,
    },

    /// Buffer capacity exceeded
    BufferFull,

    /// Path exceeds MAX_PATH_DEPTH
    PathTooDeep,

    /// Authentication failed - wrong credentials
    #[cfg(feature = "authentication")]
    AuthenticationFailed,

    /// Tried to execute command while logged out
    #[cfg(feature = "authentication")]
    NotAuthenticated,

    /// I/O error occurred
    IoError,

    /// Async command called from sync context
    #[cfg(feature = "async")]
    AsyncInSyncContext,

    /// Operation timed out
    Timeout,

    /// Command executed but reported failure
    CommandFailed(heapless::String<128>),

    /// Generic error with message
    Other(heapless::String<128>),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::CommandNotFound => write!(f, "Command not found"),
            CliError::InvalidPath => write!(f, "Invalid path"),
            CliError::InvalidArgumentCount {
                expected_min,
                expected_max,
                received,
            } => {
                if expected_min == expected_max {
                    write!(f, "Expected {} arguments, got {}", expected_min, received)
                } else {
                    write!(
                        f,
                        "Expected {}-{} arguments, got {}",
                        expected_min, expected_max, received
                    )
                }
            }
            CliError::InvalidArgumentFormat {
                arg_index,
                expected,
            } => {
                write!(f, "Argument {}: expected {}", arg_index + 1, expected)
            }
            CliError::BufferFull => write!(f, "Buffer full"),
            CliError::PathTooDeep => write!(f, "Path too deep"),
            #[cfg(feature = "authentication")]
            CliError::AuthenticationFailed => write!(f, "Authentication failed"),
            #[cfg(feature = "authentication")]
            CliError::NotAuthenticated => write!(f, "Not authenticated"),
            CliError::IoError => write!(f, "I/O error"),
            #[cfg(feature = "async")]
            CliError::AsyncInSyncContext => write!(f, "Async command requires async context"),
            CliError::Timeout => write!(f, "Timeout"),
            CliError::CommandFailed(msg) => write!(f, "{}", msg),
            CliError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;
    use std::format;

    #[test]
    fn test_error_display() {
        assert_eq!(
            format!("{}", CliError::CommandNotFound),
            "Command not found"
        );
        assert_eq!(format!("{}", CliError::InvalidPath), "Invalid path");

        let err = CliError::InvalidArgumentCount {
            expected_min: 2,
            expected_max: 2,
            received: 1,
        };
        assert_eq!(format!("{}", err), "Expected 2 arguments, got 1");

        let err = CliError::InvalidArgumentCount {
            expected_min: 1,
            expected_max: 3,
            received: 4,
        };
        assert_eq!(format!("{}", err), "Expected 1-3 arguments, got 4");

        let mut expected = heapless::String::new();
        expected.push_str("integer").unwrap();
        let err = CliError::InvalidArgumentFormat {
            arg_index: 0,
            expected,
        };
        assert_eq!(format!("{}", err), "Argument 1: expected integer");

        let mut expected = heapless::String::new();
        expected.push_str("IP address").unwrap();
        let err = CliError::InvalidArgumentFormat {
            arg_index: 2,
            expected,
        };
        assert_eq!(format!("{}", err), "Argument 3: expected IP address");
    }

    #[test]
    fn test_error_equality() {
        assert_eq!(CliError::CommandNotFound, CliError::CommandNotFound);
        assert_ne!(CliError::CommandNotFound, CliError::InvalidPath);
    }
}
