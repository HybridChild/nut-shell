//! Access level definition for STM32 examples

use nut_shell::AccessLevel;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, AccessLevel)]
pub enum Stm32AccessLevel {
    User = 0,
    Admin = 1,
}
