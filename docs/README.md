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
Usage examples, platform implementations, configuration, troubleshooting. **Read when integrating nut-shell.**

### [DESIGN.md](DESIGN.md) - Architecture
Design decisions, patterns, feature gating, rationale. **Read when understanding why or implementing features.**

### [SECURITY.md](SECURITY.md) - Authentication & Access Control
Password hashing, credential storage, access control, threat model. **Read when implementing authentication.**

### [PHILOSOPHY.md](PHILOSOPHY.md) - Design Philosophy
Feature decision framework, what we include/exclude. **Read when evaluating feature requests.**

### [IO_DESIGN.md](IO_DESIGN.md) - I/O Abstraction
CharIo trait, buffering model, platform adapters. **Read when implementing CharIo.**

### [DEVELOPMENT.md](DEVELOPMENT.md) - Development Guide
Build commands, testing workflows, CI. **Read when building or contributing.**

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
