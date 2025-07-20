# Contributing to Icarus SDK

Thank you for your interest in contributing to the Icarus SDK! This document provides guidelines and instructions for contributing to the open source foundation of the Icarus ecosystem.

## Code of Conduct

By participating in this project, you agree to abide by our Code of Conduct:
- Be respectful and inclusive
- Welcome newcomers and help them get started
- Focus on constructive criticism
- Respect differing viewpoints and experiences

## How to Contribute

### Reporting Issues

Before creating an issue, please:
1. Search existing issues to avoid duplicates
2. Use issue templates when available
3. Include relevant details:
   - SDK version (`cargo pkgid icarus-sdk`)
   - Rust version (`rustc --version`)
   - Minimal reproduction code
   - Error messages and stack traces

### Suggesting Features

Feature requests are welcome! Please:
1. Open an issue with `[Feature Request]` prefix
2. Describe the problem it solves
3. Provide usage examples
4. Consider implementation complexity

### Submitting Pull Requests

1. **Fork and Clone**
   ```bash
   git clone https://github.com/YOUR-USERNAME/icarus-sdk
   cd icarus-sdk
   ```

2. **Create a Branch**
   ```bash
   git checkout -b feature/your-feature-name
   # or
   git checkout -b fix/issue-description
   ```

3. **Make Changes**
   - Follow the coding standards below
   - Add tests for new functionality
   - Update documentation as needed

4. **Test Your Changes**
   ```bash
   # Run all tests
   cargo test --all

   # Run with PocketIC integration tests
   cargo test --all --features integration-tests

   # Check formatting
   cargo fmt --all -- --check

   # Run clippy
   cargo clippy --all -- -D warnings
   ```

5. **Commit Your Changes**
   ```bash
   git add .
   git commit -m "feat: add new storage type

   - Implement StableQueue for FIFO operations
   - Add comprehensive tests
   - Update documentation"
   ```

   Follow conventional commits:
   - `feat:` New feature
   - `fix:` Bug fix
   - `docs:` Documentation changes
   - `test:` Test additions/changes
   - `refactor:` Code refactoring
   - `perf:` Performance improvements
   - `chore:` Maintenance tasks

6. **Push and Create PR**
   ```bash
   git push origin feature/your-feature-name
   ```

   Then create a pull request on GitHub.

## Development Setup

### Prerequisites

- Rust 1.75.0 or later
- wasm32-unknown-unknown target
- dfx (for integration tests)

### Setup

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install dfx
sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)"

# Clone repo
git clone https://github.com/icarus-mcp/icarus-sdk
cd icarus-sdk

# Build
cargo build --all
```

## Coding Standards

### Rust Style

We follow standard Rust conventions:
- Use `cargo fmt` for formatting
- Pass `cargo clippy` with no warnings
- Prefer explicit over implicit
- Document public APIs

### Examples

**Good:**
```rust
/// Stores a key-value pair in stable storage.
///
/// # Arguments
/// * `key` - The unique identifier
/// * `value` - The data to store
///
/// # Returns
/// * `Ok(())` on success
/// * `Err(String)` with error description
#[update]
#[icarus_tool("Store data with a unique key")]
pub fn store(key: String, value: Data) -> Result<(), String> {
    validate_key(&key)?;
    STORAGE.with(|s| {
        s.borrow_mut().insert(key, value);
        Ok(())
    })
}
```

**Bad:**
```rust
// No documentation
pub fn store(k: String, v: Data) -> Result<(), String> {
    // No validation
    STORAGE.with(|s| {
        s.borrow_mut().insert(k, v);
        Ok(())
    })
}
```

### Testing

All new features must include tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_and_retrieve() {
        let key = "test_key".to_string();
        let value = Data { content: "test".to_string() };
        
        assert!(store(key.clone(), value.clone()).is_ok());
        assert_eq!(retrieve(&key), Some(value));
    }

    #[test]
    fn test_invalid_key() {
        let result = store("".to_string(), Data::default());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid key"));
    }
}
```

### Documentation

- Add rustdoc comments to all public items
- Include usage examples in module docs
- Update README.md for significant changes
- Add entries to CHANGELOG.md

## Project Structure

```
icarus-sdk/
├── icarus-core/        # Core traits and types
│   ├── src/
│   └── tests/
├── icarus-derive/      # Procedural macros
│   ├── src/
│   └── tests/
└── icarus-canister/    # ICP integration
    ├── src/
    └── tests/
```

### Key Areas for Contribution

**icarus-core:**
- New trait definitions
- Type conversions
- Error handling improvements

**icarus-derive:**
- Macro enhancements
- Better error messages
- Performance optimizations

**icarus-canister:**
- Storage abstractions
- Memory management
- Testing utilities

## Review Process

1. **Automated Checks**
   - CI runs tests, formatting, and linting
   - All checks must pass

2. **Code Review**
   - At least one maintainer approval required
   - Address all feedback constructively
   - Be patient - reviews take time

3. **Merge**
   - Maintainer merges when ready
   - Squash commits if requested
   - Delete branch after merge

## Release Process

Releases follow semantic versioning:
- MAJOR: Breaking changes
- MINOR: New features (backward compatible)
- PATCH: Bug fixes

Only maintainers can publish releases.

## Getting Help

- **Discord**: Join our [Discord server](https://discord.gg/icarus)
- **GitHub Discussions**: Ask questions and share ideas
- **Office Hours**: Weekly community calls (Thursdays 3pm UTC)

## Recognition

Contributors are recognized in:
- CHANGELOG.md entries
- GitHub contributors page
- Community announcements

## Legal

By contributing, you agree that your contributions will be licensed under the Apache 2.0 License.

## Quick Contribution Checklist

- [ ] Code follows Rust conventions
- [ ] All tests pass (`cargo test --all`)
- [ ] Code is formatted (`cargo fmt --all`)
- [ ] No clippy warnings (`cargo clippy --all`)
- [ ] Documentation is updated
- [ ] Commit messages follow conventions
- [ ] PR description explains changes
- [ ] CHANGELOG.md entry added (if applicable)

Thank you for contributing to Icarus SDK! 🚀