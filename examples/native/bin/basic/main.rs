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

mod handler;
mod tree;

use handler::ExampleHandler;
#[cfg(feature = "authentication")]
use native_examples::ExampleCredentialProvider;
use native_examples::{ExampleAccessLevel, RawModeGuard, StdioCharIo};
use nut_shell::{config::DefaultConfig, shell::Shell};
use std::io::{self as stdio, Read};
use tree::ROOT;

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

    // Create handler
    let handler = ExampleHandler;

    // Create shell (different constructors based on authentication feature)
    #[cfg(feature = "authentication")]
    let provider = ExampleCredentialProvider::new();
    #[cfg(feature = "authentication")]
    let mut shell: Shell<ExampleAccessLevel, StdioCharIo, ExampleHandler, DefaultConfig> =
        Shell::new(&ROOT, handler, &provider, io);

    #[cfg(not(feature = "authentication"))]
    let mut shell: Shell<ExampleAccessLevel, StdioCharIo, ExampleHandler, DefaultConfig> =
        Shell::new(&ROOT, handler, io);

    // Activate shell (shows welcome message and prompt)
    shell.activate()?;

    // Main loop - read characters and feed to shell
    // This pattern resembles embedded target usage:
    // - Embedded: Poll UART RX buffer for characters
    // - Native: Poll stdin for characters
    // - Both: Feed characters to shell.process_char() one at a time
    // - Shell controls all output (including echo and password masking)
    let stdin = stdio::stdin();
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
