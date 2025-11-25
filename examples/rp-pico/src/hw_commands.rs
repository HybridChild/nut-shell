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
    description: "Display unique chip ID",
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

/// Read the flash unique ID directly
///
/// This function must be called very early at startup, ideally before
/// main application code runs.
fn read_flash_id() -> [u8; 8] {
    // Placeholder - in a real implementation, this would:
    // 1. Call into ROM bootloader function
    // 2. Or use rp2040-hal if it provides this
    // For now, return a dummy ID to demonstrate the pattern
    [0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE]
}

/// Read the unique 64-bit chip ID from flash
///
/// Every RP2040 has a globally unique 64-bit identifier stored in its flash memory.
/// This ID is set at manufacturing time and cannot be changed.
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
                "Chip ID: {:02X}{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
                id[0], id[1], id[2], id[3], id[4], id[5], id[6], id[7]
            )
            .ok();
            Ok(Response::success(&msg).indented())
        }
        None => {
            let mut msg = heapless::String::<256>::new();
            write!(msg, "Chip ID: [Not initialized]\r\n").ok();
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

/// Display the reason for the last system reset
///
/// Possible reasons include:
/// - Power-on reset
/// - External reset pin
/// - Watchdog timeout
/// - Debugger reset
pub fn cmd_bootreason<C: ShellConfig>(_args: &[&str]) -> Result<Response<C>, CliError> {
    // Read the CHIP_RESET register from the PSM peripheral
    // PSM base: 0x40010000
    // CHIP_RESET offset: 0x00
    const PSM_CHIP_RESET: u32 = 0x4001_0000;
    let reset_flags = unsafe { core::ptr::read_volatile(PSM_CHIP_RESET as *const u32) };

    let mut msg = heapless::String::<256>::new();
    write!(msg, "Reset Reason:\r\n").ok();

    if reset_flags & (1 << 24) != 0 {
        write!(msg, "  [x] Power-on Reset\r\n").ok();
    }
    if reset_flags & (1 << 20) != 0 {
        write!(msg, "  [x] External Reset (RUN pin)\r\n").ok();
    }
    if reset_flags & (1 << 16) != 0 {
        write!(msg, "  [x] Watchdog Reset\r\n").ok();
    }
    if reset_flags & (1 << 8) != 0 {
        write!(msg, "  [x] Debugger Reset\r\n").ok();
    }

    if reset_flags == 0 {
        write!(msg, "  [Unknown reason]").ok();
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
