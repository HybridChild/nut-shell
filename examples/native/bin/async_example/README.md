# Native Async Example

This example demonstrates how to use nut-shell with async command execution on a native platform using the Tokio async runtime.

## Features

- **Tokio Async Runtime**: Full async/await support using Tokio
- **Async Commands**: Examples of async commands including delays, simulated HTTP fetches, and computations
- **Authentication**: Optional user authentication with password hashing
- **Command Completion**: Tab completion support for commands and directories
- **Command History**: Navigate through command history with arrow keys

## Building

From the `examples/native` directory:

```bash
# Build with all features
cargo build --bin async_example --features async,authentication,completion,history

# Build release version
cargo build --bin async_example --features async,authentication,completion,history --release
```

## Running

```bash
# Run with all features
cargo run --bin async_example --features async,authentication,completion,history

# Run without authentication
cargo run --bin async_example --features async,completion,history
```

## Usage

When authentication is enabled, use these credentials:
- **admin:admin123** - Admin access level
- **user:user123** - User access level
- **guest:guest123** - Guest access level

### Example Commands

Once logged in, try these commands:

```bash
# Show help
?

# Navigate to async directory
cd async

# List available commands
ls

# Try async delay (waits 3 seconds without blocking)
delay 3

# Simulate async HTTP fetch
fetch http://example.com

# Simulate async computation with progress
compute

# Go back to root
cd ..

# Show system info
system/info

# Echo command (sync)
echo Hello from nut-shell!
```

### Global Commands

- `?` - Show help (list global commands)
- `ls` - List contents of current directory
- `clear` - Clear the screen
- `logout` - End session (when authentication is enabled)
- `ESC ESC` - Clear input buffer

## Architecture Highlights

### Async Command Execution

This example uses `process_char_async()` instead of `process_char()` to support async command execution:

```rust
// Main loop processes characters asynchronously
loop {
    let mut buf = [0u8; 1];
    match stdin_handle.read(&mut buf) {
        Ok(_) => {
            let c = buf[0] as char;
            // Process character asynchronously
            shell.process_char_async(c).await.ok();
        }
        // ... error handling
    }
}
```

### Async Command Implementation

Commands are marked as `CommandKind::Async` and implemented in the `execute_async()` trait method:

```rust
const CMD_DELAY: CommandMeta<AsyncAccessLevel> = CommandMeta {
    id: "async_delay",
    name: "delay",
    description: "Async delay for N seconds",
    kind: CommandKind::Async,  // Mark as async
    // ... other fields
};

impl CommandHandlers<DefaultConfig> for AsyncHandlers {
    async fn execute_async(&self, id: &str, args: &[&str])
        -> Result<Response<DefaultConfig>, CliError>
    {
        match id {
            "async_delay" => {
                let seconds = args[0].parse::<u64>()?;
                sleep(Duration::from_secs(seconds)).await;
                Ok(Response::success("Delay completed"))
            }
            // ... other async commands
        }
    }
}
```

### Why Async?

Async commands allow you to:
- Perform I/O operations without blocking (network requests, file I/O)
- Execute long-running operations while remaining responsive
- Coordinate multiple concurrent tasks
- Use async libraries and frameworks (Tokio, async-std, etc.)

### Sync vs Async Commands

You can mix both sync and async commands in the same shell:
- **Sync commands** (`CommandKind::Sync`): Simple operations that complete immediately
- **Async commands** (`CommandKind::Async`): Operations that may need to wait for I/O or other async work

The shell automatically dispatches to the correct handler based on the command kind.

## Key Differences from Embedded Examples

Unlike the embedded Embassy example (rp-pico/embassy_uart_cli):
- Uses **Tokio** runtime instead of Embassy
- Runs on **native** platforms (Linux, macOS, Windows)
- Uses **standard I/O** instead of UART
- Can leverage the full Tokio ecosystem (HTTP clients, async file I/O, etc.)

## Exit

To exit the application:
- `logout` - When authentication is enabled
- `Ctrl+C` - Graceful shutdown
- `Ctrl+D` - EOF signal

## Learn More

See the [main documentation](../../../../docs/) for more details on:
- [EXAMPLES.md](../../../../docs/EXAMPLES.md) - Usage patterns and configuration
- [DESIGN.md](../../../../docs/DESIGN.md) - Architecture and design patterns
- [IO_DESIGN.md](../../../../docs/IO_DESIGN.md) - CharIo implementation details
