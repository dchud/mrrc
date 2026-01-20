# Calendar Versioning (CalVer) Proposal for MRRC

## Executive Summary

MRRC is currently using semantic versioning (0.4.0). This proposal evaluates whether to switch to calendar versioning (CalVer) and identifies all necessary changes if adopted.

**Recommendation Status**: Not finalized - this proposal documents all considerations for decision-making.

---

## 1. Current State

### Existing Versioning

| Component | Current Version | Location | Release Cadence |
|-----------|-----------------|----------|-----------------|
| Rust crate | 0.4.0 | `Cargo.toml` | ~1 major per 1-2 weeks |
| Python wrapper | 0.4.0 | `pyproject.toml` | ~1 major per 1-2 weeks |
| Changelog | SemVer format | `CHANGELOG.md` | Documents all changes |

### Current Changelog (Latest 5 Versions)

- **0.4.0** - 2026-01-09 (API parity completeness, Query DSL Python bindings, Developer experience improvements)
- **0.3.1** - 2026-01-07 (CI/CD improvements, Python wheel build fixes)
- **0.3.0** - 2026-01-06 (GIL release & concurrency, parallel I/O backend)
- **0.2.0** - 2025-12-31 (Python integration, comprehensive benchmarking)
- **0.1.0** - 2025-12-28 (Core features, ISO 2709 parsing/writing)

**Timeline**: 23 days elapsed (2025-12-17 initial commit to 2026-01-09 current), 5 releases = ~0.22 releases per day (rapid but sustainable velocity)

---

## 2. CalVer Options Evaluation

### Option A: YYYY.MM (Simple Calendar)

**Format**: e.g., `2026.01`

**Advantages**:
- Clear release date at a glance
- Simple to parse and understand
- Good for projects with monthly releases
- Fewer version numbers to manage

**Disadvantages**:
- No patch version for hotfixes
- Two releases in same month need workaround (e.g., 2026.01a, 2026.01.1)
- Can't distinguish between release on 1st vs 28th of month

**Rust ecosystem fit**: Unusual in Rust ecosystem; would stand out

**Example sequence**:
```
2025.12 (Dec release)
    ↓
2026.01 (Jan release)
    ↓
2026.01.1 (Jan hotfix)
    ↓
2026.02 (Feb release)
```

---

### Option B: YYYY.MM.PATCH (Recommended CalVer)

**Format**: e.g., `2026.01.0`, `2026.01.1`, `2026.01.2`

**Advantages**:
- Full flexibility: multiple releases per month
- Patch version clear (0 = main release, 1+ = hotfixes)
- Date context always visible
- No ambiguity with multiple releases in same month
- Industry-standard CalVer form

**Disadvantages**:
- Three-part version number (slightly longer)
- First number changes yearly (some consider this verbose)
- Breaking changes still need explicit communication (CalVer doesn't encode API semver)

**Rust ecosystem fit**: Acceptable; some Rust projects use this form

**Example sequence**:
```
2025.12.0 (Dec release)
    ↓
2026.01.0 (Jan release)
    ↓
2026.01.1 (Jan hotfix)
    ↓
2026.01.2 (Jan hotfix #2)
    ↓
2026.02.0 (Feb release)
```

---

### Option C: YYYY.0M.PATCH (With Zero-Padded Month)

**Format**: e.g., `2026.01.0`, `2026.02.0`, etc.

**Advantages**:
- Identical benefits to Option B
- Zero-padding maintains consistent version length
- Sorts better in some contexts

**Disadvantages**:
- Arguably more verbose than Option B
- No practical benefit in practice

**Example sequence**:
```
2025.12.0 → 2026.01.0 → 2026.01.1 → 2026.02.0
```

---

### Option D: Keep Semantic Versioning

**Format**: e.g., `0.5.0`, `1.0.0`, `2.0.0`

**Advantages**:
- Industry standard in Rust ecosystem
- Clearly encodes breaking changes (major), features (minor), fixes (patch)
- Familiar to all developers
- Cargo and crates.io fully optimized for SemVer

**Disadvantages**:
- Release date not encoded (need to check changelog)
- High velocity projects stay at 0.x for long time (confusing signal)
- Requires discipline to distinguish breaking vs non-breaking changes
- MRRC's rapid changes might lead to major version inflation

---

## 3. Analysis: MRRC Characteristics

### Velocity
- **Current**: 5 releases in 11 days = ~2 releases per day
- **Pattern**: Rapid iteration with high-quality output
- **Implication**: CalVer would naturally encode this; SemVer would need frequent major bumps

### Breaking Changes Frequency
- Reviewing `CHANGELOG.md`: Most entries under "Changed" are non-breaking
- Even major changes maintain backward compatibility
- **Implication**: Low frequency of true breaking changes argues for staying in SemVer space longer

### Target Audience
- Librarians building MARC systems (not version-savvy developers)
- Library systems administrators
- Python developers migrating from pymarc
- Rust developers building MARC tools
- **Implication**: Clear, simple version numbers matter for non-technical audiences

### Ecosystem Context
- Core is **Rust library** (ecosystem: SemVer is standard)
- Also **Python wrapper** (ecosystem: CalVer is increasingly common)
- Dual-target versioning complexity
- **Implication**: Shared versioning makes sense; one choice for both

### Release Philosophy
- **Goal**: Frequent, stable releases with strong backward compatibility
- **Not**: Major annual releases or breaking-change-driven versioning
- **Implication**: CalVer maps to actual release pattern better than SemVer

---

## 4. Rust Ecosystem Context

### SemVer Dominance
- crates.io registry **assumes** SemVer semantics
- Cargo.lock relies on SemVer for version resolution
- cargo-semver-checks validates SemVer compliance
- **State**: Rust community strongly prefers SemVer

### CalVer Adoption in Rust
- **Low adoption**: Few mainstream Rust crates use CalVer
- Examples that do:
  - `rustls` (switched from SemVer to CalVer)
  - `maturin` (uses CalVer)
  - `cargo-edit` (historical CalVer, now SemVer)
- **Perception**: CalVer is seen as unconventional in Rust
- **Risk**: Users might distrust non-standard versioning

### Compatibility Guarantees
- SemVer: Explicit API stability promise (major.minor.patch)
- CalVer: No encoded semantics (date-based only)
- **Gap**: CalVer requires explicit versioning policy in docs

---

## 5. Required Changes If Adopting CalVer

### 5.1 Version Updates (Immediate)

| File | Current | New | Details |
|------|---------|-----|---------|
| `Cargo.toml` | `0.4.0` | `2026.01.0` | Main Rust crate package |
| `src-python/Cargo.toml` | `0.4.0` | `2026.01.0` | PyO3 Python extension package |
| `pyproject.toml` | `0.4.0` | `2026.01.0` | Python package metadata (maturin) |
| `README.md` | Multiple refs | Update all | Install examples, dependency versions |
| `CHANGELOG.md` | Format | CalVer format | Future entries use YYYY.MM.PATCH |

### 5.2 Changelog Updates

Need to convert CHANGELOG.md sections:

**Before**:
```markdown
## [0.4.0] - 2026-01-08
...
## [0.3.1] - 2026-01-07
...
## [0.3.0] - 2026-01-06
```

**After** (if adopting CalVer):
```markdown
## [2026.01.0] - 2026-01-08
...
## [2026.01.1] - 2026-01-07  (hypothetically 0.3.1 becomes 2026.01.1)
...
## [2026.01.2] - 2026-01-06  (hypothetically 0.3.0 becomes 2026.01.2)
```

**Considerations**:
- Retroactive conversion is messy (rewrites history)
- Alternative: Keep 0.x versions in changelog, start CalVer from next release
- Need to decide: clean slate or gradual transition?

### 5.3 CI/CD Pipeline Updates

**Workflows to review** (in `.github/workflows/`):
- `build.yml` - Cargo build pipeline (no version checks found)
- `lint.yml` - Code quality gates (no version checks found)
- `test.yml` - Test execution (no version checks found)
- `python-build.yml` - Wheel building via maturin (may reference versions)
- `python-release.yml` - **RELEASE GATE** - Tag-triggered publishing (`v*` pattern)
- `python-benchmark.yml` - Benchmark execution (no version checks)
- `memory-safety.yml` - ASAN checks (no version checks)
- `coverage.yml` - Coverage reporting (no version checks)

**Changes needed**:
1. **Git tag format**: Currently triggered by `v*` pattern (e.g., `v0.4.0`) → Will continue to work for `v2026.01.0`
2. **No hardcoded version strings detected** in CI workflows - safe to proceed
3. **Maturin publish**: Reads version from `pyproject.toml` automatically (no changes needed)
4. **Cargo publish**: Reads version from `Cargo.toml` automatically (no changes needed)

**Existing safe pattern**: Both release workflows automatically detect versions from manifests, so CalVer adoption requires only updating manifest files (Cargo.toml, pyproject.toml)

### 5.4 Documentation Updates

**Files to update**:

1. **README.md**
   - Update installation version example
   - Update any "version X" references
   - Consider adding "CalVer" explanation

2. **docs/README.md** (if exists)
   - Update version references
   - Add CalVer versioning policy section

3. **CONTRIBUTING.md**
   - Add versioning guidance for contributors
   - Document how to determine next version number

4. **docs/design/** (this directory)
   - Create versioning policy document
   - Link from README

5. **pyproject.toml & Cargo.toml**
   - Update comments to explain CalVer scheme
   - Document versioning rules

### 5.5 Versioning Policy Documentation

**New file needed**: `docs/VERSIONING.md`

**Contents**:
```markdown
# MRRC Versioning Policy

MRRC uses Calendar Versioning (CalVer) with format YYYY.MM.PATCH.

## Versioning Scheme

- **YYYY**: Release year (e.g., 2026)
- **MM**: Release month (e.g., 01 for January)
- **PATCH**: Release sequence within month (0 = main release, 1+ = hotfixes)

## Examples

- 2026.01.0 - First release of January 2026
- 2026.01.1 - First hotfix in January 2026
- 2026.02.0 - First release of February 2026

## Release Types

1. **Regular Release** (PATCH = 0)
   - Scheduled monthly release
   - Contains features, improvements, non-breaking changes
   - All testing gates pass

2. **Hotfix Release** (PATCH ≥ 1)
   - Unscheduled release to fix critical bugs
   - Minimal changes from previous version
   - Same version prefix (year.month) as previous release

## API Stability

While using CalVer, MRRC maintains strong backward compatibility:

- Minor updates (YYYY.MM.0 → YYYY.MM.1): Always backward compatible
- Monthly releases (YYYY.MM.0 → YYYY.(MM+1).0): Nearly always compatible
  - Breaking changes only with clear migration path
  - Deprecated APIs remain functional for grace period
- Major API overhauls documented in CHANGELOG

See [CHANGELOG.md](../CHANGELOG.md) for details of each release.

## Deprecation Policy

- Features marked deprecated in version X
- Support continues through version X+6 months
- Full removal in version after deprecation period

## Publishing

- Rust crate: Published to [crates.io](https://crates.io/crates/mrrc)
- Python package: Published to [PyPI](https://pypi.org/project/mrrc/)
- Both use identical version numbers
```

### 5.6 Code Changes

**Minimal code changes needed**:
- Version strings in `Cargo.toml` and `pyproject.toml` only
- No source code changes required
- No functional API changes

---

## 6. Migration Path Considerations

### Option 1: Clean Break (Starting from Next Release)

**Approach**:
- Keep 0.x in CHANGELOG as historical record
- Start CalVer from next release (2026.01.0 or 2026.02.0)
- Document transition in migration guide

**Pros**:
- Clean separation in version numbering
- No retroactive changelog rewrites
- Clear point where versioning changes

**Cons**:
- Creates perceptual gap (0.4.0 → 2026.01.0 is jarring)
- Some users might get confused about what happened

### Option 2: Retroactive Conversion (Complete Rewrite)

**Approach**:
- Convert all existing versions in CHANGELOG to CalVer equivalents
- Rewrite Git tags to match (forces local history rewrite)
- Update any documentation referencing old versions

**Pros**:
- Single versioning scheme throughout history
- No gap in version numbering

**Cons**:
- Breaks existing Git history
- Complex migration for people with local clones
- Harder to maintain (rewriting public history)
- Not worth the pain for established library

### Option 3: Gradual Transition (Hybrid)

**Approach**:
- Decide on date for transition (e.g., Q2 2026)
- Keep 0.x until transition date
- At transition: 0.4.x → 2026.02.0 (first CalVer release)
- Document clearly in release notes

**Pros**:
- Gives users time to understand change
- No sudden jump
- Allows gradual update of references

**Cons**:
- Extends SemVer versioning longer
- Requires coordination across releases

---

## 7. Risk Assessment

### Risks of Adopting CalVer

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| Rust ecosystem pushback | Medium | Low | Document policy clearly; explain rationale |
| Version parsing issues | Low | Medium | Test with cargo, pip, CI systems |
| User confusion | Medium | Low | Provide migration guide; explain benefits |
| Git tag rewriting | Medium | Medium | Use clean break instead of retroactive |
| PyPI/crates.io issues | Low | High | Test publishing before committing |
| CI/CD workflow breakage | Medium | High | Audit all workflows; test locally first |
| Sorting/comparison bugs | Low | Medium | CalVer requires numeric parsing (not string) |

### Risks of Keeping SemVer

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| Semantic drift | High | Low | Document versioning policy |
| Version inflation | Medium | Low | Stay at 0.x longer; accept 1.0 when ready |
| Release date obscured | Medium | Low | Maintain changelog; document releases clearly |
| User questions | Medium | Low | Provide versioning documentation |

---

## 8. Decision Framework

### Critical Questions for MRRC

1. **Release Velocity & Cadence** ⚠️ KEY DECISION POINT
   - *Current state*: 5 releases in 23 days (~0.22/day) with full API parity achieved
   - *Question*: Will MRRC maintain ~1 release per week, or stabilize to longer cycles?
   - **CalVer signal**: Rapid cadence (weekly+) → CalVer shines
   - **SemVer signal**: Slower cadence (monthly+) or approaching stability → SemVer sufficient

2. **Target Audience & Version Literacy**
   - *Users*: Library administrators, Python developers migrating from pymarc, Rust developers
   - *Question*: Do non-technical library administrators understand version numbers?
   - **CalVer advantage**: "2026.01.0" clearly indicates "January 2026 release" to non-technical users
   - **SemVer advantage**: "0.5.0" clearly indicates "approaching 1.0 stability" to developers

3. **API Stability & Breaking Change Frequency** ⚠️ CRITICAL IF ADOPTING CALVER
   - *Current state*: [Unreleased] shows field insertion order as breaking change, but previous releases maintained compatibility
   - *Question*: Will breaking changes remain rare (1-2 per year) or become frequent?
   - **CalVer risk**: Without semantic encoding, users must consult changelog for breaking changes
   - **Mitigation**: Explicit policy document (docs/VERSIONING.md) required before adopting

4. **Rust Ecosystem Expectations**
   - *Current state*: SemVer is standard; CalVer is unconventional
   - *Question*: How important is conformity to Rust ecosystem norms?
   - **Consideration**: Users expect SemVer; CalVer adoption signals "different release philosophy"
   - **Precedent**: rustls, maturin use CalVer; precedent exists but limited

5. **Version 1.0 Milestone**
   - *Current state*: At 0.4.0 with full pymarc API parity
   - *Question*: Is 1.0.0 release planned soon (next 1-2 months), or is library staying in 0.x for stability signals?
   - **CalVer option**: "Never worry about 1.0 milestone; calendar dates signal maturity instead"
   - **SemVer option**: "Plan 1.0.0 as definitive stability marker; use 0.x for pre-release signals"

---

## 9. Comparison Table: SemVer vs CalVer for MRRC

| Criterion | SemVer (0.x+) | CalVer (YYYY.MM.PATCH) | Winner |
|-----------|---------------|------------------------|--------|
| **Ecosystem fit** | ✓ Standard in Rust | ✗ Unconventional | SemVer |
| **Release date visible** | ✗ Hidden in changelog | ✓ In version string | CalVer |
| **Breaking changes clear** | ✓ Major version | ✗ Only in docs | SemVer |
| **Hotfix support** | ✓ PATCH version | ✓ PATCH version | Tie |
| **Parsing simplicity** | ✓ Simple numeric | ✓ Simple date-based | Tie |
| **Library maturity signal** | ⚠ 0.x unclear | ✓ 2026.01 = recent | CalVer |
| **Multi-release/month** | ✓ Works fine | ✓ Works fine | Tie |
| **Tool support (cargo)** | ✓ Native | ✓ Works (less tested) | SemVer |
| **User expectation** | ✓ Known standard | ✗ Needs explanation | SemVer |
| **Long-term clarity** | ✗ Major inflation | ✓ Stays compact | CalVer |

---

## 10. Implementation Checklist (If Approved)

If decision is made to adopt CalVer, follow this order:

- [ ] Create versioning policy document (docs/VERSIONING.md)
- [ ] Update Cargo.toml with new version
- [ ] Update src-python/Cargo.toml with new version
- [ ] Update pyproject.toml with new version
- [ ] Update README.md installation example
- [ ] Update CHANGELOG.md header format
- [ ] Update any examples or docs with version refs
- [ ] Review .github/workflows/ for version handling
- [ ] Review scripts/ for version handling
- [ ] Test local builds (cargo build, maturin develop)
- [ ] Test publish dry-run: `cargo publish --dry-run`
- [ ] Create test PyPI upload to verify maturin packaging
- [ ] Update bd issue with completion status
- [ ] Document rationale in thread (why CalVer chosen)
- [ ] Publish release with clear announcement

---

## 11. Recommended Next Steps

1. **Gather feedback**: Share this proposal with project stakeholders
   - Is rapid release cadence expected to continue?
   - Do users care about version recency?
   - Is Rust ecosystem fit important?

2. **Prototype changes**: Create a branch testing CalVer migration
   - Verify all CI/CD workflows still pass
   - Test cargo and pip installations
   - Confirm Git workflow doesn't break

3. **Make decision**: Based on feedback, choose:
   - **Option 1**: Switch to YYYY.MM.PATCH CalVer
   - **Option 2**: Stay with Semantic Versioning

4. **If CalVer chosen**:
   - Use clean-break approach (no history rewriting)
   - Document in versioning policy and migration guide
   - Announce clearly in release notes

5. **If SemVer chosen**:
   - Document versioning policy for maintainers
   - Plan path to 1.0.0 (when will library be "stable"?)
   - Clarify breaking change communication strategy

---

## 12. Conclusion

Both versioning schemes can work for MRRC, with clear trade-offs:

### Recommendation Logic

**Choose CalVer (YYYY.MM.PATCH) if ALL of these are true:**
1. Release cadence will remain ~weekly (not monthly or slower)
2. Library target audience includes non-technical library administrators
3. Ecosystem convention break is acceptable
4. Commitment to document API stability explicitly (docs/VERSIONING.md mandatory)

**Choose SemVer (stay at 0.x) if ANY of these are true:**
1. Release cadence will slow to monthly or quarterly
2. Plan 1.0.0 release within 1-2 months as stability milestone
3. Rust ecosystem conformity is critical to user trust
4. Preference to let version scheme itself encode API stability semantics

### Current State Assessment

MRRC is at a decision point:
- ✅ API parity achieved (0.4.0)
- ✅ CI/CD infrastructure stable
- ✅ Active development (~1 release/week)
- ⚠️ Release cadence trajectory unclear (will it sustain or slow?)
- ⚠️ Path to 1.0 not yet defined

**This proposal documents all considerations for decision-making by the project team.**

---

## References

- [CalVer.org](https://calver.org/) - Specification and examples
- [Semantic Versioning](https://semver.org/) - Industry standard
- [Rust semver Guide](https://doc.rust-lang.org/cargo/reference/semver.html) - Cargo semantics
- [PEP 440](https://www.python.org/dev/peps/pep-0440/) - Python versioning
