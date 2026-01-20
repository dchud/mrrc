# MRRC Release Procedure

**Status**: Reference documentation for release managers  
**Estimated Duration**: 30-45 minutes for a standard release  
**Last Updated**: 2026-01-20

This document provides a step-by-step, executable procedure for preparing, testing, and publishing a new release of MRRC (MARC Rust Crate). Follow these steps sequentially to ensure nothing is missed.

## Table of Contents

1. [Pre-Release Verification](#pre-release-verification)
2. [Version Number Selection](#version-number-selection)
3. [Update Configuration Files](#update-configuration-files)
4. [Update Changelog](#update-changelog)
5. [Update Documentation](#update-documentation)
6. [Final Sanity Check](#final-sanity-check-before-git-operations)
7. [Git Operations](#git-operations)
8. [Publishing](#publishing)
9. [Post-Release Verification](#post-release-verification)
10. [Post-Release Setup](#post-release-setup)
11. [Rollback Procedures](#rollback-procedures)
12. [Troubleshooting](#troubleshooting)

---

## Pre-Release Verification

Before making any changes, verify that the codebase is in a releasable state.

### 1.1 Run Full Test Suite

```bash
cd /Users/dchud/Documents/projects/mrrc
.cargo/check.sh
```

**Expected output**: All checks pass (rustfmt, clippy, documentation, security audit, Python tests)

**Checklist**:
- [ ] Rustfmt passes (no formatting issues)
- [ ] Clippy passes (no warnings)
- [ ] Documentation builds without errors or warnings
- [ ] Security audit passes (no vulnerabilities)
- [ ] Maturin builds successfully
- [ ] All Python tests pass (core tests only, benchmarks skipped)

If any checks fail, **do not proceed**. Fix the issues and re-run.

### 1.2 Verify Git Status

```bash
git status
```

**Expected**: Clean working tree (no uncommitted changes)

```bash
git log --oneline -5
```

Review recent commits to ensure all changes are documented.

**Checklist**:
- [ ] Working tree is clean
- [ ] All intended changes are committed
- [ ] Recent commit messages are clear and descriptive

### 1.3 Verify Documentation Completeness

Check that all features merged since the last release are documented:

```bash
# View unreleased changelog section
head -150 CHANGELOG.md
```

**Checklist**:
- [ ] All recent features are listed in [Unreleased] section
- [ ] All breaking changes are documented with migration guidance
- [ ] All bug fixes are noted
- [ ] All performance improvements documented
- [ ] Known limitations are listed if applicable

**Breaking Changes Specific Check**:
If this release includes breaking changes:
- [ ] **REQUIRED**: Migration guide exists in [Unreleased] or referenced
- [ ] **REQUIRED**: Deprecation notices were given in previous release (if applicable)
- [ ] **REQUIRED**: Major version bump scheduled (e.g., 0.x → 1.0)

---

## Version Number Selection

MRRC currently uses **Semantic Versioning (SemVer)**: `MAJOR.MINOR.PATCH`

### 2.1 Determine Version Increment

**Use the following rules**:

- **PATCH** (e.g., 0.4.0 → 0.4.1): Bug fixes, documentation updates, non-breaking performance improvements
- **MINOR** (e.g., 0.4.0 → 0.5.0): New features, backward-compatible API additions
- **MAJOR** (e.g., 0.4.0 → 1.0.0): Breaking changes, significant redesigns

**Current version**: Check `Cargo.toml`:

```bash
grep '^version' Cargo.toml
```

### 2.2 Document Version Rationale

Note the reason for the chosen version increment in your release notes/commit message (e.g., "minor: added query DSL feature" or "patch: fixed field ordering bug").

---

## Update Configuration Files

Update version numbers in all configuration files. **Do this in order** and **verify each file** before proceeding.

### 3.1 Update Root Cargo.toml

**File**: `Cargo.toml`

```bash
# Edit Cargo.toml with your preferred editor
# Find the [package] section and update version:
# version = "0.4.0"  →  version = "0.5.0"  (example)
```

**Verification**:
```bash
grep '^version' Cargo.toml
```

**Checklist**:
- [ ] Root Cargo.toml updated
- [ ] Version number is correct

### 3.2 Update Python Cargo.toml

**File**: `src-python/Cargo.toml`

Update the version in the `[package]` section to match the root version:

```bash
# Same version as root Cargo.toml
```

**Verification**:
```bash
grep '^version' src-python/Cargo.toml
```

**Checklist**:
- [ ] Python Cargo.toml updated
- [ ] Version number matches root

### 3.3 Update pyproject.toml

**File**: `pyproject.toml`

Update the version in the `[project]` section:

```bash
# Find: version = "0.4.0"
# Change to: version = "0.5.0"  (example)
```

**Verification**:
```bash
grep '^version' pyproject.toml
```

**Checklist**:
- [ ] pyproject.toml updated
- [ ] Version number matches root and Python Cargo.toml

### 3.4 Verify All Versions Match

```bash
echo "Root: $(grep '^version' Cargo.toml)"
echo "Python Cargo: $(grep '^version' src-python/Cargo.toml)"
echo "PyProject: $(grep '^version' pyproject.toml)"
```

All three lines should show the same version number.

**Checklist**:
- [ ] All three version numbers are identical

---

## Update Changelog

### 4.1 Prepare Changelog Section

**File**: `CHANGELOG.md`

Locate the `## [Unreleased]` section at the top of the file.

### 4.2 Create Release Entry

Replace `## [Unreleased]` with:

```markdown
## [X.Y.Z] - YYYY-MM-DD

### Added

### Changed

### Fixed

### Performance

### Documentation

## [Unreleased]
```

Where:
- `X.Y.Z` is the new version number
- `YYYY-MM-DD` is today's date (ISO 8601 format)

### 4.3 Move Content

Move all content from the old `## [Unreleased]` section into the new versioned section, organizing by category if needed:
- **Added**: New features
- **Changed**: Changes to existing functionality
- **Fixed**: Bug fixes
- **Performance**: Performance improvements
- **Documentation**: Documentation changes

### 4.4 Create New Unreleased Section

Add a fresh `## [Unreleased]` section at the top with empty subsections:

```markdown
## [Unreleased]

### Added

### Changed

### Fixed

### Performance

### Documentation
```

### 4.5 Verify Format

**Checklist**:
- [ ] Old [Unreleased] section converted to versioned section
- [ ] New [Unreleased] section created
- [ ] Version number matches all config files
- [ ] Date is correct (today's date)
- [ ] All content is categorized appropriately
- [ ] No placeholder text remains in released section

---

## Update Documentation

Check all documentation files for version-specific content or broken references.

### 5.1 Check README.md

```bash
grep -i "version\|0\.4\|0\.5\|installation" README.md
head -50 README.md
```

Update any version-specific instructions or examples. Pay special attention to:
- Installation instructions with version constraints
- Feature availability notes tied to versions
- Example code showing output from specific versions
- Supported Rust versions or Python versions if changed

**Checklist**:
- [ ] README.md reviewed for version references
- [ ] Installation instructions are current
- [ ] Example code compiles and runs
- [ ] Badge versions (if any) are current

### 5.2 Check docs/ Directory

```bash
# Check for hardcoded version numbers
grep -r "0\.[0-9]\|version" docs/ --include="*.md" | grep -v "CHANGELOG\|history\|version =" | head -20

# Check for broken internal links
grep -r "\[.*\](.*\.md)" docs/ --include="*.md" | grep -v "http"
```

Update any version-specific documentation in:
- `docs/MIGRATION_GUIDE.md` - Add section for new release
- `docs/PERFORMANCE.md` - Update performance baselines if optimizations were made
- `docs/ARCHITECTURE.md` - Note architectural changes if any
- `docs/design/` - Update design documents if decisions changed
- Any docs with "Supported in X.Y+" style language

**Checklist**:
- [ ] docs/ directory scanned for version references
- [ ] All hardcoded versions updated or explained
- [ ] Internal links are still valid
- [ ] Migration guide updated if breaking changes
- [ ] Performance baselines are current (if applicable)

### 5.3 Check Example Code

```bash
ls -la examples/
cargo build --examples
```

Run all example code to ensure it still works with the new version:

```bash
# Run a few key examples to verify they work
cargo run --example format_conversion -- --help 2>&1 | head -5
```

If examples take file arguments, verify they run without the arguments too (should show usage/help):

```bash
# For examples that are expected to fail gracefully if missing args
cargo run --example some_example 2>&1 | head -10
```

Fix any broken examples before release.

**Checklist**:
- [ ] All examples compile successfully
- [ ] Key examples run without errors
- [ ] Example output is sensible (or shows expected error if args missing)

### 5.4 Update AGENTS.md (if workflow changed)

Check if the release procedure itself changed or if any development workflows were modified:

```bash
# Review AGENTS.md for outdated instructions
grep -i "release\|version\|publish\|tag" AGENTS.md | head -10
```

If you modified the release procedure in this session, add a note for future updates.

**Checklist**:
- [ ] AGENTS.md reviewed for stale guidance
- [ ] Release workflow notes are accurate
- [ ] If procedure changed, note updates for next release

---

## Final Sanity Check (Before Git Operations)

Before committing version changes and creating the git tag, perform one final verification pass.

### 6.0 Final Verification

```bash
# 1. Verify all version files match
echo "=== Version Check ==="
echo "Root: $(grep '^version' Cargo.toml)"
echo "Python Cargo: $(grep '^version' src-python/Cargo.toml)"
echo "PyProject: $(grep '^version' pyproject.toml)"

# 2. Verify changelog was updated
echo ""
echo "=== Changelog Check ==="
head -20 CHANGELOG.md | grep -E "Unreleased|^\[0-9]"

# 3. Run tests one more time
echo ""
echo "=== Running Tests ==="
.cargo/check.sh
```

All checks must pass before proceeding to git operations.

**Checklist**:
- [ ] All three version numbers are identical
- [ ] Changelog shows new version with today's date
- [ ] All tests pass
- [ ] Git status is clean (only version files changed)
- [ ] Confidence level: Ready to tag and push

**If anything fails**:
1. Fix the issue
2. Re-run checks above
3. Do **NOT** proceed until all pass

---

## Git Operations

### 7.1 Commit Version and Changelog Updates

```bash
git add Cargo.toml src-python/Cargo.toml pyproject.toml CHANGELOG.md
git commit -m "chore: release version X.Y.Z

- Update Cargo.toml, src-python/Cargo.toml, pyproject.toml to version X.Y.Z
- Update CHANGELOG.md with release notes and date
- All tests passing, ready for publication"
```

**Format notes**:
- Use `chore:` prefix for release commits
- Include all updated files in commit message body
- Keep message clear and concise

**Verification**:
```bash
git log --oneline -1
```

Should show your new commit.

**Checklist**:
- [ ] Commit created successfully
- [ ] Commit message is clear
- [ ] All version files included in commit

### 7.2 Create Git Tag

Create an annotated tag for the release:

```bash
git tag -a vX.Y.Z -m "Release version X.Y.Z"
```

Where `X.Y.Z` is the version number. The `v` prefix is **required** for GitHub Actions to trigger the release workflow.

**Example**:
```bash
git tag -a v0.5.0 -m "Release version 0.5.0"
```

**Verification**:
```bash
git tag -l -n1 | grep "v0\."
```

Should show your new tag.

**Checklist**:
- [ ] Tag created with `v` prefix
- [ ] Tag name matches version number
- [ ] Tag message is clear

### 7.3 Push Commit and Tag

```bash
git push origin main
git push origin vX.Y.Z
```

**Verification**:
```bash
git status
```

Should show "Your branch is up to date with 'origin/main'."

Also verify tags pushed:
```bash
git ls-remote origin | grep "refs/tags/vX.Y.Z"
```

**Checklist**:
- [ ] Commit pushed to origin/main
- [ ] Tag pushed to origin
- [ ] Git status shows "up to date"

---

## Publishing

The publishing process is **mostly automated** via GitHub Actions. The workflow is triggered by pushing the `v*` tag.

### 8.1 Verify GitHub Actions Triggered

Go to: https://github.com/dchud/mrrc/actions

Look for the "Python Release to PyPI" workflow:
- It should be triggered automatically by your tag push
- It builds wheels on Ubuntu, macOS, and Windows
- It runs tests on all wheels
- It publishes to PyPI

**Expected duration**: 10-20 minutes

**Checklist**:
- [ ] Workflow appears in GitHub Actions
- [ ] Workflow status is "in progress" or "completed"

### 8.2 Monitor Build Process

**Wait for**:
1. **build-release-wheels** job - Builds Python wheels for 3×3.9-3.12 on macOS/Ubuntu/Windows
2. **test-release-wheels** job - Tests all wheels
3. **publish-pypi** job - Publishes wheels to PyPI

Each job has multiple matrix runs (fail-fast: false means all run even if some fail).

**Expected outcomes**:
- All build jobs pass ✓
- All test jobs pass ✓
- PyPI publish job completes ✓

**If any job fails**: See [Troubleshooting](#troubleshooting)

### 8.3 Verify crates.io Publication (Manual)

The crates.io publication is **manual** (requires crates.io API token).

**Local publication** (if CI doesn't do it):

```bash
cd /Users/dchud/Documents/projects/mrrc
cargo publish --dry-run
```

If dry-run succeeds:
```bash
cargo publish
```

**Verification**:
https://crates.io/crates/mrrc/versions

Should list your new version within 1-2 minutes.

**Checklist**:
- [ ] Dry-run succeeds locally (optional)
- [ ] Version appears on crates.io
- [ ] Version documentation is correct on docs.rs

### 8.4 Verify PyPI Publication

https://pypi.org/project/mrrc/

Should show your new version with wheels for Python 3.9, 3.10, 3.11, 3.12 across macOS, Ubuntu, and Windows.

**Expected**: 12 wheels total (4 Python versions × 3 platforms)

**Checklist**:
- [ ] Version appears on PyPI
- [ ] All wheels are present (12 total)
- [ ] Documentation is correct

### 8.5 Verify GitHub Release Created

GitHub Actions automatically creates a GitHub Release with wheels attached.

Go to: https://github.com/dchud/mrrc/releases

Should see your new release with:
- Tag name: `vX.Y.Z`
- Release title (auto-generated from tag)
- Wheels attached from `dist/`
- Status: Not a draft, not a prerelease (unless version contains alpha/beta)

**Checklist**:
- [ ] Release appears on GitHub
- [ ] Wheels are attached
- [ ] Release status is correct (not draft)

---

## Post-Release Verification

### 8.1 Test Installation from PyPI (Python)

On a fresh machine or in a new virtual environment:

```bash
python -m venv /tmp/test_mrrc
source /tmp/test_mrrc/bin/activate
pip install mrrc
python -c "import mrrc; print(mrrc.__version__)"
```

Should print the new version (if `__version__` is exposed; if not, just verify import succeeds).

**Checklist**:
- [ ] Installation from PyPI succeeds
- [ ] Package imports correctly
- [ ] Version matches expected

### 8.2 Test Installation from crates.io (Rust)

In a test Rust project:

```bash
cargo init /tmp/test_mrrc_rust
cd /tmp/test_mrrc_rust
cargo add mrrc@X.Y.Z
cargo check
```

**Checklist**:
- [ ] Crate installs from crates.io
- [ ] Compilation succeeds
- [ ] Dependency resolution is correct

### 8.3 Verify docs.rs Documentation

Go to: https://docs.rs/mrrc/latest/mrrc/

Docs should be built and available. If not, they build automatically within a few minutes.

**Expected**: Latest version's documentation is current and complete

**Checklist**:
- [ ] docs.rs has documentation for new version
- [ ] Documentation is complete (no build errors)

---

## Post-Release Setup

### 9.1 Verify Sync and Beads

Sync any issue tracking changes:

```bash
bd sync
git status
```

No changes should be required unless you manually updated issue statuses.

### 9.2 Create Post-Release Issue (Optional)

If there are known issues or follow-up work:

```bash
bd create "Post-release items for X.Y.Z" \
  -t task \
  -p 3 \
  --description "Items identified during release X.Y.Z that don't block release:
  
- Item 1
- Item 2
- Item 3

Link to release: https://github.com/dchud/mrrc/releases/tag/vX.Y.Z" \
  --json
```

### 9.3 Begin Next Development Cycle

After release, update the [Unreleased] section to reflect the next development direction.

**File**: `CHANGELOG.md`

The [Unreleased] section should remain mostly empty with just subsection headers:

```markdown
## [Unreleased]

### Added

### Changed

### Fixed

### Performance

### Documentation
```

**Planning & Tracking**:

If you have identified planned work items for the next release, create beads issues INSTEAD of hardcoding them in the changelog. This keeps CHANGELOG.md clean and issues tracked in the issue system.

Example workflow:
1. After release, review roadmap and open issues
2. Create new issues for planned work: `bd create "Feature: ..." -p 2`
3. Link major items in README roadmap or docs/RELEASES.md (but NOT in CHANGELOG [Unreleased])
4. The [Unreleased] section fills up organically as work is completed during development

**Why?** CHANGELOG documents what was released, not what's planned. Planning belongs in the issue tracker (beads).

**Checklist**:
- [ ] [Unreleased] section has empty subsections ready for new content
- [ ] Major planned work items created as beads issues (not hardcoded in CHANGELOG)
- [ ] Issue tracker (beads) reflects the development roadmap
- [ ] README or docs reflect high-level roadmap if needed

### 9.4 Commit Post-Release Changes (if any)

Only commit if you made changelog changes:

```bash
git status
# If only CHANGELOG.md changed:
git add CHANGELOG.md
git commit -m "docs: clear [Unreleased] section after vX.Y.Z release"
git push origin main
```

**Checklist**:
- [ ] No uncommitted changes
- [ ] Post-release commit (if needed) pushed to main

---

## Rollback Procedures

Use these procedures only if critical issues are discovered after release.

### 10.1 Immediate: Yank on crates.io (Rust)

If a critical security or correctness issue exists:

```bash
# REQUIRES crates.io API token and owner permissions
cargo yank --vers X.Y.Z
```

This marks the version as "yanked" (not recommended), but does not delete it. Users with exact version pins can still use it; cargo will warn them.

**Announcement**: Notify users via:
- GitHub release notes (edit release to add notice)
- Email if applicable
- Issue tracker

### 10.2 Immediate: Delete Release on PyPI (Python)

If the release is newer and hasn't been heavily downloaded:

**Manual process** (no CLI tool):
1. Go to https://pypi.org/project/mrrc/
2. Click on version number
3. Click "Yank release" or "Delete release"
4. Confirm

**Note**: PyPI supports "yanking" (like crates.io) which is preferred to deletion.

**Announcement**: Same as above.

### 10.3 Revert Git Tag

If pushing the wrong tag:

```bash
git tag -d vX.Y.Z                    # Delete local tag
git push origin :refs/tags/vX.Y.Z   # Delete remote tag
```

Then fix the issue and re-tag:

```bash
git tag -a vX.Y.Z -m "Release version X.Y.Z (re-release)"
git push origin vX.Y.Z
```

### 10.4 Fix and Re-release

For issues discovered immediately:

1. **Fix the issue** in code and tests
2. **Increment PATCH version** (e.g., 0.5.0 → 0.5.1)
3. **Follow the full release procedure again** with the new version
4. **Update GitHub release notes** to reference the fix and advise users to upgrade

---

## Troubleshooting

### GitHub Actions Build Fails

**Problem**: Wheels fail to build on one platform (e.g., Windows)

**Steps**:
1. Check the GitHub Actions logs for the specific failure
2. Common causes:
   - Missing protobuf compiler installation
   - Rust toolchain version mismatch
   - Python version not available on that platform
3. Fix the issue in the workflow file (`.github/workflows/python-release.yml`)
4. Delete the failed tag: `git tag -d vX.Y.Z && git push origin :refs/tags/vX.Y.Z`
5. Commit the workflow fix: `git commit -am "fix: python-release workflow"`
6. Re-tag and push: `git tag -a vX.Y.Z -m "Release version X.Y.Z" && git push origin vX.Y.Z`

### PyPI Publication Fails

**Problem**: Wheels build successfully but PyPI publish job fails

**Steps**:
1. Check if `PYPI_API_TOKEN` is set in GitHub secrets
2. Verify token has permissions for the `mrrc` package
3. If token is expired or invalid, update it in repository settings
4. Re-run the failed job or push a new tag

**Note**: You can manually publish wheels if CI fails:
```bash
pip install twine
twine upload dist/mrrc-*.whl
```

### crates.io Publish Fails

**Problem**: Wheels publish but crates.io publication is skipped or fails

**Steps**:
1. Ensure you have crates.io API token
2. Publish manually:
   ```bash
   cd /Users/dchud/Documents/projects/mrrc
   cargo publish
   ```
3. If auth fails: `cargo login` and paste token
4. Verify on crates.io within 1-2 minutes

### Docs.rs Documentation Doesn't Build

**Problem**: Version appears on crates.io but docs.rs shows "Documentation Failed to Build"

**Steps**:
1. Check docs.rs build logs: Go to https://docs.rs/mrrc/X.Y.Z and click "Docs Build"
2. Common causes:
   - doc comments with syntax errors
   - Dependencies missing on docs.rs
3. Fix the issue in source code
4. **Do NOT re-release**: docs.rs will automatically rebuild the docs for the existing version within a few minutes
5. Verify rebuild by checking the page again

### Version Mismatch Between Cargo/PyProject

**Problem**: Deployment uses version A in Cargo.toml but version B in pyproject.toml

**Steps**:
1. Verify all three configs match: `Cargo.toml`, `src-python/Cargo.toml`, `pyproject.toml`
2. If mismatch exists:
   - Fix all to the same version
   - Delete the incorrect tag: `git tag -d vX.Y.Z && git push origin :refs/tags/vX.Y.Z`
   - Recommit and retag (see GitHub Actions Build Fails section)

### Test Failures During Release

**Problem**: One of the `test-release-wheels` jobs fails

**Steps**:
1. Check the job logs for the specific failure
2. Determine if it's:
   - **Flaky test**: Re-run the job from GitHub Actions UI
   - **Real issue**: Fix the code, recommit, and re-release
3. If fixing code:
   - Increment PATCH version
   - Follow release procedure with new version
   - Delete old broken tag if desired

---

## Checklist: Full Release Workflow

**Phase 1: Pre-Release Verification**
- [ ] `.cargo/check.sh` passes all checks
- [ ] Git status is clean
- [ ] [Unreleased] section is complete and accurate
- [ ] Breaking changes checked (if any)

**Phase 2: Version Number Selection**
- [ ] Version number determined (SemVer rationale noted)
- [ ] Current version identified from Cargo.toml

**Phase 3: Configuration File Updates**
- [ ] Cargo.toml updated
- [ ] src-python/Cargo.toml updated
- [ ] pyproject.toml updated
- [ ] All three versions match

**Phase 4: Changelog & Documentation**
- [ ] CHANGELOG.md: [Unreleased] → [X.Y.Z]
- [ ] New [Unreleased] section created
- [ ] README.md reviewed and updated
- [ ] docs/ directory scanned for version refs
- [ ] Example code compiles and runs
- [ ] AGENTS.md reviewed for stale guidance

**Phase 5: Final Sanity Check**
- [ ] Version consistency verified (all three files match)
- [ ] Changelog shows new version with correct date
- [ ] All tests pass (full `.cargo/check.sh`)
- [ ] Git status is clean (only config files changed)

**Phase 6: Git Operations**
- [ ] Version update commit created with proper message
- [ ] Commit pushed to origin/main
- [ ] Git tag created with `v` prefix
- [ ] Tag pushed to origin

**Phase 7: Publishing & Verification**
- [ ] GitHub Actions triggered successfully
- [ ] build-release-wheels job passed
- [ ] test-release-wheels job passed
- [ ] publish-pypi job completed
- [ ] PyPI publication verified (12 wheels present)
- [ ] crates.io publication verified (manual or CI)
- [ ] GitHub release created and visible
- [ ] docs.rs documentation available

**Phase 8: Post-Release Verification**
- [ ] Local PyPI installation test passed
- [ ] Local Rust dependency test passed

**Phase 9: Post-Release Setup**
- [ ] Beads sync completed
- [ ] [Unreleased] section cleaned (empty subsections)
- [ ] Post-release commit pushed (if any)
- [ ] Next development issues created (if planned)

---

## Reference Information

### File Locations

| File | Purpose | Version String |
|------|---------|-----------------|
| `Cargo.toml` | Root Rust package | Line 7: `version = "X.Y.Z"` |
| `src-python/Cargo.toml` | Python binding package | Line 3: `version = "X.Y.Z"` |
| `pyproject.toml` | Python project metadata | Line 7: `version = "X.Y.Z"` |
| `CHANGELOG.md` | Release notes | Line 8+: Version headers |
| `README.md` | Project overview | Various (checked for version refs) |

### GitHub Actions Workflows

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| `python-release.yml` | Tag push `v*` | Builds wheels, tests, publishes to PyPI |
| `lint.yml` | Push/PR | Rustfmt, clippy, doc checks |
| `test.yml` | Push/PR | Cargo tests |
| `python-build.yml` | Push/PR | Python wheel build test |

### Important URLs

| Resource | URL |
|----------|-----|
| GitHub Repo | https://github.com/dchud/mrrc |
| GitHub Actions | https://github.com/dchud/mrrc/actions |
| GitHub Releases | https://github.com/dchud/mrrc/releases |
| PyPI Package | https://pypi.org/project/mrrc/ |
| crates.io | https://crates.io/crates/mrrc |
| docs.rs | https://docs.rs/mrrc |
| GitHub Project Settings | https://github.com/dchud/mrrc/settings |

### Commands Quick Reference

```bash
# Pre-release verification
.cargo/check.sh

# Version checks
grep '^version' Cargo.toml
grep '^version' src-python/Cargo.toml
grep '^version' pyproject.toml

# Git operations
git tag -a vX.Y.Z -m "Release version X.Y.Z"
git push origin main
git push origin vX.Y.Z

# Manual crates.io publish
cargo publish --dry-run
cargo publish

# Manual PyPI publish
pip install twine
twine upload dist/mrrc-*.whl

# Test installation
pip install --upgrade mrrc
cargo add mrrc@X.Y.Z
```

---

## Related Documentation

- **AGENTS.md** - Development workflow and CI references
- **CHANGELOG.md** - Full release history
- **docs/README.md** - Documentation index
- **.github/workflows/** - CI/CD workflow definitions

---

**Questions or issues with this procedure?**
- Check the [Troubleshooting](#troubleshooting) section
- Review GitHub Actions logs for specific errors
- Consult AGENTS.md for development environment setup
- Create a GitHub issue if the procedure needs updating
