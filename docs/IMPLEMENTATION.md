# nut-shell Rust Port - Implementation Plan

**Status**: Planning Ongoing
**Estimated Timeline**: 6-8 weeks

## Overview

This document tracks the implementation phases for nut-shell. The implementation prioritizes **idiomatic Rust patterns** while maintaining behavioral correctness.

**⚠️ IMPORTANT**: Commands use **metadata/execution separation pattern**. `CommandMeta` (metadata) + `CommandHandlers` trait (execution). This enables both sync and async commands while maintaining const-initialization. See [DESIGN.md](DESIGN.md) section 1 for complete architecture details.

**When to use this document:**
- Finding out what phase of implementation we're in
- Understanding what needs to be built next
- Getting the complete build and validation workflow
- Checking task completion status

**Related Documentation:**
- **[DESIGN.md](DESIGN.md)**: Design decisions, command architecture, and rationale
- **[INTERNALS.md](INTERNALS.md)**: Complete runtime internals from input to output
- **[SPECIFICATION.md](SPECIFICATION.md)**: Exact behavioral requirements for each feature
- **[SECURITY.md](SECURITY.md)**: Security design for authentication features
- **[PHILOSOPHY.md](PHILOSOPHY.md)**: Design philosophy and feature decision framework
- **[../CLAUDE.md](../CLAUDE.md)**: Working patterns and practical guidance for implementing features

---

## Prerequisites: Essential Patterns

**IMPORTANT**: Before starting implementation, review these architectural patterns in DESIGN.md. Discovering these mid-implementation will require significant refactoring.

**Required reading:**
1. **Metadata/Execution Separation Pattern** ([DESIGN.md](DESIGN.md) Section 1) - Commands split into const metadata + generic trait for sync/async support
2. **Unified Architecture Pattern** ([DESIGN.md](DESIGN.md) Section 5.2) - Single code path for auth-enabled and auth-disabled modes
3. **Stub Function Pattern** ([DESIGN.md](DESIGN.md) Feature Gating sections) - Feature-gated modules with identical signatures
4. **Access Control Integration** ([INTERNALS.md](INTERNALS.md) Level 4) - Access checks during tree traversal

---

## Module Map

This shows which phase creates which file:

```
Phase 2:
  - src/io.rs (CharIo trait)
  - src/auth/mod.rs (AccessLevel trait, User struct)

Phase 3:
  - src/tree/mod.rs (Node enum, CommandMeta struct, Directory struct, CommandKind enum)
  - src/shell/handlers.rs (CommandHandlers trait definition)

Phase 4:
  - src/tree/path.rs (Path type)

Phase 5:
  - src/tree/completion.rs (with stub pattern)

Phase 6:
  - src/response.rs (Response type)
  - src/shell/mod.rs (Request enum, CliState enum - partial)

Phase 7:
  - src/shell/parser.rs (InputParser)
  - src/shell/history.rs (CommandHistory with stub pattern)

Phase 8:
  - src/shell/mod.rs (Shell implementation, complete)

Phase 9:
  - examples/basic.rs
  - examples/rp2040_uart.rs (optional)
```

## Implementation Phases

### Phase 1: Project Foundation ✓
**Goal**: Runnable Rust project with basic structure and testing infrastructure

**Tasks**:
- [ ] Create Cargo.toml with no_std support, heapless dependency
- [ ] Create src/lib.rs with feature gates and module declarations
- [ ] Create directory structure (shell/, tree/ modules with placeholder files)
- [ ] Create testing infrastructure:
  - `tests/fixtures/mod.rs` with `MockIo` implementation for CharIo
  - `tests/fixtures/mod.rs` with simple test tree (used across all phases)
  - `tests/fixtures/mod.rs` with `MockAccessLevel` enum (Guest/User/Admin)
- [ ] Document feature flag testing approach:
  - Test with all features: `cargo test --all-features`
  - Test with no features: `cargo test --no-default-features`
  - Test selective features: `cargo test --features authentication`
- [ ] Verify `cargo build` on native target
- [ ] Verify `cargo build --target thumbv6m-none-eabi` on embedded target

**Success Criteria**:
- Project compiles on both native and embedded targets
- MockIo available for testing CharIo implementations
- Test fixtures can be reused in subsequent phases

---

### Phase 2: I/O & Access Control Foundation
**Goal**: Core traits everything depends on

**Tasks**:
1. Implement `CharIo` trait in `io.rs`
   - Define trait with associated error type
   - Character read/write methods
   - Create `StdioStream` implementation for testing
   - Add basic tests

2. Implement access control in `auth/mod.rs` (see DESIGN.md "Module Structure")
   - `AccessLevel` trait with comparison operators
   - Example implementations (e.g., enum with Admin/User/Guest)
   - `User` struct with username and access level
   - Module is feature-gated, but `User` and `AccessLevel` are re-exported at root level (always available)
   - Only `CredentialProvider` requires authentication feature
   - Unit tests

**Success Criteria**: Can abstract I/O and access control with zero runtime cost

---

### Phase 3a: Tree Core Types
**Goal**: Define foundational tree types with access control

**Tasks**:
1. Implement in `tree/mod.rs`:
   - `Node` enum with Command and Directory variants
   - `Command` struct:
     - `name: &'static str`
     - `description: &'static str`
     - `execute: fn(&[&str]) -> Result<Response, CliError>` (function pointer)
     - `access_level: L` (generic over AccessLevel)
     - `min_args: usize`, `max_args: usize`
   - `Directory` struct:
     - `name: &'static str`
     - `children: &'static [Node<L>]` (array reference)
     - `access_level: L`
   - Type checking methods: `is_command()`, `is_directory()`, `name()`, `access_level()`

2. Unit tests for type construction and pattern matching

**Success Criteria**:
- Can define individual Command and Directory instances
- Node enum enables zero-cost dispatch via pattern matching
- Access level integration works with generic parameter

---

### Phase 3b: Tree Const Initialization
**Goal**: Build const-initializable tree structures in ROM

**Tasks**:
1. Implement const tree construction patterns:
   - Define command functions as `fn` (not closures)
   - Create const Command definitions
   - Create const Directory definitions with child arrays
   - Nest directories to create hierarchical structure
   - Example: `/system/reboot`, `/hw/led/set`, etc.

2. Create example tree as test fixture in `tests/fixtures/mod.rs`:
   ```rust
   pub const TEST_TREE: Directory<MockAccessLevel> = Directory {
       name: "/",
       children: &[/* ... */],
       access_level: MockAccessLevel::Guest,
   };
   ```

3. Verify const initialization with integration test
4. Verify tree can be placed in ROM (check with `nm` or `objdump`)

**Success Criteria**:
- Tree lives in ROM with zero runtime initialization
- Can construct complex nested tree structures at compile time
- Test fixture available for use in subsequent phases

---

### Phase 4: Path Navigation
**Goal**: Unix-style path resolution with access control integration

**Tasks**:
1. Implement `Path` type in `tree/path.rs`:
   - Parse absolute paths (`/foo/bar`)
   - Parse relative paths (`../foo`, `./bar`, `bar`)
   - Handle ".." (parent) and "." (current) components
   - Path normalization
   - Component iteration
   - Implement path parsing (~190 lines)

2. Add path resolution to `Directory` in `tree/mod.rs`:
   - `find_child(&self, name: &str) -> Option<&Node<L>>`
   - Basic tree walking without access control
   - Use index stack pattern: push child indices, pop for parent
   - Walk tree using stored indices

3. **IMPORTANT**: Prepare for access control integration (implemented in Phase 8):
   - Path resolution will need `current_user` context
   - Access checks happen at EVERY segment during traversal
   - Security principle: Return `Err(CliError::InvalidPath)` for both non-existent and inaccessible nodes
   - This prevents revealing existence of restricted commands
   - Note: Full integration happens in Phase 8 when `Shell::resolve_path()` is implemented

4. Comprehensive tests:
   - Path parsing edge cases
   - Parent navigation (`..`)
   - Absolute vs relative paths
   - Invalid paths return None
   - Deep tree navigation
   - Document placeholder for access control tests (added in Phase 8)

**Success Criteria**:
- Can navigate tree with complex paths like `../system/debug`
- Path resolution methods ready for access control integration
- Understand that full security integration requires `current_user` context (Phase 8)

---

### Phase 5: Tab Completion
**Goal**: Smart command/path completion (optional feature)

**Tasks**:
1. Implement in `tree/completion.rs`:
   - Prefix matching for commands and directories
   - Return multiple matches when ambiguous
   - Auto-append "/" for directories
   - Handle partial path completion (`sys/de<TAB>` → `system/debug`)
   - Implement completion logic (~229 lines)

2. Implement feature gating using stub function pattern (see DESIGN.md "Feature Gating & Optional Features"):
   - Add `completion` feature flag to Cargo.toml
   - Add `#[cfg(feature = "completion")]` conditional compilation within module contents
   - Implement stub function pattern: `suggest_completions()` returns empty `Vec` when disabled
   - Module always exists, contents are feature-gated
   - Single `handle_tab()` implementation calls stub functions (no dual methods needed)
   - Parser handles tab key identically in both modes (stub returns empty results)

3. Tests for completion scenarios:
   - Single match completion
   - Multiple match display
   - No matches
   - Directory vs command completion
   - Test builds with feature enabled/disabled
   - Verify no_std compliance with feature disabled
   - Measure code size impact (should be ~2KB)

**Success Criteria**:
- Tab completion works for partial names with proper directory handling
- Feature can be disabled via `--no-default-features` flag
- Graceful degradation when completion disabled
- Code size savings measurable (~2KB)

---

### Phase 6: Request/Response Types
**Goal**: Type-safe command processing types (MUST complete before Phase 7)

**Why this phase comes first**: Phase 7 (Input Processing) needs to convert input buffers into `Request` types, so these types must exist first.

**Tasks**:
1. Implement `Request` enum in `shell/mod.rs`:
   - `Login { username, password }` - Authentication attempt
   - `InvalidLogin` - Failed login
   - `Command { path, args, original }` - Execute command
   - `TabComplete { path }` - Request completions
   - `History { up, buffer }` - Navigate history

2. Implement `CliState` enum in `shell/mod.rs`:
   - `Inactive` - CLI not active
   - `LoggedOut` - Awaiting authentication (feature-gated variant)
   - `LoggedIn` - Authenticated or auth-disabled mode

3. Implement `Response` in `response.rs`:
   - Success/error variants
   - Formatting flags:
     - `prefix_newline` - Add newline before message
     - `indent_message` - Indent output (2 spaces)
     - `postfix_newline` - Add newline after message
     - `show_prompt` - Display prompt after response
   - Helper constructors: `Response::success()`, `Response::error()`
   - Message content and status code
   - See INTERNALS.md Level 7 for complete response formatting

4. Tests for request/response handling

**Success Criteria**:
- Can represent all CLI operations type-safely
- Response type supports all formatting modes needed by global commands and custom commands
- Input Parser (Phase 7) can convert buffers to Request types

---

### Phase 7: Input Processing
**Goal**: Terminal I/O with escape sequences

**Tasks**:
1. Implement `InputParser` in `shell/parser.rs`:
   - Character-by-character processing
   - Escape sequence state machine (up/down arrows, double-ESC)
   - Double-ESC clear buffer (always enabled, ~50-100 bytes, see PHILOSOPHY.md)
   - Backspace and delete handling
   - Tab key detection
   - Password masking mode for login
   - Buffer management with `heapless::String`
   - Convert buffer to Request when complete
   - Implement input parser (~397 lines)
   - Note: Left/right arrows, Home/End keys are future additions (see PHILOSOPHY.md "Recommended Additions")

2. Implement `CommandHistory` in `shell/history.rs` using stub type pattern (see DESIGN.md "Feature Gating & Optional Features"):
   - Circular buffer with const generic size
   - O(1) add, previous, next operations
   - Position tracking for navigation
   - Implement command history (~85 lines)
   - Feature-gated: Type always exists, methods no-op when `history` feature disabled
   - Zero-size stub type when disabled

3. Comprehensive tests:
   - Escape sequence parsing (up/down arrows, double-ESC)
   - Double-ESC clears buffer and exits history navigation
   - ESC + [ starts escape sequence (not cleared)
   - Backspace in middle of line
   - History navigation
   - Password masking
   - Buffer overflow handling

**Success Criteria**:
- Correctly parse all terminal input
- Handle arrows, backspace, tab, double-ESC
- Double-ESC clears input buffer without clearing screen
- O(1) history operations

---

### Phase 8: Shell Orchestration
**Goal**: Bring it all together with unified architecture

**Tasks**:
1. Implement `Shell` struct in `shell/mod.rs` using unified architecture pattern:

   a. **Field definitions**:
   ```rust
   pub struct Shell<'tree, L, IO>
   where
       L: AccessLevel,
       IO: CharIo,
   {
       // ALWAYS present (not feature-gated)
       tree: &'tree Directory<L>,
       current_user: Option<User<L>>,
       state: CliState,
       input_buffer: heapless::String<MAX_INPUT>,
       current_path: heapless::Vec<usize, MAX_PATH_DEPTH>,
       parser: InputParser,
       history: CommandHistory<HISTORY_SIZE>,
       io: IO,

       // ONLY this field is feature-gated
       #[cfg(feature = "authentication")]
       credential_provider: &'tree dyn CredentialProvider<L>,
   }
   ```

   b. **Feature-conditional constructors**:
   ```rust
   // Constructor when authentication enabled
   #[cfg(feature = "authentication")]
   impl<'tree, L, IO> Shell<'tree, L, IO> {
       pub fn new(
           tree: &'tree Directory<L>,
           provider: &'tree dyn CredentialProvider<L>,
           io: IO
       ) -> Self {
           Self {
               tree,
               current_user: None,  // Start logged out
               state: CliState::LoggedOut,
               credential_provider: provider,
               // ... other fields
           }
       }
   }

   // Constructor when authentication disabled
   #[cfg(not(feature = "authentication"))]
   impl<'tree, L, IO> Shell<'tree, L, IO> {
       pub fn new(
           tree: &'tree Directory<L>,
           io: IO
       ) -> Self {
           Self {
               tree,
               current_user: None,  // No user needed
               state: CliState::LoggedIn,  // Start in logged-in state
               // ... other fields
           }
       }
   }
   ```

   c. **Core methods** (same implementation for both modes):
   - `activate()` - Show welcome message, initial prompt
   - `process_char()` - Main character processing loop
   - `generate_prompt()` - Create `username@path> ` prompt (unified for both modes)
   - `resolve_path()` - Path navigation with access control checks at each segment
   - `execute_command()` - Run command with argument validation

   d. **Global commands**:
   - `help` - List global commands
   - `?` - Show current directory contents with descriptions
   - `logout` - End session (only available when authentication enabled)
   - `clear` - Clear screen (platform-dependent)

   e. **Integration with optional features** (using stub patterns):
   - Tab completion: calls `completion::suggest_completions()` (returns empty when disabled)
   - History navigation: calls `history.previous()`/`history.next()` (no-op when disabled)

   f. **Implement Shell orchestration** (~589 lines total)

   Note: No `cd`, `ls`, `pwd`, or `tree` commands per syntax design (see DESIGN.md)

2. Integration tests with mock I/O:
   - Login flow (auth enabled)
   - Navigation between directories
   - Command execution
   - Access control enforcement
   - Tab completion (both enabled and disabled via stubs)
   - History navigation (both enabled and disabled via stubs)
   - Test unified architecture: auth-enabled vs auth-disabled modes
   - Test feature combinations: all features, no features, individual features

**Success Criteria**:
- End-to-end CLI functionality works with all feature combinations
- Unified architecture correctly handles both auth modes
- Stub patterns enable graceful degradation when features disabled

---

### Phase 9: Examples
**Goal**: Demonstrate usage

**Tasks**:
1. Create `examples/basic.rs`:
   - Native stdio CLI
   - Example command tree (system commands, config, etc.)
   - Simple commands (echo, reboot, version, etc.)
   - Interactive session
   - Proper error handling

2. Create `examples/rp2040_uart.rs` (optional):
   - RP2040-specific UART I/O implementation
   - Minimal command tree for embedded
   - Hardware initialization
   - Verify on actual Pico hardware

3. Add documentation comments showing example usage

**Success Criteria**: Can run interactive CLI session with examples

---

### Phase 10: Testing & Polish
**Goal**: Match target quality and functionality

**Tasks**:
1. Write comprehensive tests:
   - Tree operations test
   - CLI test
   - Input parser test
   - Tab completion test
   - Command history test

2. Add Rust-specific tests:
   - Const initialization validation
   - Lifetime safety (compile tests)
   - Zero-size-type optimization checks
   - ROM placement verification

3. Documentation pass:
   - Module-level docs
   - Public API docs
   - Examples in docs
   - Architecture decision records

4. Performance validation:
   - Memory usage profiling
   - Stack usage analysis
   - Verify ROM placement
   - Measure baseline performance

5. Create README.md:
   - Project overview
   - Quick start guide
   - API examples
   - Build instructions
   - Performance characteristics

**Success Criteria**: Comprehensive test coverage, quality documentation

---

## Workflow Best Practices

### Test-Driven Development

For each phase:
1. **Write tests first** based on behavioral specification (see SPECIFICATION.md)
2. **Implement minimal functionality** to pass tests
3. **Iterate** until all tests pass
4. **Refine and optimize** with confidence
5. **Document** public APIs
6. **Commit** working increments

### Testing Strategy

**Unit Tests**: Per module, test individual components
- Path parsing: `path.rs`
- Tree navigation: `tree/mod.rs`
- History operations: `history.rs`
- Parser state machine: `parser.rs`

**Integration Tests**: End-to-end CLI functionality
- Login flow
- Command execution
- Navigation
- Tab completion
- History navigation

**Embedded Tests**: Platform-specific validation
- ROM placement verification
- Stack usage analysis
- Actual hardware testing (Pico)

### Build & Validation Commands - Complete Workflows

**Note:** For a quick reference, see CLAUDE.md "Common Build Commands"

#### Quick Iteration (Development)
```bash
cargo check                              # Fast compile check
cargo test                               # Run all tests
cargo test test_name                     # Run specific test
cargo clippy                             # Lint
cargo fmt                                # Format
cargo run --example basic                # Manual testing
```

#### Feature Validation
```bash
# Test all feature combinations
cargo test --all-features
cargo test --no-default-features
cargo test --no-default-features --features authentication
cargo test --no-default-features --features completion

# Verify compilation with specific features
cargo check --features authentication
cargo clippy --features completion
```

#### Embedded Target Verification
```bash
# Verify no_std compliance
cargo check --target thumbv6m-none-eabi

# Build for embedded (various configurations)
cargo build --target thumbv6m-none-eabi --release
cargo build --target thumbv6m-none-eabi --release --no-default-features
cargo build --target thumbv6m-none-eabi --release --features authentication

# Measure and compare binary sizes
cargo size --target thumbv6m-none-eabi --release -- -A
cargo size --target thumbv6m-none-eabi --release --no-default-features -- -A
```

#### Pre-Commit Validation
```bash
# Full check (one-liner)
cargo fmt && \
cargo clippy --all-features -- -D warnings && \
cargo test --all-features && \
cargo check --target thumbv6m-none-eabi --release

# Or step-by-step:
cargo fmt                                          # 1. Format
cargo check --all-features                         # 2. Compile check
cargo clippy --all-features -- -D warnings         # 3. Lint
cargo test --all-features                          # 4. Test
cargo check --target thumbv6m-none-eabi --release  # 5. Embedded check
cargo doc --no-deps --all-features                 # 6. Doc check
```

#### CI Simulation (Full Validation)
```bash
# All feature combinations
cargo test --all-features
cargo test --no-default-features
cargo test --features authentication
cargo test --features completion

cargo build --all-features
cargo build --no-default-features
cargo build --features authentication
cargo build --features completion

# Embedded builds
cargo build --target thumbv6m-none-eabi --release --all-features
cargo build --target thumbv6m-none-eabi --release --no-default-features

# Quality checks
cargo fmt -- --check
cargo clippy --all-features -- -D warnings
cargo clippy --no-default-features -- -D warnings
cargo doc --no-deps --all-features
```

#### Troubleshooting
```bash
cargo build -vv                          # Verbose build output
cargo tree                               # Show dependency tree
cargo tree --target thumbv6m-none-eabi   # Embedded dependencies
cargo tree --format "{p} {f}"            # Show feature resolution
cargo clean && cargo build               # Clean rebuild
cargo expand --lib                       # Expand macros
```

## Current Status

### Completed
- ✅ Architecture analysis and simplification (documented in CLAUDE.md)
- ✅ Implementation plan documentation
- ✅ Documentation structure refactored (CLAUDE.md = permanent, IMPLEMENTATION.md = task tracking)
- ✅ Cargo.toml created
- ✅ src/lib.rs created with module declarations

### In Progress
- 🟡 Phase 1: Project Foundation (directory structure pending)

### Upcoming
- ⬜ Phase 2: I/O & Access Control Foundation
- ⬜ Phase 3: Tree Data Model
- ⬜ Phase 4: Path Navigation
- ⬜ Phase 5: Tab Completion
- ⬜ Phase 6: Request/Response Types
- ⬜ Phase 7: Input Processing
- ⬜ Phase 8: Shell Orchestration
- ⬜ Phase 9: Examples
- ⬜ Phase 10: Testing & Polish

## Notes

- **Update this document** as implementation progresses (task completion status only)
- **Track blockers** and design questions as they arise
- **Archive when complete** (move to docs/ or delete) - this is a temporary tracking document
- **Reference CLAUDE.md** for architecture decisions and design rationale
- **Reference SPECIFICATION.md** for behavioral requirements
