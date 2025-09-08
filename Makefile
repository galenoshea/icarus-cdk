.PHONY: help test test-quick test-e2e test-all test-pre-push test-parallel build clean deep-clean release install-hooks ci coverage

# Default target
help:
	@echo "Icarus SDK Development Commands"
	@echo "================================"
	@echo ""
	@echo "Setup:"
	@echo "  make install-hooks  - Install git hooks for code quality"
	@echo ""
	@echo "Development:"
	@echo "  make test          - Run unit and integration tests"
	@echo "  make test-quick    - Run only unit tests (fastest)"
	@echo "  make test-e2e      - Run end-to-end CLI tests (local only)"
	@echo "  make test-all      - Run all tests (unit, integration, and E2E)"
	@echo "  make test-pre-push - Run comprehensive tests as in pre-push hook"
	@echo "  make test-parallel - Run tests in parallel (faster execution)"
	@echo "  make build         - Build all crates"
	@echo "  make clean         - Clean build artifacts (cargo clean)"
	@echo "  make deep-clean    - Deep clean all artifacts, caches, and temporary files"
	@echo "  make ci            - Run CI checks locally (fast, no coverage)"
	@echo "  make coverage      - Run tests with code coverage analysis"
	@echo ""
	@echo "Release:"
	@echo "  make release-patch - Release patch version (0.x.y -> 0.x.y+1)"
	@echo "  make release-minor - Release minor version (0.x.y -> 0.x+1.0)"
	@echo "  make release-major - Release major version (x.y.z -> x+1.0.0)"

# Install git hooks
install-hooks:
	@./scripts/install-hooks.sh

# Run unit and integration tests (only packages with actual tests)
test:
	@echo "🧪 Running unit tests..."
	@cargo test --package icarus-core --package icarus-canister --lib
	@echo "🔧 Running CLI bin tests..."
	@cargo test --package icarus-cli --bins

# Quick test - unit tests only (fastest feedback)
test-quick:
	@echo "🚀 Running quick unit tests..."
	@cargo test --package icarus-core --package icarus-canister --lib --release --quiet

# Run E2E tests for CLI (now parallel by default)
test-e2e:
	@echo "Building CLI binary for E2E tests..."
	@cargo build --package icarus-cli --bin icarus --release
	@echo "Running E2E tests for CLI (parallel execution)..."
	@cd cli && cargo test --test '*' --release

# Run all tests
test-all: test test-e2e

# Run comprehensive tests as in pre-push hook
test-pre-push:
	@echo "🔍 Running format check..."
	@cargo fmt --all -- --check
	@echo "🔍 Running clippy..."
	@cargo clippy --all-targets --all-features -- -D warnings
	@echo "🧪 Running unit and integration tests..."
	@cargo test --all --lib --bins --tests
	@echo "🔨 Building CLI for E2E tests..."
	@cargo build --package icarus-cli --bin icarus --release
	@echo "🧪 Running E2E tests (this may take a few minutes)..."
	@cd cli && cargo test --test '*' --release
	@echo "✅ All pre-push tests passed!"

# Run tests in parallel (optimized for speed)
test-parallel:
	@echo "Running tests in parallel..."
	@if command -v cargo-nextest >/dev/null 2>&1; then \
		echo "Using cargo-nextest for maximum parallelization..."; \
		cargo nextest run --all-features --release; \
	else \
		echo "Unit & Integration tests..."; \
		cargo test --lib --bins --release & \
		PID1=$$!; \
		cargo test --test '*' --release & \
		PID2=$$!; \
		echo "E2E tests (parallel)..."; \
		cd cli && cargo test --test '*' --release & \
		PID3=$$!; \
		wait $$PID1 $$PID2 $$PID3; \
	fi; \
	echo "All parallel tests completed!"

# Build all crates
build:
	@cargo build --all

# Clean build artifacts (basic)
clean:
	@cargo clean

# Deep clean - removes all artifacts, caches, and temporary files
deep-clean:
	@./scripts/clean.sh

# Run CI simulation locally (fast, no coverage)
ci:
	@echo "🚀 Starting optimized CI simulation..."
	@START_TIME=$$(date +%s); \
	if [ -f ./scripts/ci.sh ]; then \
		./scripts/ci.sh; \
	elif [ -f ./scripts/test-ci.sh ]; then \
		./scripts/test-ci.sh; \
	else \
		echo "⚡ Running fast CI checks..."; \
		cargo fmt --all -- --check && \
		cargo clippy --all-targets --all-features -- -D warnings && \
		$(MAKE) test-parallel; \
	fi; \
	END_TIME=$$(date +%s); \
	DURATION=$$((END_TIME - START_TIME)); \
	echo "✅ CI completed in $$DURATION seconds"

# Run tests with code coverage analysis
coverage:
	@echo "Running tests with coverage analysis..."
	@if command -v cargo-llvm-cov >/dev/null 2>&1; then \
		cargo llvm-cov --all-features --workspace --html; \
		echo "Coverage report generated in target/llvm-cov/html/"; \
	else \
		echo "Installing cargo-llvm-cov..."; \
		cargo install cargo-llvm-cov; \
		cargo llvm-cov --all-features --workspace --html; \
	fi

# Release commands
release-patch:
	@./scripts/release.sh patch

release-minor:
	@./scripts/release.sh minor

release-major:
	@./scripts/release.sh major