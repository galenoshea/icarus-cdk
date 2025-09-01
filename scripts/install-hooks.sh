#!/bin/bash

# Install Git hooks for the Icarus SDK project
# This script sets up pre-commit hooks to ensure code quality

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
HOOKS_DIR="$PROJECT_ROOT/.git/hooks"

echo "Installing Git hooks for Icarus SDK..."

# Check if hooks already exist
if [ -f "$HOOKS_DIR/pre-commit" ]; then
    echo "⚠️  pre-commit hook already exists. Backing up to pre-commit.bak"
    cp "$HOOKS_DIR/pre-commit" "$HOOKS_DIR/pre-commit.bak"
fi

if [ -f "$HOOKS_DIR/pre-push" ]; then
    echo "⚠️  pre-push hook already exists. Backing up to pre-push.bak"
    cp "$HOOKS_DIR/pre-push" "$HOOKS_DIR/pre-push.bak"
fi

# Create pre-commit hook
cat > "$HOOKS_DIR/pre-commit" << 'EOF'
#!/bin/bash

# Pre-commit hook for Icarus SDK
# Ensures code quality before committing

set -e

echo "Running pre-commit checks..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print step
print_step() {
    echo -e "${YELLOW}▶ $1${NC}"
}

# Function to print success
print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

# Function to print error
print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check formatting
print_step "Checking formatting..."
if cargo fmt -- --check > /dev/null 2>&1; then
    print_success "Formatting OK"
else
    print_error "Formatting issues found. Run 'cargo fmt' to fix."
    exit 1
fi

# Run clippy
print_step "Running clippy..."
if cargo clippy --quiet -- -D warnings 2>&1 | grep -q "error:"; then
    print_error "Clippy found issues. Fix them before committing."
    cargo clippy -- -D warnings
    exit 1
else
    print_success "Clippy OK"
fi

# Run tests
print_step "Running tests..."
if cargo test --quiet > /dev/null 2>&1; then
    print_success "Tests passed"
else
    print_error "Tests failed. Fix them before committing."
    cargo test
    exit 1
fi

echo -e "${GREEN}All pre-commit checks passed!${NC}"
EOF

# Make the hook executable
chmod +x "$HOOKS_DIR/pre-commit"

# Create pre-push hook
cat > "$HOOKS_DIR/pre-push" << 'EOF'
#!/bin/bash

# Pre-push hook for Icarus SDK
# Runs comprehensive tests before pushing

set -e

echo "Running pre-push checks..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

print_step() {
    echo -e "${YELLOW}▶ $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Run the full CI test suite
print_step "Running CI test suite..."
if ./scripts/test-ci.sh > /dev/null 2>&1; then
    print_success "CI tests passed"
else
    print_error "CI tests failed. Run './scripts/test-ci.sh' for details."
    exit 1
fi

echo -e "${GREEN}Ready to push!${NC}"
EOF

# Make the hook executable
chmod +x "$HOOKS_DIR/pre-push"

echo "✅ Git hooks installed successfully!"
echo ""
echo "Hooks installed:"
echo "  • pre-commit: Runs formatting, clippy, and tests"
echo "  • pre-push: Runs full CI test suite"
echo ""
echo "To skip hooks temporarily, use:"
echo "  git commit --no-verify"
echo "  git push --no-verify"
echo ""
echo "To uninstall hooks:"
echo "  rm .git/hooks/pre-commit .git/hooks/pre-push"