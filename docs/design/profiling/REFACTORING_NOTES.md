# Profiling Documentation Refactoring Notes

**Status:** Pending refactoring per mrrc-dpk  
**Reason:** Scope change from comparative analysis to within-mode bottleneck identification

## Problem Statement

The existing profiling documentation in `docs/design/profiling/` was written before the scope change to **within-mode profiling**. It contains comparative analysis and optimization proposals that should be separated.

### What's Wrong

1. **PROFILING_SUMMARY.md** (✗ Needs refactoring)
   - Contains cross-mode comparisons (line 151-159): "Comparison to Python Wrapper"
   - Includes optimization strategy (lines 44-90) which is not pure profiling
   - Mixes profiling data with decision-making
   - References non-existent `docs/design/OPTIMIZATION_PROPOSAL.md`

2. **PROFILING_PLAN.md** (✗ Needs review)
   - Line 13: "Python wrapper achieves 50% of Rust performance... suggesting optimization opportunities"
   - This is comparative reasoning, not within-mode profiling
   - Asks "What does Python's ProducerConsumerPipeline do better?" (line 114)

3. **PHASE_2_DETAILED_ANALYSIS.md** (✗ Partially needs refactoring)
   - Contains good bottleneck analysis for Rust mode (keep)
   - But includes optimization proposals (lines 240-244): "focus should be on..."
   - These belong in OPTIMIZATION_PROPOSAL.md, not profiling docs

4. **RUST_SINGLE_THREADED_PROFILING_RESULTS.md** (⚠️ Mostly OK)
   - Baseline metrics are good and within-mode
   - Line 17 has a question about Python: "Why does Python ProducerConsumerPipeline achieve..." (comparative)
   - This question should be moved to optimization analysis, not here

5. **RUST_CONCURRENT_PROFILING_RESULTS.md** (?)
   - Not reviewed yet

6. **README.md** (✓ OK)
   - Already emphasizes within-mode principle
   - New, explains correct scope

## What Should Happen

### Pure Profiling Data (Keep in docs/design/profiling/)

- Throughput metrics (rec/s)
- Latency measurements (µs/record)
- Memory allocation patterns (hotspots, heap usage)
- CPU intensity analysis (cycles per record)
- Bottleneck identification (what limits performance in this mode?)
- Call graphs and flame graphs
- Allocation hotspots

### Optimization Proposals (Move to docs/design/OPTIMIZATION_PROPOSAL.md)

- "We should implement Phase 1: SmallVec + Compact Tags"
- "Expected result: 1.06M → 1.12M rec/s"
- Comparative reasoning: "Python's ProducerConsumerPipeline does X, Rust could benefit"
- Implementation recommendations
- Trade-offs and options

### References to Other Modes (Handle Carefully)

- OK: "For context, Python wrapper achieves X rec/s, but we're profiling Rust independently"
- NOT OK: "Python is faster, so we should implement Y to catch up"
- NOT OK: Using other mode's performance to justify Rust optimization

## Documents to Refactor

| Document | Status | Action |
|----------|--------|--------|
| PROFILING_SUMMARY.md | ✗ Needs major refactoring | Extract optimization sections to OPTIMIZATION_PROPOSAL.md; remove cross-mode comparison table; keep metrics + bottleneck analysis |
| PROFILING_PLAN.md | ⚠️ Needs minor edits | Remove "Python achieves X" justification; rephrase as "to identify bottlenecks in this mode" |
| PHASE_2_DETAILED_ANALYSIS.md | ⚠️ Needs section moves | Keep analysis; move "Phase 1/2/3 optimization strategy" to OPTIMIZATION_PROPOSAL.md |
| RUST_SINGLE_THREADED_PROFILING_RESULTS.md | ⚠️ Needs edits | Remove comparative question about Python; focus on "what limits Rust" |
| PROFILING_GUIDE.md | ✓ OK | Methodological guide, not comparative |
| README.md | ✓ OK | Already corrected |

## Guidelines for Future Profiling Docs

### DO
- ✅ Measure within-mode performance (throughput, memory, CPU)
- ✅ Identify bottlenecks specific to this mode
- ✅ Quantify costs (GIL, FFI, synchronization, allocation)
- ✅ Ask "where does this mode spend time?"
- ✅ Provide data for optimization decisions
- ✅ Compare same mode across different input sizes
- ✅ Note baseline metrics for regression testing

### DON'T
- ❌ Compare performance to other modes
- ❌ Propose "implement X because Y mode does it"
- ❌ Include optimization decisions in profiling docs
- ❌ Ask "why is this slower than that mode?"
- ❌ Use cross-mode comparison to justify changes
- ❌ Mix profiling analysis with implementation strategy

## File Organization After Refactoring

```
docs/design/profiling/
├── README.md (methodology and principles) ✓
├── PROFILING_GUIDE.md (tools and how-to) ✓
├── pure_rust_single_thread_profile.md (within-mode analysis)
├── pure_rust_concurrent_profile.md (within-mode analysis)
├── pymrrc_single_thread_profile.md (within-mode analysis) ✓
├── pymrrc_concurrent_profile.md (within-mode analysis)
└── [OLD DOCS - TO REFACTOR]
    ├── PROFILING_PLAN.md (archive or refactor)
    ├── PROFILING_SUMMARY.md (archive or refactor)
    ├── PHASE_2_DETAILED_ANALYSIS.md (archive or refactor)
    └── RUST_*_PROFILING_RESULTS.md (archive or refactor)

docs/design/
├── OPTIMIZATION_PROPOSAL.md (optimization decisions & strategy)
├── profiling/ (profiling data & analysis)
└── ... (other design docs)
```

## Related Issues

- **mrrc-dpk**: Refactor documentation to align with within-mode scope
- **mrrc-u33**: Epic for performance optimization (scope already updated)
- **mrrc-u33.4**: Completed Python single-threaded profiling (aligned with scope)

## Next Steps

1. Review and approve refactoring plan
2. Create OPTIMIZATION_PROPOSAL.md skeleton
3. Move optimization content from old profiling docs
4. Refactor old docs to remove comparative sections
5. Archive old docs with historical note (if desired)
6. Update cross-references in other docs
