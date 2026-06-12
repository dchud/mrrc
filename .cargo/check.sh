#!/bin/bash

# Local CI-equivalent checks
# Run before pushing to match GitHub Actions lint.yml

set -e

# Options
MEMORY_CHECKS=false
QUICK=false
RELEASE=false

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
        --release)
            RELEASE=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Match python-build.yml: allow building the extension against Python versions
# newer than pyo3's explicit support (inert on already-supported versions).
export PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1


echo "=== Rustfmt check ==="
cargo fmt --all -- --check

echo ""
echo "=== CHANGELOG lint ==="
bash scripts/lint-changelog.sh

echo ""
echo "=== Process-label lint ==="
python3 scripts/lint_process_labels.py

echo ""
echo "=== Clippy check (mrrc core) ==="
cargo clippy --package mrrc --all-targets -- -D warnings

echo ""
echo "=== Clippy check (mrrc-python) ==="
cargo clippy --package mrrc-python --all-targets -- -D warnings

echo ""
echo "=== Python lint (ruff) ==="
uv run ruff check mrrc/ tests/python/

echo ""
echo "=== Rust library + integration tests ==="
cargo test --lib --tests --package mrrc -q

echo ""
echo "=== Rust doc tests ==="
cargo test --doc --package mrrc -q

if [ "$QUICK" = false ]; then
    echo ""
    echo "=== Compile examples ==="
    cargo build --examples --quiet

    echo ""
    echo "=== Type-check benchmarks ==="
    # cargo check skips codegen and linking: the bench profile inherits
    # fat LTO + codegen-units=1, so `cargo bench --no-run` does a serial
    # whole-program LTO link per bench target (minutes); type-checking
    # catches the same compile errors in seconds. CodSpeed CI does the
    # real LTO build.
    cargo check --benches --quiet

    echo ""
    echo "=== Documentation check ==="
    RUSTDOCFLAGS="-D warnings" cargo doc --all --no-deps --document-private-items

    echo ""
    echo "=== Security audit ==="
    cargo audit

    echo ""
    echo "=== Unused dependency check ==="
    # Fix with: cargo install cargo-machete --locked
    command -v cargo-machete &> /dev/null \
        || { echo "ERROR: cargo-machete missing — run 'cargo install cargo-machete --locked'"; exit 1; }
    cargo machete

    echo ""
    if [ "$RELEASE" = true ]; then
        echo "=== Maturin Python extension build (--release) ==="
        # Release-mode codegen (inlining, optimizations) matches the wheels
        # CI builds; use this to reproduce perf-sensitive regressions locally.
        uv run maturin develop --release
    else
        echo "=== Maturin Python extension build ==="
        uv run maturin develop
    fi

    echo ""
    echo "=== Python tests (core functionality, excludes benchmarks) ==="
    # The pymarc parity oracle skips silently when pymarc is missing;
    # fail here instead. pymarc comes from the oracle extra — fix with:
    #   uv sync --all-extras
    uv run python -c "import pymarc" 2>/dev/null \
        || { echo "ERROR: pymarc (oracle extra) missing — run 'uv sync --all-extras'"; exit 1; }
    uv run python -m pytest tests/python/ -m "not benchmark" -q

    echo ""
    echo "=== Error-code source-of-truth reconciliation ==="
    # Asserts every MarcError code is documented in error-codes.md, has a
    # wired case in error_coverage.toml, and is constructed in production
    # code — catching drift the error_coverage harness above can't (a new
    # code with no manifest case). See scripts/verify_error_docs.py.
    uv run python scripts/verify_error_docs.py

    echo ""
    echo "=== Python type check (mypy on mrrc/) ==="
    uv run mypy mrrc/

    echo ""
    echo "=== Python type check (pyright on mrrc/) ==="
    uv run pyright mrrc/

    echo ""
    echo "=== Stub verification (stubtest: _mrrc.pyi vs compiled extension) ==="
    # Diffs every name/signature in mrrc/_mrrc.pyi against the compiled
    # _mrrc extension, failing on any drift. Requires the maturin build above.
    uv run python -m mypy.stubtest mrrc._mrrc

    echo ""
    echo "=== Documentation site build (mkdocs --strict) ==="
    # Needs the `docs` extra: run `uv sync --all-extras` if mkdocs is missing.
    # --strict turns build warnings (broken links, unresolved mkdocstrings
    # references) into errors, matching the CI docs gate.
    uv run mkdocs build --strict
fi

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
