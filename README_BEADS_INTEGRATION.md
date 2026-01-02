# Beads Integration: GIL Release Hybrid Implementation Plan

**Status:** ✅ Complete & Ready to Execute  
**Date:** January 2, 2026  
**Time to Execute:** 30 minutes

---

## What This Is

A comprehensive analysis of the detailed GIL Release implementation plan (`GIL_RELEASE_HYBRID_IMPLEMENTATION_PLAN_REVISIONS.md`) cross-referenced against existing bd (beads) issue tracking, identifying gaps and providing step-by-step beads integration instructions.

## Three New Documents Created

### 1. **BEADS_ACTION_SUMMARY.md** (Quick Reference - 5 min read)
- 6 critical issues identified
- What's missing in beads
- Required operations with bd commands
- Timeline estimate (14 days serial, 8-9 optimized)

### 2. **GIL_RELEASE_HYBRID_IMPLEMENTATION_PLAN_WITH_BEADS_MAPPING.md** (Detailed - 20 min read)
- Full cross-reference of plan vs. existing beads
- All gaps catalogued with issue IDs
- Complete task breakdown (Phase C: C.0-C.Gate, Phase H: H.0-H.Gate)
- Each task mapped to plan sections (§ & line numbers)
- Dependency chains visualized
- Success metrics & gate criteria

### 3. **BEADS_IMPLEMENTATION_CHECKLIST.md** (Execution - 30 min to run)
- Step-by-step instructions
- Ready-to-copy bd commands
- Phase-by-phase (7 phases × 5 min each)
- Verification checklist before declaring complete

---

## Critical Gaps Found

| Gap | Impact | Fix |
|-----|--------|-----|
| **Phase C has no subtasks** | Cannot track work | Create C.0-C.Gate (6 tasks) |
| **Phase H epic missing** | 50% of plan untracked | Create H epic + H.0-H.Gate (10 tasks) |
| **Diagnostic tests not tracked** | Cannot verify GIL | Create infrastructure tasks (2 tasks) |
| **Duplicate Phase C epic** | Confusion in tracking | Close mrrc-d0m |
| **Phase G missing H blocker** | Docs premature | Add H.Gate as dependent |

**Total new work items:** 19 tasks + 1 epic + 1 closed epic = 20 operations

---

## Execution Path

### Option A: Quick Start (30 minutes)
Run `BEADS_IMPLEMENTATION_CHECKLIST.md` phases 1-7:
1. Verify state (5 min)
2. Create Phase C subtasks (10 min) 
3. Create Phase H epic + subtasks (15 min)
4. Create infrastructure tasks (5 min)
5. Update existing epics (5 min)
6. Verify dependencies (5 min)
7. Documentation (included)

### Option B: Deep Dive First (1 hour)
1. Read `BEADS_ACTION_SUMMARY.md` (5 min)
2. Read `GIL_RELEASE_HYBRID_IMPLEMENTATION_PLAN_WITH_BEADS_MAPPING.md` (20 min)
3. Review original plan references as needed (15 min)
4. Run checklist (30 min)
5. Verify all tasks created correctly (10 min)

---

## Key Plan Decisions (Captured in Beads)

These will become part of each task description:

**Phase C (Batch Reading):**
- Batch size = 100 records (with validation sweep 10-500)
- Hard limits: 200 records/batch OR 300KB max
- GIL acquire/release: Single cycle per batch (100x reduction)
- Gate criterion: ≥1.8x speedup on 2-thread concurrent read

**Phase H (Rust I/O + Rayon):**
- ReaderBackend enum with 3 variants (PythonFile, RustFile, CursorBackend)
- Type detection: 8 supported input types + fail-fast for unknown
- Record boundary scanner: 0x1E delimiter detection
- Rayon pool: Respects RAYON_NUM_THREADS env var, default all cores
- Producer-consumer pipeline: Bounded channel (1000 records), backpressure
- Gate criterion: ≥2.5x speedup on 4-thread concurrent read

**Dependencies:**
- C.Gate → H.3 (sequential baseline blocks on Phase C completion)
- H.Gate → Phase G (docs must cover Phase H threading model)

**Diagnostic Infrastructure:**
- GIL verification test (wall-clock measurement, threaded reader)
- Batch size benchmarking utility (sweep 10-500, measure speedup curve)
- Python file I/O profiler (identify Python .read() bottleneck)
- Memory safety: ASAN on Linux, Leaks on macOS
- Panic safety: Rayon panic hooks, error propagation channels

---

## Timeline Impact

**Current Status:** Phases A-B-D complete, E-F-G in progress

**After Beads Integration:**
- Phase C: 5 days (critical path) 
- Phase H: 9 days (blocked on C.Gate)
- **Critical path:** 14 days serial, ~8-9 days with parallelism (H.0-H.2b in parallel with C.2-C.3)

**Phase F (Benchmarks) → Phase C Decision Gate:**
- Phase F happens *before* Phase C implementation starts
- Phase F asks: "Do we need Phase C optimization?"
- Answer informs Phase C execution priority
- Both Phase B baseline and Phase C.Gate measurement included

---

## Success Criteria

Before declaring beads integration complete:

✅ Phase C epic (mrrc-ppp) has 6 subtasks (C.0-C.Gate)  
✅ Phase H epic created with 10 subtasks (H.0-H.Gate)  
✅ All dependencies configured correctly in bd  
✅ C.Gate blocks H.3 (sequential baseline)  
✅ H.Gate blocks Phase G (documentation release)  
✅ Infrastructure tasks created (diagnostic + memory safety)  
✅ Phase G description updated with H blocker  
✅ Duplicate epic closed (mrrc-d0m)  
✅ All tasks reference original plan (§ & line numbers)  
✅ Timeline estimate captured in epic descriptions

---

## What Happens Next

After executing the checklist:

1. **Day 1:** Start Phase C (C.0) + Phase H.0 PoC (independent parallel start)
2. **Days 2-5:** Phase C implementation (C.1-C.Gate)
3. **Days 2-4 (parallel):** Phase H backend work (H.1-H.2b) while C.2-C.3 in flight
4. **Day 6 onwards:** Phase H sequential baseline (H.3) after C.Gate passes
5. **Days 7-9:** Phase H parallelism (H.4a-H.4c)
6. **Day 10:** Phase H.Gate (parallel benchmarking)
7. **After H.Gate:** Phase G documentation refresh

---

## Questions?

### For Plan Details
See original: `docs/design/GIL_RELEASE_HYBRID_IMPLEMENTATION_PLAN_REVISIONS.md` (500+ lines with full technical specs)

### For Beads Mapping
See detailed mapping: `docs/design/GIL_RELEASE_HYBRID_IMPLEMENTATION_PLAN_WITH_BEADS_MAPPING.md` (800+ lines, all §/line references)

### For Implementation
See checklist: `BEADS_IMPLEMENTATION_CHECKLIST.md` (step-by-step ready-to-run bd commands)

### For Quick Reference
See summary: `BEADS_ACTION_SUMMARY.md` (critical issues + timeline)

---

## Ready to Execute?

```bash
# Option 1: Follow the checklist step-by-step (30 min)
# Open: BEADS_IMPLEMENTATION_CHECKLIST.md
# Run each phase in order
# Verify at end

# Option 2: Execute all commands at once
# Save Phase 2, 3, 4 command sections to shell script
# Run script
# Verify with bd list

# Option 3: Manual beads UI
# Read checklist
# Create tasks one-by-one in bd web UI
# Verify dependencies
```

**All three documents ready in your workspace.**

**Status:** ✅ Ready  
**Effort:** 30 minutes  
**Risk:** Very low (straightforward task creation, no code changes)  
**Benefit:** 19 actionable work items tracked, clear critical path, 100% plan coverage in issue tracking
