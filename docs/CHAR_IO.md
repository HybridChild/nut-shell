# CharIo Trait Design

This document explains the CharIo trait design that enables nut-shell to work efficiently in both bare-metal and async runtime environments.

**Related Documentation:**
- **[DESIGN.md](DESIGN.md)**: Architecture decisions including CharIo buffering model (Section 1.5)
- **[EXAMPLES.md](EXAMPLES.md)**: CharIo implementation examples
- **[DEVELOPMENT.md](DEVELOPMENT.md)**: Build and testing workflows

---

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

---

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

---

## Why This Design Works

### Single Codebase, Platform-Specific Behavior

Same `Shell` code works everywhere:
- No `#[cfg(async)]` feature flags in core
- CharIo trait is platform-agnostic
- Implementations handle platform-specific details

**Bare-metal benefits:**
- No output buffer allocation needed
- Direct UART writes
- Compiler optimizes away abstraction

**Async benefits:**
- Single buffer (~256 bytes)
- Batched I/O transactions
- Task suspends only on read/flush

**Architecture rationale:** See [DESIGN.md](DESIGN.md) Section 1.5 for design decision and rejected alternatives.

---

## Implementation Requirements

### Output Buffer Sizing (Async Only)

Async implementations must buffer output. Recommended sizes:

| Buffer Size | Use Case |
|-------------|----------|
| 256 bytes | Standard (handles prompts + directory listings) |
| 512 bytes | Verbose command responses |
| 1024 bytes | Maximum safety margin |

**On overflow:** Return error from `put_char()`, Shell propagates to user.

**Bare-metal:** No output buffer needed (immediate flush).

### Input Buffer (Shell Configuration)

Configured via `ShellConfig` trait (not CharIo):
- Default: 128 bytes (`DefaultConfig::MAX_INPUT`)
- Minimal: 64 bytes (`MinimalConfig::MAX_INPUT`)
- See [EXAMPLES.md](EXAMPLES.md#custom-configuration) for custom sizing

---

## Platform Examples

Complete working implementations are in the `examples/` directory:

| Platform | Example | Key Pattern |
|----------|---------|-------------|
| **Bare-metal UART** | `examples/pico_uart.rs` | ISR fills queue, blocking TX |
| **Embassy USB-CDC** | `examples/embassy_usb_cdc.rs` | Buffer + flush, `\r` → `\r\n` |
| **Embassy UART** | `examples/embassy_uart.rs` | DMA transfers, deferred flush |
| **Native** | `examples/native_simple.rs` | Stdio with immediate flush |

**See each example for complete CharIo implementations and platform-specific setup.**

---

## Summary

**Design decision:** CharIo implementations MUST buffer output. Flush timing is platform-dependent.

**Implementation:**
- Bare-metal: `put_char()` writes to UART directly
- Async: `put_char()` writes to buffer, `flush()` called externally

**Benefits:**
- Works in both bare-metal and async runtimes
- Zero overhead for bare-metal
- Efficient batching for async
- No async trait complexity
- Stable Rust compatible

**For architecture rationale and rejected alternatives, see [DESIGN.md](DESIGN.md) Section 1.5.**
