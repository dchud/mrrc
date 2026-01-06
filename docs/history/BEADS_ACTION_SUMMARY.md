# Beads Integration Action Summary

**Date:** January 2, 2026  
**Status:** Ready to Execute  
**Document:** See `docs/design/GIL_RELEASE_HYBRID_IMPLEMENTATION_PLAN_WITH_BEADS_MAPPING.md` for full details

---

## Critical Issues Found

### 1. Phase C Epic: Lacks Subtasks (Critical)
- **Current State:** `mrrc-ppp` epic exists but has NO subtasks
- **Plan Requirement:** 6 subtasks (C.0 through C.Gate) with dependency chain
- **Impact:** Cannot track progress; no clear definition of work

### 2. Duplicate Phase C Epic (Cleanup)
- **Current State:** Both `mrrc-ppp` and `mrrc-d0m` (identical title)
- **Action:** Close `mrrc-d0m` with reason "Duplicate of mrrc-ppp"

### 3. Phase H Epic: Does Not Exist (Critical)
- **Current State:** No Phase H epic or subtasks in beads
- **Plan Requirement:** 1 epic + 10 subtasks with full dependency chain
- **Impact:** Phase H work cannot be tracked or prioritized

### 4. Diagnostic Infrastructure: Missing (High)
- **Current State:** No tasks for test suite setup or memory safety CI
- **Plan Requirement:** Explicit infrastructure tasks for C.0 and C.Gate gates
- **Tasks Needed:**
  - Infrastructure: Diagnostic Test Suite (tests/concurrent_gil_tests.rs, scripts/benchmark_batch_sizes.py, etc.)
  - Infrastructure: Memory Safety - ASAN/Valgrind CI Integration

### 5. Phase G Dependency: Incorrect (Medium)
- **Current State:** Phase G only depends on Phase F
- **Plan Requirement:** Phase G must also depend on Phase H.Gate
- **Action:** Update `mrrc-9wi.6` description to reference Phase H.Gate blocker

### 6. Batch Size Gate Criterion: Inconsistent (Medium)
- **Current State:** Phase F (mrrc-pfw) references ≥2.0x speedup for Phase C decision
- **Plan Requirement:** Phase C.Gate uses ≥1.8x speedup as target
- **Action:** Clarify that Phase F is pre-C; Phase C.Gate is post-C with 1.8x target

---

## Required Beads Operations

### Immediate (Today)

#### 1. Create Phase C Subtasks under `mrrc-ppp`
```bash
# C.0: Foundation (1 day)
bd create "C.0: Data Structure, State Machine & GIL Verification Test" \
  --parent mrrc-ppp -t task -p 1 --deps discovered-from:mrrc-18s

# C.1: read_batch (1-2 days) - depends on C.0
bd create "C.1: Implement read_batch() Method with Single GIL Cycle" \
  --parent mrrc-ppp -t task -p 1

# C.2: Queue FSM (1 day) - depends on C.1
bd create "C.2: Update __next__() to Use Queue & EOF State Machine" \
  --parent mrrc-ppp -t task -p 1

# C.3: Idempotence (1 day) - depends on C.2
bd create "C.3: Iterator Semantics & Idempotence Verification" \
  --parent mrrc-ppp -t task -p 1

# C.4: Memory Profiling (1 day) - parallel with C.2-C.3
bd create "C.4: Memory Profiling & Bounds Validation" \
  --parent mrrc-ppp -t task -p 2

# C.Gate: Benchmarking (1 day) - depends on C.3, C.4
bd create "C.Gate: Benchmark Batch Sizes (10-500 sweep) - ≥1.8x Speedup Gate" \
  --parent mrrc-ppp -t task -p 1
```

#### 2. Create Phase H Epic + All Subtasks
```bash
# Phase H Epic (depends on C.Gate)
H_EPIC=$(bd create "Phase H: Pure Rust I/O & Rayon Parallelism" \
  -t epic -p 1 --deps discovered-from:mrrc-ppp.Gate --json | jq -r '.id')

# H.0: Rayon PoC (1 day) - can start immediately, independent
bd create "H.0: Rayon PoC - Thread Pool & Channel Pipeline Validation" \
  --parent $H_EPIC -t task -p 1

# H.1: Type Detection (1 day) - depends on H.0
bd create "H.1: ReaderBackend Enum & Type Detection Algorithm" \
  --parent $H_EPIC -t task -p 1

# H.2: RustFile Backend (1-2 days) - depends on H.1
bd create "H.2: RustFile Backend Implementation - Sequential Read" \
  --parent $H_EPIC -t task -p 1

# H.2b: CursorBackend (1 day) - depends on H.1
bd create "H.2b: CursorBackend Implementation - Memory-Mapped & Bytes" \
  --parent $H_EPIC -t task -p 1

# H.3: Sequential Baseline (1 day) - depends on C.Gate, H.2, H.2b
bd create "H.3: Sequential Baseline & Parity Tests (RustFile + CursorBackend)" \
  --parent $H_EPIC -t task -p 1

# H.4a: Boundary Scanner (1 day)
bd create "H.4a: Record Boundary Scanner (0x1E delimiter + multi-threaded)" \
  --parent $H_EPIC -t task -p 1

# H.4b: Rayon Parser Pool (1 day) - depends on H.4a
bd create "H.4b: Rayon Parser Pool - Parallel Batch Processing" \
  --parent $H_EPIC -t task -p 1

# H.4c: Producer-Consumer Pipeline (1-2 days) - depends on H.4b
bd create "H.4c: Producer-Consumer Pipeline - Backpressure & Channels" \
  --parent $H_EPIC -t task -p 1

# H.5: Integration Tests (1 day) - depends on H.4c
bd create "H.5: Integration Tests & Error Propagation Validation" \
  --parent $H_EPIC -t task -p 1

# H.Gate: Parallel Benchmarking (1 day) - depends on H.5
bd create "H.Gate: Parallel Benchmarking - ≥2.5x Speedup (4-thread) Gate" \
  --parent $H_EPIC -t task -p 1
```

#### 3. Create Infrastructure Tasks
```bash
# Diagnostic Test Suite (prerequisite to C.0)
bd create "Infrastructure: Diagnostic Test Suite (GIL verification, benchmarking utilities)" \
  -t task -p 1

# Memory Safety CI (prerequisite to C.4)
bd create "Infrastructure: Memory Safety - ASAN/Valgrind CI Integration" \
  -t task -p 2
```

### Soon (Within 2 hours)

#### 4. Update Phase G Description
Edit `mrrc-9wi.6` to add:
- Blocker: Phase H.Gate must complete
- Doc sections: Add Phase H threading model, RAYON_NUM_THREADS tuning, backpressure explanation

#### 5. Update Phase F Description (Optional)
Add note: "Phase C decision gate uses ≥1.8x speedup criterion (after Phase C implementation). This Phase F benchmark informs whether Phase C is needed at all."

---

## Dependency Summary

```
Current Phase Chain:
A (Buffer) → B (GIL Release) → D (Writer) → E (Validation) → F (Benchmarks) → G (Docs)

Revised Chain with Phase C & H:
A → B → D → E → F → [Decision: Phase C needed?]
                 ↓
            Phase C → H.0 PoC
                ↓        ↓
            C.Gate    H.1-H.2b (parallel)
                ↓        ↓
            H.3 ← ─ ─ ─┘
                ↓
            H.4a-c
                ↓
            H.Gate
                ↓
                G (Docs)
```

**Critical Blockers:**
- C.Gate blocks H.3 (sequential baseline needs C complete)
- H.Gate blocks Phase G release (docs must cover Phase H threading model)

---

## Metrics & Gates

| Gate | When | Target | Blocker |
|------|------|--------|---------|
| **C.Gate** | End of Phase C | ≥1.8x speedup (2-thread) | Blocks H.3 start |
| **H.Gate** | End of Phase H | ≥2.5x speedup (4-thread) | Blocks Phase G release |

---

## Estimated Timeline

| Phase | Duration | Status | Blocker |
|-------|----------|--------|---------|
| Phase C (full) | 5 days | Ready to start | None (can start Day 1) |
| Phase H.0 PoC | 1 day | Can start immediately | None |
| Phase H.1-H.2b | 3 days | Can start Day 1 | None (H.0 completion advised) |
| Phase H.3 | 1 day | Can start Day 6 | Waits for C.Gate pass |
| Phase H.4a-c | 3 days | Can start Day 7 | H.3 completion |
| Phase H.Gate | 1 day | Day 10 | H.5 completion |
| **Total (serial)** | 14 days | - | - |
| **Total (optimized)** | 8-9 days | - | C.Gate + 1 day for H.3 delay |

---

## Document References

Full details in: `docs/design/GIL_RELEASE_HYBRID_IMPLEMENTATION_PLAN_WITH_BEADS_MAPPING.md`

Sections:
- §1: Detailed beads mapping (what exists, what's missing)
- §2: Beads operations (deletions, creations, updates)
- §3: Task breakdown with plan references (§3.1-3.4 for C, §4 for H)
- §4: Existing phase updates (E, F, G)
- §5: Sequencing recommendations (critical path, parallel opportunities)
- §6: Metrics & gates
- §7: Next steps (immediate actions)

---

## Before You Start

1. Review `GIL_RELEASE_HYBRID_IMPLEMENTATION_PLAN_REVISIONS.md` (original detailed plan)
2. Review this summary + full mapping document
3. Run beads commands above in order
4. Verify dependency chain: `bd dep show mrrc-ppp && bd dep show $H_EPIC`
5. Start with Phase C (Day 1) or Phase H.0 PoC (Day 1) in parallel

---

**Ready to Execute:** Yes  
**Confidence Level:** High (all gaps identified, all tasks specified, all dependencies documented)  
**Questions:** Refer to detailed mapping document; all §/line references verified against original plan
