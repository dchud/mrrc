# MRRC Release Procedure

**Status**: Executable reference for humans and coding agents  
**Estimated Duration**: 30-45 minutes for a standard release  
**Last Updated**: 2026-02-10

This document provides a step-by-step, executable procedure for preparing, testing, and publishing a new release of MRRC. Follow these steps sequentially to ensure nothing is missed.

## Quick Start for Agents

To prepare a release, run with a specific version number:
```bash
# Set the version to release (e.g., 0.5.0)
export VERSION="0.5.0"

# Validate environment and dependencies
# Then follow Sections 1-6 for preparation, or Sections 1-9 for full workflow
```

**Terminology**:

- **"Prepare"** (Sections 1–6, 7.0–7.2): version and changelog updates committed on a `release/vX.Y.Z` branch, with the release PR opened and its required checks green
- **"Publish"** (Sections 7.3–8): merge the release PR, tag the merged commit, and push the tag to trigger publication (registry steps automated via GitHub Actions)
- **"Post-Release"** (Sections 9-10): Cleanup and next cycle setup

## Definition of Done: "Prepare Release X.Y.Z"

When a coding agent receives a task "prepare release version X.Y.Z", it should follow sections 1–7.2 (Preflight through opening the release PR). The release is **prepared and ready to publish** when:

### Machine-Checkable Success Criteria

```bash
# 1. Workspace version updated (single source: [workspace.package] in Cargo.toml)
WS_VER=$(sed -n '/^\[workspace\.package\]/,/^\[/p' Cargo.toml | grep '^version' | sed 's/.*"\([^"]*\)".*/\1/')
[ "$WS_VER" = "$VERSION" ] || exit 1

# 2. CHANGELOG.md structure correct
[ "$(grep -c "## \[Unreleased\]" CHANGELOG.md)" = "1" ] || exit 1
grep -q "## \[$VERSION\]" CHANGELOG.md || exit 1
bash scripts/lint-changelog.sh || exit 1

# 3. All pre-release checks passed
bash .cargo/check.sh || exit 1

# 4. Git state correct (on the release branch, working tree clean after commit)
[ "$(git rev-parse --abbrev-ref HEAD)" = "release/v$VERSION" ] || exit 1
git diff --quiet || exit 1
git diff --cached --quiet || exit 1

# 5. Release commit exists on the branch
git log --oneline -1 | grep -q "chore(release): v$VERSION" || exit 1

# 6. Release PR is open against main (the tag is created later, after merge)
[ "$(gh pr view "release/v$VERSION" --json state -q .state 2>/dev/null)" = "OPEN" ] || exit 1

# If all above succeed, preparation is complete
echo "✓ Release v$VERSION prepared — PR open and ready to merge"
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

# Version (single source: [workspace.package] in Cargo.toml)
WS=$(sed -n '/^\[workspace\.package\]/,/^\[/p' Cargo.toml | grep '^version' | sed 's/.*"\([^"]*\)".*/\1/')
[ "$WS" = "$VERSION" ] && echo "✓ Versions match" || (echo "✗ Version mismatch" && exit 1)

# Changelog
[ "$(grep -c "## \[Unreleased\]" CHANGELOG.md)" = "1" ] && echo "✓ Changelog structure OK" || (echo "✗ Changelog issue" && exit 1)
bash scripts/lint-changelog.sh && echo "✓ Changelog lint OK" || (echo "✗ Changelog lint failed" && exit 1)

# Git state
[ "$(git rev-parse --abbrev-ref HEAD)" = "release/v$VERSION" ] && echo "✓ On release branch" || (echo "✗ Wrong branch" && exit 1)
git diff --quiet && echo "✓ No unstaged changes" || (echo "✗ Unstaged changes" && exit 1)

# Commit exists
git log --oneline -1 | grep -q "chore(release)" && echo "✓ Release commit exists" || (echo "✗ Release commit missing" && exit 1)

# Release PR open
[ "$(gh pr view "release/v$VERSION" --json state -q .state 2>/dev/null)" = "OPEN" ] && echo "✓ Release PR open" || (echo "✗ Release PR not open" && exit 1)

echo ""
echo "✓ Release v$VERSION is PREPARED — PR open and ready to merge"
echo "  Next (maintainer): merge the PR, then on an updated main: git tag -a v$VERSION -m 'Release version $VERSION' && git push origin v$VERSION"
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

- Rust at or above the MSRV declared in `Cargo.toml` (`rust-version`); the repo's `rust-toolchain.toml` pin always satisfies it
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

- [ ] All recent features are listed in `[Unreleased]` section
- [ ] All breaking changes are documented with migration guidance
- [ ] All bug fixes are noted
- [ ] All performance improvements documented
- [ ] Known limitations are listed if applicable

**Breaking Changes Specific Check**:
If this release includes breaking changes:

- [ ] **REQUIRED**: Migration guide exists in `[Unreleased]` or referenced
- [ ] **REQUIRED**: Deprecation notices were given in previous release (if applicable)
- [ ] **REQUIRED**: Major version bump scheduled (e.g., 0.x → 1.0)

### 1.4 Lint Changelog

Run the CHANGELOG lint script to catch structural drift in the `[Unreleased]`
block. `.cargo/check.sh` runs this on every PR, so normally it will already
have passed — running it here is belt-and-braces before tagging a release.

```bash
bash scripts/lint-changelog.sh
```

The script fails if:

- A `###` subsection heading appears more than once. Topic-grouped
  `### Added — <topic>` entries with distinct topics are allowed; a bare
  `### Added` mixed with `### Added — <topic>` entries is flagged as
  drift (this is the shape the lint was written to prevent).
- Subsections are out of Keep-a-Changelog order. Canonical order is
  `Breaking, Added, Changed, Deprecated, Removed, Fixed, Security,
  Dependencies`; `### Added — <topic>` variants all rank as `Added`.
- Any line in `[Unreleased]` exceeds 100 columns (fenced code blocks
  excepted). The working target is ~72–80 columns for bullet content,
  matching the existing wrap style.

It warns (without failing) on `###` heading names that are not in the
canonical set or the `### Added — <topic>` form.

**Checklist**:

- [ ] `scripts/lint-changelog.sh` exits 0
- [ ] No drift warnings left unaddressed

### 1.5 Review CHANGELOG Entries for Concision

The lint in 1.4 checks structure and line length but not brevity. Before
tagging, read the `[Unreleased]` section by eye against the **Entry
length** convention in [Update Changelog](#update-changelog): each bullet
is one short paragraph (~25–40 words, ≤4–6 wrapped lines) that leads with
the user-visible *what* and why a user would care.

Condense any entry that drifted during the cycle. The usual offenders,
which belong in the commit message body or PR description rather than the
CHANGELOG:

- Per-file enumerations ("touched X, Y, Z").
- Error-count tables, before/after metrics, audit deltas.
- "Verified by …" / test-coverage notes.
- Internal process labels (bead IDs, milestone names, planning phases) —
  linked PR/issue numbers are fine as permanent provenance.

```bash
# Read the [Unreleased] section to review entry-by-entry
sed -n '/## \[Unreleased\]/,/## \[/p' CHANGELOG.md
```

**Checklist**:

- [ ] Each `[Unreleased]` bullet is a concise, user-facing one-paragraph entry
- [ ] No per-file lists, metrics tables, audit deltas, or "verified by" notes
- [ ] No process labels (bead IDs, milestones, phases) in entries

---

## Version Number Selection

MRRC currently uses **Semantic Versioning (SemVer)**: `MAJOR.MINOR.PATCH`

### 2.1 Determine Version Increment

**Use the following rules**:

- **PATCH** (e.g., 0.4.0 → 0.4.1): Bug fixes, documentation updates, non-breaking performance improvements
- **MINOR** (e.g., 0.4.0 → 0.5.0): New features, backward-compatible API additions
- **MAJOR** (e.g., 0.4.0 → 1.0.0): Breaking changes, significant redesigns

**Current version**: Check `[workspace.package]` in `Cargo.toml`:

```bash
sed -n '/^\[workspace\.package\]/,/^\[/p' Cargo.toml | grep '^version'
```

### 2.2 Document Version Rationale

Note the reason for the chosen version increment in your release notes/commit message (e.g., "minor: added query DSL feature" or "patch: fixed field ordering bug").

---

## Update Configuration Files

The version is declared once, in `[workspace.package]` in the root `Cargo.toml`. Both crates inherit it (`version.workspace = true`), and `pyproject.toml` declares `dynamic = ["version"]`, so maturin reads the same value. One edit updates everything.

The VERSION variable is used throughout. Ensure `$VERSION` is set before starting (see [Preflight Dependencies](#preflight-dependencies-setup)).

### 3.1 Update the Workspace Version

**File**: `Cargo.toml` (in repo root)

Update the `version` field in the `[workspace.package]` section:

```bash
# Show current version
echo "=== Current workspace version ==="
sed -n '/^\[workspace\.package\]/,/^\[/p' Cargo.toml | grep '^version'

# Update version (using sed)
sed -i.bak '/^\[workspace\.package\]/,/^\[/{
  s/^version = .*/version = "'$VERSION'"/
}' Cargo.toml

# Refresh the committed Cargo.lock so it records the new crate versions
cargo update --workspace --quiet

# Verify
echo "=== Updated workspace version ==="
sed -n '/^\[workspace\.package\]/,/^\[/p' Cargo.toml | grep '^version'
```

**Checklist**:

- [ ] `[workspace.package]` shows `version = "X.Y.Z"` where X.Y.Z matches $VERSION
- [ ] No other section's version was changed
- [ ] Cargo.lock shows both `mrrc` and `mrrc-python` at the new version

### 3.2 Verify Both Crates Inherit the New Version

```bash
echo "=== Version Consistency Check ==="
cargo metadata --format-version 1 --no-deps \
  | jq -r '.packages[] | "\(.name) \(.version)"'
echo "Expected: mrrc $VERSION and mrrc-python $VERSION"

cargo metadata --format-version 1 --no-deps \
  | jq -e --arg v "$VERSION" 'all(.packages[]; .version == $v)' >/dev/null \
  && echo "✓ All versions match" \
  || { echo "ERROR: Version mismatch detected!"; exit 1; }
```

**Checklist**:

- [ ] Both crates report the new version
- [ ] Script exits with success (code 0)

### 3.3 Sweep Version-Pinned Prose

The workspace version bump does not touch prose that names a specific version. Update these by hand to match `$VERSION`:

- `README.md` — the roadmap line naming the current release (e.g. "Version 0.8.2 is suitable for testing")
- `docs/getting-started/installation.md` — the `mrrc = "0.X"` dependency example

```bash
rg -n '0\.[0-9]+\.[0-9]+|mrrc = "0\.' README.md docs/getting-started/installation.md
```

**Checklist**:

- [ ] No prose claims an older version than `$VERSION`
- [ ] The installation dependency example matches the new minor line

---

## Update Changelog

**File**: `CHANGELOG.md` (in repo root)

The changelog requires a deterministic transformation: the current `[Unreleased]` section becomes the new release, and a fresh `[Unreleased]` is created above it.

### CHANGELOG Conventions

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
Contributors writing entries into `[Unreleased]` during a release cycle
should follow these conventions so `scripts/lint-changelog.sh` (wired into
`.cargo/check.sh`) stays quiet:

- **Subsection order** (within any version block):
  `Breaking` (optional top callout), `Added`, `Changed`, `Deprecated`,
  `Removed`, `Fixed`, `Security`, `Dependencies`.
- **Topic-grouped Added sections** are allowed: `### Added — <topic>`
  (e.g., `### Added — expanded property-test suite`). Use them when a
  single release has multiple distinct feature areas that each warrant
  narrative framing. Don't mix a bare `### Added` flat list with
  `### Added — <topic>` subsections in the same block.
- **Line wrap** target is ~72–80 columns for bullet content, with
  two-space continuation indent under the `-`. The lint fails above 100
  columns. Fenced code blocks are exempt; inline markdown links are not,
  so wrap bullets that contain links so the URL lands on its own
  continuation line if needed.
- **Entry length.** Each bullet is one short paragraph (~25–40 words,
  ≤4–6 wrapped lines). Lead with the user-visible *what* and why a
  user would care. Per-file enumerations, error-count tables,
  before/after metrics, audit deltas, and "verified by …" notes
  belong in the commit message body and PR description, not the
  CHANGELOG. The lint script doesn't enforce this — it's a
  reviewer-eyeball convention.
- **No process labels.** Don't reference internal bead IDs
  (`bd-XXXX`), milestone names, or planning phases in entries.
  Linked PR/issue numbers are fine (they're permanent provenance);
  bead IDs are project-tracking artifacts and don't help users
  reading "what's new in X.Y.Z". (This is a project-wide rule for
  persistent artifacts, not just CHANGELOG.)

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

Before committing the version changes and opening the release PR, perform one final verification pass.

### 6.0 Final Verification

```bash
# 1. Verify both crates report the workspace version
echo "=== Version Check ==="
cargo metadata --format-version 1 --no-deps | jq -r '.packages[] | "\(.name) \(.version)"'

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

- [ ] Both crates report the new version (inherited from `[workspace.package]`)
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
rm -f Cargo.toml.bak CHANGELOG.md.bak
```

**Checklist**:

- [ ] No `.bak` files remain in the repository root

---

## Git Operations

`main` is protected by a branch ruleset with required status checks, so the release commit **cannot be pushed to `main` directly** — it lands through a pull request, and the tag is created on `main` only **after** that PR merges. (Tag pushes to `refs/tags/*` are not gated.) Split of responsibility: an agent preparing the release runs Sections 1–7.2, ending with the release PR open and its checks green; the maintainer runs 7.3 (merge, tag, push tag), which triggers publication.

### 7.0 Create the Release Branch

You reach this point on `main` with the version/changelog edits from Sections 3–5 in your working tree (not yet committed). Confirm the starting state, then move those edits onto a release branch.

```bash
# Verify you started from main, up to date with the remote
git fetch origin
[ "$(git rev-parse --abbrev-ref HEAD)" = "main" ] || { echo "ERROR: not on main"; exit 1; }
# Local main must not be ahead of origin (the bump edits are unstaged, not committed)
[ "$(git rev-list --count origin/main..HEAD)" = "0" ] || { echo "ERROR: local main is ahead of origin"; exit 1; }

# Verify origin
echo "Remote origin: $(git config --get remote.origin.url)"

# Move the version/changelog edits onto a release branch
git checkout -b "release/v$VERSION"
```

**Checklist**:

- [ ] Started from an up-to-date `main`
- [ ] On `release/v$VERSION` with the Section 3–5 edits carried over
- [ ] Remote origin is correct

### 7.1 Commit Version and Changelog Updates

Stage and commit the updated files:

```bash
# Stage the exact files we modified
git add Cargo.toml Cargo.lock CHANGELOG.md

# Verify staging
echo "=== Staged changes ==="
git diff --cached --stat

# Commit with clear message
git commit -m "chore(release): v$VERSION

- Update workspace version to $VERSION (both crates and the Python package inherit it)
- Update CHANGELOG.md with release notes and date ($RELEASE_DATE)
- All pre-release checks passing, ready for publication"

# Show the commit
echo "=== New commit ==="
git log --oneline -1
```

**Checklist**:

- [ ] Only these 3 files staged: Cargo.toml, Cargo.lock, CHANGELOG.md
- [ ] Commit message includes version number
- [ ] `git log --oneline -1` shows the new commit
- [ ] Git status shows nothing to commit

### 7.2 Open the Release PR

The release commit reaches `main` through a pull request that passes the required status checks. **Do not tag yet** — the tag must point at the commit as it lands on `main` after merge.

```bash
# Push the release branch and open the PR
git push -u origin "release/v$VERSION"
gh pr create --base main --title "Release v$VERSION" \
  --body "Workspace version bump to $VERSION and CHANGELOG finalization. Merging this, then tagging the merged commit, triggers publication."

# Watch the required checks
gh pr checks --watch
```

**This is the end of "prepare."** An agent stops here and hands off to the maintainer; the required checks must be green before the PR is merged.

**Checklist**:

- [ ] Release branch pushed to origin
- [ ] PR opened against `main`, titled `Release v$VERSION`
- [ ] All required status checks pass on the PR

### 7.3 Merge, Tag, and Push the Tag (maintainer)

After review, the maintainer merges the release PR and creates the tag on the merged commit. **Tagging triggers publication**, so it happens only once the bump is on `main`.

```bash
# Merge the release PR (squash) and update local main
gh pr merge "release/v$VERSION" --squash --delete-branch
git checkout main
git pull --ff-only

# Confirm the release commit is on main
git log --oneline -1            # expect: chore(release): v$VERSION

# Create the annotated tag on main's release commit. The `v` prefix is required
# for GitHub Actions to trigger; tag pushes are not gated by the ruleset.
git rev-parse "v$VERSION" >/dev/null 2>&1 && { echo "ERROR: tag v$VERSION already exists"; exit 1; }
git tag -a "v$VERSION" -m "Release version $VERSION"
git push origin "v$VERSION"

# Verify the tag pushed
git ls-remote origin | grep "refs/tags/v$VERSION"
```

**Checklist**:

- [ ] Release PR merged; local `main` fast-forwarded to include `chore(release): v$VERSION`
- [ ] Tag created on `main`'s release commit with the `v` prefix
- [ ] Tag pushed to origin
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
1. **build-release-wheels** job - Builds native wheels for Python 3.10-3.14 on macOS/Ubuntu/Windows (15
   wheels; the macOS wheels are universal2, covering arm64 and Intel)
2. **build-cross-linux-wheels** job - Cross-compiles aarch64 and i686 Linux wheels for Python 3.10-3.14 (10 wheels)
3. **build-sdist** / **test-sdist** jobs - Build the source distribution and prove it compiles and
   imports in a clean environment (the fallback for platforms outside the wheel matrix)
4. **test-release-wheels** / **validate-cross-wheels** / **smoke-test-cross-wheels** jobs - Test
   native wheels; validate cross-compiled wheel tags; install and import one representative
   cross-compiled wheel per target (aarch64 under QEMU, i686 in a 32-bit container)
5. **publish-pypi** job - Publishes all wheels plus the sdist to PyPI (`skip-existing`, so a re-run
   after a partial failure uploads only the missing files)
6. **github-release** job - Creates the GitHub Release with artifacts and CHANGELOG notes

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

Should show your new version with wheels for Python 3.10, 3.11, 3.12, 3.13, 3.14 across macOS
(universal2: arm64 + Intel), Ubuntu, and Windows, plus cross-compiled aarch64 and i686 Linux
wheels, plus the source distribution.

**Expected**: 26 files total (25 wheels: 5 Python versions × 3 native platforms + 5 × 2
cross-compiled Linux targets; plus 1 sdist)

**Checklist**:

- [ ] Version appears on PyPI
- [ ] All files are present (25 wheels + 1 sdist)
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

Beads changes ride into `main` inside PRs (the release PR carries any closures made during the release), so the DB and `main` are normally already in sync. Confirm:

```bash
br sync --status
```

If it reports the DB is ahead, those pending changes will flush into your next PR's `.beads/issues.jsonl` — never a direct push to `main`.

### 10.2 Create Post-Release Issue (Optional)

If there are known issues or follow-up work:

```bash
br create "Post-release items for X.Y.Z" \
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

After release, update the `[Unreleased]` section to reflect the next development direction.

**File**: `CHANGELOG.md`

The `[Unreleased]` section should remain mostly empty with just subsection headers:

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
2. Create new issues for planned work: `br create "Feature: ..." -p 2`
3. Link major items in README roadmap or docs/RELEASES.md (but NOT in CHANGELOG `[Unreleased]`)
4. The `[Unreleased]` section fills up organically as work is completed during development

**Why?** CHANGELOG documents what was released, not what's planned. Planning belongs in the issue tracker (beads).

**Checklist**:

- [ ] `[Unreleased]` section has empty subsections ready for new content
- [ ] Major planned work items created as beads issues (not hardcoded in CHANGELOG)
- [ ] Issue tracker (beads) reflects the development roadmap
- [ ] README or docs reflect high-level roadmap if needed

### 10.4 Commit Post-Release Changes (if any)

Only commit if you made changelog changes. Like every change to `main`, this lands through a PR (direct pushes are blocked).

```bash
git status
# If only CHANGELOG.md changed:
git checkout -b "chore/post-v$VERSION-changelog"
git add CHANGELOG.md
git commit -m "docs: clear [Unreleased] section after v$VERSION release"
git push -u origin "chore/post-v$VERSION-changelog"
gh pr create --base main --title "Clear [Unreleased] after v$VERSION" \
  --body "Reset the CHANGELOG [Unreleased] section for the next cycle."
# Merge after the required checks pass.
```

**Checklist**:

- [ ] No uncommitted changes
- [ ] Post-release changelog reset (if needed) landed on `main` via PR

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
5. Land the workflow fix on `main` through a PR (direct pushes are blocked; see Section 7)
6. Once the fix is on `main`, re-tag and push: `git tag -a vX.Y.Z -m "Release version X.Y.Z" && git push origin vX.Y.Z`

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

### Version Mismatch Between Crate and Wheel

**Problem**: A published artifact reports a version different from `[workspace.package]`

**Steps**:
1. The version has a single source: `[workspace.package]` in the root `Cargo.toml`. Both crates inherit it and `pyproject.toml` is `dynamic = ["version"]`, so a mismatch means the artifact was built from a stale checkout or a tag pointing at the wrong commit.
2. If mismatch exists:
   - Fix the workspace version
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
- [ ] `[Unreleased]` section is complete and accurate
- [ ] Breaking changes checked (if any)

**Phase 2: Version Number Selection**

- [ ] Version number determined (SemVer rationale noted)
- [ ] Current version identified from Cargo.toml

**Phase 3: Configuration File Updates**

- [ ] `[workspace.package]` version in Cargo.toml updated
- [ ] Cargo.lock refreshed (`cargo update --workspace`)
- [ ] Both crates report the new version (`cargo metadata`)

**Phase 4: Changelog & Documentation**

- [ ] CHANGELOG.md: `[Unreleased]` → `[X.Y.Z]`
- [ ] New `[Unreleased]` section created
- [ ] README.md reviewed and updated
- [ ] docs/ directory scanned for version refs
- [ ] Example code compiles and runs
- [ ] AGENTS.md reviewed for stale guidance

**Phase 5: Final Sanity Check**

- [ ] Version consistency verified (both crates inherit the workspace version)
- [ ] Changelog shows new version with correct date
- [ ] All tests pass (full `.cargo/check.sh`)
- [ ] Git status is clean (only config files changed)

**Phase 6: Git Operations**

- [ ] Version update commit created on a `release/vX.Y.Z` branch
- [ ] Release PR opened and required checks green
- [ ] Release PR merged to `main` (maintainer)
- [ ] Git tag created with `v` prefix on the merged commit and pushed

**Phase 7: Publishing & Verification**

- [ ] GitHub Actions triggered successfully
- [ ] build-release-wheels job passed
- [ ] test-release-wheels job passed
- [ ] publish-pypi job completed
- [ ] PyPI publication verified (25 wheels + 1 sdist present)
- [ ] crates.io publication verified (manual or CI)
- [ ] GitHub release created with changelog notes
- [ ] docs.rs documentation available

**Phase 8: Post-Release Verification**

- [ ] Local PyPI installation test passed
- [ ] Local Rust dependency test passed

**Phase 9: Post-Release Setup**

- [ ] Beads sync completed
- [ ] `[Unreleased]` section cleaned (empty subsections)
- [ ] Post-release commit pushed (if any)
- [ ] Next development issues created (if planned)

---

## Reference Information

### File Locations

| File | Purpose | Version String |
|------|---------|-----------------|
| `Cargo.toml` | Workspace root | `[workspace.package]` `version = "X.Y.Z"` — the single source |
| `src-python/Cargo.toml` | Python binding package | `version.workspace = true` (inherited, no edit) |
| `pyproject.toml` | Python project metadata | `dynamic = ["version"]` (read by maturin, no edit) |
| `CHANGELOG.md` | Release notes | Version headers |
| `README.md` | Project overview | Various (checked for version refs) |

### GitHub Actions Workflows

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| `python-release.yml` | Tag push `v*`; manual dispatch | Builds wheels, tests, publishes to PyPI; dispatch is a dry run, or a TestPyPI rehearsal with `publish-target=testpypi` |
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

# Version checks (single source: [workspace.package] in Cargo.toml)
sed -n '/^\[workspace\.package\]/,/^\[/p' Cargo.toml | grep '^version'
cargo metadata --format-version 1 --no-deps | jq -r '.packages[] | "\(.name) \(.version)"'

# Git operations (main is PR-gated; tag after the release PR merges)
git checkout -b release/vX.Y.Z          # carries the version/changelog edits
git add Cargo.toml Cargo.lock CHANGELOG.md && git commit -m "chore(release): vX.Y.Z"
git push -u origin release/vX.Y.Z && gh pr create --base main --title "Release vX.Y.Z"
# after the PR merges and you are on an updated main:
git tag -a vX.Y.Z -m "Release version X.Y.Z" && git push origin vX.Y.Z

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
