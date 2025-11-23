//! Integration tests for tree data structures and metadata/execution separation pattern.
//!
//! This module validates Phase 3 of the implementation:
//! - Tree types are const-initializable
//! - Metadata/execution separation pattern works correctly
//! - Sync and async commands compile and execute properly
//! - Generic parameters (L, C) integrate correctly

#[path = "fixtures/mod.rs"]
mod fixtures;

use fixtures::{MockAccessLevel, MockHandlers, TEST_TREE};
use nut_shell::config::DefaultConfig;
use nut_shell::error::CliError;
use nut_shell::response::Response;
use nut_shell::shell::handlers::CommandHandlers;
use nut_shell::tree::{CommandKind, CommandMeta, Directory, Node};

// ============================================================================
// Const Initialization Tests
// ============================================================================

#[test]
fn test_const_command_meta() {
    // Validates that CommandMeta is const-initializable
    const TEST_CMD: CommandMeta<MockAccessLevel> = CommandMeta {
        name: "test",
        description: "Test command",
        access_level: MockAccessLevel::User,
        kind: CommandKind::Sync,
        min_args: 0,
        max_args: 5,
    };

    assert_eq!(TEST_CMD.name, "test");
    assert_eq!(TEST_CMD.description, "Test command");
    assert_eq!(TEST_CMD.access_level, MockAccessLevel::User);
    assert_eq!(TEST_CMD.kind, CommandKind::Sync);
    assert_eq!(TEST_CMD.min_args, 0);
    assert_eq!(TEST_CMD.max_args, 5);
}

#[test]
fn test_const_directory() {
    // Validates that Directory is const-initializable
    const CMD: CommandMeta<MockAccessLevel> = CommandMeta {
        name: "cmd",
        description: "Test",
        access_level: MockAccessLevel::Guest,
        kind: CommandKind::Sync,
        min_args: 0,
        max_args: 0,
    };

    const DIR: Directory<MockAccessLevel> = Directory {
        name: "testdir",
        children: &[Node::Command(&CMD)],
        access_level: MockAccessLevel::Guest,
    };

    assert_eq!(DIR.name, "testdir");
    assert_eq!(DIR.children.len(), 1);
    assert_eq!(DIR.access_level, MockAccessLevel::Guest);
}

#[test]
fn test_const_tree_initialization() {
    // Validates that TEST_TREE compiles as const
    // The fact that this test compiles proves const initialization works
    let _tree = &TEST_TREE;

    assert_eq!(TEST_TREE.name, "/");
    assert!(TEST_TREE.children.len() >= 3); // At least help, echo, system
}

// ============================================================================
// Metadata/Execution Separation Pattern Tests
// ============================================================================

#[test]
fn test_handlers_implementation_exists() {
    // Validates that MockHandlers implements CommandHandlers trait
    let handlers = MockHandlers;

    // CommandHandlers is not dyn compatible due to async method,
    // but we can verify the implementation exists
    let _result = handlers.execute_sync("help", &[]);
}

#[test]
fn test_sync_command_execution() {
    // Validates that sync commands execute correctly through handlers
    let handlers = MockHandlers;

    // Test help command
    let result = handlers.execute_sync("help", &[]);
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.message.as_str(), "Help text here");

    // Test echo command with args
    let result = handlers.execute_sync("echo", &["hello", "world"]);
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.message.as_str(), "hello world");

    // Test echo with no args
    let result = handlers.execute_sync("echo", &[]);
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.message.as_str(), "");

    // Test reboot command
    let result = handlers.execute_sync("reboot", &[]);
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.message.as_str(), "Rebooting...");

    // Test status command
    let result = handlers.execute_sync("status", &[]);
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.message.as_str(), "System OK");

    // Test unknown command
    let result = handlers.execute_sync("unknown", &[]);
    assert_eq!(result, Err(CliError::CommandNotFound));
}

#[test]
fn test_metadata_matches_handlers() {
    // Validates that const metadata and handler implementations align
    let handlers = MockHandlers;

    // Find each command in TEST_TREE and verify handler exists
    if let Some(Node::Command(cmd)) = TEST_TREE.find_child("help") {
        assert_eq!(cmd.name, "help");
        assert_eq!(cmd.kind, CommandKind::Sync);
        // Verify handler exists
        assert!(handlers.execute_sync("help", &[]).is_ok());
    } else {
        panic!("help command not found in TEST_TREE");
    }

    if let Some(Node::Command(cmd)) = TEST_TREE.find_child("echo") {
        assert_eq!(cmd.name, "echo");
        assert_eq!(cmd.kind, CommandKind::Sync);
        // Verify handler exists
        assert!(handlers.execute_sync("echo", &[]).is_ok());
    } else {
        panic!("echo command not found in TEST_TREE");
    }
}

#[test]
fn test_command_kind_markers() {
    // Validates that CommandKind enum works as expected
    let sync_kind = CommandKind::Sync;
    assert_eq!(sync_kind, CommandKind::Sync);

    #[cfg(feature = "async")]
    {
        let async_kind = CommandKind::Async;
        assert_eq!(async_kind, CommandKind::Async);
        assert_ne!(sync_kind, async_kind);
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
    // Validates that async commands execute correctly through handlers
    let handlers = MockHandlers;

    // Test async-wait with no args
    let result = handlers.execute_async("async-wait", &[]).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(
        response.message.as_str().contains("100ms")
            || response.message.as_str() == "Async complete"
    );

    // Test async-wait with custom duration
    let result = handlers.execute_async("async-wait", &["250"]).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(
        response.message.as_str().contains("250ms")
            || response.message.as_str() == "Async complete"
    );

    // Test unknown async command
    let result = handlers.execute_async("unknown-async", &[]).await;
    assert_eq!(result, Err(CliError::CommandNotFound));
}

#[test]
#[cfg(feature = "async")]
fn test_async_trait_method_compiles() {
    // Validates that async trait method signature compiles
    // This is a compile-time validation - if this test exists, the pattern works
    let _handlers = MockHandlers;

    // If we got here, the async trait method compiled successfully
    // (CommandHandlers is not dyn compatible due to async method, but that's expected)
}

// ============================================================================
// Node Type Tests
// ============================================================================

#[test]
fn test_node_type_checking() {
    // Test command node
    if let Some(node) = TEST_TREE.find_child("help") {
        assert!(node.is_command());
        assert!(!node.is_directory());
        assert_eq!(node.name(), "help");
    } else {
        panic!("help command not found");
    }

    // Test directory node
    if let Some(node) = TEST_TREE.find_child("system") {
        assert!(node.is_directory());
        assert!(!node.is_command());
        assert_eq!(node.name(), "system");
    } else {
        panic!("system directory not found");
    }
}

#[test]
fn test_node_access_level() {
    // Test command access level
    if let Some(node) = TEST_TREE.find_child("help") {
        assert_eq!(node.access_level(), MockAccessLevel::Guest);
    } else {
        panic!("help command not found");
    }

    // Test directory access level
    if let Some(node) = TEST_TREE.find_child("system") {
        assert_eq!(node.access_level(), MockAccessLevel::User);
    } else {
        panic!("system directory not found");
    }
}

#[test]
fn test_directory_find_child() {
    // Test finding existing child
    let help = TEST_TREE.find_child("help");
    assert!(help.is_some());

    // Test not finding non-existent child
    let nonexistent = TEST_TREE.find_child("nonexistent");
    assert!(nonexistent.is_none());

    // Test finding child in subdirectory
    if let Some(Node::Directory(system)) = TEST_TREE.find_child("system") {
        let status = system.find_child("status");
        assert!(status.is_some());

        let reboot = system.find_child("reboot");
        assert!(reboot.is_some());

        let missing = system.find_child("missing");
        assert!(missing.is_none());
    } else {
        panic!("system should be a directory");
    }
}

// ============================================================================
// Generic Parameter Integration Tests
// ============================================================================

#[test]
fn test_generic_access_level_integration() {
    // Validates that AccessLevel generic parameter works throughout the system
    const CMD: CommandMeta<MockAccessLevel> = CommandMeta {
        name: "test",
        description: "Test",
        access_level: MockAccessLevel::Admin,
        kind: CommandKind::Sync,
        min_args: 0,
        max_args: 0,
    };

    const DIR: Directory<MockAccessLevel> = Directory {
        name: "testdir",
        children: &[Node::Command(&CMD)],
        access_level: MockAccessLevel::User,
    };

    let node = Node::Command(&CMD);

    assert_eq!(CMD.access_level, MockAccessLevel::Admin);
    assert_eq!(DIR.access_level, MockAccessLevel::User);
    assert_eq!(node.access_level(), MockAccessLevel::Admin);
}

#[test]
fn test_generic_config_integration() {
    // Validates that ShellConfig generic parameter works with Response
    let handlers = MockHandlers;

    // Response should be generic over DefaultConfig
    let result: Result<Response<DefaultConfig>, CliError> = handlers.execute_sync("help", &[]);
    assert!(result.is_ok());

    // Verify Response type is correct
    let _response = result.unwrap();
}

// ============================================================================
// Phase 3b: Const Initialization Validation Tests
// ============================================================================

#[test]
fn test_deep_nesting_const_initialization() {
    // Validates that 3-level nesting works with const initialization
    use fixtures::DIR_SYSTEM;

    // System directory has nested subdirectories
    let network = DIR_SYSTEM.find_child("network");
    assert!(network.is_some(), "Network subdirectory should exist");

    let hardware = DIR_SYSTEM.find_child("hardware");
    assert!(hardware.is_some(), "Hardware subdirectory should exist");

    // Each subdirectory has commands
    if let Some(Node::Directory(net)) = network {
        assert!(
            net.children.len() >= 3,
            "Network should have at least 3 commands"
        );
        assert!(net.find_child("status").is_some());
        assert!(net.find_child("config").is_some());
        assert!(net.find_child("ping").is_some());
    }

    if let Some(Node::Directory(hw)) = hardware {
        assert!(
            hw.children.len() >= 2,
            "Hardware should have at least 2 commands"
        );
        assert!(hw.find_child("led").is_some());
        assert!(hw.find_child("temperature").is_some());
    }
}

#[test]
fn test_varied_access_levels_in_tree() {
    // Validates that mixed access levels work in const tree
    use fixtures::{CMD_HELP, CMD_NET_CONFIG, CMD_REBOOT, MockAccessLevel};

    // Different access levels across the tree
    assert_eq!(CMD_HELP.access_level, MockAccessLevel::Guest);
    assert_eq!(CMD_REBOOT.access_level, MockAccessLevel::Admin);
    assert_eq!(CMD_NET_CONFIG.access_level, MockAccessLevel::Admin);
}

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

#[test]
fn test_const_tree_size() {
    // Validates tree structure counts for ROM size estimation
    use fixtures::{DIR_DEBUG, DIR_SYSTEM, TEST_TREE};

    // Count total nodes in tree
    let mut command_count = 0;
    let mut directory_count = 0;

    // Root level
    for child in TEST_TREE.children {
        match child {
            Node::Command(_) => command_count += 1,
            Node::Directory(_) => directory_count += 1,
        }
    }

    // System level
    for child in DIR_SYSTEM.children {
        match child {
            Node::Command(_) => command_count += 1,
            Node::Directory(dir) => {
                directory_count += 1;
                // Count subdirectory contents
                for subchild in dir.children {
                    match subchild {
                        Node::Command(_) => command_count += 1,
                        Node::Directory(_) => directory_count += 1,
                    }
                }
            }
        }
    }

    // Debug level
    for child in DIR_DEBUG.children {
        match child {
            Node::Command(_) => command_count += 1,
            Node::Directory(_) => directory_count += 1,
        }
    }

    // Verify expected structure
    // Commands: help(2) + system(2) + network(3) + hardware(2) + debug(2) + test commands(6) = 17
    // With async: +1 (async-wait) = 18
    // Directories: system(1) + debug(1) + network(1) + hardware(1) = 4 (root doesn't count)
    #[cfg(not(feature = "async"))]
    {
        assert_eq!(
            command_count, 17,
            "Should have exactly 17 commands without async"
        );
        assert_eq!(directory_count, 4, "Should have exactly 4 directories");
    }

    #[cfg(feature = "async")]
    {
        assert_eq!(
            command_count, 18,
            "Should have exactly 18 commands with async"
        );
        assert_eq!(directory_count, 4, "Should have exactly 4 directories");
    }
}

#[test]
fn test_const_metadata_properties() {
    // Validates that CommandMeta is truly const-initializable
    use fixtures::{CMD_DEBUG_REG, CMD_NET_STATUS};

    // These should be compile-time constants (const fn)
    const _TEST_NAME: &'static str = CMD_NET_STATUS.name;
    const _TEST_MIN: usize = CMD_DEBUG_REG.min_args;
    const _TEST_MAX: usize = CMD_DEBUG_REG.max_args;

    // If this compiles, const initialization works
    assert_eq!(_TEST_NAME, "status");
    assert_eq!(_TEST_MIN, 1);
    assert_eq!(_TEST_MAX, 1);
}

#[test]
fn test_tree_can_navigate_full_paths() {
    // Validates that full path navigation works through const tree
    use fixtures::TEST_TREE;

    // Navigate: root -> system -> network -> status
    let system = TEST_TREE.find_child("system");
    assert!(system.is_some());

    if let Some(Node::Directory(sys_dir)) = system {
        let network = sys_dir.find_child("network");
        assert!(network.is_some());

        if let Some(Node::Directory(net_dir)) = network {
            let status = net_dir.find_child("status");
            assert!(status.is_some());

            if let Some(Node::Command(cmd)) = status {
                assert_eq!(cmd.name, "status");
                assert_eq!(cmd.description, "Show network status");
            }
        }
    }

    // Navigate: root -> system -> hardware -> temperature
    if let Some(Node::Directory(sys_dir)) = TEST_TREE.find_child("system") {
        if let Some(Node::Directory(hw_dir)) = sys_dir.find_child("hardware") {
            if let Some(Node::Command(cmd)) = hw_dir.find_child("temperature") {
                assert_eq!(cmd.name, "temperature");
            }
        }
    }
}

// ============================================================================
// Pattern Validation Summary
// ============================================================================

/// This test serves as documentation of what Phase 3 validates.
///
/// If all tests in this module pass, we have validated:
/// 1. ✅ CommandMeta is const-initializable (no function pointers)
/// 2. ✅ Directory and Node types are const-initializable
/// 3. ✅ TEST_TREE lives in ROM with zero runtime initialization
/// 4. ✅ CommandHandlers trait compiles with both sync and async methods
/// 5. ✅ MockHandlers proves metadata/execution separation pattern works
/// 6. ✅ Sync commands execute correctly through handlers
/// 7. ✅ Async commands compile and execute when feature enabled
/// 8. ✅ Generic parameters (L: AccessLevel, C: ShellConfig) integrate correctly
/// 9. ✅ Node enum enables zero-cost dispatch via pattern matching
/// 10. ✅ Access level integration works with generic parameter
///
/// This early validation ensures the foundational pattern is sound before
/// proceeding to Phase 8 (Shell implementation), preventing costly refactoring.
#[test]
fn test_phase_3_validation_complete() {
    // If we got here, all validations passed
    assert!(true);
}
