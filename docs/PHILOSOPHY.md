# PHILOSOPHY

## Core Principle

> **nut-shell provides the essential interactive CLI primitives for embedded systems - nothing more, nothing less.**

Every feature must justify its existence through the lens of embedded constraints: flash size, RAM usage, and runtime overhead. We favor simplicity and predictability over convenience and flexibility.

---

## Scope

### What's Included

| Feature | Status | Cost | Rationale |
|---------|--------|------|-----------|
| **Path-based navigation** | Core | ~1KB flash | Essential CLI primitive |
| **Command execution** | Core | ~1KB flash | Sync and async via metadata/execution separation |
| **Access control** | Core | 0 bytes | User-defined hierarchies via traits |
| **Input parsing** | Core | ~1KB flash | Line editing with backspace, double-ESC |
| **Global commands** | Core | ~300 bytes | Essential interactive commands |
| **Authentication** | Optional | ~2KB flash | Password hashing, access control enforcement |
| **Tab completion** | Default | ~2KB flash | Command/path prefix matching, reduces typing |
| **Command history** | Default | ~0.8KB flash, ~1.3KB RAM | Arrow key navigation, configurable size (N=10) |

### What's Excluded

| Category | Features | Why Excluded |
|----------|----------|--------------|
| **Shell scripting** | Piping, variables, conditionals, loops | Requires dynamic allocation, host-side tools better suited |
| **Command aliases** | `st` → `status` shortcuts | Tab completion already solves this, costs flash for alias table |
| **Output paging** | `more`/`less` pagination | Terminal emulators provide scrollback, adds ~1-2KB |
| **Audit logging** | Persistent command history | Platform-specific, flash wear concerns, application-layer concern |
| **Advanced editing** | Vi-mode, undo/redo, Ctrl+K/U/W | Power-user features with diminishing returns (~50-100 bytes each) |
| **Session features** | Multiple sessions, auto-timeout | Requires multi-threading or timers (platform-specific) |
| **ANSI colors** | Built-in color support | Terminal capability detection complexity, application-specific |

**Note:** Audit logging can be implemented in your `CommandHandler` to meet application-specific requirements.

---

## Design Principles

### 1. Embedded-First Constraints

Design for resource-constrained microcontrollers, not Linux:
- **Flash:** 32KB-256KB typical
- **RAM:** 8KB-264KB typical
- **Serial:** 9600-115200 baud (slow connections)
- **No heap allocation** - Pure stack + static only
- **Single-threaded** - Deterministic execution
- **no_std compatible** - No standard library dependencies

### 2. Path-Based Philosophy

Unix-style paths replace traditional shell commands:
- `system/status` instead of `cd system && status`
- `../network/status` instead of `cd ../network && status`
- Prompt shows current location (`user@/current/path>`)
- Tab completion makes paths fast to type
- `ls` shows contents of current location

**Benefits:** Less typing, no state confusion, scriptable with absolute paths, natural for hierarchical commands.

### 3. Optional Features via Feature Flags

Features should be independently disableable:
- `--no-default-features` = minimal working CLI
- Each feature adds specific, measurable value
- No cascading dependencies between optional features
- Core functionality works without any optional features

### 4. Interactive Discovery

Users learn through interaction, not documentation:
- `?` shows global commands
- `ls` shows current directory with descriptions
- Tab completion reveals available options
- Error messages are specific and actionable

---

## Decision Criteria

### When to Include a Feature

1. **Essential for CLI functionality** - Without it, it's not a usable CLI
2. **Strong embedded use case** - Solves problem unique to embedded contexts
3. **No reasonable alternative** - Can't be handled by terminal, host tools, or application layer
4. **Justified cost** - Value proportional to flash/RAM consumption
5. **Feature-gatable** - Can be made optional if non-essential

### When to Exclude a Feature

1. **Terminal emulator handles it** - Scrollback, colors, line wrapping
2. **Host-side tools handle it better** - Scripting, batch commands, automation
3. **Application layer concern** - Logging, business logic, application-specific behavior
4. **Desktop shell behavior** - Feature doesn't translate to embedded constraints
5. **Requires dynamic allocation** - Breaks no_std compatibility
6. **Cost exceeds value** - Flash/RAM cost without proportional benefit

### Key Questions

Before adding a feature, ask:
1. Why can't the user implement this in their command handler?
2. Why can't the terminal emulator or host-side script handle this?
3. What are real embedded systems that need this?
4. How many bytes will this cost?
5. Does it work in no_std with static allocation only?

---

## Success Criteria

**Goals:**
- ✅ Compile for thumbv6m-none-eabi (Cortex-M0/M0+)
- ✅ Fit in <2KB flash with all features
- ✅ Zero heap allocation (stack + static only)
- ✅ Independent feature toggling
- ✅ Graceful degradation (minimal build still useful)

**Non-Goals:**
- ❌ Feature parity with bash/zsh
- ❌ Maximum flexibility/configurability
- ❌ Support every possible use case
- ❌ Be all things to all users

**When in doubt, say NO.** Features are forever. Simplicity is a feature.

---

## Related Documentation

- **[DESIGN.md](DESIGN.md)** - Architecture implementing these principles
- **[SECURITY.md](SECURITY.md)** - Security-by-design in authentication
- **[EXAMPLES.md](EXAMPLES.md)** - Usage examples within these constraints
- **[DEVELOPMENT.md](DEVELOPMENT.md)** - Build workflows and testing
