//! RP2040 (Raspberry Pi Pico) UART example
//!
//! This example demonstrates nut-shell running on RP2040 hardware with UART communication.
//!
//! # Hardware Setup
//! - UART TX: GP0
//! - UART RX: GP1
//! - Baud rate: 115200
//!
//! # Building
//! ```bash
//! cd examples/rp-pico
//! cargo build --release --bin uart_cli
//! ```
//!
//! # Flashing
//! ```bash
//! # Using picotool
//! picotool load -x target/thumbv6m-none-eabi/release/uart_cli
//!
//! # Or using elf2uf2-rs
//! elf2uf2-rs target/thumbv6m-none-eabi/release/uart_cli uart_cli.uf2
//! # Then copy uart_cli.uf2 to the RPI-RP2 drive
//! ```
//!
//! # Connecting
//! Connect to the serial port at 115200 baud:
//! ```bash
//! # Linux
//! screen /dev/ttyACM0 115200
//!
//! # macOS
//! screen /dev/tty.usbmodem* 115200
#![no_std]
#![no_main]

use core::fmt::Write;
use cortex_m::delay::Delay;
use cortex_m_rt::entry;
use fugit::HertzU32;
use heapless;
use panic_halt as _;

// Link in the Boot ROM - required for RP2040
#[unsafe(link_section = ".boot2")]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

use rp2040_hal::{
    clocks::{init_clocks_and_plls, Clock},
    gpio::{FunctionUart, Pin, PullDown},
    pac,
    uart::{DataBits, StopBits, UartConfig, UartPeripheral},
    watchdog::Watchdog,
    Sio,
};

use nut_shell::{
    auth::AccessLevel,
    config::DefaultConfig,
    io::CharIo,
    response::Response,
    shell::{handlers::CommandHandlers, Shell},
    tree::{CommandKind, CommandMeta, Directory, Node},
    CliError,
};

use nut_shell::auth::{PasswordHasher, Sha256Hasher, User};

// =============================================================================
// Access Level Definition
// =============================================================================

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PicoAccessLevel {
    User = 0,
    Admin = 1,
}

impl AccessLevel for PicoAccessLevel {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "User" => Some(Self::User),
            "Admin" => Some(Self::Admin),
            _ => None,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::User => "User",
            Self::Admin => "Admin",
        }
    }
}

// =============================================================================
// Command Tree Definition (Minimal for embedded)
// =============================================================================

const CMD_LED: CommandMeta<PicoAccessLevel> = CommandMeta {
    name: "led",
    description: "Toggle onboard LED",
    access_level: PicoAccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 1,
    max_args: 1,
};

const CMD_INFO: CommandMeta<PicoAccessLevel> = CommandMeta {
    name: "info",
    description: "Show device information",
    access_level: PicoAccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

const CMD_REBOOT: CommandMeta<PicoAccessLevel> = CommandMeta {
    name: "reboot",
    description: "Reboot the device",
    access_level: PicoAccessLevel::Admin,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

const SYSTEM_DIR: Directory<PicoAccessLevel> = Directory {
    name: "system",
    children: &[Node::Command(&CMD_REBOOT), Node::Command(&CMD_INFO)],
    access_level: PicoAccessLevel::User,
};

const ROOT: Directory<PicoAccessLevel> = Directory {
    name: "/",
    children: &[Node::Directory(&SYSTEM_DIR), Node::Command(&CMD_LED)],
    access_level: PicoAccessLevel::User,
};

// =============================================================================
// Command Handlers
// =============================================================================

struct PicoHandlers;

impl CommandHandlers<DefaultConfig> for PicoHandlers {
    fn execute_sync(
        &self,
        name: &str,
        args: &[&str],
    ) -> Result<Response<DefaultConfig>, CliError> {
        match name {
            "led" => {
                let state = args[0];
                match state {
                    "on" | "off" => {
                        // In a real implementation, you would control the LED here
                        // For now, just acknowledge the command
                        let mut msg = heapless::String::<128>::new();
                        write!(msg, "LED turned {}", state).ok();
                        Ok(Response::success(&msg))
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
            "info" => {
                let mut msg = heapless::String::<256>::new();
                write!(msg, "Device: Raspberry Pi Pico\r\n").ok();
                write!(msg, "Chip: RP2040\r\n").ok();
                write!(msg, "Firmware: nut-shell v0.1.0\r\n").ok();
                write!(msg, "UART: GP0(TX)/GP1(RX) @ 115200").ok();
                Ok(Response::success(&msg))
            }
            "reboot" => {
                // In a real implementation, trigger watchdog reset
                Ok(Response::success("Rebooting...\r\n(Not implemented in example)"))
            }
            _ => Err(CliError::CommandNotFound),
        }
    }
}

// =============================================================================
// Credential Provider
// =============================================================================

struct PicoCredentialProvider {
    users: [User<PicoAccessLevel>; 2],
    hasher: Sha256Hasher,
}

impl PicoCredentialProvider {
    fn new() -> Self {
        let hasher = Sha256Hasher;

        // Create users with hashed passwords
        let admin_salt: [u8; 16] = [1; 16];
        let user_salt: [u8; 16] = [2; 16];

        let admin_hash = hasher.hash("pico123", &admin_salt);
        let user_hash = hasher.hash("pico456", &user_salt);

        let mut admin_username = heapless::String::new();
        admin_username.push_str("admin").unwrap();
        let admin = User {
            username: admin_username,
            access_level: PicoAccessLevel::Admin,
            password_hash: admin_hash,
            salt: admin_salt,
        };

        let mut user_username = heapless::String::new();
        user_username.push_str("user").unwrap();
        let user = User {
            username: user_username,
            access_level: PicoAccessLevel::User,
            password_hash: user_hash,
            salt: user_salt,
        };

        Self {
            users: [admin, user],
            hasher,
        }
    }
}

impl nut_shell::auth::CredentialProvider<PicoAccessLevel> for PicoCredentialProvider {
    type Error = ();

    fn find_user(&self, username: &str) -> Result<Option<User<PicoAccessLevel>>, Self::Error> {
        Ok(self
            .users
            .iter()
            .find(|u| u.username.as_str() == username)
            .cloned())
    }

    fn verify_password(&self, user: &User<PicoAccessLevel>, password: &str) -> bool {
        self.hasher.verify(password, &user.salt, &user.password_hash)
    }

    fn list_users(&self) -> Result<heapless::Vec<&str, 32>, Self::Error> {
        let mut list = heapless::Vec::new();
        for user in &self.users {
            list.push(user.username.as_str()).ok();
        }
        Ok(list)
    }
}

// =============================================================================
// UART CharIo Implementation
// =============================================================================

type UartPins = (
    Pin<rp2040_hal::gpio::bank0::Gpio0, FunctionUart, PullDown>,
    Pin<rp2040_hal::gpio::bank0::Gpio1, FunctionUart, PullDown>,
);
type UartType = UartPeripheral<rp2040_hal::uart::Enabled, pac::UART0, UartPins>;

struct UartCharIo {
    uart: UartType,
}

impl UartCharIo {
    fn new(uart: UartType) -> Self {
        Self { uart }
    }
}

impl CharIo for UartCharIo {
    type Error = ();

    fn get_char(&mut self) -> Result<Option<char>, Self::Error> {
        // Non-blocking read
        if self.uart.uart_is_readable() {
            let mut buf = [0u8; 1];
            match self.uart.read_raw(&mut buf) {
                Ok(n) if n > 0 => Ok(Some(buf[0] as char)),
                Ok(_) => Ok(None),
                Err(_) => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    fn put_char(&mut self, c: char) -> Result<(), Self::Error> {
        // Blocking write for simplicity
        self.uart.write_full_blocking(&[c as u8]);
        Ok(())
    }

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        self.uart.write_full_blocking(s.as_bytes());
        Ok(())
    }
}

// =============================================================================
// Main Entry Point
// =============================================================================

#[entry]
fn main() -> ! {
    // Get peripheral access
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    // Set up watchdog
    let mut watchdog = Watchdog::new(pac.WATCHDOG);

    // Configure clocks
    let xosc_crystal_freq = 12_000_000; // 12 MHz crystal on Pico
    let clocks = init_clocks_and_plls(
        xosc_crystal_freq,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    // Set up delay
    let mut delay = Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    // Set up GPIO
    let sio = Sio::new(pac.SIO);
    let pins = rp2040_hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Configure UART on GP0 (TX) and GP1 (RX)
    let uart_pins = (
        pins.gpio0.into_function::<FunctionUart>(),
        pins.gpio1.into_function::<FunctionUart>(),
    );

    let uart = UartPeripheral::new(pac.UART0, uart_pins, &mut pac.RESETS)
        .enable(
            UartConfig::new(
                HertzU32::from_raw(115200),
                DataBits::Eight,
                None,
                StopBits::One,
            ),
            clocks.peripheral_clock.freq(),
        )
        .unwrap();

    // Create CharIo wrapper
    let io = UartCharIo::new(uart);

    // Create handlers
    let handlers = PicoHandlers;

    // Create shell with authentication
    let provider = PicoCredentialProvider::new();
    let mut shell: Shell<PicoAccessLevel, UartCharIo, PicoHandlers, DefaultConfig> =
        Shell::new(&ROOT, handlers, &provider, io);

    // Activate shell (show welcome and prompt)
    shell.activate().ok();

    // Main polling loop
    // The shell.poll() method checks for incoming UART characters and processes them
    loop {
        // Poll for incoming characters and process them
        shell.poll().ok();

        // Small delay to prevent busy-waiting and reduce CPU usage
        delay.delay_us(100u32);
    }
}
