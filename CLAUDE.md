# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

**Important**: Before adding to or modifying the information in this file, always consider the following:
- This file should only contain information that is useful for Claude Code, not for human developers.
- The information in this file should be formatted and presented in the way that is optimal for Claude Code, not for human developers.

## Project Overview

**nut-shell** is a lightweight library for adding a flexible command-line interface to embedded systems. The implementation targets **no_std** environments with static allocation, specifically designed for platforms like the Raspberry Pi Pico (RP2040).

**Important Note on Contributing:**
The library is now complete and production-ready. When contributing:
- **Follow established patterns** - See DESIGN.md for architectural patterns
- **Discuss significant changes** - Open an issue for architectural modifications
- **Test thoroughly** - Run all feature combinations (see docs/DEVELOPMENT.md)
- **Update documentation** - Keep docs synchronized with code changes

---

## Documentation Navigation

**When to consult each document:**

| Need | Document | What You'll Find |
|------|----------|------------------|
| Usage examples and configuration | **[docs/EXAMPLES.md](docs/EXAMPLES.md)** | Quick start, platform examples, common patterns, troubleshooting |
| Why it's designed this way | **[docs/DESIGN.md](docs/DESIGN.md)** | Design rationale, unified architecture pattern, feature gating |
| Security patterns and credential storage | **[docs/SECURITY.md](docs/SECURITY.md)** | Password hashing, access control, authentication flow |
| Design philosophy and feature criteria | **[docs/PHILOSOPHY.md](docs/PHILOSOPHY.md)** | What we include/exclude, decision framework |
| CharIo implementation and buffering | **[docs/IO_DESIGN.md](docs/IO_DESIGN.md)** | Sync/async I/O patterns, buffering model, platform adapters |
| Build commands and testing | **[docs/DEVELOPMENT.md](docs/DEVELOPMENT.md)** | Complete build workflows, CI simulation, troubleshooting |
| API reference | **Run `cargo doc --open`** | Complete API documentation generated from source code |

---

## Quick Reference - Common Tasks

### Adding a New Command

**Pattern:** Metadata/execution separation (CommandMeta + CommandHandlers trait)

**Steps:**
1. Define command function: `fn cmd<C: ShellConfig>(args: &[&str]) -> Result<Response<C>, CliError>`
2. Create const metadata: `CommandMeta { id, name, access_level, kind: Sync/Async, ... }`
3. Add to tree: `Node::Command(&CMD)` in directory's children
4. Implement handler: Map command ID to function in `CommandHandlers::execute_sync/async()`

**Key points:**
- Unique `id` field for dispatch (allows duplicate display names in different dirs)
- `CommandKind::Sync` or `Async` marker determines handler method
- Async requires `async` feature and `process_char_async()`

**See [docs/EXAMPLES.md](docs/EXAMPLES.md) for complete examples.** Full architecture in [docs/DESIGN.md](docs/DESIGN.md) section 1.

### Implementing Global Commands (ls, ?, clear, logout)

Global commands are reserved keywords handled outside the tree structure.

**Help command (?) output format:**
```rust
fn show_help(&mut self) -> Result<(), IO::Error> {
    self.io.write_str("  ?        - List global commands\r\n")?;
    self.io
        .write_str("  ls       - List directory contents\r\n")?;

    #[cfg(feature = "authentication")]
    self.io.write_str("  logout   - End session\r\n")?;

    self.io.write_str("  clear    - Clear screen\r\n")?;
    self.io.write_str("  ESC ESC  - Clear input buffer\r\n")?;

    Ok(())
}
```

**Important:**
- `ESC ESC` is not a command (it's a keyboard shortcut), but include it in `?` output for discoverability
- `logout` only shown when authentication feature enabled

### Implementing a Feature-Gated Module

**Stub Function Pattern** (recommended - minimizes `#[cfg]` branching):
```rust
// Module always exists, contents gated
#[cfg(feature = "my_feature")]
pub fn do_something() -> Result<Vec<&str, 32>> { /* real impl */ }

#[cfg(not(feature = "my_feature"))]
pub fn do_something() -> Result<Vec<&str, 32>> { Ok(Vec::new()) }  // No-op stub
```

**Benefits:** Single code path, zero `#[cfg]` in Shell code, compiler optimizes away stubs

**Stub Type Pattern** (for stateful types): Use zero-size PhantomData stub + identical API

**See [docs/DESIGN.md](docs/DESIGN.md) "Feature Gating & Optional Features" for complete patterns, variations, and examples.**

### Implementing CommandHandlers Trait

Maps command IDs to functions. Generic over `ShellConfig` for buffer size matching.

```rust
impl CommandHandlers<MyConfig> for MyHandlers {
    fn execute_sync(&self, id: &str, args: &[&str]) -> Result<Response<MyConfig>, CliError> {
        match id {
            "reboot" => reboot_fn::<MyConfig>(args),
            _ => Err(CliError::CommandNotFound),
        }
    }

    #[cfg(feature = "async")]
    async fn execute_async(&self, id: &str, args: &[&str]) -> Result<Response<MyConfig>, CliError> {
        match id {
            "http_get" => http_get_async::<MyConfig>(args).await,
            _ => Err(CliError::CommandNotFound),
        }
    }
}
```

**Stateful:** Add fields to handler struct, access in match arms. **See [docs/EXAMPLES.md](docs/EXAMPLES.md) for complete patterns.**

### Implementing AccessLevel Trait

**Use the derive macro** (always available):

```rust
use nut_shell::AccessLevel;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, AccessLevel)]
pub enum MyAccessLevel {
    Guest = 0,
    User = 1,
    Admin = 2,
}
```

The macro automatically generates `from_str()` and `as_str()` implementations using variant names as strings.

**Manual implementation** (if you prefer explicit control):

```rust
impl nut_shell::auth::AccessLevel for MyAccessLevel {
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

**Note:** The derive macro is **zero-cost** - it generates identical code and has no runtime overhead. Proc-macro dependencies are build-time only with zero impact on binary size.

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

### Customizing Messages

All user-visible messages (welcome, login prompts, error messages) are configurable via the `ShellConfig` trait. Messages are stored in ROM with zero runtime cost.

```rust
struct MyAppConfig;

impl ShellConfig for MyAppConfig {
    // Buffer sizes (required)
    const MAX_INPUT: usize = 128;
    const MAX_PATH_DEPTH: usize = 8;
    const MAX_ARGS: usize = 16;
    const MAX_PROMPT: usize = 64;
    const MAX_RESPONSE: usize = 256;
    const HISTORY_SIZE: usize = 10;

    // Customize messages for your application
    const MSG_WELCOME: &'static str = "🚀 MyDevice v1.0 Ready\r\n";
    const MSG_LOGIN_PROMPT: &'static str = "Login (user:pass): ";
    const MSG_LOGIN_SUCCESS: &'static str = "✓ Access granted\r\n";
    const MSG_LOGIN_FAILED: &'static str = "✗ Access denied\r\n";
    const MSG_LOGOUT: &'static str = "Session terminated\r\n";
    const MSG_INVALID_LOGIN_FORMAT: &'static str = "Format: username:password\r\n";
}

// Use your custom config when creating the Shell
let mut shell: Shell<_, _, _, _, MyAppConfig> = Shell::new(&TREE, handlers, io);
```

**Customization use cases:**
- Brand your CLI with custom welcome messages
- Localize messages for different languages
- Add emojis or ANSI colors for better UX
- Adjust tone (formal vs casual, technical vs user-friendly)
- Match your device's personality

**Note:** All messages stored in flash memory, no RAM overhead.

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
    pub id: &'static str,          // Unique identifier for handler dispatch
    pub name: &'static str,        // Display name (can duplicate)
    pub description: &'static str,
    pub access_level: L,
    pub kind: CommandKind,  // Sync or Async marker
    pub min_args: usize,
    pub max_args: usize,
}

// Execution logic (user-implemented trait, generic over config)
pub trait CommandHandlers<C: ShellConfig> {
    fn execute_sync(&self, id: &str, args: &[&str]) -> Result<Response<C>, CliError>;

    #[cfg(feature = "async")]
    async fn execute_async(&self, id: &str, args: &[&str]) -> Result<Response<C>, CliError>;
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
- Unique ID field allows commands with same display name in different directories
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

**Constructor behavior and lifecycle:**
- `Shell::new()` creates shell in `CliState::Inactive` state (both auth modes)
- `Shell::activate()` transitions to appropriate state and shows welcome message:
  - Auth enabled: `Inactive` → `LoggedOut` (awaiting login)
  - Auth disabled: `Inactive` → `LoggedIn` (ready for commands)
- `Shell::deactivate()` returns to `Inactive` state:
  - Clears user session, input buffer, and current path
  - Shell ignores all input until `activate()` is called again
  - Useful for clean shutdown, temporary suspension, or reset to initial state

**State and user semantics:**
- `state = Inactive, current_user = None` → Shell created but not activated, or deactivated
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

**Key patterns:**
- **Const initialization test:** Reference tree in test to verify const compilation
- **Mock I/O:** `VecDeque` for input, `Vec` for output capture
- **Integration tests:** Process characters sequentially, assert state changes
- **Feature testing:** Use `#[cfg(not(feature = "..."))]` for disabled path tests

**See test files for complete examples.**

---

## Common Build Commands

```bash
cargo check                              # Fast check
cargo test --all-features                # Test all features
cargo test --no-default-features         # Test minimal
cargo check --target thumbv6m-none-eabi  # Verify no_std
cargo fmt && cargo clippy --all-features -- -D warnings && cargo test --all-features  # Pre-commit
```

**See [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) for complete build workflows and CI configuration.**

---

## Contributing Workflow

**Library Status:** Production-ready (all implementation phases complete)

**General Approach for Contributions:**
1. **Understand existing architecture** - Review [docs/DESIGN.md](docs/DESIGN.md) for patterns
2. **Check feature criteria** - See [docs/PHILOSOPHY.md](docs/PHILOSOPHY.md) before adding features
3. **Write tests first** - Follow TDD approach
4. **Implement changes** following established patterns
5. **Test on native target** (`cargo test --all-features`)
6. **Verify embedded target** (`cargo check --target thumbv6m-none-eabi`)
7. **Test feature combinations** (see [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md))
8. **Update documentation** - Keep API docs and guides synchronized
9. **Run pre-commit checks** - Format, lint, test, embedded verification

**When working with the codebase:**
- **[docs/EXAMPLES.md](docs/EXAMPLES.md)** for "how do I use this?"
- **[docs/DESIGN.md](docs/DESIGN.md)** for "why is it designed this way?"
- **Run `cargo doc --open`** for "what's the API signature?"
- **[docs/SECURITY.md](docs/SECURITY.md)** for authentication/access control specifics
- **[docs/PHILOSOPHY.md](docs/PHILOSOPHY.md)** for "should we add this feature?"
- **[docs/IO_DESIGN.md](docs/IO_DESIGN.md)** for "how do I implement CharIo?"
- **[docs/DEVELOPMENT.md](docs/DEVELOPMENT.md)** for build commands and testing workflows
- **This file (CLAUDE.md)** for constraints and patterns

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

- **[docs/EXAMPLES.md](docs/EXAMPLES.md)** - Usage examples, configuration guide, troubleshooting
- **[docs/DESIGN.md](docs/DESIGN.md)** - Design decisions, rationale, feature gating
- **[docs/SECURITY.md](docs/SECURITY.md)** - Authentication, access control security
- **[docs/PHILOSOPHY.md](docs/PHILOSOPHY.md)** - Design philosophy, feature framework
- **[docs/IO_DESIGN.md](docs/IO_DESIGN.md)** - CharIo trait, buffering model, async patterns
- **[docs/DEVELOPMENT.md](docs/DEVELOPMENT.md)** - Build commands, testing workflows, CI
- **Run `cargo doc --open`** - Complete API reference from source code
