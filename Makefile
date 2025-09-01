.PHONY: help test build clean release install-hooks ci

# Default target
help:
	@echo "Icarus SDK Development Commands"
	@echo "================================"
	@echo ""
	@echo "Setup:"
	@echo "  make install-hooks  - Install git hooks for code quality"
	@echo ""
	@echo "Development:"
	@echo "  make test          - Run all tests"
	@echo "  make build         - Build all crates"
	@echo "  make clean         - Clean build artifacts"
	@echo "  make ci            - Run CI checks locally"
	@echo ""
	@echo "Release:"
	@echo "  make release-patch - Release patch version (0.x.y -> 0.x.y+1)"
	@echo "  make release-minor - Release minor version (0.x.y -> 0.x+1.0)"
	@echo "  make release-major - Release major version (x.y.z -> x+1.0.0)"

# Install git hooks
install-hooks:
	@./scripts/install-hooks.sh

# Run all tests
test:
	@cargo test --all

# Build all crates
build:
	@cargo build --all

# Clean build artifacts
clean:
	@cargo clean

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