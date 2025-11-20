//! Configuration traits and implementations for buffer sizing.
//!
//! The `ShellConfig` trait allows compile-time configuration of buffer sizes
//! and capacity limits without runtime overhead.

/// Shell configuration trait defining buffer sizes and capacity limits.
///
/// All values are const (zero runtime cost). Implementations define buffer sizes
/// for input, paths, arguments, prompts, responses, and history.
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
}
