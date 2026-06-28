# Contributing to MRRC

The contributor guide is published at
<https://dchud.github.io/mrrc/contributing/> (source in `docs/contributing/`).
Start there. This file covers only the essentials GitHub surfaces directly.

## Reporting

- Bugs and feature requests: open a GitHub issue.
- Security vulnerabilities: follow [SECURITY.md](SECURITY.md) rather than opening a
  public issue.

## Where to look

- [Development setup](https://dchud.github.io/mrrc/contributing/development-setup/)
  — toolchain, build, tests, code style, IDE.
- [Testing](https://dchud.github.io/mrrc/contributing/testing/)
  — running the suite, fixtures, coverage, memory-safety checks.
- [Project layout](https://dchud.github.io/mrrc/contributing/project-layout/) and
  [Architecture](https://dchud.github.io/mrrc/contributing/architecture/)
  — how the Rust core, PyO3 bindings, and Python wrapper fit together.
- [Release procedure](https://dchud.github.io/mrrc/contributing/release-procedure/).
- [Error codes](https://dchud.github.io/mrrc/reference/error-codes/)
  — the `Exxx`/`Wxxx` catalog, the stability rules, and how to add a code.

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
