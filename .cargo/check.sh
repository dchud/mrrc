#!/bin/bash

# Local CI-equivalent checks
# Run before pushing to match GitHub Actions lint.yml

set -e

# Options
MEMORY_CHECKS=false

# Parse command-line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --memory-checks)
            MEMORY_CHECKS=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Activate Python virtual environment for maturin builds
if [ -f "venv/bin/activate" ]; then
    source venv/bin/activate
fi

echo "=== Rustfmt check ==="
cargo fmt --all -- --check

echo ""
echo "=== Clippy check (mrrc core) ==="
cargo clippy --package mrrc --all-targets -- -D warnings

echo ""
echo "=== Clippy check (mrrc-python) ==="
cargo clippy --package mrrc-python --all-targets -- -D warnings

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
echo "=== Rust library tests ==="
cargo test --lib --package mrrc --all-targets -q

echo ""
echo "=== Python tests ==="
python -m pytest tests/python/test_unit_basic.py tests/python/test_pymarc_compatibility.py -q

# ASAN memory safety checks (optional, nightly feature)
if [ "$MEMORY_CHECKS" = true ]; then
    echo ""
    echo "=== ASAN memory safety checks ==="
    export RUSTFLAGS="-Z sanitizer=address"
    export RUSTDOCFLAGS="${RUSTFLAGS}"
    export LSAN_OPTIONS="suppressions=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/asan_suppressions.txt"
    
    # Run ASAN on library tests
    cargo test --lib --package mrrc --all-targets -q
    
    # Clear ASAN flags after tests
    unset RUSTFLAGS RUSTDOCFLAGS LSAN_OPTIONS
fi

echo ""
echo "✓ All checks passed"
