#!/bin/bash
# Run benchmarks for fastalloc

set -e

echo "=== Running fastalloc benchmarks ==="
echo

echo "Building in release mode..."
cargo build --release --all-features
echo

echo "Running benchmarks..."
cargo bench --all-features

echo
echo "=== Benchmarks complete ==="
echo "Results saved to target/criterion/"
