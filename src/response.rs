//! Response types for command execution.
//!
//! The `Response` type represents successful command execution with formatting flags
//! and message content. Command failures are represented via `CliError::CommandFailed`.
//! Generic over `ShellConfig` for buffer sizing.
//!
//! See [TYPE_REFERENCE.md](../docs/TYPE_REFERENCE.md) and [INTERNALS.md](../docs/INTERNALS.md)
//! Level 7 for complete response formatting details.

use crate::config::ShellConfig;
use core::marker::PhantomData;

/// Command execution response.
///
/// Generic over `C: ShellConfig` to use configured buffer size for messages.
/// Contains message and formatting flags. Represents successful command execution.
/// Command failures should return `Err(CliError::CommandFailed(msg))`.
#[derive(Debug, Clone, PartialEq)]
pub struct Response<C: ShellConfig> {
    /// Response message (uses C::MAX_RESPONSE buffer size)
    pub message: heapless::String<256>, // TODO: Use C::MAX_RESPONSE when const generics stabilize

    /// Message is inline (don't echo newline after command input)
    pub inline_message: bool,

    /// Add newline before message (in response formatter)
    pub prefix_newline: bool,

    /// Indent output (2 spaces)
    pub indent_message: bool,

    /// Add newline after message
    pub postfix_newline: bool,

    /// Display prompt after response
    pub show_prompt: bool,

    /// Prevent input from being saved to history
    #[cfg(feature = "history")]
    pub exclude_from_history: bool,

    /// Phantom data for config type (will be used when const generics stabilize)
    _phantom: PhantomData<C>,
}

impl<C: ShellConfig> Response<C> {
    /// Create success response with default formatting.
    ///
    /// Default: include in history, show prompt, add postfix newline.
    pub fn success(message: &str) -> Self {
        let mut msg = heapless::String::new();
        let _ = msg.push_str(message);

        Self {
            message: msg,
            inline_message: false,
            prefix_newline: false,
            indent_message: false,
            postfix_newline: true,
            show_prompt: true,
            #[cfg(feature = "history")]
            exclude_from_history: false,
            _phantom: PhantomData,
        }
    }

    /// Create success response that excludes input from history.
    ///
    /// Use for commands handling sensitive data (passwords, credentials).
    #[cfg(feature = "history")]
    pub fn success_no_history(message: &str) -> Self {
        let mut response = Self::success(message);
        response.exclude_from_history = true;
        response
    }

    /// Builder method to exclude response from history.
    ///
    /// Chainable method for excluding input from command history.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// Response::success("Logged in").without_history()
    /// ```
    #[cfg(feature = "history")]
    pub fn without_history(mut self) -> Self {
        self.exclude_from_history = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DefaultConfig;

    #[test]
    fn test_success_response() {
        let response = Response::<DefaultConfig>::success("OK");
        assert_eq!(response.message.as_str(), "OK");
        assert!(response.show_prompt);
        assert!(response.postfix_newline);
    }

    #[test]
    #[cfg(feature = "history")]
    fn test_without_history() {
        let response = Response::<DefaultConfig>::success("OK").without_history();
        assert!(response.exclude_from_history);

        let response = Response::<DefaultConfig>::success_no_history("OK");
        assert!(response.exclude_from_history);
    }
}
