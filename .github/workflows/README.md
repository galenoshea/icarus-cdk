# GitHub Actions Workflows

This directory contains the CI/CD workflows for the Icarus SDK project.

## Workflows

### 🚀 CI (`ci.yml`)
**Purpose**: Primary continuous integration pipeline for all code changes.

**Triggers**:
- Push to `main` branch
- Pull requests to `main` branch
- Manual dispatch via GitHub UI

**Features**:
- ⚡ Optimized for speed (5-7 minutes typical runtime)
- 🔄 Parallel test execution for unit and integration tests
- 📦 Smart caching for dependencies and build artifacts
- 🚫 Auto-cancels in-progress runs when new commits are pushed
- ✅ Runs unit tests, integration tests, and E2E tests

**Jobs**:
1. **Build & Cache**: Builds all artifacts and caches for subsequent jobs
2. **Quick Tests**: Runs unit and doc tests in parallel
3. **Integration Tests**: Tests individual crates
4. **E2E Tests**: Sequential execution following real user workflow (new → build → validate)

---

### 📊 Coverage (`coverage.yml`)
**Purpose**: Deep code coverage analysis with detailed reporting.

**Triggers**:
- 📅 Weekly schedule (Sundays at midnight UTC)
- Push to `release/**` branches
- Manual dispatch via GitHub UI

**Features**:
- 🔍 Comprehensive test coverage using `cargo-llvm-cov`
- 📈 HTML and JSON coverage reports
- 🎯 Coverage thresholds enforcement (9.5% minimum)
- 🚫 Auto-cancels redundant runs

**Jobs**:
- Single job that runs all tests with coverage instrumentation
- Generates and uploads coverage artifacts

---

### 📦 Release (`release.yml`)
**Purpose**: Automated release process to crates.io and GitHub.

**Triggers**:
- Push of version tags (`v*.*.*`)
- Manual dispatch with optional dry-run

**Features**:
- ✅ Version validation across all workspace crates
- 📝 CHANGELOG verification
- 🏗️ Multi-platform binary builds (Linux, macOS, Windows)
- 📤 Sequential publishing to crates.io
- 🎉 GitHub release creation with artifacts
- 🔒 Prevents concurrent releases

**Jobs**:
1. **Validate**: Checks versions and CHANGELOG
2. **Test**: Runs tests on multiple platforms
3. **Build**: Creates release binaries for all platforms
4. **Publish**: Publishes crates to crates.io in dependency order
5. **Release**: Creates GitHub release with binaries

## Workflow Features

### Concurrency Control
All workflows use concurrency groups to:
- Cancel outdated CI runs when new commits are pushed
- Prevent multiple simultaneous releases
- Optimize resource usage

### Manual Triggers
All workflows support `workflow_dispatch` for manual execution through:
- GitHub Actions UI
- GitHub CLI: `gh workflow run <workflow-name>`
- VS Code GitHub Actions extension

### Caching Strategy
- **Cargo registry**: Cached across all workflows
- **Build artifacts**: Shared between jobs in the same workflow
- **Target directory**: Incremental compilation caching
- **Tool binaries**: `ic-wasm` and other tools cached

## Maintenance

### Adding a New Workflow
1. Create `.yml` file in this directory
2. Include concurrency group for resource optimization
3. Add workflow badge to main README.md
4. Document the workflow in this file

### Debugging Workflows
- Use `workflow_dispatch` for testing changes
- Check job logs in GitHub Actions tab
- Use VS Code GitHub Actions extension for local visualization
- Enable debug logging with `ACTIONS_STEP_DEBUG=true` secret

### Performance Tips
- Use path filters to skip unnecessary runs
- Leverage job matrices for parallel execution
- Share artifacts between jobs instead of rebuilding
- Use `cargo check` before `cargo build` when possible

## Related Documentation
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [VS Code GitHub Actions Extension](https://marketplace.visualstudio.com/items?itemName=GitHub.vscode-github-actions)
- [Cargo Documentation](https://doc.rust-lang.org/cargo/)