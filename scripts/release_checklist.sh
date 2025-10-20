#!/bin/bash
# Pre-release checklist for fastalloc

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "=== fastalloc Release Checklist ==="
echo

# Function to run check
check() {
    local name="$1"
    local cmd="$2"
    
    echo -n "Checking $name... "
    if eval "$cmd" > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
        return 0
    else
        echo -e "${RED}✗${NC}"
        return 1
    fi
}

failed=0

# Format check
if ! check "formatting" "cargo fmt -- --check"; then
    echo "  Run: cargo fmt"
    ((failed++))
fi

# Clippy check
if ! check "clippy" "cargo clippy --all-features -- -D warnings"; then
    echo "  Run: cargo clippy --all-features --fix"
    ((failed++))
fi

# Test check
if ! check "tests" "cargo test --all-features"; then
    echo "  Run: cargo test --all-features"
    ((failed++))
fi

# Doc test check
if ! check "doc tests" "cargo test --doc --all-features"; then
    echo "  Run: cargo test --doc"
    ((failed++))
fi

# Build check
if ! check "build" "cargo build --release --all-features"; then
    echo "  Run: cargo build --release"
    ((failed++))
fi

# Package check
if ! check "package" "cargo package --allow-dirty"; then
    echo "  Fix packaging errors"
    ((failed++))
fi

# Documentation build
if ! check "documentation" "cargo doc --all-features --no-deps"; then
    echo "  Fix documentation errors"
    ((failed++))
fi

echo
echo "=== Manual Checks ==="
echo "[ ] Updated version in Cargo.toml"
echo "[ ] Updated CHANGELOG.md"
echo "[ ] All examples run successfully"
echo "[ ] README.md is up to date"
echo "[ ] Documentation is complete"
echo "[ ] Benchmarks run successfully"
echo "[ ] Git tag created (vX.Y.Z)"

echo
if [ $failed -eq 0 ]; then
    echo -e "${GREEN}All automated checks passed!${NC}"
    echo "Complete manual checks and then:"
    echo "  git tag v<version>"
    echo "  git push origin v<version>"
else
    echo -e "${RED}$failed check(s) failed${NC}"
    echo "Fix issues before releasing"
    exit 1
fi
