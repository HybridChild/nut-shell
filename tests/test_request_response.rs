//! Integration tests for Request and Response types.
//!
//! Validates Phase 5 implementation:
//! - Request enum variants and feature gating
//! - Response struct with formatting flags
//! - History exclusion functionality
//! - HistoryDirection and CliState enums

use nut_shell::config::DefaultConfig;
use nut_shell::response::Response;
use nut_shell::shell::{CliState, HistoryDirection, Request};

// ========================================
// HistoryDirection Tests
// ========================================

#[test]
fn test_history_direction_values() {
    assert_eq!(HistoryDirection::Previous as u8, 0);
    assert_eq!(HistoryDirection::Next as u8, 1);
}

#[test]
fn test_history_direction_copy() {
    let dir1 = HistoryDirection::Previous;
    let dir2 = dir1; // Should be Copy
    assert_eq!(dir1, dir2);
}

#[test]
fn test_history_direction_clone() {
    let dir1 = HistoryDirection::Next;
    let dir2 = dir1.clone();
    assert_eq!(dir1, dir2);
}

// ========================================
// CliState Tests
// ========================================

#[test]
fn test_cli_state_inactive() {
    let state = CliState::Inactive;
    assert_eq!(state, CliState::Inactive);
}

#[test]
fn test_cli_state_logged_in() {
    let state = CliState::LoggedIn;
    assert_eq!(state, CliState::LoggedIn);
}

#[test]
#[cfg(feature = "authentication")]
fn test_cli_state_logged_out() {
    let state = CliState::LoggedOut;
    assert_eq!(state, CliState::LoggedOut);
    assert_ne!(state, CliState::LoggedIn);
}

#[test]
fn test_cli_state_copy() {
    let state1 = CliState::Inactive;
    let state2 = state1; // Should be Copy
    assert_eq!(state1, state2);
}

// ========================================
// Response Tests
// ========================================

#[test]
fn test_response_success_default() {
    let response = Response::<DefaultConfig>::success("Command executed");
    assert!(response.is_success);
    assert_eq!(response.message.as_str(), "Command executed");
    assert!(!response.inline_message);
    assert!(!response.prefix_newline);
    assert!(!response.indent_message);
    assert!(response.postfix_newline);
    assert!(response.show_prompt);
}

#[test]
fn test_response_error_default() {
    let response = Response::<DefaultConfig>::error("Command failed");
    assert!(!response.is_success);
    assert_eq!(response.message.as_str(), "Command failed");
    assert!(!response.inline_message);
    assert!(!response.prefix_newline);
    assert!(!response.indent_message);
    assert!(response.postfix_newline);
    assert!(response.show_prompt);
}

#[test]
fn test_response_empty_message() {
    let response = Response::<DefaultConfig>::success("");
    assert!(response.is_success);
    assert_eq!(response.message.as_str(), "");
}

#[test]
fn test_response_long_message() {
    let long_msg = "A".repeat(250);
    let response = Response::<DefaultConfig>::success(&long_msg);
    assert!(response.is_success);
    // Message should be truncated or fit within buffer
    assert!(response.message.len() <= 256);
}

#[test]
#[cfg(feature = "history")]
fn test_response_exclude_from_history_default() {
    let response = Response::<DefaultConfig>::success("Test");
    assert!(!response.exclude_from_history);
}

#[test]
#[cfg(feature = "history")]
fn test_response_success_no_history() {
    let response = Response::<DefaultConfig>::success_no_history("Sensitive data");
    assert!(response.is_success);
    assert!(response.exclude_from_history);
    assert_eq!(response.message.as_str(), "Sensitive data");
}

#[test]
#[cfg(feature = "history")]
fn test_response_without_history_builder() {
    let response = Response::<DefaultConfig>::success("Password set")
        .without_history();
    assert!(response.is_success);
    assert!(response.exclude_from_history);
}

#[test]
#[cfg(feature = "history")]
fn test_response_error_without_history() {
    let response = Response::<DefaultConfig>::error("Auth failed")
        .without_history();
    assert!(!response.is_success);
    assert!(response.exclude_from_history);
}

// ========================================
// Request Tests - Command Variant
// ========================================

#[test]
fn test_request_command_no_args() {
    let mut path = heapless::String::<128>::new();
    path.push_str("help").unwrap();
    let args = heapless::Vec::new();
    #[cfg(feature = "history")]
    let original = {
        let mut s = heapless::String::<128>::new();
        s.push_str("help").unwrap();
        s
    };

    let request = Request::<DefaultConfig>::Command {
        path,
        args,
        #[cfg(feature = "history")]
        original,
        _phantom: core::marker::PhantomData,
    };

    match request {
        Request::Command { path, args, .. } => {
            assert_eq!(path.as_str(), "help");
            assert_eq!(args.len(), 0);
        }
        #[allow(unreachable_patterns)]
        _ => panic!("Expected Command variant"),
    }
}

#[test]
fn test_request_command_with_args() {
    let mut path = heapless::String::<128>::new();
    path.push_str("echo").unwrap();

    let mut args = heapless::Vec::new();
    let mut hello = heapless::String::<128>::new();
    hello.push_str("hello").unwrap();
    let mut world = heapless::String::<128>::new();
    world.push_str("world").unwrap();
    args.push(hello).unwrap();
    args.push(world).unwrap();

    #[cfg(feature = "history")]
    let original = {
        let mut s = heapless::String::<128>::new();
        s.push_str("echo hello world").unwrap();
        s
    };

    let request = Request::<DefaultConfig>::Command {
        path,
        args,
        #[cfg(feature = "history")]
        original,
        _phantom: core::marker::PhantomData,
    };

    match request {
        Request::Command { path, args, .. } => {
            assert_eq!(path.as_str(), "echo");
            assert_eq!(args.len(), 2);
            assert_eq!(args[0].as_str(), "hello");
            assert_eq!(args[1].as_str(), "world");
        }
        #[allow(unreachable_patterns)]
        _ => panic!("Expected Command variant"),
    }
}

#[test]
#[cfg(feature = "history")]
fn test_request_command_with_original() {
    let mut path = heapless::String::<128>::new();
    path.push_str("reboot").unwrap();
    let mut original = heapless::String::<128>::new();
    original.push_str("reboot").unwrap();

    let request = Request::<DefaultConfig>::Command {
        path,
        args: heapless::Vec::new(),
        original,
        _phantom: core::marker::PhantomData,
    };

    match request {
        Request::Command { path, original, .. } => {
            assert_eq!(path.as_str(), "reboot");
            assert_eq!(original.as_str(), "reboot");
        }
        #[allow(unreachable_patterns)]
        _ => panic!("Expected Command variant"),
    }
}

// ========================================
// Request Tests - Authentication Variants
// ========================================

#[test]
#[cfg(feature = "authentication")]
fn test_request_login() {
    let mut username = heapless::String::<32>::new();
    username.push_str("admin").unwrap();
    let mut password = heapless::String::<64>::new();
    password.push_str("secret123").unwrap();

    let request = Request::<DefaultConfig>::Login {
        username,
        password,
    };

    match request {
        Request::Login { username, password } => {
            assert_eq!(username.as_str(), "admin");
            assert_eq!(password.as_str(), "secret123");
        }
        #[allow(unreachable_patterns)]
        _ => panic!("Expected Login variant"),
    }
}

#[test]
#[cfg(feature = "authentication")]
fn test_request_invalid_login() {
    let request = Request::<DefaultConfig>::InvalidLogin;

    match request {
        Request::InvalidLogin => {
            // Success - variant exists
        }
        #[allow(unreachable_patterns)]
        _ => panic!("Expected InvalidLogin variant"),
    }
}

// ========================================
// Request Tests - Completion Variant
// ========================================

#[test]
#[cfg(feature = "completion")]
fn test_request_tab_complete() {
    let mut path = heapless::String::<128>::new();
    path.push_str("sys").unwrap();

    let request = Request::<DefaultConfig>::TabComplete {
        path,
    };

    match request {
        Request::TabComplete { path } => {
            assert_eq!(path.as_str(), "sys");
        }
        #[allow(unreachable_patterns)]
        _ => panic!("Expected TabComplete variant"),
    }
}

#[test]
#[cfg(feature = "completion")]
fn test_request_tab_complete_empty() {
    let request = Request::<DefaultConfig>::TabComplete {
        path: heapless::String::new(),
    };

    match request {
        Request::TabComplete { path } => {
            assert_eq!(path.as_str(), "");
        }
        #[allow(unreachable_patterns)]
        _ => panic!("Expected TabComplete variant"),
    }
}

// ========================================
// Request Tests - History Variant
// ========================================

#[test]
#[cfg(feature = "history")]
fn test_request_history_previous() {
    let mut buffer = heapless::String::<128>::new();
    buffer.push_str("current input").unwrap();

    let request = Request::<DefaultConfig>::History {
        direction: HistoryDirection::Previous,
        buffer,
    };

    match request {
        Request::History { direction, buffer } => {
            assert_eq!(direction, HistoryDirection::Previous);
            assert_eq!(buffer.as_str(), "current input");
        }
        #[allow(unreachable_patterns)]
        _ => panic!("Expected History variant"),
    }
}

#[test]
#[cfg(feature = "history")]
fn test_request_history_next() {
    let request = Request::<DefaultConfig>::History {
        direction: HistoryDirection::Next,
        buffer: heapless::String::new(),
    };

    match request {
        Request::History { direction, buffer } => {
            assert_eq!(direction, HistoryDirection::Next);
            assert_eq!(buffer.as_str(), "");
        }
        #[allow(unreachable_patterns)]
        _ => panic!("Expected History variant"),
    }
}

// ========================================
// Integration Tests
// ========================================

#[test]
fn test_request_response_workflow_success() {
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
    let response = match request {
        Request::Command { path, .. } => {
            if path.as_str() == "status" {
                Response::<DefaultConfig>::success("System OK")
            } else {
                Response::<DefaultConfig>::error("Unknown command")
            }
        }
        #[allow(unreachable_patterns)]
        _ => Response::<DefaultConfig>::error("Invalid request"),
    };

    assert!(response.is_success);
    assert_eq!(response.message.as_str(), "System OK");
}

#[test]
fn test_request_response_workflow_error() {
    let mut path = heapless::String::<128>::new();
    path.push_str("invalid").unwrap();
    #[cfg(feature = "history")]
    let original = {
        let mut s = heapless::String::<128>::new();
        s.push_str("invalid").unwrap();
        s
    };

    let request = Request::<DefaultConfig>::Command {
        path,
        args: heapless::Vec::new(),
        #[cfg(feature = "history")]
        original,
        _phantom: core::marker::PhantomData,
    };

    let response = match request {
        Request::Command { .. } => {
            Response::<DefaultConfig>::error("Command not found")
        }
        #[allow(unreachable_patterns)]
        _ => Response::<DefaultConfig>::error("Invalid request"),
    };

    assert!(!response.is_success);
    assert_eq!(response.message.as_str(), "Command not found");
}

#[test]
#[cfg(feature = "history")]
fn test_history_workflow() {
    // Test that exclude_from_history flag works in workflow
    let response = Response::<DefaultConfig>::success("Logged in")
        .without_history();

    // Shell would check this flag before adding to history
    if !response.exclude_from_history {
        panic!("Response should be excluded from history");
    }
}

// ========================================
// Feature Combination Tests
// ========================================

#[test]
fn test_cli_state_matches_auth_feature() {
    // Without authentication feature, only Inactive and LoggedIn exist
    let _inactive = CliState::Inactive;
    let _logged_in = CliState::LoggedIn;

    // LoggedOut only exists with authentication feature
    #[cfg(feature = "authentication")]
    let _logged_out = CliState::LoggedOut;
}

#[test]
fn test_request_variants_match_features() {
    // Command variant always exists
    let _cmd = Request::<DefaultConfig>::Command {
        path: heapless::String::new(),
        args: heapless::Vec::new(),
        #[cfg(feature = "history")]
        original: heapless::String::new(),
        _phantom: core::marker::PhantomData,
    };

    // Other variants only exist with their respective features
    #[cfg(feature = "authentication")]
    let _login = Request::<DefaultConfig>::Login {
        username: heapless::String::new(),
        password: heapless::String::new(),
    };

    #[cfg(feature = "completion")]
    let _complete = Request::<DefaultConfig>::TabComplete {
        path: heapless::String::new(),
    };

    #[cfg(feature = "history")]
    let _history = Request::<DefaultConfig>::History {
        direction: HistoryDirection::Previous,
        buffer: heapless::String::new(),
    };
}
