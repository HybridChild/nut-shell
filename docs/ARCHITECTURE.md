# cli-service - Architecture

This document records the architectural decisions for cli-service. It explains the rationale behind structural choices and documents alternatives considered.

**When to use this document:**
- Understanding why a design decision was made
- Evaluating trade-offs between architectural alternatives
- Learning about feature gating patterns and implementation
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

**Note:** Examples show prompts with authentication enabled (`user@path>`). Without authentication, the username prefix may be omitted or use a default value (implementation-defined). See SPECIFICATION.md for complete prompt format details.

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
?         # Show current directory contents with descriptions
help      # List available global commands
logout    # End session (only when authentication feature enabled)
clear     # Clear screen (optional, platform-dependent)
```

### Disambiguation Rules

1. **Reserved keyword check**: Check if input matches reserved keywords (`help`, `?`, `logout`, `clear`)
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

**Why no trailing slash convention?**
- Tree structure unambiguously determines if path is directory or command
- Less typing on slow serial connections
- Simpler mental model

**Why reserved keywords instead of prefixed globals?**
- Small reserved word list (3-4 keywords) validated at compile time
- Cleaner syntax for global commands
- Avoids conflicts with absolute path syntax

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
        "help" | "?" | "logout" | "clear" => return global_command(path_str),
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

### 1. Path Resolution Location
**Decision**: Methods on `Directory` (`resolve_path`) and `Path` (parsing)

**Rationale**: Emphasizes tree navigation, keeps related functionality together

**Alternative Considered**: Separate PathResolver class

### 2. Request Type Structure
**Decision**: Single enum in `cli/mod.rs`

**Rationale**: Pattern matching provides type-safe dispatch, reduces file count

**Alternative Considered**: Separate types with trait-based dispatch

### 3. Node Polymorphism
**Decision**: Enum with Command/Directory variants

**Rationale**: Zero-cost dispatch, enables const initialization

**Alternative Considered**: Trait objects (runtime overhead, no const init)

### 4. Authentication System
**Decision**: Optional authentication with trait-based credential providers, using a unified architecture that minimizes code branching

**Rationale**: Different deployments have different security requirements:
- Development/debugging environments may not need authentication
- Production embedded systems require secure access control
- Flexibility needed for various credential storage backends (build-time, flash, external)

**Key Design Elements**:
- Generic `AccessLevel` trait allows user-defined permission hierarchies
- `CredentialProvider` trait enables pluggable authentication backends
- Password hashing (SHA-256 with salts) instead of plaintext storage
- User credentials never hardcoded in source code
- **Unified architecture**: Single state machine and code path for both auth-enabled and auth-disabled modes

**Unified Architecture Pattern**:
The implementation uses a single code path for both authentication modes to minimize complexity:

- **Always track current user**: `current_user: Option<User<L>>` exists in both modes
  - Auth enabled, logged out: `None` (awaiting login)
  - Auth enabled, logged in: `Some(user)` (authenticated user)
  - Auth disabled: `None` (no user needed, access checks skipped)

- **Single state machine**: Same `CliState` enum, different initial state
  - Auth enabled: Starts in `LoggedOut` state, transitions to `LoggedIn` after authentication
  - Auth disabled: Starts in `LoggedIn` state immediately

- **State and user combination determines behavior**:
  - `state = LoggedOut, current_user = None` → Awaiting login (auth enabled)
  - `state = LoggedIn, current_user = Some(user)` → Authenticated (auth enabled)
  - `state = LoggedIn, current_user = None` → Auth disabled (no checks)

- **Unified prompt generation**: Single `generate_prompt()` function
  - Always uses format: `username@path> `
  - `None` or empty username → `@path> `
  - Authenticated user → `username@path> `

- **Conditional fields only when necessary**: Only `credential_provider` requires feature gating
  - State management, user tracking, and prompt logic work identically in both modes

**Alternative Considered**: Separate implementations for auth-enabled/disabled (rejected due to code duplication)

**Feature Gating**: Authentication is optional and can be disabled via Cargo features for unsecured development environments or when authentication is handled externally. When disabled, access control checks are eliminated and all commands are accessible. Estimated code savings: ~2KB for core auth logic, plus dependencies (sha2, subtle). See "Feature Gating & Optional Features" section below for detailed configuration patterns. See SECURITY.md for comprehensive security design and credential storage options.

**Security Note**: When authentication is enabled, access control failures return "Invalid path" errors (same as non-existent paths) to prevent revealing the existence of restricted commands/directories. See SPECIFICATION.md for complete error handling behavior.

### 5. Completion Implementation
**Decision**: Free functions or trait methods in `completion` module

**Rationale**: No state needed, lightweight module organization

**Alternative Considered**: Separate Completer type with stateful instance

**Feature Gating**: Tab completion is optional and can be disabled via Cargo features to reduce code size (~2KB) in constrained environments. When disabled, the entire `completion` module is eliminated at compile time with zero runtime overhead. See "Feature Gating & Optional Features" section below for detailed configuration patterns and use cases

### 6. State Management
**Decision**: Inline `CliState` enum in `cli/mod.rs`

**Rationale**: Only 3 variants, too small for separate file

**Alternative Considered**: Separate state.rs file

### 7. Double-ESC Clear Behavior
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
- **authentication**: User login and access control system (default: enabled)
- **completion**: Tab completion for commands and paths (default: enabled)
- **history**: Command history navigation with arrow keys (default: enabled)

**Philosophy:**
- Features are enabled by default for best user experience
- Can be disabled individually or in combination for constrained environments
- No runtime overhead when disabled (eliminated at compile time)
- Graceful degradation when features are unavailable

---

### Authentication Feature

#### Cargo.toml Configuration

```toml
[features]
default = ["authentication"]

# Core authentication system
authentication = ["dep:sha2", "dep:subtle"]

# Flash storage provider (requires RP2040)
flash-storage = ["authentication", "rp2040-flash"]

# Optional: Additional providers
ldap-auth = ["authentication", "ldap3"]
external-auth = ["authentication"]

[dependencies]
heapless = "0.8"

# Conditional dependencies
sha2 = { version = "0.10", default-features = false, optional = true }
subtle = { version = "2.5", default-features = false, optional = true }
rp2040-flash = { version = "0.3", optional = true }
```

#### Conditional Compilation - Unified Architecture

The implementation minimizes code branching by using a unified architecture:

```rust
// src/lib.rs
#[cfg(feature = "authentication")]
pub mod auth;

pub use auth::{User, AccessLevel};  // Always available

#[cfg(feature = "authentication")]
pub use auth::CredentialProvider;

// src/cli/mod.rs
pub struct CliService<'tree, L, IO>
where
    L: AccessLevel,
    IO: CharIo,
{
    // UNIFIED: Always track current user (None = logged out, Some = logged in)
    current_user: Option<User<L>>,

    // UNIFIED: Same state machine for both modes
    state: CliState,

    // CONDITIONAL: Only need provider when auth enabled
    #[cfg(feature = "authentication")]
    credential_provider: &'tree dyn CredentialProvider<L>,

    // ... other fields (tree, io, parser, history, etc.)
}

// State enum - same for both modes, different variants available
pub enum CliState {
    #[cfg(feature = "authentication")]
    LoggedOut,  // Only exists when auth enabled

    LoggedIn,   // Always available
    Inactive,   // Always available
}

// Constructors differ based on feature
#[cfg(feature = "authentication")]
impl<'tree, L, IO> CliService<'tree, L, IO> {
    pub fn new(
        tree: &'tree Directory<L>,
        provider: &'tree dyn CredentialProvider<L>,
        io: IO
    ) -> Self {
        Self {
            current_user: None,  // Start logged out
            state: CliState::LoggedOut,
            credential_provider: provider,
            // ... other fields
        }
    }
}

#[cfg(not(feature = "authentication"))]
impl<'tree, L, IO> CliService<'tree, L, IO> {
    pub fn new(tree: &'tree Directory<L>, io: IO) -> Self {
        Self {
            current_user: None,  // No user needed when auth disabled
            state: CliState::LoggedIn,  // Start in logged-in state
            // ... other fields
        }
    }
}

// Unified activation - state determines behavior
impl<'tree, L, IO> CliService<'tree, L, IO> {
    pub fn activate(&mut self) -> Result<(), IO::Error> {
        // Show welcome message based on state
        match self.state {
            #[cfg(feature = "authentication")]
            CliState::LoggedOut => {
                self.io.write_str("\r\nWelcome to CLI Service. Please login.\r\n\r\n")?;
            }
            CliState::LoggedIn => {
                self.io.write_str("\r\nWelcome to CLI Service. Type 'help' for help.\r\n\r\n")?;
            }
            CliState::Inactive => {}
        }

        // Always show prompt (format depends on current_user)
        self.show_prompt()?;
        Ok(())
    }

    // Unified prompt generation - simplified
    fn generate_prompt(&self) -> heapless::String<64> {
        let mut prompt = heapless::String::new();

        // Get username (empty string if no current user or system user)
        let username = self.current_user
            .as_ref()
            .map(|u| u.username.as_str())
            .unwrap_or("");

        // Always use format: username@path>
        prompt.push_str(username).ok();
        prompt.push('@').ok();
        prompt.push_str(&self.current_path()).ok();
        prompt.push_str("> ").ok();
        prompt
    }

    // Unified access control
    fn check_access(&self, node: &Node<L>) -> Result<(), CliError> {
        #[cfg(feature = "authentication")]
        {
            let user = self.current_user
                .as_ref()
                .ok_or(CliError::NotLoggedIn)?;

            if user.access_level < node.access_level() {
                return Err(CliError::InvalidPath);  // Security: hide inaccessible
            }
        }

        #[cfg(not(feature = "authentication"))]
        {
            let _ = node;  // Auth disabled, always allow
        }

        Ok(())
    }
}
```

**Benefits of Unified Approach:**
- Single state machine instead of divergent implementations
- Simplified prompt generation (always `username@path>` format, username may be empty)
- Same access control structure (just different enforcement)
- Minimal `#[cfg]` blocks scattered throughout code
- Easy to reason about behavior in both modes
- Consistent user experience (always shows welcome message and `@` separator)

#### Build Examples

```bash
# Default build (authentication enabled)
cargo build

# Disable authentication for debugging
cargo build --no-default-features

# Production build with flash storage
cargo build --release --features flash-storage

# Embedded target
cargo build --target thumbv6m-none-eabi --release --features flash-storage
```

---

### Auto-Completion Feature

Tab completion is an optional feature that provides interactive command and path completion. While it enhances user experience significantly, it can be disabled to reduce code size in severely constrained embedded environments or when only programmatic/scripted CLI access is expected.

#### Cargo.toml Configuration

```toml
[features]
default = ["authentication", "completion"]

# Core authentication system
authentication = []

# Tab completion for commands and paths
completion = []

[dependencies]
heapless = "0.8"

# No additional dependencies required for completion
# (uses only core Rust and heapless for bounded collections)
```

#### Code Size Impact

| Build Configuration | Flash Usage | RAM Impact | Use Case |
|---------------------|-------------|------------|----------|
| **With completion** | +~2KB | Temporary only (stack) | Interactive CLI usage |
| **Without completion** | Baseline | None | Scripted/programmatic access |

**Memory Characteristics:**
- Completion algorithm is stateless (no persistent RAM usage)
- Temporary allocations during tab processing only
- Uses `heapless::Vec` for bounded match results
- All completion code placed in ROM
- Estimated compiled size: 1.5-2.5KB depending on optimization level

#### Conditional Compilation - Stub Function Pattern

**IMPORTANT:** Completion uses the **stub function pattern** to minimize `#[cfg]` branching in the main code path. This aligns with the unified architecture principle.

**Pattern**: Provide identical function signatures for both feature-enabled and feature-disabled builds. The feature-disabled version returns empty/no-op results.

```rust
// src/tree/completion.rs - Module always exists, contents conditionally compiled
#![cfg_attr(not(feature = "completion"), allow(unused_variables))]

use crate::tree::{Node, Directory};
use crate::auth::AccessLevel;
use heapless::Vec;

pub const MAX_SUGGESTIONS: usize = 32;

// Feature-enabled: Full implementation
#[cfg(feature = "completion")]
pub fn suggest_completions<'a, L: AccessLevel>(
    node: &'a Node<L>,
    partial_input: &str,
    access_level: Option<L>,
) -> Result<Vec<&'a str, MAX_SUGGESTIONS>, CliError> {
    let mut suggestions = Vec::new();

    match node {
        Node::Directory(dir) => {
            for child in dir.children {
                let name = child.name();
                if name.starts_with(partial_input) {
                    if has_access(child, access_level) {
                        suggestions.push(name).map_err(|_| CliError::BufferFull)?;
                    }
                }
            }
        }
        Node::Command(_) => {
            // No children to complete
        }
    }

    Ok(suggestions)
}

// Feature-disabled: Stub returns empty results
#[cfg(not(feature = "completion"))]
pub fn suggest_completions<'a, L: AccessLevel>(
    _node: &'a Node<L>,
    _partial_input: &str,
    _access_level: Option<L>,
) -> Result<Vec<&'a str, MAX_SUGGESTIONS>, CliError> {
    Ok(Vec::new())  // No suggestions
}

// Helper function only needed when feature enabled
#[cfg(feature = "completion")]
fn has_access<L: AccessLevel>(node: &Node<L>, user_level: Option<L>) -> bool {
    // Access check logic
}
```

```rust
// src/tree/mod.rs
pub mod completion;  // Always include module (contents are feature-gated)

pub use completion::{suggest_completions, MAX_SUGGESTIONS};

// src/cli/mod.rs
pub struct CliService<'tree, L, IO>
where
    L: AccessLevel,
    IO: CharIo,
{
    // No feature-gated fields needed for completion
    // (stateless algorithm, no persistent state)

    // ... other fields
}

// Single implementation - NO feature gates needed!
impl<'tree, L, IO> CliService<'tree, L, IO> {
    fn handle_tab(&mut self) -> Result<Response, CliError> {
        let current = self.get_current_node()?;

        // Call works in both modes - stub returns empty Vec when disabled
        let suggestions = completion::suggest_completions(
            current,
            self.input_buffer.as_str(),
            self.current_user.as_ref().map(|u| u.access_level),
        )?;

        // Behavior naturally adapts to empty vs. populated suggestions
        if suggestions.len() == 1 {
            // Auto-complete with single match
            self.complete_input(suggestions[0])?;
        } else if suggestions.len() > 1 {
            // Display multiple options
            self.display_suggestions(&suggestions)?;
        }
        // If empty (or feature disabled): no-op

        Ok(())
    }
}
```

**Benefits of Stub Function Pattern:**
- **Zero `#[cfg]` in main input processing**: Single code path, no branching
- **Zero runtime overhead**: Empty Vec creation optimized away by compiler
- **Unified architecture alignment**: Behavior determined by return value, not feature flags
- **Const-friendly**: No trait objects, just function pointers
- **Minimal `#[cfg]` surface**: Feature gates isolated to completion module only
- **Easy testing**: Can test both paths by toggling feature

**Code Size Impact:**
- **With feature disabled**: ~0 bytes (stub and caller optimized away)
- **With feature enabled**: ~1.5-2.5KB (tree traversal + string matching logic)

#### Implementation Details

**When completion is enabled:**
1. Tab key triggers path resolution and prefix matching
2. Current directory and access level determine visible options
3. Common prefix auto-completed if unambiguous
4. Multiple matches displayed for user selection
5. Directories shown with trailing `/` separator

**When completion is disabled:**
1. Tab key silently ignored (no action)
2. All completion code eliminated from binary
3. Zero runtime overhead
4. `CompletionResult` type and module not compiled

#### Build Examples

```bash
# Default build (completion enabled)
cargo build

# Minimal build without completion
cargo build --no-default-features --features authentication

# Embedded target with completion
cargo build --target thumbv6m-none-eabi --release

# Embedded target without completion (maximum size optimization)
cargo build --target thumbv6m-none-eabi --release \
  --no-default-features --features authentication

# Testing build without optional features
cargo build --no-default-features

# Explicit feature specification (same as default)
cargo build --features "authentication,completion"
```

#### When to Enable/Disable

**Enable completion when:**
- ✅ Interactive CLI usage expected (human operators)
- ✅ Flash size is not critically constrained (<90% capacity)
- ✅ User experience is a priority
- ✅ Training/learning environment for new users
- ✅ Development and debugging workflows

**Disable completion when:**
- ❌ Flash size is critically constrained (>95% capacity)
- ❌ Only programmatic/scripted CLI access expected
- ❌ Minimizing attack surface is required
- ❌ Every byte counts (bootloader, recovery mode, minimal systems)
- ❌ No interactive terminal available (headless operation)

**Security Considerations:**
- Completion reveals available commands/paths to authenticated users
- Does not bypass access control (respects `AccessLevel`)
- No sensitive data exposed through completion
- Minimal attack surface (stateless algorithm)
- Safe to enable in most security contexts

---

### Command History Feature

Command history provides arrow key navigation to recall and re-execute previously entered commands. While it significantly enhances interactive user experience, it can be disabled to save both flash and RAM in severely constrained embedded environments or when only programmatic/scripted CLI access is expected.

#### Cargo.toml Configuration

```toml
[features]
default = ["authentication", "completion", "history"]

# Core authentication system
authentication = []

# Tab completion for commands and paths
completion = []

# Command history navigation
history = []

[dependencies]
heapless = "0.8"

# No additional dependencies required for history
# (uses only heapless for bounded circular buffer)
```

#### Resource Impact

| Build Configuration | Flash Usage | RAM Impact | Use Case |
|---------------------|-------------|------------|----------|
| **With history (N=10)** | +~500-800 bytes | ~1.3 KB | Interactive CLI, debugging |
| **With history (N=4)** | +~500-800 bytes | ~0.5 KB | RAM-constrained interactive |
| **Without history** | Baseline | 0 bytes | Scripted/programmatic only |

**Memory Characteristics:**
- **Flash (code size)**: ~500-800 bytes for history logic (circular buffer, navigation state)
- **RAM (runtime)**: Configurable via const generic `HISTORY_SIZE`
  - Default (N=10): ~1.3 KB (10 entries × 128 bytes + overhead)
  - Constrained (N=4): ~0.5 KB (4 entries × 128 bytes + overhead)
  - Disabled: 0 bytes (entire structure eliminated)
- **Why not just use N=0?** Zero-capacity saves RAM but not flash (code still compiled). Feature gating eliminates both.

**Design Decision: Feature Gating vs. Zero-Capacity**

We chose feature gating over allowing `HISTORY_SIZE = 0` because:
1. **Flash savings matter**: ~500-800 bytes saved on RP2040 (meaningful for bootloaders)
2. **RAM already configurable**: Users who want history can choose their desired capacity
3. **Clear intent**: Disabled feature vs. confusing "enabled but capacity 0" configuration
4. **Code clarity**: Stub pattern eliminates dead code paths
5. **Consistent with completion**: Both are interactive UX features that can be omitted

#### Conditional Compilation - Stub Function Pattern

Command history uses the stub pattern to maintain a single code path:

```rust
// src/cli/history.rs - Module always exists, contents conditionally compiled
#![cfg_attr(not(feature = "history"), allow(unused_variables))]

use heapless::{String, Vec};

// Type exists in both modes with different implementations
#[cfg(feature = "history")]
pub struct CommandHistory<const N: usize> {
    buffer: Vec<String<128>, N>,
    position: Option<usize>,
    original_buffer: Option<String<128>>,
}

#[cfg(not(feature = "history"))]
pub struct CommandHistory<const N: usize> {
    _phantom: core::marker::PhantomData<[(); N]>,
}

// Feature-enabled: Full circular buffer implementation
#[cfg(feature = "history")]
impl<const N: usize> CommandHistory<N> {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            position: None,
            original_buffer: None,
        }
    }

    pub fn add(&mut self, cmd: &str) {
        if self.buffer.is_full() {
            self.buffer.remove(0);  // Remove oldest
        }
        let mut entry = String::new();
        entry.push_str(cmd).ok();
        self.buffer.push(entry).ok();
        self.position = None;  // Reset navigation
    }

    pub fn previous(&mut self) -> Option<String<128>> {
        if self.buffer.is_empty() {
            return None;
        }

        let pos = match self.position {
            None => self.buffer.len() - 1,
            Some(0) => return None,  // Already at oldest
            Some(p) => p - 1,
        };

        self.position = Some(pos);
        self.buffer.get(pos).cloned()
    }

    pub fn next(&mut self) -> Option<String<128>> {
        let pos = match self.position {
            None => return None,  // Not navigating
            Some(p) if p >= self.buffer.len() - 1 => {
                // Reached newest, restore original
                self.position = None;
                return self.original_buffer.take();
            }
            Some(p) => p + 1,
        };

        self.position = Some(pos);
        self.buffer.get(pos).cloned()
    }

    pub fn reset(&mut self) {
        self.position = None;
        self.original_buffer = None;
    }

    pub fn save_current(&mut self, buffer: String<128>) {
        self.original_buffer = Some(buffer);
    }
}

// Feature-disabled: Stub returns empty/no-op
#[cfg(not(feature = "history"))]
impl<const N: usize> CommandHistory<N> {
    pub fn new() -> Self {
        Self { _phantom: core::marker::PhantomData }
    }

    pub fn add(&mut self, _cmd: &str) {
        // No-op
    }

    pub fn previous(&mut self) -> Option<String<128>> {
        None  // No history available
    }

    pub fn next(&mut self) -> Option<String<128>> {
        None  // No history available
    }

    pub fn reset(&mut self) {
        // No-op
    }

    pub fn save_current(&mut self, _buffer: String<128>) {
        // No-op
    }
}
```

```rust
// src/cli/mod.rs
pub mod history;
pub use history::CommandHistory;

pub struct CliService<'tree, L, IO, const HISTORY_SIZE: usize>
where
    L: AccessLevel,
    IO: CharIo,
{
    history: CommandHistory<HISTORY_SIZE>,  // Always present
    // ... other fields
}

// Single implementation - NO feature gates needed!
impl<'tree, L, IO, const HISTORY_SIZE: usize> CliService<'tree, L, IO, HISTORY_SIZE> {
    fn handle_up_arrow(&mut self) -> Result<(), CliError> {
        // Works in both modes - stub returns None when disabled
        if let Some(cmd) = self.history.previous() {
            self.input_buffer = cmd;
            self.redraw_line()?;
        }
        Ok(())
    }

    fn handle_down_arrow(&mut self) -> Result<(), CliError> {
        if let Some(cmd) = self.history.next() {
            self.input_buffer = cmd;
            self.redraw_line()?;
        }
        Ok(())
    }

    fn handle_enter(&mut self) -> Result<(), CliError> {
        let cmd = self.input_buffer.clone();

        // Execute command...
        let result = self.execute_command(&cmd)?;

        // Add to history only if successful (stub no-ops when disabled)
        if result.is_success() {
            self.history.add(cmd.as_str());
        }

        self.history.reset();
        Ok(())
    }
}
```

**Benefits of Stub Function Pattern:**
- **Zero `#[cfg]` in main CLI service**: Single code path for input handling
- **Zero overhead when disabled**: Entire struct and all methods optimized away
- **Configurable capacity when enabled**: Use const generic `HISTORY_SIZE` (4, 10, etc.)
- **Unified architecture alignment**: Behavior determined by return value (None), not feature flags
- **Minimal `#[cfg]` surface**: Feature gates isolated to history module only
- **Easy testing**: Toggle feature to test both interactive and minimal builds

#### Build Examples

```bash
# Default build (history enabled with 10 entries)
cargo build

# Minimal build without history
cargo build --no-default-features --features authentication

# Embedded target with small history (4 entries)
# Note: HISTORY_SIZE configured via type parameter, not feature flag
cargo build --target thumbv6m-none-eabi --release

# Embedded target without history (maximum size + RAM optimization)
cargo build --target thumbv6m-none-eabi --release \
  --no-default-features --features authentication

# Test all feature combinations
cargo test --all-features
cargo test --no-default-features
cargo test --features authentication,history
```

#### When to Enable/Disable

**Enable history when:**
- ✅ Interactive CLI usage by human operators
- ✅ Development and debugging workflows
- ✅ RAM available (>512 bytes free for N=4, >1.3KB for N=10)
- ✅ Flash not critically constrained
- ✅ Repeated command execution expected
- ✅ Training/learning environment

**Disable history when:**
- ❌ Flash size is critically constrained (bootloaders, recovery mode)
- ❌ RAM extremely limited (<16 KB total system RAM)
- ❌ Only programmatic/scripted CLI access (commands sent once)
- ❌ Read-only/kiosk mode (users shouldn't recall commands)
- ❌ Headless operation with no interactive terminal
- ❌ Minimal attack surface required

**Security Considerations:**
- History stores successfully executed commands only (not failed attempts)
- Login credentials are never stored in history
- History cleared on logout (when authentication enabled)
- No sensitive data leakage through history recall
- Minimal attack surface (simple circular buffer)
- Safe to enable in most security contexts

---

### Combined Feature Configuration

Multiple features can be enabled or disabled in combination to suit different deployment scenarios.

#### Common Configuration Patterns

```toml
# Full-featured build (default)
[features]
default = ["authentication", "completion", "history"]

# Minimal embedded (size-optimized)
[features]
default = []

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
# Development workstation (full features, fast iteration)
cargo build --all-features

# Production embedded device (all features)
cargo build --target thumbv6m-none-eabi --release

# Constrained device (authentication only, ~4-5KB flash + 1.3KB RAM saved)
cargo build --target thumbv6m-none-eabi --release \
  --no-default-features --features authentication

# Unsecured lab equipment (interactive features only, for ease of use)
cargo build --no-default-features --features completion,history

# Minimal bootloader/recovery (no optional features)
cargo build --target thumbv6m-none-eabi --release --no-default-features

# CI/CD testing (test all feature combinations)
cargo test --all-features
cargo test --no-default-features
cargo test --features authentication
cargo test --features completion
cargo test --features history
cargo test --features authentication,completion
cargo test --features authentication,history
cargo test --features completion,history
```

#### Feature Dependencies

```
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
| `--no-default-features` | Baseline | Baseline | Absolute minimum |
| `--features authentication` | +~2 KB | +0 bytes | Secured, non-interactive |
| `--features completion` | +~2 KB | +0 bytes | Interactive, unsecured, stateless |
| `--features history` | +~0.5-0.8 KB | +1.3 KB (N=10) | Non-interactive with recall |
| `--features completion,history` | +~2.5-3 KB | +1.3 KB | Interactive, unsecured |
| `--features authentication,completion` | +~4 KB | +0 bytes | Secured, interactive, stateless |
| `--all-features` (default) | +~4.5-5 KB | +1.3 KB | Full-featured |

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

**Organized structure (~14 modules with all features):**

```
src/
├── lib.rs              # Public API and feature gates
├── cli/
│   ├── mod.rs          # CliService + Request enum + CliState enum
│   ├── parser.rs       # InputParser (escape sequences, line editing)
│   └── history.rs      # CommandHistory (circular buffer)
├── tree/
│   ├── mod.rs          # Node enum + Directory + Command structs
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
└── io.rs               # CharIo trait
```

**Rationale for consolidation:**
- **Request types**: Single enum provides type-safe dispatch via pattern matching
- **State management**: Inline in cli/mod.rs (small, tightly coupled with service)
- **Path resolution**: Methods on existing types (tree navigation as core concern)
- **Tree types**: Combined in tree/mod.rs (related const-init concerns)
- **Authentication**: Trait-based system in auth/ module (optional, pluggable backends)
- **Completion**: Free functions in tree/completion.rs (optional, stateless logic)

## References

- **SPECIFICATION.md**: Complete behavioral specification
- **IMPLEMENTATION.md**: Implementation tracking and phased development plan
- **SECURITY.md**: Authentication, access control, and security design
- **CLAUDE.md**: Working patterns and practical implementation guidance
