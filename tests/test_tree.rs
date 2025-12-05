//! Integration tests for tree data structures and metadata/execution separation pattern.
//!
//! Validates const-initialization, metadata/execution separation, sync/async commands,
//! and generic parameter integration.

#[allow(clippy::duplicate_mod)]
#[path = "fixtures/mod.rs"]
mod fixtures;
use fixtures::{MockAccessLevel, MockHandler, TEST_TREE};
use nut_shell::shell::handler::CommandHandler;
use nut_shell::tree::{CommandKind, Node};

#[cfg(feature = "async")]
use nut_shell::error::CliError;

// ============================================================================
// Metadata/Execution Separation Pattern Tests
// ============================================================================

#[test]
fn test_metadata_matches_handler() {
    // Validates that const metadata and handler implementations align
    let handler = MockHandler;

    // Find each command in TEST_TREE and verify handler exists
    if let Some(Node::Command(cmd)) = TEST_TREE.find_child("help") {
        assert_eq!(cmd.name, "help");
        assert_eq!(cmd.kind, CommandKind::Sync);
        // Verify handler exists
        assert!(handler.execute_sync("help", &[]).is_ok());
    } else {
        panic!("help command not found in TEST_TREE");
    }

    if let Some(Node::Command(cmd)) = TEST_TREE.find_child("echo") {
        assert_eq!(cmd.name, "echo");
        assert_eq!(cmd.kind, CommandKind::Sync);
        // Verify handler exists
        assert!(handler.execute_sync("echo", &[]).is_ok());
    } else {
        panic!("echo command not found in TEST_TREE");
    }
}

// ============================================================================
// Async Feature Tests
// ============================================================================

#[test]
#[cfg(feature = "async")]
fn test_async_command_metadata() {
    // Validates that async CommandMeta compiles when feature enabled
    use fixtures::CMD_ASYNC_WAIT;

    assert_eq!(CMD_ASYNC_WAIT.name, "async-wait");
    assert_eq!(CMD_ASYNC_WAIT.kind, CommandKind::Async);
    assert_eq!(CMD_ASYNC_WAIT.min_args, 0);
    assert_eq!(CMD_ASYNC_WAIT.max_args, 1);
}

#[test]
#[cfg(feature = "async")]
fn test_async_command_in_tree() {
    // Validates that async command is in tree when feature enabled
    use fixtures::DIR_SYSTEM;

    let async_cmd = DIR_SYSTEM.find_child("async-wait");
    assert!(async_cmd.is_some());

    if let Some(Node::Command(cmd)) = async_cmd {
        assert_eq!(cmd.name, "async-wait");
        assert_eq!(cmd.kind, CommandKind::Async);
    } else {
        panic!("async-wait should be a command node");
    }
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_async_command_execution() {
    // Validates that async commands execute correctly through handler
    let handler = MockHandler;

    // Test async-wait with no args
    let result = handler.execute_async("async-wait", &[]).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.message.as_str().contains("Waited 100ms"));

    // Test async-wait with custom duration
    let result = handler.execute_async("async-wait", &["250"]).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.message.as_str().contains("Waited 250ms"));

    // Test unknown async command
    let result = handler.execute_async("unknown-async", &[]).await;
    assert_eq!(result, Err(CliError::CommandNotFound));
}

// ============================================================================
// Node API Tests
// ============================================================================

#[test]
fn test_node_api() {
    // Test command node properties
    let help_node = TEST_TREE
        .find_child("help")
        .expect("help command should exist");
    assert!(help_node.is_command(), "help should be a command");
    assert!(!help_node.is_directory(), "help should not be a directory");
    assert_eq!(help_node.name(), "help", "help name should match");
    assert_eq!(
        help_node.access_level(),
        MockAccessLevel::Guest,
        "help should be Guest level"
    );

    // Test directory node properties
    let system_node = TEST_TREE
        .find_child("system")
        .expect("system directory should exist");
    assert!(system_node.is_directory(), "system should be a directory");
    assert!(!system_node.is_command(), "system should not be a command");
    assert_eq!(system_node.name(), "system", "system name should match");
    assert_eq!(
        system_node.access_level(),
        MockAccessLevel::User,
        "system should be User level"
    );

    // Test finding existing and non-existent children
    assert!(
        TEST_TREE.find_child("help").is_some(),
        "should find existing child"
    );
    assert!(
        TEST_TREE.find_child("nonexistent").is_none(),
        "should not find non-existent child"
    );

    // Test finding children in subdirectories
    if let Some(Node::Directory(system)) = TEST_TREE.find_child("system") {
        assert!(
            system.find_child("status").is_some(),
            "should find status in system"
        );
        assert!(
            system.find_child("reboot").is_some(),
            "should find reboot in system"
        );
        assert!(
            system.find_child("missing").is_none(),
            "should not find missing child"
        );
    } else {
        panic!("system should be a directory");
    }
}

// ============================================================================
// Const Initialization Validation Tests
// ============================================================================

#[test]
fn test_varied_argument_counts() {
    // Validates commands with different argument patterns
    use fixtures::{CMD_ECHO, CMD_HELP, CMD_HW_LED, CMD_NET_CONFIG, CMD_NET_PING};

    // No args
    assert_eq!(CMD_HELP.min_args, 0);
    assert_eq!(CMD_HELP.max_args, 0);

    // Variable args
    assert_eq!(CMD_ECHO.min_args, 0);
    assert_eq!(CMD_ECHO.max_args, 16);

    // Required args with range
    assert_eq!(CMD_NET_CONFIG.min_args, 2);
    assert_eq!(CMD_NET_CONFIG.max_args, 4);

    // Optional args
    assert_eq!(CMD_NET_PING.min_args, 1);
    assert_eq!(CMD_NET_PING.max_args, 2);

    // Exact arg count
    assert_eq!(CMD_HW_LED.min_args, 1);
    assert_eq!(CMD_HW_LED.max_args, 1);
}
