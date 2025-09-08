# E2E Test Performance Optimization Analysis

## Current Bottlenecks

### 1. **Test Execution Times**
- `test_new_command`: ~78s total (7 tests, ~0.24s actual test time)
- `test_build_command`: ~22s total (8 tests, ~15s actual test time)
- Most time spent on compilation, not test execution

### 2. **Key Issues Identified**

#### A. Serial Test Execution
- Tests run with `--test-threads=1` in CI
- SharedTestProject uses a global lock, forcing serialization
- No parallelization between test files

#### B. Build Overhead
- Each test file recompiles test harness (~60-75s)
- Shared project helps but still requires locks
- `cargo build --release` for WASM takes significant time

#### C. Redundant Operations
- Multiple tests verify the same functionality
- Some tests create new projects when SharedTestProject would suffice
- Build verification happens multiple times

## Optimization Opportunities

### 1. **Enable Parallel Execution** (30-50% improvement)
```rust
// Remove --test-threads=1 from CI
// Use test-specific directories instead of shared locks
// Leverage parallel::get_test_project_dir()
```

### 2. **Optimize SharedTestProject** (40-60% improvement)
```rust
// Pre-build in CI as artifact
// Remove locks for read-only operations
// Use copy-on-write for modifications
```

### 3. **CI-Specific Optimizations**
- GitHub Actions matrix to run test files in parallel
- Cache cargo registry and target directories
- Use sccache for distributed compilation
- Pre-compile WASM artifacts

### 4. **Test Consolidation**
- Combine related assertions to reduce project creation
- Use snapshots for validation instead of full builds
- Share build artifacts between tests

## Implementation Plan

### Phase 1: Enable Parallel Execution
- Remove global locks from read-only tests
- Use parallel-safe project directories
- Enable parallel test execution

### Phase 2: CI Optimization
- Set up GitHub Actions cache
- Implement test sharding
- Pre-build shared artifacts

### Phase 3: Test Refactoring
- Consolidate redundant tests
- Optimize slow tests
- Implement snapshot testing