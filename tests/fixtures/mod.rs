//! Test fixtures and utilities for nut-shell testing.
//!
//! Provides:
//! - `MockIo`: Test implementation of CharIo trait
//! - `MockAccessLevel`: Simple access level for testing
//! - `TEST_TREE`: Simple command tree for testing
//! - Helper functions for common test scenarios

#![allow(dead_code)]

use nut_shell::auth::AccessLevel;
use nut_shell::tree::{CommandKind, CommandMeta, Directory, Node};
use nut_shell::CharIo;
use std::collections::VecDeque;

// ============================================================================
// MockIo - Test I/O Implementation
// ============================================================================

/// Mock I/O for testing.
///
/// Provides in-memory character I/O with input queue and output capture.
/// Uses `std` types (VecDeque, Vec) since tests run with std support.
#[derive(Debug)]
pub struct MockIo {
    /// Input queue (simulates user typing)
    input: VecDeque<char>,

    /// Output capture (collects all output)
    output: Vec<char>,
}

impl MockIo {
    /// Create new MockIo with empty buffers.
    pub fn new() -> Self {
        Self {
            input: VecDeque::new(),
            output: Vec::new(),
        }
    }

    /// Create MockIo with pre-loaded input string.
    pub fn with_input(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            output: Vec::new(),
        }
    }

    /// Add input to queue (simulates user typing).
    pub fn push_input(&mut self, s: &str) {
        for c in s.chars() {
            self.input.push_back(c);
        }
    }

    /// Add single character to input queue.
    pub fn push_char(&mut self, c: char) {
        self.input.push_back(c);
    }

    /// Get captured output as string.
    pub fn output(&self) -> String {
        self.output.iter().collect()
    }

    /// Get captured output as bytes (useful for checking ANSI sequences).
    pub fn output_bytes(&self) -> Vec<u8> {
        self.output.iter().map(|&c| c as u8).collect()
    }

    /// Clear output buffer.
    pub fn clear_output(&mut self) {
        self.output.clear();
    }

    /// Check if input queue is empty.
    pub fn input_empty(&self) -> bool {
        self.input.is_empty()
    }

    /// Remaining input count.
    pub fn input_len(&self) -> usize {
        self.input.len()
    }
}

impl Default for MockIo {
    fn default() -> Self {
        Self::new()
    }
}

impl CharIo for MockIo {
    type Error = ();

    fn get_char(&mut self) -> Result<Option<char>, Self::Error> {
        Ok(self.input.pop_front())
    }

    fn put_char(&mut self, c: char) -> Result<(), Self::Error> {
        self.output.push(c);
        Ok(())
    }

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        for c in s.chars() {
            self.output.push(c);
        }
        Ok(())
    }
}

// ============================================================================
// MockAccessLevel - Simple Access Level for Testing
// ============================================================================

/// Simple access level for testing.
///
/// Three-level hierarchy: Guest < User < Admin
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum MockAccessLevel {
    /// Guest access (lowest)
    Guest = 0,

    /// User access (medium)
    User = 1,

    /// Admin access (highest)
    Admin = 2,
}

impl AccessLevel for MockAccessLevel {
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

// ============================================================================
// TEST_TREE - Simple Command Tree for Testing
// ============================================================================

/// Test command: help
pub const CMD_HELP: CommandMeta<MockAccessLevel> = CommandMeta {
    name: "help",
    description: "Show help",
    access_level: MockAccessLevel::Guest,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

/// Test command: echo
pub const CMD_ECHO: CommandMeta<MockAccessLevel> = CommandMeta {
    name: "echo",
    description: "Echo arguments",
    access_level: MockAccessLevel::Guest,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 16,
};

/// Test command: reboot (requires admin)
pub const CMD_REBOOT: CommandMeta<MockAccessLevel> = CommandMeta {
    name: "reboot",
    description: "Reboot system",
    access_level: MockAccessLevel::Admin,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

/// Test command: status (in system/ directory)
pub const CMD_STATUS: CommandMeta<MockAccessLevel> = CommandMeta {
    name: "status",
    description: "Show system status",
    access_level: MockAccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

/// Test directory: system/
pub const DIR_SYSTEM: Directory<MockAccessLevel> = Directory {
    name: "system",
    children: &[
        Node::Command(&CMD_STATUS),
        Node::Command(&CMD_REBOOT),
    ],
    access_level: MockAccessLevel::User,
};

/// Test directory: debug/ (admin only)
pub const DIR_DEBUG: Directory<MockAccessLevel> = Directory {
    name: "debug",
    children: &[
        // Empty for now, can add commands in specific tests
    ],
    access_level: MockAccessLevel::Admin,
};

/// Root directory for testing.
///
/// Structure:
/// ```text
/// /
/// ├── help (Guest)
/// ├── echo (Guest)
/// ├── system/ (User)
/// │   ├── status (User)
/// │   └── reboot (Admin)
/// └── debug/ (Admin)
/// ```
pub const TEST_TREE: Directory<MockAccessLevel> = Directory {
    name: "/",
    children: &[
        Node::Command(&CMD_HELP),
        Node::Command(&CMD_ECHO),
        Node::Directory(&DIR_SYSTEM),
        Node::Directory(&DIR_DEBUG),
    ],
    access_level: MockAccessLevel::Guest,
};

// ============================================================================
// Helper Functions
// ============================================================================

/// Create MockIo with input ending in newline.
pub fn io_with_command(cmd: &str) -> MockIo {
    let mut input = String::from(cmd);
    if !input.ends_with('\n') {
        input.push('\n');
    }
    MockIo::with_input(&input)
}

/// Create MockIo with multiple commands.
pub fn io_with_commands(cmds: &[&str]) -> MockIo {
    let mut input = String::new();
    for cmd in cmds {
        input.push_str(cmd);
        if !cmd.ends_with('\n') {
            input.push('\n');
        }
    }
    MockIo::with_input(&input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_io_basic() {
        let mut io = MockIo::new();

        io.push_input("hello");
        assert_eq!(io.get_char().unwrap(), Some('h'));
        assert_eq!(io.get_char().unwrap(), Some('e'));

        io.put_char('x').unwrap();
        io.write_str("yz").unwrap();
        assert_eq!(io.output(), "xyz");
    }

    #[test]
    fn test_mock_io_with_input() {
        let mut io = MockIo::with_input("test\n");
        assert_eq!(io.input_len(), 5);

        assert_eq!(io.get_char().unwrap(), Some('t'));
        assert_eq!(io.get_char().unwrap(), Some('e'));
        assert_eq!(io.get_char().unwrap(), Some('s'));
        assert_eq!(io.get_char().unwrap(), Some('t'));
        assert_eq!(io.get_char().unwrap(), Some('\n'));
        assert_eq!(io.get_char().unwrap(), None);
        assert!(io.input_empty());
    }

    #[test]
    fn test_mock_access_level() {
        assert!(MockAccessLevel::Admin > MockAccessLevel::User);
        assert!(MockAccessLevel::User > MockAccessLevel::Guest);

        assert_eq!(MockAccessLevel::from_str("Admin"), Some(MockAccessLevel::Admin));
        assert_eq!(MockAccessLevel::from_str("Invalid"), None);

        assert_eq!(MockAccessLevel::Admin.as_str(), "Admin");
    }

    #[test]
    fn test_tree_structure() {
        // Root has 4 children
        assert_eq!(TEST_TREE.children.len(), 4);

        // Can find commands
        assert!(TEST_TREE.find_child("help").is_some());
        assert!(TEST_TREE.find_child("echo").is_some());

        // Can find directories
        let system = TEST_TREE.find_child("system");
        assert!(system.is_some());

        if let Some(Node::Directory(dir)) = system {
            assert_eq!(dir.name, "system");
            assert_eq!(dir.children.len(), 2);
            assert!(dir.find_child("status").is_some());
            assert!(dir.find_child("reboot").is_some());
        } else {
            panic!("Expected directory node");
        }
    }

    #[test]
    fn test_helper_functions() {
        let io = io_with_command("test");
        assert_eq!(io.input_len(), 5); // "test\n"

        let io = io_with_commands(&["cmd1", "cmd2"]);
        assert_eq!(io.input_len(), 10); // "cmd1\ncmd2\n"
    }
}
