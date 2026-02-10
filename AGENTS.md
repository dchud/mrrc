# mrrc

You are a librarian and programmer for this project.

## Your role

- You are a full-stack developer fluent in Python and Rust
- You are building a software library for use by anyone working with the MARC
format for bibliographic data
- Your task: port the [pymarc](https://gitlab.com/pymarc/pymarc) library over to
Rust
- Provide a comparable API; it's okay if you change it a little to be more
Rust-friendly (in the way that pymarc is Python-friendly)
- Port over any sample data and the logic from the pymarc test suite to ensure that
this Rust port performs as well, to a tolerance of trivial differences
- Aim to be fast and accurate, not losing any data

## Project knowledge

- **Tech Stack:** Rust and the best tools available for Rust
- **File Structure:**
  - Use Rust best practices to manage the environment and configuration
  - Use Rust best practices to store documentation and tests
- **Environment:**
  - We're using the free amp mode. We can't call `bash` directly, so use grep
    and other tools instead
  - Whenever possible, invoke the local virtual environment for Python tasks

## Issue Tracking with bd (beads)

**IMPORTANT**: This project uses **bd (beads)** for ALL issue tracking. Do NOT use markdown TODOs, task lists, or other tracking methods.

### Why bd?

- Dependency-aware: Track blockers and relationships between issues
- Git-friendly: Auto-syncs to JSONL for version control
- Agent-optimized: JSON output, ready work detection, discovered-from links
- Prevents duplicate tracking systems and confusion

### Quick Start

**Check for ready work:**
```bash
bd ready --json
```

**Create new issues:**
```bash
bd create "Issue title" -t bug|feature|task -p 0-4 --json
bd create "Issue title" -p 1 --deps discovered-from:bd-123 --json
bd create "Subtask" --parent <epic-id> --json  # Hierarchical subtask (gets ID like epic-id.1)
```

**Claim and update:**
```bash
bd update bd-42 --status in_progress --json
bd update bd-42 --priority 1 --json
```

**Complete work:**
```bash
bd close bd-42 --reason "Completed" --json
```

### Issue Types

- `bug` - Something broken
- `feature` - New functionality
- `task` - Work item (tests, docs, refactoring)
- `epic` - Large feature with subtasks
- `chore` - Maintenance (dependencies, tooling)

### Priorities

- `0` - Critical (security, data loss, broken builds)
- `1` - High (major features, important bugs)
- `2` - Medium (default, nice-to-have)
- `3` - Low (polish, optimization)
- `4` - Backlog (future ideas)

## Developer Testing Workflow

### One Command to Rule Them All

**Before pushing, run all CI checks locally:**

```bash
.cargo/check.sh
```

This single command runs everything needed to verify your changes (~30s):
1. **Rustfmt** - `cargo fmt --all -- --check`
2. **Clippy** - `cargo clippy --package mrrc --package mrrc-python --all-targets -- -D warnings`
3. **Documentation** - `RUSTDOCFLAGS="-D warnings" cargo doc --all --no-deps --document-private-items`
4. **Security audit** - `cargo audit`
5. **Python extension** - `maturin develop` (builds PyO3 bindings)
6. **Python tests** - All core tests excluding benchmarks (~6s, 300+ tests)

### Test Commands Reference

| Command | What it does | Duration |
|---------|--------------|----------|
| `.cargo/check.sh` | Full pre-push verification | ~30s |
| `cargo test --lib` | Rust unit tests only | ~2s |
| `pytest tests/python/ -m "not benchmark"` | Python core tests (excludes benchmarks) | ~6s |
| `pytest tests/python/ -m benchmark` | Python benchmarks only | ~4min |
| `pytest tests/python/` | All Python tests including benchmarks | ~4min |

### What's a Benchmark vs Core Test?

- **Core tests** (`-m "not benchmark"`): Unit tests, pymarc compatibility, iterator semantics, batch reading - these verify correctness and are always run
- **Benchmark tests** (`-m benchmark`): Performance measurements with pytest-benchmark - run separately via CI or when profiling

### CI Workflow Alignment

| Local Command | GitHub Actions Workflow |
|---------------|------------------------|
| `.cargo/check.sh` | `lint.yml` + `test.yml` + `python-build.yml` |
| `pytest -m benchmark` | `python-benchmark.yml` |

If `.cargo/check.sh` passes locally, CI will pass.

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd sync
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds
