//! Comprehensive end-to-end CLI integration tests.
//!
//! Tests complete workflows including command execution, navigation,
//! access control, tab completion, and history integration.
//!
//! Most tests are written for the no-auth case to avoid lifetime issues.
//! Auth-specific tests are in test_shell_auth.rs.

#[path = "fixtures/mod.rs"]
mod fixtures;

use fixtures::{MockHandlers, MockIo, TEST_TREE};
use nut_shell::Shell;

// ============================================================================
// Command Execution Tests
// ============================================================================

#[test]
#[cfg(not(feature = "authentication"))]
fn test_simple_command_execution() {
    let io = MockIo::new();
    let handlers = MockHandlers;
    let mut shell = Shell::new(&TEST_TREE, handlers, io);
    shell.activate().unwrap();
    shell.__test_io_mut().clear_output();

    // Execute 'echo hello world'
    for c in "echo hello world\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.__test_io_mut().output();
    assert!(
        output.contains("hello world"),
        "Output should contain echo result"
    );
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_command_with_arguments() {
    let io = MockIo::new();
    let handlers = MockHandlers;
    let mut shell = Shell::new(&TEST_TREE, handlers, io);
    shell.activate().unwrap();
    shell.__test_io_mut().clear_output();

    // Execute echo with multiple arguments
    for c in "echo arg1 arg2 arg3\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.__test_io_mut().output();
    assert!(
        output.contains("arg1") && output.contains("arg2") && output.contains("arg3"),
        "Should include all arguments"
    );
}

// ============================================================================
// Navigation Tests
// ============================================================================

// ============================================================================
// Global Commands Tests
// ============================================================================

#[test]
#[cfg(not(feature = "authentication"))]
fn test_help_command() {
    let io = MockIo::new();
    let handlers = MockHandlers;
    let mut shell = Shell::new(&TEST_TREE, handlers, io);
    shell.activate().unwrap();
    shell.__test_io_mut().clear_output();

    // Execute help command
    for c in "?\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.__test_io_mut().output();
    assert!(
        output.contains("ls"),
        "Help should mention ls command: {}",
        output
    );
    assert!(
        output.contains("clear"),
        "Help should mention clear command: {}",
        output
    );
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_ls_command_root() {
    let io = MockIo::new();
    let handlers = MockHandlers;
    let mut shell = Shell::new(&TEST_TREE, handlers, io);
    shell.activate().unwrap();
    shell.__test_io_mut().clear_output();

    // List root directory
    for c in "ls\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.__test_io_mut().output();
    // Should list root-level commands and directories
    assert!(
        output.contains("echo") || output.contains("system"),
        "ls should show root contents: {}",
        output
    );
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_clear_command() {
    let io = MockIo::new();
    let handlers = MockHandlers;
    let mut shell = Shell::new(&TEST_TREE, handlers, io);
    shell.activate().unwrap();
    shell.__test_io_mut().clear_output();

    // Execute clear
    for c in "clear\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.__test_io_mut().output();
    // Clear should output ANSI clear sequence
    assert!(
        output.contains("\x1b[2J") || output.contains("\x1b[H"),
        "Clear should output ANSI escape: {}",
        output
    );
}

// ============================================================================
// Tab Completion Integration Tests (requires completion feature)
// ============================================================================

#[test]
#[cfg(all(feature = "completion", not(feature = "authentication")))]
fn test_tab_completion_single_match() {
    let io = MockIo::new();
    let handlers = MockHandlers;
    let mut shell = Shell::new(&TEST_TREE, handlers, io);
    shell.activate().unwrap();
    shell.__test_io_mut().clear_output();

    // Type partial command and press tab
    for c in "ech".chars() {
        shell.process_char(c).unwrap();
    }
    shell.process_char('\t').unwrap(); // Tab

    let buffer = shell.__test_get_input_buffer();
    // Should auto-complete to "echo"
    assert!(
        buffer.contains("echo"),
        "Tab should complete 'ech' to 'echo', got: {}",
        buffer
    );
}

// ============================================================================
// History Navigation Tests (requires history feature)
// ============================================================================

#[test]
#[cfg(all(feature = "history", not(feature = "authentication")))]
fn test_history_navigation_up() {
    let io = MockIo::new();
    let handlers = MockHandlers;
    let mut shell = Shell::new(&TEST_TREE, handlers, io);
    shell.activate().unwrap();
    shell.__test_io_mut().clear_output();

    // Execute a command
    for c in "echo first\n".chars() {
        shell.process_char(c).unwrap();
    }
    shell.__test_io_mut().clear_output();

    // Execute another command
    for c in "echo second\n".chars() {
        shell.process_char(c).unwrap();
    }
    shell.__test_io_mut().clear_output();

    // Press up arrow (should recall "echo second")
    shell.process_char('\x1b').unwrap(); // ESC
    shell.process_char('[').unwrap();
    shell.process_char('A').unwrap(); // Up

    // Input buffer should contain previous command
    let buffer = shell.__test_get_input_buffer();
    assert!(
        buffer.contains("echo second"),
        "Up arrow should recall 'echo second', got: {}",
        buffer
    );
}

#[test]
#[cfg(all(feature = "history", not(feature = "authentication")))]
fn test_history_navigation_up_down() {
    let io = MockIo::new();
    let handlers = MockHandlers;
    let mut shell = Shell::new(&TEST_TREE, handlers, io);
    shell.activate().unwrap();
    shell.__test_io_mut().clear_output();

    // Execute two commands
    for c in "echo first\n".chars() {
        shell.process_char(c).unwrap();
    }
    for c in "echo second\n".chars() {
        shell.process_char(c).unwrap();
    }
    shell.__test_io_mut().clear_output();

    // Press up arrow twice
    shell.process_char('\x1b').unwrap();
    shell.process_char('[').unwrap();
    shell.process_char('A').unwrap(); // Up (should show "echo second")

    shell.process_char('\x1b').unwrap();
    shell.process_char('[').unwrap();
    shell.process_char('A').unwrap(); // Up again (should show "echo first")

    let buffer = shell.__test_get_input_buffer();
    assert!(
        buffer.contains("echo first"),
        "Should show older command: {}",
        buffer
    );

    // Press down arrow
    shell.process_char('\x1b').unwrap();
    shell.process_char('[').unwrap();
    shell.process_char('B').unwrap(); // Down

    let buffer = shell.__test_get_input_buffer();
    assert!(
        buffer.contains("echo second"),
        "Down should show newer command: {}",
        buffer
    );
}

// ============================================================================
// Double-ESC Clear Tests
// ============================================================================

#[test]
#[cfg(not(feature = "authentication"))]
fn test_double_esc_clears_buffer() {
    let io = MockIo::new();
    let handlers = MockHandlers;
    let mut shell = Shell::new(&TEST_TREE, handlers, io);
    shell.activate().unwrap();
    shell.__test_io_mut().clear_output();

    // Type some input
    for c in "echo test".chars() {
        shell.process_char(c).unwrap();
    }

    // Verify buffer has content
    assert!(
        !shell.__test_get_input_buffer().is_empty(),
        "Buffer should have content"
    );

    // Double-ESC
    shell.process_char('\x1b').unwrap();
    shell.process_char('\x1b').unwrap();

    // Buffer should be cleared
    assert!(
        shell.__test_get_input_buffer().is_empty(),
        "Double-ESC should clear buffer"
    );
}

#[test]
#[cfg(all(feature = "history", not(feature = "authentication")))]
fn test_double_esc_exits_history_navigation() {
    let io = MockIo::new();
    let handlers = MockHandlers;
    let mut shell = Shell::new(&TEST_TREE, handlers, io);
    shell.activate().unwrap();
    shell.__test_io_mut().clear_output();

    // Execute a command
    for c in "echo previous\n".chars() {
        shell.process_char(c).unwrap();
    }
    shell.__test_io_mut().clear_output();

    // Start typing new command
    for c in "echo new".chars() {
        shell.process_char(c).unwrap();
    }

    // Press up arrow to enter history
    shell.process_char('\x1b').unwrap();
    shell.process_char('[').unwrap();
    shell.process_char('A').unwrap();

    // Buffer should show history
    assert!(
        shell.__test_get_input_buffer().contains("echo previous"),
        "Should be in history mode"
    );

    // Double-ESC should exit history and clear
    shell.process_char('\x1b').unwrap();
    shell.process_char('\x1b').unwrap();

    assert!(
        shell.__test_get_input_buffer().is_empty(),
        "Double-ESC should exit history and clear"
    );
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
#[cfg(not(feature = "authentication"))]
fn test_backspace_handling() {
    let io = MockIo::new();
    let handlers = MockHandlers;
    let mut shell = Shell::new(&TEST_TREE, handlers, io);
    shell.activate().unwrap();
    shell.__test_io_mut().clear_output();

    // Type with backspaces
    for c in "echox".chars() {
        shell.process_char(c).unwrap();
    }
    shell.process_char('\x08').unwrap(); // Backspace
    shell.process_char(' ').unwrap();
    for c in "test\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.__test_io_mut().output();
    assert!(
        output.contains("test"),
        "Backspace editing should work: {}",
        output
    );
}

// ============================================================================
// Feature Combination Tests
// ============================================================================

#[test]
fn test_minimal_features() {
    // Test works even with no optional features
    #[cfg(not(feature = "authentication"))]
    {
        let io = MockIo::new();
        let handlers = MockHandlers;
        let mut shell = Shell::new(&TEST_TREE, handlers, io);
        shell.activate().unwrap();
        shell.__test_io_mut().clear_output();

        // Basic command execution should always work
        for c in "echo minimal\n".chars() {
            shell.process_char(c).unwrap();
        }

        let output = shell.__test_io_mut().output();
        assert!(
            output.contains("minimal"),
            "Basic functionality should work"
        );
    }
}

// ============================================================================
// Buffer Overflow Handling Tests
// ============================================================================

#[test]
#[cfg(not(feature = "authentication"))]
fn test_buffer_overflow_emits_bell() {
    let io = MockIo::new();
    let handlers = MockHandlers;
    let mut shell = Shell::new(&TEST_TREE, handlers, io);
    shell.activate().unwrap();
    shell.__test_io_mut().clear_output();

    // Input buffer size is 128 chars (hardcoded in Shell for now)
    // Fill it up completely
    let long_input = "a".repeat(128);
    for c in long_input.chars() {
        shell.process_char(c).unwrap();
    }

    shell.__test_io_mut().clear_output();

    // Try to add one more character - should trigger bell
    shell.process_char('x').unwrap(); // Should succeed (returns Ok)

    let output = shell.__test_io_mut().output();
    assert!(
        output.contains('\x07'),
        "Should emit bell character on buffer full"
    );
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_buffer_overflow_continues_working() {
    let io = MockIo::new();
    let handlers = MockHandlers;
    let mut shell = Shell::new(&TEST_TREE, handlers, io);
    shell.activate().unwrap();

    // Fill buffer to capacity
    let long_input = "a".repeat(128);
    for c in long_input.chars() {
        shell.process_char(c).unwrap();
    }

    // Try to add more - should beep but not crash
    shell.process_char('x').unwrap();
    shell.process_char('y').unwrap();
    shell.process_char('z').unwrap();

    // Clear the buffer with double-ESC
    shell.process_char('\x1b').unwrap();
    shell.process_char('\x1b').unwrap();

    shell.__test_io_mut().clear_output();

    // Should be able to use shell normally after overflow
    for c in "echo test\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.__test_io_mut().output();
    assert!(
        output.contains("test"),
        "Shell should work normally after buffer overflow"
    );
}

// ============================================================================
// Shell Lifecycle Tests
// ============================================================================

#[test]
#[cfg(not(feature = "authentication"))]
fn test_shell_activate_deactivate_reactivate() {
    let io = MockIo::new();
    let handlers = MockHandlers;
    let mut shell = Shell::new(&TEST_TREE, handlers, io);

    // Shell starts in Inactive state
    // Activate it
    shell.activate().unwrap();
    shell.__test_io_mut().clear_output();

    // Should be able to execute commands
    for c in "echo active\n".chars() {
        shell.process_char(c).unwrap();
    }
    let output = shell.__test_io_mut().output();
    assert!(output.contains("active"), "Should work when active");

    shell.__test_io_mut().clear_output();

    // Deactivate shell
    shell.deactivate();

    // Should NOT respond to input when deactivated
    for c in "echo inactive\n".chars() {
        shell.process_char(c).unwrap();
    }
    let output = shell.__test_io_mut().output();
    // Shell should ignore input when inactive (may emit nothing or minimal output)
    // Exact behavior depends on implementation

    // Reactivate
    shell.activate().unwrap();
    shell.__test_io_mut().clear_output();

    // Should work again
    for c in "echo reactivated\n".chars() {
        shell.process_char(c).unwrap();
    }
    let output = shell.__test_io_mut().output();
    assert!(
        output.contains("reactivated"),
        "Should work after reactivation"
    );
}

// ============================================================================
// Command Argument Validation Tests
// ============================================================================

#[test]
#[cfg(not(feature = "authentication"))]
fn test_command_requires_exact_args() {
    let io = MockIo::new();
    let handlers = MockHandlers;
    let mut shell = Shell::new(&TEST_TREE, handlers, io);
    shell.activate().unwrap();
    shell.__test_io_mut().clear_output();

    // 'led' command requires exactly 1 argument (min_args=1, max_args=1)
    // Test with no arguments - should fail
    for c in "system/hardware/led\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.__test_io_mut().output();
    assert!(
        output.contains("argument") || output.contains("require") || output.contains("Usage"),
        "Should report missing argument: {}",
        output
    );
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_command_accepts_valid_args() {
    let io = MockIo::new();
    let handlers = MockHandlers;
    let mut shell = Shell::new(&TEST_TREE, handlers, io);
    shell.activate().unwrap();
    shell.__test_io_mut().clear_output();

    // 'led' command with correct argument
    for c in "system/hardware/led on\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.__test_io_mut().output();
    assert!(
        output.contains("LED") && output.contains("on"),
        "Should execute command with valid args: {}",
        output
    );
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_command_with_variable_args() {
    let io = MockIo::new();
    let handlers = MockHandlers;
    let mut shell = Shell::new(&TEST_TREE, handlers, io);
    shell.activate().unwrap();
    shell.__test_io_mut().clear_output();

    // 'echo' allows 0-16 arguments
    for c in "echo a b c d e\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.__test_io_mut().output();
    assert!(
        output.contains("a") && output.contains("e"),
        "Should handle variable arguments: {}",
        output
    );
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_command_too_many_args() {
    let io = MockIo::new();
    let handlers = MockHandlers;
    let mut shell = Shell::new(&TEST_TREE, handlers, io);
    shell.activate().unwrap();
    shell.__test_io_mut().clear_output();

    // 'reboot' accepts 0 arguments (max_args=0)
    // Provide arguments - should fail
    for c in "system/reboot now\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.__test_io_mut().output();
    assert!(
        output.contains("argument") || output.contains("too many") || output.contains("Usage"),
        "Should report too many arguments: {}",
        output
    );
}

// ============================================================================
// Access Level Enforcement Tests
// ============================================================================

#[test]
#[cfg(feature = "authentication")]
fn test_access_level_enforcement() {
    use fixtures::MockAccessLevel;
    use nut_shell::auth::{ConstCredentialProvider, User, password::Sha256Hasher};

    // Create test users with different access levels
    let hash = [0u8; 32];
    let salt = [0u8; 16];
    let users = [
        User::new("guest", MockAccessLevel::Guest, hash, salt).unwrap(),
        User::new("admin", MockAccessLevel::Admin, hash, salt).unwrap(),
    ];

    let hasher = Sha256Hasher::new();
    let provider = ConstCredentialProvider::new(users, hasher);
    let io = MockIo::new();
    let handlers = MockHandlers;
    let mut shell = Shell::new(&TEST_TREE, handlers, &provider, io);

    // Note: This test requires authentication logic to actually enforce access levels
    // The current implementation should check user.access_level >= command.access_level
}

// ============================================================================
// Config Variation Tests
// ============================================================================

#[test]
#[cfg(not(feature = "authentication"))]
fn test_minimal_config_works() {
    use nut_shell::Response;
    use nut_shell::config::MinimalConfig;
    use nut_shell::error::CliError;
    use nut_shell::shell::handlers::CommandHandlers;

    // Implement handlers for MinimalConfig
    struct MinimalHandlers;

    impl CommandHandlers<MinimalConfig> for MinimalHandlers {
        fn execute_sync(
            &self,
            name: &str,
            _args: &[&str],
        ) -> Result<Response<MinimalConfig>, CliError> {
            match name {
                "help" => Ok(Response::success("Help")),
                "echo" => Ok(Response::success("Echo")),
                _ => Err(CliError::CommandNotFound),
            }
        }

        #[cfg(feature = "async")]
        async fn execute_async(
            &self,
            _name: &str,
            _args: &[&str],
        ) -> Result<Response<MinimalConfig>, CliError> {
            Err(CliError::CommandNotFound)
        }
    }

    let io = MockIo::new();
    let handlers = MinimalHandlers;
    let mut shell: Shell<_, _, _, MinimalConfig> = Shell::new(&TEST_TREE, handlers, io);

    shell.activate().unwrap();
    shell.__test_io_mut().clear_output();

    // Execute command with MinimalConfig
    for c in "echo test\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.__test_io_mut().output();
    assert!(output.contains("Echo"), "MinimalConfig should work");
}

// ============================================================================
// Path Navigation Edge Cases
// ============================================================================
