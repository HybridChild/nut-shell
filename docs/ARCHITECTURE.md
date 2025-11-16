# cli-service - Architecture

This document records the architectural decisions for cli-service. It explains the rationale behind structural choices and documents alternatives considered.

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
- Using `/tree` would conflict with absolute path syntax (`/system/tree`)
- Small reserved word list (3-4 keywords) validated at compile time
- Cleaner syntax for global commands

**Why no `tree` global command?**
- Engineers typically know structure (defined in code)
- Tab completion + `?` command sufficient for exploration
- Saves ~50-100 lines of tree rendering code
- Not needed for the intended use cases

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

---

## Feature Gating & Optional Features

### Overview

The Rust implementation provides optional features that can be enabled or disabled at compile time to accommodate different deployment scenarios and resource constraints. This allows fine-grained control over code size, dependencies, and functionality.

**Available Optional Features:**
- **authentication**: User login and access control system (default: enabled)
- **completion**: Tab completion for commands and paths (default: enabled)

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

#### Conditional Compilation

```rust
// src/tree/mod.rs
#[cfg(feature = "completion")]
pub mod completion;

#[cfg(feature = "completion")]
pub use completion::{CompletionResult, complete_path};

// src/cli/mod.rs
pub struct CliService<'tree, L, IO>
where
    L: AccessLevel,
    IO: CharIo,
{
    #[cfg(feature = "completion")]
    last_completion: Option<CompletionResult>,

    // ... other fields
}

// Tab key handling with dual implementation
#[cfg(feature = "completion")]
impl<'tree, L, IO> CliService<'tree, L, IO> {
    fn handle_tab(&mut self) -> Result<Response, CliError> {
        // Full completion logic
        #[cfg(feature = "authentication")]
        let access_level = self.current_user.as_ref().map(|u| &u.access_level);

        #[cfg(not(feature = "authentication"))]
        let access_level = None;  // No auth: all nodes visible

        let result = completion::complete_path(
            &self.input_buffer,
            self.current_directory(),
            access_level
        )?;

        // Store for potential re-display
        self.last_completion = Some(result.clone());

        // Return completion suggestions to user
        Ok(Response::completion(result))
    }
}

#[cfg(not(feature = "completion"))]
impl<'tree, L, IO> CliService<'tree, L, IO> {
    fn handle_tab(&mut self) -> Result<Response, CliError> {
        // Option 1: Silent ignore (recommended for embedded)
        Ok(Response::empty())

        // Option 2: Echo literal tab character
        // self.io.put_char('\t')?;
        // Ok(Response::empty())
    }
}
```

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

### Combined Feature Configuration

Multiple features can be enabled or disabled in combination to suit different deployment scenarios.

#### Common Configuration Patterns

```toml
# Full-featured build (default)
[features]
default = ["authentication", "completion"]

# Minimal embedded (size-optimized)
[features]
default = []

# Interactive but unsecured (development only)
[features]
default = ["completion"]

# Secured but non-interactive (scripted access)
[features]
default = ["authentication"]
```

#### Build Examples by Scenario

```bash
# Development workstation (full features, fast iteration)
cargo build --all-features

# Production embedded device (both features)
cargo build --target thumbv6m-none-eabi --release

# Constrained device (authentication only, ~2KB saved)
cargo build --target thumbv6m-none-eabi --release \
  --no-default-features --features authentication

# Unsecured lab equipment (completion only, for ease of use)
cargo build --no-default-features --features completion

# Minimal bootloader/recovery (no optional features)
cargo build --target thumbv6m-none-eabi --release --no-default-features

# CI/CD testing (test all feature combinations)
cargo test --all-features
cargo test --no-default-features
cargo test --features authentication
cargo test --features completion
```

#### Feature Dependencies

```
authentication (independent)
  ├── No dependencies on other features
  └── Requires: sha2, subtle (optional crates)

completion (independent)
  ├── No dependencies on other features
  └── Requires: No additional crates (uses heapless only)

Note: Features are completely independent and can be
enabled in any combination without conflicts.
```

#### Code Size Comparison

| Configuration | Estimated Flash | Use Case |
|---------------|----------------|----------|
| `--no-default-features` | Baseline | Absolute minimum |
| `--features authentication` | Baseline + ~2KB | Secured, non-interactive |
| `--features completion` | Baseline + ~2KB | Interactive, unsecured |
| `--features authentication,completion` | Baseline + ~4KB | Full-featured (default) |

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
