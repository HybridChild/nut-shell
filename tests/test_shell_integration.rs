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
use nut_shell::config::DefaultConfig;
use nut_shell::error::CliError;
use nut_shell::response::Response;
use nut_shell::shell::Request;

// ============================================================================
// Request/Response Workflow Tests
// ============================================================================

#[test]
fn test_request_response_workflow() {
    // Simulate command execution workflow
    let mut path = heapless::String::<128>::new();
    path.push_str("status").unwrap();
    #[cfg(feature = "history")]
    let original = {
        let mut s = heapless::String::<128>::new();
        s.push_str("status").unwrap();
        s
    };

    let request = Request::<DefaultConfig>::Command {
        path,
        args: heapless::Vec::new(),
        #[cfg(feature = "history")]
        original,
        _phantom: core::marker::PhantomData,
    };

    // Extract command info
    let result: Result<Response<DefaultConfig>, CliError> = match request {
        Request::Command { path, .. } => {
            if path.as_str() == "status" {
                Ok(Response::<DefaultConfig>::success("System OK"))
            } else {
                let mut msg = heapless::String::new();
                msg.push_str("Unknown command").unwrap();
                Err(CliError::CommandFailed(msg))
            }
        }
        #[allow(unreachable_patterns)]
        _ => {
            let mut msg = heapless::String::new();
            msg.push_str("Invalid request").unwrap();
            Err(CliError::CommandFailed(msg))
        }
    };

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.message.as_str(), "System OK");
}

// ============================================================================
// Command Execution Tests
// ============================================================================

#[test]
#[cfg(not(feature = "authentication"))]
fn test_simple_command_execution() {
    let io = MockIo::new();
    let handler = MockHandler;
    let mut shell = Shell::new(&TEST_TREE, handler, io);
    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // Execute 'echo hello world'
    for c in "echo hello world\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.io_mut().output();
    assert!(
        output.contains("hello world"),
        "Output should contain echo result"
    );
}

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

    // Type partial command, press tab to complete, then execute
    for c in "ech".chars() {
        shell.process_char(c).unwrap();
    }
    shell.process_char('\t').unwrap(); // Tab completes to "echo"

    // Clear output and execute the completed command with an argument
    shell.io_mut().clear_output();
    for c in " completion_test\n".chars() {
        shell.process_char(c).unwrap();
    }

    // Verify the command executed correctly (proves tab completion worked)
    let output = shell.io_mut().output();
    assert!(
        output.contains("completion_test"),
        "Tab should have completed 'ech' to 'echo', output: {}",
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

    // Press up arrow (should recall "echo second") and execute
    shell.process_char('\x1b').unwrap(); // ESC
    shell.process_char('[').unwrap();
    shell.process_char('A').unwrap(); // Up
    shell.io_mut().clear_output();
    shell.process_char('\n').unwrap(); // Execute

    // Verify the recalled command executed
    let output = shell.io_mut().output();
    assert!(
        output.contains("second"),
        "Up arrow should recall and execute 'echo second', got: {}",
        output
    );
}

#[test]
#[cfg(all(feature = "history", not(feature = "authentication")))]
fn test_history_navigation_up_down() {
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

    // Press up arrow twice to get to "echo first"
    shell.process_char('\x1b').unwrap();
    shell.process_char('[').unwrap();
    shell.process_char('A').unwrap(); // Up (should show "echo second")

    shell.process_char('\x1b').unwrap();
    shell.process_char('[').unwrap();
    shell.process_char('A').unwrap(); // Up again (should show "echo first")

    // Execute to verify we got "echo first"
    shell.io_mut().clear_output();
    shell.process_char('\n').unwrap();
    let output = shell.io_mut().output();
    assert!(
        output.contains("first"),
        "Up twice should recall 'echo first': {}",
        output
    );
    shell.io_mut().clear_output();

    // Now test down arrow: up three times then down once should give "echo second"
    // History now contains: "echo first", "echo second", "echo first" (just executed)
    shell.process_char('\x1b').unwrap();
    shell.process_char('[').unwrap();
    shell.process_char('A').unwrap(); // Up (at "echo first" - most recent)

    shell.process_char('\x1b').unwrap();
    shell.process_char('[').unwrap();
    shell.process_char('A').unwrap(); // Up again (at "echo second")

    shell.process_char('\x1b').unwrap();
    shell.process_char('[').unwrap();
    shell.process_char('A').unwrap(); // Up again (at "echo first" - oldest)

    shell.process_char('\x1b').unwrap();
    shell.process_char('[').unwrap();
    shell.process_char('B').unwrap(); // Down (should show "echo second")

    // Execute to verify we got "echo second"
    shell.io_mut().clear_output();
    shell.process_char('\n').unwrap();
    let output = shell.io_mut().output();
    assert!(
        output.contains("second"),
        "Down should recall 'echo second': {}",
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

    // Double-ESC should clear the buffer
    shell.process_char('\x1b').unwrap();
    shell.process_char('\x1b').unwrap();

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
        output.contains("argument") || output.contains("require") || output.contains("Usage"),
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
        output.contains("LED") && output.contains("on"),
        "Should execute command with valid args: {}",
        output
    );
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_command_with_variable_args() {
    let io = MockIo::new();
    let handler = MockHandler;
    let mut shell = Shell::new(&TEST_TREE, handler, io);
    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // 'echo' allows 0-16 arguments
    for c in "echo a b c d e\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.io_mut().output();
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
        output.contains("argument") || output.contains("too many") || output.contains("Usage"),
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
// Config Variation Tests
// ============================================================================

#[test]
#[cfg(not(feature = "authentication"))]
fn test_minimal_config_works() {
    use nut_shell::Response;
    use nut_shell::config::MinimalConfig;
    use nut_shell::error::CliError;
    use nut_shell::shell::handler::CommandHandler;

    // Implement handler for MinimalConfig
    struct MinimalHandler;

    impl CommandHandler<MinimalConfig> for MinimalHandler {
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
    let handler = MinimalHandler;
    let mut shell: Shell<_, _, _, MinimalConfig> = Shell::new(&TEST_TREE, handler, io);

    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // Execute command with MinimalConfig
    for c in "echo test\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.io_mut().output();
    assert!(output.contains("Echo"), "MinimalConfig should work");
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
        output.contains("Async complete") || output.contains("100ms"),
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
        output.contains("Async complete") || output.contains("250ms"),
        "Async command with args should execute"
    );
}

#[test]
#[cfg(all(not(feature = "async"), feature = "async"))]
fn test_async_command_errors_without_async_feature() {
    // This test ensures that trying to call an async command from sync context fails
    // Note: This test should never actually run because the feature combination is impossible
    // It's here for documentation purposes
    compile_error!("This should not compile - contradictory feature requirements");
}

// ============================================================================
// Path Navigation Edge Cases
// ============================================================================
