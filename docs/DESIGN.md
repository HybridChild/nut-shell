# DESIGN

This document records architectural decisions for **nut-shell**. It explains why key design choices were made and what alternatives were rejected.

**When to use this document:**
- Understanding why a design decision was made
- Learning feature gating patterns for new features
- Evaluating trade-offs between architectural alternatives

## Table of Contents

1. [Core Architecture Decisions](#core-architecture-decisions)
   - Metadata/Execution Separation Pattern
   - Unified Architecture (Authentication)
   - Completion/History: Opt-Out UX
   - Node Type System
   - CharIo Buffering Model
2. [Feature Gating Patterns](#feature-gating-patterns)
3. [Module Structure](#module-structure)

---

## Core Architecture Decisions

### 1. Metadata/Execution Separation Pattern

**Decision**: Separate command metadata (const in ROM) from execution logic (generic trait)

**Why**: Solves the async command type system problem while maintaining const-initialization:
- Command metadata (`CommandMeta`) is const-initializable and stored in ROM
- Execution logic via `CommandHandler` trait can be async without heap allocation
- Single codebase supports both sync and async commands
- Zero-cost for sync-only builds via monomorphization

**Architecture:**
```rust
// Metadata (const-initializable, in ROM)
pub struct CommandMeta<L: AccessLevel> {
    pub id: &'static str,          // Unique identifier for handler dispatch
    pub name: &'static str,        // Display name (can duplicate)
    pub description: &'static str,
    pub access_level: L,
    pub kind: CommandKind,          // Sync or Async marker
    pub min_args: usize,
    pub max_args: usize,
}

// Execution logic (generic trait)
pub trait CommandHandler<C: ShellConfig> {
    fn execute_sync(&self, id: &str, args: &[&str]) -> Result<Response<C>, CliError>;

    #[cfg(feature = "async")]
    async fn execute_async(&self, id: &str, args: &[&str]) -> Result<Response<C>, CliError>;
}

// Shell generic over handlers and config
pub struct Shell<'tree, L, IO, H, C>
where
    H: CommandHandler<C>,
    C: ShellConfig,
{ ... }
```

**Alternatives Rejected:**
- Function pointers only → can't store async functions (each has unique `impl Future` type)
- Enum with Async variant → can't const-initialize `impl Future` types
- Async trait with Pin<Box> → requires heap allocation (unavailable in no_std)
- Two separate libraries → 90%+ code duplication, maintenance burden

**Trade-offs:**
- ✅ Command name duplication (tree metadata + handler match) → explicit, debuggable
- ✅ Additional generics `H: CommandHandler<C>` → zero runtime cost via monomorphization
- ✅ Manual match statements in handlers → future macro can reduce boilerplate

**Code Size Impact (RP2040):**
- Sync only: +200-300 bytes (dispatch logic)
- Async enabled: +1.2-1.8KB (async machinery)

### 2. Authentication: Opt-In Security

**Decision**: Optional authentication via unified architecture pattern

**Why**:
- Development/lab environments don't need authentication overhead
- Production systems require explicit security choices (not hidden defaults)
- Multiple credential storage backends needed (build-time, flash, external)

**Implementation**: Uses **Pattern 1: Unified Architecture** (see below) - single code path for both modes. State and field values determine behavior, not `#[cfg]` branching. Core fields (`current_user`, `state`) always present; only credential provider is feature-gated.

**Alternative Rejected**: Separate implementations for auth-enabled/disabled → code duplication, maintenance burden

**See also**: [SECURITY.md](SECURITY.md) for security architecture details

### 3. Completion/History: Opt-Out UX

**Decision**: Tab completion and command history enabled by default

**Why**:
- Better default user experience for interactive use
- Small cost (~2.5-3KB flash, ~1.3KB RAM for both)
- Can be disabled individually for constrained environments

**Implementation**: Uses **Pattern 2: Stub Function Pattern** (see below) - identical signatures, empty results when disabled

### 4. Node Type System

**Decision**: Enum with `CommandMeta`/`Directory` variants

**Why**:
- Zero-cost dispatch via pattern matching (vs vtable overhead)
- Enables const initialization (required for ROM placement)
- Metadata-only commands (execution via separate `CommandHandler` trait)

**Alternative Rejected**: Trait objects → runtime overhead, no const init

### 5. CharIo Buffering Model

**Decision**: Non-async trait with explicit buffering contract

**Why**:
- Works in both bare-metal and async runtimes without trait complexity
- Bare-metal can flush immediately (blocking acceptable)
- Async implementations buffer and flush externally
- No unstable features or async_trait dependencies required

**Architecture**: See [CHAR_IO.md](CHAR_IO.md) for complete buffering model details.

**Alternatives Rejected:**

1. **Async CharIo trait** - Requires `async_trait` or unstable features, makes `process_char()` async, no benefit for bare-metal
2. **Callback-based** - Can't propagate errors, lifetime issues in `no_std`, awkward API

---

## Feature Gating Patterns

Two patterns used throughout the codebase for optional features. Follow these when adding new features.

### Pattern 1: Unified Architecture (Authentication)

**Use when**: Feature affects core state machine behavior

**Principle**: Single code path for both modes. State values determine behavior, not `#[cfg]` branching.

**Key characteristics:**
- Core fields (e.g., `current_user`, `state`) always present (not feature-gated)
- Only feature-specific dependencies conditionally compiled
- State variant determines behavior (e.g., `LoggedOut` vs `LoggedIn`)
- Constructor and initial state differ between modes

**Example:**
```rust
pub struct Shell<'tree, L, IO> {
    current_user: Option<User<L>>,  // Always present
    state: CliState,                // Always present

    #[cfg(feature = "authentication")]
    credential_provider: &'tree dyn CredentialProvider<L>,  // Conditional
}

pub enum CliState {
    #[cfg(feature = "authentication")]
    LoggedOut,  // Only when auth enabled
    LoggedIn,   // Always available
}

// Constructor differs by feature
#[cfg(feature = "authentication")]
impl Shell {
    pub fn new(tree, provider, io) -> Self {
        Self { state: CliState::LoggedOut, current_user: None, ... }
    }
}

#[cfg(not(feature = "authentication"))]
impl Shell {
    pub fn new(tree, io) -> Self {
        Self { state: CliState::LoggedIn, current_user: None, ... }
    }
}
```

**Benefits**: Single state machine, minimal branching, behavior determined by state

### Pattern 2: Stub Function Pattern (Completion, History)

**Use when**: Feature is self-contained functionality

**Principle**: Identical function signatures for both modes. Feature-disabled version returns empty/no-op results.

**Key characteristics:**
- Module always exists, contents conditionally compiled
- Same function signature in both modes
- Feature-disabled version returns empty (e.g., `Vec::new()`)
- No feature-specific fields in main Shell
- Zero `#[cfg]` in calling code

**Example:**
```rust
// src/tree/completion.rs
#![cfg_attr(not(feature = "completion"), allow(unused_variables))]

// Feature-enabled: Full implementation
#[cfg(feature = "completion")]
pub fn suggest_completions<'a, L>(node: &'a Node<L>, input: &str)
    -> Result<Vec<&'a str, 32>, CliError>
{
    // Real implementation
}

// Feature-disabled: Stub with identical signature
#[cfg(not(feature = "completion"))]
pub fn suggest_completions<'a, L>(node: &'a Node<L>, input: &str)
    -> Result<Vec<&'a str, 32>, CliError>
{
    Ok(Vec::new())  // Empty result
}

// src/shell/mod.rs - NO feature gates needed!
impl Shell {
    fn handle_tab(&mut self) -> Result<(), CliError> {
        let suggestions = completion::suggest_completions(node, input)?;

        // Behavior adapts naturally
        if suggestions.len() == 1 {
            self.complete_input(suggestions[0])?;
        }
        Ok(())
    }
}
```

**Benefits**: Zero `#[cfg]` in main code path, compiler optimizes away stub calls

### Feature Overview

| Feature | Pattern | Default | Flash | RAM | Dependencies |
|---------|---------|---------|-------|-----|--------------|
| async | Metadata/Execution | disabled | +1.2-1.8KB | 0 | none |
| authentication | Unified Architecture | disabled | +~2KB | 0 | sha2, subtle |
| completion | Stub Function | enabled | +~2KB | 0 | none |
| history | Stub Function | enabled | +~0.5-0.8KB | ~1.3KB | none |

**Build examples:**
```bash
# Default (completion + history)
cargo build

# All features
cargo build --all-features

# Minimal (no optional features)
cargo build --no-default-features

# Specific combination
cargo build --no-default-features --features async,authentication
```

---

## Module Structure

**Organized structure (~15 modules with all features):**

```
src/
├── lib.rs              # Public API and feature gates
├── shell/
│   ├── mod.rs          # Shell + Request enum + CliState enum
│   ├── parser.rs       # InputParser (escape sequences, line editing)
│   ├── history.rs      # CommandHistory (circular buffer)
│   └── handlers.rs     # CommandHandler trait definition
├── tree/
│   ├── mod.rs          # Node enum + Directory + CommandMeta structs
│   ├── path.rs         # Path type + resolution methods
│   └── completion.rs   # Tab completion logic (optional, feature-gated)
├── auth/               # Authentication module (optional, feature-gated)
│   ├── mod.rs          # User + AccessLevel trait + CredentialProvider trait
│   ├── password.rs     # Password hashing (SHA-256)
│   └── providers/      # Credential storage backends
│       ├── buildtime.rs    # Build-time environment variables
│       ├── flash.rs        # Flash storage (RP2040)
│       └── const_provider.rs  # Hardcoded (examples/testing only)
├── response.rs         # Response type + formatting
└── io.rs               # CharIo trait (see CHAR_IO.md for details)
```

**Design rationale:**
- **Request types**: Single enum provides type-safe dispatch via pattern matching
- **State management**: Inline in shell/mod.rs (small, tightly coupled with Shell)
- **Command metadata**: `CommandMeta` in tree/mod.rs (metadata-only, const-init)
- **Command execution**: `CommandHandler` trait in shell/handlers.rs (user-implemented)
- **Authentication**: Trait-based system in auth/ module (optional, pluggable backends)
- **Completion**: Free functions in tree/completion.rs (optional, stateless logic)

---

## See Also

- **[EXAMPLES.md](EXAMPLES.md)**: Usage examples and configuration patterns
- **[SECURITY.md](SECURITY.md)**: Authentication, access control, and security design
- **[PHILOSOPHY.md](PHILOSOPHY.md)**: Design philosophy and feature decision framework
- **[CHAR_IO.md](CHAR_IO.md)**: CharIo trait and platform adapter guide
- **[DEVELOPMENT.md](DEVELOPMENT.md)**: Build commands and development workflows
- **[../CLAUDE.md](../CLAUDE.md)**: AI-assisted development guidance
