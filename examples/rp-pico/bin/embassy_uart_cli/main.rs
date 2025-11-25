//! RP2040 (Raspberry Pi Pico) Embassy UART example with async support
//!
//! This example demonstrates nut-shell running on RP2040 hardware with Embassy async runtime,
//! showcasing async command execution with UART communication.
//!
//! # Hardware Setup
//! - UART TX: GP0
//! - UART RX: GP1
//! - Baud rate: 115200
//!
//! # Building
//! ```bash
//! cd examples/rp-pico
//! cargo build --release --bin embassy_uart_cli
//! ```
//!
//! # Flashing
//! ```bash
//! # Using picotool
//! picotool load -x target/thumbv6m-none-eabi/release/embassy_uart_cli
//!
//! # Or using elf2uf2-rs
//! elf2uf2-rs target/thumbv6m-none-eabi/release/embassy_uart_cli embassy_uart_cli.uf2
//! # Then copy embassy_uart_cli.uf2 to the RPI-RP2 drive
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
//! ```
//!
//! # Features
//! - Embassy async runtime
//! - Async command execution with `process_char_async()`
//! - Buffered UART I/O (deferred flush pattern)
//! - Async delay command demonstration

#![no_std]
#![no_main]

use core::fmt::Write;
use embassy_executor::Spawner;
use embassy_rp::{
    bind_interrupts,
    gpio::{Level, Output},
    peripherals::UART0,
    uart::{self, BufferedInterruptHandler, BufferedUart, BufferedUartRx, BufferedUartTx},
};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};
use embedded_io_async::{Read as AsyncRead, Write as AsyncWrite};
use heapless;
use panic_halt as _;
use static_cell::StaticCell;

use nut_shell::{
    AccessLevel, CliError,
    config::DefaultConfig,
    io::CharIo,
    response::Response,
    shell::{Shell, handlers::CommandHandlers},
    tree::{CommandKind, CommandMeta, Directory, Node},
};

use nut_shell::auth::{PasswordHasher, Sha256Hasher, User};

// Bind UART interrupt handler
bind_interrupts!(struct Irqs {
    UART0_IRQ => BufferedInterruptHandler<UART0>;
});

// =============================================================================
// Access Level Definition
// =============================================================================

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, AccessLevel)]
pub enum PicoAccessLevel {
    User = 0,
    Admin = 1,
}

// =============================================================================
// Command Tree Definition
// =============================================================================

const CMD_LED: CommandMeta<PicoAccessLevel> = CommandMeta {
    id: "led",
    name: "led",
    description: "Toggle onboard LED",
    access_level: PicoAccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 1,
    max_args: 1,
};

const CMD_INFO: CommandMeta<PicoAccessLevel> = CommandMeta {
    id: "system_info",
    name: "info",
    description: "Show device information",
    access_level: PicoAccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

const CMD_DELAY: CommandMeta<PicoAccessLevel> = CommandMeta {
    id: "system_delay",
    name: "delay",
    description: "Async delay demonstration (seconds)",
    access_level: PicoAccessLevel::User,
    kind: CommandKind::Async,
    min_args: 1,
    max_args: 1,
};

const CMD_REBOOT: CommandMeta<PicoAccessLevel> = CommandMeta {
    id: "system_reboot",
    name: "reboot",
    description: "Reboot the device",
    access_level: PicoAccessLevel::Admin,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

const SYSTEM_DIR: Directory<PicoAccessLevel> = Directory {
    name: "system",
    children: &[
        Node::Command(&CMD_REBOOT),
        Node::Command(&CMD_INFO),
        Node::Command(&CMD_DELAY),
    ],
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

struct PicoHandlers {
    led_channel: &'static Channel<ThreadModeRawMutex, LedCommand, 1>,
}

enum LedCommand {
    On,
    Off,
}

impl CommandHandlers<DefaultConfig> for PicoHandlers {
    fn execute_sync(&self, id: &str, args: &[&str]) -> Result<Response<DefaultConfig>, CliError> {
        match id {
            "led" => {
                let state = args[0];
                match state {
                    "on" => {
                        self.led_channel.try_send(LedCommand::On).ok();
                        Ok(Response::success("LED turned on"))
                    }
                    "off" => {
                        self.led_channel.try_send(LedCommand::Off).ok();
                        Ok(Response::success("LED turned off"))
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
            "system_info" => {
                let mut msg = heapless::String::<256>::new();
                write!(msg, "Device: Raspberry Pi Pico\r\n").ok();
                write!(msg, "Chip: RP2040\r\n").ok();
                write!(msg, "Runtime: Embassy\r\n").ok();
                write!(msg, "Firmware: nut-shell v0.1.0\r\n").ok();
                write!(msg, "UART: GP0(TX)/GP1(RX) @ 115200").ok();
                Ok(Response::success(&msg))
            }
            "system_reboot" => {
                // In a real implementation, trigger watchdog reset
                Ok(Response::success(
                    "Rebooting...\r\n(Not implemented in example)",
                ))
            }
            _ => Err(CliError::CommandNotFound),
        }
    }

    async fn execute_async(&self, id: &str, args: &[&str]) -> Result<Response<DefaultConfig>, CliError> {
        match id {
            "system_delay" => {
                // Parse delay duration
                let seconds = args[0].parse::<u64>().map_err(|_| {
                    let mut expected = heapless::String::<32>::new();
                    expected.push_str("positive integer").ok();
                    CliError::InvalidArgumentFormat {
                        arg_index: 0,
                        expected,
                    }
                })?;

                if seconds > 60 {
                    let mut msg = heapless::String::<256>::new();
                    write!(msg, "Maximum delay is 60 seconds").ok();
                    return Err(CliError::CommandFailed(msg));
                }

                // Async delay using Embassy timer
                Timer::after(Duration::from_secs(seconds)).await;

                let mut msg = heapless::String::<64>::new();
                write!(msg, "Delayed for {} second(s)", seconds).ok();
                Ok(Response::success(&msg))
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
        self.hasher
            .verify(password, &user.salt, &user.password_hash)
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
// UART CharIo Implementation (Buffered for Embassy)
// =============================================================================

use core::cell::RefCell;

/// Buffered UART I/O adapter for Embassy.
///
/// Implements the deferred flush pattern described in IO_DESIGN.md:
/// - `put_char()` and `write_str()` buffer to memory only
/// - Output is stored in an internal buffer accessed via RefCell
struct BufferedUartCharIo {
    output_buffer: &'static RefCell<heapless::Vec<u8, 512>>,
}

impl BufferedUartCharIo {
    fn new(output_buffer: &'static RefCell<heapless::Vec<u8, 512>>) -> Self {
        Self { output_buffer }
    }

    /// Check if buffer has data to flush
    fn has_data(&self) -> bool {
        !self.output_buffer.borrow().is_empty()
    }

    /// Get buffered data for flushing
    fn take_buffer(&self) -> heapless::Vec<u8, 512> {
        let mut buf = self.output_buffer.borrow_mut();
        let data = buf.clone();
        buf.clear();
        data
    }
}

impl CharIo for BufferedUartCharIo {
    type Error = ();

    fn get_char(&mut self) -> Result<Option<char>, Self::Error> {
        // Not used in async pattern - read happens externally
        Ok(None)
    }

    fn put_char(&mut self, c: char) -> Result<(), Self::Error> {
        // Buffer to memory only (deferred flush pattern)
        self.output_buffer.borrow_mut().push(c as u8).ok();
        Ok(())
    }

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        // Buffer to memory only (deferred flush pattern)
        let mut buf = self.output_buffer.borrow_mut();
        for c in s.bytes() {
            buf.push(c).ok();
        }
        Ok(())
    }
}

// =============================================================================
// Embassy Tasks
// =============================================================================

/// LED control task.
#[embassy_executor::task]
async fn led_task(
    mut led: Output<'static>,
    channel: &'static Channel<ThreadModeRawMutex, LedCommand, 1>,
) {
    loop {
        match channel.receive().await {
            LedCommand::On => led.set_high(),
            LedCommand::Off => led.set_low(),
        }
    }
}

/// Shell task with async command processing.
#[embassy_executor::task]
async fn shell_task(
    mut tx: BufferedUartTx,
    mut rx: BufferedUartRx,
    led_channel: &'static Channel<ThreadModeRawMutex, LedCommand, 1>,
) {
    // Create output buffer wrapped in RefCell for interior mutability
    static OUTPUT_BUFFER: StaticCell<RefCell<heapless::Vec<u8, 512>>> = StaticCell::new();
    let output_buffer = OUTPUT_BUFFER.init(RefCell::new(heapless::Vec::new()));

    // Create buffered I/O (we'll create two references to the same buffer)
    let io = BufferedUartCharIo::new(output_buffer);
    let io_flush = BufferedUartCharIo::new(output_buffer); // Second reference for flushing

    // Create handlers
    let handlers = PicoHandlers { led_channel };

    // Create credential provider (runtime initialization)
    let provider = PicoCredentialProvider::new();

    // Create shell
    let mut shell: Shell<PicoAccessLevel, BufferedUartCharIo, PicoHandlers, DefaultConfig> =
        Shell::new(&ROOT, handlers, &provider, io);

    // Activate shell
    shell.activate().ok();

    // Flush initial output (welcome message)
    if io_flush.has_data() {
        let data = io_flush.take_buffer();
        AsyncWrite::write_all(&mut tx, &data).await.ok();
    }

    // Main async loop
    loop {
        // Read character from UART (async)
        let mut buf = [0u8; 1];
        match AsyncRead::read_exact(&mut rx, &mut buf).await {
            Ok(_) => {
                let c = buf[0] as char;

                // Process character (async)
                shell.process_char_async(c).await.ok();

                // Flush buffered output (deferred flush pattern)
                if io_flush.has_data() {
                    let data = io_flush.take_buffer();
                    AsyncWrite::write_all(&mut tx, &data).await.ok();
                }
            }
            Err(_) => {
                // UART error - could log or handle
                Timer::after(Duration::from_millis(100)).await;
            }
        }
    }
}

// =============================================================================
// Main Entry Point
// =============================================================================

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialize peripherals
    let p = embassy_rp::init(Default::default());

    // Set up onboard LED (GP25)
    let led = Output::new(p.PIN_25, Level::Low);

    // Create LED command channel
    static LED_CHANNEL: StaticCell<Channel<ThreadModeRawMutex, LedCommand, 1>> = StaticCell::new();
    let led_channel = LED_CHANNEL.init(Channel::new());

    // Configure UART on GP0 (TX) and GP1 (RX)
    static TX_BUF: StaticCell<[u8; 256]> = StaticCell::new();
    static RX_BUF: StaticCell<[u8; 256]> = StaticCell::new();
    let tx_buf = TX_BUF.init([0u8; 256]);
    let rx_buf = RX_BUF.init([0u8; 256]);

    let uart = BufferedUart::new(
        p.UART0,
        p.PIN_0,  // tx_pin
        p.PIN_1,  // rx_pin
        Irqs,
        tx_buf,
        rx_buf,
        uart::Config::default(),
    );
    let (tx, rx) = uart.split();

    // Spawn tasks
    spawner.spawn(led_task(led, led_channel)).unwrap();
    spawner.spawn(shell_task(tx, rx, led_channel)).unwrap();
}

