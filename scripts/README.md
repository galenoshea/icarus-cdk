# Icarus SDK Scripts

This directory contains automation scripts for development and release workflows.

## Core Scripts

### 🧪 `test-ci.sh`
**Purpose**: Simulate GitHub Actions CI locally before pushing  
**Usage**: `./scripts/test-ci.sh`  
**When to use**: Before pushing changes to ensure CI will pass

### 🚀 `release.sh`
**Purpose**: Create a new release with automatic version bumping  
**Usage**: 
- `./scripts/release.sh patch` - Bump patch version (0.2.1 → 0.2.2)
- `./scripts/release.sh minor` - Bump minor version (0.2.1 → 0.3.0)
- `./scripts/release.sh major` - Bump major version (0.2.1 → 1.0.0)

**What it does**:
1. Checks for clean working directory
2. Runs all tests
3. Runs clippy checks
4. Uses cargo-release to bump versions
5. Creates git commit and tag
6. Pushes to GitHub to trigger release workflow

### 🔧 `install-hooks.sh`
**Purpose**: Install git hooks for automated quality checks  
**Usage**: `./scripts/install-hooks.sh`  
**When to use**: Once after cloning the repository

### 🧹 `clean.sh`
**Purpose**: Deep clean all build artifacts, caches, and temporary files  
**Usage**: `./scripts/clean.sh [--non-interactive]`  
**When to use**: When you need to clean all build artifacts and start fresh

**What it does**:
1. Removes target directories (Rust build artifacts)
2. Cleans .dfx directories (ICP local network data)
3. Removes temporary test directories
4. Cleans node_modules if present
5. Removes other build artifacts and caches

**Options**:
- `--non-interactive`: Skip confirmation prompts (useful in CI)

## Git Hooks (Installed by install-hooks.sh)

### Pre-commit Hook
- Checks code formatting
- Runs clippy with warnings as errors
- Runs tests

### Pre-push Hook
- Runs full CI simulation via test-ci.sh
- Ensures all checks pass before push

## Quick Start for New Contributors

```bash
# 1. Clone the repository
git clone https://github.com/galenoshea/icarus-sdk.git
cd icarus-sdk

# 2. Install git hooks
./scripts/install-hooks.sh

# 3. Before pushing changes, test locally
./scripts/test-ci.sh

# 4. To create a release
./scripts/release.sh patch
```

## Script Maintenance

- All scripts should be executable (`chmod +x`)
- Use consistent error handling and colored output
- Keep scripts focused on a single purpose
- Document any external dependencies