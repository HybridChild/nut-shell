//! Basic example demonstrating nut-shell usage on native platform
//!
//! This example creates a simple CLI with command tree navigation,
//! and various commands demonstrating different features.
//!
//! To run with all features (authentication, completion, history):
//! ```bash
//! cargo run --example basic --features authentication,completion,history
//! ```
//!
//! To run without authentication:
//! ```bash
//! cargo run --example basic --features completion,history
//! ```
//!
//! Default credentials (when authentication enabled):
//! - admin:admin123 (Admin access)
//! - user:user123 (User access)
//! - guest:guest123 (Guest access)

use core::fmt::Write;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use nut_shell::{
    CliError,
    auth::AccessLevel,
    config::DefaultConfig,
    io::CharIo,
    response::Response,
    shell::{Shell, handlers::CommandHandlers},
    tree::{CommandKind, CommandMeta, Directory, Node},
};
use std::io::{self, Read};

#[cfg(feature = "authentication")]
use nut_shell::auth::{PasswordHasher, Sha256Hasher, User};

// =============================================================================
// Access Level Definition
// =============================================================================

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExampleAccessLevel {
    Guest = 0,
    User = 1,
    Admin = 2,
}

impl AccessLevel for ExampleAccessLevel {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "Guest" => Some(Self::Guest),
            "User" => Some(Self::User),
            "Admin" => Some(Self::Admin),
            _ => None,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::Guest => "Guest",
            Self::User => "User",
            Self::Admin => "Admin",
        }
    }
}

// =============================================================================
// Command Tree Definition
// =============================================================================

// System commands
const CMD_REBOOT: CommandMeta<ExampleAccessLevel> = CommandMeta {
    name: "reboot",
    description: "Reboot the system (simulated)",
    access_level: ExampleAccessLevel::Admin,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

const CMD_STATUS: CommandMeta<ExampleAccessLevel> = CommandMeta {
    name: "status",
    description: "Show system status",
    access_level: ExampleAccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

const CMD_VERSION: CommandMeta<ExampleAccessLevel> = CommandMeta {
    name: "version",
    description: "Show version information",
    access_level: ExampleAccessLevel::Guest,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

const SYSTEM_DIR: Directory<ExampleAccessLevel> = Directory {
    name: "system",
    children: &[
        Node::Command(&CMD_REBOOT),
        Node::Command(&CMD_STATUS),
        Node::Command(&CMD_VERSION),
    ],
    access_level: ExampleAccessLevel::Guest,
};

// Config commands
const CMD_CONFIG_GET: CommandMeta<ExampleAccessLevel> = CommandMeta {
    name: "get",
    description: "Get configuration value",
    access_level: ExampleAccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 1,
    max_args: 1,
};

const CMD_CONFIG_SET: CommandMeta<ExampleAccessLevel> = CommandMeta {
    name: "set",
    description: "Set configuration value",
    access_level: ExampleAccessLevel::Admin,
    kind: CommandKind::Sync,
    min_args: 2,
    max_args: 2,
};

const CONFIG_DIR: Directory<ExampleAccessLevel> = Directory {
    name: "config",
    children: &[
        Node::Command(&CMD_CONFIG_GET),
        Node::Command(&CMD_CONFIG_SET),
    ],
    access_level: ExampleAccessLevel::User,
};

// Root-level commands
const CMD_ECHO: CommandMeta<ExampleAccessLevel> = CommandMeta {
    name: "echo",
    description: "Echo arguments back",
    access_level: ExampleAccessLevel::Guest,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 16,
};

const CMD_UPTIME: CommandMeta<ExampleAccessLevel> = CommandMeta {
    name: "uptime",
    description: "Show system uptime (simulated)",
    access_level: ExampleAccessLevel::Guest,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

// Root directory
const ROOT: Directory<ExampleAccessLevel> = Directory {
    name: "/",
    children: &[
        Node::Directory(&SYSTEM_DIR),
        Node::Directory(&CONFIG_DIR),
        Node::Command(&CMD_ECHO),
        Node::Command(&CMD_UPTIME),
    ],
    access_level: ExampleAccessLevel::Guest,
};

// =============================================================================
// Command Handlers
// =============================================================================

struct ExampleHandlers;

impl CommandHandlers<DefaultConfig> for ExampleHandlers {
    fn execute_sync(&self, name: &str, args: &[&str]) -> Result<Response<DefaultConfig>, CliError> {
        match name {
            "reboot" => Ok(Response::success("System rebooting...\r\nGoodbye!")),
            "status" => {
                let mut msg = heapless::String::<256>::new();
                write!(msg, "System Status:\r\n").ok();
                write!(msg, "  CPU Usage: 23%\r\n").ok();
                write!(msg, "  Memory: 45% used\r\n").ok();
                write!(msg, "  Uptime: 42 hours").ok();
                Ok(Response::success(&msg))
            }
            "version" => Ok(Response::success(
                "nut-shell v0.1.0\r\nRust embedded CLI framework",
            )),
            "get" => {
                let key = args[0];
                let mut msg = heapless::String::<256>::new();
                write!(msg, "Config[{}] = <simulated value>", key).ok();
                Ok(Response::success(&msg))
            }
            "set" => {
                let key = args[0];
                let value = args[1];
                let mut msg = heapless::String::<256>::new();
                write!(msg, "Config[{}] set to '{}'", key, value).ok();
                Ok(Response::success(&msg))
            }
            "echo" => {
                if args.is_empty() {
                    Ok(Response::success(""))
                } else {
                    let mut msg = heapless::String::<256>::new();
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 {
                            msg.push(' ').ok();
                        }
                        msg.push_str(arg).ok();
                    }
                    Ok(Response::success(&msg))
                }
            }
            "uptime" => Ok(Response::success("System uptime: 42 hours, 13 minutes")),
            _ => Err(CliError::CommandNotFound),
        }
    }

    #[cfg(feature = "async")]
    async fn execute_async(
        &self,
        name: &str,
        _args: &[&str],
    ) -> Result<Response<DefaultConfig>, CliError> {
        // This example doesn't use async commands
        // Return error for any command name
        let mut msg = heapless::String::<256>::new();
        write!(
            msg,
            "Async command '{}' not supported in this example",
            name
        )
        .ok();
        Err(CliError::Other(msg))
    }
}

// =============================================================================
// Credential Provider (when authentication enabled)
// =============================================================================

#[cfg(feature = "authentication")]
struct ExampleCredentialProvider {
    users: [User<ExampleAccessLevel>; 3],
    hasher: Sha256Hasher,
}

#[cfg(feature = "authentication")]
impl ExampleCredentialProvider {
    fn new() -> Self {
        let hasher = Sha256Hasher;

        // Create users with hashed passwords
        let admin_salt: [u8; 16] = [1; 16];
        let user_salt: [u8; 16] = [2; 16];
        let guest_salt: [u8; 16] = [3; 16];

        let admin_hash = hasher.hash("admin123", &admin_salt);
        let user_hash = hasher.hash("user123", &user_salt);
        let guest_hash = hasher.hash("guest123", &guest_salt);

        let mut admin_username = heapless::String::new();
        admin_username.push_str("admin").unwrap();
        let admin = User {
            username: admin_username,
            access_level: ExampleAccessLevel::Admin,
            password_hash: admin_hash,
            salt: admin_salt,
        };

        let mut user_username = heapless::String::new();
        user_username.push_str("user").unwrap();
        let user = User {
            username: user_username,
            access_level: ExampleAccessLevel::User,
            password_hash: user_hash,
            salt: user_salt,
        };

        let mut guest_username = heapless::String::new();
        guest_username.push_str("guest").unwrap();
        let guest = User {
            username: guest_username,
            access_level: ExampleAccessLevel::Guest,
            password_hash: guest_hash,
            salt: guest_salt,
        };

        Self {
            users: [admin, user, guest],
            hasher,
        }
    }
}

#[cfg(feature = "authentication")]
impl nut_shell::auth::CredentialProvider<ExampleAccessLevel> for ExampleCredentialProvider {
    type Error = ();

    fn find_user(&self, username: &str) -> Result<Option<User<ExampleAccessLevel>>, Self::Error> {
        Ok(self
            .users
            .iter()
            .find(|u| u.username.as_str() == username)
            .cloned())
    }

    fn verify_password(&self, user: &User<ExampleAccessLevel>, password: &str) -> bool {
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
// Terminal Raw Mode Guard
// =============================================================================

/// RAII guard that enables raw terminal mode on creation and restores on drop.
/// This ensures the terminal is always restored, even on panic or error.
struct RawModeGuard;

impl RawModeGuard {
    fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        // Always try to restore terminal mode
        let _ = disable_raw_mode();
    }
}

// =============================================================================
// I/O Implementation
// =============================================================================

struct StdioCharIo {
    stdin: io::Stdin,
}

impl StdioCharIo {
    fn new() -> Self {
        Self { stdin: io::stdin() }
    }
}

impl CharIo for StdioCharIo {
    type Error = io::Error;

    fn get_char(&mut self) -> Result<Option<char>, Self::Error> {
        let mut buf = [0u8; 1];
        let mut handle = self.stdin.lock();

        // Non-blocking read would require platform-specific code
        // For this example, we'll use blocking reads
        match handle.read(&mut buf) {
            Ok(0) => Ok(None), // EOF
            Ok(_) => {
                // Simple ASCII to char conversion
                // For a production CLI, you'd want proper UTF-8 handling
                Ok(Some(buf[0] as char))
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn put_char(&mut self, c: char) -> Result<(), Self::Error> {
        print!("{}", c);
        use std::io::Write;
        std::io::stdout().flush()?;
        Ok(())
    }

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        print!("{}", s);
        use std::io::Write;
        std::io::stdout().flush()?;
        Ok(())
    }
}

// =============================================================================
// Main
// =============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("nut-shell Basic Example");
    println!("=======================\n");

    #[cfg(feature = "authentication")]
    {
        println!("Authentication enabled. Available credentials:");
        println!("  admin:admin123  (Admin access)");
        println!("  user:user123    (User access)");
        println!("  guest:guest123  (Guest access)");
        println!();
    }

    #[cfg(not(feature = "authentication"))]
    {
        println!("Authentication disabled. All commands available.");
        println!();
    }

    println!("Type '?' for help, 'logout' to exit (with auth), or Ctrl+C to quit.\n");

    // Enable raw terminal mode to resemble embedded target behavior:
    // - No local echo (shell controls all echoing for password masking)
    // - No line buffering (process characters immediately)
    // - No special key processing by terminal (Tab, arrows passed to shell)
    let _raw_mode_guard = RawModeGuard::new()?;

    // Create I/O
    let io = StdioCharIo::new();

    // Create handlers
    let handlers = ExampleHandlers;

    // Create shell (different constructors based on authentication feature)
    #[cfg(feature = "authentication")]
    let provider = ExampleCredentialProvider::new();
    #[cfg(feature = "authentication")]
    let mut shell: Shell<ExampleAccessLevel, StdioCharIo, ExampleHandlers, DefaultConfig> =
        Shell::new(&ROOT, handlers, &provider, io);

    #[cfg(not(feature = "authentication"))]
    let mut shell: Shell<ExampleAccessLevel, StdioCharIo, ExampleHandlers, DefaultConfig> =
        Shell::new(&ROOT, handlers, io);

    // Activate shell (shows welcome message and prompt)
    shell.activate()?;

    // Main loop - read characters and feed to shell
    // This pattern resembles embedded target usage:
    // - Embedded: Poll UART RX buffer for characters
    // - Native: Poll stdin for characters
    // - Both: Feed characters to shell.process_char() one at a time
    // - Shell controls all output (including echo and password masking)
    let stdin = io::stdin();
    let mut stdin_handle = stdin.lock();

    loop {
        // Read one character at a time (like polling UART on embedded target)
        let mut buf = [0u8; 1];
        match stdin_handle.read(&mut buf) {
            Ok(0) => break, // EOF (Ctrl+D on Unix)
            Ok(_) => {
                // In raw mode, Ctrl+C becomes character 0x03 instead of sending SIGINT.
                // For this native example, we detect it and exit gracefully.
                // On embedded targets, you might:
                // - Ignore Ctrl+C entirely (no concept of "interrupt")
                // - Use it as a special command (e.g., abort current operation)
                // - Implement different exit mechanisms (reset button, watchdog, etc.)
                if buf[0] == 0x03 {
                    println!("\r\n"); // Move to new line before exit
                    break;
                }

                let c = buf[0] as char;
                // Feed character to shell (shell controls echoing)
                shell.process_char(c)?;
            }
            Err(e) => {
                // Restore terminal before printing error
                drop(_raw_mode_guard);
                eprintln!("\nError reading input: {}", e);
                break;
            }
        }
    }

    // Guard will automatically restore terminal mode on drop
    Ok(())
}
