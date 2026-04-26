# MRRC — MARC Rust Crate

Rust library for MARC bibliographic records (ISO 2709) with pymarc-compatible
Python bindings via PyO3/maturin.

## Key Files

| Path | Purpose |
|------|---------|
| `src/` | Rust core library (parsing, serialization, query DSL) |
| `src-python/` | PyO3 bindings (`mrrc-python` crate) |
| `mrrc/__init__.py` | Python wrapper — re-exports and extends Rust types |
| `mrrc/_mrrc.pyi` | **Canonical Python API** — ground-truth type stubs |
| `.cargo/check.sh` | Local CI script (run before every push) |
| `tests/python/` | Active Python test suite |
| `examples/` | Runnable Rust and Python examples |
| `docs/` | mkdocs-material documentation site |
| `fuzz/` | Standalone cargo-fuzz workspace (nightly toolchain, isolated from root) |
| `docs/contributing/fuzzing.md` | Fuzzing install, local run, and CI-failure triage playbook |

## Build & Test

```bash
# Full pre-push check (~30s) — always run this before pushing
.cargo/check.sh

# Quick mode (skips docs, audit, maturin rebuild)
.cargo/check.sh --quick

# Individual commands
cargo test --lib --tests --package mrrc -q    # Rust tests
cargo test --doc --package mrrc -q            # Doc tests
uv run maturin develop --release              # Build Python extension
uv run python -m pytest tests/python/ -m "not benchmark" -q  # Python tests
uv run ruff check mrrc/ tests/python/         # Python lint

# Documentation site (mkdocs-material)
uv sync --all-extras                          # Install docs alongside test+dev
uv run mkdocs build                           # Build site to ./site/
uv run mkdocs serve                           # Live-reload preview at :8000
```

The `docs` extra in `pyproject.toml` carries `mkdocs-material`,
`mkdocstrings[python]`, and `pymdown-extensions`. CI installs it via
`uv sync --extra docs --no-install-project` — `--no-install-project`
skips building the mrrc Rust+PyO3 extension, which would require a
Rust toolchain + maturin in the docs-only job. Do not `uv pip install`
these ad-hoc — track them in the extra so contributors and CI stay in
sync.

**Use `uv sync --all-extras`, not `uv sync --extra docs`.** `uv sync`
is *replacing*, not additive — `--extra docs` alone drops `test` and
`dev` deps from the venv, so the next pytest run breaks. `--all-extras`
keeps everything resolved.

## Architecture

### Rust Core (`src/`)

The `mrrc` crate handles all parsing, serialization, and record manipulation.
Formats: ISO 2709, JSON, MARCXML, CSV, Dublin Core, MODS, BIBFRAME (RDF).

### Python Bindings (`src-python/src/`)

PyO3 `#[pyclass]` types expose Rust structs to Python. The compiled extension
is `_mrrc` (note the underscore).

### Python Wrapper (`mrrc/__init__.py`)

Re-exports Rust types and adds pymarc-compatible conveniences. The wrapper
pattern: `MARCReader` wraps `_MARCReader` via `self._inner` delegation.

**Adding a new feature requires all layers:**

1. Rust implementation in `src/`
2. PyO3 `#[getter]` / `#[pymethods]` in `src-python/src/`
3. Python re-export or wrapper in `mrrc/__init__.py`
4. Type stub update in `mrrc/_mrrc.pyi`

### GIL 3-Phase Model (MARCReader)

1. **Acquire GIL** — receive Python file object or path
2. **Release GIL** — parse record bytes in pure Rust
3. **Acquire GIL** — wrap result as Python object and return

This enables true multi-thread parallelism via `ThreadPoolExecutor`.

## Issue Tracking

This project uses **br** (beads_rust) for issue tracking.

```bash
br ready --json          # Show unblocked work
br create "Title" -t bug|feature|task -p 0-4 --json
br update <id> --status in_progress --json
br close <id> --reason "Completed" --json
br sync --flush-only     # Export DB to JSONL (never auto-commits)
```

Do NOT use `br edit` (opens `$EDITOR`, blocks agents).

## Warnings

- **`docs/history/`** — archival only (89 files). Do not modify.
- **Never close issues before CI passes** — commit, push, verify CI on all
  platforms, then close.

## Workflow

- New issue = new branch from `main`
- Always run `.cargo/check.sh` before pushing
- Use `uv` for all Python invocations

## See Also

- [`AGENTS.md`](AGENTS.md) — expanded agent instructions (testing workflow,
  session completion checklist)
