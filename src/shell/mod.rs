//! Shell orchestration and command processing.
//!
//! The `Shell` struct brings together all components to provide interactive CLI functionality.
//! See [DESIGN.md](../../docs/DESIGN.md) for unified architecture pattern.

use crate::auth::AccessLevel;
use crate::config::ShellConfig;
use core::marker::PhantomData;

// Sub-modules
pub mod handlers;
pub mod parser;
pub mod history;

// Re-export key types
pub use handlers::CommandHandlers;

/// History navigation direction.
///
/// Used by `Request::History` variant. Self-documenting alternative to bool.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum HistoryDirection {
    /// Up arrow key (navigate to older command)
    Previous = 0,

    /// Down arrow key (navigate to newer command or restore original)
    Next = 1,
}

/// CLI state (authentication state).
///
/// Tracks whether the CLI is active and whether user is authenticated.
/// Used by unified architecture pattern to drive behavior.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CliState {
    /// CLI not active
    Inactive,

    /// Awaiting authentication (feature-gated, but always defined)
    #[cfg(feature = "authentication")]
    LoggedOut,

    /// Authenticated or auth-disabled mode
    LoggedIn,
}

/// Request type representing parsed user input.
///
/// Generic over `C: ShellConfig` to use configured buffer sizes.
/// Variants are feature-gated based on available features.
///
/// See [TYPE_REFERENCE.md](../../docs/TYPE_REFERENCE.md) for complete type definition.
#[derive(Debug, Clone)]
pub enum Request<C: ShellConfig> {
    /// Authentication attempt (feature-gated: authentication)
    #[cfg(feature = "authentication")]
    Login {
        /// Username
        username: heapless::String<32>,
        /// Password
        password: heapless::String<64>,
    },

    /// Failed login (feature-gated: authentication)
    #[cfg(feature = "authentication")]
    InvalidLogin,

    /// Execute command
    Command {
        /// Command path
        path: heapless::String<128>, // TODO: Use C::MAX_INPUT when const generics stabilize
        /// Command arguments
        args: heapless::Vec<heapless::String<128>, 16>, // TODO: Use C::MAX_INPUT and C::MAX_ARGS
        /// Original command string (for history, feature-gated)
        #[cfg(feature = "history")]
        original: heapless::String<128>, // TODO: Use C::MAX_INPUT
        /// Phantom data for config type (will be used when const generics stabilize)
        _phantom: PhantomData<C>,
    },

    /// Request completions (feature-gated: completion)
    #[cfg(feature = "completion")]
    TabComplete {
        /// Partial path to complete
        path: heapless::String<128>, // TODO: Use C::MAX_INPUT
    },

    /// Navigate history (feature-gated: history)
    #[cfg(feature = "history")]
    History {
        /// Navigation direction
        direction: HistoryDirection,
        /// Current buffer content
        buffer: heapless::String<128>, // TODO: Use C::MAX_INPUT
    },
}

/// Shell orchestration struct (placeholder for Phase 8).
///
/// Generic over:
/// - `'tree`: Lifetime of command tree (typically 'static)
/// - `L`: AccessLevel implementation
/// - `IO`: CharIo implementation
/// - `H`: CommandHandlers implementation
/// - `C`: ShellConfig implementation
///
/// See [DESIGN.md](../../docs/DESIGN.md) for unified architecture pattern.
#[derive(Debug)]
pub struct Shell<'tree, L, IO, H, C>
where
    L: AccessLevel,
    IO: crate::io::CharIo,
    H: CommandHandlers<C>,
    C: ShellConfig,
{
    _phantom: core::marker::PhantomData<(&'tree L, IO, H, C)>,
}

// Placeholder implementations will be added in Phase 8

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_direction() {
        assert_eq!(HistoryDirection::Previous as u8, 0);
        assert_eq!(HistoryDirection::Next as u8, 1);
    }

    #[test]
    fn test_cli_state() {
        assert_eq!(CliState::Inactive, CliState::Inactive);
        assert_eq!(CliState::LoggedIn, CliState::LoggedIn);

        #[cfg(feature = "authentication")]
        assert_ne!(CliState::LoggedOut, CliState::LoggedIn);
    }
}
