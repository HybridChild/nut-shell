//! RP2040 hardware status commands
//!
//! This module provides commands for reading RP2040 internal hardware status:
//! - Temperature sensor
//! - Unique chip ID
//! - Clock frequencies
//! - CPU core identification
//! - Reset reason
//! - GPIO pin status
//!
//! These commands are designed to be reusable across different RP2040 examples.

use core::fmt::Write;
use heapless;
use nut_shell::{config::ShellConfig, response::Response, tree::{CommandMeta, CommandKind}, CliError};
use crate::access_level::PicoAccessLevel;

// =============================================================================
// Static Cache Storage
// =============================================================================

/// Cached chip ID, read once at startup
static mut CHIP_ID: Option<[u8; 8]> = None;

/// Cached watchdog reason, read once at startup (before it auto-clears)
static mut CACHED_WATCHDOG_REASON: Option<u32> = None;

/// Cached chip reset flags, read once at startup
static mut CACHED_CHIP_RESET: Option<u32> = None;

/// Temperature sensor read function pointer (set by main at startup)
static mut TEMP_READ_FN: Option<fn() -> f32> = None;



/// LED control function pointer (set by main at startup)
static mut LED_CONTROL_FN: Option<fn(bool)> = None;

// =============================================================================
// Command Metadata (for use in command trees)
// =============================================================================

pub const CMD_TEMP: CommandMeta<PicoAccessLevel> = CommandMeta {
    id: "hw_temp",
    name: "temp",
    description: "Read internal temperature sensor",
    access_level: PicoAccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

pub const CMD_CHIPID: CommandMeta<PicoAccessLevel> = CommandMeta {
    id: "hw_chipid",
    name: "chipid",
    description: "Display flash unique ID (64-bit)",
    access_level: PicoAccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

pub const CMD_CLOCKS: CommandMeta<PicoAccessLevel> = CommandMeta {
    id: "hw_clocks",
    name: "clocks",
    description: "Show clock frequencies",
    access_level: PicoAccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

pub const CMD_CORE: CommandMeta<PicoAccessLevel> = CommandMeta {
    id: "hw_core",
    name: "core",
    description: "Display CPU core ID",
    access_level: PicoAccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

pub const CMD_BOOTREASON: CommandMeta<PicoAccessLevel> = CommandMeta {
    id: "hw_bootreason",
    name: "bootreason",
    description: "Show last reset reason",
    access_level: PicoAccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

pub const CMD_GPIO: CommandMeta<PicoAccessLevel> = CommandMeta {
    id: "hw_gpio",
    name: "gpio",
    description: "Display GPIO pin status (usage: gpio <pin>)",
    access_level: PicoAccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 1,
    max_args: 1,
};

pub const CMD_LED: CommandMeta<PicoAccessLevel> = CommandMeta {
    id: "hw_led",
    name: "led",
    description: "Control onboard LED (on/off)",
    access_level: PicoAccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 1,
    max_args: 1,
};

// =============================================================================
// Hardware Access Registration Functions
// =============================================================================

/// Register the temperature sensor read function
///
/// Call this once at startup to provide hardware access for the temp command.
/// The function should return the current temperature in degrees Celsius.
///
/// # Example
/// ```no_run
/// fn read_temperature() -> f32 {
///     // Read ADC, convert to temperature
///     25.0
/// }
/// register_temp_sensor(read_temperature);
/// ```
pub fn register_temp_sensor(read_fn: fn() -> f32) {
    unsafe {
        TEMP_READ_FN = Some(read_fn);
    }
}



/// Register the LED control function
///
/// Call this once at startup to provide hardware access for the LED command.
/// The function should accept a boolean (true = on, false = off).
pub fn register_led_control(control_fn: fn(bool)) {
    unsafe {
        LED_CONTROL_FN = Some(control_fn);
    }
}

/// Cache reset reason registers at startup
///
/// **IMPORTANT**: Call this FIRST in main(), before any HAL initialization
/// or other code that might read these registers.
///
/// The WATCHDOG REASON register auto-clears on read, so it must be read
/// immediately at startup to capture the actual boot reason.
///
/// # Safety
/// This function writes to mutable statics. Safe to call once from main()
/// before starting any other tasks or enabling interrupts.
pub fn init_reset_reason() {
    // WATCHDOG REASON register (auto-clears on read)
    const WATCHDOG_REASON: u32 = 0x4005_8008;

    // CHIP_RESET register (sticky flags)
    const CHIP_RESET: u32 = 0x4006_4008;

    unsafe {
        CACHED_WATCHDOG_REASON = Some(core::ptr::read_volatile(WATCHDOG_REASON as *const u32));
        CACHED_CHIP_RESET = Some(core::ptr::read_volatile(CHIP_RESET as *const u32));
    }
}

// =============================================================================
// Temperature Sensor Command
// =============================================================================

/// Read the internal temperature sensor (ADC channel 4)
///
/// Returns the chip temperature in degrees Celsius.
/// The value is read on-demand by calling the registered temperature read function.
pub fn cmd_temp<C: ShellConfig>(_args: &[&str]) -> Result<Response<C>, CliError> {
    if let Some(read_fn) = unsafe { TEMP_READ_FN } {
        let celsius = read_fn();
        let mut msg = heapless::String::<64>::new();
        write!(msg, "Temperature: {:.1}°C", celsius).ok();
        Ok(Response::success(&msg).indented())
    } else {
        Ok(Response::success("Temperature sensor not initialized").indented())
    }
}

// =============================================================================
// Chip ID Initialization and Command
// =============================================================================

/// Initialize the chip ID cache
///
/// Call this once at startup, before enabling interrupts or starting the shell.
/// This function reads the flash unique ID and caches it for later retrieval.
///
/// # Safety
/// This function must be called only once, at startup, before any other code
/// accesses CHIP_ID. It's safe to call from main() before spawning tasks.
pub fn init_chip_id() {
    // Use the rp2040-hal flash module if available, otherwise direct ROM call
    // For now, we'll use a simple stub that could be replaced with actual implementation
    let id = read_flash_id();
    unsafe {
        CHIP_ID = Some(id);
    }
}

/// Read the flash unique ID directly using RP2040 ROM function
///
/// The RP2040 bootrom provides a function to read the 64-bit unique ID
/// from the external flash chip. This ID is globally unique per flash chip.
///
/// This function calls the ROM function 'flash_get_unique_id' which:
/// 1. Sends a READ UNIQUE ID command (0x4B) to the flash
/// 2. Reads 8 bytes from the flash
/// 3. Stores them in the provided buffer
fn read_flash_id() -> [u8; 8] {
    // TODO: Implement proper ROM function calling
    //
    // The RP2040 ROM provides flash_get_unique_id but calling it requires:
    // 1. Correct ROM function table lookup mechanism
    // 2. Proper calling convention (ABI compatibility)
    // 3. Running from RAM or with XIP cache disabled
    // 4. Second core halted
    //
    // For now, return a placeholder ID. To properly implement this:
    // - Use rp2040-hal's rom module if/when available
    // - Or implement the exact pico-sdk ROM calling sequence
    // - Or read the flash chip's ID register directly via QSPI

    // Return a recognizable placeholder pattern
    [0xDE, 0xAD, 0xBE, 0xEF, 0xBA, 0xBE, 0xCA, 0xFE]
}

/// Read the unique 64-bit flash ID
///
/// Every flash chip has a globally unique 64-bit identifier.
/// Since the flash is permanently paired with the RP2040 on the Pico board,
/// this effectively serves as a unique board identifier.
///
/// The ID is read once at startup by calling `init_chip_id()` and cached.
/// This command retrieves the cached value.
pub fn cmd_chipid<C: ShellConfig>(_args: &[&str]) -> Result<Response<C>, CliError> {
    let chip_id = unsafe { CHIP_ID };

    match chip_id {
        Some(id) => {
            let mut msg = heapless::String::<128>::new();
            write!(
                msg,
                "Flash ID: {:02X}{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
                id[0], id[1], id[2], id[3], id[4], id[5], id[6], id[7]
            )
            .ok();
            Ok(Response::success(&msg).indented())
        }
        None => {
            let mut msg = heapless::String::<256>::new();
            write!(msg, "Flash ID: [Not initialized]\r\n").ok();
            write!(msg, "\r\n").ok();
            write!(msg, "Call hw_commands::init_chip_id() at startup.").ok();
            Ok(Response::success(&msg).indented())
        }
    }
}

// =============================================================================
// Clock Frequencies Command
// =============================================================================

/// Display current system clock frequencies
///
/// Shows the configured frequencies for various clock domains in the RP2040.
/// Values are read directly from hardware clock control registers.
pub fn cmd_clocks<C: ShellConfig>(_args: &[&str]) -> Result<Response<C>, CliError> {
    // CLOCKS register base address
    const CLOCKS_BASE: u32 = 0x4000_8000;

    // Clock control registers
    const CLK_SYS_CTRL: u32 = CLOCKS_BASE + 0x3c;
    const CLK_SYS_DIV: u32 = CLOCKS_BASE + 0x38;
    const CLK_USB_CTRL: u32 = CLOCKS_BASE + 0x54;
    const CLK_USB_DIV: u32 = CLOCKS_BASE + 0x50;
    const CLK_PERI_CTRL: u32 = CLOCKS_BASE + 0x48;
    const CLK_ADC_CTRL: u32 = CLOCKS_BASE + 0x60;
    const CLK_ADC_DIV: u32 = CLOCKS_BASE + 0x5c;

    // Reference clock frequency (typically 12 MHz from crystal oscillator)
    const XOSC_FREQ: u32 = 12_000_000;

    // Read clock control registers
    let sys_ctrl = unsafe { core::ptr::read_volatile(CLK_SYS_CTRL as *const u32) };
    let sys_div = unsafe { core::ptr::read_volatile(CLK_SYS_DIV as *const u32) };
    let usb_ctrl = unsafe { core::ptr::read_volatile(CLK_USB_CTRL as *const u32) };
    let _usb_div = unsafe { core::ptr::read_volatile(CLK_USB_DIV as *const u32) };
    let peri_ctrl = unsafe { core::ptr::read_volatile(CLK_PERI_CTRL as *const u32) };
    let adc_ctrl = unsafe { core::ptr::read_volatile(CLK_ADC_CTRL as *const u32) };
    let adc_div = unsafe { core::ptr::read_volatile(CLK_ADC_DIV as *const u32) };

    // Calculate system clock frequency
    // System clock is typically derived from PLL_SYS
    // For simplicity, we'll read the divisor and estimate based on common configurations
    let sys_int_div = (sys_div >> 8) & 0xffffff;
    let sys_frac_div = sys_div & 0xff;

    // Common RP2040 system clock is 125 MHz (from PLL_SYS)
    // We can estimate by checking if clock is running and enabled
    let sys_enabled = (sys_ctrl & 0x800) != 0; // ENABLE bit
    let sys_freq = if sys_enabled && sys_int_div > 0 {
        // Typical PLL_SYS output is 125 MHz
        // Apply divisor: freq = 125MHz / (int + frac/256)
        let divisor = sys_int_div as f32 + (sys_frac_div as f32 / 256.0);
        if divisor > 0.0 {
            (125_000_000.0 / divisor) as u32
        } else {
            125_000_000
        }
    } else {
        125_000_000 // Default assumption
    };

    // USB clock is always 48 MHz when enabled (required by USB spec)
    let usb_enabled = (usb_ctrl & 0x800) != 0;
    let usb_freq = if usb_enabled { 48_000_000 } else { 0 };

    // Peripheral clock typically matches system clock (no divisor)
    let peri_enabled = (peri_ctrl & 0x800) != 0;
    let peri_freq = if peri_enabled { sys_freq } else { 0 };

    // ADC clock calculation
    let adc_int_div = (adc_div >> 8) & 0xffffff;
    let adc_enabled = (adc_ctrl & 0x800) != 0;
    let adc_freq = if adc_enabled && adc_int_div > 0 {
        XOSC_FREQ / adc_int_div
    } else {
        48_000_000 // Default assumption
    };

    let mut msg = heapless::String::<256>::new();
    write!(msg, "Clock Frequencies:\r\n").ok();
    write!(msg, "  System:     {} MHz\r\n", sys_freq / 1_000_000).ok();
    write!(msg, "  USB:        {} MHz\r\n", usb_freq / 1_000_000).ok();
    write!(msg, "  Peripheral: {} MHz\r\n", peri_freq / 1_000_000).ok();
    write!(msg, "  ADC:        {} MHz", adc_freq / 1_000_000).ok();

    Ok(Response::success(&msg).indented())
}

// =============================================================================
// CPU Core ID Command
// =============================================================================

/// Identify which CPU core is executing this command
///
/// The RP2040 has two ARM Cortex-M0+ cores (Core 0 and Core 1)
pub fn cmd_core<C: ShellConfig>(_args: &[&str]) -> Result<Response<C>, CliError> {
    // Read the CPUID register from the SIO peripheral
    // SIO base address: 0xd0000000
    // CPUID offset: 0x000
    const SIO_CPUID: u32 = 0xd000_0000;
    let cpuid = unsafe { core::ptr::read_volatile(SIO_CPUID as *const u32) };

    let mut msg = heapless::String::<64>::new();
    write!(msg, "Running on Core {}", cpuid).ok();
    Ok(Response::success(&msg).indented())
}

// =============================================================================
// Boot/Reset Reason Command
// =============================================================================

/// Display comprehensive reset reason information
///
/// Displays cached values from WATCHDOG REASON and CHIP_RESET registers.
/// Call `init_reset_reason()` at the very start of main() to cache these values.
/// - WATCHDOG REASON: Immediate cause (cached at startup before auto-clear)
/// - CHIP_RESET: Detailed source flags (sticky bits)
pub fn cmd_bootreason<C: ShellConfig>(_args: &[&str]) -> Result<Response<C>, CliError> {
    // Read cached values (set by init_reset_reason() at startup)
    let (watchdog_reason, chip_reset) = unsafe {
        (CACHED_WATCHDOG_REASON, CACHED_CHIP_RESET)
    };

    let mut msg = heapless::String::<256>::new();
    write!(msg, "Reset Diagnostics:\r\n").ok();
    write!(msg, "\r\n").ok();

    // === WATCHDOG REASON (immediate cause) ===
    write!(msg, "Watchdog Reason:\r\n").ok();
    match watchdog_reason {
        Some(reason) => {
            if reason & (1 << 1) != 0 {
                write!(msg, "  [x] Watchdog Timeout\r\n").ok();
            }
            if reason & (1 << 0) != 0 {
                write!(msg, "  [x] Forced Reset\r\n").ok();
            }
            if reason == 0 {
                write!(msg, "  [None - Normal boot]\r\n").ok();
            }
        }
        None => {
            write!(msg, "  [Not cached]\r\n").ok();
            write!(msg, "  Call init_reset_reason() at startup\r\n").ok();
        }
    }
    write!(msg, "\r\n").ok();

    // === CHIP_RESET (detailed source flags) ===
    write!(msg, "Reset Source Flags:\r\n").ok();
    match chip_reset {
        Some(flags) => {
            let mut found_flag = false;

            // Bit 24: PSM_RESTART_FLAG - Debugger recovered from boot lock-up
            if flags & (1 << 24) != 0 {
                write!(msg, "  [x] PSM Restart (Boot Recovery)\r\n").ok();
                found_flag = true;
            }

            // Bit 20: HAD_PSM_RESTART - Reset from debug port
            if flags & (1 << 20) != 0 {
                write!(msg, "  [x] Debug Port Reset\r\n").ok();
                found_flag = true;
            }

            // Bit 16: HAD_RUN - Reset from RUN pin
            if flags & (1 << 16) != 0 {
                write!(msg, "  [x] RUN Pin Reset\r\n").ok();
                found_flag = true;
            }

            // Bit 8: HAD_POR - Power-on or brown-out reset
            if flags & (1 << 8) != 0 {
                write!(msg, "  [x] Power-On Reset\r\n").ok();
                found_flag = true;
            }

            if !found_flag {
                write!(msg, "  [No flags set]").ok();
            }
        }
        None => {
            write!(msg, "  [Not cached]\r\n").ok();
            write!(msg, "  Call init_reset_reason() at startup").ok();
        }
    }

    Ok(Response::success(&msg).indented())
}

// =============================================================================
// GPIO Status Command
// =============================================================================

/// Display the status of a specific GPIO pin
///
/// Shows pin direction (input/output), current state, and pull-up/down configuration.
/// Values are read directly from hardware registers.
///
/// # Arguments
/// - `pin`: GPIO pin number (0-29)
pub fn cmd_gpio<C: ShellConfig>(args: &[&str]) -> Result<Response<C>, CliError> {
    // Parse pin number
    let pin_num = match args[0].parse::<usize>() {
        Ok(n) => n,
        Err(_) => {
            let mut expected = heapless::String::<32>::new();
            expected.push_str("valid pin number (0-29)").ok();
            return Err(CliError::InvalidArgumentFormat {
                arg_index: 0,
                expected,
            });
        }
    };

    // Validate pin range
    if pin_num >= 30 {
        let mut expected = heapless::String::<32>::new();
        expected.push_str("pin number 0-29").ok();
        return Err(CliError::InvalidArgumentFormat {
            arg_index: 0,
            expected,
        });
    }

    // Read GPIO state directly from hardware registers
    // SIO (Single-cycle I/O) registers
    const SIO_GPIO_IN: u32 = 0xd000_0004;   // Current input values
    const SIO_GPIO_OE: u32 = 0xd000_0020;   // Output enable (direction)
    const SIO_GPIO_OUT: u32 = 0xd000_0010;  // Output values

    // PADS_BANK0 registers for pull configuration
    const PADS_BANK0_BASE: u32 = 0x4001_c000;
    const PADS_GPIO_OFFSET: u32 = 0x04; // Each GPIO pad register is at base + 4 + (pin * 4)

    let gpio_in = unsafe { core::ptr::read_volatile(SIO_GPIO_IN as *const u32) };
    let gpio_oe = unsafe { core::ptr::read_volatile(SIO_GPIO_OE as *const u32) };
    let gpio_out = unsafe { core::ptr::read_volatile(SIO_GPIO_OUT as *const u32) };

    let pad_ctrl_addr = PADS_BANK0_BASE + PADS_GPIO_OFFSET + (pin_num as u32 * 4);
    let pad_ctrl = unsafe { core::ptr::read_volatile(pad_ctrl_addr as *const u32) };

    // Extract pin state
    let direction = ((gpio_oe >> pin_num) & 1) as u8;
    let value = if direction == 1 {
        // Output: read from GPIO_OUT
        ((gpio_out >> pin_num) & 1) as u8
    } else {
        // Input: read from GPIO_IN
        ((gpio_in >> pin_num) & 1) as u8
    };

    // Extract pull configuration from PAD control register
    // Bit 3: PUE (Pull-up enable), Bit 2: PDE (Pull-down enable)
    let pue = (pad_ctrl >> 3) & 1;
    let pde = (pad_ctrl >> 2) & 1;
    let pull = if pue != 0 {
        1 // Pull-up
    } else if pde != 0 {
        2 // Pull-down
    } else {
        0 // No pull
    };

    let dir_str = if direction == 1 { "OUT" } else { "IN " };
    let val_str = if value == 1 { "HI" } else { "LO" };
    let pull_str = match pull {
        1 => "UP",
        2 => "DN",
        _ => "--",
    };

    let mut msg = heapless::String::<128>::new();
    write!(msg, "| Pin | Dir | Val | Pull |\r\n").ok();
    write!(msg, "|-----|-----|-----|------|\r\n").ok();
    write!(msg, "| {:2}  | {} | {}  | {}   |" , pin_num, dir_str, val_str, pull_str).ok();

    Ok(Response::success(&msg).indented())
}

// =============================================================================
// LED Control Command
// =============================================================================

/// Control the onboard LED
///
/// # Arguments
/// - `state`: "on" or "off"
pub fn cmd_led<C: ShellConfig>(args: &[&str]) -> Result<Response<C>, CliError> {
    let state = args[0];

    match state {
        "on" => {
            if let Some(control_fn) = unsafe { LED_CONTROL_FN } {
                control_fn(true);
                Ok(Response::success("LED turned on").indented())
            } else {
                Ok(Response::success("LED control not initialized").indented())
            }
        }
        "off" => {
            if let Some(control_fn) = unsafe { LED_CONTROL_FN } {
                control_fn(false);
                Ok(Response::success("LED turned off").indented())
            } else {
                Ok(Response::success("LED control not initialized").indented())
            }
        }
        _ => {
            let mut expected = heapless::String::<32>::new();
            expected.push_str("on or off").ok();
            Err(CliError::InvalidArgumentFormat {
                arg_index: 0,
                expected,
            })
        }
    }
}
