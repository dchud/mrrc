# MRRC Release Procedure

**Status**: Executable reference for humans and coding agents  
**Estimated Duration**: 30-45 minutes for a standard release  
**Last Updated**: 2026-02-10

This document provides a step-by-step, executable procedure for preparing, testing, and publishing a new release of MRRC (MARC Rust Crate). Follow these steps sequentially to ensure nothing is missed.

## Quick Start for Agents

To prepare a release, run with a specific version number:
```bash
# Set the version to release (e.g., 0.5.0)
export VERSION="0.5.0"

# Validate environment and dependencies
# Then follow Sections 1-6 for preparation, or Sections 1-9 for full workflow
```

**Terminology**:

- **"Prepare"** (Sections 1-6): Version updates through git tag creation (ends with tag ready to push)
- **"Publish"** (Sections 7-8): Publishing to registries (automated via GitHub Actions)
- **"Post-Release"** (Sections 9-10): Cleanup and next cycle setup

## Definition of Done: "Prepare Release X.Y.Z"

When a coding agent receives a task "prepare release version X.Y.Z", it should follow sections 1-7.2 (Preflight through Create Git Tag). The release is **prepared and ready to publish** when:

### Machine-Checkable Success Criteria

```bash
# 1. All version files updated and matching
ROOT_VER=$(sed -n '/^\[package\]/,/^\[/p' Cargo.toml | grep '^version' | sed 's/.*"\([^"]*\)".*/\1/')
[ "$ROOT_VER" = "$VERSION" ] || exit 1

# 2. CHANGELOG.md structure correct
[ "$(grep -c "## \[Unreleased\]" CHANGELOG.md)" = "1" ] || exit 1
grep -q "## \[$VERSION\]" CHANGELOG.md || exit 1

# 3. All pre-release checks passed
bash .cargo/check.sh || exit 1

# 4. Git state correct
[ "$(git rev-parse --abbrev-ref HEAD)" = "main" ] || exit 1
git diff --quiet || exit 1
git diff --cached --quiet || exit 1

# 5. Release commit exists locally
git log --oneline -1 | grep -q "chore(release): v$VERSION" || exit 1

# 6. Release tag exists locally (not pushed yet)
git rev-parse "v$VERSION" >/dev/null || exit 1

# If all above succeed, preparation is complete
echo "✓ Release v$VERSION prepared and ready to push"
```

**Status Check Command** (agent can use to verify work):
```bash
# Quickly verify all success criteria
cd "$(git rev-parse --show-toplevel)"
VERSION="0.5.0"  # Set to target version

# Run all checks
bash << 'EOF'
set -e
echo "Checking release preparation..."

# Versions
ROOT=$(sed -n '/^\[package\]/,/^\[/p' Cargo.toml | grep '^version' | sed 's/.*"\([^"]*\)".*/\1/')
[ "$ROOT" = "$VERSION" ] && echo "✓ Versions match" || (echo "✗ Version mismatch" && exit 1)

# Changelog
[ "$(grep -c "## \[Unreleased\]" CHANGELOG.md)" = "1" ] && echo "✓ Changelog structure OK" || (echo "✗ Changelog issue" && exit 1)

# Git state
[ "$(git rev-parse --abbrev-ref HEAD)" = "main" ] && echo "✓ On main branch" || (echo "✗ Wrong branch" && exit 1)
git diff --quiet && echo "✓ No unstaged changes" || (echo "✗ Unstaged changes" && exit 1)

# Commit exists
git log --oneline -1 | grep -q "chore(release)" && echo "✓ Release commit exists" || (echo "✗ Release commit missing" && exit 1)

# Tag exists
git rev-parse "v$VERSION" >/dev/null && echo "✓ Release tag created locally" || (echo "✗ Tag missing" && exit 1)

echo ""
echo "✓ Release v$VERSION is READY TO PUSH"
echo "  Next: git push origin main && git push origin v$VERSION"
EOF
```

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

## Preflight Dependencies & Setup

**Run this first to validate your environment is ready.**

### P.1 Determine Repo Root

All commands must run from the repository root. Set up the environment:

```bash
# Determine repo root and validate
REPO_ROOT="$(git rev-parse --show-toplevel)" || { echo "Not in a git repo"; exit 1; }
cd "$REPO_ROOT"
echo "Repo root: $(pwd)"
```

**Checklist**:

- [ ] You are in a git repository
- [ ] Path contains `.git`, `Cargo.toml`, `src-python/`, `docs/` directories

### P.2 Validate Required Tools

Verify all tools are installed and accessible:

```bash
# Rust toolchain
rustc --version || { echo "rustc not found"; exit 1; }
cargo --version || { echo "cargo not found"; exit 1; }

# Python and build tools
python3 --version || { echo "python3 not found"; exit 1; }
maturin --version || { echo "maturin not found"; exit 1; }

# Utility tools
git --version || { echo "git not found"; exit 1; }
```

**Expected**:

- Rust 1.71+ (see `Cargo.toml` rust-version)
- Python 3.10+
- maturin 1.0+

**Checklist**:

- [ ] All tools are installed and at expected versions

### P.3 Validate Publishing Credentials (if publishing)

PyPI publishing uses **OIDC trusted publishing** — no API token or secret is needed. The `python-release.yml` workflow has `id-token: write` permission and `pypa/gh-action-pypi-publish` auto-detects the OIDC token.

For crates.io (manual publish):

```bash
# For crates.io publication (manual as fallback)
cargo login --registry crates-io 2>&1 | head -1 || echo "Note: crates.io token check"
```

**Checklist**:

- [ ] PyPI: OIDC trusted publisher is configured at pypi.org for `dchud/mrrc`
- [ ] crates.io token is configured locally (if manual publish needed)

### P.4 Set VERSION Variable

Set the version you are releasing:

```bash
# Example: 0.5.0
export VERSION="0.5.0"

# Validate format (MAJOR.MINOR.PATCH)
echo "Releasing version: $VERSION"
```

**Checklist**:

- [ ] VERSION is set in shell (e.g., `echo $VERSION` prints `0.5.0`)
- [ ] VERSION follows SemVer format (X.Y.Z)

---

## Pre-Release Verification

Before making any changes, verify that the codebase is in a releasable state.

### 1.1 Run Full Test Suite

```bash
cd "$REPO_ROOT"
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

The VERSION variable is used throughout. Ensure `$VERSION` is set before starting (see [Preflight Dependencies](#preflight-dependencies--setup)).

### 3.1 Update Root Cargo.toml

**File**: `Cargo.toml` (in repo root)

Find the `[package]` section (should be near the top) and update the `version` field:

```bash
# Show current version
echo "=== Current Root Cargo.toml version ==="
sed -n '/^\[package\]/,/^\[/p' Cargo.toml | grep '^version'

# Update version (using sed)
sed -i.bak '/^\[package\]/,/^\[/{
  s/^version = .*/version = "'$VERSION'"/
}' Cargo.toml

# Verify
echo "=== Updated Root Cargo.toml version ==="
sed -n '/^\[package\]/,/^\[/p' Cargo.toml | grep '^version'
```

**Checklist**:

- [ ] Root Cargo.toml updated
- [ ] Version number shows `version = "X.Y.Z"` where X.Y.Z matches $VERSION
- [ ] No other section's version was changed

### 3.2 Update Python Cargo.toml

**File**: `src-python/Cargo.toml`

Update the version in the `[package]` section to match the root version:

```bash
# Show current version
echo "=== Current Python Cargo.toml version ==="
sed -n '/^\[package\]/,/^\[/p' src-python/Cargo.toml | grep '^version'

# Update version
sed -i.bak 's/^version = .*/version = "'$VERSION'"/' src-python/Cargo.toml

# Verify
echo "=== Updated Python Cargo.toml version ==="
grep '^version' src-python/Cargo.toml
```

**Checklist**:

- [ ] Python Cargo.toml updated
- [ ] Version number matches root (should show `version = "X.Y.Z"`)

### 3.3 Update pyproject.toml

**File**: `pyproject.toml` (in repo root)

Update the version in the `[project]` section:

```bash
# Show current version
echo "=== Current pyproject.toml version ==="
sed -n '/^\[project\]/,/^\[/p' pyproject.toml | grep '^version'

# Update version
sed -i.bak 's/^version = .*/version = "'$VERSION'"/' pyproject.toml

# Verify
echo "=== Updated pyproject.toml version ==="
sed -n '/^\[project\]/,/^\[/p' pyproject.toml | grep '^version'
```

**Checklist**:

- [ ] pyproject.toml updated
- [ ] Version number matches root and Python Cargo.toml

### 3.4 Verify All Versions Match

```bash
echo "=== Version Consistency Check ==="
ROOT_VER=$(sed -n '/^\[package\]/,/^\[/p' Cargo.toml | grep '^version' | head -1 | sed 's/.*= "\([^"]*\)".*/\1/')
PYTHON_VER=$(grep '^version' src-python/Cargo.toml | head -1 | sed 's/.*= "\([^"]*\)".*/\1/')
PYPROJECT_VER=$(sed -n '/^\[project\]/,/^\[/p' pyproject.toml | grep '^version' | head -1 | sed 's/.*= "\([^"]*\)".*/\1/')

echo "Root Cargo.toml: $ROOT_VER"
echo "Python Cargo.toml: $PYTHON_VER"
echo "pyproject.toml: $PYPROJECT_VER"
echo "Expected: $VERSION"

# Exit with failure if any don't match
if [ "$ROOT_VER" != "$VERSION" ] || [ "$PYTHON_VER" != "$VERSION" ] || [ "$PYPROJECT_VER" != "$VERSION" ]; then
  echo "ERROR: Version mismatch detected!"
  exit 1
fi
echo "✓ All versions match"
```

**Checklist**:

- [ ] All three version numbers are identical
- [ ] All match the $VERSION variable
- [ ] Script exits with success (code 0)

---

## Update Changelog

**File**: `CHANGELOG.md` (in repo root)

The changelog requires a deterministic transformation: the current `[Unreleased]` section becomes the new release, and a fresh `[Unreleased]` is created above it.

### 4.1 Validate Changelog Structure

Ensure the file starts with `## [Unreleased]`:

```bash
head -10 CHANGELOG.md
```

**Expected output**:
```
## [Unreleased]

### Added
...
```

**Checklist**:

- [ ] File starts with `## [Unreleased]`
- [ ] There is exactly one `## [Unreleased]` section (check with: `grep -c "## \[Unreleased\]" CHANGELOG.md`)

### 4.2 Create Release Entry

The transformation adds a dated entry before the new Unreleased. Use this procedure:

```bash
# Get today's date in ISO 8601 format
RELEASE_DATE=$(date +%Y-%m-%d)

# Create a temporary file with the new structure
{
  # New empty [Unreleased] section
  echo "## [Unreleased]"
  echo ""
  echo "### Added"
  echo ""
  echo "### Changed"
  echo ""
  echo "### Fixed"
  echo ""
  echo "### Performance"
  echo ""
  echo "### Documentation"
  echo ""
  
  # Replace old [Unreleased] with versioned header and keep the rest
  sed '1,/^## \[Unreleased\]/d' CHANGELOG.md | sed "1s/^/## [$VERSION] - $RELEASE_DATE\n\n/"
} > CHANGELOG.md.tmp

# Backup and replace
mv CHANGELOG.md CHANGELOG.md.bak
mv CHANGELOG.md.tmp CHANGELOG.md
```

### 4.3 Verify Changelog Structure

Validate the transformation succeeded:

```bash
echo "=== Unreleased Sections ==="
grep -n "## \[Unreleased\]" CHANGELOG.md

echo ""
echo "=== Release Sections ==="
grep -n "## \[$VERSION\]" CHANGELOG.md

echo ""
echo "=== First 20 lines ==="
head -20 CHANGELOG.md
```

**Expected**:

- Exactly one `## [Unreleased]` (at the top, line ~1)
- Exactly one `## [X.Y.Z] - YYYY-MM-DD` (around line ~9-11)
- Empty subsections under `[Unreleased]` (Added, Changed, Fixed, Performance, Documentation)
- Release content under the versioned section

**Checklist**:

- [ ] One `## [Unreleased]` section at the top
- [ ] One `## [$VERSION] - YYYY-MM-DD` section below it
- [ ] Release date is today (ISO 8601 format)
- [ ] No duplicate `## [Unreleased]` sections
- [ ] Content is properly categorized

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

- `docs/getting-started/installation.md` - Update `mrrc = "X.Y"` in the Rust Cargo.toml example
- `docs/getting-started/quickstart-rust.md` - Update `mrrc = "X.Y"` in the Add Dependency example
- `docs/guides/migration-from-pymarc.md` - Add section for new release
- `docs/guides/performance-tuning.md` - Update performance baselines if optimizations were made
- `docs/contributing/architecture.md` - Note architectural changes if any
- `docs/design/` - Update design documents if decisions changed
- Any docs with "Supported in X.Y+" style language

**Checklist**:

- [ ] docs/ directory scanned for version references
- [ ] `docs/getting-started/installation.md` Rust dependency version updated
- [ ] `docs/getting-started/quickstart-rust.md` Rust dependency version updated
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

### 6.1 Clean Up Backup Files

Remove the `.bak` files created by sed during version updates:

```bash
rm -f Cargo.toml.bak src-python/Cargo.toml.bak pyproject.toml.bak CHANGELOG.md.bak
```

**Checklist**:

- [ ] No `.bak` files remain in repository root or src-python/

---

## Git Operations

Before starting, ensure you are on the `main` branch with a clean working tree.

### 7.0 Validate Git State

```bash
# Verify branch
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [ "$CURRENT_BRANCH" != "main" ]; then
  echo "ERROR: Not on main branch (on: $CURRENT_BRANCH)"
  exit 1
fi

# Ensure up to date with remote
git fetch origin

# Check for uncommitted changes
if ! git diff --quiet || ! git diff --cached --quiet; then
  echo "ERROR: Working tree is not clean"
  git status
  exit 1
fi

# Verify origin is set correctly
ORIGIN_URL=$(git config --get remote.origin.url)
echo "Remote origin: $ORIGIN_URL"
```

**Checklist**:

- [ ] On `main` branch
- [ ] Working tree is clean (no staged or unstaged changes)
- [ ] Remote origin is correct
- [ ] Script exits with code 0

### 7.1 Commit Version and Changelog Updates

Stage and commit the updated files:

```bash
# Stage the exact files we modified
git add Cargo.toml src-python/Cargo.toml pyproject.toml CHANGELOG.md

# Verify staging
echo "=== Staged changes ==="
git diff --cached --stat

# Commit with clear message
git commit -m "chore(release): v$VERSION

- Update Cargo.toml, src-python/Cargo.toml, pyproject.toml to $VERSION
- Update CHANGELOG.md with release notes and date ($RELEASE_DATE)
- All pre-release checks passing, ready for publication"

# Show the commit
echo "=== New commit ==="
git log --oneline -1
```

**Checklist**:

- [ ] Only these 4 files staged: Cargo.toml, src-python/Cargo.toml, pyproject.toml, CHANGELOG.md
- [ ] Commit message includes version number
- [ ] `git log --oneline -1` shows the new commit
- [ ] Git status shows nothing to commit

### 7.2 Create Git Tag

Create an annotated tag for the release. **The `v` prefix is required** for GitHub Actions to trigger.

```bash
# Verify tag doesn't already exist
if git rev-parse "v$VERSION" >/dev/null 2>&1; then
  echo "ERROR: Tag v$VERSION already exists"
  exit 1
fi

# Create annotated tag (signed optional, but not required for CI)
git tag -a "v$VERSION" -m "Release version $VERSION"

# Verify
echo "=== New tag ==="
git tag -l "v$VERSION" -n1
```

**Checklist**:

- [ ] Tag does not already exist
- [ ] Tag created with `v` prefix (e.g., `v0.5.0`)
- [ ] Tag message is "Release version X.Y.Z"
- [ ] `git tag -l v$VERSION -n1` shows the tag

### 7.3 Push Commit and Tag to Origin

Push the commit and tag in the correct order:

```bash
# Push commit to main
echo "Pushing commit to origin/main..."
git push origin main

# Verify commit pushed
echo "=== Verifying commit pushed ==="
git log --oneline -1
git rev-list --left-right --count origin/main...HEAD
# (should show: 0	0)

# Push tag
echo "Pushing tag v$VERSION to origin..."
git push origin "v$VERSION"

# Verify tag pushed
echo "=== Verifying tag pushed ==="
git ls-remote origin | grep "refs/tags/v$VERSION"

# Final status check
echo "=== Final git status ==="
git status
```

**Expected output**:

- Commit is up to date with origin/main
- Tag appears in `git ls-remote origin`
- `git status` shows nothing to commit

**Checklist**:

- [ ] Commit pushed to origin/main
- [ ] Tag pushed to origin
- [ ] `git status` shows "Your branch is up to date with 'origin/main'."
- [ ] Tag is visible in GitHub: https://github.com/dchud/mrrc/tags

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
1. **build-release-wheels** job - Builds Python wheels for 5×3.10-3.14 on macOS/Ubuntu/Windows
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
cd "$REPO_ROOT"
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

Should show your new version with wheels for Python 3.10, 3.11, 3.12, 3.13, 3.14 across macOS, Ubuntu, and Windows.

**Expected**: 15 wheels total (5 Python versions × 3 platforms)

**Checklist**:

- [ ] Version appears on PyPI
- [ ] All wheels are present (15 total)
- [ ] Documentation is correct

### 8.5 Verify GitHub Release Created

GitHub Actions automatically creates a GitHub Release with wheels attached and release notes extracted from `CHANGELOG.md` (the section matching the tag version).

Go to: https://github.com/dchud/mrrc/releases

Should see your new release with:

- Tag name: `vX.Y.Z`
- Release title (auto-generated from tag)
- Release notes body (auto-extracted from CHANGELOG.md)
- Wheels attached from `dist/`
- Status: Not a draft, not a prerelease (unless version contains alpha/beta)

**Checklist**:

- [ ] Release appears on GitHub
- [ ] Release notes match the CHANGELOG section for this version
- [ ] Wheels are attached
- [ ] Release status is correct (not draft)

---

## Post-Release Verification

### 9.1 Test Installation from PyPI (Python)

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

### 9.2 Test Installation from crates.io (Rust)

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

### 9.3 Verify docs.rs Documentation

Go to: https://docs.rs/mrrc/latest/mrrc/

Docs should be built and available. If not, they build automatically within a few minutes.

**Expected**: Latest version's documentation is current and complete

**Checklist**:

- [ ] docs.rs has documentation for new version
- [ ] Documentation is complete (no build errors)

---

## Post-Release Setup

### 10.1 Verify Sync and Beads

Sync any issue tracking changes:

```bash
bd sync
git status
```

No changes should be required unless you manually updated issue statuses.

### 10.2 Create Post-Release Issue (Optional)

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

### 10.3 Begin Next Development Cycle

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

### 10.4 Commit Post-Release Changes (if any)

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

### 11.1 Immediate: Yank on crates.io (Rust)

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

### 11.2 Immediate: Delete Release on PyPI (Python)

If the release is newer and hasn't been heavily downloaded:

**Manual process** (no CLI tool):
1. Go to https://pypi.org/project/mrrc/
2. Click on version number
3. Click "Yank release" or "Delete release"
4. Confirm

**Note**: PyPI supports "yanking" (like crates.io) which is preferred to deletion.

**Announcement**: Same as above.

### 11.3 Revert Git Tag

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

### 11.4 Fix and Re-release

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
   - Rust toolchain version mismatch
   - Python version not available on that platform
3. Fix the issue in the workflow file (`.github/workflows/python-release.yml`)
4. Delete the failed tag: `git tag -d vX.Y.Z && git push origin :refs/tags/vX.Y.Z`
5. Commit the workflow fix: `git commit -am "fix: python-release workflow"`
6. Re-tag and push: `git tag -a vX.Y.Z -m "Release version X.Y.Z" && git push origin vX.Y.Z`

### PyPI Publication Fails

**Problem**: Wheels build successfully but PyPI publish job fails

**Steps**:
1. Verify OIDC trusted publisher is configured at pypi.org for `dchud/mrrc` (workflow: `python-release.yml`)
2. Ensure the `publish-pypi` job has `id-token: write` permission
3. Check that the workflow is using `pypa/gh-action-pypi-publish` without a `password:` parameter (OIDC is auto-detected)
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
   cd "$REPO_ROOT"
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
- [ ] PyPI publication verified (15 wheels present)
- [ ] crates.io publication verified (manual or CI)
- [ ] GitHub release created with changelog notes
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
