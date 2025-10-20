#!/bin/bash
# Run all tests for fastalloc

set -e

echo "=== Running fastalloc test suite ==="
echo

echo "1. Unit and integration tests..."
cargo test --all-features
echo "✓ Tests passed"
echo

echo "2. Doc tests..."
cargo test --doc --all-features
echo "✓ Doc tests passed"
echo

echo "3. Tests without default features..."
cargo test --no-default-features
echo "✓ no_std tests passed"
echo

echo "4. Feature-specific tests..."
cargo test --features stats
cargo test --features serde
cargo test --features parking_lot
echo "✓ Feature tests passed"
echo

echo "=== All tests passed! ==="
