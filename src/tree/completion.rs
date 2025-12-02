//! Tab completion for commands and paths.
//!
//! Provides smart completion with prefix matching and directory handling.
//! Uses stub function pattern - module always exists, functions return empty when disabled.
//!
//! See [DESIGN.md](../../docs/DESIGN.md) "Feature Gating & Optional Features" for pattern details.

#![cfg_attr(not(feature = "completion"), allow(unused_variables))]

use crate::auth::AccessLevel;
use crate::error::CliError;
use crate::tree::Directory;

#[cfg(feature = "completion")]
use crate::tree::Node;

/// Completion result containing the completed text and match information.
#[derive(Debug, Clone, PartialEq)]
pub struct CompletionResult<const MAX_MATCHES: usize> {
    /// The completion text (common prefix if multiple matches)
    // TODO: Use C::MAX_INPUT when const generics stabilize
    pub completion: heapless::String<128>,

    /// True if exactly one match found
    pub is_complete: bool,

    /// True if the single match is a directory
    pub is_directory: bool,

    /// All matching node names (for display)
    // TODO: Consider using C::MAX_INPUT or a separate config constant when const generics stabilize
    pub all_matches: heapless::Vec<heapless::String<64>, MAX_MATCHES>,
}

impl<const MAX_MATCHES: usize> CompletionResult<MAX_MATCHES> {
    /// Create empty completion result.
    pub fn empty() -> Self {
        Self {
            completion: heapless::String::new(),
            is_complete: false,
            is_directory: false,
            all_matches: heapless::Vec::new(),
        }
    }
}

// ============================================================================
// Feature-enabled implementation
// ============================================================================

/// Suggest completions for a partial path input.
///
/// # Feature-enabled behavior
///
/// Performs prefix matching against nodes in the current directory:
/// 1. Finds all nodes whose names start with the input prefix
/// 2. If single match: returns complete name (+ "/" for directories)
/// 3. If multiple matches: returns common prefix and all match names
/// 4. If no matches: returns empty result
///
/// # Feature-disabled behavior
///
/// Returns empty CompletionResult (graceful degradation).
///
/// # Parameters
///
/// - `dir`: Current directory to search within
/// - `input`: Partial input to complete (e.g., "st" for "status")
/// - `current_user`: Current user (for access control filtering)
///
/// # Returns
///
/// - `Ok(CompletionResult)` - Completion results (may be empty)
/// - `Err(CliError)` - Error during completion processing
///
/// # Examples
///
/// ```rust,ignore
/// // Single match
/// let result = suggest_completions(&dir, "sta", Some(&user))?;
/// assert_eq!(result.completion.as_str(), "status");
/// assert!(result.is_complete);
///
/// // Multiple matches (common prefix)
/// let result = suggest_completions(&dir, "s", Some(&user))?;
/// assert_eq!(result.completion.as_str(), "s");  // Common prefix
/// assert!(!result.is_complete);
/// assert_eq!(result.all_matches.len(), 2);  // "status", "system"
/// ```
#[cfg(feature = "completion")]
#[allow(clippy::result_large_err)]
pub fn suggest_completions<L: AccessLevel, const MAX_MATCHES: usize>(
    dir: &Directory<L>,
    input: &str,
    current_user: Option<&crate::auth::User<L>>,
) -> Result<CompletionResult<MAX_MATCHES>, CliError> {
    // Find all matching nodes
    let mut matches: heapless::Vec<(&str, bool), MAX_MATCHES> = heapless::Vec::new();

    for child in dir.children.iter() {
        // Check access control
        let node_level = match child {
            Node::Command(cmd) => cmd.access_level,
            Node::Directory(d) => d.access_level,
        };

        // Filter by access level
        if let Some(user) = current_user
            && user.access_level < node_level
        {
            continue; // User lacks access, skip this node
        }

        let name = child.name();
        let is_dir = child.is_directory();

        // Check prefix match
        if name.starts_with(input) {
            matches
                .push((name, is_dir))
                .map_err(|_| CliError::BufferFull)?;
        }
    }

    // No matches
    if matches.is_empty() {
        return Ok(CompletionResult::empty());
    }

    // Single match - complete!
    if matches.len() == 1 {
        let (name, is_dir) = matches[0];
        let mut completion = heapless::String::new();
        completion
            .push_str(name)
            .map_err(|_| CliError::BufferFull)?;

        // Auto-append "/" for directories
        if is_dir {
            completion.push('/').map_err(|_| CliError::BufferFull)?;
        }

        let mut all_matches = heapless::Vec::new();
        let mut match_str = heapless::String::new();
        match_str.push_str(name).map_err(|_| CliError::BufferFull)?;
        all_matches
            .push(match_str)
            .map_err(|_| CliError::BufferFull)?;

        return Ok(CompletionResult {
            completion,
            is_complete: true,
            is_directory: is_dir,
            all_matches,
        });
    }

    // Multiple matches - find common prefix
    let common_prefix = find_common_prefix(&matches);

    let mut completion = heapless::String::new();
    completion
        .push_str(common_prefix)
        .map_err(|_| CliError::BufferFull)?;

    // Collect all match names for display
    // TODO: Consider using C::MAX_INPUT or a separate config constant when const generics stabilize
    let mut all_matches: heapless::Vec<heapless::String<64>, MAX_MATCHES> = heapless::Vec::new();
    for (name, _) in matches.iter() {
        let mut match_str = heapless::String::new();
        match_str.push_str(name).map_err(|_| CliError::BufferFull)?;
        all_matches
            .push(match_str)
            .map_err(|_| CliError::BufferFull)?;
    }

    Ok(CompletionResult {
        completion,
        is_complete: false,
        is_directory: false,
        all_matches,
    })
}

/// Find common prefix among multiple matches.
///
/// # Returns
///
/// Longest common prefix string. If no common prefix beyond what was already typed, returns empty.
#[cfg(feature = "completion")]
fn find_common_prefix<'a>(matches: &[(&'a str, bool)]) -> &'a str {
    if matches.is_empty() {
        return "";
    }

    let first = matches[0].0;

    // Find shortest match length
    let min_len = matches.iter().map(|(s, _)| s.len()).min().unwrap_or(0);

    // Find common prefix length
    let mut prefix_len = 0;
    for i in 0..min_len {
        let ch = first.as_bytes()[i];
        let all_match = matches.iter().all(|(s, _)| s.as_bytes()[i] == ch);
        if all_match {
            prefix_len = i + 1;
        } else {
            break;
        }
    }

    &first[..prefix_len]
}

// ============================================================================
// Feature-disabled stub implementation
// ============================================================================

/// Stub implementation when completion feature is disabled.
///
/// Returns empty CompletionResult (no completions available).
#[cfg(not(feature = "completion"))]
pub fn suggest_completions<L: AccessLevel, const MAX_MATCHES: usize>(
    _dir: &Directory<L>,
    _input: &str,
    _current_user: Option<&crate::auth::User<L>>,
) -> Result<CompletionResult<MAX_MATCHES>, CliError> {
    Ok(CompletionResult::empty())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::AccessLevel;
    use crate::tree::{CommandKind, CommandMeta, Directory, Node};

    // Test access level
    #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    enum TestLevel {
        Guest = 0,
        User = 1,
        Admin = 2,
    }

    impl AccessLevel for TestLevel {
        fn from_str(s: &str) -> Option<Self> {
            match s {
                "Guest" => Some(Self::Guest),
                "User" => Some(Self::User),
                "Admin" => Some(Self::Admin),
                _ => None,
            }
        }

        fn as_str(&self) -> &'static str {
            match self {
                Self::Guest => "Guest",
                Self::User => "User",
                Self::Admin => "Admin",
            }
        }
    }

    // Test fixtures
    const CMD_STATUS: CommandMeta<TestLevel> = CommandMeta {
        id: "status",
        name: "status",
        description: "Show status",
        access_level: TestLevel::User,
        kind: CommandKind::Sync,
        min_args: 0,
        max_args: 0,
    };

    const CMD_START: CommandMeta<TestLevel> = CommandMeta {
        id: "start",
        name: "start",
        description: "Start service",
        access_level: TestLevel::User,
        kind: CommandKind::Sync,
        min_args: 0,
        max_args: 1,
    };

    const CMD_STOP: CommandMeta<TestLevel> = CommandMeta {
        id: "stop",
        name: "stop",
        description: "Stop service",
        access_level: TestLevel::User,
        kind: CommandKind::Sync,
        min_args: 0,
        max_args: 0,
    };

    const CMD_REBOOT: CommandMeta<TestLevel> = CommandMeta {
        id: "reboot",
        name: "reboot",
        description: "Reboot system",
        access_level: TestLevel::Admin,
        kind: CommandKind::Sync,
        min_args: 0,
        max_args: 0,
    };

    const DIR_SYSTEM: Directory<TestLevel> = Directory {
        name: "system",
        children: &[],
        access_level: TestLevel::User,
    };

    const DIR_SERVICES: Directory<TestLevel> = Directory {
        name: "services",
        children: &[],
        access_level: TestLevel::User,
    };

    const TEST_DIR: Directory<TestLevel> = Directory {
        name: "test",
        children: &[
            Node::Command(&CMD_STATUS),
            Node::Command(&CMD_START),
            Node::Command(&CMD_STOP),
            Node::Command(&CMD_REBOOT),
            Node::Directory(&DIR_SYSTEM),
            Node::Directory(&DIR_SERVICES),
        ],
        access_level: TestLevel::Guest,
    };

    #[test]
    #[cfg(feature = "completion")]
    fn test_single_match_command() {
        let result = suggest_completions::<TestLevel, 16>(&TEST_DIR, "reb", None).unwrap();

        assert_eq!(result.completion.as_str(), "reboot");
        assert!(result.is_complete);
        assert!(!result.is_directory);
        assert_eq!(result.all_matches.len(), 1);
        assert_eq!(result.all_matches[0].as_str(), "reboot");
    }

    #[test]
    #[cfg(feature = "completion")]
    fn test_single_match_directory() {
        let result = suggest_completions::<TestLevel, 16>(&TEST_DIR, "syst", None).unwrap();

        assert_eq!(result.completion.as_str(), "system/");
        assert!(result.is_complete);
        assert!(result.is_directory);
        assert_eq!(result.all_matches.len(), 1);
    }

    #[test]
    #[cfg(feature = "completion")]
    fn test_multiple_matches_with_common_prefix() {
        let result = suggest_completions::<TestLevel, 16>(&TEST_DIR, "st", None).unwrap();

        // Common prefix is "st" for "status", "start", "stop"
        assert_eq!(result.completion.as_str(), "st");
        assert!(!result.is_complete);
        assert!(!result.is_directory);
        assert_eq!(result.all_matches.len(), 3);

        // Check all matches present (verify each is in the result)
        let match_names: [&str; 3] = ["status", "start", "stop"];
        for expected in &match_names {
            assert!(
                result.all_matches.iter().any(|m| m.as_str() == *expected),
                "Expected to find '{}' in matches",
                expected
            );
        }
    }

    #[test]
    #[cfg(feature = "completion")]
    fn test_multiple_matches_directories() {
        let result = suggest_completions::<TestLevel, 16>(&TEST_DIR, "s", None).unwrap();

        // Should match: status, start, stop, system, services
        assert_eq!(result.completion.as_str(), "s");
        assert!(!result.is_complete);
        assert_eq!(result.all_matches.len(), 5);
    }

    #[test]
    #[cfg(feature = "completion")]
    fn test_no_matches() {
        let result = suggest_completions::<TestLevel, 16>(&TEST_DIR, "xyz", None).unwrap();

        assert_eq!(result.completion.as_str(), "");
        assert!(!result.is_complete);
        assert!(!result.is_directory);
        assert_eq!(result.all_matches.len(), 0);
    }

    #[test]
    #[cfg(feature = "completion")]
    fn test_exact_match_command() {
        let result = suggest_completions::<TestLevel, 16>(&TEST_DIR, "status", None).unwrap();

        assert_eq!(result.completion.as_str(), "status");
        assert!(result.is_complete);
        assert!(!result.is_directory);
        assert_eq!(result.all_matches.len(), 1);
    }

    #[test]
    #[cfg(feature = "completion")]
    fn test_exact_match_directory() {
        let result = suggest_completions::<TestLevel, 16>(&TEST_DIR, "system", None).unwrap();

        assert_eq!(result.completion.as_str(), "system/");
        assert!(result.is_complete);
        assert!(result.is_directory);
        assert_eq!(result.all_matches.len(), 1);
    }

    #[test]
    #[cfg(feature = "completion")]
    fn test_empty_input_matches_all() {
        let result = suggest_completions::<TestLevel, 16>(&TEST_DIR, "", None).unwrap();

        // Empty input matches everything
        assert!(!result.is_complete);
        assert_eq!(result.all_matches.len(), 6); // 4 commands + 2 directories
    }

    #[test]
    #[cfg(feature = "completion")]
    fn test_case_sensitive_matching() {
        let result = suggest_completions::<TestLevel, 16>(&TEST_DIR, "ST", None).unwrap();

        // No matches (case-sensitive)
        assert_eq!(result.completion.as_str(), "");
        assert!(!result.is_complete);
        assert_eq!(result.all_matches.len(), 0);
    }

    #[test]
    #[cfg(not(feature = "completion"))]
    fn test_stub_returns_empty() {
        let result = suggest_completions::<TestLevel, 16>(&TEST_DIR, "st", None).unwrap();

        // Stub always returns empty
        assert_eq!(result.completion.as_str(), "");
        assert!(!result.is_complete);
        assert!(!result.is_directory);
        assert_eq!(result.all_matches.len(), 0);
    }

    #[test]
    #[cfg(feature = "completion")]
    fn test_access_control_filtering() {
        use crate::auth::User;

        // Create guest user (no access to Admin commands)
        let guest_user = User {
            username: {
                let mut s = heapless::String::new();
                s.push_str("guest").unwrap();
                s
            },
            access_level: TestLevel::Guest,
            #[cfg(feature = "authentication")]
            password_hash: [0u8; 32],
            #[cfg(feature = "authentication")]
            salt: [0u8; 16],
        };

        // "r" should NOT match "reboot" (Admin only) for guest user
        let result =
            suggest_completions::<TestLevel, 16>(&TEST_DIR, "r", Some(&guest_user)).unwrap();

        assert_eq!(result.completion.as_str(), "");
        assert!(!result.is_complete);
        assert_eq!(result.all_matches.len(), 0);

        // Create admin user
        let admin_user = User {
            username: {
                let mut s = heapless::String::new();
                s.push_str("admin").unwrap();
                s
            },
            access_level: TestLevel::Admin,
            #[cfg(feature = "authentication")]
            password_hash: [0u8; 32],
            #[cfg(feature = "authentication")]
            salt: [0u8; 16],
        };

        // "r" should match "reboot" for admin user
        let result =
            suggest_completions::<TestLevel, 16>(&TEST_DIR, "r", Some(&admin_user)).unwrap();

        assert_eq!(result.completion.as_str(), "reboot");
        assert!(result.is_complete);
        assert_eq!(result.all_matches.len(), 1);
    }

    #[test]
    #[cfg(feature = "completion")]
    fn test_common_prefix_calculation() {
        // Test internal helper
        let matches = [("start", false), ("status", false), ("stop", false)];
        let prefix = find_common_prefix(&matches);
        assert_eq!(prefix, "st");

        let matches = [("network", false), ("netscan", false)];
        let prefix = find_common_prefix(&matches);
        assert_eq!(prefix, "net");

        let matches = [("abc", false), ("xyz", false)];
        let prefix = find_common_prefix(&matches);
        assert_eq!(prefix, ""); // No common prefix
    }
}
