# Changelog

All notable changes to the Icarus SDK will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release of Icarus SDK
- Core abstractions for building MCP servers on ICP
- Procedural macros for reducing boilerplate
- Canister integration with stable memory
- Authentication system with whitelisting
- HTTP outcalls support
- Persistent state management
- Comprehensive documentation

### Features
- `icarus-core`: Protocol types, error handling, session management
- `icarus-derive`: Procedural macros including `#[icarus_module]` and `#[icarus_tool]`
- `icarus-canister`: Stable storage, memory management, auth tools
- Main `icarus` crate: Convenient re-exports and prelude

## [0.1.0] - TBD

- Initial release to crates.io

[Unreleased]: https://github.com/galenoshea/icarus-sdk/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/galenoshea/icarus-sdk/releases/tag/v0.1.0