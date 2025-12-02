//! Tests for Shell authentication and password masking.
//!
//! These tests validate the authentication flow and password masking behavior
//! as specified in SPECIFICATION.md.

#![cfg(feature = "authentication")]

#[allow(clippy::duplicate_mod)]
#[path = "fixtures/mod.rs"]
mod fixtures;
use fixtures::{MockAccessLevel, MockHandlers, MockIo, TEST_TREE};
use nut_shell::auth::{ConstCredentialProvider, User, password::Sha256Hasher};
use nut_shell::config::DefaultConfig;
use nut_shell::shell::Shell;

// ============================================================================
// Test Credential Provider
// ============================================================================

/// Create test users for authentication testing.
fn create_test_users() -> [User<MockAccessLevel>; 2] {
    // Create dummy hash and salt (for testing, actual values don't matter)
    let hash = [0u8; 32];
    let salt = [0u8; 16];

    [
        User::new("admin", MockAccessLevel::Admin, hash, salt).unwrap(),
        User::new("user", MockAccessLevel::User, hash, salt).unwrap(),
    ]
}

// ============================================================================
// Password Masking Tests
// ============================================================================

#[test]
fn test_password_masking_basic() {
    let users = create_test_users();
    let hasher = Sha256Hasher::new();
    let provider = ConstCredentialProvider::new(users, hasher);
    let io = MockIo::new();
    let handlers = MockHandlers;
    let mut shell: Shell<_, _, _, DefaultConfig> = Shell::new(&TEST_TREE, handlers, &provider, io);

    // Activate shell (shows welcome and login prompt)
    shell.activate().unwrap();

    // Clear initial output
    shell.io_mut().clear_output();

    // Type username
    for c in "admin".chars() {
        shell.process_char(c).unwrap();
    }

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
    for c in "pass".chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.io().output();
    assert_eq!(output, "****", "Password should be masked with asterisks");
}

#[test]
fn test_password_masking_full_sequence() {
    let users = create_test_users();
    let hasher = Sha256Hasher::new();
    let provider = ConstCredentialProvider::new(users, hasher);
    let io = MockIo::new();
    let handlers = MockHandlers;
    let mut shell: Shell<_, _, _, DefaultConfig> = Shell::new(&TEST_TREE, handlers, &provider, io);

    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // Type "admin:pass123"
    let input = "admin:pass123";
    for c in input.chars() {
        shell.process_char(c).unwrap();
    }

    let output = shell.io().output();

    // Should contain "admin:" echoed normally
    assert!(
        output.contains("admin:"),
        "Username and colon should be visible"
    );

    // Should contain "*******" for the password (7 characters for "pass123")
    assert!(
        output.contains("*******"),
        "Password should be masked (7 chars)"
    );

    // Should NOT contain actual password
    assert!(
        !output.contains("pass123"),
        "Actual password should not appear"
    );
}

#[test]
fn test_password_with_special_chars() {
    let users = create_test_users();
    let hasher = Sha256Hasher::new();
    let provider = ConstCredentialProvider::new(users, hasher);
    let io = MockIo::new();
    let handlers = MockHandlers;
    let mut shell: Shell<_, _, _, DefaultConfig> = Shell::new(&TEST_TREE, handlers, &provider, io);

    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // Type username with @ and password with special characters
    let input = "user@example.com:P@ss!";
    for c in input.chars() {
        shell.process_char(c).unwrap();
    }

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
    let users = create_test_users();
    let hasher = Sha256Hasher::new();
    let provider = ConstCredentialProvider::new(users, hasher);
    let io = MockIo::new();
    let handlers = MockHandlers;
    let mut shell: Shell<_, _, _, DefaultConfig> = Shell::new(&TEST_TREE, handlers, &provider, io);

    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // Type password with additional colons (as per spec, they're part of password)
    let input = "admin:P:a:s:s";
    for c in input.chars() {
        shell.process_char(c).unwrap();
    }

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
    let users = create_test_users();
    let hasher = Sha256Hasher::new();
    let provider = ConstCredentialProvider::new(users, hasher);
    let io = MockIo::new();
    let handlers = MockHandlers;
    let mut shell: Shell<_, _, _, DefaultConfig> = Shell::new(&TEST_TREE, handlers, &provider, io);

    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // Type "admin:pass"
    for c in "admin:pass".chars() {
        shell.process_char(c).unwrap();
    }

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
    let users = create_test_users();
    let hasher = Sha256Hasher::new();
    let provider = ConstCredentialProvider::new(users, hasher);
    let io = MockIo::new();
    let handlers = MockHandlers;
    let mut shell: Shell<_, _, _, DefaultConfig> = Shell::new(&TEST_TREE, handlers, &provider, io);

    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // Type just colon and password (empty username per spec is invalid, but masking should work)
    let input = ":password";
    for c in input.chars() {
        shell.process_char(c).unwrap();
    }

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
    let users = create_test_users();
    let hasher = Sha256Hasher::new();
    let provider = ConstCredentialProvider::new(users, hasher);
    let io = MockIo::new();
    let handlers = MockHandlers;
    let mut shell: Shell<_, _, _, DefaultConfig> = Shell::new(&TEST_TREE, handlers, &provider, io);

    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // Type password with unicode characters
    let input = "admin:päss";
    for c in input.chars() {
        shell.process_char(c).unwrap();
    }

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
    let users = create_test_users();
    let hasher = Sha256Hasher::new();
    let provider = ConstCredentialProvider::new(users, hasher);
    let io = MockIo::new();
    let handlers = MockHandlers;
    let mut shell: Shell<_, _, _, DefaultConfig> = Shell::new(&TEST_TREE, handlers, &provider, io);

    shell.activate().unwrap();
    shell.io_mut().clear_output();

    // Type partial login
    for c in "admin:pass".chars() {
        shell.process_char(c).unwrap();
    }

    shell.io_mut().clear_output();

    // Double ESC to clear
    shell.process_char('\x1b').unwrap(); // First ESC
    shell.process_char('\x1b').unwrap(); // Second ESC

    let output = shell.io().output();

    // Should trigger ClearAndRedraw (sends CR and clear sequence)
    assert!(output.contains("\r"), "Should redraw after double ESC");
}

#[test]
fn test_character_by_character_masking() {
    let users = create_test_users();
    let hasher = Sha256Hasher::new();
    let provider = ConstCredentialProvider::new(users, hasher);
    let io = MockIo::new();
    let handlers = MockHandlers;
    let mut shell: Shell<_, _, _, DefaultConfig> = Shell::new(&TEST_TREE, handlers, &provider, io);

    shell.activate().unwrap();

    // Type username
    shell.io_mut().clear_output();
    shell.process_char('a').unwrap();
    assert_eq!(
        shell.io().output(),
        "a",
        "First char of username visible"
    );

    shell.io_mut().clear_output();
    shell.process_char('d').unwrap();
    assert_eq!(
        shell.io().output(),
        "d",
        "Second char of username visible"
    );

    // Type colon
    shell.io_mut().clear_output();
    shell.process_char(':').unwrap();
    assert_eq!(shell.io().output(), ":", "Colon visible");

    // Type first password char
    shell.io_mut().clear_output();
    shell.process_char('p').unwrap();
    assert_eq!(
        shell.io().output(),
        "*",
        "First password char masked"
    );

    // Type second password char
    shell.io_mut().clear_output();
    shell.process_char('a').unwrap();
    assert_eq!(
        shell.io().output(),
        "*",
        "Second password char masked"
    );

    // Type third password char
    shell.io_mut().clear_output();
    shell.process_char('s').unwrap();
    assert_eq!(
        shell.io().output(),
        "*",
        "Third password char masked"
    );
}
