#!/bin/bash

# Local CI-equivalent checks
# Run before pushing to match GitHub Actions lint.yml

set -e

# Options
MEMORY_CHECKS=false
QUICK=false

# Parse command-line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --memory-checks)
            MEMORY_CHECKS=true
            shift
            ;;
        --quick)
            QUICK=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done


echo "=== Rustfmt check ==="
cargo fmt --all -- --check

echo ""
echo "=== Clippy check (mrrc core) ==="
cargo clippy --package mrrc --all-targets -- -D warnings

echo ""
echo "=== Clippy check (mrrc-python) ==="
cargo clippy --package mrrc-python --all-targets -- -D warnings

if [ "$QUICK" = false ]; then
    echo ""
    echo "=== Documentation check ==="
    RUSTDOCFLAGS="-D warnings" cargo doc --all --no-deps --document-private-items

    echo ""
    echo "=== Security audit ==="
    cargo audit

    echo ""
    echo "=== Maturin Python extension build ==="
    uv run maturin develop
fi

echo ""
echo "=== Rust library + integration tests ==="
cargo test --lib --tests --package mrrc -q

echo ""
echo "=== Rust doc tests ==="
cargo test --doc --package mrrc -q

echo ""
echo "=== Python tests (core functionality, excludes benchmarks) ==="
uv run python -m pytest tests/python/ -m "not benchmark" -q

echo ""
echo "=== Python lint (ruff) ==="
uv run ruff check mrrc/ tests/python/

# ASAN memory safety checks (optional, nightly feature)
if [ "$MEMORY_CHECKS" = true ]; then
    echo ""
    echo "=== ASAN memory safety checks ==="
    
    # Check if rustup is available (required for nightly)
    if ! command -v rustup &> /dev/null; then
        echo "Error: ASAN memory checks require Rust nightly toolchain"
        echo ""
        echo "Currently using Homebrew Rust (stable only)."
        echo "To use ASAN, install Rust via rustup:"
        echo ""
        echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        echo ""
        echo "Then install nightly:"
        echo "  rustup install nightly"
        echo ""
        exit 1
    fi
    
    # Verify nightly is installed
    if ! rustup toolchain list | grep -q "nightly"; then
        echo "Error: Rust nightly toolchain not found"
        echo "Install it with: rustup install nightly"
        exit 1
    fi
    
    export RUSTFLAGS="-Z sanitizer=address"
    export RUSTDOCFLAGS="${RUSTFLAGS}"
    export LSAN_OPTIONS="suppressions=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/asan_suppressions.txt"
    
    # Run ASAN on library tests using nightly toolchain
    cargo +nightly test --lib --package mrrc -q
    
    # Clear ASAN flags after tests
    unset RUSTFLAGS RUSTDOCFLAGS LSAN_OPTIONS
fi

echo ""
echo "✓ All checks passed"
