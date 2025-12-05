//! Core shell functionality tests.
//!
//! Tests command execution, directory navigation, path resolution,
//! argument validation, and access control.

#[allow(clippy::duplicate_mod)]
#[path = "fixtures/mod.rs"]
mod fixtures;

#[allow(clippy::duplicate_mod)]
#[path = "helpers.rs"]
mod helpers;

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
        (
            "system/status",
            "System OK",
            "simple path without navigation",
        ),
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

#[test]
#[cfg(not(feature = "authentication"))]
fn test_command_with_leading_spaces() {
    let mut shell = helpers::create_test_shell();

    // Leading spaces should be trimmed
    let output = helpers::execute_command(&mut shell, "   echo test");
    assert!(output.contains("test"), "Leading spaces should be trimmed");
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_command_with_trailing_spaces() {
    let mut shell = helpers::create_test_shell();

    // Trailing spaces should be trimmed
    let output = helpers::execute_command(&mut shell, "echo test   ");
    assert!(output.contains("test"), "Trailing spaces should be trimmed");
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_command_with_multiple_spaces_between_args() {
    let mut shell = helpers::create_test_shell();

    // Multiple spaces between args should be normalized to single space
    let output = helpers::execute_command(&mut shell, "echo arg1    arg2    arg3");
    helpers::assert_contains_all(&output, &["arg1", "arg2", "arg3"]);
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_command_with_exact_buffer_size() {
    // Command exactly at buffer limit should work
    let mut shell = helpers::create_test_shell();

    // Create command that's close to buffer size (128 chars)
    // "echo " + 122 chars of args = 127 chars (leave 1 for null/safety)
    let args = "a".repeat(120);
    let cmd = format!("echo {}", args);

    let output = helpers::execute_command(&mut shell, &cmd);
    assert!(
        output.contains(&args),
        "Should handle command at buffer limit"
    );
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_repeated_command_execution() {
    // Executing same command multiple times should work consistently
    let mut shell = helpers::create_test_shell();

    for i in 0..5 {
        let output = helpers::execute_command(&mut shell, "echo test");
        assert!(
            output.contains("test"),
            "Repeated execution #{} should work",
            i + 1
        );
    }
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_rapid_state_changes() {
    // Rapidly navigating and executing commands should maintain state correctly
    let mut shell = helpers::create_test_shell();

    helpers::execute_command(&mut shell, "system");
    helpers::execute_command(&mut shell, "status");
    helpers::execute_command(&mut shell, "..");
    helpers::execute_command(&mut shell, "debug");
    helpers::execute_command(&mut shell, "memory");
    helpers::execute_command(&mut shell, "/");

    let output = helpers::execute_command(&mut shell, "echo final");
    helpers::assert_contains_all(&output, &["@/>", "final"]);
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

#[test]
#[cfg(not(feature = "authentication"))]
fn test_navigate_to_current_directory() {
    // Using "." should stay in current directory
    let mut shell = helpers::create_test_shell();

    helpers::execute_command(&mut shell, "system");

    let output = helpers::execute_command(&mut shell, ".");

    helpers::assert_prompt(&output, "@/system>");
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_multiple_parent_navigation() {
    // Multiple ".." should navigate up multiple levels
    let mut shell = helpers::create_test_shell();

    helpers::execute_command(&mut shell, "system/network");

    let output = helpers::execute_command(&mut shell, "../..");

    helpers::assert_prompt(&output, "@/>");
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_navigate_with_mixed_dots() {
    // Mix of "." and ".." should work correctly
    let mut shell = helpers::create_test_shell();

    helpers::execute_command(&mut shell, "system");

    let output = helpers::execute_command(&mut shell, "./network/../hardware");

    helpers::assert_prompt(&output, "@/system/hardware>");
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_absolute_path_from_subdirectory() {
    // Absolute paths should work from any directory
    let mut shell = helpers::create_test_shell();

    helpers::execute_command(&mut shell, "system/network");

    let output = helpers::execute_command(&mut shell, "/debug");

    helpers::assert_prompt(&output, "@/debug>");
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_navigate_to_root_explicitly() {
    // "/" should navigate to root
    let mut shell = helpers::create_test_shell();

    helpers::execute_command(&mut shell, "system/network");

    let output = helpers::execute_command(&mut shell, "/");

    helpers::assert_prompt(&output, "@/>");
}

// ============================================================================
// Global Commands Tests
// ============================================================================

#[test]
#[cfg(not(feature = "authentication"))]
fn test_global_commands() {
    // Table-driven test for global commands
    let test_cases = [
        (
            "?",
            &["ls", "clear"] as &[&str],
            "help command shows available commands",
        ),
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
        ("debug/memory", "Command not found", false), // Guest cannot access Admin directories
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
        ("system/reboot", "Reboot"), // Admin can execute Admin commands
        ("debug/memory", "Memory"),  // Admin can access Admin directories
        ("echo test", "test"),       // Admin can also execute Guest-level commands
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
