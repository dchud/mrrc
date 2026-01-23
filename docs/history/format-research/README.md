# Binary Format Research for MARC Data

**Epic:** mrrc-fks (Binary Format Evaluation)  
**Status:** ✅ COMPLETE (9/10 evaluations done; strategy finalized)  
**Last Updated:** 2026-01-19

---

## Purpose

Systematic evaluation of modern binary serialization formats for MARC bibliographic data, assessing round-trip fidelity, robustness, performance, and ecosystem fit. Produces actionable recommendations for format support in mrrc library and Python wrapper.

**Critical Constraint:** Formats that reorder fields or subfield codes are rejected (data loss for semantics-preserving applications). All recommended formats achieve **100% perfect round-trip preservation**.

---

## Document Guide (Start Here)

### For Decision-Makers
1. **[FORMAT_SUPPORT_STRATEGY.md](./FORMAT_SUPPORT_STRATEGY.md)** ← **START HERE**
   - Executive overview and format recommendations
   - Cost/benefit analysis for Tier 1, 2, 3 formats
   - Implementation roadmap (phases, effort, timeline)
   - Final decision matrix (11 days to ship Tier 1 + 2)

2. **[COMPARISON_MATRIX.md](./COMPARISON_MATRIX.md)**
   - Side-by-side performance metrics (read/write/roundtrip speed)
   - File size and compression analysis
   - Memory efficiency benchmarks
   - Use case fit scoring (API, batch processing, analytics, archival)
   - Customer personas and format recommendations

### For Implementers
3. **[EVALUATION_FRAMEWORK.md](./EVALUATION_FRAMEWORK.md)**
   - Standardized evaluation methodology
   - Three-layer assessment (fidelity → robustness → performance)
   - Pass/fail criteria for each layer
   - Edge case definitions (15 MARC-specific test cases)

4. **[FIDELITY_TEST_SET.md](./FIDELITY_TEST_SET.md)**
   - 100-record test collection with edge cases
   - Records designed to stress field/subfield ordering, whitespace, UTF-8
   - Verification procedure for round-trip fidelity

### For Reference
5. **[BASELINE_ISO2709.md](./BASELINE_ISO2709.md)**
   - Baseline performance characteristics (903k rec/sec read)
   - ISO 2709 binary format specification reference
   - Throughput derivation methodology

6. **Individual Format Evaluations:**
   - [EVALUATION_PROTOBUF.md](./EVALUATION_PROTOBUF.md) (mrrc-fks.1) — ✅ Complete
   - [EVALUATION_FLATBUFFERS.md](./EVALUATION_FLATBUFFERS.md) (mrrc-fks.2) — ✅ Complete
   - [EVALUATION_ARROW.md](./EVALUATION_ARROW.md) (mrrc-fks.7) — ✅ Complete
   - [EVALUATION_MESSAGEPACK.md](./EVALUATION_MESSAGEPACK.md) (mrrc-fks.5) — ✅ Complete
   - [EVALUATION_CBOR.md](./EVALUATION_CBOR.md) (mrrc-fks.6) — ✅ Complete
   - [EVALUATION_AVRO.md](./EVALUATION_AVRO.md) (mrrc-fks.4) — ✅ Complete
   - [EVALUATION_PARQUET.md](./EVALUATION_PARQUET.md) (mrrc-ks7) — ✅ Complete
   - [EVALUATION_POLARS_ARROW_DUCKDB.md](./EVALUATION_POLARS_ARROW_DUCKDB.md) (mrrc-fks.10) — ✅ Complete

---

## Evaluation Summary

### Format Decisions

**TIER 1: MUST SHIP (Non-Negotiable)**
- ✅ **ISO 2709** — Baseline; 50+ year proven standard; 900k rec/sec
- ✅ **Protobuf** — Modern API; schema evolution; multi-language; gRPC

**TIER 2: SHIP TOGETHER (High ROI; 7 dev days total)**
- ✅ **Arrow (Columnar)** — 3 days | Analytics + ecosystem standard (865k rec/sec)
- ✅ **FlatBuffers** — 2 days | Mobile/embedded + zero-copy (259k rec/sec; 64% memory savings)
- ✅ **MessagePack** — 2 days | Compact + universal (750k rec/sec; 25% file size savings)

**TIER 3: DEFER (Implement on customer demand only)**
- ⏸️ **CBOR** — Government/academic archival (2 days)
- ⏸️ **Avro** — Kafka data lake integration (2 days)
- ⏸️ **Arrow Analytics** — Discovery optimization (1 day; POC complete)

**EXCLUDE: Do Not Implement**
- ❌ **Parquet** — Redundant with Arrow (use Arrow IPC → external export)
- ❌ **JSON/YAML/XML** — Different project scope (handled in pymarc)
- ❌ **Bincode** — Rust-only; no cross-platform appeal
- ❌ **Ion** — Unclear MARC value; Protobuf superior

### Performance Highlights

| Format | Read Speed | vs ISO 2709 | Memory | File Size | Fidelity |
|--------|------------|------------|--------|-----------|----------|
| **ISO 2709** | 903k rec/sec | Baseline | 45 MB | 2.6 MB | ✅ 100% |
| **Protobuf** | 100k rec/sec | -88.9% | 45-50 MB | 7.5-8.5 MB | ✅ 100% |
| **FlatBuffers** | 259k rec/sec | -71.3% | 16 MB (-64%) | 6.7 MB | ✅ 100% |
| **Arrow** | 865k rec/sec | -4.2% | 30-35 MB | 1.8 MB (-30%) | ✅ 100% |
| **MessagePack** | 750k rec/sec | -17.0% | 40-45 MB | 2.0 MB (-25%) | ✅ 100% |
| **Arrow Analytics** | 1.77M rec/sec | +1.96× | 5.8 MB | 1.8 MB | ✅ 100% |

### Evaluation Methodology

All formats evaluated via **three-layer assessment:**

1. **Layer 1: Fidelity** — 100% round-trip preservation (field/subfield ordering, indicators, UTF-8 content) against 100-record test set with 15 edge cases. **Pass/fail gate.**

2. **Layer 2: Robustness** — Graceful error handling on 7 malformed inputs (truncated records, invalid tags, oversized fields, malformed UTF-8, etc.). No panics. **Pass/fail gate.**

3. **Layer 3: Performance** — Read/write throughput, file size, memory efficiency benchmarks. Only evaluated if Layers 1+2 pass.

---

## File Structure & Cross-References

```
docs/design/format-research/
├── README.md ← YOU ARE HERE
│
├── FRAMEWORK & PLANNING
│   ├── EVALUATION_FRAMEWORK.md     (mrrc-fks.8) — Evaluation methodology
│   ├── FIDELITY_TEST_SET.md        — 100-record test data + edge cases
│   ├── TEMPLATE_evaluation.md      — Blank report template
│   └── REVISIONS_SUMMARY.md        — Document evolution log
│
├── SYNTHESIS & STRATEGY (mrrc-fks.9)
│   ├── COMPARISON_MATRIX.md        — Aggregated results (all formats)
│   └── FORMAT_SUPPORT_STRATEGY.md  — Final recommendations + roadmap
│
├── BASELINES & REFERENCES
│   └── BASELINE_ISO2709.md         — ISO 2709 performance baseline
│
└── INDIVIDUAL EVALUATIONS (Completed)
    ├── EVALUATION_PROTOBUF.md               (mrrc-fks.1)  ✅
    ├── EVALUATION_FLATBUFFERS.md            (mrrc-fks.2)  ✅
    ├── EVALUATION_PARQUET.md                (mrrc-ks7)    ✅
    ├── EVALUATION_AVRO.md                   (mrrc-fks.4)  ✅
    ├── EVALUATION_MESSAGEPACK.md            (mrrc-fks.5)  ✅
    ├── EVALUATION_CBOR.md                   (mrrc-fks.6)  ✅
    ├── EVALUATION_ARROW.md                  (mrrc-fks.7)  ✅
    └── EVALUATION_POLARS_ARROW_DUCKDB.md    (mrrc-fks.10) ✅
```

---

## Key Findings

### Format Tiers Justified by ROI

**Why Tier 1 + 2 (11 days) is sufficient:**
- **ISO 2709 + Protobuf (4-5 days):** Covers 100% of legacy + modern API users
- **Arrow + FlatBuffers + MessagePack (7 days):** Solves distinct personas:
  - Arrow: Data scientists (analytics, DuckDB/Polars integration)
  - FlatBuffers: Mobile/embedded developers (64% memory savings, zero-copy)
  - MessagePack: REST API developers (25% file size, 50+ languages)
- **Tier 3 (on-demand):** Niche verticals (Kafka, government archival) defer without blocking release

### Why Specific Formats Were Excluded

| Format | Decision | Rationale |
|--------|----------|-----------|
| **Parquet** | ❌ EXCLUDE | Redundant with Arrow; user → Arrow IPC → external Parquet (3-line code) |
| **Bincode** | ❌ EXCLUDE | Fast serde (~80% of MessagePack), but Rust-only; no cross-platform appeal |
| **Ion** | ❌ EXCLUDE | Excellent flexibility but low ecosystem (6 languages); Protobuf superior for schema |
| **JSON Lines** | ⏸️ RESEARCH | Post-release; valuable for dev ergonomics but outside binary format scope |
| **Custom MARC Schema** | ⏸️ RESEARCH | Interesting for ISO 2709 evolution; requires community buy-in |

---

## Implementation Roadmap (Format Support Strategy)

**Phase 0 (Foundation):** 1.5 days — Traits, module structure, test fixtures  
**Phase 1 (Core):** 3-5 days — ISO 2709 refactor + Protobuf  
**Phase 2 (High-Value):** 6-8 days — Arrow, FlatBuffers, MessagePack (parallelizable)  
**Phase 4 (Polish):** 5-7 days — Python wrapper + documentation  

**Critical Path:** 15-18 days wall time (Tier 1 + 2 complete)  
**MVP Option:** 7-8 days (Tier 1 only; ship Tier 2 in v1.1 if needed)

See [FORMAT_SUPPORT_STRATEGY.md](./FORMAT_SUPPORT_STRATEGY.md) Part 5 for detailed task breakdown.

---

## Related Issues (Closed)

| Issue | Title | Status |
|-------|-------|--------|
| **mrrc-fks** | Binary Format Evaluation Epic | ✅ Closed |
| **mrrc-fks.1** | Protobuf Evaluation | ✅ Complete |
| **mrrc-fks.2** | FlatBuffers Evaluation | ✅ Complete |
| **mrrc-fks.3** | Parquet Evaluation | ✅ Complete |
| **mrrc-fks.4** | Avro Evaluation | ✅ Complete |
| **mrrc-fks.5** | MessagePack Evaluation | ✅ Complete |
| **mrrc-fks.6** | CBOR Evaluation | ✅ Complete |
| **mrrc-fks.7** | Arrow Evaluation | ✅ Complete |
| **mrrc-fks.8** | Evaluation Framework | ✅ Complete |
| **mrrc-fks.9** | Format Strategy & Recommendations | ✅ Complete |
| **mrrc-fks.10** | Arrow Analytics (Polars/DuckDB) | ✅ Complete |

---

## Follow-Up Work (Not Blocking)

Identified during evaluation but deferred:

- **mrrc-fks.11:** Streaming Arrow IPC evaluation (>100M records)
- **mrrc-fks.12:** Protobuf/FlatBuffers schema evolution upgrade testing
- **mrrc-fks.13:** Cross-language round-trip verification (Rust ↔ Python ↔ Java)
- **Optional:** Bincode POC, JSON Lines benchmarking, Custom MARC Binary Schema research

---

## How to Use This Research

**For Release Planning:**
→ Read [FORMAT_SUPPORT_STRATEGY.md](./FORMAT_SUPPORT_STRATEGY.md) Part 10 (Final Recommendations)

**For Implementation:**
→ Use [FORMAT_SUPPORT_STRATEGY.md](./FORMAT_SUPPORT_STRATEGY.md) Part 5 (Detailed Phase Breakdown)

**For Performance Analysis:**
→ Consult [COMPARISON_MATRIX.md](./COMPARISON_MATRIX.md) (aggregated metrics)

**For Customer Questions:**
→ Use [COMPARISON_MATRIX.md](./COMPARISON_MATRIX.md) customer personas (Part 7.2)

**For Format-Specific Details:**
→ Read individual evaluation documents (e.g., EVALUATION_PROTOBUF.md for Protobuf design choices)

---

## Document Metadata

| Item | Value |
|------|-------|
| **Framework Version** | 2.0 (mrrc-fks.8, 2026-01-14) |
| **Comparison Matrix Version** | 2.2 (mrrc-fks.9, 2026-01-19) |
| **Strategy Version** | 1.0 (mrrc-fks.9 follow-up, 2026-01-19) |
| **Total Evaluation Effort** | ~120+ person-hours (8 formats evaluated) |
| **Formats Evaluated** | 8 (9 with Arrow Analytics) |
| **Formats Recommended** | 7 (Tier 1 + 2 + Analytics) |
| **Edge Cases Tested** | 15 per format |
| **Test Records** | 100+ per format |
| **Baseline Throughput** | 903,560 rec/sec (ISO 2709) |

---

## Questions?

- **Implementation details?** → See FORMAT_SUPPORT_STRATEGY.md Part 5 (Phase breakdown)
- **Performance comparisons?** → See COMPARISON_MATRIX.md (performance metrics)
- **Why was format X excluded?** → See FORMAT_SUPPORT_STRATEGY.md Part 1.3 (exclusions + rationale)
- **What's the timeline?** → See FORMAT_SUPPORT_STRATEGY.md Part 10.3 (timeline + resources)
- **How do I add a new format?** → See EVALUATION_FRAMEWORK.md + TEMPLATE_evaluation.md
