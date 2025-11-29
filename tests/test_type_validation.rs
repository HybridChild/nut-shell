//! Type-Level Integration Validation
//!
//! This checkpoint validates that all core types integrate correctly before
//! proceeding to Phase 7 (Tab Completion) and Phase 8 (Shell implementation).
//!
//! **Purpose**: Discover type integration issues NOW rather than during Shell implementation.
//!
//! **What we validate**:
//! - All foundational types instantiate without compilation errors
//! - Generic parameters (L, IO, H, C) work together correctly
//! - Path parsing and tree navigation work end-to-end
//! - Request/Response types integrate with tree and command handlers
//! - Both DefaultConfig and MinimalConfig work correctly
//! - Feature combinations compile cleanly
//! - Lifetime relationships between tree and runtime state are sound

#[path = "fixtures/mod.rs"]
mod fixtures;

use nut_shell::CharIo;
use nut_shell::auth::{AccessLevel, User};
use nut_shell::config::{DefaultConfig, MinimalConfig, ShellConfig};
use nut_shell::error::CliError;
use nut_shell::response::Response;
use nut_shell::shell::handlers::CommandHandler;
use nut_shell::shell::{CliState, HistoryDirection, Request};
use nut_shell::tree::path::Path;
use nut_shell::tree::{CommandKind, CommandMeta, Directory, Node};

use fixtures::{MockAccessLevel, MockHandlers, MockIo, TEST_TREE};

// Type alias for Path with DefaultConfig's MAX_PATH_DEPTH
type TestPath<'a> = Path<'a, { DefaultConfig::MAX_PATH_DEPTH }>;

// ============================================================================
// Test 1: All Core Types Instantiate Together
// ============================================================================

#[test]
fn test_all_types_instantiate_with_default_config() {
    // CharIo implementation
    let mut io = MockIo::new();
    assert!(io.get_char().unwrap().is_none());
    io.put_char('x').unwrap();
    assert_eq!(io.output(), "x");

    // AccessLevel implementation
    let guest = MockAccessLevel::Guest;
    let admin = MockAccessLevel::Admin;
    assert!(admin > guest);
    assert_eq!(guest.as_str(), "Guest");

    // User struct (always available)
    let user = User {
        username: {
            let mut s = heapless::String::<32>::new();
            s.push_str("testuser").unwrap();
            s
        },
        access_level: MockAccessLevel::User,
        #[cfg(feature = "authentication")]
        password_hash: [0u8; 32],
        #[cfg(feature = "authentication")]
        salt: [0u8; 16],
    };
    assert_eq!(user.username.as_str(), "testuser");
    assert_eq!(user.access_level, MockAccessLevel::User);

    // Tree types (const-initializable)
    let tree: &'static Directory<MockAccessLevel> = &TEST_TREE;
    assert_eq!(tree.name, "/");
    assert_eq!(tree.access_level, MockAccessLevel::Guest);

    // CommandMeta is const-initializable
    const TEST_CMD: CommandMeta<MockAccessLevel> = CommandMeta {
        id: "test",
        name: "test",
        description: "Test command",
        access_level: MockAccessLevel::User,
        kind: CommandKind::Sync,
        min_args: 0,
        max_args: 1,
    };
    assert_eq!(TEST_CMD.id, "test");
    assert_eq!(TEST_CMD.name, "test");
    assert_eq!(TEST_CMD.kind, CommandKind::Sync);

    // Path parsing
    let path = TestPath::parse("/system/network").unwrap();
    assert!(path.is_absolute());
    assert_eq!(path.segment_count(), 2);

    // Response type
    let response: Response<DefaultConfig> = Response::success("Test message");
    assert!(!response.message.is_empty());

    // Request type (Command variant, most common)
    let request: Request<DefaultConfig> = Request::Command {
        path: {
            let mut s = heapless::String::<128>::new();
            s.push_str("system/status").unwrap();
            s
        },
        args: heapless::Vec::new(),
        #[cfg(feature = "history")]
        original: {
            let mut s = heapless::String::<128>::new();
            s.push_str("system/status").unwrap();
            s
        },
        _phantom: core::marker::PhantomData,
    };

    // Extract path from request
    if let Request::Command { path, .. } = request {
        assert_eq!(path.as_str(), "system/status");
    }

    // CommandHandler trait
    let handlers = MockHandlers;
    let result = handlers.execute_sync("echo", &["hello", "world"]);
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.message.as_str(), "hello world");

    // CliState enum
    let state = CliState::LoggedIn;
    assert_eq!(state, CliState::LoggedIn);

    #[cfg(feature = "authentication")]
    {
        let logged_out = CliState::LoggedOut;
        assert_ne!(logged_out, CliState::LoggedIn);
    }

    // HistoryDirection enum
    let dir = HistoryDirection::Previous;
    assert_eq!(dir, HistoryDirection::Previous);
}

#[test]
fn test_all_types_instantiate_with_minimal_config() {
    // Validate MinimalConfig works with all types
    type Config = MinimalConfig;

    // Response with MinimalConfig
    let response: Response<Config> = Response::success("Minimal");
    assert!(!response.message.is_empty());

    // Request with MinimalConfig
    let request: Request<Config> = Request::Command {
        path: {
            let mut s = heapless::String::<128>::new();
            s.push_str("test").unwrap();
            s
        },
        args: heapless::Vec::new(),
        #[cfg(feature = "history")]
        original: heapless::String::new(),
        _phantom: core::marker::PhantomData,
    };

    if let Request::Command { path, .. } = request {
        assert_eq!(path.as_str(), "test");
    }

    // Verify MinimalConfig constants
    assert_eq!(Config::MAX_INPUT, 64);
    assert_eq!(Config::MAX_PATH_DEPTH, 4);
    assert_eq!(Config::MAX_ARGS, 8);
    assert_eq!(Config::MAX_PROMPT, 32);
    assert_eq!(Config::MAX_RESPONSE, 128);
    #[cfg(feature = "history")]
    assert_eq!(Config::HISTORY_SIZE, 4);
    #[cfg(not(feature = "history"))]
    assert_eq!(Config::HISTORY_SIZE, 0);
}

// ============================================================================
// Test 2: Path Parsing and Tree Navigation Integration
// ============================================================================

#[test]
fn test_path_parsing_and_tree_navigation() {
    // Parse absolute path
    let path = TestPath::parse("/system/network/status").unwrap();
    assert!(path.is_absolute());
    assert_eq!(path.segments(), &["system", "network", "status"]);

    // Navigate tree using path segments
    let tree = &TEST_TREE;

    // Find "system" directory
    let system_node = tree.find_child("system");
    assert!(system_node.is_some());

    if let Some(Node::Directory(system_dir)) = system_node {
        assert_eq!(system_dir.name, "system");

        // Find "network" subdirectory
        let network_node = system_dir.find_child("network");
        assert!(network_node.is_some());

        if let Some(Node::Directory(network_dir)) = network_node {
            assert_eq!(network_dir.name, "network");

            // Find "status" command
            let status_node = network_dir.find_child("status");
            assert!(status_node.is_some());

            if let Some(Node::Command(cmd)) = status_node {
                assert_eq!(cmd.name, "status");
                assert_eq!(cmd.description, "Show network status");
                assert_eq!(cmd.access_level, MockAccessLevel::User);
            } else {
                panic!("Expected command node");
            }
        } else {
            panic!("Expected network directory");
        }
    } else {
        panic!("Expected system directory");
    }
}

#[test]
fn test_relative_path_parsing() {
    // Parse relative paths
    let path = TestPath::parse("system/status").unwrap();
    assert!(!path.is_absolute());
    assert_eq!(path.segments(), &["system", "status"]);

    let path = TestPath::parse("../debug").unwrap();
    assert!(!path.is_absolute());
    assert_eq!(path.segments(), &["..", "debug"]);

    let path = TestPath::parse("./help").unwrap();
    assert!(!path.is_absolute());
    assert_eq!(path.segments(), &[".", "help"]);
}

#[test]
fn test_path_depth_validation() {
    // Valid path within limit (MAX_PATH_DEPTH = 8)
    let path = TestPath::parse("/a/b/c/d/e/f/g/h");
    assert!(path.is_ok());

    // Path exceeding limit
    let path = TestPath::parse("/a/b/c/d/e/f/g/h/i");
    assert!(matches!(path, Err(CliError::PathTooDeep)));
}

// ============================================================================
// Test 3: Request/Response Integration with Handlers
// ============================================================================

#[test]
fn test_request_response_with_handlers() {
    let handlers = MockHandlers;

    // Create command request
    let request: Request<DefaultConfig> = Request::Command {
        path: {
            let mut s = heapless::String::<128>::new();
            s.push_str("echo").unwrap();
            s
        },
        args: {
            let mut v = heapless::Vec::new();
            let mut arg1 = heapless::String::<128>::new();
            arg1.push_str("Hello").unwrap();
            v.push(arg1).unwrap();
            let mut arg2 = heapless::String::<128>::new();
            arg2.push_str("World").unwrap();
            v.push(arg2).unwrap();
            v
        },
        #[cfg(feature = "history")]
        original: {
            let mut s = heapless::String::<128>::new();
            s.push_str("echo Hello World").unwrap();
            s
        },
        _phantom: core::marker::PhantomData,
    };

    // Execute command
    if let Request::Command { args, .. } = request {
        // Convert args to &[&str]
        let arg_refs: heapless::Vec<&str, 16> = args.iter().map(|s| s.as_str()).collect();
        let arg_slice: &[&str] = &arg_refs;

        let result = handlers.execute_sync("echo", arg_slice);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.message.as_str(), "Hello World");
    }
}

#[test]
fn test_command_not_found_error() {
    let handlers = MockHandlers;

    let result = handlers.execute_sync("nonexistent", &[]);
    assert!(matches!(result, Err(CliError::CommandNotFound)));
}

#[test]
fn test_handlers_with_different_commands() {
    let handlers = MockHandlers;

    // Test reboot command (ID: "reboot")
    let result = handlers.execute_sync("reboot", &[]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().message.as_str(), "Rebooting...");

    // Test status command (ID: "status")
    let result = handlers.execute_sync("status", &[]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().message.as_str(), "System OK");

    // Test led command with argument (ID: "hw_led")
    let result = handlers.execute_sync("hw_led", &["on"]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().message.as_str(), "LED: on");
}

// ============================================================================
// Test 4: Generic Parameter Inference
// ============================================================================

#[test]
fn test_generic_parameter_inference() {
    // Verify that generic parameters infer naturally in typical usage

    // Type inference for Response
    let response = Response::<DefaultConfig>::success("Test");
    assert!(!response.message.is_empty());

    // Type inference for handlers
    let handlers: MockHandlers = MockHandlers;
    let _result: Result<Response<DefaultConfig>, CliError> = handlers.execute_sync("help", &[]);

    // Type inference for tree navigation
    let tree: &Directory<MockAccessLevel> = &TEST_TREE;
    let _node: Option<&Node<MockAccessLevel>> = tree.find_child("help");
}

// ============================================================================
// Test 5: Lifetime Relationships
// ============================================================================

#[test]
fn test_lifetime_relationships() {
    // Verify that static tree lifetime works correctly

    // Static tree reference
    let tree: &'static Directory<MockAccessLevel> = &TEST_TREE;

    // Function that requires 'static lifetime
    fn requires_static_tree(tree: &'static Directory<MockAccessLevel>) -> &'static str {
        tree.name
    }

    let name = requires_static_tree(tree);
    assert_eq!(name, "/");

    // Verify command metadata is also 'static
    fn _get_command_name(node: &'static Node<MockAccessLevel>) -> Option<&'static str> {
        match node {
            Node::Command(cmd) => Some(cmd.name),
            Node::Directory(_) => None,
        }
    }

    if let Some(Node::Command(cmd)) = tree.find_child("help") {
        // This should compile because TEST_TREE is static
        let cmd_static: &'static CommandMeta<MockAccessLevel> = cmd;
        assert_eq!(cmd_static.name, "help");
    }
}

// ============================================================================
// Test 6: Const Initialization Validation
// ============================================================================

#[test]
fn test_const_initialization() {
    // Verify that CommandMeta and Directory can be const-initialized

    const CMD: CommandMeta<MockAccessLevel> = CommandMeta {
        id: "const_test",
        name: "const_test",
        description: "Test const init",
        access_level: MockAccessLevel::User,
        kind: CommandKind::Sync,
        min_args: 0,
        max_args: 0,
    };

    assert_eq!(CMD.id, "const_test");
    assert_eq!(CMD.name, "const_test");
    assert_eq!(CMD.kind, CommandKind::Sync);

    // Verify tree is const-initializable (already proven by TEST_TREE)
    const _TREE: &Directory<MockAccessLevel> = &TEST_TREE;

    // This test compiling proves const initialization works
}

// ============================================================================
// Test 7: Feature-Gated Request Variants
// ============================================================================

#[cfg(feature = "authentication")]
#[test]
fn test_login_request() {
    let request: Request<DefaultConfig> = Request::Login {
        username: {
            let mut s = heapless::String::<32>::new();
            s.push_str("admin").unwrap();
            s
        },
        password: {
            let mut s = heapless::String::<64>::new();
            s.push_str("password123").unwrap();
            s
        },
    };

    if let Request::Login { username, password } = request {
        assert_eq!(username.as_str(), "admin");
        assert_eq!(password.as_str(), "password123");
    } else {
        panic!("Expected Login request");
    }

    // InvalidLogin variant
    let invalid: Request<DefaultConfig> = Request::InvalidLogin;
    assert!(matches!(invalid, Request::InvalidLogin));
}

#[cfg(feature = "completion")]
#[test]
fn test_tab_complete_request() {
    let request: Request<DefaultConfig> = Request::TabComplete {
        path: {
            let mut s = heapless::String::<128>::new();
            s.push_str("sys").unwrap();
            s
        },
    };

    if let Request::TabComplete { path } = request {
        assert_eq!(path.as_str(), "sys");
    } else {
        panic!("Expected TabComplete request");
    }
}

#[cfg(feature = "history")]
#[test]
fn test_history_request() {
    let request: Request<DefaultConfig> = Request::History {
        direction: HistoryDirection::Previous,
        buffer: {
            let mut s = heapless::String::<128>::new();
            s.push_str("current input").unwrap();
            s
        },
    };

    if let Request::History { direction, buffer } = request {
        assert_eq!(direction, HistoryDirection::Previous);
        assert_eq!(buffer.as_str(), "current input");
    } else {
        panic!("Expected History request");
    }
}

// ============================================================================
// Test 8: Access Level Integration with Tree
// ============================================================================

#[test]
fn test_access_levels_in_tree() {
    let tree = &TEST_TREE;

    // Guest-level command at root
    if let Some(Node::Command(help)) = tree.find_child("help") {
        assert_eq!(help.access_level, MockAccessLevel::Guest);
    } else {
        panic!("Expected help command");
    }

    // User-level directory
    if let Some(Node::Directory(system)) = tree.find_child("system") {
        assert_eq!(system.access_level, MockAccessLevel::User);

        // Admin-level command in subdirectory
        if let Some(Node::Command(reboot)) = system.find_child("reboot") {
            assert_eq!(reboot.access_level, MockAccessLevel::Admin);
        } else {
            panic!("Expected reboot command");
        }
    } else {
        panic!("Expected system directory");
    }

    // Admin-level directory
    if let Some(Node::Directory(debug)) = tree.find_child("debug") {
        assert_eq!(debug.access_level, MockAccessLevel::Admin);
    } else {
        panic!("Expected debug directory");
    }
}

// ============================================================================
// Test 9: Async Command Validation
// ============================================================================

#[cfg(feature = "async")]
#[test]
fn test_async_command_metadata() {
    use nut_shell::tree::CommandKind;

    let tree = &TEST_TREE;

    // Find async command
    if let Some(Node::Directory(system)) = tree.find_child("system") {
        if let Some(Node::Command(async_cmd)) = system.find_child("async-wait") {
            assert_eq!(async_cmd.name, "async-wait");
            assert_eq!(async_cmd.kind, CommandKind::Async);
        } else {
            panic!("Expected async-wait command when async feature enabled");
        }
    }
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_async_handler_execution() {
    let handlers = MockHandlers;

    // Execute async command
    let result = handlers.execute_async("async-wait", &[]).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert!(response.message.contains("Waited") || response.message.contains("Async"));
}

// ============================================================================
// Test 10: CharIo Integration
// ============================================================================

#[test]
fn test_char_io_with_mock() {
    let mut io = MockIo::with_input("test\n");

    // Read characters
    assert_eq!(io.get_char().unwrap(), Some('t'));
    assert_eq!(io.get_char().unwrap(), Some('e'));
    assert_eq!(io.get_char().unwrap(), Some('s'));
    assert_eq!(io.get_char().unwrap(), Some('t'));
    assert_eq!(io.get_char().unwrap(), Some('\n'));
    assert_eq!(io.get_char().unwrap(), None);

    // Write characters
    io.put_char('o').unwrap();
    io.put_char('k').unwrap();
    assert_eq!(io.output(), "ok");

    // Write string
    io.write_str(" done").unwrap();
    assert_eq!(io.output(), "ok done");
}

// ============================================================================
// Summary Test: Everything Together
// ============================================================================

#[test]
fn test_complete_integration() {
    // This test brings ALL types together to validate end-to-end integration

    // 1. Setup I/O
    let mut io = MockIo::new();

    // 2. Setup tree
    let tree: &'static Directory<MockAccessLevel> = &TEST_TREE;

    // 3. Setup handlers
    let handlers = MockHandlers;

    // 4. Create user
    let user = User {
        username: {
            let mut s = heapless::String::<32>::new();
            s.push_str("admin").unwrap();
            s
        },
        access_level: MockAccessLevel::Admin,
        #[cfg(feature = "authentication")]
        password_hash: [0u8; 32],
        #[cfg(feature = "authentication")]
        salt: [0u8; 16],
    };

    // 5. Parse path and navigate tree
    let path = TestPath::parse("/system/network/status").unwrap();
    assert_eq!(path.segments(), &["system", "network", "status"]);

    // Navigate to command
    let mut current: &Directory<MockAccessLevel> = tree;

    for segment in path.segments() {
        if let Some(Node::Directory(dir)) = current.find_child(segment) {
            current = dir;
        } else if let Some(Node::Command(cmd)) = current.find_child(segment) {
            // Found command - verify access
            assert!(user.access_level >= cmd.access_level);

            // Execute command
            let result = handlers.execute_sync(cmd.name, &[]);
            assert!(result.is_ok());

            let response = result.unwrap();

            // Write response to I/O
            io.write_str(response.message.as_str()).unwrap();
            io.write_str("\r\n").unwrap();

            break;
        }
    }

    // 6. Verify output was written
    assert!(!io.output().is_empty());
}

// ============================================================================
// Config Validation
// ============================================================================

#[test]
fn test_config_constants() {
    // DefaultConfig
    assert_eq!(DefaultConfig::MAX_INPUT, 128);
    assert_eq!(DefaultConfig::MAX_PATH_DEPTH, 8);
    assert_eq!(DefaultConfig::MAX_ARGS, 16);
    assert_eq!(DefaultConfig::MAX_PROMPT, 64);
    assert_eq!(DefaultConfig::MAX_RESPONSE, 256);
    #[cfg(feature = "history")]
    assert_eq!(DefaultConfig::HISTORY_SIZE, 10);
    #[cfg(not(feature = "history"))]
    assert_eq!(DefaultConfig::HISTORY_SIZE, 0);

    // MinimalConfig
    assert_eq!(MinimalConfig::MAX_INPUT, 64);
    assert_eq!(MinimalConfig::MAX_PATH_DEPTH, 4);
    assert_eq!(MinimalConfig::MAX_ARGS, 8);
    assert_eq!(MinimalConfig::MAX_PROMPT, 32);
    assert_eq!(MinimalConfig::MAX_RESPONSE, 128);
    #[cfg(feature = "history")]
    assert_eq!(MinimalConfig::HISTORY_SIZE, 4);
    #[cfg(not(feature = "history"))]
    assert_eq!(MinimalConfig::HISTORY_SIZE, 0);

    // MinimalConfig should be smaller in all dimensions
    assert!(MinimalConfig::MAX_INPUT < DefaultConfig::MAX_INPUT);
    assert!(MinimalConfig::MAX_PATH_DEPTH < DefaultConfig::MAX_PATH_DEPTH);
    assert!(MinimalConfig::MAX_ARGS < DefaultConfig::MAX_ARGS);
}
