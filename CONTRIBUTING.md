# Contributing to MRRC

Thank you for your interest in contributing to MRRC (MARC Rust Crate)! This document provides guidelines and instructions for getting involved.

## Code of Conduct

Be respectful, inclusive, and professional in all interactions. We're building a library for librarians and information professionals—let's maintain that spirit of collaboration.

## Getting Started

### Prerequisites

- **Rust 1.70+**: Install from [rustup.rs](https://rustup.rs/)
- **Git**: For version control
- **Cargo**: Comes with Rust

### Development Setup

1. **Clone the repository**:
   ```bash
   git clone https://github.com/dchud/mrrc.git
   cd mrrc
   ```

2. **Install development tools**:
   ```bash
   rustup update
   cargo install cargo-tarpaulin  # For coverage reports
   ```

3. **Build the project**:
   ```bash
   cargo build
   ```

4. **Run tests**:
   ```bash
   cargo test
   ```

5. **Check code quality**:
   ```bash
   .cargo/check.sh  # Runs rustfmt, clippy, and doc checks
   ```

## Development Workflow

### Issue Tracking

MRRC uses **Beads** (`bd`) for issue tracking. This provides dependency-aware issue management integrated with git.

**Install Beads**:
```bash
# Follow instructions at https://github.com/dchud/beads
```

**Check for ready work**:
```bash
bd ready --json
```

**Create a new issue**:
```bash
bd create "Issue title" -t feature -p 2 --json
```

**Claim an issue**:
```bash
bd update <issue-id> --status in_progress --json
```

**Close an issue**:
```bash
bd close <issue-id> --reason "Description of what was completed" --json
```

### Making Changes

1. **Create a branch** (optional but recommended):
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** following Rust conventions:
   - Use `cargo fmt` to format code
   - Run clippy: `cargo clippy --all --all-targets -- -D warnings`
   - Add doc comments for public items
   - Write tests for new functionality

3. **Write tests**:
   - Add unit tests in the same file using `#[cfg(test)]` modules
   - Add integration tests in `tests/` directory
   - Run tests frequently: `cargo test`

4. **Document your changes**:
   - Add doc comments to public functions/types
   - Include examples in doc comments (marked with ` ```no_run ` or ` ``` `)
   - Update module-level documentation if needed

5. **Run quality checks**:
   ```bash
   .cargo/check.sh
   ```

   This runs:
   - `cargo fmt --all -- --check` (Rustfmt)
   - `cargo clippy --all --all-targets -- -D warnings` (Clippy)
   - `RUSTDOCFLAGS="-D warnings" cargo doc --all --no-deps --document-private-items` (Doc checks)

### Memory Safety Checks

Memory safety validation uses ASAN (Address Sanitizer) to detect potential memory issues:

**Optional memory safety testing** (requires nightly Rust):
```bash
.cargo/check.sh --memory-checks
```

This runs ASAN on library tests, detecting issues like:
- Use-after-free
- Memory leaks
- Heap buffer overflows
- Data races

**When to run memory checks:**
- After major changes to memory-critical code
- When updating dependencies
- Before submitting PRs with complex pointer/allocation changes
- As part of pre-release validation

**Interpreting ASAN output:**
- Memory issues will appear in test output with line numbers
- Suppressions for expected issues are documented in `.cargo/asan_suppressions.txt`
- See [ASAN documentation](https://github.com/google/sanitizers/wiki/AddressSanitizer) for detailed output interpretation

**For library maintainers:**
See `docs/design/MEMORY_SAFETY_CI.md` for comprehensive memory safety infrastructure details.

### Before Pushing

Always run the quality gates locally:

```bash
.cargo/check.sh
cargo test
cargo test --doc
```

For complex memory-related changes, also run:
```bash
.cargo/check.sh --memory-checks
```

Only push when all checks pass.

## Commit Messages

Write clear, descriptive commit messages:

```
Short description (50 chars or less)

Longer explanation of what changed and why, if needed.
Reference issue IDs: closes #123, related to #456
```

Example:
```
Add field linkage support for MARC 880 fields

Implement LinkageInfo struct for parsing subfield 6.
Add Record::get_linked_field() and bidirectional lookup.
Closes issue mrrc-08k: Phase 3 linked field navigation.
```

## Testing

### Unit Tests
```bash
cargo test --lib
```

### Integration Tests
```bash
cargo test --test "*"
```

### Doc Tests
```bash
cargo test --doc
```

### All Tests
```bash
cargo test
```

### Coverage Reports
```bash
cargo tarpaulin --out Html --all --timeout 300
open tarpaulin-report.html  # macOS
xdg-open tarpaulin-report.html  # Linux
start tarpaulin-report.html  # Windows
```

## Code Style

- **Formatting**: Use `cargo fmt`
- **Linting**: Follow clippy warnings (`cargo clippy`)
- **Documentation**: All public items must have doc comments
- **Examples**: Public functions should include doc comment examples
- **Error Handling**: Use `Result<T>` and `?` operator; avoid `unwrap()` in library code

### Doc Comment Template

```rust
/// Brief one-line description.
///
/// More detailed explanation of what this does, when to use it,
/// and important notes about behavior.
///
/// # Examples
///
/// ```
/// # use mrrc::{Record, Leader, Field};
/// let record = Record::builder(Leader::default()).build();
/// assert_eq!(record.fields().count(), 0);
/// ```
///
/// # Errors
///
/// Returns an error if [condition], such as [error type].
///
/// # Panics
///
/// Panics if [unlikely condition].
pub fn my_function() -> Result<()> {
    // implementation
}
```

## Pull Request Process

1. **Create a Pull Request** on GitHub with a clear description
2. **Link related issues**: Reference Beads issue IDs in the PR description
3. **Add tests**: Include tests for new functionality
4. **Update documentation**: Ensure all public APIs are documented
5. **Ensure CI passes**: GitHub Actions will run automated checks
6. **Request review**: Tag maintainers for review
7. **Address feedback**: Make requested changes and update PR

## Feature Development

For significant features:

1. **Create a design document** in `docs/design/`:
    - Overview and problem statement
    - Proposed solution with examples
    - Implementation phases/roadmap
    - Testing strategy
    - Known limitations/risks

2. **Create parent epic issue**:
   ```bash
   bd create "Epic: Your feature" -t epic -p 2
   ```

3. **Create subtask issues**:
   ```bash
   bd create "Phase 1: Part of feature" -t task -p 2 --parent <epic-id>
   ```

4. **Link to design doc**: Add dependency in beads
   ```bash
   bd update <issue-id> --deps discovered-from:doc-filename
   ```

5. **Document progress**: Update issue descriptions as phases complete

## Areas for Contribution

### High Priority
- **Performance Optimizations**: Profile and optimize hot paths
- **Test Coverage**: Improve coverage of edge cases
- **Documentation Examples**: Add more real-world examples
- **Validation**: Enhance field indicator validation

### Medium Priority
- **Format Support**: Additional serialization formats
- **Query Extensions**: New query patterns and helpers
- **Error Messages**: More descriptive error context
- **Logging**: Debug/trace logging for troubleshooting

### Nice to Have
- **Benchmarks**: Performance measurement suite
- **Tooling**: Integration with other MARC tools
- **Bindings**: Language bindings (Python, Node.js, etc.)

## Communication

- **GitHub Issues**: For bugs and feature requests
- **GitHub Discussions**: For questions and ideas
- **Beads Issues**: For tracked work items
- **Pull Requests**: For code review
- **Design Documents**: See `docs/design/` for architectural decisions
- **Project History**: See `docs/history/` for implementation notes and audits

## Release Process

Releases follow semantic versioning (MAJOR.MINOR.PATCH):

1. Update `Cargo.toml` version
2. Update `CHANGELOG.md` with new features/fixes
3. Create git tag: `git tag v0.x.y`
4. Push tags: `git push --tags`
5. Publish to crates.io: `cargo publish`

## Resources

- **[MARC 21 Standard](https://www.loc.gov/marc/)** - Official MARC specification
- **[ISO 2709](https://en.wikipedia.org/wiki/MARC_standards)** - Binary format specification
- **[pymarc Documentation](https://pymarc.readthedocs.io/)** - Reference implementation
- **[Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)** - Idiomatic Rust patterns

## Questions?

- Open a GitHub issue for clarification
- Check existing issues for similar questions
- Review documentation in `docs/` (including `docs/design/` and `docs/history/`)

Thank you for contributing to MRRC!
