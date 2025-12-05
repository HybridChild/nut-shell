//! Input editing and terminal behavior tests.
//!
//! Tests backspace, buffer management, double-ESC, prompt formatting,
//! ANSI sequences, and other terminal UI behaviors.

#[allow(clippy::duplicate_mod)]
#[path = "fixtures/mod.rs"]
mod fixtures;

#[allow(clippy::duplicate_mod)]
#[path = "helpers.rs"]
mod helpers;

// ============================================================================
// Input Editing Tests (documents interactive editing behavior)
// ============================================================================

#[test]
#[cfg(not(feature = "authentication"))]
fn test_empty_command_does_nothing() {
    let mut shell = helpers::create_test_shell();

    // Press enter with no input - should just show new prompt
    shell.io_mut().clear_output();
    helpers::press_enter(&mut shell);

    let output = shell.io_mut().output();
    helpers::assert_prompt(&output, "@/>");
    helpers::assert_contains_none(&output, &["Error", "Command"]);
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_backspace_sequence() {
    // Documents backspace behavior: removes last char and emits backspace sequence
    let mut shell = helpers::create_test_shell();

    helpers::type_input(&mut shell, "test");
    shell.io_mut().clear_output();

    helpers::press_backspace(&mut shell);

    let output = shell.io_mut().output();
    helpers::assert_contains_ansi(&output, "\x08"); // Backspace
    helpers::assert_contains_ansi(&output, " "); // Space
    helpers::assert_contains_ansi(&output, "\x08"); // Backspace again (VT100 sequence)
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_backspace_until_empty() {
    let mut shell = helpers::create_test_shell();

    helpers::type_input(&mut shell, "test");
    helpers::press_backspace_n(&mut shell, 4);

    // Execute empty buffer - should do nothing
    shell.io_mut().clear_output();
    helpers::press_enter(&mut shell);

    let output = shell.io_mut().output();
    helpers::assert_prompt(&output, "@/>");
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

#[test]
#[cfg(not(feature = "authentication"))]
fn test_double_esc_clears_and_shows_prompt() {
    // Documents double-ESC behavior: clears buffer, sends CR + clear-to-EOL, re-shows prompt
    let mut shell = helpers::create_test_shell();

    helpers::type_input(&mut shell, "some command");
    shell.io_mut().clear_output();

    helpers::press_double_esc(&mut shell);

    let output = shell.io_mut().output();
    helpers::assert_contains_ansi(&output, "\r"); // Carriage return
    helpers::assert_contains_ansi(&output, "\x1b[K"); // Clear to end of line
    helpers::assert_prompt(&output, "@/>");
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

#[test]
#[cfg(not(feature = "authentication"))]
fn test_bell_on_buffer_overflow() {
    // Documents bell (^G) emission when buffer is full
    let mut shell = helpers::create_test_shell();

    // Fill buffer
    helpers::type_input(&mut shell, &"a".repeat(128));
    shell.io_mut().clear_output();

    // Try to add more - should emit bell
    helpers::type_input(&mut shell, "x");

    let output = shell.io_mut().output();
    assert_eq!(
        helpers::count_char(&output, '\x07'),
        1,
        "Should emit exactly one bell character"
    );
}

// ============================================================================
// Terminal Behavior Documentation Tests
// ============================================================================

#[test]
#[cfg(not(feature = "authentication"))]
fn test_prompt_format() {
    // Documents prompt format: user@path>
    let mut shell = helpers::create_test_shell();

    shell.io_mut().clear_output();
    helpers::press_enter(&mut shell);

    let output = shell.io_mut().output();
    helpers::assert_prompt(&output, "@/>");
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_prompt_updates_with_navigation() {
    // Prompt should reflect current directory
    let mut shell = helpers::create_test_shell();

    helpers::execute_command(&mut shell, "system");

    shell.io_mut().clear_output();
    helpers::press_enter(&mut shell);

    let output = shell.io_mut().output();
    helpers::assert_prompt(&output, "@/system>");
}
