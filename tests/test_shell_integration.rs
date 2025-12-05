//! Comprehensive end-to-end CLI integration tests.
//!
//! Tests complete workflows including command execution, navigation,
//! access control, tab completion, and history integration.
//!
//! Most tests are written for the no-auth case to avoid lifetime issues.
//! Auth-specific tests are in test_shell_auth.rs.

#[allow(clippy::duplicate_mod)]
#[path = "fixtures/mod.rs"]
mod fixtures;

#[path = "helpers.rs"]
mod helpers;

use fixtures::TEST_TREE;
use nut_shell::Shell;

// ============================================================================
// Command Execution Tests
// ============================================================================

#[test]
#[cfg(not(feature = "authentication"))]
fn test_command_with_arguments() {
    let mut shell = helpers::create_test_shell();

    let output = helpers::execute_command(&mut shell, "echo arg1 arg2 arg3");

    helpers::assert_contains_all(&output, &["arg1", "arg2", "arg3"]);
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_path_based_command_execution() {
    // Table-driven test for path-based execution scenarios
    let test_cases = [
        // (command, expected_output, description)
        ("system/status", "System OK", "simple path without navigation"),
        ("system/network/status", "Network OK", "deeply nested path"),
        ("system/hardware/led on", "LED: on", "path with arguments"),
    ];

    for (cmd, expected, description) in test_cases {
        let mut shell = helpers::create_test_shell();
        let output = helpers::execute_command(&mut shell, cmd);

        assert!(
            output.contains(expected),
            "Failed '{}': expected '{}' in output: {}",
            description,
            expected,
            output
        );
    }
}

// ============================================================================
// Directory Navigation Tests
// ============================================================================

#[test]
#[cfg(not(feature = "authentication"))]
fn test_navigate_to_directory() {
    let mut shell = helpers::create_test_shell();

    // Navigate to system directory
    helpers::execute_command(&mut shell, "system");

    // Execute command in navigated directory
    let output = helpers::execute_command(&mut shell, "status");

    helpers::assert_contains_all(&output, &["@/system>", "System OK"]);
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_navigate_with_relative_path() {
    let mut shell = helpers::create_test_shell();

    // Navigate using multi-segment relative path
    helpers::execute_command(&mut shell, "system/network");

    let output = helpers::execute_command(&mut shell, "status");

    helpers::assert_contains_all(&output, &["@/system/network>", "Network OK"]);
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_navigate_parent_directory() {
    let mut shell = helpers::create_test_shell();

    // Navigate to system/network
    helpers::execute_command(&mut shell, "system/network");

    // Navigate up one level using ..
    helpers::execute_command(&mut shell, "..");

    // Should be in system/ now
    let output = helpers::execute_command(&mut shell, "status");

    helpers::assert_contains_all(&output, &["@/system>", "System OK"]);
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_navigate_absolute_path() {
    let mut shell = helpers::create_test_shell();

    // Navigate to system first
    helpers::execute_command(&mut shell, "system");

    // Navigate to debug using absolute path
    helpers::execute_command(&mut shell, "/debug");

    // Should be in debug/
    let output = helpers::execute_command(&mut shell, "memory");

    helpers::assert_contains_all(&output, &["@/debug>", "Memory"]);
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_navigate_invalid_directory() {
    let mut shell = helpers::create_test_shell();

    let output = helpers::execute_command(&mut shell, "nonexistent");

    assert!(output.contains("Error: Command not found"));
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_navigate_parent_beyond_root() {
    let mut shell = helpers::create_test_shell();

    // Try to navigate above root
    helpers::execute_command(&mut shell, "..");

    // Should still be at root
    let output = helpers::execute_command(&mut shell, "echo still at root");

    helpers::assert_contains_all(&output, &["@/>", "still at root"]);
}

// ============================================================================
// Global Commands Tests
// ============================================================================

#[test]
#[cfg(not(feature = "authentication"))]
fn test_global_commands() {
    // Table-driven test for global commands
    let test_cases = [
        ("?", &["ls", "clear"] as &[&str], "help command shows available commands"),
        ("ls", &["echo", "system"], "ls lists root contents"),
        ("clear", &["\x1b[2J"], "clear outputs ANSI escape sequence"),
    ];

    for (cmd, expected_fragments, description) in test_cases {
        let mut shell = helpers::create_test_shell();
        let output = helpers::execute_command(&mut shell, cmd);

        for fragment in expected_fragments {
            assert!(
                output.contains(fragment),
                "Failed '{}': expected '{}' in output",
                description,
                fragment
            );
        }
    }
}

// ============================================================================
// Tab Completion Integration Tests (requires completion feature)
// ============================================================================

#[test]
#[cfg(all(feature = "completion", not(feature = "authentication")))]
fn test_tab_completion_single_match() {
    let mut shell = helpers::create_test_shell();

    // Type partial command
    helpers::type_input(&mut shell, "ech");

    shell.io_mut().clear_output();

    // Press tab - should emit "o" to complete "echo"
    shell.process_char('\t').unwrap();

    let completion_output = shell.io_mut().output();
    assert!(
        completion_output.contains('o'),
        "Tab should complete 'ech' to 'echo': {}",
        completion_output
    );

    // Execute with an argument
    let output = helpers::execute_command(&mut shell, " completion_test");

    assert!(output.contains("completion_test"));
}

// ============================================================================
// History Navigation Tests (requires history feature)
// ============================================================================

/// Helper to create a shell with populated history.
#[cfg(all(feature = "history", not(feature = "authentication")))]
fn setup_shell_with_history(commands: &[&str]) -> Shell<'static, fixtures::MockAccessLevel, MockIo, MockHandler, nut_shell::config::DefaultConfig> {
    let mut shell = helpers::create_test_shell();

    for cmd in commands {
        helpers::execute_command(&mut shell, cmd);
    }

    shell.io_mut().clear_output();
    shell
}

#[test]
#[cfg(all(feature = "history", not(feature = "authentication")))]
fn test_history_navigation_up() {
    let mut shell = setup_shell_with_history(&["echo first", "echo second"]);

    // Press up arrow (should recall "echo second")
    shell.process_char('\x1b').unwrap(); // ESC
    shell.process_char('[').unwrap();
    shell.process_char('A').unwrap(); // Up

    let output = shell.io_mut().output();
    assert!(output.contains("echo second"));

    // Execute and verify
    shell.io_mut().clear_output();
    shell.process_char('\n').unwrap();

    let output = shell.io_mut().output();
    assert!(output.contains("second"));
}

#[test]
#[cfg(all(feature = "history", not(feature = "authentication")))]
fn test_history_navigation_up_multiple() {
    let mut shell = setup_shell_with_history(&["echo first", "echo second"]);

    // Press up arrow once - should recall "echo second"
    shell.process_char('\x1b').unwrap();
    shell.process_char('[').unwrap();
    shell.process_char('A').unwrap();

    let output = shell.io_mut().output();
    assert!(output.contains("echo second"));

    shell.io_mut().clear_output();

    // Press up arrow again - should recall "echo first"
    shell.process_char('\x1b').unwrap();
    shell.process_char('[').unwrap();
    shell.process_char('A').unwrap();

    let output = shell.io_mut().output();
    assert!(output.contains("echo first"));

    // Execute to verify the buffer contains "echo first"
    shell.io_mut().clear_output();
    shell.process_char('\n').unwrap();

    let output = shell.io_mut().output();
    assert!(output.contains("first"));
}

#[test]
#[cfg(all(feature = "history", not(feature = "authentication")))]
fn test_history_navigation_down() {
    let mut shell = setup_shell_with_history(&["echo first", "echo second", "echo third"]);

    // Navigate up twice to get to "echo second"
    shell.process_char('\x1b').unwrap();
    shell.process_char('[').unwrap();
    shell.process_char('A').unwrap(); // Up to "echo third"

    shell.io_mut().clear_output();

    shell.process_char('\x1b').unwrap();
    shell.process_char('[').unwrap();
    shell.process_char('A').unwrap(); // Up to "echo second"

    let output = shell.io_mut().output();
    assert!(output.contains("echo second"));

    shell.io_mut().clear_output();

    // Press down arrow - should move forward to "echo third"
    shell.process_char('\x1b').unwrap();
    shell.process_char('[').unwrap();
    shell.process_char('B').unwrap();

    let output = shell.io_mut().output();
    assert!(output.contains("echo third"));

    // Execute to verify
    shell.io_mut().clear_output();
    shell.process_char('\n').unwrap();

    let output = shell.io_mut().output();
    assert!(output.contains("third"));
}

// ============================================================================
// Double-ESC Clear Tests
// ============================================================================

#[test]
#[cfg(not(feature = "authentication"))]
fn test_double_esc_clears_buffer() {
    let mut shell = helpers::create_test_shell();

    // Type some input but don't execute
    helpers::type_input(&mut shell, "echo test");

    // Verify input was echoed before clearing
    let output_before = shell.io_mut().output();
    assert!(output_before.contains("echo test"));

    shell.io_mut().clear_output();

    // Double-ESC should clear the buffer
    shell.process_char('\x1b').unwrap();
    shell.process_char('\x1b').unwrap();

    let clear_output = shell.io_mut().output();

    // Should send clear sequence: \r (CR) + \x1b[K (clear to end of line) + prompt
    helpers::assert_contains_ansi(&clear_output, "\r");
    helpers::assert_contains_ansi(&clear_output, "\x1b[K");

    // Now press enter - nothing should execute since buffer was cleared
    shell.io_mut().clear_output();
    shell.process_char('\n').unwrap();

    let output = shell.io_mut().output();
    helpers::assert_contains_none(&output, &["test"]);
}

// ============================================================================
// Input Editing Tests
// ============================================================================

#[test]
#[cfg(not(feature = "authentication"))]
fn test_backspace_editing() {
    let mut shell = helpers::create_test_shell();

    // Type with backspaces and verify proper editing
    helpers::type_input(&mut shell, "echox");
    shell.process_char('\x08').unwrap(); // Backspace
    helpers::type_input(&mut shell, " test");

    shell.io_mut().clear_output();
    shell.process_char('\n').unwrap();

    let output = shell.io_mut().output();
    assert!(output.contains("test"), "Backspace editing should work");
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_backspace_at_start() {
    let mut shell = helpers::create_test_shell();

    // Backspace on empty buffer should not crash or produce unexpected output
    shell.process_char('\x08').unwrap();

    // Should be able to type and execute normally
    let output = helpers::execute_command(&mut shell, "echo ok");
    assert!(output.contains("ok"));
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_backspace_boundary() {
    let mut shell = helpers::create_test_shell();

    // Type, backspace everything, then type new command
    helpers::type_input(&mut shell, "wrong");

    // Backspace 5 times (5 characters)
    for _ in 0..5 {
        shell.process_char('\x08').unwrap();
    }

    // Additional backspaces should not cause issues
    shell.process_char('\x08').unwrap();
    shell.process_char('\x08').unwrap();

    // Type correct command
    let output = helpers::execute_command(&mut shell, "echo correct");
    assert!(output.contains("correct"));
}

// ============================================================================
// Feature Combination Tests
// ============================================================================

#[test]
fn test_minimal_features() {
    // Test works even with no optional features
    #[cfg(not(feature = "authentication"))]
    {
        let mut shell = helpers::create_test_shell();
        let output = helpers::execute_command(&mut shell, "echo minimal");
        assert!(output.contains("minimal"));
    }
}

// ============================================================================
// Buffer Overflow Handling Tests
// ============================================================================

#[test]
#[cfg(not(feature = "authentication"))]
fn test_buffer_overflow_emits_bell() {
    let mut shell = helpers::create_test_shell();

    // Input buffer size is 128 chars
    let long_input = "a".repeat(128);
    for c in long_input.chars() {
        shell.process_char(c).unwrap();
    }

    shell.io_mut().clear_output();

    // Try to add one more character - should trigger bell
    shell.process_char('x').unwrap();

    let output = shell.io_mut().output();
    assert!(
        output.contains('\x07'),
        "Should emit bell character on buffer full"
    );
}

// ============================================================================
// Command Argument Validation Tests
// ============================================================================

#[test]
#[cfg(not(feature = "authentication"))]
fn test_argument_validation() {
    // Table-driven test for argument validation scenarios
    let test_cases = [
        // (command, expected_error, description)
        (
            "system/hardware/led",
            "Expected 1 arguments, got 0",
            "command requires exactly 1 arg but got 0",
        ),
        (
            "system/hardware/led on",
            "LED: on",
            "command with correct argument count",
        ),
        (
            "system/reboot now",
            "Expected 0 arguments, got 1",
            "command accepts 0 args but got 1",
        ),
    ];

    for (cmd, expected, description) in test_cases {
        let mut shell = helpers::create_test_shell();
        let output = helpers::execute_command(&mut shell, cmd);

        assert!(
            output.contains(expected),
            "Failed '{}': expected '{}' in output: {}",
            description,
            expected,
            output
        );
    }
}

// ============================================================================
// Access Level Enforcement Tests
// ============================================================================
//
// NOTE: Access control works by hiding inaccessible nodes (security feature).
// When a user tries to access a command/directory they don't have permission for,
// the system returns "Command not found" rather than "access denied" to prevent
// information disclosure about the system structure.

#[test]
#[cfg(feature = "authentication")]
fn test_guest_access_control() {
    let mut shell = helpers::create_auth_shell();

    // Login as guest
    helpers::execute_command_auth(&mut shell, "guest:guest123");

    let test_cases = [
        ("echo hello", "hello", true), // Guest can execute Guest-level commands
        ("system/reboot", "Command not found", false), // Guest cannot execute Admin commands
        ("debug/memory", "Command not found", false),  // Guest cannot access Admin directories
    ];

    for (cmd, expected, should_succeed) in test_cases {
        let output = helpers::execute_command_auth(&mut shell, cmd);

        if should_succeed {
            assert!(
                output.contains(expected),
                "Guest should be able to: '{}', got: {}",
                cmd,
                output
            );
        } else {
            assert!(
                output.contains(expected) || output.contains("Invalid path"),
                "Guest should not access: '{}', got: {}",
                cmd,
                output
            );
        }
    }
}

#[test]
#[cfg(feature = "authentication")]
fn test_admin_access_control() {
    let mut shell = helpers::create_auth_shell();

    // Login as admin
    helpers::execute_command_auth(&mut shell, "admin:admin123");

    let test_cases = [
        ("system/reboot", "Reboot"),      // Admin can execute Admin commands
        ("debug/memory", "Memory"),       // Admin can access Admin directories
        ("echo test", "test"),            // Admin can also execute Guest-level commands
    ];

    for (cmd, expected) in test_cases {
        let output = helpers::execute_command_auth(&mut shell, cmd);
        assert!(
            output.contains(expected),
            "Admin should access '{}', got: {}",
            cmd,
            output
        );
    }
}

// ============================================================================
// Async Command Execution Tests
// ============================================================================

#[tokio::test]
#[cfg(all(feature = "async", not(feature = "authentication")))]
async fn test_async_command_execution() {
    let io = MockIo::new();
    let handler = MockHandler;
    let mut shell = Shell::new(&TEST_TREE, handler, io);
    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // Navigate to system directory where async-wait is located
    for c in "system\n".chars() {
        shell.process_char_async(c).await.unwrap();
    }

    // Execute async-wait command
    for c in "async-wait\n".chars() {
        shell.process_char_async(c).await.unwrap();
    }

    let output = shell.io_mut().output();
    assert!(
        output.contains("Waited 100ms"),
        "Async command should execute. Output: {}",
        output
    );
}

#[tokio::test]
#[cfg(all(feature = "async", not(feature = "authentication")))]
async fn test_async_command_with_arguments() {
    let io = MockIo::new();
    let handler = MockHandler;
    let mut shell = Shell::new(&TEST_TREE, handler, io);
    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // Navigate to system and execute async-wait with custom duration
    for c in "system\n".chars() {
        shell.process_char_async(c).await.unwrap();
    }
    shell.io_mut().clear_output();

    for c in "async-wait 250\n".chars() {
        shell.process_char_async(c).await.unwrap();
    }

    let output = shell.io_mut().output();
    assert!(output.contains("Waited 250ms"));
}
