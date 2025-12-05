//! Type-Level Integration Tests
//!
//! Validates that all core types integrate correctly across the library.
//!
//! **Purpose**: Catch type integration issues early through compile-time and runtime checks.
//!
//! **What we validate**:
//! - All foundational types instantiate without compilation errors
//! - Generic parameters (L, IO, H, C) work together correctly
//! - Path parsing and tree navigation work end-to-end
//! - Request/Response types integrate with tree and command handler
//! - Both DefaultConfig and MinimalConfig work correctly
//! - Lifetime relationships between tree and runtime state are sound

#[allow(clippy::duplicate_mod)]
#[path = "fixtures/mod.rs"]
mod fixtures;
use fixtures::{MockAccessLevel, MockHandler, MockIo, TEST_TREE};
use nut_shell::CharIo;
use nut_shell::auth::{AccessLevel, User};
use nut_shell::config::{DefaultConfig, MinimalConfig, ShellConfig};
use nut_shell::error::CliError;
use nut_shell::response::Response;
use nut_shell::shell::handler::CommandHandler;
use nut_shell::shell::{CliState, HistoryDirection, Request};
use nut_shell::tree::path::Path;
use nut_shell::tree::{CommandKind, CommandMeta, Directory};

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
    const _TREE: &Directory<MockAccessLevel> = &TEST_TREE;
    assert_eq!(TEST_TREE.name, "/");
    assert_eq!(TEST_TREE.access_level, MockAccessLevel::Guest);

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
    #[allow(irrefutable_let_patterns)]
    if let Request::Command { path, .. } = request {
        assert_eq!(path.as_str(), "system/status");
    }

    // CommandHandler trait
    let handler = MockHandler;
    let result = handler.execute_sync("echo", &["hello", "world"]);
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

    #[allow(irrefutable_let_patterns)]
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
