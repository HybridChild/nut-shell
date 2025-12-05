//! Optional feature tests (completion, history, async).
//!
//! Tests tab completion, command history navigation, and async command execution.
//! These features are optional and can be disabled at compile time.

#[allow(clippy::duplicate_mod)]
#[path = "helpers.rs"]
mod helpers;

// Imports used in feature-gated tests
#[allow(unused_imports)]
use helpers::fixtures::{MockAccessLevel, MockHandler, MockIo, TEST_TREE};
#[allow(unused_imports)]
use nut_shell::Shell;

// ============================================================================
// Tab Completion Tests (requires completion feature)
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

#[test]
#[cfg(all(feature = "completion", not(feature = "authentication")))]
fn test_tab_with_no_input() {
    // Tab on empty input should show all available options
    let mut shell = helpers::create_test_shell();

    shell.io_mut().clear_output();
    helpers::press_tab(&mut shell);

    let output = shell.io_mut().output();
    // Should show multiple matches (all root commands/dirs)
    helpers::assert_contains_all(&output, &["echo", "system"]);
}

#[test]
#[cfg(all(feature = "completion", not(feature = "authentication")))]
fn test_tab_with_no_matches() {
    let mut shell = helpers::create_test_shell();

    helpers::type_input(&mut shell, "xyz");
    shell.io_mut().clear_output();

    helpers::press_tab(&mut shell);

    let output = shell.io_mut().output();
    // No completion - should emit bell character
    assert!(
        output.contains('\x07'),
        "Should emit bell character when no matches found"
    );
}

#[test]
#[cfg(all(feature = "completion", not(feature = "authentication")))]
fn test_tab_completion_in_subdirectory() {
    // After navigating to a directory, tab should complete from that context
    let mut shell = helpers::create_test_shell();

    helpers::execute_command(&mut shell, "system");

    // Now tab should complete commands in /system
    helpers::type_input(&mut shell, "st");
    shell.io_mut().clear_output();

    helpers::press_tab(&mut shell);

    let output = shell.io_mut().output();
    assert!(
        output.contains("atus") || output.contains("status"),
        "Should complete 'status' from 'st' in /system: {}",
        output
    );
}

#[test]
#[cfg(all(feature = "completion", not(feature = "authentication")))]
fn test_tab_completes_directory_with_slash() {
    // When tab completes a directory, it should append "/"
    let mut shell = helpers::create_test_shell();

    helpers::type_input(&mut shell, "syst");
    shell.io_mut().clear_output();

    helpers::press_tab(&mut shell);

    let output = shell.io_mut().output();
    assert!(
        output.contains("em/"),
        "Directory completion should append '/': {}",
        output
    );
}

// ============================================================================
// History Navigation Tests (requires history feature)
// ============================================================================

/// Helper to create a shell with populated history.
#[cfg(all(feature = "history", not(feature = "authentication")))]
fn setup_shell_with_history(
    commands: &[&str],
) -> Shell<'static, MockAccessLevel, MockIo, MockHandler, nut_shell::config::DefaultConfig> {
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

#[test]
#[cfg(all(feature = "history", not(feature = "authentication")))]
fn test_history_empty_buffer() {
    // Up arrow on fresh shell with no history - should do nothing
    let mut shell = helpers::create_test_shell();

    shell.io_mut().clear_output();
    helpers::press_up_arrow(&mut shell);

    let output = shell.io_mut().output();
    assert!(
        output.is_empty() || output.trim().is_empty(),
        "Up arrow on empty history should do nothing"
    );
}

#[test]
#[cfg(all(feature = "history", not(feature = "authentication")))]
fn test_history_up_at_oldest() {
    // Up arrow when at oldest command should stay at oldest
    let mut shell = helpers::create_test_shell();

    helpers::execute_command(&mut shell, "echo first");
    helpers::execute_command(&mut shell, "echo second");

    // Go up twice to reach oldest
    helpers::press_up_arrow(&mut shell);
    helpers::press_up_arrow(&mut shell);

    shell.io_mut().clear_output();

    // Press up again - should stay at "echo first"
    helpers::press_up_arrow(&mut shell);

    let output = shell.io_mut().output();
    assert!(
        output.contains("echo first"),
        "Should stay at oldest command: {}",
        output
    );
}

#[test]
#[cfg(all(feature = "history", not(feature = "authentication")))]
fn test_history_down_at_newest() {
    // Down arrow at newest position should clear to empty
    let mut shell = helpers::create_test_shell();

    helpers::execute_command(&mut shell, "echo test");

    helpers::press_up_arrow(&mut shell);
    shell.io_mut().clear_output();

    // Press down - should go to empty (beyond newest)
    helpers::press_down_arrow(&mut shell);

    let output = shell.io_mut().output();
    // Should show cleared line
    assert!(
        output.contains("\r") || output.contains("\x1b"),
        "Down at newest should clear buffer: {}",
        output
    );
}

#[test]
#[cfg(all(feature = "history", not(feature = "authentication")))]
fn test_history_after_failed_command() {
    // Failed commands should still be added to history
    let mut shell = helpers::create_test_shell();

    helpers::execute_command(&mut shell, "nonexistent");
    helpers::execute_command(&mut shell, "echo valid");

    shell.io_mut().clear_output();
    helpers::press_up_arrow(&mut shell);

    let output = shell.io_mut().output();
    assert!(
        output.contains("echo valid"),
        "Should recall last command (even after failed command): {}",
        output
    );
}

#[test]
#[cfg(all(feature = "history", not(feature = "authentication")))]
fn test_history_edit_recalled_command() {
    // User should be able to edit a recalled command before executing
    let mut shell = helpers::create_test_shell();

    helpers::execute_command(&mut shell, "echo original");

    // Recall and edit
    helpers::press_up_arrow(&mut shell);
    shell.io_mut().clear_output();

    // Backspace 8 times to remove "original"
    helpers::press_backspace_n(&mut shell, 8);

    // Type new text
    helpers::type_input(&mut shell, "modified");

    // Execute
    shell.io_mut().clear_output();
    helpers::press_enter(&mut shell);

    let output = shell.io_mut().output();
    assert!(
        output.contains("modified"),
        "Should execute edited command: {}",
        output
    );
}

// ============================================================================
// Async Command Execution Tests (requires async feature)
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
