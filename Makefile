.PHONY: help test test-e2e test-all build clean deep-clean release install-hooks ci

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
	@echo "  make test-e2e      - Run end-to-end CLI tests"
	@echo "  make test-all      - Run all tests (unit, integration, and E2E)"
	@echo "  make build         - Build all crates"
	@echo "  make clean         - Clean build artifacts (cargo clean)"
	@echo "  make deep-clean    - Deep clean all artifacts, caches, and temporary files"
	@echo "  make ci            - Run CI checks locally"
	@echo ""
	@echo "Release:"
	@echo "  make release-patch - Release patch version (0.x.y -> 0.x.y+1)"
	@echo "  make release-minor - Release minor version (0.x.y -> 0.x+1.0)"
	@echo "  make release-major - Release major version (x.y.z -> x+1.0.0)"

# Install git hooks
install-hooks:
	@./scripts/install-hooks.sh

# Run unit and integration tests
test:
	@cargo test --all --lib --bins

# Run E2E tests for CLI
test-e2e:
	@echo "Building CLI binary for E2E tests..."
	@cargo build --package icarus-cli --bin icarus --release
	@echo "Running E2E tests for CLI..."
	@cd cli && cargo test --test '*' -- --test-threads=1

# Run all tests
test-all: test test-e2e

# Build all crates
build:
	@cargo build --all

# Clean build artifacts (basic)
clean:
	@cargo clean

# Deep clean - removes all artifacts, caches, and temporary files
deep-clean:
	@./scripts/clean.sh

# Run CI simulation locally
ci:
	@./scripts/test-ci.sh

# Release commands
release-patch:
	@./scripts/release.sh patch

release-minor:
	@./scripts/release.sh minor

release-major:
	@./scripts/release.sh major