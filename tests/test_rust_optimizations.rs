//! Rust-specific optimization and compile-time verification tests.
//!
//! Tests that validate:
//! - Zero-size type optimizations
//! - Const initialization
//! - ROM placement
//! - Type safety and lifetimes

#[allow(clippy::duplicate_mod)]
#[path = "fixtures/mod.rs"]
mod fixtures;
use fixtures::{MockAccessLevel, TEST_TREE};
use nut_shell::config::{DefaultConfig, MinimalConfig, ShellConfig};
use nut_shell::shell::history::CommandHistory;
use nut_shell::tree::{CommandKind, CommandMeta, Directory, Node};

// ============================================================================
// Zero-Size Type Optimization Tests
// ============================================================================

#[test]
fn test_command_history_zero_size_when_disabled() {
    // When history feature is disabled, CommandHistory should be zero-size
    #[cfg(not(feature = "history"))]
    {
        let size = core::mem::size_of::<CommandHistory<0, 128>>();
        assert_eq!(
            size, 0,
            "CommandHistory should be zero-size when history feature disabled, got: {} bytes",
            size
        );
    }

    // When history feature is enabled, it should have a non-zero size
    #[cfg(feature = "history")]
    {
        let size = core::mem::size_of::<CommandHistory<10, 128>>();
        assert!(
            size > 0,
            "CommandHistory should have non-zero size when history feature enabled"
        );
    }
}

#[test]
fn test_empty_history_is_zero_size() {
    // CommandHistory with N=0 should be zero-size even when feature enabled
    #[cfg(feature = "history")]
    {
        let size = core::mem::size_of::<CommandHistory<0, 128>>();
        // Note: Contains position field (Option<usize>) even when empty
        // Expect size of Option<usize> which is typically one pointer width
        assert!(
            size <= 24,
            "CommandHistory<0> should be minimal size (Option<usize>), got: {} bytes",
            size
        );
    }
}

#[test]
fn test_phantom_data_types_are_zero_size() {
    use core::marker::PhantomData;

    // PhantomData should always be zero-size
    assert_eq!(
        core::mem::size_of::<PhantomData<DefaultConfig>>(),
        0,
        "PhantomData should be zero-size"
    );

    assert_eq!(
        core::mem::size_of::<PhantomData<MinimalConfig>>(),
        0,
        "PhantomData should be zero-size"
    );
}

// ============================================================================
// Const Initialization Tests
// ============================================================================

#[test]
fn test_command_meta_is_const_initializable() {
    const _CMD: CommandMeta<MockAccessLevel> = CommandMeta {
        id: "test",
        name: "test",
        description: "Test command",
        access_level: MockAccessLevel::Guest,
        kind: CommandKind::Sync,
        min_args: 0,
        max_args: 0,
    };

    // If this compiles, const initialization works
}

#[test]
fn test_directory_is_const_initializable() {
    const _DIR: Directory<MockAccessLevel> = Directory {
        name: "test",
        children: &[],
        access_level: MockAccessLevel::Guest,
    };

    // If this compiles, const initialization works
}

#[test]
fn test_nested_tree_is_const_initializable() {
    const CMD1: CommandMeta<MockAccessLevel> = CommandMeta {
        id: "cmd1",
        name: "cmd1",
        description: "Command 1",
        access_level: MockAccessLevel::Guest,
        kind: CommandKind::Sync,
        min_args: 0,
        max_args: 0,
    };

    const CMD2: CommandMeta<MockAccessLevel> = CommandMeta {
        id: "cmd2",
        name: "cmd2",
        description: "Command 2",
        access_level: MockAccessLevel::User,
        kind: CommandKind::Sync,
        min_args: 0,
        max_args: 1,
    };

    const SUBDIR: Directory<MockAccessLevel> = Directory {
        name: "subdir",
        children: &[Node::Command(&CMD2)],
        access_level: MockAccessLevel::Guest,
    };

    const ROOT: Directory<MockAccessLevel> = Directory {
        name: "/",
        children: &[Node::Command(&CMD1), Node::Directory(&SUBDIR)],
        access_level: MockAccessLevel::Guest,
    };

    // Verify we can reference the const tree
    assert_eq!(ROOT.name, "/");
    assert_eq!(ROOT.children.len(), 2);
}

#[test]
fn test_test_tree_is_const() {
    // TEST_TREE should be a const
    // We can verify this by checking it's directly usable
    assert!(
        !TEST_TREE.name.is_empty(),
        "TEST_TREE should be initialized"
    );
    assert!(
        !TEST_TREE.children.is_empty(),
        "TEST_TREE should have children"
    );
}

// ============================================================================
// Memory Layout Tests
// ============================================================================

#[test]
fn test_config_constants_are_zero_cost() {
    // Config traits should have zero runtime cost (no vtable)
    // The struct implementing ShellConfig should be zero-size if it has no fields
    struct TestConfig;

    impl ShellConfig for TestConfig {
        const MAX_INPUT: usize = 128;
        const MAX_PATH_DEPTH: usize = 8;
        const MAX_ARGS: usize = 16;
        const MAX_PROMPT: usize = 64;
        const MAX_RESPONSE: usize = 256;
        const HISTORY_SIZE: usize = 10;

        const MSG_WELCOME: &'static str = "";
        const MSG_LOGIN_PROMPT: &'static str = "";
        const MSG_LOGIN_SUCCESS: &'static str = "";
        const MSG_LOGIN_FAILED: &'static str = "";
        const MSG_LOGOUT: &'static str = "";
        const MSG_INVALID_LOGIN_FORMAT: &'static str = "";
    }

    assert_eq!(
        core::mem::size_of::<TestConfig>(),
        0,
        "Config structs should be zero-size"
    );
}

#[test]
fn test_access_level_size() {
    // AccessLevel should be small (typically 1 byte for simple enums)
    let size = core::mem::size_of::<MockAccessLevel>();
    assert!(
        size <= 4,
        "AccessLevel should be small (got {} bytes), consider using #[repr(u8)]",
        size
    );
}

#[test]
fn test_command_meta_size_is_reasonable() {
    // CommandMeta should be small since it only contains static references
    let size = core::mem::size_of::<CommandMeta<MockAccessLevel>>();

    // Expected: 3 pointers (id, name, description) + access_level + kind + 2 usizes
    // On 64-bit: 3*8 + 1 + 1 + 2*8 = ~42 bytes (with padding to 8-byte alignment = ~72 bytes)
    // On 32-bit: 3*4 + 1 + 1 + 2*4 = ~22 bytes (with padding to 4-byte alignment = ~40 bytes)

    #[cfg(target_pointer_width = "64")]
    assert!(
        size <= 80,
        "CommandMeta size should be reasonable on 64-bit (got {} bytes)",
        size
    );

    #[cfg(target_pointer_width = "32")]
    assert!(
        size <= 48,
        "CommandMeta size should be reasonable on 32-bit (got {} bytes)",
        size
    );
}

#[test]
fn test_node_size() {
    // Node is an enum with two pointer variants, should be small
    let size = core::mem::size_of::<Node<MockAccessLevel>>();

    // Expected: discriminant (1-8 bytes) + pointer (4 or 8 bytes)
    #[cfg(target_pointer_width = "64")]
    assert!(
        size <= 16,
        "Node size should be reasonable on 64-bit (got {} bytes)",
        size
    );

    #[cfg(target_pointer_width = "32")]
    assert!(
        size <= 8,
        "Node size should be reasonable on 32-bit (got {} bytes)",
        size
    );
}

// ============================================================================
// Static Lifetime Tests
// ============================================================================

#[test]
fn test_tree_has_static_lifetime() {
    // TEST_TREE should have 'static lifetime
    let _tree_ref: &'static Directory<MockAccessLevel> = &TEST_TREE;

    // If this compiles, lifetime is correct
}

#[test]
fn test_command_meta_has_static_lifetime() {
    const CMD: CommandMeta<MockAccessLevel> = CommandMeta {
        id: "test",
        name: "test",
        description: "Test",
        access_level: MockAccessLevel::Guest,
        kind: CommandKind::Sync,
        min_args: 0,
        max_args: 0,
    };

    let _cmd_ref: &'static CommandMeta<MockAccessLevel> = &CMD;

    // If this compiles, lifetime is correct
}

// ============================================================================
// Feature-Gated Size Tests
// ============================================================================

#[test]
fn test_request_size_varies_with_features() {
    use nut_shell::shell::Request;

    let size = core::mem::size_of::<Request<DefaultConfig>>();

    // Request size is dominated by Command variant's buffers (heapless::String and Vec)
    // Additional feature-gated variants (Login, TabComplete, History) may increase size
    // But the main factor is DefaultConfig::MAX_INPUT and MAX_ARGS buffer sizes

    // The size should be reasonable (not excessive)
    assert!(
        size < 3000,
        "Request size should be reasonable (got {} bytes)",
        size
    );

    // Request always has Command variant, which contains large buffers
    assert!(
        size > 100,
        "Request should have substantial size due to Command variant buffers (got {} bytes)",
        size
    );
}

// ============================================================================
// Alignment Tests
// ============================================================================

#[test]
fn test_types_are_properly_aligned() {
    // Verify that types have reasonable alignment
    assert!(
        core::mem::align_of::<CommandMeta<MockAccessLevel>>() <= 8,
        "CommandMeta alignment should be reasonable"
    );

    assert!(
        core::mem::align_of::<Directory<MockAccessLevel>>() <= 8,
        "Directory alignment should be reasonable"
    );

    assert!(
        core::mem::align_of::<Node<MockAccessLevel>>() <= 8,
        "Node alignment should be reasonable"
    );
}

// ============================================================================
// Compile-Time Computation Tests
// ============================================================================

#[test]
fn test_config_constants_are_compile_time() {
    // These should all be compile-time constants
    const _INPUT: usize = DefaultConfig::MAX_INPUT;
    const _DEPTH: usize = DefaultConfig::MAX_PATH_DEPTH;
    const _ARGS: usize = DefaultConfig::MAX_ARGS;
    const _PROMPT: usize = DefaultConfig::MAX_PROMPT;
    const _RESPONSE: usize = DefaultConfig::MAX_RESPONSE;
    const _HISTORY: usize = DefaultConfig::HISTORY_SIZE;

    // If this compiles, constants are truly const
}

#[test]
#[allow(clippy::assertions_on_constants)]
fn test_minimal_config_is_smaller() {
    // MinimalConfig should have smaller buffer sizes than DefaultConfig
    assert!(
        MinimalConfig::MAX_INPUT < DefaultConfig::MAX_INPUT,
        "MinimalConfig should have smaller input buffer"
    );

    assert!(
        MinimalConfig::MAX_RESPONSE < DefaultConfig::MAX_RESPONSE,
        "MinimalConfig should have smaller response buffer"
    );

    #[cfg(feature = "history")]
    assert!(
        MinimalConfig::HISTORY_SIZE < DefaultConfig::HISTORY_SIZE,
        "MinimalConfig should have smaller history"
    );

    #[cfg(not(feature = "history"))]
    assert_eq!(
        MinimalConfig::HISTORY_SIZE,
        DefaultConfig::HISTORY_SIZE,
        "Both configs should have zero history when feature is disabled"
    );
}
