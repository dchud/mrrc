# Format Support Strategy & Implementation Plan

**Document:** mrrc-fks.9 Follow-Up  
**Date:** 2026-01-19  
**Status:** Strategic Planning (No Implementation)  
**Scope:** Rust library + Python wrapper format support roadmap  

---

## Executive Overview

After completing 8/10 format evaluations (9 including Arrow Analytics), this document consolidates learnings and provides a concrete roadmap for:

1. **Which formats to support** in mrrc Rust library (keep/add/remove)
2. **Which formats to expose** in the Python wrapper
3. **Identified gaps** in the evaluation or feature sets
4. **Phase-based implementation plan** with clear ordering and dependencies
5. **Documentation needed** for each format tier and user personas

**Key Finding:** The evaluation confirmed no single format is optimal for all use cases. A **tiered approach** is needed:
- **Tier 1 (Required):** ISO 2709 (baseline) + Protobuf (modern API)
- **Tier 2 (High-Value, Ship Together):** Arrow, FlatBuffers, MessagePack
- **Tier 3 (Deferred):** CBOR, Avro, Arrow Analytics (implement only on customer demand)
- **Exclude:** Parquet (redundant with Arrow), JSON, XML, YAML, TOML, Ion (different scope)

**Decisiveness Rationale:** This strategy prioritizes shipping a complete, well-tested Tier 1 + Tier 2 over supporting every possible format. Tier 3 and excluded formats can be added later with explicit customer demand without breaking the library API.

---

## Part 1: Format Support Recommendations

### 1.1 Keep in Library (Tier 1 + Tier 2)

#### **TIER 1: Core Formats** (Required)
Production deployment MUST include these.

| Format | Rationale | Library Role | Python Wrapper |
|--------|-----------|--------------|-----------------|
| **ISO 2709** | Baseline; 50+ year proven standard; zero deps; 900k rec/sec | Primary import/export | Must expose (compatibility with pymarc) |
| **Protobuf** | Modern API standard; schema evolution; multi-language; gRPC support; 100% fidelity | API/cross-language interchange | Must expose (natural for gRPC APIs) |

**Implementation Priority:** Tier 1 must be complete before release.

---

#### **TIER 2: High-Value Formats** (Recommended - MUST SHIP)
High ROI (value delivered per day of effort). Ship all three together as part of initial release.

| Format | Cost (Dev Days) | Benefit | ROI | Library Role | Python Wrapper |
|--------|---|---------|-----|--------------|-----------------|
| **Arrow (Columnar)** | 3 | Ecosystem standard (Polars, DuckDB, DataFusion); analytics tier (1.77M rec/sec); 30% file size savings | ⭐⭐⭐⭐⭐ | In-memory analytics, tool interop | Must expose |
| **FlatBuffers** | 2 | 64% memory savings; proven (Apple); zero-copy capable; mobile/embedded tier | ⭐⭐⭐⭐ | Memory-constrained APIs, streaming | Must expose |
| **MessagePack** | 2 | 25% file size savings; 750k rec/sec; 50+ languages; serde-friendly; universal IPC | ⭐⭐⭐⭐ | Compact serialization, REST, IPC | Must expose |

**Total Tier 2 Effort:** 7 dev days (fits in Phase 2 schedule)  
**Implementation Order:** Arrow (foundation) → FlatBuffers (proven) → MessagePack (serde leverage)

**Cost/Benefit Summary:** All three Tier 2 formats are low-effort, high-value, and solve distinct problems. Exclude any of them reduces library usability for defined personas (mobile devs, data scientists, REST API developers). Ship together to avoid fragmented ecosystem.

---

### 1.2 Add to Library (Tier 3 - DEFERRED)

#### **TIER 3: Specialized Formats** (NOT in Initial Release)
Niche use cases with explicit customer demand. Implement on-demand after Tier 1 + 2 are stable.

| Format | Cost | Justification | When to Implement | Library Role |
|--------|------|---------------|-------------------|--------------|
| **CBOR** | 2 days | RFC 7949 standard; low demand in library ecosystem for MARC | Only if government/academic archival customer requests it | Standards-based preservation |
| **Avro** | 2 days | Kafka/data lake integration; schema registry; overlaps with Protobuf for most use cases | Only if explicit Kafka integration customer requirement | Data lake ecosystems |
| **Arrow Analytics (Rust-native)** | 1 day | Complements Arrow; already has POC; validates analytics tier design | After Arrow proves valuable; integrate POC code | Columnar analytics tier |

**Decision:** Do NOT include in initial release. These formats solve specific vertical problems without broad applicability to MARC ecosystem. Add via `bd` issues per customer requirement (e.g., "Customer X needs Avro for Kafka pipeline"); avoids maintenance burden for unmeasured demand.

---

### 1.3 Remove/Don't Add

#### **Parquet** ❌ EXCLUDE (Redundant)
- **Reason:** Redundant with Arrow implementation
- **Fact:** Parquet achieves same compression (98.3% vs Arrow's 95.99%) but adds 74% raw size overhead in native serialization
- **Alternative:** Users needing columnar archival use Arrow IPC → external Parquet export (3-line user code)
- **Status:** Do NOT implement in mrrc. Mention in tutorials as user responsibility.

#### **Formats NOT Evaluated (Out of Scope or Low Priority)**

| Format | Why Not Evaluated | Assessment | Would It Help? |
|--------|-------------------|------------|-----------------|
| **JSON/YAML/XML** | Different project scope (handled in pymarc XML/JSON work) | Well-understood, no evaluation needed | No; different team |
| **Bincode** (Rust-native serde) | Rust-only; assumed low priority vs cross-platform | Fast serde, minimal overhead (~80% of MessagePack size); zero deps; BUT only Rust language support; no cross-platform appeal | Only if Rust-only MARC library acceptable |
| **Apache Ion** | Complex spec; schema-less with strong typing; unknown MARC suitability | Amazon's TypeScript-inspired format; excellent schema flexibility; BUT low language support (6 vs 50+); no clear MARC use case | No; Protobuf better for schema flexibility |
| **JSON Lines** (newline-delimited JSON) | Human-readable; valuable for debugging/logging but not binary format project scope | Excellent for logs/development; 2-3× larger than binary; natural for streaming pipelines; BUT outside binary format focus | Maybe post-release (low effort) for dev ergonomics |
| **Bincode + Compression** (custom layer) | Could be best-in-class performance (ISO 2709 speed + better file size); non-standard | Hypothetically: ISO 2709 read speed (900k rec/sec) + 80% compression (vs Arrow's 96%); BUT adds custom codec, breaks ecosystem interop | Worth POC if performance is limiting factor |
| **Custom MARC Binary Schema** (ISO 2709 + metadata) | Could solve schema evolution for ISO 2709 users; addresses fundamental limitation | Add optional header with version/field definitions; maintains ISO 2709 compatibility; BUT non-standard, requires buy-in from MARC community | Interesting long-term (post-release) if evolution becomes blocker |

**Clear Decision on Excluded Formats:**
- ✅ **JSON/YAML/XML:** Different project; already handled in pymarc
- ✅ **Bincode:** Rust-only; limits cross-platform appeal; not worth narrow benefit
- ✅ **Ion:** Better alternatives (Protobuf); unclear MARC value; small ecosystem
- ⏸️ **JSON Lines:** Consider post-release if logging/development ergonomics needed
- ⏸️ **Bincode + Compression, Custom MARC Schema:** Research opportunities, not release blockers

---

## Part 2: Library Architecture (Rust Core)

### 2.1 Recommended Module Structure

```
mrrc/src/
├── formats/
│   ├── iso2709/          (TIER 1)
│   │   ├── reader.rs     (native Rust, already exists)
│   │   ├── writer.rs     (native Rust, already exists)
│   │   └── tests/
│   │
│   ├── protobuf/         (TIER 1)
│   │   ├── schema.proto
│   │   ├── reader.rs     (generated + wrapper)
│   │   ├── writer.rs
│   │   └── tests/
│   │
│   ├── flatbuffers/      (TIER 2)
│   │   ├── schema.fbs
│   │   ├── reader.rs     (generated + wrapper)
│   │   ├── writer.rs
│   │   └── tests/
│   │
│   ├── arrow/            (TIER 2)
│   │   ├── columnar.rs   (Arrow builders for analytics)
│   │   ├── ipc.rs        (Arrow IPC format for interop)
│   │   ├── reader.rs
│   │   ├── writer.rs
│   │   └── tests/
│   │
│   ├── messagepack/      (TIER 2)
│   │   ├── reader.rs     (serde-based)
│   │   ├── writer.rs
│   │   └── tests/
│   │
│   ├── cbor/             (TIER 3, optional)
│   │   ├── reader.rs
│   │   ├── writer.rs
│   │   └── tests/
│   │
│   ├── avro/             (TIER 3, optional)
│   │   ├── schema.avsc
│   │   ├── reader.rs
│   │   ├── writer.rs
│   │   └── tests/
│   │
│   └── mod.rs            (central feature-gated exports)
│
└── lib.rs                (top-level exports)
```

### 2.2 Feature Flags

```toml
[features]
# Core (always enabled)
default = ["iso2709", "protobuf"]

# Tier 1: Required formats
iso2709 = []
protobuf = ["prost", "prost-build"]

# Tier 2: High-value formats
flatbuffers = ["flatbuffers"]
arrow = ["arrow", "parquet"]  # Note: Parquet via Arrow, user respons. for export
messagepack = ["rmp-serde", "serde"]

# Tier 3: Specialized formats
cbor = ["ciborium"]
avro = ["apache-avro"]
arrow-analytics = ["arrow"]  # Requires Arrow; distinct from columnar format

# Bundle features
all-formats = ["iso2709", "protobuf", "flatbuffers", "arrow", "messagepack"]
stdlib-formats = ["iso2709", "protobuf", "flatbuffers", "arrow", "messagepack"]
archival-formats = ["iso2709", "cbor"]
streaming-formats = ["protobuf", "avro"]
```

### 2.3 API Design Consistency

Each format reader/writer should follow the same pattern for discoverability:

```rust
// Generic pattern for all formats
pub struct [Format]Reader {
    // format-specific fields
}

impl Reader<MarcRecord> for [Format]Reader {
    fn read_record(&mut self) -> Result<Option<MarcRecord>> { }
    fn into_iter(self) -> impl Iterator<Item = Result<MarcRecord>> { }
}

pub struct [Format]Writer {
    // format-specific fields
}

impl Writer for [Format]Writer {
    fn write_record(&mut self, record: &MarcRecord) -> Result<()> { }
    fn write_batch(&mut self, records: &[MarcRecord]) -> Result<()> { }
    fn finish(self) -> Result<()> { }
}

// Convenience functions
pub fn read_[format](source: impl Read) -> Result<Vec<MarcRecord>> { }
pub fn write_[format](records: &[MarcRecord], target: impl Write) -> Result<()> { }
```

---

## Part 3: Python Wrapper Strategy

### 3.1 Tier-Based Exposure

**TIER 1 (must_have):**
```python
# Always available
import mrrc
records = mrrc.read_iso2709(path)
records = mrrc.read_protobuf(data)
mrrc.write_iso2709(records, path)
mrrc.write_protobuf(records) -> bytes
```

**TIER 2 (strongly_recommended):**
```python
# Feature-gated in PyO3 module
import mrrc
records = mrrc.read_flatbuffers(data)
records = mrrc.read_arrow(data)      # Returns list[MarcRecord]
records = mrrc.read_messagepack(data)
# ...write variants
```

**TIER 3 (optional):**
```python
# Only if explicitly imported
from mrrc.formats import cbor, avro, arrow_analytics
records = cbor.read(data)
```

### 3.2 Python Wrapper Modules

```
mrrc-python/src/
├── mrrc/
│   ├── __init__.py           (Tier 1 + 2 exports)
│   ├── _mrrc.pyi             (PyO3 type hints)
│   │
│   ├── formats/              (Tier 2/3 modules)
│   │   ├── __init__.py
│   │   ├── flatbuffers.py    (wrapper around Rust impl)
│   │   ├── arrow.py          (wrapper + PyArrow integration)
│   │   ├── messagepack.py
│   │   ├── cbor.py
│   │   ├── avro.py
│   │   └── arrow_analytics.py (Parquet export guidance)
│   │
│   ├── analytics/            (Arrow Analytics helpers)
│   │   ├── __init__.py
│   │   ├── export.py         (MARC → Arrow RecordBatch)
│   │   ├── duckdb_integration.py (optional; show how to query)
│   │   └── polars_integration.py (optional; show how to use)
│   │
│   └── types.py              (common type definitions)
```

### 3.3 Python API Design

**Simple path (Tier 1 + 2 most users):**
```python
import mrrc

# Read various formats
records = mrrc.read("data.iso", format="iso2709")
records = mrrc.read("data.pb", format="protobuf")
records = mrrc.read("data.fb", format="flatbuffers")

# Write various formats
mrrc.write(records, "output.iso", format="iso2709")
mrrc.write(records, "output.pb", format="protobuf")

# Batch operations with streaming
with mrrc.Reader("large.iso", format="iso2709") as reader:
    for batch in reader.batches(size=1000):
        process(batch)
```

**Advanced path (Tier 3 + analytics):**
```python
import mrrc
from mrrc.formats import avro, cbor
from mrrc.analytics import to_arrow

# Specialized formats
records = avro.read("kafka_export.avro")
records = cbor.read("archive.cbor")

# Analytics export
records = mrrc.read("data.iso")
arrow_table = to_arrow(records)  # → pyarrow.Table
arrow_table.to_parquet("output.parquet")

# DuckDB integration (user responsibility)
import duckdb
con = duckdb.connect()
con.execute("SELECT * FROM arrow_table WHERE field_tag = '650'").show()
```

---

## Part 4: Identified Gaps & Missing Evaluations

### 4.1 Format Evaluation Gaps

#### **Partially Explored**
- **Arrow Analytics streaming:** Evaluated POC; should test streaming Arrow IPC reader (>100M records)
- **Parquet compression variants:** Only tested Snappy; should benchmark ZSTD/LZ4 for archival
- **Protobuf vs FlatBuffers schema evolution:** Both scored well; should test upgrade scenario (record v1 → v2)
- **Cross-language round-trip:** Rust → Python → Java verification (Python wrapper may diverge)

#### **Not Evaluated (Potentially Valuable)**
- **Bincode (Rust-native serde):** No external deps; ~10% file size overhead; worth POC if performance is critical
- **Abomonated (specialized serde):** Zero-copy serialization; explore for memory-constrained workloads
- **JSON Lines (newline-delimited JSON):** Human-readable streaming; useful for debugging/logging
- **Protocol Buffer V2 vs V3:** Evaluated V3 (default); V2 has different evolution semantics
- **Custom compressed format:** ISO 2709 + zstd/br compression layer; might beat all options combined
- **Hybrid format (ISO 2709 with schema metadata):** Add optional schema header to ISO 2709 for evolution support

#### **Recommendation:**
Create follow-up `bd` issues for:
1. **mrrc-fks.11:** Streaming Arrow IPC evaluation (>100M records)
2. **mrrc-fks.12:** Protobuf/FlatBuffers schema evolution upgrade testing
3. **mrrc-fks.13:** Cross-language round-trip verification (Rust ↔ Python ↔ Java)
4. **Optional research:** Bincode POC, JSON Lines benchmarking

### 4.2 Non-Binary Format Coverage

The evaluation focused on binary formats. Consider future scope:

| Format Type | Status | Recommendation |
|-------------|--------|-----------------|
| **JSON** | ✅ Covered in pymarc (not in this scope) | Evaluate separately if needed |
| **XML/MARC-XML** | ✅ Covered in pymarc | Different project |
| **CSV/TSV** | ⚠️ Partial (MARC field extraction) | Low priority; users export via other tools |
| **RDF/Linked Data** | ❌ Not evaluated | Future project if semantic MARC demanded |
| **YAML** | ❌ Not evaluated | Low priority; JSON preferred for MARC |

**Action:** Create separate MARC-XML and JSON evaluation documents if customer demand exists.

---

## Part 5: Implementation Plan

### 5.1 Phase Structure

**Phase 0 (Foundation):** Establish test framework, module structure, documentation patterns  
**Phase 1 (Core):** ISO 2709 validation, Protobuf implementation  
**Phase 2 (Value-Add):** Arrow, FlatBuffers, MessagePack  
**Phase 3 (Specialized):** CBOR, Avro, Arrow Analytics  
**Phase 4 (Polish):** Documentation, Python wrapper, tutorials  

---

### 5.2 Detailed Implementation Roadmap

#### **PHASE 0: Foundation (2-3 days)**

**Objective:** Set up project structure and validation framework before implementing first new format.

| Task | Effort | Dependencies | Deliverable |
|------|--------|--------------|------------|
| Create `formats/mod.rs` with feature flags | 2 hrs | — | Modular format architecture |
| Establish generic Reader/Writer traits | 4 hrs | — | `traits.rs` with standard API |
| Create format test fixtures from FIDELITY_TEST_SET | 2 hrs | FIDELITY_TEST_SET.md | Shared test data for all formats |
| Add Cargo.toml feature gates + docs | 2 hrs | — | Build-time format selection |
| Create FORMAT_IMPLEMENTATION_CHECKLIST (how to add a format) | 3 hrs | — | Template for format additions |
| Update main README with format matrix + installation guide | 2 hrs | — | User-facing documentation |

**Estimate:** ~15 hrs (1.5 days)  
**Dependencies:** None  
**Blocking:** Nothing; can start immediately

**Expected Issue Creation:** bd create "Phase 0: Foundation setup" --parent mrrc-fks.9

---

#### **PHASE 1: Core Formats (4-5 days)**

##### **1A: ISO 2709 Validation & Refactoring (2 days)**

**Objective:** Ensure ISO 2709 reader/writer conform to new module structure and generic traits.

| Task | Effort | Dependencies | Deliverable |
|------|--------|--------------|------------|
| Refactor existing ISO 2709 reader into `formats/iso2709/reader.rs` | 3 hrs | Phase 0 | Trait-compliant reader |
| Refactor existing ISO 2709 writer into `formats/iso2709/writer.rs` | 3 hrs | Phase 0 | Trait-compliant writer |
| Add round-trip tests (FIDELITY_TEST_SET) | 2 hrs | Phase 0 | 100+ test cases |
| Benchmark vs baseline (should be identical) | 1 hr | Phase 0 | Performance regression check |
| Document ISO 2709 format specifics (compression, streaming, edge cases) | 2 hrs | — | FORMATS_ISO2709.md |

**Estimate:** ~11 hrs (1.5 days)  
**Dependencies:** Phase 0  
**Blocking:** Protobuf work (needs traits established)

**Expected Issue:** bd create "Phase 1A: ISO 2709 refactoring" --parent mrrc-fks.9

---

##### **1B: Protobuf Implementation (2-3 days)**

**Objective:** Implement Protobuf reader/writer with full schema versioning support.

| Task | Effort | Dependencies | Deliverable |
|------|--------|--------------|------------|
| Define MARC.proto schema (field tags, indicators, subfields) | 4 hrs | Phase 0 | `src/formats/protobuf/schema.proto` |
| Generate Rust code via prost-build | 1 hr | Phase 0 | Auto-generated `marc.rs` |
| Implement ProtobufReader (trait-compliant) | 3 hrs | Phase 1A | `src/formats/protobuf/reader.rs` |
| Implement ProtobufWriter (trait-compliant) | 3 hrs | Phase 1A | `src/formats/protobuf/writer.rs` |
| Add round-trip tests (FIDELITY_TEST_SET) | 2 hrs | Phase 0 | 100+ test cases |
| Test schema versioning (add optional field, old reader still works) | 2 hrs | Phase 1B | Evolution verification |
| Benchmark (target: 100k rec/sec) | 1 hr | Phase 1B | Performance verification |
| Document schema design + evolution strategy | 2 hrs | — | FORMATS_PROTOBUF.md |

**Estimate:** ~18 hrs (2-3 days)  
**Dependencies:** Phase 0, Phase 1A  
**Blocking:** Python wrapper (Tier 1 Protobuf support)

**Expected Issue:** bd create "Phase 1B: Protobuf implementation" --parent mrrc-fks.9 --deps mrrc-fks.1

---

#### **PHASE 2: High-Value Formats (6-8 days)**

##### **2A: Arrow (Columnar + Analytics) (3 days)**

**Objective:** Implement Arrow reader/writer for both interchange and analytics use cases.

| Task | Effort | Dependencies | Deliverable |
|------|--------|--------------|------------|
| Design Arrow schema for row-oriented interchange (MarcRecord → RecordBatch) | 2 hrs | Phase 0 | Arrow schema design |
| Implement ArrowWriter (builds RecordBatch from MarcRecord) | 4 hrs | Phase 0 | `src/formats/arrow/writer.rs` |
| Implement ArrowReader (reads RecordBatch, reconstructs MarcRecord) | 4 hrs | Phase 0 | `src/formats/arrow/reader.rs` |
| Implement ArrowIpcWriter (serializes to Arrow IPC format for DuckDB/Polars) | 2 hrs | Phase 2A | `src/formats/arrow/ipc.rs` |
| Add round-trip tests (FIDELITY_TEST_SET) | 2 hrs | Phase 0 | 100+ test cases |
| Test IPC interop with DuckDB (read via Python) | 2 hrs | Phase 2A | Integration verification |
| Benchmark (target: 865k rec/sec) | 1 hr | Phase 2A | Performance verification |
| Implement Arrow Analytics builder (MARC → long format columnar) | 3 hrs | Phase 2A | `src/formats/arrow/columnar.rs` |
| Document Arrow interchange + analytics tier distinction | 2 hrs | — | FORMATS_ARROW.md |

**Estimate:** ~22 hrs (3 days)  
**Dependencies:** Phase 1A (MarcRecord trait)  
**Blocking:** Python wrapper Arrow support, Arrow Analytics tier

**Expected Issue:** bd create "Phase 2A: Arrow implementation" --parent mrrc-fks.9 --deps mrrc-fks.7

---

##### **2B: FlatBuffers (2 days)**

**Objective:** Implement FlatBuffers reader/writer for memory-efficient APIs.

| Task | Effort | Dependencies | Deliverable |
|------|--------|--------------|------------|
| Define MARC.fbs schema (field tags, indicators, subfields) | 2 hrs | Phase 0 | `src/formats/flatbuffers/schema.fbs` |
| Generate Rust code via flatc | 1 hr | Phase 0 | Auto-generated builders |
| Implement FlatBuffersWriter (trait-compliant) | 3 hrs | Phase 1A | `src/formats/flatbuffers/writer.rs` |
| Implement FlatBuffersReader (trait-compliant) | 3 hrs | Phase 1A | `src/formats/flatbuffers/reader.rs` |
| Add round-trip tests (FIDELITY_TEST_SET) | 2 hrs | Phase 0 | 100+ test cases |
| Memory profile verification (target: 64% memory savings) | 1 hr | Phase 2B | Memory regression check |
| Benchmark (target: 259k rec/sec) | 1 hr | Phase 2B | Performance verification |
| Document FlatBuffers schema + zero-copy design | 2 hrs | — | FORMATS_FLATBUFFERS.md |

**Estimate:** ~15 hrs (2 days)  
**Dependencies:** Phase 1A  
**Blocking:** Python wrapper FlatBuffers support

**Expected Issue:** bd create "Phase 2B: FlatBuffers implementation" --parent mrrc-fks.9 --deps mrrc-fks.2

---

##### **2C: MessagePack (2 days)**

**Objective:** Implement MessagePack reader/writer for compact, universal serialization.

| Task | Effort | Dependencies | Deliverable |
|------|--------|--------------|------------|
| Define MessagePack schema via serde impl on MarcRecord | 2 hrs | Phase 1A | Serde traits on MarcRecord |
| Implement MessagePackWriter (serialize MarcRecord) | 2 hrs | Phase 1A | `src/formats/messagepack/writer.rs` |
| Implement MessagePackReader (deserialize to MarcRecord) | 2 hrs | Phase 1A | `src/formats/messagepack/reader.rs` |
| Add round-trip tests (FIDELITY_TEST_SET) | 2 hrs | Phase 0 | 100+ test cases |
| Benchmark (target: 750k rec/sec) | 1 hr | Phase 2C | Performance verification |
| Compare with MessagePack-Schema and raw serde (document tradeoff) | 1 hr | Phase 2C | Design decision |
| Document MessagePack schema-less design + compatibility | 2 hrs | — | FORMATS_MESSAGEPACK.md |

**Estimate:** ~12 hrs (2 days)  
**Dependencies:** Phase 1A  
**Blocking:** Python wrapper MessagePack support

**Expected Issue:** bd create "Phase 2C: MessagePack implementation" --parent mrrc-fks.9 --deps mrrc-fks.5

---

#### **PHASE 3: Specialized Formats (On-Demand)**

To be implemented only if customer demand or explicit project requirement.

| Format | Effort | Condition |
|--------|--------|-----------|
| **CBOR** | 2 days | Archival requirement OR government/academic mandate |
| **Avro** | 2 days | Kafka/data lake integration required |
| **Arrow Analytics** | 1 day | Heavy MARC analytics workload (integrate existing POC) |

**Expected Issues:**
- bd create "Phase 3A: CBOR implementation" --parent mrrc-fks.9 --priority 3
- bd create "Phase 3B: Avro implementation" --parent mrrc-fks.9 --priority 3
- bd create "Phase 3C: Arrow Analytics integration" --parent mrrc-fks.9 --priority 3

---

#### **PHASE 4: Python Wrapper & Documentation (5-7 days)**

##### **4A: Python PyO3 Bindings (3 days)**

**Objective:** Expose all Tier 1 + Tier 2 formats to Python with type hints and convenient APIs.

| Task | Effort | Dependencies | Deliverable |
|------|--------|--------------|------------|
| Update PyO3 module to export Tier 1 formats (iso2709, protobuf) | 2 hrs | Phase 1 | Python access to core formats |
| Add Tier 2 format exports (arrow, flatbuffers, messagepack) | 2 hrs | Phase 2 | Python access to high-value formats |
| Implement format-agnostic `mrrc.read(path, format=...)` function | 2 hrs | Phase 1-2 | Convenient single API |
| Implement format-agnostic `mrrc.write(records, path, format=...)` function | 2 hrs | Phase 1-2 | Convenient single API |
| Add `.pyi` type hints for all formats | 2 hrs | Phase 1-2 | IDE autocompletion |
| Test round-trip fidelity via Python (FIDELITY_TEST_SET) | 2 hrs | Phase 4A | Cross-language verification |
| Benchmark Python wrapper overhead | 1 hr | Phase 4A | Performance profile |

**Estimate:** ~13 hrs (2 days)  
**Dependencies:** Phase 1 + Phase 2  
**Blocking:** User-facing Python documentation

**Expected Issue:** bd create "Phase 4A: Python PyO3 bindings" --parent mrrc-fks.9

---

##### **4B: Format Modules & Helpers (1 day)**

**Objective:** Create user-friendly format-specific modules and integration guides.

| Task | Effort | Dependencies | Deliverable |
|------|--------|--------------|------------|
| Create `mrrc/formats/` package with format-specific modules | 2 hrs | Phase 4A | Format submodule structure |
| Implement `mrrc.formats.arrow.export_to_parquet()` helper | 2 hrs | Phase 2A | Parquet export convenience |
| Implement `mrrc.analytics` module with Arrow analytics helpers | 2 hrs | Phase 2A | Analytics export convenience |
| Create integration examples (DuckDB queries, Polars dataframes) | 2 hrs | Phase 2A | User education |

**Estimate:** ~8 hrs (1 day)  
**Dependencies:** Phase 2A, Phase 4A  
**Blocking:** Documentation

**Expected Issue:** bd create "Phase 4B: Python format modules" --parent mrrc-fks.9

---

##### **4C: Comprehensive Documentation (2-3 days)**

**Objective:** Write user guides, tutorials, and architecture documentation.

| Task | Effort | Dependencies | Deliverable |
|------|--------|--------------|------------|
| Update main README with format support matrix (which versions?) | 2 hrs | Phase 4A-4B | User-facing overview |
| Write INSTALLATION_GUIDE.md (feature flags, optional deps) | 2 hrs | Phase 0 | Getting started |
| Write FORMAT_SELECTION_GUIDE.md (decision tree for users) | 3 hrs | All | Format choice guidance |
| Create PYTHON_TUTORIAL.md (read/write examples, format conversions) | 3 hrs | Phase 4A-4B | Python user education |
| Create RUST_TUTORIAL.md (trait-based APIs, custom implementations) | 2 hrs | Phase 4A | Rust user education |
| Write STREAMING_GUIDE.md (large file handling, memory optimization) | 2 hrs | Phase 1A | Performance guidance |
| Create MIGRATION_GUIDE.md (pymarc → mrrc format mapping) | 2 hrs | All | Legacy user support |
| Update API docs (rustdoc + Python docstrings) | 2 hrs | Phase 4A | Code-level documentation |
| Write ARCHITECTURE.md (format hierarchy, feature interactions) | 2 hrs | Phase 0 | Maintainer documentation |

**Estimate:** ~20 hrs (2-3 days)  
**Dependencies:** All phases  
**Blocking:** Release

**Expected Issue:** bd create "Phase 4C: Comprehensive documentation" --parent mrrc-fks.9

---

### 5.3 Implementation Sequence & Critical Path

```
PHASE 0: Foundation (1.5 days) [CRITICAL]
  └─ Traits, module structure, test fixtures
      │
      ├─→ PHASE 1A: ISO 2709 refactoring (1.5 days) [CRITICAL]
      │    │
      │    └─→ PHASE 1B: Protobuf (2-3 days) [CRITICAL]
      │         ├─→ PHASE 2A: Arrow (3 days) [HIGH VALUE]
      │         ├─→ PHASE 2B: FlatBuffers (2 days) [HIGH VALUE]
      │         └─→ PHASE 2C: MessagePack (2 days) [HIGH VALUE]
      │              │
      │              └─→ PHASE 4A: Python bindings (2 days)
      │                   │
      │                   └─→ PHASE 4B: Python modules (1 day)
      │                        │
      │                        └─→ PHASE 4C: Documentation (2-3 days) [BLOCKING]
      │
      └─→ PHASE 3: Specialized (On-demand)
           └─ CBOR, Avro, Arrow Analytics
```

**Critical Path Duration:** 15-18 days (Tier 1 + Tier 2 complete)

**Fast-Track Option (MVP):** Phase 0 + Phase 1A + Phase 1B + Phase 4C = 7-8 days (ISO 2709 + Protobuf only)

---

### 5.4 Parallel Tracks Opportunity

After Phase 1B, the following can run in parallel:
- Phase 2A, 2B, 2C can be independent (no inter-format dependencies)
- Documentation can start after Phase 1 is feature-complete
- Python wrapper work can start once Phase 2 is underway

**Recommended Parallel Structure:**
```
After Phase 1B:
- Developer 1: Phase 2A (Arrow)
- Developer 2: Phase 2B (FlatBuffers) 
- Developer 3: Phase 2C (MessagePack)
- Developer 4: Phase 4C (Documentation) [can start early]
```

---

## Part 6: Documentation Gaps & New Documents Needed

### 6.1 New Documents to Create

#### **User-Facing Documentation**

| Document | Purpose | Owner | Effort | Phase |
|----------|---------|-------|--------|-------|
| `INSTALLATION_GUIDE.md` | How to install mrrc with desired formats | Tech writer | 2 hrs | Phase 4C |
| `FORMAT_SELECTION_GUIDE.md` | Decision tree for choosing formats | Product | 3 hrs | Phase 4C |
| `PYTHON_TUTORIAL.md` | Examples: read/write/convert in Python | Tech writer | 3 hrs | Phase 4C |
| `RUST_TUTORIAL.md` | Examples: Reader/Writer trait usage | Tech writer | 2 hrs | Phase 4C |
| `STREAMING_GUIDE.md` | Large file handling & memory optimization | Architect | 2 hrs | Phase 4C |
| `MIGRATION_GUIDE.md` | pymarc → mrrc format compatibility | Tech writer | 2 hrs | Phase 4C |
| `FORMATS_*.md` (per format) | Format-specific design + tradeoffs | Implementer | 2 hrs each | Per phase |

#### **Developer/Maintainer Documentation**

| Document | Purpose | Owner | Effort | Phase |
|----------|---------|-------|--------|-------|
| `FORMAT_IMPLEMENTATION_CHECKLIST.md` | Template for adding new formats | Architect | 3 hrs | Phase 0 |
| `ARCHITECTURE.md` | Module organization + dependencies | Architect | 2 hrs | Phase 0 |
| `TRAIT_DESIGN.md` | Reader/Writer trait specification | Architect | 1.5 hrs | Phase 0 |

#### **Evaluation Follow-Ups**

| Document | Purpose | Owner | Effort | Phase |
|----------|---------|-------|--------|-------|
| Updated `COMPARISON_MATRIX.md` | Include implementation status + timeline | Architect | 2 hrs | After Phase 4C |
| `EVALUATION_BINCODE.md` (optional) | Rust-native serde POC | Researcher | 4 hrs | Post-release |
| `EVALUATION_JSON_LINES.md` (optional) | Human-readable streaming format | Researcher | 4 hrs | Post-release |

### 6.2 Documentation Update Checklist

**Existing documents to update:**

- [ ] `README.md` - Add format support matrix, installation, quick start
- [ ] `CONTRIBUTING.md` - Add "How to implement a format" section (link to checklist)
- [ ] `Cargo.toml` - Add feature flag documentation
- [ ] `src/lib.rs` - Add module-level docs explaining format tiers
- [ ] `src/formats/mod.rs` - Document Reader/Writer traits
- [ ] `pyproject.toml` - Add Python format availability docs

---

## Part 7: Final Format Support Decision Matrix

### 7.1 Decision Framework: Clear Keep/Defer/Exclude

```
TIER 1: MUST SHIP (Before Release)
├─ ISO 2709 (baseline; already exists; zero deps; 900k rec/sec)
└─ Protobuf (modern API; schema evolution; multi-language; gRPC)
   Effort: 4-5 days | Blocking: Release date

TIER 2: SHIP TOGETHER (High ROI; Distinct Personas)
├─ Arrow (3 days | Analytics + ecosystem standard)
├─ FlatBuffers (2 days | Mobile/embedded + zero-copy)
└─ MessagePack (2 days | Compact + universal)
   Effort: 7 days | Blocking: None (after Tier 1)

TIER 3: DEFER (Niche; Implement on Customer Demand)
├─ CBOR (2 days | Government/academic archival)
├─ Avro (2 days | Kafka data lake)
└─ Arrow Analytics (1 day | Discovery optimization)
   Effort: on-demand | Blocking: Nothing

EXCLUDE: Do NOT Implement
├─ Parquet (redundant; user→Arrow IPC→Parquet)
├─ JSON/YAML/XML (different project)
├─ Bincode (Rust-only; limited appeal)
└─ Ion (unclear value; Protobuf better)

RESEARCH (Post-Release, If Needed)
├─ JSON Lines (dev ergonomics; low effort)
├─ Bincode + Compression (performance investigation)
└─ Custom MARC Binary Schema (evolution research)
```

**Rationale:** Tier 1 + 2 (11 days total) provides complete solution for defined personas (librarians, API devs, data scientists, mobile devs). Tier 3 and research items are implementation options, not blockers. This avoids "format fatigue" and maintains quality.

### 7.2 Customer Personas & Format Recommendations

| Persona | Primary Formats | Secondary Formats | Why |
|---------|-----------------|-------------------|-----|
| **MARC Librarian (pymarc migrant)** | ISO 2709, Protobuf | Arrow (analytics) | Familiar with ISO 2709; gRPC for modern APIs |
| **REST API Developer** | Protobuf, FlatBuffers | MessagePack | gRPC/REST standard; memory-efficient |
| **Data Scientist** | Arrow, MessagePack | Avro | Analytics-first; Parquet export optional |
| **Embedded/Mobile Dev** | FlatBuffers, MessagePack | Arrow | Memory-constrained; zero-copy important |
| **Enterprise Data Lake** | Avro, Arrow | Protobuf | Kafka/Hadoop integration; schema registry |
| **Preservation/Archival** | ISO 2709, CBOR | Protobuf | Long-term standard; RFC 7949 compliance |
| **High-Frequency Batch** | ISO 2709, Arrow | MessagePack | Maximum throughput; minimal overhead |

---

## Part 8: Risk Mitigation & Known Issues

### 8.1 Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|-----------|
| Dependency bloat (too many formats) | Medium | Low | Feature flags; clear tier structure; minimum viable set |
| Breaking API changes across formats | Low | High | Stabilize Reader/Writer traits in Phase 0; test cross-version compat |
| Performance regression (adding formats) | Low | Medium | Benchmark each phase; CI performance gates |
| Python wrapper maintenance burden | Medium | Medium | Keep wrapper thin; delegate to PyO3 + serde where possible |
| Schema evolution issues (Protobuf/FlatBuffers) | Low | High | Design schemas carefully in Phase 1; add evolution tests |
| DuckDB/Polars interop breaks | Low | Medium | Test with pinned versions; monitor upstream changes |

### 8.2 Known Limitations

| Limitation | Workaround | Priority |
|------------|------------|----------|
| Parquet export not built-in (user responsibility) | Document in tutorials; provide Arrow IPC → Parquet example | Low |
| Arrow Analytics requires full materialization | OK for typical collections (<10M records); document trade-offs | Medium |
| MessagePack schema-less (no versioning) | Document and accept limitation; use Protobuf for schema evolution | Low |
| FlatBuffers append-only schema evolution | Document constraint; offer Protobuf alternative for complex evolution | Low |
| Python round-trip fidelity unverified (yet) | Phase 4A testing; create cross-language CI check | High |

---

## Part 9: Success Criteria & Completion Definition

### 9.1 Phase Completion Criteria

**Phase 0:**
- [ ] All feature flags defined and documented
- [ ] Reader/Writer traits established and tested
- [ ] Test fixtures available for all formats
- [ ] Module structure in place with placeholders

**Phase 1:**
- [ ] ISO 2709 refactored and benchmarked (≥900k rec/sec)
- [ ] Protobuf reader/writer complete (≥100k rec/sec)
- [ ] Schema versioning tested (forward + backward compat)
- [ ] Round-trip fidelity 100% (FIDELITY_TEST_SET)
- [ ] Documentation: FORMATS_ISO2709.md, FORMATS_PROTOBUF.md

**Phase 2:**
- [ ] Arrow, FlatBuffers, MessagePack all complete
- [ ] Performance targets met (Arrow ≥865k, FlatBuffers ≥259k, MessagePack ≥750k)
- [ ] Round-trip fidelity 100% for all formats
- [ ] IPC interop verified (Arrow ↔ DuckDB)
- [ ] Documentation: FORMATS_ARROW.md, FORMATS_FLATBUFFERS.md, FORMATS_MESSAGEPACK.md

**Phase 4 (Complete Release):**
- [ ] Python wrapper exports all Tier 1 + 2 formats
- [ ] Type hints (`.pyi`) complete
- [ ] Comprehensive tutorials (Python + Rust)
- [ ] Format selection guide + installation instructions
- [ ] Updated README with full matrix
- [ ] CI tests: Round-trip + cross-language + performance gates

### 9.2 Quality Gates Before Release

```
Release Blockers (must-have):
- [ ] All Tier 1 formats pass round-trip tests (100% fidelity)
- [ ] All Tier 1 formats meet performance targets (±10% variance)
- [ ] No regressions in existing ISO 2709 functionality
- [ ] Python wrapper tested with Tier 1 formats
- [ ] Installation docs complete (feature flags clear)
- [ ] MIGRATION_GUIDE.md for pymarc users written

Release Desirables (nice-to-have):
- [ ] All Tier 2 formats complete + tested
- [ ] Python wrapper for all Tier 2 formats
- [ ] Streaming guide documentation
- [ ] Benchmark report vs pymarc
```

---

## Part 10: Recommendations Summary (FINAL)

### 10.1 Library (Rust mrrc) - IMPLEMENTATION DIRECTIVE

**TIER 1: SHIP IN INITIAL RELEASE (Non-Negotiable)**
1. **ISO 2709** - Refactor to trait-based; maintain 900k rec/sec performance
2. **Protobuf** - Complete implementation with schema versioning tests

**TIER 2: SHIP IN INITIAL RELEASE (Collectively = Complete Ecosystem)**
3. **Arrow** - Row-oriented interchange + columnar analytics; integrate POC
4. **FlatBuffers** - Mobile/embedded streaming APIs; 64% memory savings
5. **MessagePack** - Compact serialization; 50+ language support; serde-friendly

**TIER 3: DEFER TO PHASE 4+ (Customer Demand Driven)**
- CBOR, Avro, Arrow Analytics (see Part 1.2 for decision criteria)

**EXPLICITLY EXCLUDE (Do Not Implement)**
- **Parquet** - Use Arrow IPC → external Parquet (user responsibility)
- **JSON/YAML/XML** - Different project scope
- **Bincode** - Rust-only; no cross-platform appeal
- **Ion** - Unclear MARC value; Protobuf superior

**Total Effort:** Tier 1 (4-5 days) + Tier 2 (7 days) = **11 days to complete release**  
**Rationale:** This provides end-to-end solution for all major personas. Tier 3 adds complexity without proportional benefit. Smaller, well-tested core > every possible format.

---

### 10.2 Python Wrapper - IMPLEMENTATION DIRECTIVE

**TIER 1: EXPOSE IN INITIAL RELEASE**
- `mrrc.read_iso2709(path)` → `list[MarcRecord]`
- `mrrc.write_iso2709(records, path)` → `None`
- `mrrc.read_protobuf(data)` → `list[MarcRecord]`
- `mrrc.write_protobuf(records)` → `bytes`
- Convenience: `mrrc.read(path, format="iso2709"|"protobuf")`

**TIER 2: EXPOSE IN INITIAL RELEASE (At Same Time as Rust)**
- `mrrc.read_arrow(data)` → `list[MarcRecord]`
- `mrrc.write_arrow(records)` → `bytes`
- `mrrc.read_flatbuffers(data)` → `list[MarcRecord]`
- `mrrc.write_flatbuffers(records)` → `bytes`
- `mrrc.read_messagepack(data)` → `list[MarcRecord]`
- `mrrc.write_messagepack(records)` → `bytes`
- Analytics helper: `mrrc.analytics.to_arrow(records)` → `pyarrow.Table`

**TIER 3: OPTIONAL (Customer Request Driven)**
- `mrrc.formats.cbor`, `mrrc.formats.avro`, `mrrc.formats.arrow_analytics`

**DO NOT EXPOSE**
- Parquet export (recommend user: `arrow_table.to_parquet("file.parquet")`)

---

### 10.3 Timeline & Resources (Tier 1 + 2 Only)

**RELEASE TIMELINE (Tier 1 + Tier 2 formats only):**

| Phase | Duration | Est. Effort | FTE Required | Blocking? |
|-------|----------|-------------|--------------|-----------|
| Phase 0 | 1.5 days | 15 hrs | 2 | Yes |
| Phase 1 (ISO 2709 + Protobuf) | 3-5 days | 29 hrs | 2 | Yes |
| Phase 2 (Arrow + FlatBuffers + MessagePack) | 6-8 days | 49 hrs | 3 (parallel) | No |
| Phase 4 (Python wrapper + docs) | 5-7 days | 41 hrs | 2 | Yes |
| **Release Total** | **15-18 days** | **134 hrs** | **2-3 avg** | — |

**Phasing Strategy:**
- ✅ **Critical Path:** Phases 0 → 1 → 4 (must complete before release)
- ✅ **Parallel Opportunity:** Phase 2 (Arrow/FlatBuffers/MessagePack independent) can run while Phase 4 docs start
- ✅ **Estimated Wall Time:** 15-18 days with parallelization after Phase 1B
- ⏸️ **Tier 3 Timeline:** Phase 3+ items deferred indefinitely; open as customer-demand issues

**MVP Option (Tier 1 Only):** 7-8 days (ISO 2709 + Protobuf only; ship Tier 2 in v1.1 if needed)

---

### 10.4 Next Immediate Steps

1. **Create Phase 0 issue:** `bd create "Phase 0: Foundation setup" -p 1 --parent mrrc-fks.9`
2. **Establish writer ownership:** Assign Phase leads (ISO 2709, Protobuf, Arrow, FlatBuffers, etc.)
3. **Finalize FORMAT_IMPLEMENTATION_CHECKLIST.md** in Phase 0
4. **Set up CI gates:** Performance benchmarking, round-trip tests
5. **Schedule kick-off:** Phase 0 starts immediately after mrrc-fks.9 approval

---

## Appendix: Format at a Glance

### Quick Reference: When to Use Each Format

| Use Case | Recommended Format | Why |
|----------|-------------------|-----|
| **Default/Legacy** | ISO 2709 | Proven standard, maximum performance |
| **Modern API** | Protobuf | Schema contracts, gRPC, cross-language |
| **Streaming, Low Memory** | FlatBuffers | 64% memory savings, zero-copy |
| **Analytics, In-Memory** | Arrow | Columnar format, DuckDB/Polars integration |
| **Compact Storage** | MessagePack | 25% smaller, 50+ language support |
| **Government Archival** | CBOR | RFC 7949 standard, diagnostic notation |
| **Kafka/Data Lakes** | Avro | Schema registry, self-describing |
| **Discovery Optimization** | Arrow Analytics | SQL queries, field frequency analysis |

---

## Part 8: Cleanup & Consolidation of Evaluation Artifacts

This section documents how evaluation code, tests, documentation, and schemas from the format research phase will be consolidated into the production implementation or archived.

### 8.1 Evaluation Artifacts Overview

The format research project produced the following evaluation artifacts:

| Artifact | Location | Type | Status |
|----------|----------|------|--------|
| **Evaluation Benchmarks** | `benches/eval_*.rs` (CBOR, Avro, MessagePack) | Code | Evaluation only |
| **Evaluation Tests** | `tests/flatbuffers_evaluation.rs` | Code | Evaluation only |
| **Protobuf Implementation** | `src/protobuf.rs` | Code | Production-ready |
| **Arrow Implementation** | `src/arrow_impl.rs` | Code | Needs refactoring |
| **FlatBuffers Implementation** | `src/flatbuffers_impl.rs` | Code | Placeholder (serde-based) |
| **Evaluation Documentation** | `docs/design/format-research/EVALUATION_*.md` (10 files) | Docs | Reference only |
| **Framework Documentation** | `docs/design/format-research/EVALUATION_FRAMEWORK.md` | Docs | Foundation |
| **Fidelity Test Specification** | `docs/design/format-research/FIDELITY_TEST_SET.md` | Docs | Reusable |
| **Baseline Comparison** | `docs/design/format-research/BASELINE_ISO2709.md` | Docs | Reference |
| **Comparison Matrix** | `docs/design/format-research/COMPARISON_MATRIX.md` | Docs | Reference |
| **Example/POC Code** | `examples/polars_arrow_eval.rs` | Code | Evaluation POC |

### 8.2 Consolidation Strategy by Artifact Type

#### **8.2.1 Code: Evaluation Benchmarks (benches/eval_*.rs)**

**What:** Standalone benchmark files for CBOR, Avro, MessagePack formats  
**Current Location:** `benches/eval_avro.rs`, `benches/eval_cbor.rs`, `benches/eval_messagepack.rs`  
**Action:** Migrate → Phase 2 per-format benchmarks

| Format | Action | Destination | Phase | Notes |
|--------|--------|-------------|-------|-------|
| **MessagePack** | Migrate | `benches/` → Phase 2C tests | Phase 2C | Core logic reusable; performance targets already defined |
| **CBOR** | Archive | `docs/design/format-research/archive/` | Phase 3 (deferred) | Keep for Tier 3 implementation reference |
| **Avro** | Archive | `docs/design/format-research/archive/` | Phase 3 (deferred) | Keep for Tier 3 implementation reference |

**Implementation:** After Phase 0 is complete, migrate MessagePack eval code structure into the per-format test suite. Archive CBOR/Avro evaluation benchmarks in a `/archive` subdirectory of the format-research docs for future Phase 3 implementation reference.

---

#### **8.2.2 Code: Evaluation Tests (tests/flatbuffers_evaluation.rs)**

**What:** Comprehensive FlatBuffers round-trip evaluation test  
**Current Location:** `tests/flatbuffers_evaluation.rs`  
**Action:** Migrate → Phase 2B FlatBuffers test suite

The evaluation test includes:
- Fidelity comparison logic (comparing original vs. round-tripped records)
- Performance metric collection
- Batch serialization/deserialization logic

All of this is reusable in Phase 2B. The test infrastructure pattern (fidelity comparison functions, metrics collection) should be extracted and generalized in Phase 0 as part of the test infrastructure.

**Implementation:** Extract comparison logic into `tests/common/fidelity.rs` utility module in Phase 0. Phase 2B reuses this for FlatBuffers-specific round-trip tests.

---

#### **8.2.3 Code: Production Implementations**

##### **Protobuf (src/protobuf.rs)** ✅ Production-Ready
- **Status:** Already complete and tested
- **Action:** No changes; part of Phase 1B deliverable
- **Note:** Uses generated code from `build.rs` via prost

##### **Arrow (src/arrow_impl.rs)** ⚠️ Needs Refactoring
- **Status:** Evaluation implementation; placeholder for production
- **Current Design:** Single-file, evaluation-focused
- **Action:** 
  - Phase 2A: Refactor into modular structure (`src/formats/arrow/` directory)
  - Split into: `columnar.rs` (arrow array builders), `ipc.rs` (Arrow IPC format), `reader.rs`, `writer.rs`
  - Remove evaluation-only code (diagnostic output, etc.)
  - Implement Reader/Writer trait pattern consistently with other formats

##### **FlatBuffers (src/flatbuffers_impl.rs)** ❌ Placeholder, Will Replace
- **Status:** Evaluation implementation using serde_json as intermediary
- **Current Design:** Simplified for evaluation; NOT production-ready
- **Action:**
  - Phase 0: Delete `src/flatbuffers_impl.rs` (placeholder)
  - Phase 2B: Replace with proper FlatBuffers schema-based implementation via flatc code generation
  - New implementation: `src/formats/flatbuffers/` (schema.fbs → reader.rs, writer.rs)

---

#### **8.2.4 Code: Examples & POCs (examples/)**

**Polars/Arrow Evaluation (examples/polars_arrow_eval.rs)**
- **Status:** Evaluation POC demonstrating Arrow integration with Polars/DuckDB
- **Action:** Migrate → Phase 4B integration examples
- **Reuse:** Core logic shows Arrow → Polars dataframe conversion; adapt as tutorial example in `examples/arrow_to_polars.rs`
- **Archive:** Keep original in `docs/design/format-research/archive/` for reference

---

#### **8.2.5 Data: Test Fixtures**

**FIDELITY_TEST_SET (tests/data/fixtures/)**
- **Status:** 100-record test set used in evaluation
- **Action:** Reuse in Phase 0 and all subsequent phases
- **Location:** Keep in `tests/data/fixtures/fidelity_test_100.mrc`
- **Note:** Reference spec in `docs/design/format-research/FIDELITY_TEST_SET.md` (keep as living doc)

**Performance Test Sets (tests/data/fixtures/10k_records.mrc, etc.)**
- **Status:** Used in evaluation benchmarks
- **Action:** Reuse in Phase 2 benchmarks
- **Cleanup:** Consolidate naming and location after Phase 0

---

#### **8.2.6 Documentation: Evaluation Reports (EVALUATION_*.md)**

**Scope:** 10 evaluation documents (Arrow, Avro, CBOR, FlatBuffers, MessagePack, Parquet, Polars/Arrow/DuckDB, Protobuf, plus Framework and Baseline)

| Document | Purpose | Action | Archive Location |
|----------|---------|--------|------------------|
| `EVALUATION_FRAMEWORK.md` | Methodology template | **Keep** (living reference) | `docs/design/format-research/` |
| `EVALUATION_PROTOBUF.md` | Protobuf evaluation | Archive | `archive/evaluation-reports/` |
| `EVALUATION_ARROW.md` | Arrow evaluation | Archive | `archive/evaluation-reports/` |
| `EVALUATION_FLATBUFFERS.md` | FlatBuffers evaluation | Archive | `archive/evaluation-reports/` |
| `EVALUATION_MESSAGEPACK.md` | MessagePack evaluation | Archive | `archive/evaluation-reports/` |
| `EVALUATION_CBOR.md` | CBOR evaluation | Archive | `archive/evaluation-reports/` |
| `EVALUATION_AVRO.md` | Avro evaluation | Archive | `archive/evaluation-reports/` |
| `EVALUATION_PARQUET.md` | Parquet evaluation (excluded) | Archive | `archive/evaluation-reports/` |
| `EVALUATION_POLARS_ARROW_DUCKDB.md` | Ecosystem evaluation | Archive | `archive/evaluation-reports/` |
| `BASELINE_ISO2709.md` | ISO 2709 baseline | Archive | `archive/evaluation-reports/` |
| `COMPARISON_MATRIX.md` | Format comparison | **Keep & Update** (post-Phase 4) | `docs/design/format-research/` |
| `FIDELITY_TEST_SET.md` | Test spec | **Keep** (living doc) | `docs/design/format-research/` |

**Implementation:** 
- Move evaluation reports to `docs/design/format-research/archive/evaluation-reports/` in Phase 4C
- Update `COMPARISON_MATRIX.md` after Phase 4C with implementation status, timelines, and production results
- Keep framework docs and test specifications as reference material

---

### 8.3 Cleanup Tasks in Implementation Plan

The following cleanup tasks are integrated into the mrrc-d4g epic:

| Task ID | Phase | Description | Blocking |
|---------|-------|-------------|----------|
| `mrrc-d4g.1.5` | Phase 0 | Consolidate evaluation code into test infrastructure | Phase 2A-2C |
| `mrrc-d4g.1.6` | Phase 0 | Remove evaluation-only implementations (flatbuffers_impl, arrow_impl placeholders) | Phase 2A, 2B |
| `mrrc-d4g.3.4` | Phase 2 | Migrate evaluation benchmarks to per-format test suites | None |
| `mrrc-d4g.4.3.10` | Phase 4C | Remove/archive evaluation documentation (EVALUATION_*.md, etc.) | None |

---

### 8.4 Detailed Cleanup Workflow

#### **Phase 0: Foundation (setup for consolidation)**

1. **Extract test utilities:**
   - Create `tests/common/fidelity.rs` with record comparison helpers
   - Create `tests/common/metrics.rs` with performance collection
   - Both extracted from `tests/flatbuffers_evaluation.rs` for reuse

2. **Organize evaluation code:**
   - Note locations of `benches/eval_*.rs` for later migration
   - Identify reusable patterns in `src/arrow_impl.rs` and `src/flatbuffers_impl.rs`

#### **Phase 1B (after Protobuf complete)**

1. **Verify Protobuf implementation:**
   - Ensure `src/protobuf.rs` passes FIDELITY_TEST_SET
   - Document integration points

#### **Phase 2A (Arrow implementation)**

1. **Refactor Arrow implementation:**
   - Delete `src/arrow_impl.rs` (evaluation version)
   - Create `src/formats/arrow/` module structure
   - Implement proper Reader/Writer traits
   - Port relevant logic from evaluation code

2. **Create Arrow benchmarks:**
   - Start with evaluation benchmark structure from `benches/eval_messagepack.rs` pattern
   - Adapt for Arrow-specific metrics

#### **Phase 2B (FlatBuffers implementation)**

1. **Delete placeholder implementation:**
   - Remove `src/flatbuffers_impl.rs`

2. **Implement FlatBuffers from scratch:**
   - Use flatc code generation
   - Define `src/formats/flatbuffers/schema.fbs`
   - Migrate evaluation test logic into Phase 2B test suite

3. **Migrate evaluation tests:**
   - Extract fidelity comparison logic from `tests/flatbuffers_evaluation.rs`
   - Move to Phase 2B test suite
   - Archive evaluation-only test file

#### **Phase 2C (MessagePack implementation)**

1. **Migrate MessagePack benchmark:**
   - Port `benches/eval_messagepack.rs` core logic
   - Adapt to production implementation
   - Archive evaluation benchmark

2. **Archive CBOR/Avro evaluation benchmarks:**
   - Move `benches/eval_cbor.rs` → `docs/design/format-research/archive/evaluation-code/`
   - Move `benches/eval_avro.rs` → `docs/design/format-research/archive/evaluation-code/`

#### **Phase 4B (Python modules & examples)**

1. **Migrate Polars example:**
   - Refactor `examples/polars_arrow_eval.rs` → `examples/arrow_to_polars.rs`
   - Create tutorial-style example for Phase 4B documentation
   - Archive evaluation POC

#### **Phase 4C (Documentation & final cleanup)**

1. **Update documentation before archival:**
   - Update `COMPARISON_MATRIX.md` with final implementation status, performance results, and actual timelines
   - Verify `FIDELITY_TEST_SET.md` and `EVALUATION_FRAMEWORK.md` are current
   - Ensure `FORMAT_SUPPORT_STRATEGY.md` Part 8 accurately reflects all cleanup actions

2. **Archive entire format-research directory:**
   - Move `docs/design/format-research/` → `docs/history/format-research/`
   - This keeps the completed evaluation project separate from active implementation work
   - Create `docs/history/format-research/README.md` as archive index (see Section 8.5 template)
   - Preserves all evaluation reports, code, tests, and decision documentation

3. **Transition notification:**
   - Add note to `docs/design/` README (or create) explaining format-research project moved to history
   - Point implementation references to `docs/history/format-research/` if needed for Tier 3
   - Cross-link to mrrc-d4g epic for ongoing implementation status

4. **Verify post-archival:**
   - Confirm `docs/history/format-research/` contains all original artifacts
   - Verify `docs/design/format-research/` no longer exists in active docs
   - Update any internal links in other docs that referenced format-research path

---

### 8.5 Archival Strategy

**Why Archive Instead of Delete:**
- Preserves decision history and evaluation methodology for future reference
- Enables post-release analysis (e.g., "Did Arrow perform as expected?")
- Supports Tier 3 format implementation (CBOR/Avro can reference evaluation results)
- Documents what was evaluated and why certain formats were excluded
- Keeps active implementation docs separate from historical evaluation artifacts

**Archive Location & Strategy:** 

After Phase 4C completes, move the entire `docs/design/format-research/` directory to `docs/history/format-research/`. This separates the completed evaluation project from the ongoing implementation work.

```
BEFORE (during implementation):
docs/
├── design/
│   ├── format-research/          ← Active during Phase 0-4
│   │   ├── FORMAT_SUPPORT_STRATEGY.md
│   │   ├── EVALUATION_FRAMEWORK.md
│   │   └── ...
│   └── other/

AFTER (post-Phase 4C):
docs/
├── design/
│   ├── other/                    ← Non-format-research docs
├── history/
│   ├── format-research/          ← Historical evaluation project
│   │   ├── FORMAT_SUPPORT_STRATEGY.md
│   │   ├── EVALUATION_FRAMEWORK.md
│   │   ├── EVALUATION_*.md
│   │   ├── eval_*.rs
│   │   └── README.md (archive index)
```

**Archive Index (README in docs/history/format-research/):**
```markdown
# Format Evaluation Archive

Historical artifacts from the binary format evaluation project
(mrrc format research, Phase 0-4, completed January 2026).

## About This Project

This directory contains all evaluation reports, benchmarks, POCs, and decision documentation for the mrrc binary format evaluation project. The evaluation concluded with the Format Support Strategy, which recommended implementing Tier 1 (ISO 2709, Protobuf) and Tier 2 (Arrow, FlatBuffers, MessagePack) formats.

Production implementation is tracked in epic `mrrc-d4g` and not documented here.

## Contents

### Decision Documents
- `FORMAT_SUPPORT_STRATEGY.md` — Final strategy document with implementation plan
- `COMPARISON_MATRIX.md` — Detailed format comparison (updated with implementation results)

### Evaluation Reports
- `EVALUATION_FRAMEWORK.md` — Methodology and evaluation criteria
- `EVALUATION_PROTOBUF.md`, `EVALUATION_ARROW.md`, etc. — Per-format detailed evaluations
- `BASELINE_ISO2709.md` — ISO 2709 baseline performance metrics

### Test Specifications
- `FIDELITY_TEST_SET.md` — Test set specification (100 records, edge cases, coverage)

### Evaluation Code
- `benches/eval_*.rs` — Standalone benchmarks (CBOR, Avro, MessagePack)
- `tests/flatbuffers_evaluation.rs` — FlatBuffers evaluation test
- `examples/polars_arrow_eval.rs` — Arrow/Polars integration POC

## Using This Archive

### For Tier 3 Implementation (CBOR, Avro, Arrow Analytics)

When implementing deferred Tier 3 formats, reference the original evaluation:

1. Read `EVALUATION_[FORMAT].md` for detailed analysis
2. Review performance and fidelity targets in the report
3. Check `FIDELITY_TEST_SET.md` for test specification
4. Follow `EVALUATION_FRAMEWORK.md` methodology for verification

### For Future Format Evaluation

To evaluate new formats (e.g., JSON Lines post-release):

1. Use `EVALUATION_FRAMEWORK.md` as a template
2. Follow the same fidelity/performance testing methodology
3. Create a new EVALUATION_[FORMAT].md report
4. Update `COMPARISON_MATRIX.md` with results

### For Production Implementation Reference

The production implementation of Tier 1 and Tier 2 formats is NOT documented here.
For implementation status and details, see:

- Implementation planning: `/docs/design/format-research/FORMAT_SUPPORT_STRATEGY.md` (Part 5)
- Issue tracking: Epic `mrrc-d4g` in beads (`bd ready`)
- Source code: `src/formats/` (production modules)

## Project Timeline

| Phase | Dates | Status |
|-------|-------|--------|
| Evaluation Project | Jan 2026 | Complete (see evaluation reports) |
| Implementation (mrrc-d4g) | Jan-Feb 2026 | In Progress (tracked separately) |

---

*This archive was created January 2026 after completion of the binary format evaluation project.*
```

---

### 8.6 Migration Checklist (by Phase)

Use this checklist when executing cleanup tasks:

- **Phase 0:**
  - [ ] Create `tests/common/fidelity.rs` with record comparison helpers
  - [ ] Create `tests/common/metrics.rs` with performance utilities
  - [ ] Document evaluation code locations and reuse strategy

- **Phase 2A:**
  - [ ] Delete `src/arrow_impl.rs`
  - [ ] Create `src/formats/arrow/` module structure
  - [ ] Create Arrow benchmarks (reuse eval_messagepack pattern)

- **Phase 2B:**
  - [ ] Delete `src/flatbuffers_impl.rs`
  - [ ] Create `src/formats/flatbuffers/schema.fbs` and code generation
  - [ ] Migrate `tests/flatbuffers_evaluation.rs` logic into Phase 2B tests
  - [ ] Archive `tests/flatbuffers_evaluation.rs` original

- **Phase 2C:**
  - [ ] Migrate `benches/eval_messagepack.rs` core logic
  - [ ] Archive `benches/eval_cbor.rs` → archive/evaluation-code/
  - [ ] Archive `benches/eval_avro.rs` → archive/evaluation-code/

- **Phase 4B:**
  - [ ] Refactor `examples/polars_arrow_eval.rs` → `examples/arrow_to_polars.rs`
  - [ ] Archive evaluation POC

- **Phase 4C:**
  - [ ] Update COMPARISON_MATRIX.md with final implementation results
  - [ ] Verify FIDELITY_TEST_SET.md and EVALUATION_FRAMEWORK.md are current
  - [ ] Move entire `docs/design/format-research/` → `docs/history/format-research/`
  - [ ] Create `docs/history/format-research/README.md` as archive index
  - [ ] Add transition note to `docs/design/` README
  - [ ] Verify no broken links to old format-research path

---

**Document Complete**  
For implementation planning questions, refer to Part 5 (Implementation Plan).  
For format selection questions, refer to Part 10 (Recommendations Summary).
