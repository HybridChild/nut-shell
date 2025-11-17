# CharIo Design: Universal I/O Abstraction

This document explains the I/O abstraction design that enables cli-service to work efficiently in both bare-metal and async runtime environments (like Embassy).

**Related Documentation:**
- **[DESIGN.md](DESIGN.md)**: Metadata/execution separation pattern (Section 1) - enables async command support
- **[SPECIFICATION.md](SPECIFICATION.md)**: Terminal I/O behavior specification
- **[INTERNALS.md](INTERNALS.md)**: Complete data flow including I/O processing
- **[IMPLEMENTATION.md](IMPLEMENTATION.md)**: CharIo implementation tasks (Phase 1)

## Design Problem

The CLI needs to work in two very different environments:

### Bare-Metal (Blocking I/O)
```rust
// Simple polling loop
loop {
    if uart.is_readable() {
        let c = uart.read_byte();  // Blocks until byte available
        cli.process_char(c);
        // Output already written to UART
    }
}
```

### Embassy (Async I/O)
```rust
// Async task
loop {
    let c = uart.read().await;  // Suspends task until byte available
    cli.process_char(c);
    // Can't .await inside process_char()!
}
```

**The challenge:** `process_char()` needs to output immediately (echo, responses, prompts), but Embassy's I/O is async and requires `.await`.

## Solution: Explicit Buffering Model

### Core Principle

**All CharIo implementations MUST buffer output.** The difference is **when** they flush:

- **Bare-metal:** Flushes immediately (blocking acceptable)
- **Async runtimes:** Defers flush (batches output)

### CharIo Trait Design

```rust
/// Character I/O abstraction for CLI service.
///
/// # Buffering Model
///
/// All implementations MUST buffer output internally. The `put_char()` and
/// `write_str()` methods write to a buffer and MUST NOT perform I/O that
/// could await or block indefinitely.
///
/// ## For Bare-Metal Platforms
///
/// Bare-metal implementations may flush immediately in `put_char()`, as
/// blocking is acceptable in single-threaded embedded systems:
///
/// ```rust
/// impl CharIo for UartIo {
///     fn put_char(&mut self, c: char) -> Result<(), Self::Error> {
///         self.uart.write_byte(c as u8);  // Blocking write - OK!
///         Ok(())
///     }
/// }
/// ```
///
/// ## For Async Runtimes (Embassy, RTIC, etc.)
///
/// Async implementations MUST buffer to memory only. Flushing happens
/// externally after `process_char()` returns:
///
/// ```rust
/// impl CharIo for EmbassyUsbIo {
///     fn put_char(&mut self, c: char) -> Result<(), Self::Error> {
///         self.output_buffer.push(c as u8).ok();  // Memory only
///         Ok(())
///     }
/// }
///
/// impl EmbassyUsbIo {
///     pub async fn flush(&mut self) -> Result<()> {
///         self.class.write_packet(&self.output_buffer).await
///     }
/// }
/// ```
///
pub trait CharIo {
    type Error;

    /// Read a character if available (non-blocking).
    ///
    /// Returns:
    /// - `Ok(Some(c))` - Character available
    /// - `Ok(None)` - No character available (would block)
    /// - `Err(e)` - I/O error
    ///
    /// # Implementation Notes
    ///
    /// - Bare-metal: Check UART FIFO, return immediately
    /// - Async: Return `None` (reading happens externally via async)
    fn get_char(&mut self) -> Result<Option<char>, Self::Error>;

    /// Write a character to output buffer.
    ///
    /// This method MUST NOT block indefinitely. Implementations may:
    /// - Write to memory buffer (async platforms)
    /// - Write directly to hardware (bare-metal, blocking acceptable)
    /// - Return error if buffer full
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Buffer is full (memory-buffered implementations)
    /// - Hardware error (direct-write implementations)
    fn put_char(&mut self, c: char) -> Result<(), Self::Error>;

    /// Write a string to output buffer.
    ///
    /// Default implementation calls `put_char()` for each character.
    /// Implementations may override for efficiency.
    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        for c in s.chars() {
            self.put_char(c)?;
        }
        Ok(())
    }
}
```

## Usage Patterns

### Pattern 1: Bare-Metal UART (Immediate Flush)

```rust
use embedded_hal::serial::{Read, Write};

struct UartIo<UART> {
    uart: UART,
}

impl<UART> CharIo for UartIo<UART>
where
    UART: Read<u8> + Write<u8>,
{
    type Error = UART::Error;

    fn get_char(&mut self) -> Result<Option<char>, Self::Error> {
        match self.uart.read() {
            Ok(byte) => Ok(Some(byte as char)),
            Err(nb::Error::WouldBlock) => Ok(None),
            Err(nb::Error::Other(e)) => Err(e),
        }
    }

    fn put_char(&mut self, c: char) -> Result<(), Self::Error> {
        // Direct write - blocks until UART ready
        nb::block!(self.uart.write(c as u8))?;
        Ok(())
    }
}

// Usage - simple polling loop
fn main() -> ! {
    let uart = setup_uart();
    let mut io = UartIo { uart };
    let mut cli = CliService::new(&TREE, &mut io);
    cli.activate();

    loop {
        // Non-blocking read
        if let Ok(Some(c)) = io.get_char() {
            // Process character - output written immediately
            cli.process_char(c).ok();
        }

        // Do other work...
    }
}
```

**Characteristics:**
- ✅ Zero memory overhead (no output buffer)
- ✅ Immediate visual feedback
- ✅ Simple implementation
- ⚠️ Blocks on UART writes (acceptable in single-threaded)

### Pattern 2: Embassy USB (Deferred Flush)

```rust
use embassy_usb::class::cdc_acm::CdcAcmClass;

struct EmbassyUsbIo<'d, D: embassy_usb::driver::Driver<'d>> {
    class: CdcAcmClass<'d, D>,
    output_buffer: heapless::Vec<u8, 256>,
}

impl<'d, D: embassy_usb::driver::Driver<'d>> CharIo for EmbassyUsbIo<'d, D> {
    type Error = core::convert::Infallible;  // Buffering can't fail

    fn get_char(&mut self) -> Result<Option<char>, Self::Error> {
        // Not used - reading happens externally
        Ok(None)
    }

    fn put_char(&mut self, c: char) -> Result<(), Self::Error> {
        // Write to memory buffer only (never blocks!)
        self.output_buffer.push(c as u8).ok();
        Ok(())
    }

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        // Optimized: extend buffer directly
        self.output_buffer.extend_from_slice(s.as_bytes()).ok();
        Ok(())
    }
}

impl<'d, D: embassy_usb::driver::Driver<'d>> EmbassyUsbIo<'d, D> {
    /// Flush buffered output (async)
    pub async fn flush(&mut self) -> Result<(), D::EndpointError> {
        if !self.output_buffer.is_empty() {
            self.class.write_packet(&self.output_buffer).await?;
            self.output_buffer.clear();
        }
        Ok(())
    }
}

// Usage - batch processing pattern
#[embassy_executor::task]
async fn cli_task(class: CdcAcmClass<'static, Driver<'static, USB>>) {
    let mut io = EmbassyUsbIo::new(class);
    let mut cli = CliService::new(&TREE, &mut io);

    cli.activate();      // Writes welcome message to buffer
    io.flush().await.ok();  // Flush welcome

    let mut buffer = [0u8; 64];
    loop {
        // 1. AWAIT - task suspends until data available
        let n = io.class.read_packet(&mut buffer).await.unwrap();

        // 2. PROCESS - cli calls io.put_char() internally
        for &byte in &buffer[..n] {
            cli.process_char(byte as char).ok();
            // ^ Buffers echo, responses, prompts to memory
        }

        // 3. FLUSH - single async write of all output
        io.flush().await.ok();
    }
}
```

**Characteristics:**
- ✅ Efficient - one USB transaction per batch
- ✅ Non-blocking - task suspends on `.await`
- ✅ Low latency - typical 1-2ms (USB poll rate)
- ✅ Handles paste/scripts efficiently
- ⚠️ Requires manual flush (explicit in loop)

### Pattern 3: Embassy UART (Deferred Flush)

```rust
use embassy_rp::uart::BufferedUart;

struct EmbassyUartIo<'d> {
    uart: BufferedUart<'d, UART0>,
    output_buffer: heapless::Vec<u8, 256>,
}

impl CharIo for EmbassyUartIo<'_> {
    type Error = core::convert::Infallible;

    fn get_char(&mut self) -> Result<Option<char>, Self::Error> {
        Ok(None)  // Reading happens externally
    }

    fn put_char(&mut self, c: char) -> Result<(), Self::Error> {
        self.output_buffer.push(c as u8).ok();
        Ok(())
    }
}

impl EmbassyUartIo<'_> {
    pub async fn flush(&mut self) -> Result<(), embassy_rp::uart::Error> {
        if !self.output_buffer.is_empty() {
            self.uart.write(&self.output_buffer).await?;
            self.output_buffer.clear();
        }
        Ok(())
    }
}

// Usage - same pattern as USB
#[embassy_executor::task]
async fn cli_task(uart: BufferedUart<'static, UART0>) {
    let mut io = EmbassyUartIo::new(uart);
    let mut cli = CliService::new(&TREE, &mut io);

    cli.activate();
    io.flush().await.ok();  // flush welcome message

    let mut buffer = [0u8; 64];
    loop {
        let n = io.uart.read(&mut buffer).await.unwrap();

        for &byte in &buffer[..n] {
            cli.process_char(byte as char).ok();
        }

        io.flush().await.ok();
    }
}
```

## Why This Design Works

### 1. Respects CliService Ownership Model

From INTERNALS.md, `CliService` owns/references the `CharIo`:

```rust
impl<'tree, L, IO> CliService<'tree, L, IO>
where
    IO: CharIo,
{
    pub fn process_char(&mut self, c: char) -> Result<(), IO::Error> {
        // CLI calls self.io.put_char() internally
        self.io.put_char(c)?;
        // ...
    }
}
```

### 2. Zero-Cost for Bare-Metal

Bare-metal implementations can flush immediately:
- No output buffer allocation needed
- Direct UART writes
- Compiler optimizes away the abstraction

### 3. Efficient for Async

Async implementations batch output:
- Single `heapless::Vec` buffer (~256 bytes)
- One I/O transaction per `process_char()` call (or batch)
- Task suspends only on read/flush (not during processing)

### 4. Single Codebase

Same `CliService` code works everywhere:
- No `#[cfg(async)]` feature flags needed for core
- CharIo trait is platform-agnostic
- Implementations handle platform-specific details

## Buffer Sizing Recommendations

### Output Buffer (Async Platforms)

Recommended sizes based on expected output:

| Use Case | Buffer Size | Rationale |
|----------|-------------|-----------|
| Simple prompts | 64 bytes | `user@/path> ` + short responses |
| Directory listings | 256 bytes | Multiple lines of output |
| Command help text | 512 bytes | Verbose responses |
| Maximum safety | 1024 bytes | Handles any single command output |

**Note:** Buffer overflows should be rare if sized correctly. If overflow occurs:
- Return error from `put_char()`
- CliService will propagate error
- Alternatively: flush mid-process (complex, not recommended)

### Input Buffer (Both Platforms)

Fixed at **128 bytes** (defined in CliService):
- Sufficient for command paths + arguments
- `heapless::String<128>` in CliService struct

## Performance Comparison

### Bare-Metal (Immediate Flush)

| Metric | Value |
|--------|-------|
| Output latency | ~10-100µs per char |
| Memory overhead | 0 bytes (no buffer) |
| CPU overhead | Blocking on UART |

### Embassy USB (Deferred Flush)

| Metric | Value |
|--------|-------|
| Output latency | 1-2ms (USB poll rate) |
| Memory overhead | ~256 bytes |
| CPU overhead | ~0% idle (task suspended) |

### Embassy UART (Deferred Flush)

| Metric | Value |
|--------|-------|
| Output latency | <1ms |
| Memory overhead | ~256 bytes |
| CPU overhead | ~0% idle (DMA + suspension) |

## Alternative Designs Considered

### ❌ Alternative 1: Async CharIo Trait

```rust
trait AsyncCharIo {
    async fn get_char(&mut self) -> Result<char>;
    async fn put_char(&mut self, c: char) -> Result<()>;
}
```

**Problems:**
- Requires `async_trait` or unstable `async fn` in traits
- Makes `process_char()` async (complex state machine)
- No benefit for bare-metal
- Higher complexity

### ❌ Alternative 2: Callback-Based

```rust
trait CharIo {
    fn set_output_callback(&mut self, cb: impl FnMut(char));
}
```

**Problems:**
- Can't propagate errors
- Lifetime issues with closures in `no_std`
- Awkward API

### ✅ Chosen: Explicit Buffering

Simple, efficient, works everywhere.

## Common Issues and Solutions

### Issue: Characters Echoed Multiple Times

**Symptom:** Typing `a` shows `aaa`

**Cause:** Both terminal and CLI echoing

**Solution:** Disable local echo in terminal:
```bash
# Linux/macOS
stty -echo -F /dev/ttyACM0

# Or use screen with proper settings
screen /dev/ttyACM0 115200,cs8,-parenb,-cstopb,-echo
```

### Issue: Backspace Not Working

**Symptom:** Backspace shows `^H` or `^?`

**Cause:** Terminal sending wrong control code

**Solution:** CLI handles both `0x08` (BS) and `0x7F` (DEL) - configure terminal:
```bash
# Set backspace key
stty erase ^H
```

### Issue: High CPU Usage (Async Platforms)

**Symptom:** Executor using excessive CPU

**Cause:** Polling instead of awaiting

**Solution:** Ensure you're using `.await`, not polling:
```rust
// ❌ WRONG - polling loop (wastes CPU)
loop {
    if let Some(c) = try_read_char() {
        cli.process_char(c)?;
    }
}

// ✅ RIGHT - await (task suspends)
loop {
    let c = read_char().await;  // Task suspends here - zero CPU
    cli.process_char(c)?;
}
```

### Issue: Output Truncated

**Symptom:** Long responses cut off

**Cause:** Output buffer too small

**Solution:** Increase buffer size:
```rust
struct MyIo {
    output_buffer: heapless::Vec<u8, 512>, // Increased from 256
}
```

## Advanced Usage Patterns

### Long-Running Commands (Async Platforms)

For commands that take significant time, spawn background tasks:

```rust
// With Embassy
static SPAWNER: StaticCell<Spawner> = StaticCell::new();

fn firmware_update_cmd(args: &[&str]) -> Result<Response, CliError> {
    // Spawn background task
    SPAWNER.get().spawn(firmware_update_task()).ok();

    Ok(Response::success("Firmware update started in background"))
}

#[embassy_executor::task]
async fn firmware_update_task() {
    // Long-running operation that doesn't block CLI
    for i in 0..100 {
        write_flash_page(i).await;
        embassy_time::Timer::after_millis(100).await;
    }
}
```

### Multiple CLI Sessions

For multi-user support (e.g., USB + Telnet):

```rust
#[embassy_executor::task(pool_size = 4)]
async fn cli_session<IO: CharIo>(id: u8, mut io: IO) {
    let mut cli = CliService::new(&ROOT, &mut io);
    cli.activate();
    io.flush().await.ok();

    // Each session has independent state
    let mut buffer = [0u8; 64];
    loop {
        let n = io.read(&mut buffer).await?;
        for &byte in &buffer[..n] {
            cli.process_char(byte as char).ok();
        }
        io.flush().await.ok();
    }
}
```

### Shared State Between Commands

Use static or reference to shared state:

```rust
use core::sync::atomic::{AtomicBool, Ordering};

static LED_STATE: AtomicBool = AtomicBool::new(false);

fn led_toggle_cmd(_args: &[&str]) -> Result<Response, CliError> {
    let new_state = !LED_STATE.load(Ordering::Relaxed);
    LED_STATE.store(new_state, Ordering::Relaxed);

    let msg = if new_state { "LED on" } else { "LED off" };
    Ok(Response::success(msg))
}
```

## Anti-Patterns to Avoid

### ❌ Don't: Make CharIo Async

```rust
// AVOID - adds complexity without benefit
trait CharIo {
    async fn put_char(&mut self, c: char) -> Result<()>;
}
```

**Why:** Makes `process_char()` async, requires complex state machine, no benefit for bare-metal.

### ❌ Don't: Flush After Every Character

```rust
// AVOID - inefficient for async platforms
for byte in batch {
    cli.process_char(byte as char)?;
    io.flush().await?;  // Too many I/O operations!
}
```

**Why:** Each flush is an I/O transaction (1-2ms on USB). Batch instead.

### ❌ Don't: Block in Sync Command Execute Functions

```rust
// AVOID - blocks entire CLI in sync mode
fn bad_command(_args: &[&str]) -> Result<Response, CliError> {
    // Blocking operation
    for _ in 0..1000000 { }  // Blocks everything!
    Ok(Response::success("Done"))
}
```

**Why:** CLI can't process input during blocking. Solutions:
- **Async platforms**: Use async commands with `process_char_async()` (see below)
- **Bare-metal**: Keep commands fast, or spawn background tasks manually

### ❌ Don't: Allocate on Heap in Commands

```rust
// AVOID - no_std incompatible
fn bad_command(_args: &[&str]) -> Result<Response, CliError> {
    let data = vec![1, 2, 3];  // Heap allocation - won't compile!
    Ok(Response::success("Done"))
}
```

**Why:** Library is `no_std`. Use `heapless` collections instead.

## Async Command Support

With metadata/execution separation, commands can be naturally async without requiring manual spawning or global state.

### Using Async Commands

**Command definition:**
```rust
// Metadata marks command as async
const HTTP_GET: CommandMeta<Level> = CommandMeta {
    name: "http-get",
    kind: CommandKind::Async,  // Marked as async
    // ... other metadata
};

// Handler implementation with natural async/await
impl CommandHandlers for MyHandlers {
    async fn execute_async(&self, name: &str, args: &[&str]) -> Result<Response, CliError> {
        match name {
            "http-get" => {
                let url = args[0];
                let response = HTTP_CLIENT.get(url).await?;  // Natural async!
                Ok(Response::success(&response))
            }
            _ => Err(CliError::CommandNotFound),
        }
    }
}
```

**Embassy task with async commands:**
```rust
#[embassy_executor::task]
async fn cli_task(usb: CdcAcmClass<'static, Driver<'static, USB>>) {
    let mut io = EmbassyUsbIo::new(usb);
    let handlers = MyHandlers;
    let mut cli = CliService::new(&ROOT, handlers, io);

    cli.activate().ok();
    io.flush().await.ok();

    let mut buffer = [0u8; 64];
    loop {
        // 1. AWAIT input
        let n = io.class.read_packet(&mut buffer).await.unwrap();

        // 2. PROCESS (may await on async commands)
        for &byte in &buffer[..n] {
            cli.process_char_async(byte as char).await.ok();
            // If async command: awaits until complete
            // If sync command: returns immediately
        }

        // 3. FLUSH output
        io.flush().await.ok();
    }
}
```

**Behavior:**
- User types: `http-get https://example.com`
- `process_char_async()` awaits the HTTP request
- CLI blocked during request, but other Embassy tasks continue
- Response displayed when complete
- No manual spawning, no global state, no polling needed!

**Benefits over manual spawning:**
- ✅ No global `SPAWNER` statics
- ✅ No global result tracking mutexes
- ✅ No manual polling or status commands
- ✅ Direct error propagation via `?`
- ✅ 3 lines instead of 20+ lines

Commands use the metadata/execution separation pattern (CommandMeta + CommandHandlers trait) which enables natural async command support. See [DESIGN.md](DESIGN.md) section 1 for complete architecture details.

## Summary

**Design decision:** CharIo implementations MUST buffer output. Flush timing is platform-dependent:

- **Bare-metal:** Immediate flush (blocking acceptable)
- **Async:** Deferred flush (manual, after process_char)

**Async command support:**
- Commands can be marked as `Async` via `CommandKind`
- Use `process_char_async()` to await async commands inline
- Natural async/await without manual spawning

**Benefits:**
- ✅ Works in both bare-metal and async runtimes (Embassy, RTIC, etc.)
- ✅ Zero overhead for bare-metal
- ✅ Efficient batching for async
- ✅ Single CliService implementation
- ✅ No async trait complexity (uses handler pattern)
- ✅ Stable Rust compatible
- ✅ Natural async/await for async commands

**Implementation:**
- Bare-metal: `put_char()` writes to UART directly, sync commands only
- Async: `put_char()` writes to buffer, `flush()` called externally, supports async commands

**For complete examples:** See the `examples/` directory for reference implementations on different platforms.
