#!/bin/bash

echo "Installing git hooks..."

# Create hooks directory if it doesn't exist
mkdir -p .git/hooks

# Create the pre-commit hook
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "🚀 Running pre-commit checks..."

# Check if we have any Rust files staged
if ! git diff --cached --name-only | grep -q "\.rs$"; then
    echo "📋 No Rust files staged, skipping Rust checks"
    exit 0
fi

# Save current state
STASH_NAME="pre-commit-$(date +%s)"
git stash push -q --keep-index -m "$STASH_NAME"

# Function to restore state on exit
cleanup() {
    # Restore the stash if it exists
    if git stash list | grep -q "$STASH_NAME"; then
        git stash pop -q
    fi
}
trap cleanup EXIT

# Run cargo fmt
echo -e "${YELLOW}📝 Running cargo fmt...${NC}"
if ! cargo fmt; then
    echo -e "${RED}❌ Formatting failed${NC}"
    exit 1
fi

# Check if formatting changed any files
if ! git diff --exit-code > /dev/null; then
    echo -e "${GREEN}✨ Files were formatted. Adding changes...${NC}"
    git add -u
fi

# Run cargo clippy with autofix
echo -e "${YELLOW}🔧 Running cargo clippy with fixes...${NC}"
if ! cargo clippy --fix --allow-dirty --allow-staged -- -D warnings 2>/dev/null; then
    # If autofix failed, try without autofix to get better error messages
    echo -e "${RED}❌ Clippy found issues that couldn't be auto-fixed:${NC}"
    cargo clippy -- -D warnings
    exit 1
fi

# Check if clippy fixed any files
if ! git diff --exit-code > /dev/null; then
    echo -e "${GREEN}✨ Clippy fixed some issues. Adding changes...${NC}"
    git add -u
fi

# Run final checks to ensure everything passes
echo -e "${YELLOW}✅ Running final checks...${NC}"

# Check formatting
if ! cargo fmt -- --check; then
    echo -e "${RED}❌ Formatting check failed${NC}"
    exit 1
fi

# Check clippy
if ! cargo clippy -- -D warnings; then
    echo -e "${RED}❌ Clippy check failed${NC}"
    exit 1
fi

# Optional: Run tests (uncomment if you want tests to run on every commit)
# echo -e "${YELLOW}🧪 Running tests...${NC}"
# if ! cargo test --quiet; then
#     echo -e "${RED}❌ Tests failed${NC}"
#     exit 1
# fi

echo -e "${GREEN}✨ All pre-commit checks passed!${NC}"
EOF

# Make the hook executable
chmod +x .git/hooks/pre-commit

echo "✅ Git hooks installed successfully!"
echo ""
echo "The pre-commit hook will:"
echo "  - Format code with cargo fmt"
echo "  - Fix linting issues with cargo clippy --fix"
echo "  - Ensure all checks pass before allowing commit"
echo ""
echo "To skip the hook for a single commit, use: git commit --no-verify"