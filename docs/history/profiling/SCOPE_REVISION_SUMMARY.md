# Scope Revision Summary: Within-Mode Profiling

**Date:** 2026-01-08  
**Change:** Performance profiling scope changed from comparative to within-mode analysis

---

## What Changed

### Before
Profiling tickets explicitly framed around comparisons:
- "Investigate why Rust concurrent achieves 2.52x while Python achieves 3.74x"
- "Comparison to Python wrapper baseline"
- Optimization recommendations based on "matching Python's performance"

### After
All profiling now within-mode analysis:
- "What limits this implementation's performance?"
- "Where does it spend time?"
- "What are optimization opportunities in this mode?"
- Optimization decisions separate in OPTIMIZATION_PROPOSAL.md

---

## Affected Closed Tickets

### ✓ Salvageable Work

**mrrc-u33.4** (Python single-threaded)
- Status: Already refactored, complete ✓
- No further action needed

**mrrc-u33.2** (Rust single-threaded profiling)
- Status: Closed
- Work product: RUST_SINGLE_THREADED_PROFILING_RESULTS.md (refactored)
- Action: Adequate for within-mode analysis; no redo needed

**mrrc-u33.3** (Rust concurrent profiling)
- Status: Closed
- Work product: RUST_CONCURRENT_PROFILING_RESULTS.md
- Action: Refactoring in progress (mrrc-dpk.1)

### ⚠️ Refactored Away

**mrrc-u33.1** (Comparative analysis)
- Status: Closed with notes
- Original: Explicitly comparative ("why does Python outperform Rust?")
- Resolution: Closed; comparative analysis refactored to OPTIMIZATION_PROPOSAL.md
- New equivalent: Work continues through individual profiling tasks (u33.7, u33.8, u33.9, u33.10)

---

## Reorganized Open Tasks

### Changed Status

| Ticket | Before | After | Reason |
|--------|--------|-------|--------|
| mrrc-u33.6 | Open (duplicate of u33.7) | **Closed** | Consolidated |
| mrrc-u33.1 | Open (comparative) | **Closed** | Refactored; work continues elsewhere |

### Rescoped for Within-Mode Focus

| Ticket | Change |
|--------|--------|
| mrrc-u33.7 | Renamed/refocused: "Deep-dive analysis: Pure Rust single-threaded parsing bottlenecks" |
| mrrc-u33.8 | Refocused: "Rayon task scheduling efficiency" (within-mode questions) |
| mrrc-u33.9 | Refocused: "Memory bandwidth and cache efficiency" (within-mode analysis) |
| mrrc-u33.10 | Refocused: "Workload-dependent performance characteristics" (within-mode inflection points) |

### Supporting Tasks

| Ticket | Purpose |
|--------|---------|
| mrrc-u33.2.2 | Flamegraph profiling for u33.7 |
| mrrc-u33.2.3 | Heaptrack memory profiling for u33.7 |

---

## Documentation Changes

### Created
- **OPTIMIZATION_PROPOSAL.md** - Optimization strategies and decisions (extracted from profiling)
- **REFACTORING_NOTES.md** - Refactoring guidelines and scope clarification
- **ASSESSMENT_CLOSED_TICKETS.md** - Analysis of closed tickets against new scope
- **SCOPE_REVISION_SUMMARY.md** (this document)

### Refactored
- PROFILING_SUMMARY.md - Removed cross-mode comparison table
- PROFILING_PLAN.md - Removed comparative justification
- PHASE_2_DETAILED_ANALYSIS.md - Separated optimization proposals
- RUST_SINGLE_THREADED_PROFILING_RESULTS.md - Removed comparative questions
- README.md - Added refactoring status note

### In Progress
- RUST_CONCURRENT_PROFILING_RESULTS.md (mrrc-dpk.1) - Remove "matches Python" framing

---

## Key Principles Going Forward

**Within-Mode Profiling Means:**
- ✅ Measure each implementation in isolation
- ✅ Identify bottlenecks specific to that mode
- ✅ Quantify costs and overheads
- ✅ Ask "what limits performance in this mode?"
- ✅ Provide data for optimization decisions

**NOT Comparative:**
- ❌ Compare to other modes
- ❌ Justify improvements by referencing other modes
- ❌ Use cross-mode comparisons to reason about changes
- ❌ Ask "why is mode X faster than mode Y?"
- ❌ Mix profiling with optimization decisions

---

## Work Status Summary

| Category | Status | Notes |
|----------|--------|-------|
| **Completed & Delivered** | ✓ | pymrrc single-threaded profiling (mrrc-u33.4) |
| **Completed & Refactored** | ✓ | Rust single-threaded profiling (mrrc-u33.2) |
| **Completed & Pending Refactor** | ⏳ | Rust concurrent profiling (mrrc-u33.3) → mrrc-dpk.1 |
| **Closed & Refactored** | ✓ | Comparative analysis (mrrc-u33.1) → moved to OPTIMIZATION_PROPOSAL.md |
| **Reorganized as Within-Mode** | ✓ | mrrc-u33.6, u33.7-u33.10 rescoped |
| **Documentation Cleaned** | ✓ | All profiling docs refactored for scope |

---

## Next Steps

1. **Complete mrrc-dpk.1** - Finish refactoring RUST_CONCURRENT_PROFILING_RESULTS.md
2. **Execute remaining profiling** - mrrc-u33.7, u33.8, u33.9, u33.10 tasks
3. **Consolidate findings** - Synthesize all profiling into baseline metrics
4. **Plan optimizations** - Use OPTIMIZATION_PROPOSAL.md as guide

---

## References

- Epic: mrrc-u33 (Performance optimization review)
- Refactoring task: mrrc-dpk (Refactor profiling documentation)
- Refactoring subtask: mrrc-dpk.1 (RUST_CONCURRENT_PROFILING_RESULTS.md)
- Documentation: docs/design/profiling/
- Optimization strategy: docs/design/OPTIMIZATION_PROPOSAL.md
