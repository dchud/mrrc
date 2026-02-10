# Memory Safety Runbook

Operational procedures for maintaining memory safety infrastructure in MRRC.

## Overview

This runbook covers:
- Suppression file review and maintenance
- Adding new suppressions
- Escalation procedures
- Quarterly review checklist

**Audience**: MRRC maintainers and release managers  
**Frequency**: Quarterly review + ad-hoc as issues arise

## Suppression File Management

### Location

- **File**: `.cargo/asan_suppressions.txt`
- **Format**: LSAN (Leak Sanitizer) suppression syntax
- **Version control**: Tracked in git (full audit trail)

### File Structure

```
# Category Header
# ==============

# Description of suppression
# Why it's safe (technical explanation)
# Reference: URL or issue tracker

leak:pattern_name
leak:another_pattern
```

### When to Update

**Add a suppression when:**
1. ASAN detects a real issue (not a bug)
2. Issue is in Python, PyO3, libc, or dependency
3. Issue is confirmed safe (researched, documented)
4. Issue has clear technical rationale

**DO NOT add suppressions for:**
- "definitely lost" leaks in mrrc code
- Unknown patterns (investigate first!)
- Patterns that grow with test iterations
- Anything in src/ or src-python/ without full investigation

## Quarterly Review Procedure

Perform this quarterly (every 3 months) or after major dependency updates.

### Checklist

```markdown
### Q1/Q2/Q3/Q4 (YYYY) - Memory Safety Review

Date: YYYY-MM-DD
Reviewer: [name]
Status: In Progress / Complete

#### Pre-Review Tasks
- [ ] Run latest ASAN tests: `.cargo/check.sh --memory-checks`
- [ ] Check nightly CI runs: GitHub Actions → Memory Safety Checks
- [ ] Review dependency changes: `git log --oneline --all -- Cargo.toml` (last 3 months)

#### Rust/Nightly Updates
- [ ] Verify nightly version: `rustc --version` (after `rustup update nightly`)
- [ ] Any new sanitizer features?
- [ ] Any breaking changes to ASAN behavior?

#### Suppression File Review
- [ ] All suppressions still relevant?
  - [ ] PyO3 version unchanged? (if changed, review patterns)
  - [ ] Python version unchanged? (if changed, test on new version)
  - [ ] Dependencies unchanged? (if changed, run ASAN)
- [ ] Any suppressions that can be removed?
  - (e.g., dependency upgraded past problematic version)
- [ ] Any new false positives?

#### Dependency Analysis
- [ ] Major version updates? (crypto, PyO3, etc.)
- [ ] New memory-critical dependencies?
- [ ] Run full test suite: `cargo test --all`

#### CI Status
- [ ] Nightly job passing? (GitHub Actions)
- [ ] Any false positives introduced?
- [ ] Job runtime acceptable (<10 minutes)?

#### Action Items
- [ ] Issue for any suppressions to add
- [ ] Issue for any problematic dependencies
- [ ] Update this runbook if procedures change

#### Sign-Off
Reviewed by: [reviewer]
Date: YYYY-MM-DD
Status: ✓ Complete / ⚠️ Issues found (see action items)
```

### Running the Review

1. **Schedule**: Set reminder for quarter start (Jan 1, Apr 1, Jul 1, Oct 1)
2. **Timing**: Allocate 1-2 hours
3. **Tools needed**: Git, Rust, cargo, this runbook
4. **Document**: Update this file with results

### Common Findings

**Expected (routine):**
- All tests pass
- No new suppressions needed
- Suppression file still accurate

**Investigate further:**
- New ASAN warnings in nightly CI
- Test timeouts on ASAN (might need optimization)
- Suppression patterns no longer matching

## Adding a New Suppression

### Step 1: Diagnose the Issue

```bash
# Run ASAN with verbose output
ASAN_OPTIONS=verbosity=2 .cargo/check.sh --memory-checks 2>&1 | tee asan-output.log

# Or from nightly CI
# Download workflow logs from GitHub Actions
```

### Step 2: Identify the Pattern

Examine ASAN output:

```
==12345==ERROR: LeakSanitizer: detected memory leaks

Direct leak of 1024 byte(s) in 1 object(s) allocated from:
    #0 0x7f... in malloc (/path/libc.so.6+0x...)
    #1 0x7f... in PyModule_Create2 (/usr/lib/libpython3.10.so.1.10+0x...)
    #2 0x7f... in PyInit_mrrc ...
```

**Key info:**
- **Direct leak**: Memory that was never freed (vs indirect: freed via wrong path)
- **Allocated from**: Function that allocated the memory (usually external)
- **Stack trace**: Call chain (look for external library names)

### Step 3: Research Context

```bash
# Is this a known pattern?
grep -r "PyModule_Create2" .valgrind.supp .cargo/asan_suppressions.txt

# Has it been reported?
# Search: "python pymodule leak" + version

# Is it in a community suppression file?
# Search: "pyo3" "valgrind suppressions"
```

**Check references:**
- PyO3 GitHub issues (search "memory leak")
- Python docs (sys.intern, PyModule initialization)
- Community suppression files (other projects)

### Step 4: Verify It's Safe

Ask:
1. **Is it in mrrc code?** → Fix it, don't suppress
2. **Is it in a dependency?** → Suppress if safe and documented
3. **Does it grow with iterations?** → Real leak, investigate more
4. **Is it suppressed elsewhere?** → Copy existing suppression

### Step 5: Add to Suppression File

Edit `.cargo/asan_suppressions.txt`:

```
# ============================================================================
# New Suppression Category
# ============================================================================
# What is this suppression for (1 sentence)
# Why it's not a bug: [technical explanation]
# - Reference: [URL, issue, or community source]
# - Status: Known pattern / Expected behavior / Dependency limitation
# - Tested: [date] on Python X.Y, PyO3 X.Y

{
   suppression_name
   Memcheck:Leak
   match-leak-kinds: reachable
   fun:malloc
   fun:PyModule_*
}
```

### Step 6: Test the Suppression

```bash
# Rebuild with new suppression
.cargo/check.sh --memory-checks

# Should now pass
# If still fails, refine pattern or investigate further
```

### Step 7: Document and Commit

```bash
git add .cargo/asan_suppressions.txt
git commit -m "Add suppression: [category]

- Suppress: [what pattern]
- Reason: [why it's safe]
- Reference: [source]
- Tested: [test results]"
```

## Escalation: Real Memory Leak Found

If ASAN detects a real leak in mrrc code:

### Immediate Steps

1. **Confirm it's real**:
   - Not in suppression file
   - Located in `mrrc` code (not dependency)
   - "definitely lost" (not reachable leak)
   - Reproducible (doesn't appear intermittently)

2. **Document the issue**:
   ```bash
   ASAN_OPTIONS=verbosity=2 .cargo/check.sh --memory-checks 2>&1 | tee leak-report.log
   ```

3. **Create issue**:
   ```bash
   bd create "Memory leak in [location]" \
     -t bug -p 0 \
     --deps discovered-from:asan \
     -d "ASAN detected leak:
     [paste key parts of ASAN output]
     
     To reproduce:
     .cargo/check.sh --memory-checks
     
     See leak-report.log for full output"
   ```

### Investigation

1. **Understand the leak**:
   - Where is memory allocated?
   - Why isn't it freed?
   - Is it a logic bug or API misuse?

2. **Check history**:
   - When was this introduced?
   - Related commits?
   ```bash
   git log -p --all -S "allocation_site" -- src/
   ```

3. **Fix the issue**:
   - Implement proper cleanup
   - Add tests to prevent regression
   - Run ASAN again to verify fix

4. **Close the issue**:
   ```bash
   bd close <issue-id> --reason "Fixed via commit SHA..."
   ```

## Updating for New Versions

### PyO3 Major Version Update

```markdown
## When PyO3 version changes significantly:

1. Update Cargo.toml version
2. Run ASAN tests:
   .cargo/check.sh --memory-checks
3. Review new suppressions needed:
   - Check PyO3 changelog for memory-related changes
   - Test PyO3 examples from docs
4. Update .cargo/asan_suppressions.txt if needed
5. Commit with note: "Update ASAN suppressions for PyO3 X.Y"
```

### Python Version Update

```markdown
## When Python version changes:

1. Test on new Python:
   - Build extension: maturin develop
   - Run ASAN: .cargo/check.sh --memory-checks
2. Check for new Python leaks:
   - Compare ASAN output with previous version
   - Suppress new safe patterns only
3. Document in suppressions file:
   "Tested on Python X.Y, PyO3 X.Y"
```

### Rust Nightly Changes

```markdown
## When Rust nightly updates (monthly):

1. Update: rustup update nightly
2. Run ASAN: .cargo/check.sh --memory-checks
3. If breakage, check:
   - ASAN documentation for new features/flags
   - Rust sanitizer tracking issue (github.com/rust-lang/rust)
   - Adjust RUSTFLAGS if needed
```

## Performance Tuning

### If ASAN Tests Are Too Slow

**Symptom**: `.cargo/check.sh --memory-checks` takes >1 minute

**Options** (in order of preference):

1. **Accept the overhead** (1-2 min is normal)
2. **Run local subset**:
   ```bash
   RUSTFLAGS="-Z sanitizer=address" cargo test --lib
   ```
3. **Disable some checks** (in `asan_suppressions.txt`):
   ```bash
   # Temporarily disable memory leak detection if only interested in UAF
   LSAN_OPTIONS=detect_leaks=0
   ```
4. **Upgrade hardware** (ASAN is CPU-hungry)

### If ASAN Tests Are Flaky

**Symptom**: ASAN sometimes reports issue, sometimes doesn't

**Causes**: Usually threading/timing issues (real bugs!)

**Actions**:
1. **Increase iterations**:
   ```bash
   RUSTFLAGS="-Z sanitizer=address" \
   cargo test --lib -- --test-threads=1
   ```
2. **Check for data races**:
   ```bash
   # Use ThreadSanitizer
   RUSTFLAGS="-Z sanitizer=thread" cargo test --lib
   ```
3. **Investigate test code**:
   - Are tests actually thread-safe?
   - Any global state?
   - Race condition in test setup?

## Reporting Issues

### To MRRC Developers

```
Title: ASAN detected [issue type]

ASAN found a [use-after-free / memory leak / buffer overflow]:
- Location: [file:line]
- Function: [function name]
- Severity: [blocking / non-blocking]

To reproduce:
  .cargo/check.sh --memory-checks

Details:
  [relevant parts of ASAN output]
```

### To External Projects (PyO3, Python, Dependencies)

```
Title: Memory leak / issue in [dependency]

Package: [name] version X.Y.Z
Observed in: MRRC project (Rust+PyO3)

Issue:
  [brief description]
  [ASAN or Valgrind output]

Steps to reproduce:
  [minimal example]
```

## Tools and Commands

### Common ASAN Commands

```bash
# Standard run
.cargo/check.sh --memory-checks

# Verbose output
ASAN_OPTIONS=verbosity=2 .cargo/check.sh --memory-checks

# Strict mode (might catch more false positives)
ASAN_OPTIONS=halt_on_error=1 .cargo/check.sh --memory-checks

# Without leak detection (if only checking UAF)
ASAN_OPTIONS=detect_leaks=0 .cargo/check.sh --memory-checks

# Single-threaded (more deterministic for flaky tests)
RUSTFLAGS="-Z sanitizer=address" cargo test --lib -- --test-threads=1
```

### Suppression Syntax Validation

```bash
# Check suppression file format
# (LSAN will error if syntax is wrong when running ASAN)
.cargo/check.sh --memory-checks 2>&1 | grep -i "invalid suppression"
```

### Viewing CI Results

```bash
# GitHub CLI (if installed)
gh run list --workflow=memory-safety.yml
gh run view <run-id> --log

# Or via GitHub web UI:
# Actions → Memory Safety Checks → [latest run]
```

## References

- `docs/MEMORY_SAFETY.md`: User guide
- `docs/design/MEMORY_SAFETY_CI.md`: Technical infrastructure
- `.cargo/asan_suppressions.txt`: Current suppressions
- `docs/design/VALGRIND_BASELINE.md`: Valgrind investigation results
- `CONTRIBUTING.md`: Contributing guidelines (includes memory safety section)

## Document History

| Date | Version | Changes |
|------|---------|---------|
| 2026-01-09 | 1.0 | Initial runbook created |

---

**Last updated:** 2026-01-09  
**Maintained by:** MRRC development team  
**Review frequency:** Quarterly
