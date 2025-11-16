# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

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

### Implementing a Feature-Gated Module

For complete feature gating patterns, configuration examples, and build instructions, see ARCHITECTURE.md "Feature Gating & Optional Features" section.

**Quick example:**
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
// WRONG: Always compiles
use crate::tree::completion;

// RIGHT: Conditional
#[cfg(feature = "completion")]
use crate::tree::completion;
```
See ARCHITECTURE.md for complete feature gating patterns.

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

---

## Common Build Commands

Quick reference for frequent operations:

```bash
# Development
cargo check                              # Fast compile check
cargo test                               # Run tests
cargo clippy                             # Lint code
cargo fmt                                # Format code

# Feature testing
cargo test --all-features                # Test with all features
cargo test --no-default-features         # Test minimal configuration

# Embedded target
cargo check --target thumbv6m-none-eabi  # Verify no_std compliance
cargo size --target thumbv6m-none-eabi --release -- -A  # Measure binary size

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
