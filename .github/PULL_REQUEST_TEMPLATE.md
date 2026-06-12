<!-- Describe what the PR changes and why. Link the issue it resolves with
"Closes" so it auto-closes on merge. -->

## Checklist

- [ ] `.cargo/check.sh` passes locally
- [ ] Tests added or updated for the change
- [ ] `CHANGELOG.md` updated under `[Unreleased]` (user-facing changes only)
- [ ] Documentation updated (docstrings, type stubs, `docs/` pages as applicable)
- [ ] Python API changes follow pymarc conventions and update all four layers
      (Rust core, PyO3 bindings, Python wrapper, type stubs)
- [ ] Related tracked issue referenced (bead ID at the bottom of this description,
      if one exists)
