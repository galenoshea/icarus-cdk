# Contributing to Icarus SDK

Thank you for your interest in contributing to the Icarus SDK! This document provides guidelines and instructions for contributing to the project.

## Code of Conduct

By participating in this project, you agree to abide by our code of conduct:
- Be respectful and inclusive
- Welcome newcomers and help them get started
- Focus on constructive criticism
- Respect differing viewpoints and experiences

## How to Contribute

### Reporting Issues

Before creating an issue, please:
1. Search existing issues to avoid duplicates
2. Use the issue templates when available
3. Provide clear reproduction steps
4. Include relevant system information

### Suggesting Features

Feature requests are welcome! Please:
1. Check if the feature has already been requested
2. Clearly describe the use case
3. Explain why existing features don't solve your problem
4. Consider submitting a PR if you can implement it

### Pull Requests

We love pull requests! Here's how to contribute code:

#### 1. Setup Development Environment

```bash
# Fork and clone the repository
git clone https://github.com/YOUR_USERNAME/icarus-sdk.git
cd icarus-sdk

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install required tools
cargo install cargo-release
cargo install cargo-audit
rustup component add clippy rustfmt

# Build the project
cargo build --all
```

#### 2. Make Your Changes

- Create a new branch: `git checkout -b feature/your-feature-name`
- Make your changes following our coding standards
- Add tests for new functionality
- Update documentation as needed
- Ensure all tests pass: `cargo test --all`

#### 3. Code Quality Standards

Before submitting, ensure your code meets our standards:

```bash
# Format your code
cargo fmt --all

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test --all

# Check for security vulnerabilities
cargo audit
```

#### 4. Commit Guidelines

We follow conventional commits. Use these prefixes:
- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation changes
- `style:` Formatting, missing semicolons, etc.
- `refactor:` Code restructuring
- `test:` Adding tests
- `chore:` Maintenance tasks

Example:
```bash
git commit -m "feat: add support for custom storage backends"
```

#### 5. Submit Your PR

1. Push your branch: `git push origin feature/your-feature-name`
2. Create a pull request with a clear description
3. Link any related issues
4. Wait for review and address feedback

## Development Guidelines

### Project Structure

```
icarus-sdk/
├── crates/
│   ├── icarus-core/      # Core MCP protocol implementation
│   ├── icarus-derive/     # Procedural macros
│   └── icarus-canister/   # ICP canister integration
├── cli/                   # Command-line interface
├── docs/                  # Documentation
├── examples/              # Example projects
└── tests/                 # Integration tests
```

### Testing

We use a progressive testing strategy:

1. **Unit Tests**: Test individual functions
   ```rust
   #[test]
   fn test_memory_storage() {
       // Test implementation
   }
   ```

2. **Integration Tests**: Test component interactions
   ```bash
   cargo test --test '*'
   ```

3. **Doc Tests**: Ensure examples in documentation work
   ```bash
   cargo test --doc
   ```

### Documentation

- Add doc comments to all public APIs
- Include examples in doc comments when helpful
- Update relevant .md files in docs/
- Keep README.md up to date

### Performance Considerations

- Minimize allocations in hot paths
- Use `&str` instead of `String` where possible
- Leverage stable memory for large data structures
- Profile before optimizing

## Release Process

Releases are managed by maintainers:

1. Update version in Cargo.toml files
2. Update CHANGELOG.md
3. Create release PR
4. After merge, tag release: `git tag v0.2.x`
5. Push tag to trigger automated release

## Security

### Reporting Security Issues

**DO NOT** create public issues for security vulnerabilities.

Instead, please email security@icarus.dev with:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fixes (if any)

### Security Best Practices

When contributing:
- Never commit secrets or keys
- Validate all inputs
- Use safe Rust patterns
- Follow OWASP guidelines for web-facing code
- Consider IC-specific security concerns

## Getting Help

- **Discord**: Join our community server (coming soon)
- **GitHub Discussions**: Ask questions and share ideas
- **Documentation**: Check docs/ folder
- **Examples**: See examples/ for working code

## License

By contributing, you agree that your contributions will be licensed under the Business Source License (BSL-1.1) as described in the LICENSE file.

## Recognition

Contributors will be recognized in:
- The CONTRIBUTORS file
- Release notes
- Project documentation

Thank you for helping make Icarus SDK better!