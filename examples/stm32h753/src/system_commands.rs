//! System diagnostic commands for STM32H753ZI
//!
//! General-purpose diagnostic commands: uptime, memory info, benchmark, flash info.

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

pub const CMD_UPTIME: CommandMeta<H753AccessLevel> = CommandMeta {
    id: "system_uptime",
    name: "uptime",
    description: "Show system uptime",
    access_level: H753AccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

pub const CMD_MEMINFO: CommandMeta<H753AccessLevel> = CommandMeta {
    id: "system_meminfo",
    name: "meminfo",
    description: "Display memory usage statistics",
    access_level: H753AccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

pub const CMD_BENCHMARK: CommandMeta<H753AccessLevel> = CommandMeta {
    id: "system_benchmark",
    name: "benchmark",
    description: "Run CPU performance benchmark",
    access_level: H753AccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

pub const CMD_FLASH: CommandMeta<H753AccessLevel> = CommandMeta {
    id: "system_flash",
    name: "flash",
    description: "Show flash memory information",
    access_level: H753AccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

pub const CMD_CRASH: CommandMeta<H753AccessLevel> = CommandMeta {
    id: "system_crash",
    name: "crash",
    description: "Trigger controlled panic (Admin only)",
    access_level: H753AccessLevel::Admin,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

// =============================================================================
// Command implementations
// =============================================================================

pub fn cmd_meminfo<C: ShellConfig>(_args: &[&str]) -> Result<Response<C>, CliError> {
    // STM32H753ZIT6 memory layout
    const TOTAL_RAM_BYTES: u32 = 512 * 1024; // 512 KB AXI SRAM (default .data/.bss region)
    const DTCM_BYTES: u32 = 128 * 1024; // 128 KB DTCM (stack only)
    const TOTAL_FLASH_BYTES: u32 = 2 * 1024 * 1024; // 2 MB Flash

    unsafe extern "C" {
        static __sdata: u32;
        static __edata: u32;
        static __sbss: u32;
        static __ebss: u32;
        static __stext: u32;
        static __etext: u32;
        static __sidata: u32;
    }

    let (data_start, data_end, bss_start, bss_end, text_start, text_end, rodata_start) = (
        core::ptr::addr_of!(__sdata) as usize,
        core::ptr::addr_of!(__edata) as usize,
        core::ptr::addr_of!(__sbss) as usize,
        core::ptr::addr_of!(__ebss) as usize,
        core::ptr::addr_of!(__stext) as usize,
        core::ptr::addr_of!(__etext) as usize,
        core::ptr::addr_of!(__sidata) as usize,
    );

    let data_size = data_end.saturating_sub(data_start);
    let bss_size = bss_end.saturating_sub(bss_start);
    let text_size = text_end.saturating_sub(text_start);
    let rodata_size = rodata_start.saturating_sub(text_end);

    let static_ram = data_size + bss_size;
    let ram_free = (TOTAL_RAM_BYTES as usize).saturating_sub(static_ram);
    let ram_used_pct = (static_ram as u64 * 100) / TOTAL_RAM_BYTES as u64;

    let total_flash_used = text_size + rodata_size + data_size;
    let flash_used_pct = (total_flash_used as u64 * 100) / TOTAL_FLASH_BYTES as u64;

    let mut msg = heapless::String::<256>::new();
    write!(msg, "Memory (STM32H753ZIT6):\r\n").ok();
    write!(msg, "\r\n").ok();
    write!(
        msg,
        "AXI SRAM ({}K, {}% static):\r\n",
        TOTAL_RAM_BYTES / 1024,
        ram_used_pct
    )
    .ok();
    write!(msg, "  .data:  {} B\r\n", data_size).ok();
    write!(msg, "  .bss:   {} B\r\n", bss_size).ok();
    write!(msg, "  Free:   {} B\r\n", ram_free).ok();
    write!(
        msg,
        "DTCM: {}K (available; stack is in AXI SRAM)\r\n",
        DTCM_BYTES / 1024
    )
    .ok();
    write!(msg, "\r\n").ok();
    write!(msg, "Flash ({}% used):\r\n", flash_used_pct).ok();
    write!(msg, "  .text:   {} B\r\n", text_size).ok();
    write!(msg, "  .rodata: {} B", rodata_size).ok();

    Ok(Response::success(&msg).indented())
}

pub fn cmd_benchmark<C: ShellConfig>(_args: &[&str]) -> Result<Response<C>, CliError> {
    let prime_count = count_primes_up_to(10_000);

    let mut buffer = [0u8; 256];
    for i in 0..256 {
        buffer[i] = (i as u8).wrapping_mul(13).wrapping_add(7);
    }
    let mut sum: u32 = 0;
    let iterations = 1000;
    for _ in 0..iterations {
        for &byte in &buffer {
            sum = sum.wrapping_add(byte as u32);
        }
    }
    core::hint::black_box(sum);

    let mut msg = heapless::String::<256>::new();
    write!(msg, "Benchmark (Cortex-M7 @ 200 MHz):\r\n").ok();
    write!(msg, "  Primes < 10000: {}\r\n", prime_count).ok();
    write!(msg, "  Memory ops:     {} iterations\r\n", iterations).ok();
    write!(msg, "  Sum result:     {}", sum).ok();

    Ok(Response::success(&msg).indented())
}

fn count_primes_up_to(n: u32) -> u32 {
    let mut count = 0;
    for num in 2..=n {
        if is_prime(num) {
            count += 1;
        }
    }
    count
}

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

pub fn cmd_flash<C: ShellConfig>(_args: &[&str]) -> Result<Response<C>, CliError> {
    // STM32H7 flash size register (DS12117 section "Flash memory")
    // Location: 0x1FF1_E880 — lower 16 bits = flash size in KB
    const FLASH_SIZE_REG: u32 = 0x1FF1_E880;
    let flash_size_kb = unsafe { core::ptr::read_volatile(FLASH_SIZE_REG as *const u16) as u32 };

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
    let used_pct = (firmware_size as u64 * 100) / flash_size_bytes as u64;

    let mut msg = heapless::String::<256>::new();
    write!(msg, "Flash Memory:\r\n").ok();
    write!(msg, "  Total:    {} KB\r\n", flash_size_kb).ok();
    write!(msg, "  Firmware: {} KB ({}%)\r\n", used_kb, used_pct).ok();
    write!(msg, "    .text:   {} B\r\n", text_size).ok();
    write!(msg, "    .rodata: {} B\r\n", rodata_size).ok();
    write!(msg, "  Type:     Dual-bank internal Flash").ok();

    Ok(Response::success(&msg).indented())
}

pub fn cmd_crash<C: ShellConfig>(_args: &[&str]) -> Result<Response<C>, CliError> {
    panic!("Intentional crash triggered by 'crash' command");
}
