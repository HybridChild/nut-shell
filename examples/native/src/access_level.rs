//! Shared access level definition for native examples

use nut_shell::AccessLevel;

/// Standard three-tier access level hierarchy used across examples.
///
/// This provides a simple Guest < User < Admin hierarchy that's suitable
/// for demonstration purposes.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, AccessLevel)]
pub enum ExampleAccessLevel {
    Guest = 0,
    User = 1,
    Admin = 2,
}
