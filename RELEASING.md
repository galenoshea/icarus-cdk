# Release Process

This document describes how to release new versions of the Icarus SDK to crates.io.

## Prerequisites

1. **Install cargo-release**:
   ```bash
   cargo install cargo-release
   ```

2. **Set up GitHub Secret**:
   - Go to your GitHub repository settings
   - Navigate to Settings → Secrets and variables → Actions
   - Add a new secret named `CARGO_REGISTRY_TOKEN`
   - Value: Your crates.io API token (get from https://crates.io/me)

## Automated Release Process

The SDK uses automated publishing via GitHub Actions when version tags are pushed.

### 1. Prepare Release

First, ensure all changes are committed and pushed:
```bash
git status
git push origin main
```

### 2. Create Release

Use cargo-release to bump version and create tag:

```bash
# For patch release (0.1.0 → 0.1.1)
cargo release patch --execute

# For minor release (0.1.0 → 0.2.0)
cargo release minor --execute

# For major release (0.1.0 → 1.0.0)
cargo release major --execute

# For a dry run (see what would happen)
cargo release patch --dry-run
```

This will:
1. Update version in all Cargo.toml files
2. Update version references in README.md
3. Create a git commit with message "Release version X.Y.Z"
4. Create a git tag "vX.Y.Z"
5. Push the commit and tag to GitHub

### 3. Automated Publishing

Once the tag is pushed, GitHub Actions will:
1. Run all tests and checks
2. Publish crates in dependency order:
   - icarus-core
   - icarus-derive
   - icarus-canister
   - icarus (main crate)
3. Create a GitHub Release with links to crates.io

### 4. Monitor Release

Check the release status:
- GitHub Actions: https://github.com/galenoshea/icarus-sdk/actions
- Crates.io: https://crates.io/crates/icarus

## Manual Publishing (Fallback)

If automated publishing fails, you can publish manually:

```bash
# In dependency order:
cd crates/icarus-core && cargo publish
cd ../icarus-derive && cargo publish
cd ../icarus-canister && cargo publish
cd ../.. && cargo publish
```

## Version Synchronization

All workspace crates share the same version number, configured in:
- `/Cargo.toml` (workspace version)
- Individual crates inherit via `version.workspace = true`

## Pre-release Versions

For pre-release versions (alpha, beta, rc):
```bash
cargo release --pre-release alpha --execute  # 0.1.0 → 0.1.1-alpha.0
cargo release --pre-release beta --execute   # 0.1.0 → 0.1.1-beta.0
```

## Troubleshooting

### Version Mismatch Error
If the GitHub Action fails with a version mismatch:
1. Ensure all Cargo.toml files have the same version
2. Check that the git tag matches the Cargo.toml version

### Publishing Order Error
Crates must be published in dependency order. The workflow handles this automatically with sleep delays between publishes.

### Token Issues
If publishing fails with authentication errors:
1. Regenerate your crates.io token
2. Update the `CARGO_REGISTRY_TOKEN` secret in GitHub

## Best Practices

1. **Always test locally first**: Run `cargo test` and `cargo clippy`
2. **Use semantic versioning**: Follow [semver.org](https://semver.org) guidelines
3. **Update CHANGELOG.md**: Document changes before releasing
4. **Check dependencies**: Ensure all dependencies are compatible
5. **Monitor the release**: Watch GitHub Actions for any issues

## Release Checklist

- [ ] All tests pass locally
- [ ] CHANGELOG.md is updated
- [ ] Documentation is up to date
- [ ] No uncommitted changes
- [ ] Version bump type decided (patch/minor/major)
- [ ] GitHub CARGO_REGISTRY_TOKEN secret is set
- [ ] Ready to support users after release