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

/// Cached temperature reading (in degrees Celsius)
static mut CACHED_TEMP: Option<f32> = None;

/// Cached clock frequencies (Hz)
static mut CACHED_SYS_CLOCK: u32 = 0;
static mut CACHED_USB_CLOCK: u32 = 0;
static mut CACHED_PERIPHERAL_CLOCK: u32 = 0;
static mut CACHED_ADC_CLOCK: u32 = 0;

/// Cached GPIO state for up to 30 pins (RP2040 has 30 GPIOs)
/// Format: (direction: 0=input, 1=output, value: 0=low, 1=high, pull: 0=none, 1=up, 2=down)
static mut CACHED_GPIO_STATE: [(u8, u8, u8); 30] = [(0, 0, 0); 30];

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
// Cache Update Functions (called by main loop or background tasks)
// =============================================================================

/// Update the cached temperature reading
///
/// Call this periodically from your main loop or a background task.
/// The temperature should be in degrees Celsius.
///
/// # Safety
/// This function writes to a mutable static. In single-threaded embedded
/// environments (like the RP2040 examples), this is safe when called from
/// the main loop or from a single background task.
pub fn update_temperature(temp_celsius: f32) {
    unsafe {
        CACHED_TEMP = Some(temp_celsius);
    }
}

/// Update the cached clock frequencies
///
/// Call this at startup and whenever clock frequencies change.
/// All frequencies should be in Hz.
pub fn update_clocks(sys_hz: u32, usb_hz: u32, peripheral_hz: u32, adc_hz: u32) {
    unsafe {
        CACHED_SYS_CLOCK = sys_hz;
        CACHED_USB_CLOCK = usb_hz;
        CACHED_PERIPHERAL_CLOCK = peripheral_hz;
        CACHED_ADC_CLOCK = adc_hz;
    }
}

/// Update cached GPIO state for a specific pin
///
/// Call this whenever you need to refresh GPIO status.
///
/// # Parameters
/// - `pin`: GPIO pin number (0-29)
/// - `direction`: 0 = input, 1 = output
/// - `value`: 0 = low, 1 = high
/// - `pull`: 0 = none, 1 = pull-up, 2 = pull-down
pub fn update_gpio_state(pin: usize, direction: u8, value: u8, pull: u8) {
    if pin < 30 {
        unsafe {
            CACHED_GPIO_STATE[pin] = (direction, value, pull);
        }
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
/// The value is read from cache, which is updated periodically by the main loop.
pub fn cmd_temp<C: ShellConfig>(_args: &[&str]) -> Result<Response<C>, CliError> {
    let temp = unsafe { CACHED_TEMP };

    match temp {
        Some(celsius) => {
            let mut msg = heapless::String::<64>::new();
            write!(msg, "Temperature: {:.1}°C", celsius).ok();
            Ok(Response::success(&msg).indented())
        }
        None => Ok(Response::success("Temperature: [Not yet measured]").indented()),
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
/// Values are read from cache, updated at startup by the main function.
pub fn cmd_clocks<C: ShellConfig>(_args: &[&str]) -> Result<Response<C>, CliError> {
    let (sys, usb, peripheral, adc) = unsafe {
        (
            CACHED_SYS_CLOCK,
            CACHED_USB_CLOCK,
            CACHED_PERIPHERAL_CLOCK,
            CACHED_ADC_CLOCK,
        )
    };

    let mut msg = heapless::String::<256>::new();
    write!(msg, "Clock Frequencies:\r\n").ok();

    if sys > 0 {
        write!(msg, "  System:     {} MHz\r\n", sys / 1_000_000).ok();
        write!(msg, "  USB:        {} MHz\r\n", usb / 1_000_000).ok();
        write!(msg, "  Peripheral: {} MHz\r\n", peripheral / 1_000_000).ok();
        write!(msg, "  ADC:        {} MHz", adc / 1_000_000).ok();
    } else {
        write!(msg, "  [Not yet initialized]").ok();
    }

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
/// Values are read from cache, which can be updated by calling `update_gpio_state()`.
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

    let gpio_state = unsafe { CACHED_GPIO_STATE };
    let (dir, val, pull) = gpio_state[pin_num];

    let dir_str = if dir == 1 { "OUT" } else { "IN " };
    let val_str = if val == 1 { "HI" } else { "LO" };
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
