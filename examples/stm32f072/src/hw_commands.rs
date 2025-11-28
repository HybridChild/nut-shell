//! STM32F072 hardware status commands
//!
//! This module provides commands for reading STM32F072 internal hardware status:
//! - Unique chip ID
//! - Clock frequencies
//! - CPU core identification
//! - Reset reason
//!
//! These commands are designed to be reusable across different STM32F072 examples.

use crate::access_level::Stm32AccessLevel;
use core::fmt::Write;
use heapless;
use nut_shell::{
    CliError,
    config::ShellConfig,
    response::Response,
    tree::{CommandKind, CommandMeta},
};

// =============================================================================
// Command Metadata (for use in command trees)
// =============================================================================

pub const CMD_CHIPID: CommandMeta<Stm32AccessLevel> = CommandMeta {
    id: "hw_chipid",
    name: "chipid",
    description: "Display unique device ID (96-bit)",
    access_level: Stm32AccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

pub const CMD_CLOCKS: CommandMeta<Stm32AccessLevel> = CommandMeta {
    id: "hw_clocks",
    name: "clocks",
    description: "Show clock frequencies",
    access_level: Stm32AccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

pub const CMD_CORE: CommandMeta<Stm32AccessLevel> = CommandMeta {
    id: "hw_core",
    name: "core",
    description: "Display CPU core information",
    access_level: Stm32AccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

pub const CMD_BOOTREASON: CommandMeta<Stm32AccessLevel> = CommandMeta {
    id: "hw_bootreason",
    name: "bootreason",
    description: "Show last reset reason",
    access_level: Stm32AccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

// =============================================================================
// Unique Device ID Command
// =============================================================================

/// Read the unique 96-bit device identifier
///
/// Every STM32F072 chip has a globally unique 96-bit identifier programmed
/// at the factory. This ID is stored in read-only memory at a fixed address.
///
/// Location: 0x1FFF_F7AC (UID_BASE)
/// - UID[31:0]  at 0x1FFF_F7AC
/// - UID[63:32] at 0x1FFF_F7B0
/// - UID[95:64] at 0x1FFF_F7B4
pub fn cmd_chipid<C: ShellConfig>(_args: &[&str]) -> Result<Response<C>, CliError> {
    // STM32F0 unique ID register addresses (reference manual section 24.1)
    const UID_BASE: u32 = 0x1FFF_F7AC;

    // Read the 96-bit unique ID (three 32-bit words)
    let uid0 = unsafe { core::ptr::read_volatile(UID_BASE as *const u32) };
    let uid1 = unsafe { core::ptr::read_volatile((UID_BASE + 4) as *const u32) };
    let uid2 = unsafe { core::ptr::read_volatile((UID_BASE + 8) as *const u32) };

    let mut msg = heapless::String::<128>::new();
    write!(msg, "Unique ID: {:08X}{:08X}{:08X}", uid2, uid1, uid0).ok();

    Ok(Response::success(&msg).indented())
}

// =============================================================================
// Clock Frequencies Command
// =============================================================================

/// Display current system clock frequencies
///
/// Shows the configured frequencies for various clock domains in the STM32F072.
/// Values are calculated from RCC (Reset and Clock Control) registers.
pub fn cmd_clocks<C: ShellConfig>(_args: &[&str]) -> Result<Response<C>, CliError> {
    // RCC register base address
    const RCC_BASE: u32 = 0x4002_1000;
    const RCC_CFGR: u32 = RCC_BASE + 0x04; // Clock configuration register
    const RCC_CR: u32 = RCC_BASE + 0x00; // Clock control register

    // Read RCC registers
    let cfgr = unsafe { core::ptr::read_volatile(RCC_CFGR as *const u32) };
    let cr = unsafe { core::ptr::read_volatile(RCC_CR as *const u32) };

    // Determine system clock source (CFGR bits [3:2])
    let sws = (cfgr >> 2) & 0b11;
    let sysclk_src = match sws {
        0b00 => "HSI (8 MHz RC)",
        0b01 => "HSE (External)",
        0b10 => "PLL",
        _ => "Unknown",
    };

    // Determine if HSI48 is enabled and ready (CR bits [17] and [16])
    let hsi48_on = (cr >> 16) & 1;
    let hsi48_rdy = (cr >> 17) & 1;

    // For STM32F072, typical configuration from our init:
    // - System clock: 48 MHz (from HSI48)
    // - AHB: Same as system clock (HPRE = 1)
    // - APB: Same as AHB (PPRE = 1)

    // Read AHB prescaler (CFGR bits [7:4])
    let hpre = (cfgr >> 4) & 0xF;
    let ahb_div = match hpre {
        0b0000..=0b0111 => 1,
        0b1000 => 2,
        0b1001 => 4,
        0b1010 => 8,
        0b1011 => 16,
        0b1100 => 64,
        0b1101 => 128,
        0b1110 => 256,
        0b1111 => 512,
        _ => 1,
    };

    // Read APB prescaler (CFGR bits [10:8])
    let ppre = (cfgr >> 8) & 0b111;
    let apb_div = match ppre {
        0b000..=0b011 => 1,
        0b100 => 2,
        0b101 => 4,
        0b110 => 8,
        0b111 => 16,
        _ => 1,
    };

    // System clock is 48 MHz (configured in hw_setup.rs)
    let sysclk_mhz = 48;
    let ahb_mhz = sysclk_mhz / ahb_div;
    let apb_mhz = ahb_mhz / apb_div;

    let mut msg = heapless::String::<256>::new();
    write!(msg, "Clock Frequencies:\r\n").ok();
    write!(msg, "  Source:  {}\r\n", sysclk_src).ok();
    write!(msg, "  SYSCLK:  {} MHz\r\n", sysclk_mhz).ok();
    write!(msg, "  AHB:     {} MHz\r\n", ahb_mhz).ok();
    write!(msg, "  APB:     {} MHz\r\n", apb_mhz).ok();

    if hsi48_on != 0 && hsi48_rdy != 0 {
        write!(msg, "  HSI48:   48 MHz (Ready)").ok();
    } else {
        write!(msg, "  HSI48:   Disabled").ok();
    }

    Ok(Response::success(&msg).indented())
}

// =============================================================================
// CPU Core Information Command
// =============================================================================

/// Display CPU core information
///
/// The STM32F072 uses an ARM Cortex-M0 core (single core).
pub fn cmd_core<C: ShellConfig>(_args: &[&str]) -> Result<Response<C>, CliError> {
    // Read CPUID register from System Control Block (SCB)
    const SCB_CPUID: u32 = 0xE000_ED00;
    let cpuid = unsafe { core::ptr::read_volatile(SCB_CPUID as *const u32) };

    // Extract fields from CPUID register
    let implementer = (cpuid >> 24) & 0xFF;
    let variant = (cpuid >> 20) & 0xF;
    let architecture = (cpuid >> 16) & 0xF;
    let partno = (cpuid >> 4) & 0xFFF;
    let revision = cpuid & 0xF;

    let mut msg = heapless::String::<256>::new();
    write!(msg, "CPU Core Information:\r\n").ok();
    write!(msg, "  Core:        ARM Cortex-M0\r\n").ok();
    write!(msg, "  Implementer: 0x{:02X} (ARM)\r\n", implementer).ok();
    write!(msg, "  Part:        0x{:03X}\r\n", partno).ok();
    write!(msg, "  Variant:     {}\r\n", variant).ok();
    write!(msg, "  Revision:    {}\r\n", revision).ok();
    write!(msg, "  Arch:        ARMv{}", architecture).ok();

    Ok(Response::success(&msg).indented())
}

// =============================================================================
// Boot/Reset Reason Command
// =============================================================================

/// Display reset reason information
///
/// Reads the RCC_CSR (Clock Control & Status Register) to determine what
/// caused the last system reset.
pub fn cmd_bootreason<C: ShellConfig>(_args: &[&str]) -> Result<Response<C>, CliError> {
    // RCC_CSR register address (reference manual section 7.3.21)
    const RCC_CSR: u32 = 0x4002_1024;

    // Read the CSR register
    let csr = unsafe { core::ptr::read_volatile(RCC_CSR as *const u32) };

    let mut msg = heapless::String::<256>::new();
    write!(msg, "Reset Reason:\r\n").ok();

    // Check reset flags (bits [31:24])
    let mut found_flag = false;

    // Bit 31: LPWRRSTF - Low-power reset flag
    if (csr >> 31) & 1 != 0 {
        write!(msg, "  [x] Low-Power Reset\r\n").ok();
        found_flag = true;
    }

    // Bit 30: WWDGRSTF - Window watchdog reset flag
    if (csr >> 30) & 1 != 0 {
        write!(msg, "  [x] Window Watchdog Reset\r\n").ok();
        found_flag = true;
    }

    // Bit 29: IWDGRSTF - Independent watchdog reset flag
    if (csr >> 29) & 1 != 0 {
        write!(msg, "  [x] Independent Watchdog Reset\r\n").ok();
        found_flag = true;
    }

    // Bit 28: SFTRSTF - Software reset flag
    if (csr >> 28) & 1 != 0 {
        write!(msg, "  [x] Software Reset\r\n").ok();
        found_flag = true;
    }

    // Bit 27: PORRSTF - Power-on/power-down reset flag
    if (csr >> 27) & 1 != 0 {
        write!(msg, "  [x] Power-On Reset\r\n").ok();
        found_flag = true;
    }

    // Bit 26: PINRSTF - NRST pin reset flag
    if (csr >> 26) & 1 != 0 {
        write!(msg, "  [x] Pin Reset (NRST)\r\n").ok();
        found_flag = true;
    }

    // Bit 25: OBLRSTF - Option byte loader reset flag
    if (csr >> 25) & 1 != 0 {
        write!(msg, "  [x] Option Byte Reset\r\n").ok();
        found_flag = true;
    }

    if !found_flag {
        write!(msg, "  [No flags set - First boot]\r\n").ok();
    }

    // Remove new line from last line for better formatting
    msg.pop();
    msg.pop();

    Ok(Response::success(&msg).indented())
}
