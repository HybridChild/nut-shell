# CharIo Design: Universal I/O Abstraction

This document explains the I/O abstraction design that enables nut-shell to work efficiently in both bare-metal and async runtime environments (like Embassy).

**Related Documentation:**
- **[DESIGN.md](DESIGN.md)**: Metadata/execution separation pattern (Section 1) - enables async command support
- **[EXAMPLES.md](EXAMPLES.md)**: CharIo implementation examples for various platforms
- **[DEVELOPMENT.md](DEVELOPMENT.md)**: Build and testing workflows

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
/// Character I/O abstraction for CLI.
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


## Why This Design Works

### 1. Respects Shell Ownership Model

`Shell` owns/references the `CharIo`:

```rust
impl<'tree, L, IO, H, C> Shell<'tree, L, IO, H, C>
where
    L: AccessLevel,
    IO: CharIo,
    H: CommandHandler<C>,
    C: ShellConfig,
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

Same `Shell` code works everywhere:
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
- Shell will propagate error
- Alternatively: flush mid-process (complex, not recommended)

### Input Buffer (Both Platforms)

Fixed at **128 bytes** (defined in Shell):
- Sufficient for command paths + arguments
- `heapless::String<128>` in Shell struct

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

## Async Command Support

Commands can be marked as `Async` via `CommandKind` in their metadata. When using `process_char_async()`, async commands will await completion inline without requiring manual task spawning or global state.

**Architecture:**
- Command metadata marks execution mode (`CommandKind::Sync` or `CommandKind::Async`)
- Handler trait provides both `execute_sync()` and `execute_async()` methods
- Shell dispatches based on command kind

**Benefits:**
- ✅ Natural async/await without manual spawning
- ✅ No global state or result tracking needed
- ✅ Direct error propagation
- ✅ Single code path (metadata/execution separation pattern)

See [DESIGN.md](DESIGN.md) section 1 for complete architecture details and implementation patterns.

## Implementation Requirements for CharIo

### Buffer Sizing Constraints

**Input buffer (Shell):**
- Default: 128 bytes (`DefaultConfig::MAX_INPUT`)
- Configurable via `ShellConfig` trait (e.g., `MinimalConfig` uses 64 bytes)
- Overflow: Characters silently ignored, no error displayed
- Range: 64-256 bytes typical
- See [EXAMPLES.md](EXAMPLES.md#configuration-examples) for configuration details

**Output buffer (async CharIo implementations only):**
- Recommended: 256 bytes (heapless::Vec<u8, 256>)
- Must handle longest response without overflow
- Overflow: Return `Err(BufferFull)`, response truncated
- Bare-metal: No output buffer needed (immediate flush)

**Other Shell const generics:**
- MAX_PATH_DEPTH: 8 (default), 4-16 range, affects path stack size
- MAX_ARGS: 16 (default), 8-32 range, stack-only during parsing
- HISTORY_SIZE: 10 (default), 0-20 range, each entry ~130 bytes RAM

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
- ✅ Single Shell implementation
- ✅ No async trait complexity (uses handler pattern)
- ✅ Stable Rust compatible
- ✅ Natural async/await for async commands

**Implementation:**
- Bare-metal: `put_char()` writes to UART directly, sync commands only
- Async: `put_char()` writes to buffer, `flush()` called externally, supports async commands

---

## Platform Implementation Guidelines

### USB-CDC (Embassy Async)
**Critical points:**
- Normalize line endings: `\r` → `\r\n` for terminals
- Track disconnect state, return error when disconnected
- Buffer to `heapless::Vec<u8, 256>`, flush after `process_char_async()`
- USB packets are 64 bytes, batch multiple chars per transfer

### UART Bare-Metal (Interrupt-Driven)
**Critical points:**
- ISR fills queue (`heapless::Deque<u8, 64>`), main loop reads
- Use `cortex_m::interrupt::free()` for critical sections
- Blocking TX acceptable (no tasks to block)
- Never call `process_char()` from ISR (not ISR-safe)

### UART Embassy Async (DMA)
**Critical points:**
- Separate read task and shell task
- DMA for zero-copy transfers
- Buffer to `heapless::Vec`, flush via `write_all().await`
- Handle UART errors gracefully (don't crash shell)

**For complete working implementations, see `examples/` directory.**

---
