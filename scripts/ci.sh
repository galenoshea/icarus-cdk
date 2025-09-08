#!/bin/bash
# Optimized CI runner with parallel execution and better error handling

set -euo pipefail

# Configuration
readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
# PARALLEL_JOBS can be overridden by command line, so don't make it readonly
PARALLEL_JOBS="${PARALLEL_JOBS:-$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)}"

# Colors for output (disabled in CI environment)
if [[ -t 1 ]] && [[ "${NO_COLOR:-}" != "1" ]] && [[ "${CI:-}" != "true" ]]; then
    readonly RED='\033[0;31m'
    readonly GREEN='\033[0;32m'
    readonly YELLOW='\033[1;33m'
    readonly BLUE='\033[0;34m'
    readonly CYAN='\033[0;36m'
    readonly NC='\033[0m'
else
    readonly RED='' GREEN='' YELLOW='' BLUE='' CYAN='' NC=''
fi

# Logging functions
log_info() { echo -e "${BLUE}ℹ${NC} $*"; }
log_success() { echo -e "${GREEN}✓${NC} $*"; }
log_warning() { echo -e "${YELLOW}⚠${NC} $*" >&2; }
log_error() { echo -e "${RED}✗${NC} $*" >&2; }
log_step() { echo -e "\n${CYAN}▶${NC} $*"; }

# Error handling
trap 'log_error "Script failed at line $LINENO"' ERR

# Usage information
show_help() {
    cat << EOF
Usage: $(basename "$0") [OPTIONS] [COMMANDS]

Optimized CI runner for Icarus SDK

OPTIONS:
    -h, --help          Show this help message
    -v, --verbose       Enable verbose output
    -q, --quiet         Suppress non-error output
    -j, --jobs NUM      Number of parallel jobs (default: $PARALLEL_JOBS)
    --no-cache          Disable cargo cache
    --no-fail-fast      Continue on errors
    --coverage          Run with coverage enabled

COMMANDS:
    all                 Run all checks (default)
    fmt, format         Check code formatting
    clippy              Run clippy lints
    test                Run all tests
    test-unit           Run unit tests only
    test-integration    Run integration tests only
    test-e2e           Run E2E tests only
    doc                 Build documentation
    build              Build all targets
    build-wasm         Build WASM target
    version            Check version consistency
    clean              Clean build artifacts

EXAMPLES:
    $(basename "$0")                    # Run all checks
    $(basename "$0") fmt clippy test    # Run specific checks
    $(basename "$0") -j 8 all           # Run with 8 parallel jobs
    $(basename "$0") --coverage test    # Run tests with coverage

EOF
}

# Parse command line arguments
VERBOSE=false
QUIET=false
NO_CACHE=false
FAIL_FAST=true
COVERAGE=false
COMMANDS=()

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -q|--quiet)
            QUIET=true
            shift
            ;;
        -j|--jobs)
            PARALLEL_JOBS="$2"
            shift 2
            ;;
        --no-cache)
            NO_CACHE=true
            shift
            ;;
        --no-fail-fast)
            FAIL_FAST=false
            shift
            ;;
        --coverage)
            COVERAGE=true
            shift
            ;;
        *)
            COMMANDS+=("$1")
            shift
            ;;
    esac
done

# Default to all checks if no commands specified
if [[ ${#COMMANDS[@]} -eq 0 ]]; then
    COMMANDS=("all")
fi

# Change to project root
cd "$PROJECT_ROOT"

# Setup cargo flags
CARGO_FLAGS=()
if [[ "$VERBOSE" == "true" ]]; then
    CARGO_FLAGS+=("--verbose")
fi
if [[ "$QUIET" == "true" ]]; then
    CARGO_FLAGS+=("--quiet")
fi
if [[ "$NO_CACHE" == "true" ]]; then
    export CARGO_TARGET_DIR="$(mktemp -d)"
    trap 'rm -rf "$CARGO_TARGET_DIR"' EXIT
fi

# Track failures for non-fail-fast mode
FAILURES=()

# Run a command and track failures
run_check() {
    local name="$1"
    shift
    
    log_step "Running: $name"
    
    if "$@"; then
        log_success "$name passed"
        return 0
    else
        log_error "$name failed"
        FAILURES+=("$name")
        if [[ "$FAIL_FAST" == "true" ]]; then
            exit 1
        fi
        return 1
    fi
}

# Check formatting
check_fmt() {
    run_check "Formatting" cargo fmt --all -- --check
}

# Run clippy
check_clippy() {
    run_check "Clippy" cargo clippy --all-targets --all-features -- -D warnings
}

# Run tests
run_tests() {
    local test_type="${1:-all}"
    
    case "$test_type" in
        unit)
            run_check "Unit tests" cargo test --lib ${CARGO_FLAGS[@]+"${CARGO_FLAGS[@]}"}
            ;;
        integration)
            run_check "Integration tests" cargo test --test '*' ${CARGO_FLAGS[@]+"${CARGO_FLAGS[@]}"}
            ;;
        e2e)
            # Skip E2E tests when running in CI environment
            if [[ "${CI:-}" == "true" ]]; then
                log_info "E2E tests are skipped in CI (run locally in pre-push hooks)"
            else
                run_check "E2E tests" bash -c "cd cli && cargo test --test '*' --release"
            fi
            ;;
        doc)
            run_check "Doc tests" cargo test --doc ${CARGO_FLAGS[@]+"${CARGO_FLAGS[@]}"}
            ;;
        all)
            # Run tests in parallel
            log_step "Running all tests (parallel)"
            
            local pids=()
            
            # Start parallel test runs
            cargo test --lib ${CARGO_FLAGS[@]+"${CARGO_FLAGS[@]}"} &
            pids+=($!)
            
            cargo test --test '*' ${CARGO_FLAGS[@]+"${CARGO_FLAGS[@]}"} &
            pids+=($!)
            
            cargo test --doc ${CARGO_FLAGS[@]+"${CARGO_FLAGS[@]}"} &
            pids+=($!)
            
            # Skip E2E tests when running in CI environment
            if [[ "${CI:-}" != "true" ]]; then
                (cd cli && cargo test --test '*' --release) &
                pids+=($!)
            fi
            
            # Wait for all tests to complete
            local failed=false
            for pid in "${pids[@]}"; do
                if ! wait "$pid"; then
                    failed=true
                fi
            done
            
            if [[ "$failed" == "true" ]]; then
                log_error "Some tests failed"
                FAILURES+=("Tests")
                if [[ "$FAIL_FAST" == "true" ]]; then
                    exit 1
                fi
            else
                log_success "All tests passed"
            fi
            ;;
    esac
}

# Run tests with coverage
run_coverage() {
    if ! command -v cargo-llvm-cov &> /dev/null; then
        log_warning "cargo-llvm-cov not installed, installing..."
        cargo install cargo-llvm-cov
    fi
    
    run_check "Coverage" cargo llvm-cov --all-features --workspace \
        --lcov --output-path lcov.info \
        --ignore-filename-regex '(tests?/|examples/|target/)'
    
    # Generate and display report
    cargo llvm-cov report --summary-only
}

# Build documentation
build_docs() {
    run_check "Documentation" env RUSTDOCFLAGS="-D warnings" \
        cargo doc --no-deps --all-features ${CARGO_FLAGS[@]+"${CARGO_FLAGS[@]}"}
}

# Build targets
build_all() {
    run_check "Debug build" cargo build --all-features ${CARGO_FLAGS[@]+"${CARGO_FLAGS[@]}"}
    run_check "Release build" cargo build --release --all-features ${CARGO_FLAGS[@]+"${CARGO_FLAGS[@]}"}
}

# Build WASM target
build_wasm() {
    run_check "WASM build" cargo build --target wasm32-unknown-unknown --release
}

# Check version consistency
check_versions() {
    if [[ -x "$SCRIPT_DIR/check-versions.sh" ]]; then
        run_check "Version consistency" "$SCRIPT_DIR/check-versions.sh"
    else
        log_warning "Version check script not found"
    fi
}

# Clean build artifacts
clean_artifacts() {
    log_step "Cleaning build artifacts"
    cargo clean
    rm -rf target/
    log_success "Clean complete"
}

# Main execution
main() {
    local start_time=$(date +%s)
    
    log_info "Starting CI checks with $PARALLEL_JOBS parallel jobs"
    log_info "Commands: ${COMMANDS[*]}"
    
    for cmd in "${COMMANDS[@]}"; do
        case "$cmd" in
            all)
                check_fmt
                check_clippy
                check_versions
                if [[ "$COVERAGE" == "true" ]]; then
                    run_coverage
                else
                    run_tests all
                fi
                build_docs
                build_all
                build_wasm
                ;;
            fmt|format)
                check_fmt
                ;;
            clippy)
                check_clippy
                ;;
            test)
                if [[ "$COVERAGE" == "true" ]]; then
                    run_coverage
                else
                    run_tests all
                fi
                ;;
            test-unit)
                run_tests unit
                ;;
            test-integration)
                run_tests integration
                ;;
            test-e2e)
                # E2E tests should be run locally, not in CI
                if [[ "${CI:-}" == "true" ]]; then
                    log_info "E2E tests are skipped in CI (run locally in pre-push hooks)"
                else
                    run_tests e2e
                fi
                ;;
            doc)
                build_docs
                ;;
            build)
                build_all
                ;;
            build-wasm)
                build_wasm
                ;;
            version)
                check_versions
                ;;
            clean)
                clean_artifacts
                ;;
            *)
                log_error "Unknown command: $cmd"
                show_help
                exit 1
                ;;
        esac
    done
    
    # Calculate elapsed time
    local end_time=$(date +%s)
    local elapsed=$((end_time - start_time))
    local minutes=$((elapsed / 60))
    local seconds=$((elapsed % 60))
    
    # Final status
    echo
    if [[ ${#FAILURES[@]} -eq 0 ]]; then
        log_success "All checks passed! (${minutes}m ${seconds}s)"
        exit 0
    else
        log_error "Failed checks: ${FAILURES[*]} (${minutes}m ${seconds}s)"
        exit 1
    fi
}

# Run main function
main