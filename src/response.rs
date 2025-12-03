//! Response types for command execution.
//!
//! `Response` represents successful execution with message and formatting flags.

use crate::config::ShellConfig;
use core::marker::PhantomData;

/// Command execution response with message and formatting flags.
/// Command failures return `Err(CliError::CommandFailed(msg))`, not `Response`.
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

    /// Builder method to exclude command from history (chainable).
    #[cfg(feature = "history")]
    pub fn without_history(mut self) -> Self {
        self.exclude_from_history = true;
        self
    }

    /// Builder method for inline response (appears on same line as command).
    pub fn inline(mut self) -> Self {
        self.inline_message = true;
        self
    }

    /// Builder method to add blank line before response.
    pub fn with_prefix_newline(mut self) -> Self {
        self.prefix_newline = true;
        self
    }

    /// Builder method to indent response (2 spaces per line).
    pub fn indented(mut self) -> Self {
        self.indent_message = true;
        self
    }

    /// Builder method to suppress newline after response.
    pub fn without_postfix_newline(mut self) -> Self {
        self.postfix_newline = false;
        self
    }

    /// Builder method to suppress prompt after response.
    pub fn without_prompt(mut self) -> Self {
        self.show_prompt = false;
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
        assert!(!response.inline_message);
        assert!(!response.prefix_newline);
        assert!(!response.indent_message);
        assert!(response.postfix_newline);
        assert!(response.show_prompt);

        #[cfg(feature = "history")]
        assert!(!response.exclude_from_history);
    }

    #[test]
    fn test_builder_chaining() {
        let response = Response::<DefaultConfig>::success("OK")
            .inline()
            .with_prefix_newline()
            .indented()
            .without_postfix_newline()
            .without_prompt();

        assert_eq!(response.message.as_str(), "OK");
        assert!(response.inline_message);
        assert!(response.prefix_newline);
        assert!(response.indent_message);
        assert!(!response.postfix_newline);
        assert!(!response.show_prompt);
    }

    #[test]
    #[cfg(feature = "history")]
    fn test_builder_chaining_with_history() {
        let response = Response::<DefaultConfig>::success("OK")
            .inline()
            .with_prefix_newline()
            .indented()
            .without_postfix_newline()
            .without_prompt()
            .without_history();

        assert_eq!(response.message.as_str(), "OK");
        assert!(response.inline_message);
        assert!(response.prefix_newline);
        assert!(response.indent_message);
        assert!(!response.postfix_newline);
        assert!(!response.show_prompt);
        assert!(response.exclude_from_history);
    }
}
