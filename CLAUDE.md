# CLAUDE.md

This file provides guidance to Claude Code when working with this repository.

## Development Approach

**nut-shell** is a **production-ready** embedded CLI library. All core features are complete and fully tested.

**Maintenance Philosophy:**
- **Stability over features** - Avoid unnecessary changes to working code
- **Lean documentation** - Keep docs concise and professional; eliminate redundancy
- **Follow established patterns** - See DESIGN.md for architectural patterns
- **Test thoroughly** - All feature combinations must pass (see DEVELOPMENT.md)

**Before adding features:** Review PHILOSOPHY.md decision criteria. Propose significant changes via issue discussion before implementation.

---

## Documentation Standards

This repository maintains **professional, lean documentation**:

- **No redundancy** - Each concept explained once, in the right place
- **No verbosity** - Concise language, respect reader's time
- **No speculation** - Document what exists, not future possibilities
- **Proper distribution** - Examples in EXAMPLES.md, design in DESIGN.md, etc.

**When updating docs:**
1. Remove redundant explanations across files
2. Use tables/lists instead of verbose prose
3. Link to detailed docs instead of duplicating content
4. Keep README.md focused on quick start

---

## Quick Reference

### Adding a Command

**Pattern:** Metadata/execution separation (`CommandMeta` + `CommandHandler`)

**Steps:**
1. Define command function: `fn cmd<C: ShellConfig>(args: &[&str]) -> Result<Response<C>, CliError>`
2. Create const metadata: `CommandMeta { id, name, access_level, kind, min_args, max_args }`
3. Add to tree: `Node::Command(&CMD)` in directory's children
4. Implement handler: Map command ID in `CommandHandler::execute_sync()` or `execute_async()`

**Key points:**
- Unique `id` field for dispatch (allows duplicate names in different dirs)
- `CommandKind::Sync` or `Async` determines handler method
- See EXAMPLES.md for complete patterns, DESIGN.md for architecture

### Feature-Gated Module

**Stub Function Pattern** (recommended):
```rust
#[cfg(feature = "my_feature")]
pub fn do_something() -> Result<Vec<&str, 32>> { /* real impl */ }

#[cfg(not(feature = "my_feature"))]
pub fn do_something() -> Result<Vec<&str, 32>> { Ok(Vec::new()) }  // No-op stub
```

**Benefits:** Single code path, compiler optimizes away stubs. See DESIGN.md for complete patterns.

### Implementing Traits

**`CommandHandler`** - Maps command IDs to functions:
```rust
impl CommandHandler<MyConfig> for MyHandlers {
    fn execute_sync(&self, id: &str, args: &[&str]) -> Result<Response<MyConfig>, CliError> {
        match id {
            "status" => status_fn::<MyConfig>(args),
            _ => Err(CliError::CommandNotFound),
        }
    }
}
```

**`AccessLevel`** - Use derive macro:
```rust
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, AccessLevel)]
pub enum MyAccessLevel { Guest = 0, User = 1, Admin = 2 }
```

**`CharIo`** - Platform I/O abstraction:
```rust
impl CharIo for MyIo {
    type Error = MyError;
    fn get_char(&mut self) -> Result<Option<char>, Self::Error> { /* ... */ }
    fn put_char(&mut self, c: char) -> Result<(), Self::Error> { /* ... */ }
}
```

See EXAMPLES.md for complete patterns and usage.

---

## Critical Constraints

### no_std Environment
- **No heap allocation** - Use `heapless::Vec<T, N>`, `heapless::String<N>`
- **Fixed sizes at compile time** - Specify maximum capacity
- **Core dependencies only** - Check `default-features = false`
- **Tests are also no_std** - Test fixtures use `heapless` types to maintain consistency

### Static Allocation
- **Everything const-initializable** - Trees, commands, directories must be `const`
- **Lives in ROM** - Flash memory, not RAM
- **No runtime initialization** - No `lazy_static`, no `once_cell`

### String Handling
- **Const strings** - Use `&'static str` for names, descriptions
- **Runtime buffers** - Use `heapless::String<N>` with explicit capacity
- **Parsing** - Work with `&str` slices, avoid allocation

---

## Core Architecture

### Metadata/Execution Separation

Commands use **metadata/execution separation pattern**. Metadata (const in ROM) is separate from execution logic (generic trait).

```rust
// Metadata (const-initializable)
pub struct CommandMeta<L: AccessLevel> {
    pub id: &'static str,          // Unique ID for dispatch
    pub name: &'static str,        // Display name (can duplicate)
    pub kind: CommandKind,         // Sync or Async
    // ...
}

// Execution (generic trait)
pub trait CommandHandler<C: ShellConfig> {
    fn execute_sync(&self, id: &str, args: &[&str]) -> Result<Response<C>, CliError>;

    #[cfg(feature = "async")]
    async fn execute_async(&self, id: &str, args: &[&str]) -> Result<Response<C>, CliError>;
}
```

**Benefits:** Const-initialization, async support, zero-cost abstraction. See DESIGN.md for complete details.

### Unified Architecture (Authentication)

**Single code path** for both auth-enabled and auth-disabled modes:

```rust
pub struct Shell<'tree, L, IO, H, C> {
    current_user: Option<User<L>>,  // Always present
    state: CliState,                // Always present

    #[cfg(feature = "authentication")]
    credential_provider: &'tree dyn CredentialProvider<L>,  // Only field gated
}
```

**State semantics:**
- `Inactive` → Shell created but not activated
- `LoggedOut` → Awaiting login (auth enabled)
- `LoggedIn` → Ready for commands (authenticated or auth disabled)

Let `CliState` drive behavior, not feature flags. See DESIGN.md for complete pattern.

---

## Common Pitfalls

### ❌ Using std Types in no_std
```rust
// WRONG
fn parse(input: String) -> Vec<&str> { }

// RIGHT
fn parse(input: &str) -> heapless::Vec<&str, 16> { }
```

### ❌ Runtime Initialization
```rust
// WRONG
const TREE: Vec<Node> = vec![...];

// RIGHT
const TREE: &[Node] = &[...];
```

### ❌ heapless Buffer Overflow
```rust
// WRONG
buf.push_str(&long_string); // Can panic!

// RIGHT
buf.push_str(&long_string).map_err(|_| Error::BufferFull)?;
```

### ❌ Using N=0 Instead of Feature Gating
```rust
// WRONG: Saves RAM but not flash
type History = CommandHistory<0, 128>;

// RIGHT: Feature gate to eliminate code
#[cfg(feature = "history")]
type History = CommandHistory<10, 128>;

#[cfg(not(feature = "history"))]
type History = CommandHistory<0, 128>;  // Stub
```

---

## Build Commands

```bash
./scripts/ci-local                       # Run all CI checks locally (recommended)
cargo check                              # Fast check
cargo test --all-features                # Test all features
cargo test --no-default-features         # Test minimal
cargo check --target thumbv6m-none-eabi  # Verify no_std
cargo fmt && cargo clippy --all-features -- -D warnings  # Lint
```

**Before pushing:** Run `./scripts/ci-local` to verify all CI checks pass locally.

See DEVELOPMENT.md for complete workflows and CI configuration.

---

## Documentation Navigation

| Document | Purpose |
|----------|---------|
| **[EXAMPLES.md](docs/EXAMPLES.md)** | Implementation patterns, configuration, troubleshooting |
| **[DESIGN.md](docs/DESIGN.md)** | Architecture decisions and design rationale |
| **[SECURITY.md](docs/SECURITY.md)** | Authentication and access control patterns |
| **[PHILOSOPHY.md](docs/PHILOSOPHY.md)** | Design philosophy and feature criteria |
| **[CHAR_IO.md](docs/CHAR_IO.md)** | `CharIo` trait and platform adapters |
| **[DEVELOPMENT.md](docs/DEVELOPMENT.md)** | Build workflows, testing, CI |
| **`cargo doc --open`** | Complete API reference |

---

## Terminology Conventions

**Always use consistent terminology:**

- **Patterns:** "metadata/execution separation pattern", "unified architecture pattern", "stub function pattern"
- **Compound adjectives:** "path-based navigation", "feature-gated module", "const-initializable tree"
- **Code identifiers:** `Shell`, `CharIo`, `AccessLevel`, `no_std`, `CommandMeta`
- **Project name:** "nut-shell" (kebab-case)
- **Feature names:** `authentication`, `completion`, `history` (lowercase, no hyphens)

---

## Contributing Workflow

**Library Status:** Production-ready (maintenance mode)

**For contributions:**
1. Review PHILOSOPHY.md for feature criteria (default answer is NO)
2. Follow patterns in DESIGN.md
3. Write tests first (TDD)
4. Run `./scripts/ci-local` to verify all CI checks pass
5. Update documentation (keep lean and professional)

**Documentation updates:**
- Eliminate redundancy across files
- Keep explanations concise
- Use tables over verbose prose
- Link to detailed docs instead of duplicating

---

**This repository is maintained as a professional, production-ready library. Stability and clarity are priorities.**
