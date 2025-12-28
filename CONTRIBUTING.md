# Contributing to MRRC

Welcome to the MRRC project! We are building a high-performance, Rust-native library for working with MARC bibliographic records.

## Development Workflow

### Issue Tracking (Beads)

This project strictly uses **[beads](https://github.com/dchud/beads)** for issue tracking. We do not use GitHub Issues or other external trackers for development tasks.

*   **Check for work**: `bd ready`
*   **Start a task**: `bd update <id> --status in_progress`
*   **Close a task**: `bd close <id> --reason "Completed"`

### Prerequisites

*   Rust (1.71+)
*   Type-level generic knowledge (for builder patterns)

### Local CI Checks

Before pushing any code, you **MUST** run the local CI script. This ensures your code passes all quality gates that match our GitHub Actions environment.

```bash
./.cargo/check.sh
```

This runs:
1.  **Format**: `cargo fmt`
2.  **Lint**: `cargo clippy` (strict settings)
3.  **Test**: `cargo test`
4.  **Doc**: `cargo doc`

### Coding Standards

*   **Formatting**: We use `rustfmt` with custom settings in `rustfmt.toml`.
*   **Linting**: We adhere to strict `clippy` settings defined in `clippy.toml`.
*   **Error Handling**: No `unwrap()` in library code. Use `Result` and `?`.
*   **Documentation**: All public APIs must have doc comments and examples.

## Project Structure

*   `src/`: Library source code
*   `tests/`: Integration tests and test data
*   `scripts/`: Helper scripts (e.g., table generation)
*   `.beads/`: Local issue database (do not edit manually)

## Landing the Plane (Session Completion)

When finishing a work session:

1.  **Run CI**: `./.cargo/check.sh`
2.  **Sync Issues**: `bd sync`
3.  **Push**: `git push`

**Note**: Work is not complete until it is pushed and strictly passes all CI checks.
