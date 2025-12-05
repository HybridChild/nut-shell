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
// Special Input Helpers (for testing terminal input sequences)
// ============================================================================

/// Press up arrow key (history navigation).
#[cfg(not(feature = "authentication"))]
pub fn press_up_arrow(
    shell: &mut Shell<'static, MockAccessLevel, MockIo, MockHandler, DefaultConfig>,
) {
    shell.process_char('\x1b').unwrap(); // ESC
    shell.process_char('[').unwrap();
    shell.process_char('A').unwrap();
}

/// Press down arrow key (history navigation).
#[cfg(not(feature = "authentication"))]
pub fn press_down_arrow(
    shell: &mut Shell<'static, MockAccessLevel, MockIo, MockHandler, DefaultConfig>,
) {
    shell.process_char('\x1b').unwrap(); // ESC
    shell.process_char('[').unwrap();
    shell.process_char('B').unwrap();
}

/// Press tab key (completion).
#[cfg(not(feature = "authentication"))]
pub fn press_tab(
    shell: &mut Shell<'static, MockAccessLevel, MockIo, MockHandler, DefaultConfig>,
) {
    shell.process_char('\t').unwrap();
}

/// Press backspace key.
#[cfg(not(feature = "authentication"))]
pub fn press_backspace(
    shell: &mut Shell<'static, MockAccessLevel, MockIo, MockHandler, DefaultConfig>,
) {
    shell.process_char('\x08').unwrap();
}

/// Press backspace N times.
#[cfg(not(feature = "authentication"))]
pub fn press_backspace_n(
    shell: &mut Shell<'static, MockAccessLevel, MockIo, MockHandler, DefaultConfig>,
    count: usize,
) {
    for _ in 0..count {
        shell.process_char('\x08').unwrap();
    }
}

/// Press ESC twice (clear buffer).
#[cfg(not(feature = "authentication"))]
pub fn press_double_esc(
    shell: &mut Shell<'static, MockAccessLevel, MockIo, MockHandler, DefaultConfig>,
) {
    shell.process_char('\x1b').unwrap();
    shell.process_char('\x1b').unwrap();
}

/// Press enter key.
#[cfg(not(feature = "authentication"))]
pub fn press_enter(
    shell: &mut Shell<'static, MockAccessLevel, MockIo, MockHandler, DefaultConfig>,
) {
    shell.process_char('\n').unwrap();
}

// Auth versions of input helpers
#[cfg(feature = "authentication")]
pub fn press_up_arrow_auth(
    shell: &mut Shell<'static, MockAccessLevel, MockIo, MockHandler, DefaultConfig>,
) {
    shell.process_char('\x1b').unwrap();
    shell.process_char('[').unwrap();
    shell.process_char('A').unwrap();
}

#[cfg(feature = "authentication")]
pub fn press_down_arrow_auth(
    shell: &mut Shell<'static, MockAccessLevel, MockIo, MockHandler, DefaultConfig>,
) {
    shell.process_char('\x1b').unwrap();
    shell.process_char('[').unwrap();
    shell.process_char('B').unwrap();
}

#[cfg(feature = "authentication")]
pub fn press_backspace_auth(
    shell: &mut Shell<'static, MockAccessLevel, MockIo, MockHandler, DefaultConfig>,
) {
    shell.process_char('\x08').unwrap();
}

#[cfg(feature = "authentication")]
pub fn press_double_esc_auth(
    shell: &mut Shell<'static, MockAccessLevel, MockIo, MockHandler, DefaultConfig>,
) {
    shell.process_char('\x1b').unwrap();
    shell.process_char('\x1b').unwrap();
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

/// Assert that output matches expected (with helpful error message).
pub fn assert_output_matches(output: &str, expected: &str, context: &str) {
    assert_eq!(
        output, expected,
        "{}\nExpected: {:?}\nGot: {:?}",
        context, expected, output
    );
}

/// Count occurrences of a character in output.
pub fn count_char(output: &str, ch: char) -> usize {
    output.matches(ch).count()
}

/// Assert prompt is present in output.
pub fn assert_prompt(output: &str, prompt_fragment: &str) {
    assert!(
        output.contains(prompt_fragment),
        "Expected prompt containing '{}' in output: {}",
        prompt_fragment,
        output
    );
}
