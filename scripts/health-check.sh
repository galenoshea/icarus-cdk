#!/bin/bash
# Comprehensive health check for Icarus SDK project

set -euo pipefail

# Configuration
readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors
if [[ -t 1 ]]; then
    readonly RED='\033[0;31m'
    readonly GREEN='\033[0;32m'
    readonly YELLOW='\033[1;33m'
    readonly BLUE='\033[0;34m'
    readonly CYAN='\033[0;36m'
    readonly NC='\033[0m'
else
    readonly RED='' GREEN='' YELLOW='' BLUE='' CYAN='' NC=''
fi

# Score tracking
TOTAL_CHECKS=0
PASSED_CHECKS=0
WARNINGS=0
ERRORS=0

# Check function
check() {
    local name="$1"
    local cmd="$2"
    local severity="${3:-error}"  # error or warning
    
    ((TOTAL_CHECKS++))
    
    echo -n "  Checking $name... "
    
    if eval "$cmd" &>/dev/null; then
        echo -e "${GREEN}✓${NC}"
        ((PASSED_CHECKS++))
        return 0
    else
        if [[ "$severity" == "warning" ]]; then
            echo -e "${YELLOW}⚠${NC}"
            ((WARNINGS++))
            ((PASSED_CHECKS++))
        else
            echo -e "${RED}✗${NC}"
            ((ERRORS++))
        fi
        return 1
    fi
}

# Header
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${CYAN}   Icarus SDK Project Health Check${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo

cd "$PROJECT_ROOT"

# 1. Environment Checks
echo -e "${BLUE}▶ Environment${NC}"
check "Rust installed" "command -v rustc"
check "Cargo installed" "command -v cargo"
check "Git installed" "command -v git"
check "wasm32 target" "rustup target list --installed | grep -q wasm32-unknown-unknown" "warning"
check "cargo-llvm-cov" "command -v cargo-llvm-cov" "warning"
check "wasm-opt" "command -v wasm-opt" "warning"
echo

# 2. Project Structure
echo -e "${BLUE}▶ Project Structure${NC}"
check "Cargo.toml exists" "[[ -f Cargo.toml ]]"
check "Workspace configured" "grep -q '\[workspace\]' Cargo.toml"
check "README exists" "[[ -f README.md ]]"
check "LICENSE exists" "[[ -f LICENSE ]]"
check "CHANGELOG exists" "[[ -f CHANGELOG.md ]]"
check "Scripts directory" "[[ -d scripts ]]"
check "GitHub workflows" "[[ -d .github/workflows ]]"
check "Documentation" "[[ -d docs ]]"
echo

# 3. Version Consistency
echo -e "${BLUE}▶ Version Management${NC}"
check "Version consistency" "$SCRIPT_DIR/check-versions.sh"
check "Workspace versions aligned" "! grep -r 'version = ' crates/*/Cargo.toml | grep -v workspace"
echo

# 4. Code Quality
echo -e "${BLUE}▶ Code Quality${NC}"
check "Format check" "cargo fmt --all -- --check"
check "Clippy warnings" "cargo clippy --quiet -- -D warnings 2>/dev/null"
check "No TODO/FIXME" "! grep -r 'TODO\\|FIXME' --include='*.rs' src/ crates/" "warning"
echo

# 5. Build Health
echo -e "${BLUE}▶ Build System${NC}"
check "Debug build" "cargo build --quiet 2>/dev/null"
check "Release build" "cargo build --release --quiet 2>/dev/null"
check "WASM build" "cargo build --target wasm32-unknown-unknown --quiet 2>/dev/null"
check "Documentation builds" "cargo doc --no-deps --quiet 2>/dev/null"
echo

# 6. Test Coverage
echo -e "${BLUE}▶ Testing${NC}"
check "Unit tests pass" "cargo test --lib --quiet 2>/dev/null"
check "Doc tests pass" "cargo test --doc --quiet 2>/dev/null"
check "Integration tests" "cargo test --test '*' --quiet 2>/dev/null" "warning"
check "E2E tests configured" "[[ -d cli/tests/e2e ]]"
echo

# 7. Dependencies
echo -e "${BLUE}▶ Dependencies${NC}"
check "No security vulnerabilities" "! cargo audit 2>/dev/null | grep -q vulnerable" "warning"
check "Dependencies up-to-date" "cargo outdated --exit-code 1 2>/dev/null" "warning"
check "Lockfile committed" "[[ -f Cargo.lock ]]"
echo

# 8. Git Repository
echo -e "${BLUE}▶ Repository Health${NC}"
check "Clean working tree" "git diff --quiet HEAD" "warning"
check "No untracked files" "[[ -z \$(git ls-files --others --exclude-standard) ]]" "warning"
check "Remote configured" "git remote get-url origin"
check "Main branch" "git rev-parse --abbrev-ref HEAD | grep -q main"
echo

# 9. CI/CD Configuration
echo -e "${BLUE}▶ CI/CD Pipeline${NC}"
check "CI workflow exists" "[[ -f .github/workflows/ci.yml || -f .github/workflows/ci-optimized.yml ]]"
check "Release workflow" "[[ -f .github/workflows/release.yml || -f .github/workflows/release-optimized.yml ]]"
check "Pre-commit hook" "[[ -f .git/hooks/pre-commit ]]" "warning"
check "Pre-push hook" "[[ -f .git/hooks/pre-push ]]" "warning"
echo

# 10. Performance Metrics
echo -e "${BLUE}▶ Performance Metrics${NC}"
echo -n "  Measuring build time... "
BUILD_START=$(date +%s%N)
cargo build --quiet 2>/dev/null
BUILD_END=$(date +%s%N)
BUILD_TIME=$(( (BUILD_END - BUILD_START) / 1000000 ))
if [[ $BUILD_TIME -lt 10000 ]]; then
    echo -e "${GREEN}${BUILD_TIME}ms ✓${NC}"
    ((PASSED_CHECKS++))
else
    echo -e "${YELLOW}${BUILD_TIME}ms ⚠${NC}"
    ((WARNINGS++))
fi
((TOTAL_CHECKS++))

echo -n "  Counting lines of code... "
LOC=$(find . -name "*.rs" -not -path "./target/*" | xargs wc -l | tail -1 | awk '{print $1}')
echo -e "${CYAN}${LOC} lines${NC}"
echo

# Calculate score
SCORE=$(( (PASSED_CHECKS * 100) / TOTAL_CHECKS ))
GRADE="F"
if [[ $SCORE -ge 95 ]]; then
    GRADE="A+"
elif [[ $SCORE -ge 90 ]]; then
    GRADE="A"
elif [[ $SCORE -ge 85 ]]; then
    GRADE="B+"
elif [[ $SCORE -ge 80 ]]; then
    GRADE="B"
elif [[ $SCORE -ge 75 ]]; then
    GRADE="C+"
elif [[ $SCORE -ge 70 ]]; then
    GRADE="C"
elif [[ $SCORE -ge 65 ]]; then
    GRADE="D+"
elif [[ $SCORE -ge 60 ]]; then
    GRADE="D"
fi

# Summary
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${CYAN}   Health Check Summary${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo
echo "  Total Checks:  $TOTAL_CHECKS"
echo -e "  Passed:        ${GREEN}$PASSED_CHECKS${NC}"
echo -e "  Warnings:      ${YELLOW}$WARNINGS${NC}"
echo -e "  Errors:        ${RED}$ERRORS${NC}"
echo
echo -n "  Health Score:  "
if [[ $SCORE -ge 90 ]]; then
    echo -e "${GREEN}${SCORE}% (${GRADE})${NC}"
elif [[ $SCORE -ge 75 ]]; then
    echo -e "${YELLOW}${SCORE}% (${GRADE})${NC}"
else
    echo -e "${RED}${SCORE}% (${GRADE})${NC}"
fi
echo

# Recommendations
if [[ $ERRORS -gt 0 || $WARNINGS -gt 0 ]]; then
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${CYAN}   Recommendations${NC}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo
    
    if ! command -v cargo-llvm-cov &>/dev/null; then
        echo "  • Install cargo-llvm-cov for coverage: cargo install cargo-llvm-cov"
    fi
    
    if ! command -v wasm-opt &>/dev/null; then
        echo "  • Install wasm-opt for optimization: npm install -g wasm-opt"
    fi
    
    if ! command -v cargo-audit &>/dev/null; then
        echo "  • Install cargo-audit for security: cargo install cargo-audit"
    fi
    
    if [[ ! -f .git/hooks/pre-commit ]]; then
        echo "  • Set up pre-commit hook: cp scripts/hooks/pre-commit .git/hooks/"
    fi
    
    if git diff --quiet HEAD 2>/dev/null; then
        :
    else
        echo "  • Commit or stash your changes"
    fi
    
    echo
fi

# Exit with appropriate code
if [[ $ERRORS -gt 0 ]]; then
    exit 1
else
    exit 0
fi