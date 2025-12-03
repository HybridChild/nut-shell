//! Tab completion for commands and paths.
//!
//! Provides smart completion with prefix matching and directory handling.
//! Uses stub function pattern - module always exists, functions return empty when disabled.

#![cfg_attr(not(feature = "completion"), allow(unused_variables))]

use crate::auth::AccessLevel;
use crate::error::CliError;
use crate::tree::Directory;

#[cfg(feature = "completion")]
use crate::tree::Node;

/// Tab completion result with type-safe variants for different match outcomes.
#[derive(Debug, Clone, PartialEq)]
pub enum CompletionResult<const MAX_MATCHES: usize> {
    /// No matches found for the input prefix
    None,

    /// Exactly one match found (auto-completable)
    Single {
        /// The completed name (with "/" appended for directories)
        // TODO: Use C::MAX_INPUT when const generics stabilize
        completion: heapless::String<128>,

        /// True if the match is a directory
        is_directory: bool,
    },

    /// Multiple matches found (show options to user)
    Multiple {
        /// Common prefix of all matches
        // TODO: Use C::MAX_INPUT when const generics stabilize
        common_prefix: heapless::String<128>,

        /// All matching node names (for display)
        // TODO: Consider using C::MAX_INPUT or a separate config constant when const generics stabilize
        all_matches: heapless::Vec<heapless::String<64>, MAX_MATCHES>,
    },
}

impl<const MAX_MATCHES: usize> CompletionResult<MAX_MATCHES> {
    /// Create empty completion result.
    pub fn empty() -> Self {
        Self::None
    }
}

// ============================================================================
// Feature-enabled implementation
// ============================================================================

/// Suggest completions for a partial input.
///
/// Performs prefix matching against accessible nodes in the current directory.
/// Directories get "/" appended for single matches.
#[cfg(feature = "completion")]
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
        return Ok(CompletionResult::None);
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

        return Ok(CompletionResult::Single {
            completion,
            is_directory: is_dir,
        });
    }

    // Multiple matches - find common prefix
    let common_prefix_str = find_common_prefix(&matches);

    let mut common_prefix = heapless::String::new();
    common_prefix
        .push_str(common_prefix_str)
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

    Ok(CompletionResult::Multiple {
        common_prefix,
        all_matches,
    })
}

/// Find longest common prefix among multiple matches.
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

        match result {
            CompletionResult::Single {
                completion,
                is_directory,
            } => {
                assert_eq!(completion.as_str(), "reboot");
                assert!(!is_directory);
            }
            _ => panic!("Expected Single variant"),
        }
    }

    #[test]
    #[cfg(feature = "completion")]
    fn test_single_match_directory() {
        let result = suggest_completions::<TestLevel, 16>(&TEST_DIR, "syst", None).unwrap();

        match result {
            CompletionResult::Single {
                completion,
                is_directory,
            } => {
                assert_eq!(completion.as_str(), "system/");
                assert!(is_directory);
            }
            _ => panic!("Expected Single variant"),
        }
    }

    #[test]
    #[cfg(feature = "completion")]
    fn test_multiple_matches_with_common_prefix() {
        let result = suggest_completions::<TestLevel, 16>(&TEST_DIR, "st", None).unwrap();

        match result {
            CompletionResult::Multiple {
                common_prefix,
                all_matches,
            } => {
                // Common prefix is "st" for "status", "start", "stop"
                assert_eq!(common_prefix.as_str(), "st");
                assert_eq!(all_matches.len(), 3);

                // Check all matches present (verify each is in the result)
                let match_names: [&str; 3] = ["status", "start", "stop"];
                for expected in &match_names {
                    assert!(
                        all_matches.iter().any(|m| m.as_str() == *expected),
                        "Expected to find '{}' in matches",
                        expected
                    );
                }
            }
            _ => panic!("Expected Multiple variant"),
        }
    }

    #[test]
    #[cfg(feature = "completion")]
    fn test_multiple_matches_directories() {
        let result = suggest_completions::<TestLevel, 16>(&TEST_DIR, "s", None).unwrap();

        match result {
            CompletionResult::Multiple {
                common_prefix,
                all_matches,
            } => {
                // Should match: status, start, stop, system, services
                assert_eq!(common_prefix.as_str(), "s");
                assert_eq!(all_matches.len(), 5);
            }
            _ => panic!("Expected Multiple variant"),
        }
    }

    #[test]
    #[cfg(feature = "completion")]
    fn test_no_matches() {
        let result = suggest_completions::<TestLevel, 16>(&TEST_DIR, "xyz", None).unwrap();

        match result {
            CompletionResult::None => {
                // Expected - no matches
            }
            _ => panic!("Expected None variant"),
        }
    }

    #[test]
    #[cfg(feature = "completion")]
    fn test_exact_match_command() {
        let result = suggest_completions::<TestLevel, 16>(&TEST_DIR, "status", None).unwrap();

        match result {
            CompletionResult::Single {
                completion,
                is_directory,
            } => {
                assert_eq!(completion.as_str(), "status");
                assert!(!is_directory);
            }
            _ => panic!("Expected Single variant"),
        }
    }

    #[test]
    #[cfg(feature = "completion")]
    fn test_exact_match_directory() {
        let result = suggest_completions::<TestLevel, 16>(&TEST_DIR, "system", None).unwrap();

        match result {
            CompletionResult::Single {
                completion,
                is_directory,
            } => {
                assert_eq!(completion.as_str(), "system/");
                assert!(is_directory);
            }
            _ => panic!("Expected Single variant"),
        }
    }

    #[test]
    #[cfg(feature = "completion")]
    fn test_empty_input_matches_all() {
        let result = suggest_completions::<TestLevel, 16>(&TEST_DIR, "", None).unwrap();

        match result {
            CompletionResult::Multiple { all_matches, .. } => {
                // Empty input matches everything
                assert_eq!(all_matches.len(), 6); // 4 commands + 2 directories
            }
            _ => panic!("Expected Multiple variant"),
        }
    }

    #[test]
    #[cfg(feature = "completion")]
    fn test_case_sensitive_matching() {
        let result = suggest_completions::<TestLevel, 16>(&TEST_DIR, "ST", None).unwrap();

        match result {
            CompletionResult::None => {
                // Expected - no matches (case-sensitive)
            }
            _ => panic!("Expected None variant"),
        }
    }

    #[test]
    #[cfg(not(feature = "completion"))]
    fn test_stub_returns_empty() {
        let result = suggest_completions::<TestLevel, 16>(&TEST_DIR, "st", None).unwrap();

        match result {
            CompletionResult::None => {
                // Expected - stub always returns empty
            }
            _ => panic!("Expected None variant"),
        }
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

        match result {
            CompletionResult::None => {
                // Expected - no access to reboot command
            }
            _ => panic!("Expected None variant"),
        }

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

        match result {
            CompletionResult::Single { completion, .. } => {
                assert_eq!(completion.as_str(), "reboot");
            }
            _ => panic!("Expected Single variant"),
        }
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

    #[test]
    #[cfg(feature = "completion")]
    fn test_max_matches_exceeded() {
        // Create directory with more nodes than MAX_MATCHES
        const CMD1: CommandMeta<TestLevel> = CommandMeta {
            id: "a1",
            name: "a1",
            description: "Command 1",
            access_level: TestLevel::Guest,
            kind: CommandKind::Sync,
            min_args: 0,
            max_args: 0,
        };
        const CMD2: CommandMeta<TestLevel> = CommandMeta {
            id: "a2",
            name: "a2",
            description: "Command 2",
            access_level: TestLevel::Guest,
            kind: CommandKind::Sync,
            min_args: 0,
            max_args: 0,
        };
        const CMD3: CommandMeta<TestLevel> = CommandMeta {
            id: "a3",
            name: "a3",
            description: "Command 3",
            access_level: TestLevel::Guest,
            kind: CommandKind::Sync,
            min_args: 0,
            max_args: 0,
        };
        const CMD4: CommandMeta<TestLevel> = CommandMeta {
            id: "a4",
            name: "a4",
            description: "Command 4",
            access_level: TestLevel::Guest,
            kind: CommandKind::Sync,
            min_args: 0,
            max_args: 0,
        };

        const OVERFLOW_DIR: Directory<TestLevel> = Directory {
            name: "overflow",
            children: &[
                Node::Command(&CMD1),
                Node::Command(&CMD2),
                Node::Command(&CMD3),
                Node::Command(&CMD4),
            ],
            access_level: TestLevel::Guest,
        };

        // Use MAX_MATCHES = 2, but we have 4 matching items
        let result = suggest_completions::<TestLevel, 2>(&OVERFLOW_DIR, "a", None);

        // Should return BufferFull error
        assert!(matches!(result, Err(CliError::BufferFull)));
    }

    #[test]
    #[cfg(feature = "completion")]
    fn test_very_long_command_name() {
        // Create a command with name > 128 characters
        const LONG_CMD: CommandMeta<TestLevel> = CommandMeta {
            id: "long",
            name: "this_is_a_very_long_command_name_that_exceeds_the_maximum_buffer_size_of_128_characters_and_should_cause_a_buffer_overflow_error_when_completing",
            description: "Long command",
            access_level: TestLevel::Guest,
            kind: CommandKind::Sync,
            min_args: 0,
            max_args: 0,
        };

        const LONG_DIR: Directory<TestLevel> = Directory {
            name: "long",
            children: &[Node::Command(&LONG_CMD)],
            access_level: TestLevel::Guest,
        };

        // Try to complete - should return BufferFull error
        let result = suggest_completions::<TestLevel, 16>(&LONG_DIR, "this", None);

        assert!(matches!(result, Err(CliError::BufferFull)));
    }

    #[test]
    #[cfg(feature = "completion")]
    fn test_long_directory_name_with_slash() {
        // Create a directory with name = 128 characters (so adding "/" would overflow)
        const LONG_DIR_CHILD: Directory<TestLevel> = Directory {
            name: "this_is_exactly_one_hundred_twenty_eight_characters_long_directory_name_abcdefghijklmnopqrstuvwxyz_0123456789_more_padding_needed",
            children: &[],
            access_level: TestLevel::Guest,
        };

        const LONG_DIR: Directory<TestLevel> = Directory {
            name: "parent",
            children: &[Node::Directory(&LONG_DIR_CHILD)],
            access_level: TestLevel::Guest,
        };

        // Try to complete - should return BufferFull error when trying to append "/"
        let result = suggest_completions::<TestLevel, 16>(&LONG_DIR, "this", None);

        assert!(matches!(result, Err(CliError::BufferFull)));
    }

    #[test]
    #[cfg(feature = "completion")]
    fn test_match_name_exceeds_64_chars() {
        // Create commands with names > 64 characters (for all_matches buffer)
        const LONG1: CommandMeta<TestLevel> = CommandMeta {
            id: "m1",
            name: "match_name_that_is_longer_than_sixty_four_characters_abcdefghijklm",
            description: "Long 1",
            access_level: TestLevel::Guest,
            kind: CommandKind::Sync,
            min_args: 0,
            max_args: 0,
        };
        const LONG2: CommandMeta<TestLevel> = CommandMeta {
            id: "m2",
            name: "match_name_that_is_longer_than_sixty_four_characters_nopqrstuvwxyz",
            description: "Long 2",
            access_level: TestLevel::Guest,
            kind: CommandKind::Sync,
            min_args: 0,
            max_args: 0,
        };

        const LONG_MATCH_DIR: Directory<TestLevel> = Directory {
            name: "longmatch",
            children: &[Node::Command(&LONG1), Node::Command(&LONG2)],
            access_level: TestLevel::Guest,
        };

        // Multiple matches with long names should cause BufferFull when building all_matches
        let result = suggest_completions::<TestLevel, 16>(&LONG_MATCH_DIR, "match", None);

        assert!(matches!(result, Err(CliError::BufferFull)));
    }
}
