//! Shared test helpers to reduce duplication across integration tests.

#![allow(dead_code)]

#[allow(clippy::duplicate_mod)]
#[path = "fixtures/mod.rs"]
mod fixtures;

use fixtures::{MockAccessLevel, MockHandler, MockIo, TEST_TREE};
use heapless::String as HString;
use nut_shell::config::DefaultConfig;
use nut_shell::Shell;

// ============================================================================
// Shell Creation Helpers
// ============================================================================

/// Create a shell with no authentication, ready for testing.
#[cfg(not(feature = "authentication"))]
pub fn create_test_shell() -> Shell<'static, MockAccessLevel, MockIo, MockHandler, DefaultConfig> {
    let io = MockIo::new();
    let handler = MockHandler;
    let mut shell = Shell::new(&TEST_TREE, handler, io);
    shell.activate().unwrap();
    shell.io_mut().clear_output();
    shell
}

// Static credential provider for auth tests
#[cfg(feature = "authentication")]
static AUTH_PROVIDER: std::sync::OnceLock<nut_shell::auth::ConstCredentialProvider<MockAccessLevel, nut_shell::auth::password::Sha256Hasher, 2>> = std::sync::OnceLock::new();

/// Get or create the static auth provider.
#[cfg(feature = "authentication")]
fn get_auth_provider() -> &'static nut_shell::auth::ConstCredentialProvider<MockAccessLevel, nut_shell::auth::password::Sha256Hasher, 2> {
    AUTH_PROVIDER.get_or_init(|| {
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

        ConstCredentialProvider::new(users, hasher)
    })
}

/// Create an authenticated shell with test provider.
#[cfg(feature = "authentication")]
pub fn create_auth_shell() -> Shell<'static, MockAccessLevel, MockIo, MockHandler, DefaultConfig> {
    let provider = get_auth_provider();
    let io = MockIo::new();
    let handler = MockHandler;
    let mut shell = Shell::new(&TEST_TREE, handler, provider, io);

    shell.activate().unwrap();
    shell.io_mut().clear_output();

    shell
}

// ============================================================================
// Command Execution Helpers
// ============================================================================

/// Execute a command string and return the output.
#[cfg(not(feature = "authentication"))]
pub fn execute_command(
    shell: &mut Shell<'static, MockAccessLevel, MockIo, MockHandler, DefaultConfig>,
    cmd: &str,
) -> HString<1024> {
    shell.io_mut().clear_output();

    for c in cmd.chars() {
        shell.process_char(c).unwrap();
    }

    if !cmd.ends_with('\n') {
        shell.process_char('\n').unwrap();
    }

    shell.io_mut().output()
}

/// Execute a command and get output (auth version).
#[cfg(feature = "authentication")]
pub fn execute_command_auth(
    shell: &mut Shell<'static, MockAccessLevel, MockIo, MockHandler, DefaultConfig>,
    cmd: &str,
) -> HString<1024> {
    shell.io_mut().clear_output();

    for c in cmd.chars() {
        shell.process_char(c).unwrap();
    }

    if !cmd.ends_with('\n') {
        shell.process_char('\n').unwrap();
    }

    shell.io_mut().output()
}

/// Type input without executing (no trailing newline).
#[cfg(not(feature = "authentication"))]
pub fn type_input(
    shell: &mut Shell<'static, MockAccessLevel, MockIo, MockHandler, DefaultConfig>,
    input: &str,
) {
    for c in input.chars() {
        shell.process_char(c).unwrap();
    }
}

/// Type input without executing (auth version).
#[cfg(feature = "authentication")]
pub fn type_input_auth(
    shell: &mut Shell<'static, MockAccessLevel, MockIo, MockHandler, DefaultConfig>,
    input: &str,
) {
    for c in input.chars() {
        shell.process_char(c).unwrap();
    }
}

// ============================================================================
// Assertion Helpers
// ============================================================================

/// Assert that output contains an ANSI escape sequence.
pub fn assert_contains_ansi(output: &str, sequence: &str) {
    assert!(
        output.contains(sequence),
        "Expected ANSI sequence '{}' in output, got: {:?}",
        sequence.escape_default(),
        output
    );
}

/// Assert that output contains all expected strings.
pub fn assert_contains_all(output: &str, expected: &[&str]) {
    for exp in expected {
        assert!(
            output.contains(exp),
            "Expected '{}' in output, got: {}",
            exp,
            output
        );
    }
}

/// Assert that output does NOT contain any of the strings.
pub fn assert_contains_none(output: &str, forbidden: &[&str]) {
    for forbid in forbidden {
        assert!(
            !output.contains(forbid),
            "Did not expect '{}' in output, got: {}",
            forbid,
            output
        );
    }
}
