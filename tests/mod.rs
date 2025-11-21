//! Integration tests for nut-shell.
//!
//! # Feature Flag Testing
//!
//! This test suite validates behavior across different feature combinations.
//! Tests should work correctly whether features are enabled or disabled.
//!
//! ## Running Tests with Different Feature Combinations
//!
//! ```bash
//! # All features (default)
//! cargo test
//! cargo test --all-features
//!
//! # Minimal (no optional features)
//! cargo test --no-default-features
//!
//! # Individual features
//! cargo test --no-default-features --features authentication
//! cargo test --no-default-features --features completion
//! cargo test --no-default-features --features history
//!
//! # Feature combinations
//! cargo test --no-default-features --features "authentication,completion"
//! cargo test --no-default-features --features "authentication,history"
//! cargo test --no-default-features --features "completion,history"
//! ```
//!
//! ## Test Organization
//!
//! - `fixtures/` - Shared test utilities (MockIo, MockAccessLevel, TEST_TREE)
//! - Individual test files per phase/module
//!
//! ## Feature-Gated Tests
//!
//! Use `#[cfg(feature = "...")]` to conditionally compile tests:
//!
//! ```rust,ignore
//! #[test]
//! #[cfg(feature = "authentication")]
//! fn test_login() {
//!     // Only runs when authentication feature enabled
//! }
//!
//! #[test]
//! #[cfg(not(feature = "authentication"))]
//! fn test_no_auth_mode() {
//!     // Only runs when authentication feature disabled
//! }
//! ```
//!
//! ## Best Practices
//!
//! 1. **Test both modes**: Write tests for feature-enabled AND feature-disabled paths
//! 2. **Use stub pattern**: Prefer stub functions that return empty/None when disabled
//! 3. **CI validation**: All feature combinations should pass in CI
//! 4. **Document expectations**: Clearly note what behavior changes with features

// Re-export fixtures for use in test files
pub mod fixtures;

// Test modules (implementation phases):
mod test_tree; // Phase 3: Tree data model and metadata/execution separation
mod test_request_response; // Phase 5: Request/Response types
mod test_type_validation; // Checkpoint: Type-Level Integration Validation

// Shell tests
#[cfg(feature = "authentication")]
mod test_shell_auth; // Shell authentication and password masking tests

// Future test modules:
// mod test_io;
// mod test_shell;
