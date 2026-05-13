//! Access level definition for STM32H753ZI examples

use nut_shell::AccessLevel;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, AccessLevel)]
pub enum H753AccessLevel {
    User = 0,
    Admin = 1,
}
