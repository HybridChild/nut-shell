//! Access level definition for build-time credentials example

use nut_shell::AccessLevel;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, AccessLevel)]
pub enum PicoAccessLevel {
    User = 0,
    Admin = 1,
}
