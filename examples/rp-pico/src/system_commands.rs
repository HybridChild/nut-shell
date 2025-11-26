//! System diagnostic commands
//!
//! This module provides general-purpose system diagnostic commands:
//! - System uptime tracking
//! - Memory usage statistics
//! - CPU performance benchmarks
//! - Flash memory information
//! - Controlled crash for testing
//!
//! While these commands interact with RP2040 hardware (timers, linker symbols),
//! they represent higher-level system diagnostics rather than direct hardware control.

use core::fmt::Write;
use heapless;
use nut_shell::{config::ShellConfig, response::Response, tree::{CommandMeta, CommandKind}, CliError};
use crate::access_level::PicoAccessLevel;

// =============================================================================
// Static Cache Storage
// =============================================================================

/// Cached boot time (timer value at startup in microseconds)
static mut BOOT_TIME_MICROS: u64 = 0;

/// Cache the boot time for uptime calculation
///
/// Must be called early in main() before any significant delays.
/// Reads the RP2040 64-bit microsecond timer and stores it.
pub fn init_boot_time() {
    const TIMER_TIMELR: u32 = 0x4005_4028;
    const TIMER_TIMEHR: u32 = 0x4005_402C;

    unsafe {
        let low = core::ptr::read_volatile(TIMER_TIMELR as *const u32);
        let high = core::ptr::read_volatile(TIMER_TIMEHR as *const u32);
        BOOT_TIME_MICROS = ((high as u64) << 32) | (low as u64);
    }
}

// =============================================================================
// System Diagnostic Commands (Metadata)
// =============================================================================

pub const CMD_UPTIME: CommandMeta<PicoAccessLevel> = CommandMeta {
    id: "system_uptime",
    name: "uptime",
    description: "Show system uptime",
    access_level: PicoAccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

pub const CMD_MEMINFO: CommandMeta<PicoAccessLevel> = CommandMeta {
    id: "system_meminfo",
    name: "meminfo",
    description: "Display memory usage statistics",
    access_level: PicoAccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

pub const CMD_BENCHMARK: CommandMeta<PicoAccessLevel> = CommandMeta {
    id: "system_benchmark",
    name: "benchmark",
    description: "Run CPU performance benchmark",
    access_level: PicoAccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

pub const CMD_FLASH: CommandMeta<PicoAccessLevel> = CommandMeta {
    id: "system_flash",
    name: "flash",
    description: "Show flash memory information",
    access_level: PicoAccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

pub const CMD_CRASH: CommandMeta<PicoAccessLevel> = CommandMeta {
    id: "system_crash",
    name: "crash",
    description: "Trigger controlled panic (Admin only!)",
    access_level: PicoAccessLevel::Admin,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

// =============================================================================
// System Diagnostic Commands (Implementations)
// =============================================================================

/// Show system uptime
///
/// Reads the RP2040 64-bit microsecond timer and calculates elapsed time
/// since boot. The boot time is cached at startup by `init_boot_time()`.
pub fn cmd_uptime<C: ShellConfig>(_args: &[&str]) -> Result<Response<C>, CliError> {
    // Read the RP2040 64-bit microsecond timer
    // TIMER base: 0x40054000
    // TIMELR (lower 32 bits): offset 0x28
    // TIMEHR (upper 32 bits): offset 0x2C
    const TIMER_TIMELR: u32 = 0x4005_4028;
    const TIMER_TIMEHR: u32 = 0x4005_402C;

    let (low, high, boot_time) = unsafe {
        (
            core::ptr::read_volatile(TIMER_TIMELR as *const u32),
            core::ptr::read_volatile(TIMER_TIMEHR as *const u32),
            BOOT_TIME_MICROS,
        )
    };

    let current_micros = ((high as u64) << 32) | (low as u64);
    let uptime_micros = current_micros.saturating_sub(boot_time);

    let seconds = uptime_micros / 1_000_000;
    let minutes = seconds / 60;
    let hours = minutes / 60;
    let days = hours / 24;

    let secs = seconds % 60;
    let mins = minutes % 60;
    let hrs = hours % 24;

    let mut msg = heapless::String::<128>::new();
    write!(
        msg,
        "Uptime: {}d {}h {}m {}s\r\nTotal: {} seconds",
        days, hrs, mins, secs, seconds
    )
    .ok();

    Ok(Response::success(&msg).indented())
}

/// Display comprehensive memory usage statistics
///
/// Shows complete RP2040 RAM and Flash layout including all sections,
/// stack reservation, and memory usage percentages.
pub fn cmd_meminfo<C: ShellConfig>(_args: &[&str]) -> Result<Response<C>, CliError> {
    // RP2040 memory layout
    const TOTAL_RAM_BYTES: u32 = 264 * 1024;  // 264 KB SRAM
    const TOTAL_FLASH_BYTES: u32 = 2 * 1024 * 1024;  // 2 MB Flash

    // Declare linker symbols for complete memory map
    unsafe extern "C" {
        // RAM sections
        static __sdata: u32;
        static __edata: u32;
        static __sbss: u32;
        static __ebss: u32;
        static _stack_start: u32;
        static _stack_end: u32;
        // Flash sections
        static __stext: u32;
        static __etext: u32;
        static __srodata: u32;
        static __erodata: u32;
    }

    // Get all linker symbol addresses
    let (
        // RAM sections
        data_start, data_end,
        bss_start, bss_end,
        stack_start, stack_end,
        // Flash sections
        text_start, text_end,
        rodata_start, rodata_end,
    ) = (
        core::ptr::addr_of!(__sdata) as usize,
        core::ptr::addr_of!(__edata) as usize,
        core::ptr::addr_of!(__sbss) as usize,
        core::ptr::addr_of!(__ebss) as usize,
        core::ptr::addr_of!(_stack_start) as usize,
        core::ptr::addr_of!(_stack_end) as usize,
        core::ptr::addr_of!(__stext) as usize,
        core::ptr::addr_of!(__etext) as usize,
        core::ptr::addr_of!(__srodata) as usize,
        core::ptr::addr_of!(__erodata) as usize,
    );

    // Calculate section sizes
    let data_size = data_end.saturating_sub(data_start);
    let bss_size = bss_end.saturating_sub(bss_start);
    let stack_size = stack_end.saturating_sub(stack_start);
    let text_size = text_end.saturating_sub(text_start);
    let rodata_size = rodata_end.saturating_sub(rodata_start);

    let static_ram = data_size + bss_size;
    let total_ram_used = static_ram + stack_size;
    let ram_free = TOTAL_RAM_BYTES as usize - total_ram_used;
    let ram_used_percent = (total_ram_used as u64 * 100) / TOTAL_RAM_BYTES as u64;

    let total_flash_used = text_size + rodata_size + data_size; // .data stored in flash
    let flash_used_percent = (total_flash_used as u64 * 100) / TOTAL_FLASH_BYTES as u64;

    let mut msg = heapless::String::<256>::new();
    write!(msg, "Memory Map:\r\n").ok();
    write!(msg, "\r\n").ok();

    // RAM breakdown
    write!(msg, "RAM ({}K total, {}% used):\r\n",
           TOTAL_RAM_BYTES / 1024, ram_used_percent).ok();
    write!(msg, "  .data:  {} bytes\r\n", data_size).ok();
    write!(msg, "  .bss:   {} bytes\r\n", bss_size).ok();
    write!(msg, "  Stack:  {} KB\r\n", stack_size / 1024).ok();
    write!(msg, "  Used:   {} KB\r\n", total_ram_used / 1024).ok();
    write!(msg, "  Free:   {} KB\r\n", ram_free / 1024).ok();
    write!(msg, "\r\n").ok();

    // Flash breakdown
    write!(msg, "Flash ({}% used):\r\n", flash_used_percent).ok();
    write!(msg, "  .text:   {} KB\r\n", text_size / 1024).ok();
    write!(msg, "  .rodata: {} KB", rodata_size / 1024).ok();

    Ok(Response::success(&msg).indented())
}

/// Run CPU performance benchmark
///
/// Performs simple computational tests and reports performance metrics.
/// Useful for comparing clock speeds or compiler optimizations.
pub fn cmd_benchmark<C: ShellConfig>(_args: &[&str]) -> Result<Response<C>, CliError> {
    // Read start time
    const TIMER_TIMELR: u32 = 0x4005_4028;
    const TIMER_TIMEHR: u32 = 0x4005_402C;

    let read_timer = || -> u64 {
        unsafe {
            let low = core::ptr::read_volatile(TIMER_TIMELR as *const u32);
            let high = core::ptr::read_volatile(TIMER_TIMEHR as *const u32);
            ((high as u64) << 32) | (low as u64)
        }
    };

    // Benchmark 1: Prime counting (simple CPU test)
    let start = read_timer();
    let prime_count = count_primes_up_to(1000);
    let prime_time = read_timer() - start;

    // Benchmark 2: Memory operations
    let start = read_timer();
    let mut buffer = [0u8; 256];
    for i in 0..256 {
        buffer[i] = (i as u8).wrapping_mul(13).wrapping_add(7);
    }
    let mut sum: u32 = 0;
    for _ in 0..100 {
        for &byte in &buffer {
            sum = sum.wrapping_add(byte as u32);
        }
    }
    let mem_time = read_timer() - start;

    // Prevent optimization from removing the loop
    core::hint::black_box(sum);

    let mut msg = heapless::String::<256>::new();
    write!(msg, "Benchmark Results:\r\n").ok();
    write!(msg, "  Primes < 1000: {} ({} us)\r\n", prime_count, prime_time).ok();
    write!(msg, "  Memory ops: {} us\r\n", mem_time).ok();
    write!(msg, "  CPU: RP2040 Cortex-M0+").ok();

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
/// The RP2040 uses external QSPI flash (typically 2-16MB).
pub fn cmd_flash<C: ShellConfig>(_args: &[&str]) -> Result<Response<C>, CliError> {
    // Declare linker symbols for flash usage
    unsafe extern "C" {
        static __stext: u32;
        static __etext: u32;
        static __srodata: u32;
        static __erodata: u32;
    }

    let (text_start, text_end, rodata_start, rodata_end) = (
        core::ptr::addr_of!(__stext) as usize,
        core::ptr::addr_of!(__etext) as usize,
        core::ptr::addr_of!(__srodata) as usize,
        core::ptr::addr_of!(__erodata) as usize,
    );

    let text_size = text_end.saturating_sub(text_start);
    let rodata_size = rodata_end.saturating_sub(rodata_start);
    let firmware_size = text_size + rodata_size;

    // Standard Pico has 2MB flash
    const FLASH_SIZE_BYTES: u32 = 2 * 1024 * 1024;
    let flash_size_kb = FLASH_SIZE_BYTES / 1024;
    let flash_size_mb = flash_size_kb / 1024;

    let used_kb = firmware_size / 1024;
    let used_percent = (firmware_size as u64 * 100) / FLASH_SIZE_BYTES as u64;

    let mut msg = heapless::String::<256>::new();
    write!(msg, "Flash Memory:\r\n").ok();
    write!(msg, "  Total:     {} MB ({} KB)\r\n", flash_size_mb, flash_size_kb).ok();
    write!(msg, "  Firmware:  {} KB ({}%)\r\n", used_kb, used_percent).ok();
    write!(msg, "    .text:   {} bytes\r\n", text_size).ok();
    write!(msg, "    .rodata: {} bytes\r\n", rodata_size).ok();
    write!(msg, "  Type:      QSPI Flash").ok();

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
