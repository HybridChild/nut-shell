# `CharIo` Trait Design

This document explains the `CharIo` trait design that enables nut-shell to work efficiently in both bare-metal and async runtime environments.

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

## Solution: Platform-Specific Output Handling

### Core Principle

`CharIo` implementations handle output based on platform constraints:

- **Bare-metal:** Write directly to hardware (blocking acceptable, no buffering needed)
- **Async runtimes:** Buffer to memory, flush externally (batches output)

### `CharIo` Trait Design

```rust
pub trait CharIo {
    type Error;

    /// Read a character if available (non-blocking).
    /// Returns `Ok(None)` if no character ready.
    fn get_char(&mut self) -> Result<Option<char>, Self::Error>;

    /// Write a character to output buffer.
    /// MUST NOT block indefinitely.
    fn put_char(&mut self, c: char) -> Result<(), Self::Error>;

    /// Write a string to output buffer.
    /// Default implementation calls `put_char()` for each character.
    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        for c in s.chars() {
            self.put_char(c)?;
        }
        Ok(())
    }
}
```

### How It Solves The Problem

**Bare-metal implementation:**
```rust
impl CharIo for UartIo {
    fn put_char(&mut self, c: char) -> Result<(), Self::Error> {
        self.uart.write_byte(c as u8);  // Blocks until TX ready - acceptable
        Ok(())
    }
}

// Main loop
loop {
    if uart.is_readable() {
        let c = uart.read_byte();
        shell.process_char(c)?;  // Output written immediately via put_char()
    }
}
```

**Async implementation:**
```rust
impl CharIo for EmbassyUsbIo {
    fn put_char(&mut self, c: char) -> Result<(), Self::Error> {
        self.buffer.push(c as u8).map_err(|_| Error::BufferFull)  // Memory only
    }
}

impl EmbassyUsbIo {
    pub async fn flush(&mut self) -> Result<()> {
        self.class.write_packet(&self.buffer).await  // Async I/O happens here
    }
}

// Main loop
loop {
    let c = uart.read().await;
    shell.process_char_async(c).await?;  // Output buffered to memory
    io.flush().await?;                   // Async I/O happens outside shell
}
```

**Key insight:** `put_char()` never blocks indefinitely. Bare-metal blocks briefly (hardware-limited), async writes to memory (error if full).

---

## Why This Design Works

The trait solves the async output problem by **separating write semantics from I/O timing**:

- `put_char()` writes output (immediate for bare-metal, buffered for async)
- Actual I/O timing is platform-specific (blocking vs deferred flush)
- `Shell` doesn't need to know which approach is used

**Bare-metal:**
- Zero abstraction overhead - `put_char()` compiles to direct UART writes
- No buffer allocation required
- Blocking is acceptable in single-threaded loop

**Async:**
- Memory buffer replaces blocking I/O
- `Shell` outputs immediately (to buffer), task `.await`s only on flush
- Batched writes reduce I/O overhead

**Result:** Same `Shell` code works in both environments without feature flags in the core logic.

**Architecture rationale:** See [DESIGN.md](DESIGN.md) for design decision and rejected alternatives.

---

## Implementation Requirements

### Output Buffer Sizing (Async Only)

Async implementations must buffer output. **Minimum size:** `MAX_RESPONSE + MAX_PROMPT + Overhead`

| ShellConfig | MAX_RESPONSE | MAX_PROMPT | Overhead | Recommended Buffer |
|-------------|--------------|------------|----------|-------------------|
| `DefaultConfig` | 256 | 128 | 16 | **400 bytes** |
| `MinimalConfig` | 128 | 64 | 16 | **208 bytes** |
| Custom config | Variable | Variable | 16 | Use formula |

**Overhead (constant 16 bytes):**
- Response formatting: `"\r\n"` prefix/postfix, `"  "` indentation (~6 bytes)
- Error prefix: `"\r\n  Error: "` (11 bytes)
- Worst case: 16 bytes total

**Buffer smaller than recommended:** `Shell` output may overflow on error messages or verbose responses.

**On overflow:** `put_char()` returns error, `Shell` propagates to user.

**Bare-metal:** No output buffer needed (immediate flush).

### Input Buffer (`Shell` Configuration)

The input buffer stores the current command line being edited. It is **managed by `Shell`**, not `CharIo`.

**Configuration:**

| ShellConfig | MAX_INPUT | Use Case |
|-------------|-----------|----------|
| `DefaultConfig` | 128 bytes | Standard command lines |
| `MinimalConfig` | 64 bytes | Constrained environments |
| Custom config | Variable | Application-specific |

**Key points:**
- `CharIo` only handles single-character I/O (`get_char()`/`put_char()`)
- `Shell` accumulates input characters into `MAX_INPUT`-sized buffer
- When user presses Enter, `Shell` parses the buffered command
- Input buffer overflow triggers error (command rejected)

**See:** [EXAMPLES.md](EXAMPLES.md#custom-configuration) for custom `ShellConfig` implementation

---

## Platform Examples

Complete working implementations are in the `examples/` directory:

| Platform | Example | Key Pattern |
|----------|---------|-------------|
| **Bare-metal UART (STM32)** | `examples/stm32f072/bin/basic/` | Blocking UART writes (`io.rs`) |
| **Bare-metal USB (RP2040)** | `examples/rp-pico/bin/basic/` | Blocking USB-CDC writes (`io.rs`) |
| **Embassy USB (RP2040)** | `examples/rp-pico/bin/embassy/` | Buffered output with async flush (`io.rs`) |
| **Native (sync)** | `examples/native/bin/basic/` | Stdio with immediate flush (`io.rs`) |
| **Native (async)** | `examples/native/bin/async/` | Buffered stdio (`io.rs`) |

**See `io.rs` in each example for complete `CharIo` implementations and platform-specific setup.**