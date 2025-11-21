//! Comprehensive end-to-end CLI integration tests.
//!
//! Tests complete workflows including command execution, navigation,
//! access control, tab completion, and history integration.
//!
//! Most tests are written for the no-auth case to avoid lifetime issues.
//! Auth-specific tests are in test_shell_auth.rs.

#[path = "fixtures/mod.rs"]
mod fixtures;

use fixtures::{MockAccessLevel, MockHandlers, MockIo, TEST_TREE};
use nut_shell::config::DefaultConfig;
use nut_shell::shell::Shell;

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
    assert!(output.contains("hello world"), "Output should contain echo result");
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_command_not_found() {
    let io = MockIo::new();
    let handlers = MockHandlers;
    let mut shell = Shell::new(&TEST_TREE, handlers, io);
    shell.activate().unwrap();
    shell.__test_io_mut().clear_output();

    // Try nonexistent command
    for c in "nonexistent\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.__test_io_mut().output();
    assert!(
        output.contains("not found") || output.contains("Command not found"),
        "Should report command not found: {}",
        output
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
    assert!(output.contains("arg1") && output.contains("arg2") && output.contains("arg3"),
            "Should include all arguments");
}

// ============================================================================
// Navigation Tests
// ============================================================================

#[test]
#[cfg(not(feature = "authentication"))]
fn test_path_based_command_execution() {
    let io = MockIo::new();
    let handlers = MockHandlers;
    let mut shell = Shell::new(&TEST_TREE, handlers, io);
    shell.activate().unwrap();
    shell.__test_io_mut().clear_output();

    // Execute command via absolute path
    for c in "system/reboot\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.__test_io_mut().output();
    assert!(
        output.contains("Reboot") || output.contains("reboot"),
        "Should execute reboot command: {}",
        output
    );
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_nested_directory_navigation() {
    let io = MockIo::new();
    let handlers = MockHandlers;
    let mut shell = Shell::new(&TEST_TREE, handlers, io);
    shell.activate().unwrap();
    shell.__test_io_mut().clear_output();

    // Navigate to nested directory and execute command
    for c in "system/network/status\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.__test_io_mut().output();
    assert!(
        output.contains("status") || output.contains("Network"),
        "Should execute network status command: {}",
        output
    );
}

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
    assert!(output.contains("ls"), "Help should mention ls command: {}", output);
    assert!(output.contains("clear"), "Help should mention clear command: {}", output);
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
    assert!(buffer.contains("echo first"), "Should show older command: {}", buffer);

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
    assert!(!shell.__test_get_input_buffer().is_empty(), "Buffer should have content");

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
fn test_invalid_path() {
    let io = MockIo::new();
    let handlers = MockHandlers;
    let mut shell = Shell::new(&TEST_TREE, handlers, io);
    shell.activate().unwrap();
    shell.__test_io_mut().clear_output();

    // Try non-existent path
    for c in "invalid/path/command\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.__test_io_mut().output();
    assert!(
        output.contains("not found") || output.contains("Invalid"),
        "Should report invalid path: {}",
        output
    );
}

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
    assert!(output.contains("test"), "Backspace editing should work: {}", output);
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
        assert!(output.contains("minimal"), "Basic functionality should work");
    }
}
