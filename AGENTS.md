# MRRC — Agent Instructions

You are maintaining and extending a mature Rust MARC library with
pymarc-compatible Python bindings.

## Project Overview

MRRC is a Rust library for reading, writing, and manipulating MARC
bibliographic records (ISO 2709). It provides Python bindings via PyO3/maturin
that aim for API compatibility with pymarc.

## Your Role

- Full-stack developer fluent in Python and Rust
- Maintain and extend the MARC library for anyone working with bibliographic data
- Preserve pymarc API compatibility in the Python layer
- Aim for speed and accuracy — never lose data

## Key Files

| Path | Purpose |
|------|---------|
| `src/` | Rust core library (parsing, serialization, query DSL) |
| `src-python/src/` | PyO3 bindings (`mrrc-python` crate) |
| `mrrc/__init__.py` | Python wrapper — re-exports and extends Rust types |
| `mrrc/_mrrc.pyi` | **Canonical Python API** — ground-truth type stubs |
| `.cargo/check.sh` | Local CI script (run before every push) |
| `tests/python/` | Active Python test suite |
| `examples/` | Runnable Rust and Python examples |
| `docs/` | mkdocs-material documentation site |

## Architecture

### Layer Stack

1. **Rust core** (`src/`) — all parsing, serialization, record manipulation
2. **PyO3 bindings** (`src-python/src/`) — `#[pyclass]`/`#[pymethods]` wrappers
3. **Python wrapper** (`mrrc/__init__.py`) — re-exports + pymarc conveniences
4. **Type stubs** (`mrrc/_mrrc.pyi`) — canonical Python API reference

### Adding a New Feature

Every new feature touches all layers:

1. Rust implementation in `src/`
2. PyO3 `#[getter]` / `#[pymethods]` in `src-python/src/`
3. Python re-export or wrapper in `mrrc/__init__.py`
4. Type stub update in `mrrc/_mrrc.pyi`

### Python Wrapper Delegation

`MARCReader` wraps `_MARCReader` via `self._inner`. Other wrappers (`Record`,
`Field`, `Leader`) follow the same pattern. Always check
`mrrc/__init__.py` to see what the Python layer adds on top of the Rust type.

### GIL 3-Phase Model (MARCReader)

1. **Acquire GIL** — receive Python file object or path
2. **Release GIL** — parse record bytes in pure Rust
3. **Acquire GIL** — wrap result as Python object and return

## Environment

- Use `uv` to invoke the local virtual environment for all Python tasks
- Use Rust best practices for environment and configuration

## Issue Tracking with br (beads_rust)

**IMPORTANT**: This project uses **br** (beads_rust) for ALL issue tracking.
Do NOT use markdown TODOs, task lists, or other tracking methods.

### Why br?

- Dependency-aware: Track blockers and relationships between issues
- Git-friendly: Syncs to JSONL for version control
- Agent-optimized: JSON output, ready work detection, discovered-from links
- Prevents duplicate tracking systems and confusion

### Quick Start

**Check for ready work:**
```bash
br ready --json
```

**Create new issues:**
```bash
br create "Issue title" -t bug|feature|task -p 0-4 --json
br create "Issue title" -p 1 --deps discovered-from:mrrc-abc --json
br create "Subtask" --parent <epic-id> --json
```

**Claim and update:**
```bash
br update mrrc-42 --status in_progress --json
br update mrrc-42 --priority 1 --json
```

**Complete work:**
```bash
br close mrrc-42 --reason "Completed" --json
```

Do NOT use `br edit` (opens `$EDITOR`, blocks agents).

### Issue Types

- `bug` — Something broken
- `feature` — New functionality
- `task` — Work item (tests, docs, refactoring)
- `epic` — Large feature with subtasks
- `chore` — Maintenance (dependencies, tooling)

### Priorities

- `0` — Critical (security, data loss, broken builds)
- `1` — High (major features, important bugs)
- `2` — Medium (default, nice-to-have)
- `3` — Low (polish, optimization)
- `4` — Backlog (future ideas)

## Developer Testing Workflow

### One Command to Rule Them All

**Before pushing, run all CI checks locally:**

```bash
.cargo/check.sh
```

This single command runs everything needed to verify your changes (~30s):
1. **Rustfmt** — `cargo fmt --all -- --check`
2. **Clippy** — `cargo clippy --package mrrc --package mrrc-python --all-targets -- -D warnings`
3. **Documentation** — `RUSTDOCFLAGS="-D warnings" cargo doc --all --no-deps --document-private-items`
4. **Security audit** — `cargo audit`
5. **Python extension** — `maturin develop` (builds PyO3 bindings)
6. **Rust tests** — library + integration + doc tests
7. **Python tests** — all core tests excluding benchmarks (~6s, 300+ tests)
8. **Python lint** — `ruff check mrrc/ tests/python/`

### Test Commands Reference

| Command | What it does | Duration |
|---------|--------------|----------|
| `.cargo/check.sh` | Full pre-push verification | ~30s |
| `.cargo/check.sh --quick` | Skip docs, audit, maturin rebuild | ~15s |
| `cargo test --lib --tests --package mrrc -q` | Rust unit + integration tests | ~2s |
| `cargo test --doc --package mrrc -q` | Rust doc tests | ~2s |
| `uv run python -m pytest tests/python/ -m "not benchmark" -q` | Python core tests | ~6s |
| `uv run python -m pytest tests/python/ -m benchmark` | Python benchmarks | ~4min |

### What's a Benchmark vs Core Test?

- **Core tests** (`-m "not benchmark"`): Unit tests, pymarc compatibility,
  iterator semantics, batch reading — these verify correctness and always run
- **Benchmark tests** (`-m benchmark`): Performance measurements with
  pytest-benchmark — run separately via CI or when profiling

### CI Workflow Alignment

| Local Command | GitHub Actions Workflow |
|---------------|------------------------|
| `.cargo/check.sh` | `lint.yml` + `test.yml` + `python-build.yml` |
| `pytest -m benchmark` | `python-benchmark.yml` |

If `.cargo/check.sh` passes locally, CI will pass.

## Warnings

- **`docs/history/`** — archival only (89 files). Do not modify.
- **Never close issues before CI passes** — commit, push, verify CI on all
  platforms, then close.

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT
complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** — Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) — Tests, linters, builds
3. **Update issue status** — Close finished work, update in-progress items
4. **PUSH TO REMOTE** — This is MANDATORY:
   ```bash
   git pull --rebase
   br sync --flush-only
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** — Clear stashes, prune remote branches
6. **Verify** — All changes committed AND pushed
7. **Hand off** — Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing — that leaves work stranded locally
- NEVER say "ready to push when you are" — YOU must push
- If push fails, resolve and retry until it succeeds

## See Also

- [`CLAUDE.md`](CLAUDE.md) — concise project reference for Claude Code sessions
