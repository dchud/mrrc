# Beads Implementation Checklist

**Purpose:** Step-by-step instructions to update beads based on revised GIL Release implementation plan  
**Prerequisite Documents:**
- `docs/design/GIL_RELEASE_HYBRID_IMPLEMENTATION_PLAN_REVISIONS.md` (detailed plan)
- `docs/design/GIL_RELEASE_HYBRID_IMPLEMENTATION_PLAN_WITH_BEADS_MAPPING.md` (beads mapping)
- `BEADS_ACTION_SUMMARY.md` (this checklist + quick reference)

**Timeline:** 30 minutes to execute all steps

---

## Phase 1: Verification (5 minutes)

### ✅ Step 1.1: Confirm Current State
```bash
# Verify Phase C epic exists
bd list --json | jq '.[] | select(.id == "mrrc-ppp")'

# Expected: Open epic with title "Phase C: Batch Reading..."
# If not found: Create with: bd create "Phase C: Batch Reading - Enable True Parallelism via GIL Amortization" -t epic -p 1

# Verify duplicate Phase C epic
bd list --json | jq '.[] | select(.id == "mrrc-d0m")'

# Expected: Duplicate epic (can be deleted/closed later)
```

**Checklist:**
- [ ] Phase C epic `mrrc-ppp` confirmed to exist
- [ ] No current Phase H epic exists (expected)
- [ ] Duplicate `mrrc-d0m` identified for closure

---

## Phase 2: Create Phase C Subtasks (10 minutes)

### ✅ Step 2.1: C.0 - Data Structure & State Machine
```bash
C0_ID=$(bd create "C.0: Data Structure, State Machine & GIL Verification Test" \
  --parent mrrc-ppp -t task -p 1 \
  --deps discovered-from:mrrc-18s \
  --json | jq -r '.id')

echo "C.0 ID: $C0_ID"
```

**Expected Output:** Task ID like `mrrc-xyz`  
**Checklist:**
- [ ] C.0 created
- [ ] Depends on mrrc-18s (SmallVec prerequisite)

### ✅ Step 2.2: C.1 - Implement read_batch()
```bash
C1_ID=$(bd create "C.1: Implement read_batch() Method with Single GIL Cycle" \
  --parent mrrc-ppp -t task -p 1 \
  --deps discovered-from:$C0_ID \
  --json | jq -r '.id')

echo "C.1 ID: $C1_ID"
```

**Checklist:**
- [ ] C.1 created
- [ ] Depends on C.0

### ✅ Step 2.3: C.2 - Queue FSM Integration
```bash
C2_ID=$(bd create "C.2: Update __next__() to Use Queue & EOF State Machine" \
  --parent mrrc-ppp -t task -p 1 \
  --deps discovered-from:$C1_ID \
  --json | jq -r '.id')

echo "C.2 ID: $C2_ID"
```

**Checklist:**
- [ ] C.2 created
- [ ] Depends on C.1

### ✅ Step 2.4: C.3 - Iterator Semantics
```bash
C3_ID=$(bd create "C.3: Iterator Semantics & Idempotence Verification" \
  --parent mrrc-ppp -t task -p 1 \
  --deps discovered-from:$C2_ID \
  --json | jq -r '.id')

echo "C.3 ID: $C3_ID"
```

**Checklist:**
- [ ] C.3 created
- [ ] Depends on C.2

### ✅ Step 2.5: C.4 - Memory Profiling (Parallel)
```bash
C4_ID=$(bd create "C.4: Memory Profiling & Bounds Validation" \
  --parent mrrc-ppp -t task -p 2 \
  --json | jq -r '.id')

echo "C.4 ID: $C4_ID"
```

**Note:** No dependency (runs parallel with C.2-C.3)  
**Checklist:**
- [ ] C.4 created
- [ ] Priority set to 2 (lower than critical path)

### ✅ Step 2.6: C.Gate - Benchmarking Gate
```bash
CG_ID=$(bd create "C.Gate: Benchmark Batch Sizes (10-500 sweep) - ≥1.8x Speedup Gate" \
  --parent mrrc-ppp -t task -p 1 \
  --deps discovered-from:$C3_ID,discovered-from:$C4_ID \
  --json | jq -r '.id')

echo "C.Gate ID: $CG_ID"
```

**Critical:** This ID needed for Phase H epic  
**Checklist:**
- [ ] C.Gate created
- [ ] Depends on both C.3 and C.4
- [ ] Copy $CG_ID value for Phase 3

---

## Phase 3: Create Phase H Epic & Subtasks (15 minutes)

### ✅ Step 3.1: Create Phase H Epic
```bash
H_EPIC=$(bd create "Phase H: Pure Rust I/O & Rayon Parallelism" \
  -t epic -p 1 \
  --deps discovered-from:$CG_ID \
  --json | jq -r '.id')

echo "Phase H Epic ID: $H_EPIC"
```

**Critical:** This ID needed for all H subtasks  
**Checklist:**
- [ ] Phase H epic created
- [ ] Depends on C.Gate
- [ ] Copy $H_EPIC value for subtasks below

### ✅ Step 3.2: H.0 - Rayon PoC
```bash
H0_ID=$(bd create "H.0: Rayon PoC - Thread Pool & Channel Pipeline Validation" \
  --parent $H_EPIC -t task -p 1 \
  --json | jq -r '.id')

echo "H.0 ID: $H0_ID"
```

**Note:** Can start immediately (no dependencies on Phase C)  
**Checklist:**
- [ ] H.0 created

### ✅ Step 3.3: H.1 - Type Detection
```bash
H1_ID=$(bd create "H.1: ReaderBackend Enum & Type Detection Algorithm" \
  --parent $H_EPIC -t task -p 1 \
  --deps discovered-from:$H0_ID \
  --json | jq -r '.id')

echo "H.1 ID: $H1_ID"
```

**Checklist:**
- [ ] H.1 created
- [ ] Depends on H.0

### ✅ Step 3.4: H.2 - RustFile Backend
```bash
H2_ID=$(bd create "H.2: RustFile Backend Implementation - Sequential Read" \
  --parent $H_EPIC -t task -p 1 \
  --deps discovered-from:$H1_ID \
  --json | jq -r '.id')

echo "H.2 ID: $H2_ID"
```

**Checklist:**
- [ ] H.2 created
- [ ] Depends on H.1

### ✅ Step 3.5: H.2b - CursorBackend
```bash
H2B_ID=$(bd create "H.2b: CursorBackend Implementation - Memory-Mapped & Bytes" \
  --parent $H_EPIC -t task -p 1 \
  --deps discovered-from:$H1_ID \
  --json | jq -r '.id')

echo "H.2b ID: $H2B_ID"
```

**Checklist:**
- [ ] H.2b created
- [ ] Depends on H.1 (parallel with H.2)

### ✅ Step 3.6: H.3 - Sequential Baseline
```bash
H3_ID=$(bd create "H.3: Sequential Baseline & Parity Tests (RustFile + CursorBackend)" \
  --parent $H_EPIC -t task -p 1 \
  --deps discovered-from:$CG_ID,discovered-from:$H2_ID,discovered-from:$H2B_ID \
  --json | jq -r '.id')

echo "H.3 ID: $H3_ID"
```

**Critical:** H.3 is blocked by C.Gate  
**Checklist:**
- [ ] H.3 created
- [ ] Depends on C.Gate, H.2, H.2b

### ✅ Step 3.7: H.4a - Boundary Scanner
```bash
H4A_ID=$(bd create "H.4a: Record Boundary Scanner (0x1E delimiter + multi-threaded)" \
  --parent $H_EPIC -t task -p 1 \
  --json | jq -r '.id')

echo "H.4a ID: $H4A_ID"
```

**Checklist:**
- [ ] H.4a created

### ✅ Step 3.8: H.4b - Rayon Parser Pool
```bash
H4B_ID=$(bd create "H.4b: Rayon Parser Pool - Parallel Batch Processing" \
  --parent $H_EPIC -t task -p 1 \
  --deps discovered-from:$H4A_ID \
  --json | jq -r '.id')

echo "H.4b ID: $H4B_ID"
```

**Checklist:**
- [ ] H.4b created
- [ ] Depends on H.4a

### ✅ Step 3.9: H.4c - Producer-Consumer Pipeline
```bash
H4C_ID=$(bd create "H.4c: Producer-Consumer Pipeline - Backpressure & Channels" \
  --parent $H_EPIC -t task -p 1 \
  --deps discovered-from:$H4B_ID \
  --json | jq -r '.id')

echo "H.4c ID: $H4C_ID"
```

**Checklist:**
- [ ] H.4c created
- [ ] Depends on H.4b

### ✅ Step 3.10: H.5 - Integration Tests
```bash
H5_ID=$(bd create "H.5: Integration Tests & Error Propagation Validation" \
  --parent $H_EPIC -t task -p 1 \
  --deps discovered-from:$H4C_ID \
  --json | jq -r '.id')

echo "H.5 ID: $H5_ID"
```

**Checklist:**
- [ ] H.5 created
- [ ] Depends on H.4c

### ✅ Step 3.11: H.Gate - Parallel Benchmarking
```bash
HG_ID=$(bd create "H.Gate: Parallel Benchmarking - ≥2.5x Speedup (4-thread) Gate" \
  --parent $H_EPIC -t task -p 1 \
  --deps discovered-from:$H5_ID \
  --json | jq -r '.id')

echo "H.Gate ID: $HG_ID"
```

**Checklist:**
- [ ] H.Gate created
- [ ] Depends on H.5
- [ ] Copy $HG_ID for Phase 4

---

## Phase 4: Create Infrastructure Tasks (5 minutes)

### ✅ Step 4.1: Diagnostic Test Suite
```bash
DIAG_ID=$(bd create "Infrastructure: Diagnostic Test Suite (GIL verification, benchmarking utilities)" \
  -t task -p 1 \
  --json | jq -r '.id')

echo "Diagnostic ID: $DIAG_ID"
```

**Note:** Supports C.0 task  
**Checklist:**
- [ ] Diagnostic task created

### ✅ Step 4.2: Memory Safety CI
```bash
MEM_ID=$(bd create "Infrastructure: Memory Safety - ASAN/Valgrind CI Integration" \
  -t task -p 2 \
  --json | jq -r '.id')

echo "Memory Safety ID: $MEM_ID"
```

**Note:** Supports C.4 task  
**Checklist:**
- [ ] Memory Safety task created

---

## Phase 5: Update Existing Epics (5 minutes)

### ✅ Step 5.1: Update Phase G (Documentation Refresh)
Current ID: `mrrc-9wi.6`

**Action:** Edit description to add Phase H.Gate blocker

**Command to find it:**
```bash
bd list --json | jq '.[] | select(.id == "mrrc-9wi.6")'
```

**Update Required:**
Add to description:
```
BLOCKER: Phase H.Gate (mrrc-hhh.Gate) must complete before Phase G release.

Reason: Phase G documentation must cover:
1. Phase H threading model and parallelism
2. RAYON_NUM_THREADS environment variable tuning
3. Backpressure explanation for producer-consumer pipeline
4. Performance results from both Phase C and Phase H benchmarking
```

**How to update:** (Manual step - beads may require manual edit or bd update command)
- [ ] Phase G description updated with Phase H.Gate blocker

### ✅ Step 5.2: Verify Phase E & F (No changes needed)
```bash
# Phase E should depend on Phase D ✓
bd list --json | jq '.[] | select(.id == "mrrc-9wi.4")'

# Phase F should depend on Phase E ✓
bd list --json | jq '.[] | select(.id == "mrrc-9wi.5")'
```

**Checklist:**
- [ ] Phase E confirmed (no changes)
- [ ] Phase F confirmed (no changes)
- [ ] Phase G blocker added

---

## Phase 6: Verification & Cleanup (5 minutes)

### ✅ Step 6.1: Verify Dependency Chain
```bash
# Check Phase C dependencies
bd dep show mrrc-ppp

# Expected output: mrrc-18s → C.0 → C.1 → C.2 → C.3 → C.Gate
```

**Checklist:**
- [ ] Phase C chain correct

### ✅ Step 6.2: Verify Phase H Dependencies
```bash
# Check Phase H epic
bd dep show $H_EPIC

# Expected: C.Gate → H.3, H.0 independent, etc.
```

**Checklist:**
- [ ] Phase H chain correct
- [ ] H.3 blocked by C.Gate
- [ ] H.0 can start independently

### ✅ Step 6.3: List All Phase C & H Tasks
```bash
# Count Phase C tasks
bd list --json | jq '.[] | select(.parent == "mrrc-ppp")' | jq '.id' | wc -l
# Expected: 6 (C.0, C.1, C.2, C.3, C.4, C.Gate)

# List Phase C
bd list --json | jq '.[] | select(.parent == "mrrc-ppp") | {id, title}'
```

**Checklist:**
- [ ] Exactly 6 Phase C subtasks created
- [ ] Phase H epic visible
- [ ] All 10 Phase H subtasks visible

### ✅ Step 6.4: Cleanup - Optional (Close Duplicate)
```bash
# OPTIONAL: Close duplicate Phase C epic
# Note: beads may not support delete; use close instead
# bd update mrrc-d0m --status closed --reason "Duplicate of mrrc-ppp"
```

**Checklist:**
- [ ] Duplicate mrrc-d0m closed (optional; can do later)

---

## Phase 7: Final Handoff (Documentation)

### ✅ Step 7.1: Create Work Start Document
Create file: `PHASE_C_PHASE_H_START.md`

```markdown
# Phase C & H Implementation Start

**Date:** [TODAY]  
**Status:** Ready to implement

## Summary
- Phase C Epic: mrrc-ppp with 6 subtasks (C.0-C.Gate)
- Phase H Epic: [Insert ID] with 10 subtasks (H.0-H.Gate)
- All dependencies configured in beads

## Critical Path
1. Start Phase C immediately (Day 1)
2. Start Phase H.0 PoC immediately (Day 1, independent)
3. C.Gate must complete before H.3 starts
4. H.Gate must complete before Phase G release

## Baseline Measurements
- Before starting Phase C: Establish Phase B baseline if not already done
  Command: `cargo bench --bench baseline --release -- --save-baseline=phase_b`

## Next Steps
1. Begin C.0: Data Structure & State Machine
2. Run diagnostic tests as created
3. Track progress via bd (beads)
```

**Checklist:**
- [ ] Handoff document created
- [ ] All instructions preserved
- [ ] Team has clear next steps

---

## Verification Checklist (Before Declaring Complete)

### Core Tasks Created
- [ ] C.0 - Data Structure & State Machine
- [ ] C.1 - read_batch()
- [ ] C.2 - Queue FSM
- [ ] C.3 - Idempotence
- [ ] C.4 - Memory Profiling
- [ ] C.Gate - Batch Size Benchmarking
- [ ] H.0 - Rayon PoC
- [ ] H.1 - Type Detection
- [ ] H.2 - RustFile
- [ ] H.2b - CursorBackend
- [ ] H.3 - Sequential Baseline
- [ ] H.4a - Boundary Scanner
- [ ] H.4b - Rayon Parser Pool
- [ ] H.4c - Producer-Consumer
- [ ] H.5 - Integration Tests
- [ ] H.Gate - Parallel Benchmarking

### Epics Correct
- [ ] Phase C (mrrc-ppp) - 6 subtasks, priority 1
- [ ] Phase H (new) - 10 subtasks, priority 1, depends on C.Gate

### Dependencies Correct
- [ ] C.0 depends on mrrc-18s
- [ ] C.1 depends on C.0
- [ ] C.2 depends on C.1
- [ ] C.3 depends on C.2
- [ ] C.Gate depends on C.3 + C.4
- [ ] Phase H epic depends on C.Gate
- [ ] H.0 independent
- [ ] H.1 depends on H.0
- [ ] H.2 depends on H.1
- [ ] H.2b depends on H.1
- [ ] H.3 depends on C.Gate + H.2 + H.2b
- [ ] H.4a independent
- [ ] H.4b depends on H.4a
- [ ] H.4c depends on H.4b
- [ ] H.5 depends on H.4c
- [ ] H.Gate depends on H.5

### Documentation Updated
- [ ] Phase G description includes H.Gate blocker
- [ ] BEADS_ACTION_SUMMARY.md reviewed
- [ ] GIL_RELEASE_HYBRID_IMPLEMENTATION_PLAN_WITH_BEADS_MAPPING.md created
- [ ] GIL_RELEASE_HYBRID_IMPLEMENTATION_PLAN_REVISIONS.md available

### Ready to Execute
- [ ] All tasks created in beads
- [ ] Dependencies verified
- [ ] Team has clear starting point
- [ ] Handoff documentation complete

---

## Success Metrics

✅ **When complete, you should have:**
1. Phase C epic with 6 ready-to-work subtasks
2. Phase H epic with 10 ready-to-work subtasks
3. Clear dependency chain preventing parallel mistakes
4. C.Gate and H.Gate blocking Phase G release
5. All work items linked to original plan (§ references)

✅ **Ready to start:** Phase C (Day 1) + Phase H.0 PoC (Day 1)

✅ **Timeline:** 5 days (Phase C) + 9 days (Phase H) = 14 days serial, ~8-9 days optimized

---

**Document Status:** Ready to Execute  
**Estimated Execution Time:** 30 minutes  
**Difficulty:** Low (straightforward beads commands)

**Questions?** See `docs/design/GIL_RELEASE_HYBRID_IMPLEMENTATION_PLAN_WITH_BEADS_MAPPING.md` §1-7 for detailed context on each task.
