# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

**Important**: Before adding to or modifying the information in this file, always consider the following:
- This file should only contain information that is useful for Claude Code, not for human developers.
- The information in this file should be formatted and presented in the way that is optimal for Claude Code, not for human developers.

## Project Overview

**nut-shell** is a lightweight library for adding a flexible command-line interface to embedded systems. The implementation targets **no_std** environments with static allocation, specifically designed for platforms like the Raspberry Pi Pico (RP2040).

_A complete CLI framework for embedded systems, in a nutshell._

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
| Exact behavior (I/O, auth, commands) | **[docs/SPECIFICATION.md](docs/SPECIFICATION.md)** | Terminal sequences, password masking, command syntax, startup behavior |
| Why it's designed this way | **[docs/DESIGN.md](docs/DESIGN.md)** | Design rationale, unified architecture pattern, feature gating |
| How system works at runtime | **[docs/INTERNALS.md](docs/INTERNALS.md)** | Complete data flow, state machines, pseudocode implementations |
| Implementation order and tasks | **[docs/IMPLEMENTATION.md](docs/IMPLEMENTATION.md)** | 10-phase roadmap, task breakdown, what to build next |
| Exact type definitions and signatures | **[docs/TYPE_REFERENCE.md](docs/TYPE_REFERENCE.md)** | Complete struct fields, method signatures, constants, error types |
| Security patterns and credential storage | **[docs/SECURITY.md](docs/SECURITY.md)** | Password hashing, access control, authentication flow |
| Design philosophy and feature criteria | **[docs/PHILOSOPHY.md](docs/PHILOSOPHY.md)** | What we include/exclude, decision framework |
| CharIo implementation and buffering | **[docs/IO_DESIGN.md](docs/IO_DESIGN.md)** | Sync/async I/O patterns, buffering model, platform adapters |

---

## Quick Reference - Common Tasks

### Adding a New Command

**Commands use metadata/execution separation pattern** (CommandMeta + CommandHandlers trait). See [docs/DESIGN.md](docs/DESIGN.md) section 1 for complete architecture details.

```rust
// 1. Define configuration (choose or create custom)
type MyConfig = DefaultConfig;  // or MinimalConfig, or custom

// 2. Define the command function (sync or async)
fn reboot_fn<C: ShellConfig>(args: &[&str]) -> Result<Response<C>, CliError> {
    // Synchronous implementation
    Ok(Response::success("Rebooting..."))
}

async fn http_get_async<C: ShellConfig>(args: &[&str]) -> Result<Response<C>, CliError> {
    // Asynchronous implementation (requires async feature)
    let response = HTTP_CLIENT.get(args[0]).await?;
    Ok(Response::success(&response))
}

// 3. Create const command metadata (no execute function)
const REBOOT: CommandMeta<MyAccessLevel> = CommandMeta {
    name: "reboot",
    description: "Reboot the device",
    access_level: MyAccessLevel::Admin,
    kind: CommandKind::Sync,  // Mark as sync
    min_args: 0,
    max_args: 0,
};

const HTTP_GET: CommandMeta<MyAccessLevel> = CommandMeta {
    name: "http-get",
    description: "Fetch URL via HTTP",
    access_level: MyAccessLevel::User,
    kind: CommandKind::Async,  // Mark as async
    min_args: 1,
    max_args: 1,
};

// 4. Add to tree
const SYSTEM_DIR: Directory<MyAccessLevel> = Directory {
    name: "system",
    children: &[
        Node::Command(&REBOOT),
        Node::Command(&HTTP_GET),
        // ... other nodes
    ],
    access_level: MyAccessLevel::User,
};

// 5. Implement CommandHandlers trait (maps names to functions)
struct MyHandlers;

impl CommandHandlers<MyConfig> for MyHandlers {
    fn execute_sync(&self, name: &str, args: &[&str]) -> Result<Response<MyConfig>, CliError> {
        match name {
            "reboot" => reboot_fn::<MyConfig>(args),
            // ... other sync commands
            _ => Err(CliError::CommandNotFound),
        }
    }

    #[cfg(feature = "async")]
    async fn execute_async(&self, name: &str, args: &[&str]) -> Result<Response<MyConfig>, CliError> {
        match name {
            "http-get" => http_get_async::<MyConfig>(args).await,
            // ... other async commands
            _ => Err(CliError::CommandNotFound),
        }
    }
}

// 6. Instantiate Shell with handlers and config
let handlers = MyHandlers;
let mut shell: Shell<_, _, _, _, MyConfig> = Shell::new(&SYSTEM_DIR, handlers, io);
```

**Key points:**
- Command metadata (tree) is separate from execution logic (handlers)
- Commands marked as `Sync` or `Async` via `CommandKind`
- Handler trait dispatches by name to actual functions
- Async commands require `async` feature and use `process_char_async()`

### Implementing Global Commands (ls, ?, clear, logout)

Global commands are reserved keywords handled outside the tree structure.

**Help command (?) output format:**
```rust
fn help_command() -> Response {
    let mut output = heapless::String::<256>::new();

    output.push_str("  ?         - List global commands\r\n").ok();
    output.push_str("  ls        - Detail items in current directory\r\n").ok();

    #[cfg(feature = "authentication")]
    output.push_str("  logout    - Exit current session\r\n").ok();

    output.push_str("  clear     - Clear screen\r\n").ok();
    output.push_str("  ESC ESC   - Clear input buffer\r\n").ok();

    Response::success(&output)
}
```

**Important:**
- `ESC ESC` is not a command (it's a keyboard shortcut), but include it in `?` output for discoverability
- `logout` only shown when authentication feature enabled
- Use consistent spacing/alignment for readability

### Implementing a Feature-Gated Module

For complete feature gating patterns, configuration examples, and build instructions, see DESIGN.md "Feature Gating & Optional Features" section.

**Recommended Pattern: Stub Function Pattern** (aligns with unified architecture)

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

// src/shell/mod.rs - NO feature gates needed!
impl<'tree, L, IO, H, C> Shell<'tree, L, IO, H, C>
where
    H: CommandHandlers<C>,
    C: ShellConfig,
{
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

**Why use the stub function pattern?**
- Single code path (no duplicate implementations)
- Zero `#[cfg]` in main Shell code
- Compiler optimizes away stub calls
- Aligns with unified architecture pattern

**Pattern Variations:**

For stateful types (like `CommandHistory`):
```rust
// Feature-enabled: Full struct
#[cfg(feature = "history")]
pub struct CommandHistory<const N: usize, const INPUT_SIZE: usize> {
    buffer: heapless::Vec<heapless::String<INPUT_SIZE>, N>,
    position: Option<usize>,
}

// Feature-disabled: Zero-size stub
#[cfg(not(feature = "history"))]
pub struct CommandHistory<const N: usize, const INPUT_SIZE: usize> {
    _phantom: core::marker::PhantomData<[(); N]>,
}

// Both modes implement identical API
impl<const N: usize, const INPUT_SIZE: usize> CommandHistory<N, INPUT_SIZE> {
    pub fn new() -> Self { /* ... */ }
    pub fn add(&mut self, cmd: &str) { /* real or no-op */ }
    pub fn previous(&mut self) -> Option<heapless::String<INPUT_SIZE>> { /* real or None */ }
}
```

**Alternative (when stub function pattern doesn't fit):**
```rust
#[cfg(feature = "my_feature")]
pub mod my_module;

#[cfg(not(feature = "my_feature"))]
impl SomeType {
    pub fn feature_method(&self) -> Result<()> { Ok(()) }
}
```

### Implementing CommandHandlers Trait

The CommandHandlers trait maps command names to execution functions. Generic over ShellConfig to match Response buffer sizes.

```rust
pub trait CommandHandlers<C: ShellConfig> {
    /// Execute synchronous command
    fn execute_sync(&self, name: &str, args: &[&str]) -> Result<Response<C>, CliError>;

    /// Execute asynchronous command (optional, feature-gated)
    #[cfg(feature = "async")]
    async fn execute_async(&self, name: &str, args: &[&str]) -> Result<Response<C>, CliError>;
}
```

**Basic implementation:**
```rust
type MyConfig = DefaultConfig;  // Choose configuration

struct MyHandlers;

impl CommandHandlers<MyConfig> for MyHandlers {
    fn execute_sync(&self, name: &str, args: &[&str]) -> Result<Response<MyConfig>, CliError> {
        match name {
            "reboot" => reboot_fn::<MyConfig>(args),
            "status" => status_fn::<MyConfig>(args),
            "led-toggle" => led_toggle_fn::<MyConfig>(args),
            _ => Err(CliError::CommandNotFound),
        }
    }

    #[cfg(feature = "async")]
    async fn execute_async(&self, name: &str, args: &[&str]) -> Result<Response<MyConfig>, CliError> {
        match name {
            "http-get" => http_get_async::<MyConfig>(args).await,
            "wifi-connect" => wifi_connect_async::<MyConfig>(args).await,
            "flash-write" => flash_write_async::<MyConfig>(args).await,
            _ => Err(CliError::CommandNotFound),
        }
    }
}
```

**Stateful handlers (using shared references):**
```rust
type MyConfig = DefaultConfig;

struct MyHandlers<'a> {
    system: &'a SystemState,
}

impl<'a> CommandHandlers<MyConfig> for MyHandlers<'a> {
    fn execute_sync(&self, name: &str, args: &[&str]) -> Result<Response<MyConfig>, CliError> {
        match name {
            "status" => {
                let info = self.system.get_status();
                Ok(Response::success(&info))
            }
            _ => Err(CliError::CommandNotFound),
        }
    }
}
```

**Best practices:**
- Keep handler implementations simple (just dispatch)
- Command functions can access statics or captured state
- Use `CommandNotFound` for unrecognized commands
- Handler doesn't need to validate args (Shell does this)

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

### Metadata/Execution Separation Pattern

**IMPORTANT: Commands use metadata/execution separation pattern.** Command metadata (const in ROM) is separate from execution logic (generic trait). This enables both sync and async commands while maintaining const-initialization. See [docs/DESIGN.md](docs/DESIGN.md) section 1 for complete architecture details, rationale, and usage patterns.

```rust
// Metadata (const-initializable)
pub struct CommandMeta<L: AccessLevel> {
    pub name: &'static str,
    pub description: &'static str,
    pub access_level: L,
    pub kind: CommandKind,  // Sync or Async marker
    pub min_args: usize,
    pub max_args: usize,
}

// Execution logic (user-implemented trait, generic over config)
pub trait CommandHandlers<C: ShellConfig> {
    fn execute_sync(&self, name: &str, args: &[&str]) -> Result<Response<C>, CliError>;

    #[cfg(feature = "async")]
    async fn execute_async(&self, name: &str, args: &[&str]) -> Result<Response<C>, CliError>;
}

// Shell is generic over handlers and config
pub struct Shell<'tree, L, IO, H, C>
where
    H: CommandHandlers<C>,
    C: ShellConfig,
{
    handlers: H,
    // ... other fields
}
```

**Benefits:**
- Metadata stays const-initializable (lives in ROM)
- Async commands supported naturally (trait method can be async)
- Zero-cost for sync-only builds (monomorphization)
- Single codebase for both sync and async

### Unified Architecture (Auth-Enabled vs Auth-Disabled)

**IMPORTANT: Use a single code path for both authentication modes. Do NOT create duplicate implementations.**

**Implementation pattern:**
```rust
pub struct Shell<'tree, L, IO, H, C> {
    current_user: Option<User<L>>,  // Always present (not feature-gated)
    state: CliState,                // Always present (not feature-gated)
    handlers: H,                    // Generic over CommandHandlers<C>

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

**See DESIGN.md** for complete pattern with code examples.

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
    Command(&'static CommandMeta<L>),  // Metadata only
    Directory(&'static Directory<L>),
}
```
- **Zero-cost dispatch**: Pattern matching instead of vtable
- **Const-friendly**: Can initialize at compile time
- **ROM placement**: Entire tree lives in flash
- **Metadata-only**: Execution logic separate (via CommandHandlers trait)

### Shell Generics
```rust
Shell<'tree, L, IO, H, C>
where
    L: AccessLevel,    // User-defined access hierarchy
    IO: CharIo,        // Platform-specific I/O
    H: CommandHandlers<C>, // Command execution (sync/async dispatch)
    C: ShellConfig,    // Buffer sizes and capacity limits
```
- **Monomorphization**: Compiler generates specialized code per type
- **Zero overhead**: No runtime dispatch, fully inlined
- **Lifetime `'tree`**: References tree data (static or const)
- **Handler generic**: Enables both sync and async execution patterns

### Module Structure

See DESIGN.md for complete module structure, feature gating patterns, and organization rationale.

---

## Common Pitfalls & Solutions

### ❌ Forgetting Feature Gates
```rust
// WRONG (old pattern): Always compiles
use crate::tree::completion;

// OLD RIGHT: Conditional imports
#[cfg(feature = "completion")]
use crate::tree::completion;

// BETTER (stub function pattern): Module always available, contents gated
// src/tree/completion.rs provides stub when feature disabled
pub mod completion;  // No #[cfg] needed!
use crate::tree::completion::suggest_completions;  // Works always
```

**Prefer stub function pattern** (see "Implementing a Feature-Gated Module" above) to minimize `#[cfg]` branching. See DESIGN.md for complete feature gating patterns.

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
// Or better: pass as parameter through Shell
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
type History = CommandHistory<0, 128>;  // Code still compiled, just unused

// RIGHT: Feature gate to eliminate code entirely
#[cfg(feature = "history")]
type History = CommandHistory<10, 128>;

#[cfg(not(feature = "history"))]
type History = CommandHistory<0, 128>;  // Stub compiled instead
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
    let mut shell = Shell::new(&TREE, provider, &mut io);

    // Process login
    shell.process_char('a');
    shell.process_char('d');
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
- **[docs/SPECIFICATION.md](docs/SPECIFICATION.md)** for "what should this do?"
- **[docs/DESIGN.md](docs/DESIGN.md)** for "why is it designed this way?"
- **[docs/INTERNALS.md](docs/INTERNALS.md)** for "how does this work at runtime?"
- **[docs/TYPE_REFERENCE.md](docs/TYPE_REFERENCE.md)** for "what fields does this type have?" or "what's the signature?"
- **[docs/SECURITY.md](docs/SECURITY.md)** for authentication/access control specifics
- **[docs/PHILOSOPHY.md](docs/PHILOSOPHY.md)** for "should we add this feature?"
- **[docs/IO_DESIGN.md](docs/IO_DESIGN.md)** for "how do I implement CharIo?"
- **This file (CLAUDE.md)** for constraints and patterns
- **If the documented approach seems problematic**: Ask! Design can evolve based on implementation insights

---

## Documentation Terminology Conventions

To maintain consistency across all documentation, follow these conventions:

### Architecture Patterns
- **"metadata/execution separation pattern"** ✅ - Always include "pattern" suffix
- **"unified architecture pattern"** ✅ - Always include "pattern" suffix
- **"stub function pattern"** ✅ - For stateless feature-gated modules
- **"stub type pattern"** ✅ - For stateful feature-gated types (e.g., CommandHistory)

### Compound Adjectives (Always Hyphenate)
When used as adjectives before nouns, always hyphenate:
- **"path-based navigation"** ✅ not "path based navigation"
- **"feature-gated module"** ✅ not "feature gated module"
- **"user-defined hierarchy"** ✅ not "user defined hierarchy"
- **"const-initializable tree"** ✅ not "const initializable tree"

### Tree Terminology
- **"directory tree"** - Primary term for the hierarchical structure of directories and commands
- **"tree"** - Shortened form when context is clear
- **"command tree"** - Only use in example contexts (e.g., "example command tree")

### Code Identifiers in Prose
Use proper formatting when referring to code:
- **`Shell`** - Struct name (with backticks, CamelCase)
- **`CharIo`** - Trait name (with backticks, not `CharIO`)
- **`AccessLevel`** - Trait type (CamelCase) vs "access level" - concept (lowercase)
- **`no_std`** - Feature name (with backticks, even in prose, not "no-std")
- **`CommandMeta`** - Struct name (CamelCase)
- **`CommandHandlers`** - Trait name (CamelCase)

### Project Names
- **"nut-shell"** - Project/library name (kebab-case)
- **"Shell"** or **"CLI"** - Prose reference to the CLI shell (sentence case)

### Feature Names
Always lowercase, no hyphens when referring to Cargo features:
- **`authentication`** - Not "Authentication" or "auth"
- **`completion`** - Not "Completion" or "auto-completion"
- **`history`** - Not "History" or "command-history"

---

## Documentation Quick Links

- **[docs/DESIGN.md](docs/DESIGN.md)** - Design decisions, rationale, feature gating
- **[docs/INTERNALS.md](docs/INTERNALS.md)** - Runtime behavior, data flow, state machines
- **[docs/SPECIFICATION.md](docs/SPECIFICATION.md)** - Complete behavioral specification
- **[docs/TYPE_REFERENCE.md](docs/TYPE_REFERENCE.md)** - Complete type definitions, method signatures, constants
- **[docs/SECURITY.md](docs/SECURITY.md)** - Authentication, access control security
- **[docs/PHILOSOPHY.md](docs/PHILOSOPHY.md)** - Design philosophy, feature framework
- **[docs/IMPLEMENTATION.md](docs/IMPLEMENTATION.md)** - Implementation roadmap, build commands
- **[docs/IO_DESIGN.md](docs/IO_DESIGN.md)** - CharIo trait, buffering model, async patterns
