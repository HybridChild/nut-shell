//! Test fixtures and utilities for nut-shell testing.
//!
//! Provides:
//! - `MockIo`: Test implementation of CharIo trait
//! - `MockAccessLevel`: Simple access level for testing
//! - `TEST_TREE`: Simple command tree for testing
//! - Helper functions for common test scenarios

#![allow(dead_code)]

use heapless::{Deque, String as HString, Vec as HVec};
use nut_shell::CharIo;
use nut_shell::config::DefaultConfig;
use nut_shell::error::CliError;
use nut_shell::response::Response;
use nut_shell::shell::handlers::CommandHandler;
use nut_shell::tree::{CommandKind, CommandMeta, Directory, Node};
use nut_shell_macros::AccessLevel;

// ============================================================================
// MockIo - Test I/O Implementation
// ============================================================================

/// Mock I/O for testing.
///
/// Provides in-memory character I/O with input queue and output capture.
/// Uses heapless types to maintain no_std compatibility in tests.
#[derive(Debug)]
pub struct MockIo {
    /// Input queue (simulates user typing) - max 256 chars
    input: Deque<char, 256>,

    /// Output capture (collects all output) - max 4096 chars
    output: HVec<char, 4096>,
}

impl MockIo {
    /// Create new MockIo with empty buffers.
    pub fn new() -> Self {
        Self {
            input: Deque::new(),
            output: HVec::new(),
        }
    }

    /// Create MockIo with pre-loaded input string.
    pub fn with_input(input: &str) -> Self {
        let mut io = Self::new();
        for c in input.chars() {
            let _ = io.input.push_back(c); // Ignore overflow - test data should fit
        }
        io
    }

    /// Add input to queue (simulates user typing).
    pub fn push_input(&mut self, s: &str) {
        for c in s.chars() {
            let _ = self.input.push_back(c); // Ignore overflow - test data should fit
        }
    }

    /// Add single character to input queue.
    pub fn push_char(&mut self, c: char) {
        let _ = self.input.push_back(c); // Ignore overflow - test data should fit
    }

    /// Get captured output as string (up to 1024 chars).
    pub fn output(&self) -> HString<1024> {
        let mut s = HString::new();
        for &c in self.output.iter() {
            let _ = s.push(c); // Truncate if too long
        }
        s
    }

    /// Get captured output as bytes (useful for checking ANSI sequences).
    pub fn output_bytes(&self) -> HVec<u8, 1024> {
        let mut v = HVec::new();
        for &c in self.output.iter() {
            let _ = v.push(c as u8); // Truncate if too long
        }
        v
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
        let _ = self.output.push(c); // Ignore overflow - test output should fit
        Ok(())
    }

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        for c in s.chars() {
            let _ = self.output.push(c); // Ignore overflow - test output should fit
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
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, AccessLevel)]
pub enum MockAccessLevel {
    /// Guest access (lowest)
    Guest = 0,

    /// User access (medium)
    User = 1,

    /// Admin access (highest)
    Admin = 2,
}

// ============================================================================
// TEST_TREE - Simple Command Tree for Testing
// ============================================================================

/// Test command: help
pub const CMD_HELP: CommandMeta<MockAccessLevel> = CommandMeta {
    id: "help",
    name: "help",
    description: "Show help",
    access_level: MockAccessLevel::Guest,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

/// Test command: echo
pub const CMD_ECHO: CommandMeta<MockAccessLevel> = CommandMeta {
    id: "echo",
    name: "echo",
    description: "Echo arguments",
    access_level: MockAccessLevel::Guest,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 16,
};

/// Test command: reboot (requires admin)
pub const CMD_REBOOT: CommandMeta<MockAccessLevel> = CommandMeta {
    id: "reboot",
    name: "reboot",
    description: "Reboot system",
    access_level: MockAccessLevel::Admin,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

// ============================================================================
// Test Commands for Response Formatting
// ============================================================================

pub const CMD_TEST_PREFIX_NEWLINE: CommandMeta<MockAccessLevel> = CommandMeta {
    id: "test-prefix-newline",
    name: "test-prefix-newline",
    description: "Test prefix newline formatting",
    access_level: MockAccessLevel::Guest,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

pub const CMD_TEST_INDENTED: CommandMeta<MockAccessLevel> = CommandMeta {
    id: "test-indented",
    name: "test-indented",
    description: "Test indented formatting",
    access_level: MockAccessLevel::Guest,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

pub const CMD_TEST_INLINE: CommandMeta<MockAccessLevel> = CommandMeta {
    id: "test-inline",
    name: "test-inline",
    description: "Test inline formatting",
    access_level: MockAccessLevel::Guest,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

pub const CMD_TEST_NO_POSTFIX: CommandMeta<MockAccessLevel> = CommandMeta {
    id: "test-no-postfix",
    name: "test-no-postfix",
    description: "Test without postfix newline",
    access_level: MockAccessLevel::Guest,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

pub const CMD_TEST_NO_PROMPT: CommandMeta<MockAccessLevel> = CommandMeta {
    id: "test-no-prompt",
    name: "test-no-prompt",
    description: "Test without prompt",
    access_level: MockAccessLevel::Guest,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

pub const CMD_TEST_COMBINED: CommandMeta<MockAccessLevel> = CommandMeta {
    id: "test-combined",
    name: "test-combined",
    description: "Test combined formatting flags",
    access_level: MockAccessLevel::Guest,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

/// Test command: status (in system/ directory)
pub const CMD_STATUS: CommandMeta<MockAccessLevel> = CommandMeta {
    id: "status",
    name: "status",
    description: "Show system status",
    access_level: MockAccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

/// Test command: async-wait (async command for testing)
#[cfg(feature = "async")]
pub const CMD_ASYNC_WAIT: CommandMeta<MockAccessLevel> = CommandMeta {
    id: "async-wait",
    name: "async-wait",
    description: "Async test command",
    access_level: MockAccessLevel::User,
    kind: CommandKind::Async,
    min_args: 0,
    max_args: 1,
};

/// Test directory: system/
#[cfg(not(feature = "async"))]
pub const DIR_SYSTEM: Directory<MockAccessLevel> = Directory {
    name: "system",
    children: &[
        Node::Command(&CMD_STATUS),
        Node::Command(&CMD_REBOOT),
        Node::Directory(&DIR_NETWORK),
        Node::Directory(&DIR_HARDWARE),
    ],
    access_level: MockAccessLevel::User,
};

/// Test directory: system/ (with async command)
#[cfg(feature = "async")]
pub const DIR_SYSTEM: Directory<MockAccessLevel> = Directory {
    name: "system",
    children: &[
        Node::Command(&CMD_STATUS),
        Node::Command(&CMD_REBOOT),
        Node::Command(&CMD_ASYNC_WAIT),
        Node::Directory(&DIR_NETWORK),
        Node::Directory(&DIR_HARDWARE),
    ],
    access_level: MockAccessLevel::User,
};

// ============================================================================
// Network Commands (system/network/)
// ============================================================================

/// Test command: network status
pub const CMD_NET_STATUS: CommandMeta<MockAccessLevel> = CommandMeta {
    id: "net_status",
    name: "status",
    description: "Show network status",
    access_level: MockAccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

/// Test command: network config
pub const CMD_NET_CONFIG: CommandMeta<MockAccessLevel> = CommandMeta {
    id: "net_config",
    name: "config",
    description: "Configure network settings",
    access_level: MockAccessLevel::Admin,
    kind: CommandKind::Sync,
    min_args: 2,
    max_args: 4,
};

/// Test command: network ping
pub const CMD_NET_PING: CommandMeta<MockAccessLevel> = CommandMeta {
    id: "net_ping",
    name: "ping",
    description: "Ping remote host",
    access_level: MockAccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 1,
    max_args: 2,
};

/// Network subdirectory
pub const DIR_NETWORK: Directory<MockAccessLevel> = Directory {
    name: "network",
    children: &[
        Node::Command(&CMD_NET_STATUS),
        Node::Command(&CMD_NET_CONFIG),
        Node::Command(&CMD_NET_PING),
    ],
    access_level: MockAccessLevel::User,
};

// ============================================================================
// Hardware Commands (system/hardware/)
// ============================================================================

/// Test command: LED control
pub const CMD_HW_LED: CommandMeta<MockAccessLevel> = CommandMeta {
    id: "hw_led",
    name: "led",
    description: "Control LED state",
    access_level: MockAccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 1,
    max_args: 1,
};

/// Test command: temperature sensor
pub const CMD_HW_TEMP: CommandMeta<MockAccessLevel> = CommandMeta {
    id: "hw_temp",
    name: "temperature",
    description: "Read temperature sensor",
    access_level: MockAccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

/// Hardware subdirectory
pub const DIR_HARDWARE: Directory<MockAccessLevel> = Directory {
    name: "hardware",
    children: &[Node::Command(&CMD_HW_LED), Node::Command(&CMD_HW_TEMP)],
    access_level: MockAccessLevel::User,
};

// ============================================================================
// Debug Commands
// ============================================================================

/// Test command: memory dump
pub const CMD_DEBUG_MEM: CommandMeta<MockAccessLevel> = CommandMeta {
    id: "debug_mem",
    name: "memory",
    description: "Dump memory contents",
    access_level: MockAccessLevel::Admin,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 2,
};

/// Test command: register read
pub const CMD_DEBUG_REG: CommandMeta<MockAccessLevel> = CommandMeta {
    id: "debug_reg",
    name: "registers",
    description: "Read hardware registers",
    access_level: MockAccessLevel::Admin,
    kind: CommandKind::Sync,
    min_args: 1,
    max_args: 1,
};

/// Test directory: debug/ (admin only)
pub const DIR_DEBUG: Directory<MockAccessLevel> = Directory {
    name: "debug",
    children: &[Node::Command(&CMD_DEBUG_MEM), Node::Command(&CMD_DEBUG_REG)],
    access_level: MockAccessLevel::Admin,
};

/// Root directory for testing.
///
/// Demonstrates const initialization with 3-level nesting and varied command patterns.
///
/// Structure:
/// ```text
/// /
/// ├── help (Guest, 0 args)
/// ├── echo (Guest, 0-16 args)
/// ├── system/ (User)
/// │   ├── status (User, 0 args)
/// │   ├── reboot (Admin, 0 args)
/// │   ├── async-wait (User, 0-1 args) [async feature only]
/// │   ├── network/ (User)
/// │   │   ├── status (User, 0 args)
/// │   │   ├── config (Admin, 2-4 args)
/// │   │   └── ping (User, 1-2 args)
/// │   └── hardware/ (User)
/// │       ├── led (User, 1 arg)
/// │       └── temperature (User, 0 args)
/// └── debug/ (Admin)
///     ├── memory (Admin, 0-2 args)
///     └── registers (Admin, 1 arg)
/// ```
///
/// **Validation Points**:
/// - 3 levels of nesting (root → system → network/hardware)
/// - Mixed access levels (Guest, User, Admin)
/// - Varied argument counts (0 to 16 max)
/// - Feature-gated commands (async-wait)
/// - Both empty and populated directories at each level
/// - All const-initializable (lives in ROM)
pub const TEST_TREE: Directory<MockAccessLevel> = Directory {
    name: "/",
    children: &[
        Node::Command(&CMD_HELP),
        Node::Command(&CMD_ECHO),
        Node::Directory(&DIR_SYSTEM),
        Node::Directory(&DIR_DEBUG),
        // Test commands for Response formatting
        Node::Command(&CMD_TEST_PREFIX_NEWLINE),
        Node::Command(&CMD_TEST_INDENTED),
        Node::Command(&CMD_TEST_INLINE),
        Node::Command(&CMD_TEST_NO_POSTFIX),
        Node::Command(&CMD_TEST_NO_PROMPT),
        Node::Command(&CMD_TEST_COMBINED),
    ],
    access_level: MockAccessLevel::Guest,
};

// ============================================================================
// MockHandlers - Command Execution Implementation
// ============================================================================

/// Mock command handlers for testing the metadata/execution separation pattern.
///
/// Implements the execution side of the pattern, mapping command IDs to functions.
/// This validates that CommandMeta (const metadata) and CommandHandler (runtime execution)
/// work together correctly.
pub struct MockHandlers;

/// Helper to join string slices with spaces (no_std compatible).
fn join_args(args: &[&str]) -> HString<256> {
    let mut result = HString::new();
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            let _ = result.push(' ');
        }
        let _ = result.push_str(arg);
    }
    result
}

/// Helper to format a simple message (no_std compatible).
fn format_msg(parts: &[&str]) -> HString<256> {
    let mut result = HString::new();
    for part in parts {
        let _ = result.push_str(part);
    }
    result
}

impl CommandHandler<DefaultConfig> for MockHandlers {
    fn execute_sync(&self, id: &str, args: &[&str]) -> Result<Response<DefaultConfig>, CliError> {
        match id {
            // Root commands
            "help" => Ok(Response::success("Help text here")),
            "echo" => {
                if args.is_empty() {
                    Ok(Response::success(""))
                } else {
                    let msg = join_args(args);
                    Ok(Response::success(&msg))
                }
            }

            // System commands
            "reboot" => Ok(Response::success("Rebooting...")),
            "status" => Ok(Response::success("System OK")),

            // Network commands (system/network/)
            "net_status" => Ok(Response::success("Network OK")),
            "net_config" => {
                let params = join_args(args);
                let msg = format_msg(&["Network configured: ", &params]);
                Ok(Response::success(&msg))
            }
            "net_ping" => {
                let host = args.first().unwrap_or(&"localhost");
                let count = args.get(1).unwrap_or(&"4");
                let msg = format_msg(&["Pinging ", host, " (", count, " times)"]);
                Ok(Response::success(&msg))
            }

            // Hardware commands (system/hardware/)
            "hw_led" => {
                let state = args.first().unwrap_or(&"off");
                let msg = format_msg(&["LED: ", state]);
                Ok(Response::success(&msg))
            }
            "hw_temp" => Ok(Response::success("Temperature: 23.5°C")),

            // Debug commands
            "debug_mem" => {
                if args.is_empty() {
                    Ok(Response::success("Memory dump (full)"))
                } else {
                    let addr = join_args(args);
                    let msg = format_msg(&["Memory at ", &addr]);
                    Ok(Response::success(&msg))
                }
            }
            "debug_reg" => {
                let reg = args.first().unwrap_or(&"0x00");
                let msg = format_msg(&["Register ", reg, ": 0x1234"]);
                Ok(Response::success(&msg))
            }

            // Test commands for Response formatting flags
            "test-prefix-newline" => {
                Ok(Response::success("Message with prefix").with_prefix_newline())
            }
            "test-indented" => Ok(Response::success("Line 1\r\nLine 2\r\nLine 3").indented()),
            "test-inline" => Ok(Response::success("... processing").inline()),
            "test-no-postfix" => {
                Ok(Response::success("No trailing newline").without_postfix_newline())
            }
            "test-no-prompt" => Ok(Response::success("No prompt after this").without_prompt()),
            "test-combined" => Ok(Response::success("Multi\r\nLine")
                .with_prefix_newline()
                .indented()
                .without_prompt()),

            _ => Err(CliError::CommandNotFound),
        }
    }

    #[cfg(feature = "async")]
    async fn execute_async(
        &self,
        id: &str,
        args: &[&str],
    ) -> Result<Response<DefaultConfig>, CliError> {
        match id {
            "async-wait" => {
                // Simulate async operation
                let duration = args
                    .first()
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(100);

                // Format "Waited Xms" without std
                let mut msg = heapless::String::<64>::new();
                let _ = msg.push_str("Waited ");

                // Convert u32 to string manually (simple approach for tests)
                let mut num_str = heapless::String::<16>::new();
                let mut n = duration;
                if n == 0 {
                    let _ = num_str.push('0');
                } else {
                    let mut digits = heapless::Vec::<char, 16>::new();
                    while n > 0 {
                        let _ = digits.push((b'0' + (n % 10) as u8) as char);
                        n /= 10;
                    }
                    for &d in digits.iter().rev() {
                        let _ = num_str.push(d);
                    }
                }

                let _ = msg.push_str(&num_str);
                let _ = msg.push_str("ms");
                Ok(Response::success(msg.as_str()))
            }
            _ => Err(CliError::CommandNotFound),
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Create MockIo with input ending in newline.
pub fn io_with_command(cmd: &str) -> MockIo {
    let mut input = HString::<256>::new();
    let _ = input.push_str(cmd);
    if !cmd.ends_with('\n') {
        let _ = input.push('\n');
    }
    MockIo::with_input(&input)
}

/// Create MockIo with multiple commands.
pub fn io_with_commands(cmds: &[&str]) -> MockIo {
    let mut input = HString::<512>::new();
    for cmd in cmds {
        let _ = input.push_str(cmd);
        if !cmd.ends_with('\n') {
            let _ = input.push('\n');
        }
    }
    MockIo::with_input(&input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nut_shell::auth::AccessLevel;

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

        assert_eq!(
            MockAccessLevel::from_str("Admin"),
            Some(MockAccessLevel::Admin)
        );
        assert_eq!(MockAccessLevel::from_str("Invalid"), None);

        assert_eq!(MockAccessLevel::Admin.as_str(), "Admin");
    }

    #[test]
    fn test_tree_structure() {
        // Root has 4 base + 6 test commands = 10 children
        assert_eq!(TEST_TREE.children.len(), 10);

        // Can find root commands
        assert!(TEST_TREE.find_child("help").is_some());
        assert!(TEST_TREE.find_child("echo").is_some());

        // Can find root directories
        assert!(TEST_TREE.find_child("system").is_some());
        assert!(TEST_TREE.find_child("debug").is_some());

        // Validate system/ directory (2 commands + 2 subdirs = 4, or +1 with async)
        let system = TEST_TREE.find_child("system");
        assert!(system.is_some());

        if let Some(Node::Directory(dir)) = system {
            assert_eq!(dir.name, "system");

            // System has 4 children without async (status, reboot, network, hardware)
            // and 5 with async (+ async-wait)
            #[cfg(not(feature = "async"))]
            assert_eq!(dir.children.len(), 4);

            #[cfg(feature = "async")]
            assert_eq!(dir.children.len(), 5);

            // Check commands
            assert!(dir.find_child("status").is_some());
            assert!(dir.find_child("reboot").is_some());

            #[cfg(feature = "async")]
            assert!(dir.find_child("async-wait").is_some());

            // Check subdirectories
            assert!(dir.find_child("network").is_some());
            assert!(dir.find_child("hardware").is_some());

            // Validate network/ subdirectory (3 commands)
            if let Some(Node::Directory(network)) = dir.find_child("network") {
                assert_eq!(network.name, "network");
                assert_eq!(network.children.len(), 3);
                assert!(network.find_child("status").is_some());
                assert!(network.find_child("config").is_some());
                assert!(network.find_child("ping").is_some());
            } else {
                panic!("Expected network directory");
            }

            // Validate hardware/ subdirectory (2 commands)
            if let Some(Node::Directory(hardware)) = dir.find_child("hardware") {
                assert_eq!(hardware.name, "hardware");
                assert_eq!(hardware.children.len(), 2);
                assert!(hardware.find_child("led").is_some());
                assert!(hardware.find_child("temperature").is_some());
            } else {
                panic!("Expected hardware directory");
            }
        } else {
            panic!("Expected system directory node");
        }

        // Validate debug/ directory (2 commands)
        if let Some(Node::Directory(debug)) = TEST_TREE.find_child("debug") {
            assert_eq!(debug.name, "debug");
            assert_eq!(debug.children.len(), 2);
            assert!(debug.find_child("memory").is_some());
            assert!(debug.find_child("registers").is_some());
        } else {
            panic!("Expected debug directory node");
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
