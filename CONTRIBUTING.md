# Contributing to MRRC

The contributor guide lives in [`docs/contributing/`](docs/contributing/index.md)
(published at <https://dchud.github.io/mrrc/>). Start there. This file covers only
the essentials GitHub surfaces directly.

## Reporting

- Bugs and feature requests: open a GitHub issue.
- Security vulnerabilities: follow [SECURITY.md](SECURITY.md) rather than opening a
  public issue.

## Where to look

- [Development setup](docs/contributing/development-setup.md) — toolchain, build,
  tests, IDE.
- [Testing](docs/contributing/testing.md) — running the suite, fixtures, coverage.
- [Project layout](docs/contributing/project-layout.md) and
  [Architecture](docs/contributing/architecture.md) — how the Rust core, PyO3
  bindings, and Python wrapper fit together.
- [Release procedure](docs/contributing/release-procedure.md).
- [Error codes](docs/reference/error-codes.md) — the `Exxx`/`Wxxx` catalog, the
  stability rules, and how to add a code.

## Workflow

- `main` is protected; every change lands through a pull request from a branch.
- Run `.cargo/check.sh` before pushing — it mirrors CI (rustfmt, clippy, docs,
  audit, build, Rust and Python tests).
- The Tests and Lint checks are required before a PR can merge; a maintainer
  merges once they pass.
- Write clear, imperative commit messages and reference the issue a PR resolves
  with `Closes #NNN`.

## Issue tracking

mrrc tracks work with [br (beads_rust)](https://github.com/Dicklesworthstone/beads_rust).
The database is committed as `.beads/issues.jsonl` on your branch and reaches
`main` through your PR; `br` never runs git for you.
