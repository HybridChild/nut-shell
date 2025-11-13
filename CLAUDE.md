# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This repository contains a Rust port of cli-service, a lightweight library for adding a flexible command-line interface to embedded systems. The implementation targets no_std environments with static allocation, specifically designed for platforms like the Raspberry Pi Pico (RP2040).

**Note:** The original C++ implementation was located in the `CLIService/` subdirectory. All necessary behavioral specifications have been extracted and documented in `docs/SPECIFICATION.md`, so the C++ directory is no longer required as a reference.

**Key Documentation:**
- **docs/SPECIFICATION.md**: Complete behavioral specification derived from C++ implementation
- **docs/ARCHITECTURE.md**: Design decisions, rationale, and comparison with C++ implementation
- **docs/IMPLEMENTATION.md**: Phased implementation plan and current status tracking (temporary, will be archived)
- **docs/SECURITY.md**: Authentication, access control, and security design

## Common Build Commands

```bash
# Standard development
cargo build                    # Debug build (all default features)
cargo build --release          # Release build
cargo test                     # Run all tests
cargo test <test_name>         # Run specific test
cargo run --example basic      # Run basic example

# Feature configuration
cargo build --all-features                         # All optional features enabled
cargo build --no-default-features                  # Minimal build (no auth, no completion)
cargo build --features authentication              # Authentication only
cargo build --features completion                  # Tab completion only
cargo build --features "authentication,completion" # Explicit feature combination

# Embedded target (Raspberry Pi Pico)
cargo build --target thumbv6m-none-eabi --release  # Full-featured embedded build
cargo build --target thumbv6m-none-eabi --release --no-default-features  # Minimal embedded

# Size optimization
cargo build --target thumbv6m-none-eabi --release --no-default-features --features authentication
cargo size --release -- -A                         # Measure binary size
```

## Core Architectural Decisions

### Polymorphism
`Node` enum with `Command` and `Directory` variants instead of trait objects. Provides zero-cost dispatch via pattern matching, enables const initialization.

### Memory Management
Static allocation only using `heapless` crate for fixed-size buffers. No heap allocation, deterministic memory usage.

### Navigation Pattern (Path Stack)
CLI service maintains current location as vector of child indices (`heapless::Vec<usize, MAX_DEPTH>`) instead of parent pointers. Navigate down by pushing child index, up by popping. This enables:
- Const-initialized directory trees
- Nodes entirely in ROM on embedded devices
- Simple lifetime management (no self-references)

To get current directory: walk down from root using stored indices. To build prompt: iterate path stack and collect names.

### I/O Abstraction
`CharIo` trait with generic type parameters for zero-cost abstraction:
```rust
CliService<'tree, L, IO> where L: AccessLevel, IO: CharIo
```

Platform-specific implementations (UART, USB-CDC, stdio) created separately. Compiler monomorphizes per implementation (no vtable overhead). Pattern follows `embedded-hal` conventions.

### Access Control
`AccessLevel` as generic trait bound. User defines enum or type implementing trait with comparison operators. Provides compile-time type safety.

### Error Handling
`Result<Response, CliError>` pattern throughout. Each error variant represents specific failure mode (InvalidArguments, AccessDenied, etc).

### String Storage
- `&'static str` for constant names and descriptions (zero runtime cost)
- `heapless::String<N>` for runtime buffers (fixed-size, stack-allocated)

### Commands
Struct with function pointer field, enabling const initialization and ROM placement:
```rust
const COMMAND: Command = Command {
    name: "reboot",
    execute: reboot_fn,
    ...
};
```

## Implementation Patterns

### Path Stack Navigation
Service tracks location as indices into children arrays stored in a `heapless::Vec<usize, MAX_DEPTH>`. When user navigates to "system/debug", service pushes indices corresponding to those child names. Parent navigation (..) pops indices. Absolute paths clear the stack before navigation.

Walking the path stack to get current directory: start at root, index into children array repeatedly using stored indices.

**Implementation approach:**
- `Path` type handles parsing and normalization of path strings
- `Directory::resolve_path()` method performs the actual tree walking
- Path resolution returns `Option<&Node>` - None if path invalid
- Completion logic in `tree/completion` module uses similar traversal

### Generic I/O Trait
Each platform implements `CharIo` trait with platform-specific error type. Service generic over I/O type. Compiler generates optimized code per implementation, fully inlining I/O operations.

Example implementations: `PicoUart` (RP2040 UART), `StdioStream` (desktop testing), `UsbCdc` (USB serial).

### Directory Tree Construction
Trees defined as const arrays of nodes. Each node contains either command reference or array of child nodes. Entire structure can be const-initialized and placed in ROM:
```rust
const TREE: &[Node] = &[
    Node::directory("system", &SYSTEM_CHILDREN),
    Node::command(&REBOOT_CMD),
];
```

## Differences from C++ Implementation

The Rust port maintains behavioral compatibility while leveraging Rust idioms:

**Key differences:**
- C++ uses virtual inheritance for polymorphism → Rust uses `Node` enum with pattern matching
- C++ uses parent pointers for navigation → Rust uses path stack (indices vector)
- C++ allocates commands/directories dynamically → Rust uses static const initialization
- C++ uses `std::variant` for static/dynamic nodes → Rust uses single allocation pattern

**Behavioral specification:** See `docs/SPECIFICATION.md` for complete behavioral details extracted from the C++ implementation.

**Architectural decisions:** See `docs/ARCHITECTURE.md` for rationale behind structural choices and comparisons with C++.
