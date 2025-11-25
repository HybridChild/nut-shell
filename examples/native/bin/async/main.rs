//! Native async example demonstrating nut-shell with Tokio runtime
//!
//! This example showcases async command execution using Tokio as the async runtime.
//! It demonstrates how to use `process_char_async()` and implement async commands
//! that can perform I/O operations, delays, and other async work.
//!
//! To run:
//! ```bash
//! cargo run --bin async --features async,authentication,completion,history
//! ```
//!
//! Default credentials (when authentication enabled):
//! - admin:admin123 (Admin access)
//! - user:user123 (User access)
//! - guest:guest123 (Guest access)

mod handlers;
mod tree;

use handlers::AsyncHandlers;
use native_examples::{ExampleAccessLevel, RawModeGuard, StdioCharIo};
#[cfg(feature = "authentication")]
use native_examples::ExampleCredentialProvider;
use nut_shell::{config::DefaultConfig, shell::Shell};
use std::io::{self as stdio, Read};
use tree::ROOT;

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
    let io = StdioCharIo::new();

    // Create handlers
    let handlers = AsyncHandlers;

    // Create shell (different constructors based on authentication feature)
    #[cfg(feature = "authentication")]
    let provider = ExampleCredentialProvider::new();
    #[cfg(feature = "authentication")]
    let mut shell: Shell<ExampleAccessLevel, StdioCharIo, AsyncHandlers, DefaultConfig> =
        Shell::new(&ROOT, handlers, &provider, io);

    #[cfg(not(feature = "authentication"))]
    let mut shell: Shell<ExampleAccessLevel, StdioCharIo, AsyncHandlers, DefaultConfig> =
        Shell::new(&ROOT, handlers, io);

    // Activate shell (shows welcome message and prompt)
    shell.activate()?;

    // Main loop - read characters and feed to shell using async processing
    let stdin = stdio::stdin();
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
