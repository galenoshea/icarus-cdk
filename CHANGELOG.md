# Changelog

All notable changes to the Icarus SDK will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial beta release of Icarus SDK
- Core traits for building MCP tools (`IcarusTool`, `IcarusStorable`)
- Procedural macros:
  - `#[icarus_module]` - Transform modules into MCP services
  - `#[icarus_tool]` - Mark functions as MCP tools
  - `#[derive(IcarusStorable)]` - Make types compatible with stable storage
- Stable storage abstractions:
  - `StableBTreeMap` for key-value storage
  - `StableVec` for append-only lists
  - `StableCell` for singleton values
- Memory management with isolated memory regions
- Automatic Candid generation from Rust types
- PocketIC integration for testing
- Comprehensive documentation and examples

### Security
- Input validation helpers
- Principal-based access control utilities

## [0.1.0-beta.1] - 2024-01-15

### Added
- First public beta release
- Basic MCP tool creation
- Simple storage patterns
- Initial documentation

### Known Issues
- Async tools not yet supported
- Limited error recovery in macros
- Performance optimizations pending

[Unreleased]: https://github.com/icarus-mcp/icarus-sdk/compare/v0.1.0-beta.1...HEAD
[0.1.0-beta.1]: https://github.com/icarus-mcp/icarus-sdk/releases/tag/v0.1.0-beta.1