//! Tests for Shell authentication and password masking.
//!
//! These tests validate the authentication flow and password masking behavior.

#![cfg(feature = "authentication")]

#[allow(clippy::duplicate_mod)]
#[path = "fixtures/mod.rs"]
mod fixtures;

#[path = "helpers.rs"]
mod helpers;

use nut_shell::config::{DefaultConfig, ShellConfig};

// ============================================================================
// Password Masking Tests
// ============================================================================

#[test]
fn test_password_masking_basic() {
    let mut shell = helpers::create_auth_shell();

    // Type username
    helpers::type_input_auth(&mut shell, "admin");

    let output = shell.io().output();
    assert!(
        output.contains("admin"),
        "Username should be echoed normally"
    );
    assert!(!output.contains("*"), "No masking before colon");

    shell.io_mut().clear_output();

    // Type colon delimiter
    shell.process_char(':').unwrap();
    let output = shell.io().output();
    assert_eq!(output, ":", "Colon should be echoed normally");

    shell.io_mut().clear_output();

    // Type password
    helpers::type_input_auth(&mut shell, "pass");

    let output = shell.io().output();
    assert_eq!(output, "****", "Password should be masked with asterisks");
}

#[test]
fn test_password_with_special_chars() {
    let mut shell = helpers::create_auth_shell();

    // Type username with @ and password with special characters
    helpers::type_input_auth(&mut shell, "user@example.com:P@ss!");

    let output = shell.io().output();

    // Username should be visible (including @)
    assert!(
        output.contains("user@example.com:"),
        "Username with @ should be visible"
    );

    // Password should be masked (5 characters = 5 asterisks)
    assert!(
        output.contains("*****"),
        "Password with special chars should be masked"
    );

    // Should NOT contain actual password
    assert!(
        !output.contains("P@ss!"),
        "Actual password should not appear"
    );
}

#[test]
fn test_password_with_multiple_colons() {
    let mut shell = helpers::create_auth_shell();

    // Type password with additional colons (as per spec, they're part of password)
    helpers::type_input_auth(&mut shell, "admin:P:a:s:s");

    let output = shell.io().output();

    // First part should be visible
    assert!(
        output.contains("admin:"),
        "Username and first colon visible"
    );

    // Password part should be fully masked (7 characters including colons)
    // "P:a:s:s" = 7 characters
    let asterisk_count = output.matches('*').count();
    assert_eq!(
        asterisk_count, 7,
        "All password characters including colons should be masked"
    );

    // Should NOT contain the actual password characters
    assert!(
        !output.contains("P:a:s:s"),
        "Actual password should not appear"
    );
}

#[test]
fn test_password_masking_with_backspace() {
    let mut shell = helpers::create_auth_shell();

    // Type "admin:pass"
    helpers::type_input_auth(&mut shell, "admin:pass");

    // Verify password is masked (4 asterisks for "pass")
    let output_before = shell.io().output();
    assert!(output_before.contains("****"), "Password should be masked");

    shell.io_mut().clear_output();

    // Backspace once (should remove one character)
    shell.process_char('\x7f').unwrap(); // DEL character

    let output_after = shell.io().output();
    // Backspace sequence is "\x08 \x08" (backspace, space, backspace)
    assert!(
        output_after.contains("\x08"),
        "Should send backspace sequence"
    );
}

#[test]
fn test_password_masking_empty_username() {
    let mut shell = helpers::create_auth_shell();

    // Type just colon and password (empty username per spec is invalid, but masking should work)
    helpers::type_input_auth(&mut shell, ":password");

    let output = shell.io().output();

    // Colon should be visible
    assert!(output.starts_with(":"), "Colon should be visible");

    // Password should be masked (8 characters)
    assert!(
        output.contains("********"),
        "Password should be masked even with empty username"
    );
}

#[test]
fn test_password_masking_unicode_chars() {
    let mut shell = helpers::create_auth_shell();

    // Type password with unicode characters
    helpers::type_input_auth(&mut shell, "admin:päss");

    let output = shell.io().output();

    // Should have 4 asterisks (one per character: p, ä, s, s)
    let asterisk_count = output.matches('*').count();
    assert_eq!(
        asterisk_count, 4,
        "Unicode characters should be masked individually"
    );

    // Should not contain actual unicode password
    assert!(
        !output.contains("päss"),
        "Actual password should not appear"
    );
}

#[test]
fn test_double_esc_clears_masked_input() {
    let mut shell = helpers::create_auth_shell();

    // Type partial login
    helpers::type_input_auth(&mut shell, "admin:pass");

    // Verify masked password was displayed before clearing
    let output_before = shell.io_mut().output();
    assert!(
        output_before.contains("admin:****"),
        "Password should be masked before clearing: {}",
        output_before
    );

    shell.io_mut().clear_output();

    // Double ESC to clear
    shell.process_char('\x1b').unwrap(); // First ESC
    shell.process_char('\x1b').unwrap(); // Second ESC

    let clear_output = shell.io().output();

    // Should send clear sequence: \r (CR) + \x1b[K (clear to end of line) + prompt
    assert!(
        clear_output.contains("\r"),
        "Should send carriage return after double ESC: {:?}",
        clear_output
    );
    assert!(
        clear_output.contains("\x1b[K"),
        "Should send clear-to-EOL sequence after double ESC: {:?}",
        clear_output
    );

    // Now press enter - should not process the cleared login
    shell.io_mut().clear_output();
    shell.process_char('\n').unwrap();

    let output_after = shell.io_mut().output();

    // Empty input should show invalid format message, not successful login
    assert!(
        output_after.contains(DefaultConfig::MSG_INVALID_LOGIN_FORMAT),
        "Empty input after double ESC should show invalid format message, got: {}",
        output_after
    );
    assert!(
        !output_after.contains(DefaultConfig::MSG_LOGIN_SUCCESS)
            && !output_after.contains(DefaultConfig::MSG_LOGIN_FAILED),
        "Double ESC should have cleared login buffer, got: {}",
        output_after
    );
}
