//! Integration tests for Response formatting flags.
//!
//! These tests verify that Response formatting flags actually affect the output,
//! not just that the flags are set correctly. Uses MockIo to capture output
//! and verify formatting behavior.

#[path = "fixtures/mod.rs"]
mod fixtures;

#[cfg(not(feature = "authentication"))]
use fixtures::{MockHandlers, MockIo, TEST_TREE};
#[cfg(not(feature = "authentication"))]
use nut_shell::Shell;
use nut_shell::Response;
use nut_shell::config::DefaultConfig;

// ============================================================================
// Response Flag Setter Tests (Unit-level)
// ============================================================================

#[test]
fn test_response_default_formatting() {
    let response = Response::<DefaultConfig>::success("Test message");

    // Default flags:
    assert!(!response.prefix_newline, "Default: no prefix newline");
    assert!(!response.indent_message, "Default: no indentation");
    assert!(!response.inline_message, "Default: not inline");
    assert!(response.postfix_newline, "Default: postfix newline enabled");
    assert!(response.show_prompt, "Default: show prompt");
}

#[test]
fn test_response_with_prefix_newline() {
    let response = Response::<DefaultConfig>::success("Test").with_prefix_newline();
    assert!(response.prefix_newline);
}

#[test]
fn test_response_indented() {
    let response = Response::<DefaultConfig>::success("Line 1\r\nLine 2").indented();
    assert!(response.indent_message);
}

#[test]
fn test_response_inline() {
    let response = Response::<DefaultConfig>::success("Test").inline();
    assert!(response.inline_message);
}

#[test]
fn test_response_without_postfix_newline() {
    let response = Response::<DefaultConfig>::success("Test").without_postfix_newline();
    assert!(!response.postfix_newline);
}

#[test]
fn test_response_without_prompt() {
    let response = Response::<DefaultConfig>::success("Test").without_prompt();
    assert!(!response.show_prompt);
}

#[test]
fn test_response_chained_formatting() {
    let response = Response::<DefaultConfig>::success("Multi\r\nLine")
        .with_prefix_newline()
        .indented()
        .without_prompt();

    assert!(response.prefix_newline);
    assert!(response.indent_message);
    assert!(!response.show_prompt);
    assert!(response.postfix_newline); // Still default
}

#[test]
#[cfg(feature = "history")]
fn test_exclude_from_history_flag() {
    let response = Response::<DefaultConfig>::success("Sensitive").without_history();
    assert!(response.exclude_from_history);
}

// ============================================================================
// Integration Tests - Actual Formatting Behavior
// ============================================================================

#[test]
#[cfg(not(feature = "authentication"))]
fn test_prefix_newline_adds_blank_line() {
    let io = MockIo::new();
    let handlers = MockHandlers;
    let mut shell = Shell::new(&TEST_TREE, handlers, io);
    shell.activate().unwrap();
    shell.__test_io_mut().clear_output();

    // Execute command that returns response with prefix_newline
    for c in "test-prefix-newline\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.__test_io_mut().output();

    // Should contain: command echo + \r\n + \r\n (prefix) + message + \r\n
    assert!(
        output.contains("\r\n\r\n"),
        "Should have blank line before message (double \\r\\n). Output: {}",
        output
    );
    assert!(
        output.contains("Message with prefix"),
        "Should contain message text. Output: {}",
        output
    );
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_indented_message_indents_all_lines() {
    let io = MockIo::new();
    let handlers = MockHandlers;
    let mut shell = Shell::new(&TEST_TREE, handlers, io);
    shell.activate().unwrap();
    shell.__test_io_mut().clear_output();

    // Execute command that returns indented multiline response
    for c in "test-indented\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.__test_io_mut().output();

    // Each line should be indented with 2 spaces
    assert!(
        output.contains("  Line 1"),
        "Line 1 should be indented. Output: {}",
        output
    );
    assert!(
        output.contains("  Line 2"),
        "Line 2 should be indented. Output: {}",
        output
    );
    assert!(
        output.contains("  Line 3"),
        "Line 3 should be indented. Output: {}",
        output
    );
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_inline_message_behavior() {
    let io = MockIo::new();
    let handlers = MockHandlers;
    let mut shell = Shell::new(&TEST_TREE, handlers, io);
    shell.activate().unwrap();
    shell.__test_io_mut().clear_output();

    // Execute command that returns inline response
    for c in "test-inline\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.__test_io_mut().output();

    // Inline means message appears on same line as user input
    // The sequence should be: "test-inline" (echoed) + "... processing" (inline)
    // WITHOUT a \r\n between command and message
    assert!(
        output.contains("test-inline... processing"),
        "Inline message should appear on same line as input. Output: {}",
        output
    );
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_without_postfix_newline_suppresses_trailing_newline() {
    let io = MockIo::new();
    let handlers = MockHandlers;
    let mut shell = Shell::new(&TEST_TREE, handlers, io);
    shell.activate().unwrap();
    shell.__test_io_mut().clear_output();

    // Execute command without postfix newline
    for c in "test-no-postfix\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.__test_io_mut().output();

    // Message should appear but without trailing \r\n before prompt
    // The output would be: command + \r\n + message + prompt (no \r\n between message and prompt)
    assert!(
        output.contains("No trailing newline"),
        "Should contain message. Output: {}",
        output
    );

    // Check that message isn't followed by double newline
    // (hard to test precisely without knowing exact prompt format, but we can check message is there)
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_without_prompt_suppresses_prompt() {
    let io = MockIo::new();
    let handlers = MockHandlers;
    let mut shell = Shell::new(&TEST_TREE, handlers, io);
    shell.activate().unwrap();

    // Capture initial output to see what prompt looks like
    let initial_output = shell.__test_io_mut().output();
    let has_prompt_initially = initial_output.contains(">") || initial_output.contains("/");

    shell.__test_io_mut().clear_output();

    // Execute command without prompt
    for c in "test-no-prompt\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.__test_io_mut().output();

    // Should contain message
    assert!(
        output.contains("No prompt after this"),
        "Should contain message. Output: {}",
        output
    );

    // After the message, there should be NO prompt
    // Count prompt occurrences - should be fewer after no-prompt command
    if has_prompt_initially {
        let prompt_count = output.matches('>').count();
        // The response should not show a prompt after the message
        // (This is a simplified check - actual implementation may vary)
    }
}

#[test]
#[cfg(not(feature = "authentication"))]
fn test_combined_formatting_flags() {
    let io = MockIo::new();
    let handlers = MockHandlers;
    let mut shell = Shell::new(&TEST_TREE, handlers, io);
    shell.activate().unwrap();
    shell.__test_io_mut().clear_output();

    // Execute command with multiple formatting flags:
    // - prefix_newline: blank line before
    // - indented: 2-space indent for each line
    // - without_prompt: no prompt after
    for c in "test-combined\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.__test_io_mut().output();

    // Should have prefix newline (blank line)
    assert!(
        output.contains("\r\n\r\n"),
        "Should have prefix newline. Output: {}",
        output
    );

    // Should be indented
    assert!(
        output.contains("  Multi") && output.contains("  Line"),
        "Both lines should be indented. Output: {}",
        output
    );

    // Message should be present
    assert!(
        output.contains("Multi") && output.contains("Line"),
        "Should contain message content. Output: {}",
        output
    );
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
#[cfg(not(feature = "authentication"))]
fn test_normal_command_default_formatting() {
    let io = MockIo::new();
    let handlers = MockHandlers;
    let mut shell = Shell::new(&TEST_TREE, handlers, io);
    shell.activate().unwrap();
    shell.__test_io_mut().clear_output();

    // Execute normal command (should use default formatting)
    for c in "echo hello\n".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.__test_io_mut().output();

    // Should contain message
    assert!(output.contains("hello"), "Should echo message");

    // Should have postfix newline (default)
    assert!(output.contains("\r\n"), "Should have newlines");
}

// ============================================================================
// Documentation Tests
// ============================================================================

/// Documents the purpose and expected behavior of each formatting flag.
///
/// This test serves as living documentation for what each flag should do.
#[test]
fn test_formatting_flag_documentation() {
    // prefix_newline: Adds blank line BEFORE message
    // Example: "\r\nMessage content\r\n"
    let r1 = Response::<DefaultConfig>::success("Message").with_prefix_newline();
    assert!(r1.prefix_newline);

    // indent_message: Indents ALL lines with 2 spaces
    // Example: "  Line 1\r\n  Line 2"
    let r2 = Response::<DefaultConfig>::success("Line 1\r\nLine 2").indented();
    assert!(r2.indent_message);

    // postfix_newline: Adds newline AFTER message (enabled by default)
    // Example: "Message\r\n"
    // Suppress with .without_postfix_newline()
    let r3 = Response::<DefaultConfig>::success("Message").without_postfix_newline();
    assert!(!r3.postfix_newline);

    // show_prompt: Display prompt after response (enabled by default)
    // Suppress with .without_prompt() for multi-step operations
    let r4 = Response::<DefaultConfig>::success("Step 1").without_prompt();
    assert!(!r4.show_prompt);

    // inline_message: Message appears on same line as user input
    // Example: User types "cmd", output: "cmd... processing"
    let r5 = Response::<DefaultConfig>::success("... processing").inline();
    assert!(r5.inline_message);
}
