//! STM32H753ZI hardware status commands
//!
//! Commands for reading STM32H753ZI internal hardware status:
//! - Unique chip ID (96-bit UID)
//! - Clock frequencies (from RCC registers)
//! - CPU core identification (Cortex-M7 CPUID)
//! - Reset reason (RCC_RSR flags)
//!
//! All register addresses are from DS12117 (datasheet) and RM0433 (reference manual).

use crate::access_level::H753AccessLevel;
use core::fmt::Write;
use heapless;
use nut_shell::{
    CliError,
    config::ShellConfig,
    response::Response,
    tree::{CommandKind, CommandMeta},
};

// =============================================================================
// Command metadata
// =============================================================================

pub const CMD_CHIPID: CommandMeta<H753AccessLevel> = CommandMeta {
    id: "hw_chipid",
    name: "chipid",
    description: "Display unique device ID (96-bit)",
    access_level: H753AccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

pub const CMD_CLOCKS: CommandMeta<H753AccessLevel> = CommandMeta {
    id: "hw_clocks",
    name: "clocks",
    description: "Show clock frequencies",
    access_level: H753AccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

pub const CMD_CORE: CommandMeta<H753AccessLevel> = CommandMeta {
    id: "hw_core",
    name: "core",
    description: "Display CPU core information",
    access_level: H753AccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

pub const CMD_BOOTREASON: CommandMeta<H753AccessLevel> = CommandMeta {
    id: "hw_bootreason",
    name: "bootreason",
    description: "Show last reset reason",
    access_level: H753AccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

// =============================================================================
// Unique Device ID
// =============================================================================

/// Read the unique 96-bit device identifier.
///
/// STM32H7 UID base: 0x1FF1_E800 (DS12117, section "General-purpose 96-bit unique ID").
/// Three 32-bit words at offsets 0, 4, 8.
pub fn cmd_chipid<C: ShellConfig>(_args: &[&str]) -> Result<Response<C>, CliError> {
    const UID_BASE: u32 = 0x1FF1_E800;

    let uid0 = unsafe { core::ptr::read_volatile(UID_BASE as *const u32) };
    let uid1 = unsafe { core::ptr::read_volatile((UID_BASE + 4) as *const u32) };
    let uid2 = unsafe { core::ptr::read_volatile((UID_BASE + 8) as *const u32) };

    let mut msg = heapless::String::<128>::new();
    write!(msg, "Unique ID: {:08X}{:08X}{:08X}", uid2, uid1, uid0).ok();

    Ok(Response::success(&msg).indented())
}

// =============================================================================
// Clock Frequencies
// =============================================================================

/// Display current system clock frequencies.
///
/// Reads RCC_CFGR (RM0433, Table 358) to determine the active clock source.
/// RCC base: 0x5802_4400, RCC_CFGR at offset 0x10.
pub fn cmd_clocks<C: ShellConfig>(_args: &[&str]) -> Result<Response<C>, CliError> {
    // RCC register addresses (RM0433 chapter 8)
    const RCC_BASE: u32 = 0x5802_4400;
    const RCC_CFGR: u32 = RCC_BASE + 0x10; // Clock configuration register

    let cfgr = unsafe { core::ptr::read_volatile(RCC_CFGR as *const u32) };

    // SWS[2:0] at bits [5:3] — system clock switch status (RM0433 Table 358)
    let sws = (cfgr >> 3) & 0b111;
    let sysclk_src = match sws {
        0b000 => "HSI (64 MHz RC)",
        0b001 => "CSI (4 MHz RC)",
        0b010 => "HSE (8 MHz, from ST-LINK MCO)",
        0b011 => "PLL1 (configured at 200 MHz)",
        _ => "Unknown",
    };

    // AHB prescaler HPRE[3:0] at bits [11:8]
    let hpre = (cfgr >> 8) & 0xF;
    let ahb_div: u32 = match hpre {
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

    // D1CPRE[3:0] at bits [7:4] — D1 domain CPU prescaler (divides SYSCLK → CPU)
    let d1cpre = (cfgr >> 4) & 0xF;
    let cpu_div: u32 = match d1cpre {
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

    // PLL1 output frequency is not readable from a single register without
    // decoding N/M/FRACN — assume the configured 200 MHz for display.
    let sysclk_mhz: u32 = 200;
    let cpu_mhz = sysclk_mhz / cpu_div;
    let hclk_mhz = cpu_mhz / ahb_div;

    let mut msg = heapless::String::<256>::new();
    write!(msg, "Clock Frequencies:\r\n").ok();
    write!(msg, "  Source:  {}\r\n", sysclk_src).ok();
    write!(msg, "  SYSCLK:  {} MHz\r\n", sysclk_mhz).ok();
    write!(msg, "  CPU:     {} MHz\r\n", cpu_mhz).ok();
    write!(msg, "  AHB/HCLK: {} MHz\r\n", hclk_mhz).ok();
    write!(msg, "  USB:     48 MHz (PLL3Q)").ok();

    Ok(Response::success(&msg).indented())
}

// =============================================================================
// CPU Core Information
// =============================================================================

/// Display Cortex-M7 CPUID register information.
pub fn cmd_core<C: ShellConfig>(_args: &[&str]) -> Result<Response<C>, CliError> {
    const SCB_CPUID: u32 = 0xE000_ED00;
    let cpuid = unsafe { core::ptr::read_volatile(SCB_CPUID as *const u32) };

    let implementer = (cpuid >> 24) & 0xFF;
    let variant = (cpuid >> 20) & 0xF;
    let architecture = (cpuid >> 16) & 0xF;
    let partno = (cpuid >> 4) & 0xFFF;
    let revision = cpuid & 0xF;

    let core_name = if partno == 0xC27 {
        "Cortex-M7"
    } else {
        "Unknown"
    };

    let mut msg = heapless::String::<256>::new();
    write!(msg, "CPU Core (STM32H753ZIT6):\r\n").ok();
    write!(msg, "  Core:        ARM {}\r\n", core_name).ok();
    write!(msg, "  Implementer: 0x{:02X} (ARM Ltd)\r\n", implementer).ok();
    write!(msg, "  PartNo:      0x{:03X}\r\n", partno).ok();
    write!(msg, "  Variant:     r{}p{}\r\n", variant, revision).ok();
    write!(msg, "  Arch:        ARMv{}\r\n", architecture).ok();
    write!(msg, "  Max speed:   480 MHz (VOS0)").ok();

    Ok(Response::success(&msg).indented())
}

// =============================================================================
// Reset Reason
// =============================================================================

/// Display reset reason from RCC_RSR (Reset Status Register).
///
/// RCC base: 0x5802_4400, RCC_RSR at offset 0xD0 (RM0433 Table 342).
/// Write 1 to bit 16 (RMVF) to clear all reset flags.
pub fn cmd_bootreason<C: ShellConfig>(_args: &[&str]) -> Result<Response<C>, CliError> {
    // RM0433 Table 342: RCC reset status register
    const RCC_RSR: u32 = 0x5802_44D0;

    let rsr = unsafe { core::ptr::read_volatile(RCC_RSR as *const u32) };

    // Each flag line is ~38 bytes; header is 26. Buffer fits ~6 flags before silent truncation.
    // In practice only 1-2 flags are set per reset, so this is safe.
    let mut msg = heapless::String::<256>::new();
    write!(msg, "Reset Reason (RCC_RSR):\r\n").ok();

    let mut found = false;

    // Flags in RCC_RSR — bit positions per RM0433
    let flags: &[(&str, u32)] = &[
        ("LPWR2RSTF - Low-power D2 reset", (1 << 31)),
        ("LPWR1RSTF - Low-power D1 reset", (1 << 30)),
        ("WWDG2RSTF - Window watchdog 2", (1 << 29)),
        ("WWDG1RSTF - Window watchdog 1", (1 << 28)),
        ("IWDG2RSTF - Ind. watchdog 2", (1 << 27)),
        ("IWDG1RSTF - Ind. watchdog 1", (1 << 26)),
        ("SFTRSTF   - Software reset", (1 << 24)),
        ("PORRSTF   - Power-on reset", (1 << 23)),
        ("PINRSTF   - NRST pin reset", (1 << 22)),
        ("BORRSTF   - BOR reset", (1 << 21)),
        ("D2RSTF    - D2 domain reset", (1 << 20)),
        ("D1RSTF    - D1 domain reset", (1 << 19)),
        ("CPURSTF   - CPU reset", (1 << 17)),
    ];

    for (name, mask) in flags {
        if rsr & mask != 0 {
            write!(msg, "  [x] {}\r\n", name).ok();
            found = true;
        }
    }

    if !found {
        write!(msg, "  [No flags set]").ok();
    } else {
        // Remove trailing \r\n
        msg.pop();
        msg.pop();
    }

    Ok(Response::success(&msg).indented())
}
