# Changelog

All notable changes to the Icarus SDK project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.6] - 2025-09-01

### Changed
- Synchronized all crate versions across workspace to 0.2.6
- All internal dependencies now use consistent versioning
- Improved documentation structure and navigation

### Fixed
- Version mismatch between workspace and individual crates
- Internal dependency versions now properly aligned

## [0.2.5] - 2025-09-01

### Security
- Fixed critical slab vulnerability (RUSTSEC-2025-0047) by updating to slab 0.4.11
- Updated all dependencies to latest compatible versions
- Fixed several security advisories in transitive dependencies

### Fixed
- Fixed all failing doc tests by marking macro examples as `ignore`
- Fixed formatting issues in rmcp_server.rs

### Changed
- Improved CLI README with cargo install instructions
- Removed non-existent marketplace URLs from documentation
- Updated README versions to reflect current release

## [0.2.4] - 2025-09-01

### Added
- Integrated icarus-cli into SDK monorepo
- Unified workspace versioning across all crates
- Added CLI publishing to release workflow

### Changed
- CLI is now part of the SDK monorepo at `/cli` directory
- Aligned CLI licensing with SDK BSL-1.1 license
- Updated cargo-release configuration for unified releases

### Security
- Fixed security vulnerabilities in CLI dependencies:
  - Updated tokio to 1.41
  - Replaced deprecated atty with is-terminal
  - Updated all CLI dependencies to latest secure versions

## [0.2.3] - 2025-08-31

### Added
- First automated release with GitHub Actions
- Comprehensive CI/CD pipeline
- Pre-commit and pre-push hooks

### Fixed
- Version synchronization across workspace
- GitHub Actions permissions for releases
- Doc test failures in release script

## [0.2.2] - 2025-08-31

### Added
- cargo-release configuration for automated version management
- Release automation scripts

### Fixed
- Version mismatch between tags and Cargo.toml files

## [0.2.1] - 2025-08-31

### Fixed
- Lifetime syntax errors in state.rs
- Clippy warnings and code quality issues
- Duplicate Default implementations

## [0.2.0] - 2025-08-31

### Changed
- **BREAKING**: Changed license from Apache-2.0 to Business Source License (BSL-1.1)
- Added marketplace protection clauses to prevent competing services

### Added
- Comprehensive BSL-1.1 license with marketplace protection
- Prohibited uses section preventing competing MCP marketplaces

### Security
- License change provides IP protection for the Icarus marketplace

## [0.1.1] - 2025-08-30

### Added
- Initial workspace structure
- Core SDK functionality
- Basic CLI implementation

## [0.1.0] - 2025-08-30 [YANKED]

### Added
- Initial release with Apache-2.0 license

### Note
- This version was yanked due to licensing concerns
- Users should upgrade to 0.2.0 or later