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
use fixtures::{MockHandler, MockIo, TEST_TREE};
use nut_shell::Shell;

// ============================================================================
// Command Execution Tests
// ============================================================================

#[test]
#[cfg(not(feature = "authentication"))]
fn test_command_with_arguments() {
    let io = MockIo::new();
    let handler = MockHandler;
    let mut shell = Shell::new(&TEST_TREE, handler, io);
    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // Execute echo with multiple arguments
    for c in "echo arg1 arg2 arg3\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.io_mut().output();
    assert!(
        output.contains("arg1") && output.contains("arg2") && output.contains("arg3"),
        "Should include all arguments"
    );
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_execute_command_with_path() {
    let io = MockIo::new();
    let handler = MockHandler;
    let mut shell = Shell::new(&TEST_TREE, handler, io);
    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // Execute command using path without navigation (from root)
    for c in "system/status\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.io_mut().output();
    assert!(
        output.contains("System OK"),
        "Should execute command via path without navigation: {}",
        output
    );
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_execute_nested_command_with_path() {
    let io = MockIo::new();
    let handler = MockHandler;
    let mut shell = Shell::new(&TEST_TREE, handler, io);
    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // Execute deeply nested command with full path
    for c in "system/network/status\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.io_mut().output();
    assert!(
        output.contains("Network OK"),
        "Should execute nested command via full path: {}",
        output
    );
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_execute_command_with_path_and_args() {
    let io = MockIo::new();
    let handler = MockHandler;
    let mut shell = Shell::new(&TEST_TREE, handler, io);
    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // Execute command with path and arguments
    for c in "system/hardware/led on\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.io_mut().output();
    assert!(
        output.contains("LED: on"),
        "Should execute command with path and arguments: {}",
        output
    );
}

// ============================================================================
// Directory Navigation Tests
// ============================================================================

#[test]
#[cfg(not(feature = "authentication"))]
fn test_navigate_to_directory() {
    let io = MockIo::new();
    let handler = MockHandler;
    let mut shell = Shell::new(&TEST_TREE, handler, io);
    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // Navigate to system directory
    for c in "system\n".chars() {
        shell.process_char(c).unwrap();
    }

    shell.io_mut().clear_output();

    // Now we should be in system/, so 'status' command should be accessible
    for c in "status\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.io_mut().output();
    assert!(
        output.contains("@/system>") && output.contains("System OK"),
        "Should be able to execute commands in navigated directory: {}",
        output
    );
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_navigate_to_nested_directory() {
    let io = MockIo::new();
    let handler = MockHandler;
    let mut shell = Shell::new(&TEST_TREE, handler, io);
    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // Navigate to system/network directory
    for c in "system\n".chars() {
        shell.process_char(c).unwrap();
    }
    for c in "network\n".chars() {
        shell.process_char(c).unwrap();
    }

    shell.io_mut().clear_output();

    // Now we should be in system/network/, test network-specific command
    for c in "status\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.io_mut().output();
    assert!(
        output.contains("@/system/network>") && output.contains("Network OK"),
        "Should execute network status in system/network/: {}",
        output
    );
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_navigate_with_relative_path() {
    let io = MockIo::new();
    let handler = MockHandler;
    let mut shell = Shell::new(&TEST_TREE, handler, io);
    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // Navigate using relative path system/network
    for c in "system/network\n".chars() {
        shell.process_char(c).unwrap();
    }

    shell.io_mut().clear_output();

    // Execute command in current directory
    for c in "status\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.io_mut().output();
    assert!(
        output.contains("@/system/network>") && output.contains("Network OK"),
        "Should navigate using relative path: {}",
        output
    );
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_navigate_parent_directory() {
    let io = MockIo::new();
    let handler = MockHandler;
    let mut shell = Shell::new(&TEST_TREE, handler, io);
    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // Navigate to system/network
    for c in "system/network\n".chars() {
        shell.process_char(c).unwrap();
    }

    // Navigate up one level using ..
    for c in "..\n".chars() {
        shell.process_char(c).unwrap();
    }

    shell.io_mut().clear_output();

    // Should be in system/ now, test system command
    for c in "status\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.io_mut().output();
    assert!(
        output.contains("@/system>") && output.contains("System OK"),
        "Should navigate to parent with ..: {}",
        output
    );
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_navigate_parent_multiple_levels() {
    let io = MockIo::new();
    let handler = MockHandler;
    let mut shell = Shell::new(&TEST_TREE, handler, io);
    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // Navigate to system/network
    for c in "system/network\n".chars() {
        shell.process_char(c).unwrap();
    }

    // Navigate up two levels using ../..
    for c in "../..\n".chars() {
        shell.process_char(c).unwrap();
    }

    shell.io_mut().clear_output();

    // Should be at root, test root command
    for c in "echo back at root\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.io_mut().output();
    assert!(
        output.contains("@/>") && output.contains("back at root"),
        "Should navigate multiple parent levels: {}",
        output
    );
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_navigate_with_current_directory_dot() {
    let io = MockIo::new();
    let handler = MockHandler;
    let mut shell = Shell::new(&TEST_TREE, handler, io);
    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // Navigate using . (current directory) in path
    for c in "./system\n".chars() {
        shell.process_char(c).unwrap();
    }

    shell.io_mut().clear_output();

    for c in "status\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.io_mut().output();
    assert!(
        output.contains("@/system>") && output.contains("System OK"),
        "Should handle . in path: {}",
        output
    );
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_navigate_absolute_path() {
    let io = MockIo::new();
    let handler = MockHandler;
    let mut shell = Shell::new(&TEST_TREE, handler, io);
    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // Navigate to system first
    for c in "system\n".chars() {
        shell.process_char(c).unwrap();
    }

    // Navigate to debug using absolute path from system directory
    for c in "/debug\n".chars() {
        shell.process_char(c).unwrap();
    }

    shell.io_mut().clear_output();

    // Should be in debug/, test debug command
    for c in "memory\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.io_mut().output();
    assert!(
        output.contains("@/debug>") && output.contains("Memory"),
        "Should navigate using absolute path: {}",
        output
    );
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_navigate_to_root_with_slash() {
    let io = MockIo::new();
    let handler = MockHandler;
    let mut shell = Shell::new(&TEST_TREE, handler, io);
    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // Navigate to system/network
    for c in "system/network\n".chars() {
        shell.process_char(c).unwrap();
    }

    // Navigate to root using /
    for c in "/\n".chars() {
        shell.process_char(c).unwrap();
    }

    shell.io_mut().clear_output();

    // Should be at root, test root command
    for c in "echo at root\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.io_mut().output();
    assert!(
        output.contains("@/>") && output.contains("at root"),
        "Should navigate to root with /: {}",
        output
    );
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_navigate_invalid_directory() {
    let io = MockIo::new();
    let handler = MockHandler;
    let mut shell = Shell::new(&TEST_TREE, handler, io);
    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // Try to navigate to non-existent directory
    for c in "nonexistent\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.io_mut().output();
    assert!(
        output.contains("Error: Command not found"),
        "Should error on invalid directory: {}",
        output
    );
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_navigate_parent_beyond_root() {
    let io = MockIo::new();
    let handler = MockHandler;
    let mut shell = Shell::new(&TEST_TREE, handler, io);
    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // Try to navigate above root
    for c in "..\n".chars() {
        shell.process_char(c).unwrap();
    }

    shell.io_mut().clear_output();

    // Should still be at root (.. from root stays at root)
    for c in "echo still at root\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.io_mut().output();
    assert!(
        output.contains("@/>") && output.contains("still at root"),
        "Should stay at root when navigating .. from root: {}",
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
    let handler = MockHandler;
    let mut shell = Shell::new(&TEST_TREE, handler, io);
    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // Execute help command
    for c in "?\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.io_mut().output();
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
    let handler = MockHandler;
    let mut shell = Shell::new(&TEST_TREE, handler, io);
    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // List root directory
    for c in "ls\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.io_mut().output();
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
    let handler = MockHandler;
    let mut shell = Shell::new(&TEST_TREE, handler, io);
    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // Execute clear
    for c in "clear\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.io_mut().output();
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
    let handler = MockHandler;
    let mut shell = Shell::new(&TEST_TREE, handler, io);
    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // Type partial command
    for c in "ech".chars() {
        shell.process_char(c).unwrap();
    }

    shell.io_mut().clear_output();

    // Press tab - should emit "o" to complete "echo"
    shell.process_char('\t').unwrap();

    let completion_output = shell.io_mut().output();
    assert!(
        completion_output.contains('o'),
        "Tab should have emitted 'o' to complete 'ech' to 'echo': {}",
        completion_output
    );

    // Now execute with an argument
    for c in " completion_test\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.io_mut().output();
    assert!(
        output.contains("completion_test"),
        "Completed command should execute correctly: {}",
        output
    );
}

// ============================================================================
// History Navigation Tests (requires history feature)
// ============================================================================

#[test]
#[cfg(all(feature = "history", not(feature = "authentication")))]
fn test_history_navigation_up() {
    let io = MockIo::new();
    let handler = MockHandler;
    let mut shell = Shell::new(&TEST_TREE, handler, io);
    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // Execute a command
    for c in "echo first\n".chars() {
        shell.process_char(c).unwrap();
    }
    shell.io_mut().clear_output();

    // Execute another command
    for c in "echo second\n".chars() {
        shell.process_char(c).unwrap();
    }
    shell.io_mut().clear_output();

    // Press up arrow (should recall "echo second")
    shell.process_char('\x1b').unwrap(); // ESC
    shell.process_char('[').unwrap();
    shell.process_char('A').unwrap(); // Up

    let output = shell.io_mut().output();
    assert!(
        output.contains("echo second"),
        "Up arrow should recall 'echo second': {}",
        output
    );

    // Execute and verify
    shell.io_mut().clear_output();
    shell.process_char('\n').unwrap();

    let output = shell.io_mut().output();
    assert!(
        output.contains("second"),
        "Should execute 'echo second': {}",
        output
    );
}

#[test]
#[cfg(all(feature = "history", not(feature = "authentication")))]
fn test_history_navigation_up_multiple() {
    let io = MockIo::new();
    let handler = MockHandler;
    let mut shell = Shell::new(&TEST_TREE, handler, io);
    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // Execute two commands
    for c in "echo first\n".chars() {
        shell.process_char(c).unwrap();
    }
    for c in "echo second\n".chars() {
        shell.process_char(c).unwrap();
    }
    shell.io_mut().clear_output();

    // Press up arrow once - should recall "echo second"
    shell.process_char('\x1b').unwrap();
    shell.process_char('[').unwrap();
    shell.process_char('A').unwrap();

    let output = shell.io_mut().output();
    assert!(
        output.contains("echo second"),
        "First up arrow should recall 'echo second': {}",
        output
    );

    shell.io_mut().clear_output();

    // Press up arrow again - should recall "echo first"
    shell.process_char('\x1b').unwrap();
    shell.process_char('[').unwrap();
    shell.process_char('A').unwrap();

    let output = shell.io_mut().output();
    assert!(
        output.contains("echo first"),
        "Second up arrow should recall 'echo first': {}",
        output
    );

    // Execute to verify the buffer contains "echo first"
    shell.io_mut().clear_output();
    shell.process_char('\n').unwrap();

    let output = shell.io_mut().output();
    assert!(
        output.contains("first"),
        "Should execute 'echo first': {}",
        output
    );
}

#[test]
#[cfg(all(feature = "history", not(feature = "authentication")))]
fn test_history_navigation_down() {
    let io = MockIo::new();
    let handler = MockHandler;
    let mut shell = Shell::new(&TEST_TREE, handler, io);
    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // Execute three commands to have enough history
    for c in "echo first\n".chars() {
        shell.process_char(c).unwrap();
    }
    for c in "echo second\n".chars() {
        shell.process_char(c).unwrap();
    }
    for c in "echo third\n".chars() {
        shell.process_char(c).unwrap();
    }
    shell.io_mut().clear_output();

    // Navigate up twice to get to "echo second"
    shell.process_char('\x1b').unwrap();
    shell.process_char('[').unwrap();
    shell.process_char('A').unwrap(); // Up to "echo third"

    shell.io_mut().clear_output();

    shell.process_char('\x1b').unwrap();
    shell.process_char('[').unwrap();
    shell.process_char('A').unwrap(); // Up to "echo second"

    let output = shell.io_mut().output();
    assert!(
        output.contains("echo second"),
        "Should be at 'echo second': {}",
        output
    );

    shell.io_mut().clear_output();

    // Press down arrow - should move forward to "echo third"
    shell.process_char('\x1b').unwrap();
    shell.process_char('[').unwrap();
    shell.process_char('B').unwrap();

    let output = shell.io_mut().output();
    assert!(
        output.contains("echo third"),
        "Down arrow should recall 'echo third': {}",
        output
    );

    // Execute to verify
    shell.io_mut().clear_output();
    shell.process_char('\n').unwrap();

    let output = shell.io_mut().output();
    assert!(
        output.contains("third"),
        "Should execute 'echo third': {}",
        output
    );
}

// ============================================================================
// Double-ESC Clear Tests
// ============================================================================

#[test]
#[cfg(not(feature = "authentication"))]
fn test_double_esc_clears_buffer() {
    let io = MockIo::new();
    let handler = MockHandler;
    let mut shell = Shell::new(&TEST_TREE, handler, io);
    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // Type some input but don't execute
    for c in "echo test".chars() {
        shell.process_char(c).unwrap();
    }

    // Verify input was echoed before clearing
    let output_before = shell.io_mut().output();
    assert!(
        output_before.contains("echo test"),
        "Input should be echoed before clearing: {}",
        output_before
    );

    shell.io_mut().clear_output();

    // Double-ESC should clear the buffer
    shell.process_char('\x1b').unwrap();
    shell.process_char('\x1b').unwrap();

    let clear_output = shell.io_mut().output();

    // Should send clear sequence: \r (CR) + \x1b[K (clear to end of line) + prompt
    assert!(
        clear_output.contains("\r"),
        "Should send carriage return after double-ESC: {:?}",
        clear_output
    );
    assert!(
        clear_output.contains("\x1b[K"),
        "Should send clear-to-EOL sequence after double-ESC: {:?}",
        clear_output
    );

    // Now press enter - nothing should execute since buffer was cleared
    shell.io_mut().clear_output();
    shell.process_char('\n').unwrap();

    // Output should not contain "test" (command was cleared)
    let output = shell.io_mut().output();
    assert!(
        !output.contains("test"),
        "Double-ESC should have cleared buffer, got: {}",
        output
    );
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
#[cfg(not(feature = "authentication"))]
fn test_backspace_handling() {
    let io = MockIo::new();
    let handler = MockHandler;
    let mut shell = Shell::new(&TEST_TREE, handler, io);
    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // Type with backspaces
    for c in "echox".chars() {
        shell.process_char(c).unwrap();
    }
    shell.process_char('\x08').unwrap(); // Backspace
    shell.process_char(' ').unwrap();
    for c in "test\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.io_mut().output();
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
        let handler = MockHandler;
        let mut shell = Shell::new(&TEST_TREE, handler, io);
        shell.activate().unwrap();
        shell.io_mut().clear_output();

        // Basic command execution should always work
        for c in "echo minimal\n".chars() {
            shell.process_char(c).unwrap();
        }

        let output = shell.io_mut().output();
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
    let handler = MockHandler;
    let mut shell = Shell::new(&TEST_TREE, handler, io);
    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // Input buffer size is 128 chars (hardcoded in Shell for now)
    // Fill it up completely
    let long_input = "a".repeat(128);
    for c in long_input.chars() {
        shell.process_char(c).unwrap();
    }

    shell.io_mut().clear_output();

    // Try to add one more character - should trigger bell
    shell.process_char('x').unwrap(); // Should succeed (returns Ok)

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
fn test_command_requires_exact_args() {
    let io = MockIo::new();
    let handler = MockHandler;
    let mut shell = Shell::new(&TEST_TREE, handler, io);
    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // 'led' command requires exactly 1 argument (min_args=1, max_args=1)
    // Test with no arguments - should fail
    for c in "system/hardware/led\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.io_mut().output();
    assert!(
        output.contains("Error: Expected 1 arguments, got 0"),
        "Should report missing argument: {}",
        output
    );
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_command_accepts_valid_args() {
    let io = MockIo::new();
    let handler = MockHandler;
    let mut shell = Shell::new(&TEST_TREE, handler, io);
    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // 'led' command with correct argument
    for c in "system/hardware/led on\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.io_mut().output();
    assert!(
        output.contains("LED: on"),
        "Should execute command with valid args: {}",
        output
    );
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_command_too_many_args() {
    let io = MockIo::new();
    let handler = MockHandler;
    let mut shell = Shell::new(&TEST_TREE, handler, io);
    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // 'reboot' accepts 0 arguments (max_args=0)
    // Provide arguments - should fail
    for c in "system/reboot now\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.io_mut().output();
    assert!(
        output.contains("Error: Expected 0 arguments, got 1"),
        "Should report too many arguments: {}",
        output
    );
}

// ============================================================================
// Access Level Enforcement Tests
// ============================================================================
//
// NOTE: Access control works by hiding inaccessible nodes (security feature).
// When a user tries to access a command/directory they don't have permission for,
// the system returns "Command not found" rather than "access denied" to prevent
// information disclosure about the system structure.

// Helper function to create test credential provider
#[cfg(feature = "authentication")]
fn create_test_provider() -> (
    nut_shell::auth::ConstCredentialProvider<
        fixtures::MockAccessLevel,
        nut_shell::auth::password::Sha256Hasher,
        2,
    >,
    nut_shell::auth::password::Sha256Hasher,
) {
    use fixtures::MockAccessLevel;
    use nut_shell::auth::{ConstCredentialProvider, PasswordHasher, User, password::Sha256Hasher};

    let hasher = Sha256Hasher::new();

    let guest_salt = [1u8; 16];
    let guest_hash = hasher.hash("guest123", &guest_salt);

    let admin_salt = [2u8; 16];
    let admin_hash = hasher.hash("admin123", &admin_salt);

    let users = [
        User::new("guest", MockAccessLevel::Guest, guest_hash, guest_salt).unwrap(),
        User::new("admin", MockAccessLevel::Admin, admin_hash, admin_salt).unwrap(),
    ];

    (ConstCredentialProvider::new(users, hasher), hasher)
}

#[test]
#[cfg(feature = "authentication")]
fn test_guest_can_execute_guest_level_commands() {
    let (provider, _hasher) = create_test_provider();
    let io = MockIo::new();
    let handler = MockHandler;
    let mut shell = Shell::new(&TEST_TREE, handler, &provider, io);

    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // Login as guest
    for c in "guest:guest123\n".chars() {
        shell.process_char(c).unwrap();
    }
    shell.io_mut().clear_output();

    // Guest should be able to execute Guest-level commands
    for c in "echo hello\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.io_mut().output();
    assert!(
        output.contains("hello"),
        "Guest should be able to execute Guest-level commands: {}",
        output
    );
}

#[test]
#[cfg(feature = "authentication")]
fn test_guest_cannot_execute_admin_commands() {
    let (provider, _hasher) = create_test_provider();
    let io = MockIo::new();
    let handler = MockHandler;
    let mut shell = Shell::new(&TEST_TREE, handler, &provider, io);

    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // Login as guest
    for c in "guest:guest123\n".chars() {
        shell.process_char(c).unwrap();
    }
    shell.io_mut().clear_output();

    // Guest should NOT be able to execute Admin commands
    for c in "system/reboot\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.io_mut().output();
    assert!(
        output.contains("Command not found") || output.contains("Invalid path"),
        "Guest should not be able to execute Admin commands: {}",
        output
    );
}

#[test]
#[cfg(feature = "authentication")]
fn test_guest_cannot_access_admin_directories() {
    let (provider, _hasher) = create_test_provider();
    let io = MockIo::new();
    let handler = MockHandler;
    let mut shell = Shell::new(&TEST_TREE, handler, &provider, io);

    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // Login as guest
    for c in "guest:guest123\n".chars() {
        shell.process_char(c).unwrap();
    }
    shell.io_mut().clear_output();

    // Guest should NOT be able to access Admin directories
    for c in "debug/memory\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.io_mut().output();
    assert!(
        output.contains("Command not found") || output.contains("Invalid path"),
        "Guest should not be able to access Admin directories: {}",
        output
    );
}

#[test]
#[cfg(feature = "authentication")]
fn test_admin_can_execute_admin_commands() {
    let (provider, _hasher) = create_test_provider();
    let io = MockIo::new();
    let handler = MockHandler;
    let mut shell = Shell::new(&TEST_TREE, handler, &provider, io);

    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // Login as admin
    for c in "admin:admin123\n".chars() {
        shell.process_char(c).unwrap();
    }
    shell.io_mut().clear_output();

    // Admin should be able to execute Admin commands
    for c in "system/reboot\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.io_mut().output();
    assert!(
        output.contains("Reboot"),
        "Admin should be able to execute Admin commands: {}",
        output
    );
}

#[test]
#[cfg(feature = "authentication")]
fn test_admin_can_access_admin_directories() {
    let (provider, _hasher) = create_test_provider();
    let io = MockIo::new();
    let handler = MockHandler;
    let mut shell = Shell::new(&TEST_TREE, handler, &provider, io);

    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // Login as admin
    for c in "admin:admin123\n".chars() {
        shell.process_char(c).unwrap();
    }
    shell.io_mut().clear_output();

    // Admin should be able to access Admin directories
    for c in "debug/memory\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.io_mut().output();
    assert!(
        output.contains("Memory"),
        "Admin should be able to access Admin directories: {}",
        output
    );
}

// ============================================================================
// Async Command Execution Tests
// ============================================================================

#[tokio::test]
#[cfg(all(feature = "async", not(feature = "authentication")))]
async fn test_async_command_via_process_char_async() {
    // Test that process_char_async can execute async commands end-to-end
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
        "Async command should have executed. Output: {}",
        output
    );
}

#[tokio::test]
#[cfg(all(feature = "async", not(feature = "authentication")))]
async fn test_sync_command_in_async_context() {
    // Test that sync commands work fine in process_char_async
    let io = MockIo::new();
    let handler = MockHandler;
    let mut shell = Shell::new(&TEST_TREE, handler, io);
    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // Execute sync command via async process_char
    for c in "echo hello\n".chars() {
        shell.process_char_async(c).await.unwrap();
    }

    let output = shell.io_mut().output();
    assert!(
        output.contains("hello"),
        "Sync command should work in async context"
    );
}

#[tokio::test]
#[cfg(all(feature = "async", not(feature = "authentication")))]
async fn test_async_command_with_arguments() {
    // Test async command with custom arguments
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
    assert!(
        output.contains("Waited 250ms"),
        "Async command with args should execute"
    );
}
