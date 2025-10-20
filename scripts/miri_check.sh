#!/bin/bash
# Run miri to check for undefined behavior

set -e

echo "=== Running Miri checks ==="
echo

if ! command -v rustup &> /dev/null; then
    echo "Error: rustup is required"
    exit 1
fi

echo "Installing nightly toolchain..."
rustup toolchain install nightly
echo

echo "Installing miri..."
rustup +nightly component add miri
echo

echo "Setting up miri..."
cargo +nightly miri setup
echo

echo "Running miri tests..."
cargo +nightly miri test --tests

echo
echo "=== Miri checks complete ==="
