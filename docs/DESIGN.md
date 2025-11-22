# nut-shell - Architecture

This document records the architectural decisions for **nut-shell**. It explains the rationale behind structural choices and documents alternatives considered.

**When to use this document:**
- Understanding why a design decision was made
- Evaluating trade-offs between architectural alternatives
- Learning about feature gating patterns
- Understanding the unified architecture approach for optional features

## Table of Contents

1. [Command Syntax](#command-syntax)
2. [Key Design Decisions](#key-design-decisions)
3. [Feature Gating & Optional Features](#feature-gating--optional-features)
   - [Authentication Feature](#authentication-feature)
   - [Auto-Completion Feature](#auto-completion-feature)
   - [Command History Feature](#command-history-feature)
   - [Combined Feature Configuration](#combined-feature-configuration)
4. [Implementation Benefits](#implementation-benefits)
5. [Module Structure](#module-structure)
6. [References](#references)

---

## Command Syntax

The CLI uses a path-based syntax that mirrors filesystem navigation, optimized for embedded systems with minimal parsing overhead.

### Core Syntax Rules

**Note:** Examples show prompts with authentication enabled (`user@path>`). Without authentication, the username prefix may be omitted or use a default value (implementation-defined).

**Navigation** (both absolute and relative):
```
user@/> system              # Navigate to directory (relative)
user@/system> network       # Navigate to subdirectory (relative)
user@/system/network> ..    # Navigate to parent directory
user@/system> /hw/led       # Navigate using absolute path
user@/system> /             # Navigate to root
```

**Command Execution** (both absolute and relative):
```
user@/system> reboot           # Execute command in current directory (relative)
user@/> /system/reboot         # Execute command using absolute path
user@/hw/led> set 255 0 0      # Execute with positional arguments (relative)
user@/> /hw/led/set 255 0 0    # Execute with args using absolute path
```

**Global Commands** (reserved keywords):
```
ls        # List current directory contents with descriptions
?         # Show available global commands (help)
logout    # End session (only when authentication feature enabled)
clear     # Clear screen (optional, platform-dependent)
```

### Disambiguation Rules

1. **Reserved keyword check**: Check if input matches reserved keywords (`ls`, `?`, `logout`, `clear`)
2. **Path resolution**: Parse input as path + optional arguments
3. **Tree lookup**: Walk tree structure to resolve path
4. **Node type determines behavior**:
   - If path resolves to `Node::Directory` → navigate to that directory
   - If path resolves to `Node::Command` → execute that command
5. **Validation**: No command or directory may use reserved keyword names (enforced at tree construction)

### Design Rationale

**Why path-based navigation?**
- Natural for hierarchical structures
- Enables both quick navigation (`system`) and direct access (`system/network/status`)
- Scriptable over serial connection
- Minimal parser complexity (~50 lines)

**Why positional arguments only?**
- No `--flags` or `-options` reduces parser complexity
- Embedded systems typically have simple command signatures
- Fixed argument counts validated per command
- Familiar to engineers (like embedded command protocols)

### Parsing Implementation

```rust
// Pseudocode for input processing
fn parse_input(input: &str) -> Result<Request, ParseError> {
    let (path_str, args) = split_on_whitespace(input);

    // 1. Check reserved keywords first
    match path_str {
        "ls" | "?" | "logout" | "clear" => return global_command(path_str),
        _ => {}
    }

    // 2. Parse as path
    let path = Path::parse(path_str)?;

    // 3. Resolve against tree
    // Note: resolve_path() performs access control checks during traversal
    // Returns "Invalid path" error for both non-existent and inaccessible nodes
    match current_dir.resolve_path(&path)? {
        Node::Directory(_) => Request::Navigate(path),
        Node::Command(_) => Request::Execute(path, args),
    }
}
```

**Zero allocation**: All parsing uses fixed-size `heapless::Vec` buffers, no heap required.

## Key Design Decisions

### 1. Command Architecture: Metadata/Execution Separation Pattern

**Decision**: Separate command metadata (const in ROM) from execution logic (generic trait)

**Rationale**: Solves the async command type system problem while maintaining const-initialization:
- Command metadata (`CommandMeta`) is const-initializable and stored in ROM
- Execution logic provided via `CommandHandlers` trait (user-implemented)
- Trait methods can be async without heap allocation
- Zero-cost for sync-only builds via monomorphization
- Single codebase supports both sync and async commands

**Architecture:**
```rust
// Metadata (const-initializable, in ROM)
pub struct CommandMeta<L: AccessLevel> {
    pub name: &'static str,
    pub description: &'static str,
    pub access_level: L,
    pub kind: CommandKind,  // Sync or Async marker
    pub min_args: usize,
    pub max_args: usize,
}

// Execution logic (generic trait)
pub trait CommandHandlers<C: ShellConfig> {
    fn execute_sync(&self, name: &str, args: &[&str]) -> Result<Response<C>, CliError>;

    #[cfg(feature = "async")]
    async fn execute_async(&self, name: &str, args: &[&str]) -> Result<Response<C>, CliError>;
}

// Shell generic over handlers and config
pub struct Shell<'tree, L, IO, H, C>
where
    H: CommandHandlers<C>,
    C: ShellConfig,
{ ... }
```

**Alternatives Considered:**
- Function pointers only (rejected: can't store async functions - each has unique `impl Future` type)
- Enum with Async variant (rejected: can't const-initialize `impl Future` types)
- Async trait with Pin<Box> (rejected: requires heap allocation, not available in no_std)
- Two separate libraries (rejected: 90%+ code duplication, maintenance burden)

**Usage Patterns:**

*Bare-Metal (Sync Only):*
```rust
struct BareMetalHandlers;

impl<C: ShellConfig> CommandHandlers<C> for BareMetalHandlers {
    fn execute_sync(&self, name: &str, args: &[&str]) -> Result<Response<C>, CliError> {
        match name {
            "reboot" => reboot_fn::<C>(args),
            "status" => status_fn::<C>(args),
            _ => Err(CliError::CommandNotFound),
        }
    }
}

// Main loop
loop {
    if let Ok(Some(c)) = io.get_char() {
        shell.process_char(c).ok();  // Sync processing
    }
}
```

*Embassy/RTIC (Async Commands):*
```rust
struct EmbassyHandlers;

impl<C: ShellConfig> CommandHandlers<C> for EmbassyHandlers {
    fn execute_sync(&self, name: &str, args: &[&str]) -> Result<Response<C>, CliError> {
        match name {
            "reboot" => reboot_fn::<C>(args),
            _ => Err(CliError::CommandNotFound),
        }
    }

    #[cfg(feature = "async")]
    async fn execute_async(&self, name: &str, args: &[&str]) -> Result<Response<C>, CliError> {
        match name {
            "http-get" => http_get_async::<C>(args).await,    // Natural async!
            "wifi-connect" => wifi_connect_async::<C>(args).await,
            _ => Err(CliError::CommandNotFound),
        }
    }
}

// Embassy task
#[embassy_executor::task]
async fn shell_task(usb: UsbDevice) {
    let mut shell = Shell::new(&ROOT, EmbassyHandlers, usb_io);

    loop {
        let c = usb_io.read_char().await;
        shell.process_char_async(c).await.ok();  // Can await async commands
    }
}
```

**Benefits:**

*For Sync Users (Bare-Metal):*
- Zero async machinery compiled in (feature-gated)
- Minimal code size increase (~200-300 bytes for dispatch logic)
- Same const-initialized trees
- No behavioral changes

*For Async Users (Embassy, RTIC):*
- Natural async/await in commands (no manual spawning/tracking)
- No global state requirements
- Direct error propagation via `?`
- Clean, ergonomic code

*For Library Maintainers:*
- Single codebase (~95% shared code)
- Unified architecture (same parsing, tree, auth, etc.)
- Feature-gated async support
- Manageable complexity increase

**Trade-offs Accepted:**

✅ **Command name duplication** - Name appears in both tree metadata and handler match
  → Can be mitigated with future macro validation
  → Explicit dispatch is debuggable and type-safe

✅ **Additional generic parameters** - `Shell<'tree, L, IO, H, C>`
  → H: CommandHandlers<C> for command execution
  → C: ShellConfig for buffer sizes
  → Monomorphization means zero runtime cost
  → Cleaner than alternatives (async traits, heap allocation)

✅ **Manual match statements** - Handler implementations use match expressions
  → Future macro can reduce boilerplate
  → Explicit dispatch aids debugging

**Code Size Impact (RP2040):**

| Build Configuration | Flash Usage Delta |
|---------------------|-------------------|
| Sync only (no async feature) | +200-300 bytes (dispatch logic) |
| With async feature enabled | +1.2-1.8KB (async machinery) |

The ~200-300 byte increase for sync-only builds is acceptable given the improved architecture and future async support capability.

### 2. Path Resolution Location
**Decision**: Methods on `Directory` (`resolve_path`) and `Path` (parsing)

**Rationale**: Emphasizes tree navigation, keeps related functionality together

**Alternative Considered**: Separate PathResolver class

### 3. Request Type Structure
**Decision**: Single enum in `shell/mod.rs`

**Rationale**: Pattern matching provides type-safe dispatch, reduces file count

**Alternative Considered**: Separate types with trait-based dispatch

### 4. Node Polymorphism
**Decision**: Enum with CommandMeta/Directory variants

**Rationale**: Zero-cost dispatch, enables const initialization, metadata-only commands

**Alternative Considered**: Trait objects (runtime overhead, no const init)

### 5. Authentication System

**Decision**: Optional authentication with trait-based credential providers, using a unified architecture that minimizes code branching

**Rationale**: Different deployments have different security requirements:
- Development/debugging environments may not need authentication
- Production embedded systems require secure access control
- Flexibility needed for various credential storage backends (build-time, flash, external)

**Implementation Approach**: Uses **Pattern 1: Unified Architecture** (see [Feature Gating Patterns](#pattern-1-unified-architecture-authentication)) - single code path for both auth-enabled and auth-disabled modes. State and field values determine behavior rather than `#[cfg]` branching. Core fields (`current_user`, `state`) always present; only credential provider is feature-gated.

**Alternative Considered**: Separate implementations for auth-enabled/disabled (rejected due to code duplication and maintenance burden)

**See Also**:
- Implementation pattern: [Pattern 1: Unified Architecture](#pattern-1-unified-architecture-authentication)
- Feature details & usage guidance: [Authentication Feature](#authentication-feature)
- Security architecture: [SECURITY.md](SECURITY.md)

### 6. Completion Implementation

**Decision**: Free functions or trait methods in `completion` module

**Rationale**: No state needed, lightweight module organization

**Implementation Approach**: Uses **Pattern 2: Stub Function Pattern** (see [Feature Gating Patterns](#pattern-2-stub-function-pattern-completion-history)) - identical function signatures in both modes, feature-disabled version returns empty results

**Alternative Considered**: Separate Completer type with stateful instance

**See Also**: [Auto-Completion Feature](#auto-completion-feature) for feature details and usage guidance

### 7. State Management
**Decision**: Inline `CliState` enum in `shell/mod.rs`

**Rationale**: Only 3 variants, too small for separate file

**Alternative Considered**: Separate state.rs file

### 8. Double-ESC Clear Behavior
**Decision**: ESC ESC clears input buffer and exits history navigation (not feature-gated)

**Rationale**:
- Significantly improves UX for interactive users (quick cancel/clear without repeated backspace)
- Minimal code overhead (~50-100 bytes flash, 0 bytes RAM)
- Works naturally in no_std (pure state machine, no timers needed)
- Avoids ambiguity with escape sequences (ESC [ for arrows still works)
- Too small to justify feature gating

**Implementation Pattern**: Double-ESC required to distinguish from escape sequence start
- ESC ESC → clear buffer
- ESC [ → begin escape sequence (arrow keys)
- ESC + other → clear buffer, then process character

**Alternative Considered**: Single ESC with timeout (rejected - requires timer, adds complexity, not suitable for no_std)

---

## Feature Gating & Optional Features

### Overview

The Rust implementation provides optional features that can be enabled or disabled at compile time to accommodate different deployment scenarios and resource constraints. This allows fine-grained control over code size, dependencies, and functionality.

**Available Optional Features:**
- **async**: Enable async command execution with `process_char_async()` and `CommandHandlers::execute_async()` (default: disabled, see [Section 1](#1-command-architecture-metadataexecution-separation-pattern) for complete architecture)
- **authentication**: User login and access control system (default: enabled)
- **completion**: Tab completion for commands and paths (default: enabled)
- **history**: Command history navigation with arrow keys (default: enabled)

**Philosophy:**
- Features are enabled by default for best user experience
- Can be disabled individually or in combination for constrained environments
- No runtime overhead when disabled (eliminated at compile time)
- Graceful degradation when features are unavailable

---

### Feature Gating Patterns

This section explains the two main patterns used for feature gating. Individual features apply one of these patterns.

**Note:** The **async** feature is documented separately in Section 1 [Command Architecture: Metadata/Execution Separation Pattern](#1-command-architecture-metadataexecution-separation-pattern) as it extends the core execution model rather than adding optional functionality.

#### Pattern 1: Unified Architecture (Authentication)

**Principle:** Single code path for both feature-enabled and feature-disabled modes. State and field values determine behavior, not `#[cfg]` branching.

**Key characteristics:**
- Core fields (e.g., `current_user`, `state`) always present, not feature-gated
- Only feature-specific dependencies conditionally compiled
- State variant determines behavior (e.g., `LoggedOut` vs `LoggedIn`)
- Constructor and initial state differ between modes
- Single implementation for core methods (e.g., `activate()`, `generate_prompt()`)

**Example (simplified - omits H and C generics for clarity):**
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

// Constructor differs
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

**Benefits:** Single state machine, minimal branching, behavior determined by state rather than scattered `#[cfg]` blocks.

#### Pattern 2: Stub Function Pattern (Completion, History)

**Principle:** Identical function signatures for both modes. Feature-disabled version returns empty/no-op results.

**Key characteristics:**
- Module always exists, contents conditionally compiled
- Same function signature in both modes
- Feature-disabled version returns empty (e.g., `Vec::new()`)
- No feature-specific fields in main Shell
- Behavior adapts naturally to empty results

**Example (simplified):**
```rust
// src/tree/completion.rs
#![cfg_attr(not(feature = "completion"), allow(unused_variables))]

// Feature-enabled: Full implementation
#[cfg(feature = "completion")]
pub fn suggest_completions<'a, L>(node: &'a Node<L>, input: &str)
    -> Result<Vec<&'a str, 32>, CliError>
{
    // Real implementation: tree traversal, prefix matching, etc.
}

// Feature-disabled: Stub with identical signature
#[cfg(not(feature = "completion"))]
pub fn suggest_completions<'a, L>(node: &'a Node<L>, input: &str)
    -> Result<Vec<&'a str, 32>, CliError>
{
    Ok(Vec::new())  // Empty result
}

// src/shell/mod.rs - NO feature gates!
impl Shell {
    fn handle_tab(&mut self) -> Result<(), CliError> {
        let suggestions = completion::suggest_completions(node, input)?;

        // Behavior adapts naturally to empty vs. populated results
        if suggestions.len() == 1 {
            self.complete_input(suggestions[0])?;
        } else if !suggestions.is_empty() {
            self.display_suggestions(&suggestions)?;
        }
        // Empty = feature disabled, naturally no-op
        Ok(())
    }
}
```

**Benefits:** Zero `#[cfg]` in main code path, compiler optimizes away stub calls, single implementation.

#### Build Configuration Template

All features follow this Cargo.toml pattern:

```toml
[features]
default = ["authentication", "completion", "history"]

authentication = ["dep:sha2", "dep:subtle"]  # Adds dependencies
completion = []                              # No dependencies
history = []                                 # No dependencies

[dependencies]
heapless = "0.8"

sha2 = { version = "0.10", default-features = false, optional = true }
subtle = { version = "2.5", default-features = false, optional = true }
```

**Build commands:**
```bash
# All features (default)
cargo build

# No optional features
cargo build --no-default-features

# Specific features
cargo build --no-default-features --features authentication,completion

# Embedded target
cargo build --target thumbv6m-none-eabi --release
```

---

### Authentication Feature

**Purpose:** User login system with password hashing and access control.

**Pattern:** Unified Architecture (Pattern 1) - state-based behavior

**Dependencies:** `sha2` (password hashing), `subtle` (constant-time comparison)

**Resource Impact:**

| Configuration | Flash | RAM | Notes |
|--------------|-------|-----|-------|
| Enabled | +~2 KB | 0 bytes | SHA-256 hashing code |
| Disabled | Baseline | 0 bytes | All auth code eliminated |

**When to use:**
- ✅ Production systems requiring access control
- ✅ Multi-user environments
- ✅ Security-sensitive applications
- ❌ Development/lab equipment (disable for convenience)
- ❌ Single-user trusted environments

**Security notes:**
- SHA-256 password hashing with salts
- Constant-time password comparison (timing-attack resistant)
- Invalid credentials return generic error (no user enumeration)
- See [SECURITY.md](SECURITY.md) for complete security architecture

---

### Auto-Completion Feature

**Purpose:** Tab completion for commands and paths in interactive CLI sessions.

**Pattern:** Stub Function Pattern (Pattern 2) - returns empty when disabled

**Dependencies:** None (uses `heapless` only)

**Resource Impact:**

| Configuration | Flash | RAM | Notes |
|--------------|-------|-----|-------|
| Enabled | +~2 KB | 0 bytes (stack only) | Stateless algorithm |
| Disabled | ~0 bytes | 0 bytes | Stub optimized away |

**When to use:**
- ✅ Interactive CLI for human operators
- ✅ Development and debugging workflows
- ✅ User training environments
- ❌ Scripted/programmatic access only
- ❌ Severely flash-constrained systems (>95% capacity)
- ❌ Headless operation with no terminal

**Security notes:**
- Respects `AccessLevel` (only shows accessible commands)
- Does not bypass access control
- Minimal attack surface (stateless algorithm)

---

### Command History Feature

**Purpose:** Arrow key navigation to recall and re-execute previously entered commands.

**Pattern:** Stub Type Pattern (Pattern 2) - stateful type with no-op stub

**Dependencies:** None (uses `heapless` only)

**Resource Impact:**

| Configuration | Flash | RAM | Notes |
|--------------|-------|-----|-------|
| Enabled (N=10) | +~500-800 bytes | ~1.3 KB | 10-entry circular buffer |
| Enabled (N=4) | +~500-800 bytes | ~0.5 KB | RAM-constrained config |
| Disabled | ~0 bytes | 0 bytes | Entire structure eliminated |

**When to use:**
- ✅ Interactive CLI for human operators
- ✅ Development and debugging workflows
- ✅ RAM available (>512 bytes for N=4, >1.3KB for N=10)
- ✅ Repeated command execution expected
- ❌ Flash/RAM critically constrained (bootloaders, recovery mode)
- ❌ Programmatic/scripted access only
- ❌ Read-only/kiosk mode (users shouldn't recall commands)
- ❌ Headless operation with no terminal

**Security notes:**
- Stores successfully executed commands only (not failed attempts)
- Login credentials never stored in history
- History cleared on logout (when authentication enabled)
- Minimal attack surface (circular buffer)

---

### Combined Feature Configuration

Multiple features can be enabled or disabled in combination to suit different deployment scenarios.

#### Common Configuration Patterns

```toml
# Full-featured build (default - no async)
[features]
default = ["authentication", "completion", "history"]

# Full-featured with async (Embassy/RTIC environments)
[features]
default = ["async", "authentication", "completion", "history"]

# Minimal embedded (size-optimized)
[features]
default = []

# Async-only (bare-metal with async executor, no UX features)
[features]
default = ["async"]

# Interactive but unsecured (development only)
[features]
default = ["completion", "history"]

# Secured but non-interactive (scripted access)
[features]
default = ["authentication"]

# Interactive with minimal RAM (small history)
[features]
default = ["authentication", "completion", "history"]
# Note: Set HISTORY_SIZE=4 via const generic to reduce RAM from 1.3KB to 0.5KB
```

#### Build Examples by Scenario

```bash
# Development workstation (all features including async)
cargo build --all-features

# Production embedded device (default features, sync-only)
cargo build --target thumbv6m-none-eabi --release

# Production with async (Embassy/RTIC)
cargo build --target thumbv6m-none-eabi --release --features async

# Constrained device (authentication only, ~4-5KB flash + 1.3KB RAM saved)
cargo build --target thumbv6m-none-eabi --release \
  --no-default-features --features authentication

# Async with authentication (no interactive features)
cargo build --target thumbv6m-none-eabi --release \
  --no-default-features --features async,authentication

# Unsecured lab equipment (interactive features only, for ease of use)
cargo build --no-default-features --features completion,history

# Minimal bootloader/recovery (no optional features, sync-only)
cargo build --target thumbv6m-none-eabi --release --no-default-features

# CI/CD testing (test all feature combinations)
cargo test --all-features
cargo test --no-default-features
cargo test --features async
cargo test --features authentication
cargo test --features completion
cargo test --features history
cargo test --features async,authentication
cargo test --features async,completion
cargo test --features async,history
cargo test --features authentication,completion
cargo test --features authentication,history
cargo test --features completion,history
cargo test --features async,authentication,completion
cargo test --features async,authentication,history
cargo test --features async,completion,history
```

#### Feature Dependencies

```
async (independent)
  ├── No dependencies on other features
  └── Requires: Async executor (Embassy, RTIC, etc. - user-provided)
  └── See: Section 1 for complete architecture documentation

authentication (independent)
  ├── No dependencies on other features
  └── Requires: sha2, subtle (optional crates)

completion (independent)
  ├── No dependencies on other features
  └── Requires: No additional crates (uses heapless only)

history (independent)
  ├── No dependencies on other features
  └── Requires: No additional crates (uses heapless only)

Note: All features are completely independent and can be
enabled in any combination without conflicts.
```

#### Code Size Comparison

| Configuration | Estimated Flash | Estimated RAM | Use Case |
|---------------|----------------|---------------|----------|
| `--no-default-features` | Baseline | Baseline | Absolute minimum (sync-only) |
| `--features async` | +~1.2-1.8 KB | +0 bytes | Async executor, no UX features |
| `--features authentication` | +~2 KB | +0 bytes | Secured, non-interactive, sync |
| `--features completion` | +~2 KB | +0 bytes | Interactive, unsecured, stateless |
| `--features history` | +~0.5-0.8 KB | +1.3 KB (N=10) | Non-interactive with recall |
| `--features async,authentication` | +~3.2-3.8 KB | +0 bytes | Async + secured, no UX |
| `--features completion,history` | +~2.5-3 KB | +1.3 KB | Interactive, unsecured |
| `--features authentication,completion` | +~4 KB | +0 bytes | Secured, interactive, stateless |
| `--features async,authentication,completion,history` | +~5.7-6.6 KB | +1.3 KB | Full-featured with async |
| `--all-features` (default, no async) | +~4.5-5 KB | +1.3 KB | Full-featured, sync-only |

*Note: Actual sizes depend on target architecture, optimization level, and LLVM version. Use `cargo size` to measure your specific build.*

---

## Implementation Benefits

These architectural choices provide:

- **Zero-cost I/O abstraction**: Compile-time monomorphization eliminates runtime dispatch overhead
- **ROM-based trees**: Const-initialized directory structures placed in flash memory
- **O(1) history operations**: Circular buffer provides efficient history navigation
- **Zero-copy parsing**: Input parsed as string slices, no per-argument allocation
- **Lifetime safety**: Compiler prevents dangling references, no manual lifetime management
- **No runtime init**: Tree structures ready at compile time, zero initialization overhead
- **Modular architecture**: Enums and trait-based design create ~14 focused modules (8 core, 6 optional features)

## Module Structure

**Organized structure (~15 modules with all features):**

```
src/
├── lib.rs              # Public API and feature gates
├── shell/
│   ├── mod.rs          # Shell + Request enum + CliState enum
│   ├── parser.rs       # InputParser (escape sequences, line editing)
│   ├── history.rs      # CommandHistory (circular buffer)
│   └── handlers.rs     # CommandHandlers trait definition
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
└── io.rs               # CharIo trait (see IO_DESIGN.md for buffering model details)
```

**Note:** For CharIo trait design, buffering model, and sync/async patterns, see [IO_DESIGN.md](IO_DESIGN.md).

**Rationale for consolidation:**
- **Request types**: Single enum provides type-safe dispatch via pattern matching
- **State management**: Inline in shell/mod.rs (small, tightly coupled with Shell)
- **Path resolution**: Methods on existing types (tree navigation as core concern)
- **Tree types**: Combined in tree/mod.rs (related const-init concerns)
- **Command metadata**: `CommandMeta` in tree/mod.rs (metadata-only, const-init)
- **Command execution**: `CommandHandlers` trait in shell/handlers.rs (user-implemented)
- **Authentication**: Trait-based system in auth/ module (optional, pluggable backends)
- **Completion**: Free functions in tree/completion.rs (optional, stateless logic)

## See Also

- **[EXAMPLES.md](EXAMPLES.md)**: Usage examples and configuration patterns
- **[SECURITY.md](SECURITY.md)**: Authentication, access control, and security design
- **[PHILOSOPHY.md](PHILOSOPHY.md)**: Design philosophy and feature decision framework
- **[IO_DESIGN.md](IO_DESIGN.md)**: CharIo trait and platform adapter guide
- **[DEVELOPMENT.md](DEVELOPMENT.md)**: Build commands and development workflows
- **[../CLAUDE.md](../CLAUDE.md)**: AI-assisted development guidance
