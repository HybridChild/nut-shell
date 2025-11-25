//! Native async example demonstrating nut-shell with Tokio runtime
//!
//! This example showcases async command execution using Tokio as the async runtime.
//! It demonstrates how to use `process_char_async()` and implement async commands
//! that can perform I/O operations, delays, and other async work.
//!
//! To run:
//! ```bash
//! cargo run --bin async_example --features async,authentication,completion,history
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
use tokio::time::{sleep, Duration};

#[cfg(feature = "authentication")]
use nut_shell::auth::{PasswordHasher, Sha256Hasher, User};

// =============================================================================
// Access Level Definition
// =============================================================================

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AsyncAccessLevel {
    Guest = 0,
    User = 1,
    Admin = 2,
}

impl AccessLevel for AsyncAccessLevel {
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

// Async commands
const CMD_DELAY: CommandMeta<AsyncAccessLevel> = CommandMeta {
    id: "async_delay",
    name: "delay",
    description: "Async delay for N seconds (max 30)",
    access_level: AsyncAccessLevel::Guest,
    kind: CommandKind::Async,
    min_args: 1,
    max_args: 1,
};

const CMD_FETCH: CommandMeta<AsyncAccessLevel> = CommandMeta {
    id: "async_fetch",
    name: "fetch",
    description: "Simulate async HTTP fetch",
    access_level: AsyncAccessLevel::User,
    kind: CommandKind::Async,
    min_args: 1,
    max_args: 1,
};

const CMD_COMPUTE: CommandMeta<AsyncAccessLevel> = CommandMeta {
    id: "async_compute",
    name: "compute",
    description: "Simulate async computation",
    access_level: AsyncAccessLevel::User,
    kind: CommandKind::Async,
    min_args: 0,
    max_args: 0,
};

const ASYNC_DIR: Directory<AsyncAccessLevel> = Directory {
    name: "async",
    children: &[
        Node::Command(&CMD_DELAY),
        Node::Command(&CMD_FETCH),
        Node::Command(&CMD_COMPUTE),
    ],
    access_level: AsyncAccessLevel::Guest,
};

// Sync commands
const CMD_ECHO: CommandMeta<AsyncAccessLevel> = CommandMeta {
    id: "sync_echo",
    name: "echo",
    description: "Echo arguments back",
    access_level: AsyncAccessLevel::Guest,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 16,
};

const CMD_INFO: CommandMeta<AsyncAccessLevel> = CommandMeta {
    id: "sync_info",
    name: "info",
    description: "Show system information",
    access_level: AsyncAccessLevel::Guest,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

const CMD_REBOOT: CommandMeta<AsyncAccessLevel> = CommandMeta {
    id: "sync_reboot",
    name: "reboot",
    description: "Reboot the system (simulated)",
    access_level: AsyncAccessLevel::Admin,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

const SYSTEM_DIR: Directory<AsyncAccessLevel> = Directory {
    name: "system",
    children: &[
        Node::Command(&CMD_REBOOT),
        Node::Command(&CMD_INFO),
    ],
    access_level: AsyncAccessLevel::Guest,
};

// Root directory
const ROOT: Directory<AsyncAccessLevel> = Directory {
    name: "/",
    children: &[
        Node::Directory(&SYSTEM_DIR),
        Node::Directory(&ASYNC_DIR),
        Node::Command(&CMD_ECHO),
    ],
    access_level: AsyncAccessLevel::Guest,
};

// =============================================================================
// Command Handlers
// =============================================================================

struct AsyncHandlers;

impl CommandHandlers<DefaultConfig> for AsyncHandlers {
    fn execute_sync(&self, id: &str, args: &[&str]) -> Result<Response<DefaultConfig>, CliError> {
        match id {
            "sync_echo" => {
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
            "sync_info" => {
                let mut msg = heapless::String::<256>::new();
                write!(msg, "nut-shell Async Example\r\n").ok();
                write!(msg, "Runtime: Tokio\r\n").ok();
                write!(msg, "Features: async commands, authentication\r\n").ok();
                write!(msg, "Try the 'async' directory for async commands!").ok();
                Ok(Response::success(&msg))
            }
            "sync_reboot" => Ok(Response::success("System rebooting...\r\nGoodbye!")),
            _ => Err(CliError::CommandNotFound),
        }
    }

    #[cfg(feature = "async")]
    async fn execute_async(
        &self,
        id: &str,
        args: &[&str],
    ) -> Result<Response<DefaultConfig>, CliError> {
        match id {
            "async_delay" => {
                // Parse delay duration
                let seconds = args[0].parse::<u64>().map_err(|_| {
                    let mut expected = heapless::String::<32>::new();
                    expected.push_str("positive integer").ok();
                    CliError::InvalidArgumentFormat {
                        arg_index: 0,
                        expected,
                    }
                })?;

                if seconds > 30 {
                    let mut msg = heapless::String::<256>::new();
                    write!(msg, "Maximum delay is 30 seconds").ok();
                    return Err(CliError::CommandFailed(msg));
                }

                // Show starting message
                let mut start_msg = heapless::String::<128>::new();
                write!(start_msg, "Starting {}s delay...", seconds).ok();

                // Perform async delay
                sleep(Duration::from_secs(seconds)).await;

                // Return completion message
                let mut msg = heapless::String::<64>::new();
                write!(msg, "Delayed for {} second(s)", seconds).ok();
                Ok(Response::success(&msg))
            }
            "async_fetch" => {
                let url = args[0];

                // Simulate async HTTP fetch
                let mut msg = heapless::String::<256>::new();
                write!(msg, "Fetching '{}'...\r\n", url).ok();

                // Simulate network delay
                sleep(Duration::from_millis(500)).await;

                write!(msg, "Response: 200 OK\r\n").ok();
                write!(msg, "Content-Length: 1234\r\n").ok();
                write!(msg, "Fetch completed successfully!").ok();

                Ok(Response::success(&msg))
            }
            "async_compute" => {
                let mut msg = heapless::String::<256>::new();
                write!(msg, "Starting async computation...\r\n").ok();

                // Simulate some async work with periodic delays
                for i in 1..=3 {
                    sleep(Duration::from_millis(300)).await;
                    write!(msg, "Step {}/3 completed\r\n", i).ok();
                }

                write!(msg, "Computation finished!").ok();
                Ok(Response::success(&msg))
            }
            _ => Err(CliError::CommandNotFound),
        }
    }
}

// =============================================================================
// Credential Provider (when authentication enabled)
// =============================================================================

#[cfg(feature = "authentication")]
struct AsyncCredentialProvider {
    users: [User<AsyncAccessLevel>; 3],
    hasher: Sha256Hasher,
}

#[cfg(feature = "authentication")]
impl AsyncCredentialProvider {
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
            access_level: AsyncAccessLevel::Admin,
            password_hash: admin_hash,
            salt: admin_salt,
        };

        let mut user_username = heapless::String::new();
        user_username.push_str("user").unwrap();
        let user = User {
            username: user_username,
            access_level: AsyncAccessLevel::User,
            password_hash: user_hash,
            salt: user_salt,
        };

        let mut guest_username = heapless::String::new();
        guest_username.push_str("guest").unwrap();
        let guest = User {
            username: guest_username,
            access_level: AsyncAccessLevel::Guest,
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
impl nut_shell::auth::CredentialProvider<AsyncAccessLevel> for AsyncCredentialProvider {
    type Error = ();

    fn find_user(&self, username: &str) -> Result<Option<User<AsyncAccessLevel>>, Self::Error> {
        Ok(self
            .users
            .iter()
            .find(|u| u.username.as_str() == username)
            .cloned())
    }

    fn verify_password(&self, user: &User<AsyncAccessLevel>, password: &str) -> bool {
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

struct AsyncStdioCharIo {
    stdin: io::Stdin,
}

impl AsyncStdioCharIo {
    fn new() -> Self {
        Self { stdin: io::stdin() }
    }
}

impl CharIo for AsyncStdioCharIo {
    type Error = io::Error;

    fn get_char(&mut self) -> Result<Option<char>, Self::Error> {
        let mut buf = [0u8; 1];
        let mut handle = self.stdin.lock();

        // For async example, we still use blocking read in the CharIo trait
        // The async behavior comes from process_char_async(), not from I/O
        match handle.read(&mut buf) {
            Ok(0) => Ok(None), // EOF
            Ok(_) => Ok(Some(buf[0] as char)),
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("nut-shell Async Example");
    println!("=======================\n");
    println!("This example demonstrates async command execution using Tokio.");
    println!();

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

    println!("Try these async commands in the 'async' directory:");
    println!("  cd async");
    println!("  delay 3       - Async delay for 3 seconds");
    println!("  fetch http://example.com - Simulated async HTTP fetch");
    println!("  compute       - Simulated async computation");
    println!();
    println!("Type '?' for help, 'logout' to exit (with auth), or Ctrl+C to quit.\n");

    // Enable raw terminal mode
    let _raw_mode_guard = RawModeGuard::new()?;

    // Create I/O
    let io = AsyncStdioCharIo::new();

    // Create handlers
    let handlers = AsyncHandlers;

    // Create shell (different constructors based on authentication feature)
    #[cfg(feature = "authentication")]
    let provider = AsyncCredentialProvider::new();
    #[cfg(feature = "authentication")]
    let mut shell: Shell<AsyncAccessLevel, AsyncStdioCharIo, AsyncHandlers, DefaultConfig> =
        Shell::new(&ROOT, handlers, &provider, io);

    #[cfg(not(feature = "authentication"))]
    let mut shell: Shell<AsyncAccessLevel, AsyncStdioCharIo, AsyncHandlers, DefaultConfig> =
        Shell::new(&ROOT, handlers, io);

    // Activate shell (shows welcome message and prompt)
    shell.activate()?;

    // Main loop - read characters and feed to shell using async processing
    let stdin = io::stdin();
    let mut stdin_handle = stdin.lock();

    loop {
        // Read one character at a time
        let mut buf = [0u8; 1];
        match stdin_handle.read(&mut buf) {
            Ok(0) => break, // EOF (Ctrl+D on Unix)
            Ok(_) => {
                // Handle Ctrl+C gracefully
                if buf[0] == 0x03 {
                    println!("\r\n");
                    break;
                }

                let c = buf[0] as char;

                // Process character asynchronously
                // This allows async commands to run without blocking the shell
                shell.process_char_async(c).await.ok();
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
