//! Integration tests for nut-shell.
//!
//! Tests validate behavior across all feature combinations. See DEVELOPMENT.md for
//! testing workflows and feature flag usage.
//!
//! ## Test Organization
//!
//! - **test_shell_core**: Command execution, navigation, access control
//! - **test_shell_editing**: Input editing, buffer management, terminal behavior
//! - **test_shell_features**: Tab completion, history, async (feature-gated)
//! - **test_shell_auth**: Authentication and password masking
//! - **test_tree**: Tree metadata and node API
//! - **test_rust_optimizations**: Zero-size types, const init, memory layout
//!
//! Note: `helpers` module is loaded independently by each test file to share
//! test utilities across integration tests.

mod test_rust_optimizations;
mod test_shell_auth;
mod test_shell_core;
mod test_shell_editing;
mod test_shell_features;
mod test_tree;
