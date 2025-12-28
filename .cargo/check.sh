#!/bin/bash

# Local CI-equivalent checks
# Run before pushing to match GitHub Actions lint.yml

set -e

echo "=== Rustfmt check ==="
cargo fmt --all -- --check

echo ""
echo "=== Clippy check ==="
cargo clippy --package mrrc --all-targets -- -D warnings

echo ""
echo "=== Documentation check ==="
RUSTDOCFLAGS="-D warnings" cargo doc --package mrrc --no-deps --document-private-items

echo ""
echo "✓ All checks passed"
