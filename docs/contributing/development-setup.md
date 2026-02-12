# Development Setup

This guide covers setting up a local development environment for MRRC.

## Prerequisites

- **Rust** 1.71+ (see `Cargo.toml` for minimum version)
- **Python** 3.10+
- **maturin** 1.0+ (for building Python bindings)
- **uv** (recommended) or pip for Python package management

## Quick Setup

```bash
# Clone the repository
git clone https://github.com/dchud/mrrc.git
cd mrrc

# Install dependencies and create virtual environment
uv sync

# Build and install the Python package in development mode
uv run maturin develop

# Verify installation
uv run python -c "import mrrc; print('MRRC installed successfully')"
```

## Building

### Rust Library

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Build documentation
cargo doc --open
```

### Python Bindings

```bash
# Development build (debug, fast iteration)
uv run maturin develop

# Release build (optimized)
uv run maturin develop --release

# Build wheels for distribution
uv run maturin build --release
```

## Running Tests

See [Testing](testing.md) for comprehensive test documentation.

```bash
# Full pre-push checks (fmt, clippy, docs, audit, build, tests, ruff)
.cargo/check.sh

# Quick checks (fmt, clippy, tests, ruff — skips docs, audit, maturin build)
.cargo/check.sh --quick

# Rust tests only
cargo test

# Python tests only
uv run python -m pytest tests/python/ -m "not benchmark"
```

## Git Hooks

The repository includes optional git hooks in `.githooks/` that enforce
local checks before pushing. To enable them:

```bash
git config core.hooksPath .githooks
```

This activates a **pre-push hook** that runs `.cargo/check.sh` before
every push, preventing code from reaching CI that would fail basic
formatting, linting, or test checks. The push is blocked if any check
fails (bypass with `git push --no-verify` if needed).

This setting is local to your clone and does not affect other
contributors.

## IDE Setup

### VS Code

Recommended extensions:
- **rust-analyzer** - Rust language support
- **Python** - Python language support
- **Even Better TOML** - TOML file support

### PyCharm / IntelliJ

- Enable Rust plugin
- Configure Python interpreter to use virtual environment
- Set `mrrc/` as a source root for Python

## Project Structure

```
mrrc/
├── src/                    # Rust source code
├── src-python/             # Python bindings (PyO3)
├── mrrc/                   # Python package (installed)
├── tests/                  # Test suites
│   ├── python/            # Python tests
│   └── data/              # Test fixtures
├── examples/               # Example code
├── docs/                   # Documentation
├── benches/                # Rust benchmarks
└── .cargo/                 # Cargo configuration and scripts
```

## Common Tasks

### Adding a New Feature

1. Implement in Rust (`src/`)
2. Add Python bindings (`src-python/src/`)
3. Write tests (Rust unit tests + Python integration tests)
4. Update documentation
5. Run `.cargo/check.sh` before committing

### Updating Python Bindings

1. Modify `src-python/src/*.rs`
2. Run `uv run maturin develop` to rebuild
3. Test with `uv run python -m pytest tests/python/`

## Troubleshooting

### maturin develop fails

Ensure you have the correct Rust toolchain:
```bash
rustup show
rustup update stable
```

### Python import fails after rebuild

Try reinstalling:
```bash
uv run maturin develop --release
```

## Next Steps

- [Testing](testing.md) - Running the test suite
- [Architecture](architecture.md) - Understanding the codebase
- [Release Procedure](release-procedure.md) - How releases are made
