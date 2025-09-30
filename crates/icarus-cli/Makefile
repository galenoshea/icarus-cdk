# Icarus CLI Test Runner and Development Makefile
# Provides comprehensive testing, building, and CI integration

.PHONY: help test test-unit test-integration test-e2e test-mcp test-templates test-all
.PHONY: build build-release clean check fmt clippy
.PHONY: ci-test ci-prepare ci-install-deps coverage
.PHONY: bench profile install uninstall
.DEFAULT_GOAL := help

# Configuration
CARGO := cargo
RUST_LOG ?= info
TEST_TIMEOUT ?= 300
CARGO_TARGET_DIR ?= target

# Colors for output
RED := \033[0;31m
GREEN := \033[0;32m
YELLOW := \033[0;33m
BLUE := \033[0;34m
MAGENTA := \033[0;35m
CYAN := \033[0;36m
WHITE := \033[0;37m
RESET := \033[0m

# Help target
help: ## Show this help message
	@echo "$(CYAN)Icarus CLI Development Makefile$(RESET)"
	@echo ""
	@echo "$(YELLOW)Available targets:$(RESET)"
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  $(GREEN)%-20s$(RESET) %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo ""
	@echo "$(YELLOW)Environment Variables:$(RESET)"
	@echo "  $(BLUE)RUST_LOG$(RESET)         Set log level (default: info)"
	@echo "  $(BLUE)TEST_TIMEOUT$(RESET)     Test timeout in seconds (default: 300)"
	@echo "  $(BLUE)CARGO_TARGET_DIR$(RESET) Target directory for builds (default: target)"

# Development targets
build: ## Build the CLI in debug mode
	@echo "$(BLUE)Building Icarus CLI (debug)...$(RESET)"
	$(CARGO) build

build-release: ## Build the CLI in release mode
	@echo "$(BLUE)Building Icarus CLI (release)...$(RESET)"
	$(CARGO) build --release

install: build-release ## Install the CLI binary locally
	@echo "$(BLUE)Installing Icarus CLI...$(RESET)"
	$(CARGO) install --path .

uninstall: ## Uninstall the CLI binary
	@echo "$(BLUE)Uninstalling Icarus CLI...$(RESET)"
	$(CARGO) uninstall icarus-cli || true

clean: ## Clean build artifacts
	@echo "$(BLUE)Cleaning build artifacts...$(RESET)"
	$(CARGO) clean
	rm -rf coverage/
	rm -rf target/criterion/

# Code quality targets
check: ## Run cargo check
	@echo "$(BLUE)Running cargo check...$(RESET)"
	$(CARGO) check --workspace --all-targets --all-features

fmt: ## Format code using rustfmt
	@echo "$(BLUE)Formatting code...$(RESET)"
	$(CARGO) fmt --all

fmt-check: ## Check code formatting
	@echo "$(BLUE)Checking code formatting...$(RESET)"
	$(CARGO) fmt --all -- --check

clippy: ## Run clippy lints
	@echo "$(BLUE)Running clippy...$(RESET)"
	$(CARGO) clippy --workspace --all-targets --all-features -- -D warnings

clippy-fix: ## Apply clippy suggestions
	@echo "$(BLUE)Applying clippy fixes...$(RESET)"
	$(CARGO) clippy --workspace --all-targets --all-features --fix

# Test targets
test-unit: ## Run unit tests only
	@echo "$(BLUE)Running unit tests...$(RESET)"
	RUST_LOG=$(RUST_LOG) $(CARGO) test --lib --bins tests::
	RUST_LOG=$(RUST_LOG) $(CARGO) test unit_tests

test-integration: ## Run integration tests only
	@echo "$(BLUE)Running integration tests...$(RESET)"
	RUST_LOG=$(RUST_LOG) $(CARGO) test integration_tests

test-e2e: ## Run end-to-end workflow tests
	@echo "$(BLUE)Running E2E workflow tests...$(RESET)"
	RUST_LOG=$(RUST_LOG) $(CARGO) test e2e_workflow_tests

test-mcp: ## Run MCP integration tests
	@echo "$(BLUE)Running MCP integration tests...$(RESET)"
	RUST_LOG=$(RUST_LOG) $(CARGO) test mcp_integration_tests

test-templates: ## Run template validation tests
	@echo "$(BLUE)Running template validation tests...$(RESET)"
	RUST_LOG=$(RUST_LOG) $(CARGO) test template_validation_tests

test-utils: ## Run test utilities tests
	@echo "$(BLUE)Running test utilities tests...$(RESET)"
	RUST_LOG=$(RUST_LOG) $(CARGO) test test_utils

test: test-unit test-integration ## Run unit and integration tests
	@echo "$(GREEN)All basic tests completed!$(RESET)"

test-all: test-unit test-integration test-e2e test-mcp test-templates test-utils ## Run all tests
	@echo "$(GREEN)All tests completed successfully!$(RESET)"

test-quick: ## Run quick subset of tests for development
	@echo "$(BLUE)Running quick test suite...$(RESET)"
	RUST_LOG=warn $(CARGO) test --lib --bins
	RUST_LOG=warn $(CARGO) test unit_tests
	@echo "$(GREEN)Quick tests completed!$(RESET)"

# CI-specific targets
ci-prepare: ## Prepare CI environment
	@echo "$(BLUE)Preparing CI environment...$(RESET)"
	rustup component add rustfmt clippy
	$(CARGO) --version
	rustc --version

ci-install-deps: ## Install system dependencies for CI
	@echo "$(BLUE)Installing system dependencies...$(RESET)"
	@if command -v apt-get >/dev/null 2>&1; then \
		sudo apt-get update && sudo apt-get install -y build-essential pkg-config libssl-dev; \
	elif command -v yum >/dev/null 2>&1; then \
		sudo yum install -y gcc gcc-c++ openssl-devel; \
	elif command -v brew >/dev/null 2>&1; then \
		brew install openssl pkg-config; \
	fi

ci-test: ci-prepare fmt-check clippy check test-all ## Run full CI test suite
	@echo "$(GREEN)CI test suite completed successfully!$(RESET)"

# Coverage and analysis
coverage: ## Generate test coverage report
	@echo "$(BLUE)Generating coverage report...$(RESET)"
	@if command -v cargo-llvm-cov >/dev/null 2>&1; then \
		$(CARGO) llvm-cov --html --output-dir coverage test-all; \
		echo "$(GREEN)Coverage report generated in coverage/index.html$(RESET)"; \
	else \
		echo "$(YELLOW)cargo-llvm-cov not installed. Install with: cargo install cargo-llvm-cov$(RESET)"; \
	fi

coverage-lcov: ## Generate LCOV coverage report for CI
	@echo "$(BLUE)Generating LCOV coverage report...$(RESET)"
	@if command -v cargo-llvm-cov >/dev/null 2>&1; then \
		$(CARGO) llvm-cov --lcov --output-path coverage.lcov test-all; \
		echo "$(GREEN)LCOV coverage report generated: coverage.lcov$(RESET)"; \
	else \
		echo "$(YELLOW)cargo-llvm-cov not installed. Install with: cargo install cargo-llvm-cov$(RESET)"; \
	fi

# Benchmarking and profiling
bench: ## Run benchmarks
	@echo "$(BLUE)Running benchmarks...$(RESET)"
	$(CARGO) bench

profile: ## Run performance profiling
	@echo "$(BLUE)Running performance profile...$(RESET)"
	RUST_LOG=warn $(CARGO) test test_performance_metrics --release

# Development workflows
dev-setup: ## Set up development environment
	@echo "$(BLUE)Setting up development environment...$(RESET)"
	rustup update
	rustup component add rustfmt clippy
	@if ! command -v cargo-watch >/dev/null 2>&1; then \
		echo "$(BLUE)Installing cargo-watch...$(RESET)"; \
		$(CARGO) install cargo-watch; \
	fi
	@if ! command -v cargo-llvm-cov >/dev/null 2>&1; then \
		echo "$(BLUE)Installing cargo-llvm-cov...$(RESET)"; \
		$(CARGO) install cargo-llvm-cov; \
	fi
	@echo "$(GREEN)Development environment ready!$(RESET)"

dev-watch: ## Watch for changes and run tests
	@echo "$(BLUE)Watching for changes...$(RESET)"
	cargo-watch -x "test --lib" -x "test unit_tests"

dev-test-watch: ## Watch for changes and run specific test
	@echo "$(BLUE)Watching for changes (specific test)...$(RESET)"
	@read -p "Enter test name: " test_name; \
	cargo-watch -x "test $$test_name"

# Validation targets
validate-templates: ## Validate all project templates
	@echo "$(BLUE)Validating project templates...$(RESET)"
	RUST_LOG=warn $(CARGO) test test_all_templates_creation

validate-mcp: ## Validate MCP integration
	@echo "$(BLUE)Validating MCP integration...$(RESET)"
	RUST_LOG=warn $(CARGO) test test_mcp_integration_workflow

validate-build: ## Validate build process
	@echo "$(BLUE)Validating build process...$(RESET)"
	RUST_LOG=warn $(CARGO) test test_build_configurations

validate-all: validate-templates validate-mcp validate-build ## Run all validation tests
	@echo "$(GREEN)All validations passed!$(RESET)"

# Docker support (if needed)
docker-build: ## Build Docker image for testing
	@echo "$(BLUE)Building Docker test image...$(RESET)"
	docker build -t icarus-cli-test -f Dockerfile.test .

docker-test: ## Run tests in Docker container
	@echo "$(BLUE)Running tests in Docker...$(RESET)"
	docker run --rm -v $(PWD):/workspace icarus-cli-test make ci-test

# Utility targets
check-deps: ## Check for outdated dependencies
	@echo "$(BLUE)Checking dependencies...$(RESET)"
	@if command -v cargo-outdated >/dev/null 2>&1; then \
		$(CARGO) outdated; \
	else \
		echo "$(YELLOW)cargo-outdated not installed. Install with: cargo install cargo-outdated$(RESET)"; \
	fi

update-deps: ## Update dependencies
	@echo "$(BLUE)Updating dependencies...$(RESET)"
	$(CARGO) update

tree: ## Show dependency tree
	@echo "$(BLUE)Dependency tree:$(RESET)"
	$(CARGO) tree

audit: ## Run security audit
	@echo "$(BLUE)Running security audit...$(RESET)"
	@if command -v cargo-audit >/dev/null 2>&1; then \
		$(CARGO) audit; \
	else \
		echo "$(YELLOW)cargo-audit not installed. Install with: cargo install cargo-audit$(RESET)"; \
	fi

# Documentation
docs: ## Generate documentation
	@echo "$(BLUE)Generating documentation...$(RESET)"
	$(CARGO) doc --workspace --all-features --no-deps --open

docs-private: ## Generate documentation including private items
	@echo "$(BLUE)Generating documentation (including private)...$(RESET)"
	$(CARGO) doc --workspace --all-features --no-deps --document-private-items --open

# Release preparation
pre-release: clean fmt clippy check test-all coverage ## Run pre-release checks
	@echo "$(GREEN)Pre-release checks completed successfully!$(RESET)"
	@echo "$(YELLOW)Ready for release!$(RESET)"

release-dry-run: ## Dry run release process
	@echo "$(BLUE)Running release dry run...$(RESET)"
	$(CARGO) publish --dry-run

# Status and info
status: ## Show project status
	@echo "$(CYAN)Icarus CLI Project Status$(RESET)"
	@echo "$(BLUE)Rust version:$(RESET) $$(rustc --version)"
	@echo "$(BLUE)Cargo version:$(RESET) $$(cargo --version)"
	@echo "$(BLUE)Project version:$(RESET) $$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')"
	@echo "$(BLUE)Target directory:$(RESET) $(CARGO_TARGET_DIR)"
	@echo "$(BLUE)Test timeout:$(RESET) $(TEST_TIMEOUT)s"
	@echo "$(BLUE)Log level:$(RESET) $(RUST_LOG)"

env: ## Show environment information
	@echo "$(CYAN)Environment Information$(RESET)"
	@echo "$(BLUE)Operating System:$(RESET) $$(uname -s)"
	@echo "$(BLUE)Architecture:$(RESET) $$(uname -m)"
	@echo "$(BLUE)Available tools:$(RESET)"
	@for tool in cargo rustc git dfx; do \
		if command -v $$tool >/dev/null 2>&1; then \
			echo "  ✅ $$tool: $$($$tool --version 2>/dev/null | head -n1)"; \
		else \
			echo "  ❌ $$tool: not found"; \
		fi \
	done