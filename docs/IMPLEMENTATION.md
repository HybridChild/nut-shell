# CLIService Rust Port - Implementation Plan

**Status**: Planning Complete - Ready to Implement
**Last Updated**: 2025-11-09
**Estimated Timeline**: 6-8 weeks

## Overview

This document tracks the implementation phases for porting CLIService from C++ to Rust. The port prioritizes **idiomatic Rust patterns** over structural similarity to the C++ codebase, while maintaining functional equivalence.

**Key Principle**: Port C++ *behavior*, not *structure*.

**Related Documentation:**
- **ARCHITECTURE.md**: Design decisions, rationale, and comparison with C++ implementation
- **CLAUDE.md**: Working patterns and practical guidance for implementing features

## Implementation Phases

### Phase 1: Project Foundation ✓
**Goal**: Runnable Rust project with basic structure

**Tasks**:
- [x] Create Cargo.toml with no_std support, heapless dependency
- [x] Create src/lib.rs with feature gates and module declarations
- [ ] Create directory structure (cli/, tree/ modules with placeholder files)
- [ ] Verify `cargo build` on native target
- [ ] Verify `cargo build --target thumbv6m-none-eabi` on embedded target

**Success Criteria**: Project compiles on both native and embedded targets

---

### Phase 2: I/O & Access Control Foundation
**Goal**: Core traits everything depends on

**Tasks**:
1. Implement `CharIo` trait in `io.rs`
   - Define trait with associated error type
   - Character read/write methods
   - Create `StdioStream` implementation for testing
   - Add basic tests

2. Implement access control in `user.rs`
   - `AccessLevel` trait with comparison operators
   - Example implementations (e.g., enum with Admin/User/Guest)
   - `User` struct with username and access level
   - Unit tests

**C++ Reference**:
- `CharIOStreamIf.hpp` (trait definition)
- `UnixWinCharIOStream.cpp` (stdio implementation)
- `User.hpp` (user and access level)

**Success Criteria**: Can abstract I/O and access control with zero runtime cost

---

### Phase 3: Tree Data Model
**Goal**: Const-initializable directory tree in ROM

**Tasks**:
1. Implement in `tree/mod.rs`:
   - `Node` enum with Command and Directory variants
   - `Command` struct with function pointer, name, help, access level
   - `Directory` struct with name, children array, access level
   - Helper methods for const initialization
   - Type checking methods (is_command, is_directory)

2. Create example tree as test fixture
3. Verify const initialization with integration test
4. Verify tree can be placed in ROM (check with `nm` or `objdump`)

**C++ Reference**:
- `NodeIf.hpp` (base interface)
- `Directory.hpp` and `Directory.cpp`
- `CommandIf.hpp`

**Success Criteria**:
- Tree lives in ROM with zero runtime initialization
- Can construct complex tree structures at compile time

---

### Phase 4: Path Navigation
**Goal**: Unix-style path resolution using index stack

**Tasks**:
1. Implement `Path` type in `tree/path.rs`:
   - Parse absolute paths (`/foo/bar`)
   - Parse relative paths (`../foo`, `./bar`, `bar`)
   - Handle ".." (parent) and "." (current) components
   - Path normalization
   - Component iteration
   - Port logic from C++ `Path.cpp` (~190 lines)

2. Add path resolution to `Directory` in `tree/mod.rs`:
   - `find_child(&self, name: &str) -> Option<&Node>`
   - `resolve_path(&self, path: &Path) -> Option<&Node>`
   - Use index stack pattern: push child indices, pop for parent
   - Walk tree using stored indices

3. Comprehensive tests:
   - Path parsing edge cases
   - Parent navigation (`..`)
   - Absolute vs relative paths
   - Invalid paths return None
   - Deep tree navigation

**C++ Reference**:
- `Path.hpp` and `Path.cpp` (190 lines)
- `PathResolver.hpp` and `PathResolver.cpp` (83 lines)

**Success Criteria**: Can navigate tree with complex paths like `../system/debug`

---

### Phase 5: Tab Completion
**Goal**: Smart command/path completion (optional feature)

**Tasks**:
1. Implement in `tree/completion.rs`:
   - Prefix matching for commands and directories
   - Return multiple matches when ambiguous
   - Auto-append "/" for directories
   - Handle partial path completion (`sys/de<TAB>` → `system/debug`)
   - Port logic from C++ `PathCompleter` (229 lines, header-only)

2. Implement feature gating:
   - Add `completion` feature flag to Cargo.toml
   - Add `#[cfg(feature = "completion")]` conditional compilation to module
   - Implement dual `handle_tab()` methods (enabled/disabled versions)
   - Ensure `Response` type supports completion when feature enabled
   - Update parser to handle tab key appropriately when feature disabled

3. Tests for completion scenarios:
   - Single match completion
   - Multiple match display
   - No matches
   - Directory vs command completion
   - Test builds with feature enabled/disabled
   - Verify no_std compliance with feature disabled
   - Measure code size impact (should be ~2KB)

**C++ Reference**:
- `PathCompleter.hpp` (229 lines)

**Success Criteria**:
- Tab completion works for partial names with proper directory handling
- Feature can be disabled via `--no-default-features` flag
- Graceful degradation when completion disabled
- Code size savings measurable (~2KB)

---

### Phase 6: Request/Response Types
**Goal**: Type-safe command processing

**Tasks**:
1. Complete `Request` enum in `cli/mod.rs`:
   - Login { username, password }
   - InvalidLogin
   - Command { path, args, original }
   - TabComplete { path }
   - History { up, buffer }

2. Complete `CliState` enum in `cli/mod.rs`:
   - Inactive
   - LoggedOut
   - LoggedIn

3. Implement `Response` in `response.rs`:
   - Success/error variants
   - Formatting flags (show prompt, suppress output, etc.)
   - Helper constructors
   - Port from C++ `CLIResponse.hpp`

4. Tests for request/response handling

**C++ Reference**:
- `RequestBase.hpp`, `LoginRequest.hpp`, `CommandRequest.hpp`, etc. (multiple files)
- `CLIResponse.hpp`
- `CLIState.hpp`

**Success Criteria**: Can represent all CLI operations type-safely

---

### Phase 7: Input Processing
**Goal**: Terminal I/O with escape sequences

**Tasks**:
1. Implement `InputParser` in `cli/parser.rs`:
   - Character-by-character processing
   - Escape sequence state machine (arrows, home, end, etc.)
   - Backspace and delete handling
   - Tab key detection
   - Password masking mode for login
   - Buffer management with `heapless::String`
   - Convert buffer to Request when complete
   - Port from C++ `InputParser.cpp` (~397 lines)

2. Implement `CommandHistory` in `cli/history.rs`:
   - Circular buffer with const generic size
   - O(1) add, previous, next operations
   - Position tracking for navigation
   - Port from C++ `CommandHistory.hpp` (~85 lines)

3. Comprehensive tests:
   - Escape sequence parsing
   - Backspace in middle of line
   - History navigation
   - Password masking
   - Buffer overflow handling

**C++ Reference**:
- `InputParser.hpp` and `InputParser.cpp` (397 lines)
- `CommandHistory.hpp` (85 lines, header-only)

**Success Criteria**:
- Correctly parse all terminal input
- Handle arrows, backspace, tab
- O(1) history operations

---

### Phase 8: CLI Service Orchestration
**Goal**: Bring it all together

**Tasks**:
1. Implement `CliService` in `cli/mod.rs`:
   - Generic over `AccessLevel` and `CharIo`
   - Store root directory reference
   - Track current location (path stack of indices)
   - Store parser, history, current user
   - Process characters → requests → responses
   - Implement global commands: `?` (context help), `help` (global help), `logout` (auth feature), `clear` (optional)
   - Command execution with access control
   - Path resolution for navigation (absolute and relative paths)
   - Tab completion integration (feature-gated)
   - History navigation integration (arrow keys)
   - Prompt generation (username@path format)
   - Port orchestration from C++ `CLIService.cpp` (~589 lines)
   - Note: No `cd`, `ls`, `pwd`, or `tree` commands per syntax design (see ARCHITECTURE.md)

2. Integration tests with mock I/O:
   - Login flow
   - Navigation between directories
   - Command execution
   - Access control enforcement
   - Tab completion
   - History navigation

**C++ Reference**:
- `CLIService.hpp` and `CLIService.cpp` (589 lines)
- `CLIServiceConfiguration.hpp`
- `CLIMessages.hpp`

**Success Criteria**: End-to-end CLI functionality works

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
**Goal**: Match C++ quality and functionality

**Tasks**:
1. Port relevant C++ tests to Rust:
   - Tree operations test
   - CLI service test
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
   - Compare to C++ baseline

5. Create README.md:
   - Project overview
   - Quick start guide
   - API examples
   - Build instructions
   - Comparison to C++

**C++ Reference**:
- `test/` directory (8 test files)

**Success Criteria**: Comprehensive test coverage, quality documentation

---

## Workflow Best Practices

### Test-Driven Development

For each phase:
1. **Write tests first** (port from C++ reference or create new)
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

### Build Validation

Test on both targets regularly:
```bash
# Native (fast iteration)
cargo test
cargo run --example basic

# Embedded (verify no_std compliance)
cargo build --target thumbv6m-none-eabi --release
cargo size --release --target thumbv6m-none-eabi
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
- ⬜ Phase 8: CLI Service Orchestration
- ⬜ Phase 9: Examples
- ⬜ Phase 10: Testing & Polish

## Notes

- **Update this document** as implementation progresses (task completion status only)
- **Track blockers** and design questions as they arise
- **Archive when complete** (move to docs/ or delete) - this is a temporary tracking document
- **Reference CLAUDE.md** for architecture decisions and design rationale
- **Reference C++ implementation** for behavior, not structure
- **Maintain test parity** with C++ throughout development
