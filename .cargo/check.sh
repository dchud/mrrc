#!/bin/bash

# Local CI-equivalent checks
# Run before pushing to match GitHub Actions lint.yml

set -e

# Activate Python virtual environment for maturin builds
if [ -f "venv/bin/activate" ]; then
    source venv/bin/activate
fi

echo "=== Rustfmt check ==="
cargo fmt --all -- --check

echo ""
echo "=== Clippy check ==="
cargo clippy --package mrrc --all-targets -- -D warnings

echo ""
echo "=== Documentation check ==="
RUSTDOCFLAGS="-D warnings" cargo doc --all --no-deps --document-private-items

echo ""
echo "=== Security audit ==="
cargo audit

echo ""
echo "=== Maturin Python extension build ==="
maturin develop

echo ""
echo "=== Python tests ==="
python -m pytest tests/python/test_unit_basic.py tests/python/test_pymarc_compatibility.py -q

echo ""
echo "✓ All checks passed"
