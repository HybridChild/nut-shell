# nut-shell Documentation

Complete documentation for the nut-shell embedded CLI library.

## Quick Navigation

| Document | Purpose | Audience |
|----------|---------|----------|
| **[EXAMPLES.md](EXAMPLES.md)** | Usage examples and configuration patterns | Library users |
| **[DESIGN.md](DESIGN.md)** | Architecture decisions and design rationale | Contributors, advanced users |
| **[SECURITY.md](SECURITY.md)** | Authentication and access control patterns | Users implementing auth |
| **[PHILOSOPHY.md](PHILOSOPHY.md)** | Design philosophy and feature criteria | Contributors |
| **[IO_DESIGN.md](IO_DESIGN.md)** | CharIo trait and platform adapters | Platform implementers |
| **[DEVELOPMENT.md](DEVELOPMENT.md)** | Build commands and testing workflows | Contributors |

---

## Documentation by Audience

### For Library Users

**Getting Started:**
1. **[EXAMPLES.md](EXAMPLES.md)** - Start here for practical usage examples
   - Quick start guide
   - Configuration examples
   - Platform-specific implementations
   - Common patterns and troubleshooting

2. **[IO_DESIGN.md](IO_DESIGN.md)** - Implement CharIo for your platform
   - Buffering model explained
   - Sync and async patterns
   - Platform adapter examples

3. **[SECURITY.md](SECURITY.md)** - Add authentication (if needed)
   - Credential provider implementations
   - Access control patterns
   - Security considerations

**Additional Resources:**
- **API Documentation**: Run `cargo doc --open` for complete API reference
- **Examples**: Check `examples/` directory for working code

### For Contributors

**Understanding the Codebase:**
1. **[PHILOSOPHY.md](PHILOSOPHY.md)** - Understand project goals
   - Core principles
   - Feature decision framework
   - What we include vs exclude

2. **[DESIGN.md](DESIGN.md)** - Learn the architecture
   - Command architecture patterns
   - Feature gating patterns
   - Design decisions and rationale

3. **[DEVELOPMENT.md](DEVELOPMENT.md)** - Build and test
   - Build commands
   - Testing workflows
   - CI simulation
   - Troubleshooting

**Implementing Features:**
- See **[DESIGN.md](DESIGN.md)** for architectural patterns
- See **[PHILOSOPHY.md](PHILOSOPHY.md)** for feature criteria
- See **[DEVELOPMENT.md](DEVELOPMENT.md)** for testing strategies

---

## Documentation Overview

### [EXAMPLES.md](EXAMPLES.md) - Usage Guide
**28KB • Practical Examples**

Comprehensive usage examples and tutorials:
- Quick start (minimal example)
- Buffer sizing guide
- Platform examples (native stdio, RP2040 UART, Embassy async)
- Configuration examples (custom configs, feature combinations)
- Common patterns (command trees, handlers, authentication)
- Troubleshooting

**Read this when:** You're integrating nut-shell into your project.

---

### [DESIGN.md](DESIGN.md) - Architecture
**29KB • Design Decisions**

Architectural decisions and patterns:
- Command syntax rationale (path-based navigation)
- Metadata/execution separation pattern (sync + async commands)
- Feature gating patterns (unified architecture, stub functions)
- Access control system
- Module structure
- Design trade-offs and alternatives considered

**Read this when:** You want to understand why design choices were made or need to implement features following established patterns.

---

### [SECURITY.md](SECURITY.md) - Authentication & Access Control
**25KB • Security Architecture**

Security design and implementation guidance:
- Security considerations and limitations
- Password hashing (SHA-256 with salts)
- Credential storage options (build-time, flash, custom)
- Access control system (path-based validation)
- Authentication feature gating
- Implementation patterns
- Testing and validation strategies
- Threat model and assumptions

**Read this when:** Implementing authentication or evaluating security requirements.

---

### [PHILOSOPHY.md](PHILOSOPHY.md) - Design Philosophy
**18KB • Feature Framework**

Project philosophy and decision framework:
- Core principle: Essential CLI primitives only
- What we include (core functionality, interactive features)
- What we exclude (with rationale)
- Decision framework (5 key questions for new features)
- Design principles (8 core principles)
- Evolution guidelines

**Read this when:** Evaluating feature requests or understanding project scope.

---

### [IO_DESIGN.md](IO_DESIGN.md) - I/O Abstraction
**16KB • CharIo Guide**

I/O abstraction design and implementation:
- Design problem (bare-metal vs async runtimes)
- Explicit buffering model
- CharIo trait design
- Sync and async patterns
- Platform implementation examples
- Buffering strategy rationale

**Read this when:** Implementing CharIo for a new platform or debugging I/O issues.

---

### [DEVELOPMENT.md](DEVELOPMENT.md) - Development Guide
**10KB • Build & Test**

Build commands and development workflows:
- Quick reference (check, test, clippy, fmt)
- Feature validation (test all combinations)
- Embedded target verification
- Pre-commit validation
- CI simulation
- Troubleshooting
- Project structure

**Read this when:** Building, testing, or contributing to the project.

---

## Common Questions

### "How do I use nut-shell in my project?"
→ See **[EXAMPLES.md](EXAMPLES.md)** - Start with Quick Start

### "How do I implement CharIo for my platform?"
→ See **[IO_DESIGN.md](IO_DESIGN.md)** - Buffering Model & Patterns

### "How do I add authentication?"
→ See **[SECURITY.md](SECURITY.md)** - Credential Storage & Implementation Patterns

### "Why was it designed this way?"
→ See **[DESIGN.md](DESIGN.md)** - Architecture decisions with rationale

### "Should we add feature X?"
→ See **[PHILOSOPHY.md](PHILOSOPHY.md)** - Feature Decision Framework

### "How do I feature-gate a module?"
→ See **[DESIGN.md](DESIGN.md)** - Feature Gating Patterns

### "What's the build command for X?"
→ See **[DEVELOPMENT.md](DEVELOPMENT.md)** - Complete Build Command Reference

### "Where's the API documentation?"
→ Run `cargo doc --open` for complete API docs generated from source code

---

## Document Relationships

```
┌─────────────────┐
│  PHILOSOPHY.md  │  ← Why we build what we build
└────────┬────────┘
         │ guides
         ▼
┌─────────────────┐
│    DESIGN.md    │  ← Why it's designed this way
└────────┬────────┘
         │ implements
         ▼
┌─────────────────┐
│   Source Code   │  ← The implementation (see cargo doc)
└────────┬────────┘
         │ documented by
         ├────────────────────────┐
         │                        │
         ▼                        ▼
┌─────────────────┐      ┌───────────────┐
│  EXAMPLES.md    │      │ SECURITY.md   │
│  IO_DESIGN.md   │      │               │
│  DEVELOPMENT.md │      │               │
└─────────────────┘      └───────────────┘
    User guides          Security guide
```

---

## For AI Assistants (Claude Code)

See **[../CLAUDE.md](../CLAUDE.md)** for:
- Quick reference for common tasks
- Core architecture patterns
- Critical constraints (no_std, static allocation)
- Common pitfalls & solutions
- Testing patterns
- Build command reference

---

## Getting Help

1. **Check examples**: Look in `examples/` for working implementations
2. **API documentation**: Run `cargo doc --open` for detailed API docs
3. **This documentation**: Use the Quick Navigation table above
4. **Issues**: Report issues at https://github.com/anthropics/nut-shell/issues (update with actual repo URL)

---

**Project Status:** Production-ready library ✅
**Last Updated:** 2025 (update after implementation complete)
