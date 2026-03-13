# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed
- Passing arguments to a directory path now returns `InvalidArgumentCount` instead of silently ignoring them

## [0.1.1] - 2026-03-03

### Added
- Helper scripts for repo-wide cleanup and formatting (`scripts/cleanup.sh`, `scripts/format.sh`)

### Fixed
- `history`: Navigation position now resets correctly on duplicate command submission
- `rp-pico` example: ADC read type annotation updated for rp2040-hal 0.12 API change

### Changed
- Updated dependency versions (`heapless`, `cortex-m`, `rp2040-hal`, and others)

## [0.1.0] - 2025-12-06

### Added
- Initial release
- Core CLI functionality with path-based navigation
- Command execution with synchronous and async support
- Input parsing with line editing (backspace, ESC-ESC clear)
- Global commands (`ls`, `?`, `clear`)
- Optional authentication feature with SHA-256 password hashing
- Optional tab completion for commands and paths (current directory only)
- Optional command history with arrow key navigation
- `CharIo` trait for platform-agnostic I/O
- `AccessLevel` trait with derive macro for access control
- `CommandHandler` trait for command execution
- Metadata/execution separation pattern for const-initializable command trees
- Comprehensive documentation and examples
- Support for `no_std` environments with zero heap allocation
- Tested on ARM Cortex-M0 (RP2040, STM32F072)

[Unreleased]: https://github.com/HybridChild/nut-shell/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/HybridChild/nut-shell/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/HybridChild/nut-shell/releases/tag/v0.1.0
