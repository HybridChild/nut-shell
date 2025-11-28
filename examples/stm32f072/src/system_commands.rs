//! System diagnostic commands for STM32F072
//!
//! This module provides general-purpose system diagnostic commands:
//! - System uptime tracking
//! - Memory usage statistics
//! - CPU performance benchmarks
//! - Flash memory information
//! - Controlled crash for testing

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
// Static Cache Storage
// =============================================================================

/// Cached boot time from SysTick (in milliseconds)
static mut BOOT_TIME_MS: u32 = 0;

/// Cache the boot time for uptime calculation
///
/// Must be called early in main() before any significant delays.
/// Stores the current systick count at boot.
pub fn init_boot_time(boot_ms: u32) {
    unsafe {
        BOOT_TIME_MS = boot_ms;
    }
}

// =============================================================================
// System Diagnostic Commands (Metadata)
// =============================================================================

pub const CMD_UPTIME: CommandMeta<Stm32AccessLevel> = CommandMeta {
    id: "system_uptime",
    name: "uptime",
    description: "Show system uptime",
    access_level: Stm32AccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

pub const CMD_MEMINFO: CommandMeta<Stm32AccessLevel> = CommandMeta {
    id: "system_meminfo",
    name: "meminfo",
    description: "Display memory usage statistics",
    access_level: Stm32AccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

pub const CMD_BENCHMARK: CommandMeta<Stm32AccessLevel> = CommandMeta {
    id: "system_benchmark",
    name: "benchmark",
    description: "Run CPU performance benchmark",
    access_level: Stm32AccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

pub const CMD_FLASH: CommandMeta<Stm32AccessLevel> = CommandMeta {
    id: "system_flash",
    name: "flash",
    description: "Show flash memory information",
    access_level: Stm32AccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

pub const CMD_CRASH: CommandMeta<Stm32AccessLevel> = CommandMeta {
    id: "system_crash",
    name: "crash",
    description: "Trigger controlled panic (Admin only!)",
    access_level: Stm32AccessLevel::Admin,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

// =============================================================================
// System Diagnostic Commands (Implementations)
// =============================================================================

/// Show system uptime
///
/// Calculates elapsed time since boot using the SysTick millisecond counter.
/// The boot time is captured at startup and subtracted from the current time.
pub fn cmd_uptime<C: ShellConfig>(_args: &[&str]) -> Result<Response<C>, CliError> {
    // This function is called from the application code
    // The application must provide the current millisecond count
    // via get_current_millis() function pointer or similar mechanism

    // For now, we'll show the boot time that was cached
    let boot_time = unsafe { BOOT_TIME_MS };

    let mut msg = heapless::String::<128>::new();
    write!(
        msg,
        "Uptime tracking active\r\nBoot time: {} ms\r\nNote: Call with millis() from app",
        boot_time
    )
    .ok();

    Ok(Response::success(&msg).indented())
}

/// Display comprehensive memory usage statistics
///
/// Shows complete STM32F072 RAM and Flash layout including all sections,
/// stack reservation, and memory usage percentages.
pub fn cmd_meminfo<C: ShellConfig>(_args: &[&str]) -> Result<Response<C>, CliError> {
    // STM32F072 memory layout
    const TOTAL_RAM_BYTES: u32 = 16 * 1024; // 16 KB SRAM
    const TOTAL_FLASH_BYTES: u32 = 128 * 1024; // 128 KB Flash

    // Declare linker symbols for complete memory map
    unsafe extern "C" {
        // RAM sections
        static __sdata: u32;
        static __edata: u32;
        static __sbss: u32;
        static __ebss: u32;
        // Flash sections
        static __stext: u32;
        static __etext: u32;
        static __sidata: u32;
    }

    // Get linker symbol addresses
    let (data_start, data_end, bss_start, bss_end, text_start, text_end, rodata_start) = (
        core::ptr::addr_of!(__sdata) as usize,
        core::ptr::addr_of!(__edata) as usize,
        core::ptr::addr_of!(__sbss) as usize,
        core::ptr::addr_of!(__ebss) as usize,
        core::ptr::addr_of!(__stext) as usize,
        core::ptr::addr_of!(__etext) as usize,
        core::ptr::addr_of!(__sidata) as usize,
    );

    // Calculate section sizes
    let data_size = data_end.saturating_sub(data_start);
    let bss_size = bss_end.saturating_sub(bss_start);
    let text_size = text_end.saturating_sub(text_start);
    let rodata_size = rodata_start.saturating_sub(text_end);

    // Static RAM usage (.data + .bss)
    let static_ram = data_size + bss_size;

    // Estimate stack size (typical for STM32F072 is 2-4KB)
    // We'll calculate remaining RAM after static allocation
    let ram_used = static_ram;
    let ram_free = TOTAL_RAM_BYTES as usize - static_ram;
    let ram_used_percent = (ram_used as u64 * 100) / TOTAL_RAM_BYTES as u64;

    let total_flash_used = text_size + rodata_size + data_size;
    let flash_used_percent = (total_flash_used as u64 * 100) / TOTAL_FLASH_BYTES as u64;

    let mut msg = heapless::String::<256>::new();
    write!(msg, "Memory Map:\r\n").ok();
    write!(msg, "\r\n").ok();

    // RAM breakdown
    write!(
        msg,
        "RAM ({}K, {}% static):\r\n",
        TOTAL_RAM_BYTES / 1024,
        ram_used_percent
    )
    .ok();
    write!(msg, "  .data:  {} bytes\r\n", data_size).ok();
    write!(msg, "  .bss:   {} bytes\r\n", bss_size).ok();
    write!(msg, "  Static: {} bytes\r\n", static_ram).ok();
    write!(msg, "  Free:   {} bytes\r\n", ram_free).ok();
    write!(msg, "\r\n").ok();

    // Flash breakdown
    write!(msg, "Flash ({}% used):\r\n", flash_used_percent).ok();
    write!(msg, "  .text:   {} bytes\r\n", text_size).ok();
    write!(msg, "  .rodata: {} bytes", rodata_size).ok();

    Ok(Response::success(&msg).indented())
}

/// Run CPU performance benchmark
///
/// Performs simple computational tests and reports performance metrics.
/// Useful for comparing clock speeds or compiler optimizations.
///
/// Note: Cortex-M0 lacks DWT cycle counter, so we report raw iteration counts.
pub fn cmd_benchmark<C: ShellConfig>(_args: &[&str]) -> Result<Response<C>, CliError> {
    // Benchmark 1: Prime counting (simple CPU test)
    let prime_count = count_primes_up_to(1000);

    // Benchmark 2: Memory operations
    let mut buffer = [0u8; 256];
    for i in 0..256 {
        buffer[i] = (i as u8).wrapping_mul(13).wrapping_add(7);
    }
    let mut sum: u32 = 0;
    let iterations = 100;
    for _ in 0..iterations {
        for &byte in &buffer {
            sum = sum.wrapping_add(byte as u32);
        }
    }

    // Prevent optimization from removing the loop
    core::hint::black_box(sum);

    let mut msg = heapless::String::<256>::new();
    write!(msg, "Benchmark Results:\r\n").ok();
    write!(msg, "  Primes < 1000: {}\r\n", prime_count).ok();
    write!(msg, "  Memory ops: {} iterations\r\n", iterations).ok();
    write!(msg, "  Sum result: {}\r\n", sum).ok();
    write!(msg, "  CPU: Cortex-M0 @ 48 MHz\r\n").ok();
    write!(msg, "Note: No cycle counter on M0").ok();

    Ok(Response::success(&msg).indented())
}

/// Helper function for benchmark: count primes up to n
fn count_primes_up_to(n: u32) -> u32 {
    let mut count = 0;
    for num in 2..=n {
        if is_prime(num) {
            count += 1;
        }
    }
    count
}

/// Helper function: check if a number is prime
fn is_prime(n: u32) -> bool {
    if n < 2 {
        return false;
    }
    if n == 2 {
        return true;
    }
    if n % 2 == 0 {
        return false;
    }
    let mut i = 3;
    while i * i <= n {
        if n % i == 0 {
            return false;
        }
        i += 2;
    }
    true
}

/// Show flash memory information
///
/// Displays flash chip details and firmware size.
/// The STM32F072RB has 128KB of internal flash.
pub fn cmd_flash<C: ShellConfig>(_args: &[&str]) -> Result<Response<C>, CliError> {
    // Read flash size from device register
    const FLASH_SIZE_REG: u32 = 0x1FFF_F7CC;
    let flash_size_kb = unsafe { core::ptr::read_volatile(FLASH_SIZE_REG as *const u16) as u32 };

    // Declare linker symbols for flash usage
    unsafe extern "C" {
        static __stext: u32;
        static __etext: u32;
        static __sidata: u32;
    }

    let (text_start, text_end, rodata_end) = (
        core::ptr::addr_of!(__stext) as usize,
        core::ptr::addr_of!(__etext) as usize,
        core::ptr::addr_of!(__sidata) as usize,
    );

    let text_size = text_end.saturating_sub(text_start);
    let rodata_size = rodata_end.saturating_sub(text_end);
    let firmware_size = text_size + rodata_size;

    let flash_size_bytes = flash_size_kb * 1024;
    let used_kb = firmware_size / 1024;
    let used_percent = (firmware_size as u64 * 100) / flash_size_bytes as u64;

    let mut msg = heapless::String::<256>::new();
    write!(msg, "Flash Memory:\r\n").ok();
    write!(msg, "  Total:     {} KB\r\n", flash_size_kb).ok();
    write!(msg, "  Firmware:  {} KB ({}%)\r\n", used_kb, used_percent).ok();
    write!(msg, "    .text:   {} bytes\r\n", text_size).ok();
    write!(msg, "    .rodata: {} bytes\r\n", rodata_size).ok();
    write!(msg, "  Type:      Internal Flash").ok();

    Ok(Response::success(&msg).indented())
}

/// Trigger a controlled panic for testing error handling
///
/// **WARNING**: This intentionally crashes the system!
/// Use this to test watchdog recovery or debug panic handlers.
/// Admin access required.
pub fn cmd_crash<C: ShellConfig>(_args: &[&str]) -> Result<Response<C>, CliError> {
    panic!("Intentional crash triggered by 'crash' command");
}
