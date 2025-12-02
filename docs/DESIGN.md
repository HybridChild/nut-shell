# DESIGN

This document records architectural decisions for **nut-shell**. It explains why key design choices were made and what alternatives were rejected.

**When to use this document:**
- Understanding why a design decision was made
- Learning feature gating techniques for new features
- Evaluating trade-offs between architectural alternatives

## Table of Contents

1. [Core Architecture Decisions](#core-architecture-decisions)
   - Metadata/Execution Separation Pattern
   - Authentication: Opt-In Security
   - Completion/History: Opt-Out UX
   - Node Type System
   - `CharIo` Buffering Model
2. [Feature Gating](#feature-gating)
   - Stub Functions
   - Conditional State

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
    pub kind: CommandKind,         // Sync or Async marker
    pub min_args: usize,
    pub max_args: usize,
}

// Execution logic (generic trait)
pub trait CommandHandler<C: ShellConfig> {
    fn execute_sync(&self, id: &str, args: &[&str]) -> Result<Response<C>, CliError>;

    #[cfg(feature = "async")]
    async fn execute_async(&self, id: &str, args: &[&str]) -> Result<Response<C>, CliError>;
}

// Shell generic over handler and config
pub struct Shell<'tree, L, IO, H, C>
where
    H: CommandHandler<C>,
    C: ShellConfig,
{ ... }
```

**Alternatives Rejected:**
- Function pointers only → can't store async functions (each has unique `impl Future` type)
- Enum with Async variant → can't const-initialize `impl Future` types
- Async trait with Pin<Box> → requires heap allocation (unavailable in `no_std`)
- Two separate libraries → 90%+ code duplication, maintenance burden

**Trade-offs:**
- ✅ Command name duplication (tree metadata + handler match) → explicit, debuggable
- ✅ Additional generics `H: CommandHandler<C>` → zero runtime cost via monomorphization
- ✅ Manual match statements in handlers → future macro can reduce boilerplate

### 2. Authentication: Opt-In Security

**Decision**: Optional authentication via unified architecture pattern

**Why**:
- Development/lab environments don't need authentication overhead
- Production systems require explicit security choices (not hidden defaults)
- Multiple credential storage backends needed (build-time, flash, external)

**Implementation**: Uses **conditional fields** technique (see Feature Gating below) - core state fields (`current_user`, `state`) always present; only credential provider is feature-gated.

**Alternative Rejected**: Separate implementations for auth-enabled/disabled → code duplication, maintenance burden

**See also**: [SECURITY.md](SECURITY.md) for security architecture details

### 3. Completion/History: Opt-Out UX

**Decision**: Tab completion and command history enabled by default

**Why**:
- Better default user experience for interactive use
- Minimal overhead (see `size-analysis/` for measurements)
- Can be disabled individually for constrained environments

**Implementation**: Uses **stub functions** technique (see Feature Gating below) - identical signatures, empty results when disabled

### 4. Node Type System

**Decision**: Enum with `CommandMeta`/`Directory` variants

**Why**:
- Zero-cost dispatch via pattern matching (vs vtable overhead)
- Enables const initialization (required for ROM placement)
- Metadata-only commands (execution via separate `CommandHandler` trait)

**Alternative Rejected**: Trait objects → runtime overhead, no const init

### 5. `CharIo` Buffering Model

**Decision**: Non-async trait with explicit buffering contract

**Why**:
- Works in both bare-metal and async runtimes without trait complexity
- Bare-metal can flush immediately (blocking acceptable)
- Async implementations buffer and flush externally
- No unstable features or async_trait dependencies required

**Architecture**: See [CHAR_IO.md](CHAR_IO.md) for complete buffering model details.

**Alternatives Rejected:**

1. **Async `CharIo` trait** - Requires `async_trait` or unstable features, makes `process_char()` async, no benefit for bare-metal
2. **Callback-based** - Can't propagate errors, lifetime issues in `no_std`, awkward API

---

## Feature Gating

Optional features minimize overhead when disabled through conditional compilation. Two techniques used:

### Stub Functions

**When to use**: Feature adds functionality without affecting core state machine.

**Approach**: Provide identical function signatures for both enabled/disabled. Disabled version returns empty/no-op.

**Example** (completion):
```rust
// src/tree/completion.rs

#[cfg(feature = "completion")]
pub fn suggest_completions<'a, L>(node: &'a Node<L>, input: &str)
    -> Result<Vec<&'a str, 32>, CliError>
{
    // Search tree, match prefixes, return suggestions
}

#[cfg(not(feature = "completion"))]
pub fn suggest_completions<'a, L>(_node: &'a Node<L>, _input: &str)
    -> Result<Vec<&'a str, 32>, CliError>
{
    Ok(Vec::new())  // Empty - no completions available
}

// Caller in shell/mod.rs needs no #[cfg]:
let suggestions = completion::suggest_completions(node, input)?;
if suggestions.is_empty() { /* naturally handles disabled case */ }
```

**Why this works**: Caller adapts naturally to empty results. Compiler eliminates stub bodies entirely.

### Conditional State

**When to use**: Feature fundamentally changes control flow or requires external dependencies.

**Approach**: Keep state-tracking fields unconditional (e.g., `current_user`, `state`). Gate only feature-specific dependencies (e.g., `credential_provider`) and state variants.

**Example** (authentication):
```rust
pub struct Shell<'tree, L, IO> {
    current_user: Option<User<L>>,  // Always present (None when disabled)
    state: CliState,                // Always present

    #[cfg(feature = "authentication")]
    credential_provider: &'tree dyn CredentialProvider<L>,  // Only dependency gated
}

pub enum CliState {
    Inactive,
    #[cfg(feature = "authentication")]
    LoggedOut,  // State variant only exists when needed
    LoggedIn,
}
```

**Why this works**: Core state machine remains intact. When auth disabled, `Shell::new()` has different signature (no provider) and `activate()` transitions directly to `LoggedIn`. Compiler eliminates `LoggedOut` branches entirely.

**Trade-off**: Requires `#[cfg]` blocks in constructors, activation, and state matching. Accepted because alternative (duplicate state machine implementations) creates maintenance burden.

---

## See Also

- **[EXAMPLES.md](EXAMPLES.md)**: Usage examples and configuration patterns
- **[SECURITY.md](SECURITY.md)**: Authentication, access control, and security design
- **[PHILOSOPHY.md](PHILOSOPHY.md)**: Design philosophy and feature decision framework
- **[CHAR_IO.md](CHAR_IO.md)**: `CharIo` trait and platform adapter guide
- **[DEVELOPMENT.md](DEVELOPMENT.md)**: Build commands and development workflows
- **[../CLAUDE.md](../CLAUDE.md)**: AI-assisted development guidance
