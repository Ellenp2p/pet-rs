#!/usr/bin/env bash
set -euo pipefail
echo "Running cargo fmt..."
cargo fmt
echo "Building release..."
cargo build --release
echo "Running tests..."
cargo test --all --verbose
