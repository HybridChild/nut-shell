# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

**Important**: Before adding to or modifying the information in this file, always consider the following:
- This file should only contain information that is useful for Claude Code, not for human developers.
- The information in this file should be formatted and presented in the way that is optimal for Claude Code, not for human developers.

## Project Overview

**cli-service** is a lightweight library for adding a flexible command-line interface to embedded systems. The implementation targets **no_std** environments with static allocation, specifically designed for platforms like the Raspberry Pi Pico (RP2040).

**Current Status:** Architecture complete, implementation in progress (see IMPLEMENTATION.md for roadmap).

**Important Note on Design Evolution:**
The architectural decisions and patterns documented here represent our current best thinking, not immutable requirements. During implementation, if you identify a better design approach or discover issues with the current plan:
- **Feel free to suggest improvements** - your insights during implementation are valuable
- **Ask before executing architectural changes** - discuss alternatives before modifying core design decisions
- **Small improvements are fine** - refining implementation details within the existing architecture doesn't need approval
- **Documentation is a snapshot** - treat specs as guidance that can evolve, not rigid constraints

---

## Documentation Navigation

**When to consult each document:**

| Need | Document | What You'll Find |
|------|----------|------------------|
| Exact behavior (I/O, auth, commands) | **docs/SPECIFICATION.md** | Terminal sequences, password masking, command syntax, startup behavior |
| Why architecture chosen this way | **docs/ARCHITECTURE.md** | Design rationale, unified architecture pattern, feature gating |
| Implementation order and tasks | **docs/IMPLEMENTATION.md** | 10-phase roadmap, task breakdown, what to build next |
| Security patterns and credential storage | **docs/SECURITY.md** | Password hashing, access control, system user concept |

---

## Quick Reference - Common Tasks

### Adding a New Command

```rust
// 1. Define the command function
fn reboot_fn<L: AccessLevel>(args: &[&str]) -> Result<Response, CliError> {
    // Implementation
    Ok(Response::success("Rebooting..."))
}

// 2. Create const command definition
const REBOOT: Command<MyAccessLevel> = Command {
    name: "reboot",
    description: "Reboot the device",
    execute: reboot_fn,
    access_level: MyAccessLevel::Admin,
    min_args: 0,
    max_args: 0,
};

// 3. Add to tree
const SYSTEM_DIR: &[Node<MyAccessLevel>] = &[
    Node::Command(&REBOOT),
    // ... other nodes
];
```

### Implementing Global Commands (help, ?, clear, logout)

Global commands are reserved keywords handled outside the tree structure.

**Help command output format:**
```rust
fn help_command() -> Response {
    let mut output = heapless::String::<256>::new();

    output.push_str("  help      - List global commands\r\n").ok();
    output.push_str("  ?         - Detail items in current directory\r\n").ok();

    #[cfg(feature = "authentication")]
    output.push_str("  logout    - Exit current session\r\n").ok();

    output.push_str("  clear     - Clear screen\r\n").ok();
    output.push_str("  ESC ESC   - Clear input buffer\r\n").ok();

    Response::success(&output)
}
```

**Important:**
- `ESC ESC` is not a command (it's a keyboard shortcut), but include it in `help` for discoverability
- `logout` only shown when authentication feature enabled
- Use consistent spacing/alignment for readability

### Implementing a Feature-Gated Module

For complete feature gating patterns, configuration examples, and build instructions, see ARCHITECTURE.md "Feature Gating & Optional Features" section.

**Recommended Pattern: Stub Functions** (aligns with unified architecture)

```rust
// src/tree/my_feature.rs - Module always exists
#![cfg_attr(not(feature = "my_feature"), allow(unused_variables))]

// Feature-enabled: Full implementation
#[cfg(feature = "my_feature")]
pub fn do_something<L: AccessLevel>(
    node: &Node<L>,
    input: &str,
) -> Result<heapless::Vec<&str, 32>, CliError> {
    // Real implementation
}

// Feature-disabled: Stub with identical signature
#[cfg(not(feature = "my_feature"))]
pub fn do_something<L: AccessLevel>(
    _node: &Node<L>,
    _input: &str,
) -> Result<heapless::Vec<&str, 32>, CliError> {
    Ok(heapless::Vec::new())  // No-op/empty result
}
```

```rust
// src/tree/mod.rs
pub mod my_feature;  // Always include (contents are gated)
pub use my_feature::do_something;

// src/cli/mod.rs - NO feature gates needed!
impl<'tree, L, IO> CliService<'tree, L, IO> {
    fn some_method(&mut self) -> Result<(), CliError> {
        // Works in both modes - stub returns empty when disabled
        let results = my_feature::do_something(node, input)?;

        if !results.is_empty() {
            // Process results
        }
        // Empty = feature disabled, naturally no-op

        Ok(())
    }
}
```

**Why this pattern?**
- Single code path (no duplicate implementations)
- Zero `#[cfg]` in main service code
- Compiler optimizes away stub calls
- Aligns with unified architecture principle

**Pattern Variations:**

For stateful types (like `CommandHistory`):
```rust
// Feature-enabled: Full struct
#[cfg(feature = "history")]
pub struct CommandHistory<const N: usize> {
    buffer: heapless::Vec<heapless::String<128>, N>,
    position: Option<usize>,
}

// Feature-disabled: Zero-size stub
#[cfg(not(feature = "history"))]
pub struct CommandHistory<const N: usize> {
    _phantom: core::marker::PhantomData<[(); N]>,
}

// Both modes implement identical API
impl<const N: usize> CommandHistory<N> {
    pub fn new() -> Self { /* ... */ }
    pub fn add(&mut self, cmd: &str) { /* real or no-op */ }
    pub fn previous(&mut self) -> Option<heapless::String<128>> { /* real or None */ }
}
```

**Alternative (when stub pattern doesn't fit):**
```rust
#[cfg(feature = "my_feature")]
pub mod my_module;

#[cfg(not(feature = "my_feature"))]
impl SomeType {
    pub fn feature_method(&self) -> Result<()> { Ok(()) }
}
```

### Implementing AccessLevel Trait

```rust
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum MyAccessLevel {
    Guest = 0,
    User = 1,
    Admin = 2,
}

impl AccessLevel for MyAccessLevel {
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
```

### Creating a New CharIo Implementation

```rust
pub struct MyIo {
    // Platform-specific fields
}

impl CharIo for MyIo {
    type Error = MyError;

    fn get_char(&mut self) -> Result<Option<char>, Self::Error> {
        // Non-blocking read
    }

    fn put_char(&mut self, c: char) -> Result<(), Self::Error> {
        // Write character
    }
}
```

### Implementing Double-ESC Clear

Double-ESC clears the input buffer and exits history navigation. This is NOT feature-gated (always enabled).

**Parser state machine:**
```rust
enum ParserState {
    Normal,
    EscapeStart,    // Saw first ESC
    EscapeSequence, // Saw ESC [
}

match (self.state, c) {
    // Double ESC = clear
    (ParserState::EscapeStart, '\x1b') => {
        buffer.clear();
        self.state = ParserState::Normal;
        Ok(ParseEvent::ClearAndRedraw)
    }

    // ESC [ = sequence start (arrow keys)
    (ParserState::EscapeStart, '[') => {
        self.state = ParserState::EscapeSequence;
        Ok(ParseEvent::None)
    }

    // ESC + other = clear, then process char
    (ParserState::EscapeStart, other) => {
        buffer.clear();
        self.state = ParserState::Normal;
        self.process_char(other, buffer)  // Re-process
    }
}
```

**Why not feature-gated?** Minimal overhead (~50-100 bytes flash, 0 bytes RAM), high UX value.

### Testing a Module

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_all_features() {
        // Test default configuration
    }

    #[test]
    #[cfg(not(feature = "authentication"))]
    fn test_without_auth() {
        // Test feature-disabled path
    }
}
```

---

## Core Architecture Patterns

### Unified Architecture (Auth-Enabled vs Auth-Disabled)

**IMPORTANT: Use a single code path for both authentication modes. Do NOT create duplicate implementations.**

**Implementation pattern:**
```rust
pub struct CliService<'tree, L, IO> {
    current_user: Option<User<L>>,  // Always present (not feature-gated)
    state: CliState,                // Always present (not feature-gated)

    #[cfg(feature = "authentication")]
    credential_provider: &'tree dyn CredentialProvider<L>,  // Only this field is conditional

    // ... other fields
}
```

**Constructor behavior:**
- Auth enabled: `current_user = None`, `state = CliState::LoggedOut`
- Auth disabled: `current_user = None`, `state = CliState::LoggedIn`

**State and user semantics:**
- `state = LoggedOut, current_user = None` → Awaiting login (auth enabled)
- `state = LoggedIn, current_user = Some(user)` → Authenticated (auth enabled)
- `state = LoggedIn, current_user = None` → Auth disabled (no user needed)

**State-driven behavior (minimal `#[cfg]` branching):**
- Let `CliState` variant determine behavior, not feature flags
- Single `activate()`, `generate_prompt()`, `check_access()` implementation
- Feature gates only where absolutely necessary (constructor, credential provider)

**See ARCHITECTURE.md** for complete pattern with code examples.

---

## Critical Constraints

### no_std Environment
- **No heap allocation**: Cannot use `Vec`, `String`, `Box`, etc.
- **Use `heapless` instead**: `heapless::Vec<T, N>`, `heapless::String<N>`
- **Fixed sizes at compile time**: Must specify maximum capacity
- **What happens when full**: Operations return errors, not panics
- **Core dependencies only**: Check `default-features = false` for all crates

### Static Allocation Requirements
- **Everything const-initializable**: Trees, commands, directories must be `const`
- **Lives in ROM**: Data placed in flash memory, not RAM
- **No runtime initialization**: No `lazy_static`, no `once_cell`
- **Function pointers, not closures**: Use `fn` pointers for command execution

### Path Stack Navigation
- **Current directory = index path from root**: Not a pointer to current node
- **Navigate down**: Push child index onto stack
- **Navigate up (..)**: Pop index from stack
- **Get current node**: Walk from root following indices
- **Why**: Enables const initialization (no self-referential pointers)

### String Handling
- **Const strings**: Use `&'static str` for names, descriptions, help text
- **Runtime buffers**: Use `heapless::String<N>` with explicit capacity
- **Parsing**: Work with `&str` slices, avoid allocation
- **Buffer sizes**: Choose carefully (MAX_INPUT: 128, MAX_PATH_DEPTH: 8, etc.)

---

## Core Architecture

### Node Type System
```rust
enum Node<L: AccessLevel> {
    Command(&'static Command<L>),
    Directory(&'static Directory<L>),
}
```
- **Zero-cost dispatch**: Pattern matching instead of vtable
- **Const-friendly**: Can initialize at compile time
- **ROM placement**: Entire tree lives in flash

### Service Generics
```rust
CliService<'tree, L, IO>
where
    L: AccessLevel,    // User-defined access hierarchy
    IO: CharIo,        // Platform-specific I/O
```
- **Monomorphization**: Compiler generates specialized code per type
- **Zero overhead**: No runtime dispatch, fully inlined
- **Lifetime `'tree`**: References tree data (static or const)

### Module Structure

See ARCHITECTURE.md for complete module structure, feature gating patterns, and organization rationale.

---

## Common Pitfalls & Solutions

### ❌ Forgetting Feature Gates
```rust
// WRONG (old pattern): Always compiles
use crate::tree::completion;

// OLD RIGHT: Conditional imports
#[cfg(feature = "completion")]
use crate::tree::completion;

// BETTER (stub pattern): Module always available, contents gated
// src/tree/completion.rs provides stub when feature disabled
pub mod completion;  // No #[cfg] needed!
use crate::tree::completion::suggest_completions;  // Works always
```

**Prefer stub function pattern** (see "Implementing a Feature-Gated Module" above) to minimize `#[cfg]` branching. See ARCHITECTURE.md for complete feature gating patterns.

### ❌ Using std Types
```rust
// WRONG: std types in no_std
fn parse(input: String) -> Vec<&str> { }

// RIGHT: heapless or slices
fn parse(input: &str) -> heapless::Vec<&str, 16> { }
```

### ❌ Runtime Initialization
```rust
// WRONG: Can't initialize at runtime
const TREE: Vec<Node> = vec![...];

// RIGHT: Const initialization
const TREE: &[Node] = &[...];
```

### ❌ Dynamic Dispatch for Commands
```rust
// WRONG: Trait objects prevent const init
trait Command { fn execute(&self); }
const CMD: &dyn Command = &MyCommand;

// RIGHT: Function pointers
type ExecuteFn = fn(&[&str]) -> Result<Response>;
const CMD: Command = Command { execute: my_fn, ... };
```

### ❌ Mutable Static Without Synchronization
```rust
// WRONG: Unsafe mutable global
static mut STATE: State = State::new();

// RIGHT: Use Mutex or atomic types (if needed at all)
// Or better: pass as parameter through CliService
```

### ⚠️ heapless Buffer Overflow
```rust
// WRONG: Doesn't handle full buffer
let mut buf: heapless::String<64> = heapless::String::new();
buf.push_str(&long_string); // Can panic!

// RIGHT: Handle capacity errors
buf.push_str(&long_string).map_err(|_| Error::BufferFull)?;
```

### ❌ Using N=0 Instead of Feature Gating
```rust
// WRONG: Zero-capacity saves RAM but not flash
type History = CommandHistory<0>;  // Code still compiled, just unused

// RIGHT: Feature gate to eliminate code entirely
#[cfg(feature = "history")]
type History = CommandHistory<10>;

#[cfg(not(feature = "history"))]
type History = CommandHistory<0>;  // Stub compiled instead
```

**Rule of thumb:** If disabling functionality should save flash (not just RAM), use feature gating with stub pattern.

---

## Testing Patterns

### Test Const Initialization
```rust
#[test]
fn test_tree_is_const() {
    // Just referencing TREE proves it compiles as const
    let _tree = &EXAMPLE_TREE;
}
```

### Mock I/O for Testing
```rust
struct MockIo {
    input: VecDeque<char>,
    output: Vec<char>,
}

impl CharIo for MockIo {
    type Error = ();
    fn get_char(&mut self) -> Result<Option<char>> {
        Ok(self.input.pop_front())
    }
    fn put_char(&mut self, c: char) -> Result<()> {
        self.output.push(c);
        Ok(())
    }
}
```

### Integration Test Pattern
```rust
#[test]
fn test_login_and_command() {
    let mut io = MockIo::new("admin:pass123\nsystem/reboot\n");
    let mut service = CliService::new(&TREE, provider, &mut io);

    // Process login
    service.process_char('a');
    service.process_char('d');
    // ... assert state changes
}
```

### Testing Escape Sequences
```rust
#[test]
fn test_double_esc_clears_buffer() {
    let mut parser = InputParser::new();
    let mut buffer = heapless::String::<128>::new();
    buffer.push_str("some input").unwrap();

    // First ESC
    let event = parser.process_char('\x1b', &mut buffer).unwrap();
    assert_eq!(event, ParseEvent::None);  // Waiting for next char
    assert_eq!(buffer.as_str(), "some input");  // Not cleared yet

    // Second ESC
    let event = parser.process_char('\x1b', &mut buffer).unwrap();
    assert_eq!(event, ParseEvent::ClearAndRedraw);
    assert_eq!(buffer.as_str(), "");  // Cleared!
}

#[test]
fn test_esc_bracket_is_sequence() {
    let mut parser = InputParser::new();
    let mut buffer = heapless::String::<128>::new();

    // ESC [ A (up arrow)
    parser.process_char('\x1b', &mut buffer).unwrap();
    parser.process_char('[', &mut buffer).unwrap();
    let event = parser.process_char('A', &mut buffer).unwrap();

    assert_eq!(event, ParseEvent::UpArrow);  // Not cleared!
}
```

---

## Common Build Commands - Quick Reference

Quick reference for frequent operations:

```bash
# Development
cargo check                              # Fast compile check
cargo test                               # Run tests (all features enabled by default)
cargo clippy                             # Lint code
cargo fmt                                # Format code

# Feature testing
cargo test --all-features                # Test with all features (authentication, completion, history)
cargo test --no-default-features         # Test minimal configuration (no optional features)
cargo test --features authentication     # Test auth only
cargo test --features completion,history # Test interactive features only

# Embedded target
cargo check --target thumbv6m-none-eabi  # Verify no_std compliance
cargo build --target thumbv6m-none-eabi --release  # Release build (all features)
cargo build --target thumbv6m-none-eabi --release --no-default-features  # Minimal build
cargo size --target thumbv6m-none-eabi --release -- -A  # Measure binary size

# Size optimization comparisons
cargo size --target thumbv6m-none-eabi --release --all-features -- -A
cargo size --target thumbv6m-none-eabi --release --no-default-features --features authentication -- -A

# Pre-commit
cargo fmt && cargo clippy --all-features -- -D warnings && cargo test --all-features
```

**For comprehensive build workflows, CI configuration, and troubleshooting:** See IMPLEMENTATION.md

---

## Implementation Workflow

**Current Phase:** See IMPLEMENTATION.md for detailed task breakdown

**General Approach:**
1. **Consult SPECIFICATION.md** for exact behavior to implement
2. **Check IMPLEMENTATION.md** for current phase and tasks
3. **Write tests first** based on behavioral specification
4. **Implement minimal functionality** to pass tests
5. **Test on native target** (`cargo test`)
6. **Verify embedded target** (`cargo check --target thumbv6m-none-eabi`)
7. **Verify feature combinations** (test with/without features)
8. **Document public APIs** with doc comments

**When stuck:**
- SPECIFICATION.md for "what should this do?"
- ARCHITECTURE.md for "why is it designed this way?"
- SECURITY.md for authentication/access control specifics
- This file (CLAUDE.md) for constraints and patterns
- **If the documented approach seems problematic**: Ask! Design can evolve based on implementation insights
