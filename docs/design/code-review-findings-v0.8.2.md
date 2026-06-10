# Code Review Findings — v0.8.2

Comprehensive fresh-eyes review of the whole repository, conducted 2026-06-09 immediately after the
v0.8.2 release. Eight specialized review passes (code quality, architecture, security, performance,
test coverage, documentation, language/packaging best practices, CI/CD), each cross-checked against
the existing beads and GitHub issues backlog. Detailed per-phase working notes live in
`tmp/full-review/` (gitignored scratch); this document is the durable, complete record.

## Executive summary

The codebase is in good health for a pre-1.0 library, and several subsystems are genuinely
excellent: the ISO 2709 read skeleton (shared `Iso2709Builder` trait, defensively parsed, fuzzed),
the error-handling system (17-variant `MarcError` with stable doc anchors and a three-layer
drift-prevention harness), the property-test suite, and the CI topology. No Critical-severity and
no exploitable security findings exist; the worst input-driven failure class is denial of service
(bounded allocation churn), not memory unsafety.

The recurring weakness is **incomplete generalization applied asymmetrically**: abstractions were
introduced and then only partly wired in. The readers share a parse skeleton; the three writers are
copy-pasted with drifted safety posture. The `formats::` trait surface exists but covers one of
eight formats. The bibliographic record type gets property tests, fuzzing, and full reader options;
authority and holdings get a fraction of each at every layer. The Python wrapper papers over a Rust
ownership mismatch with a generation-counter handle protocol that costs a deep clone per attribute
access.

Three findings deserve immediate attention regardless of how the rest is scheduled:

1. **The declared MSRV (Rust 1.71) is false** — locked dependencies require 1.87, no CI job tests
   any MSRV, and cargo's MSRV-aware resolver will steer downstream users toward ancient dependency
   versions mrrc has never been tested against (PKG-1).
2. **Four user-facing documentation pages are wrong in ways that break copy-pasted code**,
   starting with the README's primary Python example, which raises `TypeError` (DOC-1..DOC-4).
3. **The default Python read path copies each record's bytes roughly nine times through an
   unbuffered file handle**, and the batching layer's documented GIL-amortization claims do not
   match the implementation — material for the library's headline performance story and a
   prerequisite for publishing credible pymarc comparisons (PERF-1..PERF-6).

Finding counts after merging duplicates across review passes: **0 critical, 14 high, 27 medium,
~25 low**, organized below into seven themes. Roughly a third of the findings extend or correct
existing backlog items; the rest are new.

## How to use this document

Each finding has a stable ID (`THEME-N`), a severity, an effort estimate (S = hours, M = a focused
PR, L = multi-PR), file references, a recommended fix, and a **Backlog** line using this
vocabulary:

- `new` — no existing bead or issue covers it; candidate for new ticketed work.
- `extends <id>` — an existing bead/issue covers part of it; the finding adds scope or specifics.
- `corrects <id>` — the finding invalidates or revises the premise of an existing item.
- `sequenced-with <id>` — independent work, but ordering matters; the relationship is stated.
- `covered-by <id>` — fully covered by existing backlog; listed only for completeness, do not
  re-ticket.

Agents extracting tickets: one finding generally maps to one ticket; findings marked as natural
pairs say so explicitly. The "Proposed epics" section suggests groupings; the "Backlog
reconciliation" section lists every existing bead/issue this review touched and what should happen
to it. Severity reflects user impact; effort and severity together drive priority, and the "Quick
wins" list at the end is the suggested first batch.

---

## Theme 1: Architecture and the 1.0 API surface

The 1.0-shaping work. A sketch of the coherent end state appears at the end of this theme.

### ARCH-1 — The `formats::` abstraction is aspirational; seven of eight formats bypass it
- **Severity:** High · **Effort:** L · **Area:** rust-core
- **Where:** `src/formats/mod.rs`; flat format modules `csv.rs`, `marcxml.rs`, `mods.rs`,
  `dublin_core.rs`, `json.rs`, `marcjson.rs`, `bibframe/`
- **What:** `FormatReader`/`FormatWriter`/`RecordIterator` traits and a `Format` enum exist, but
  the enum has exactly one variant (`Iso2709`) and only ISO 2709 is wired through it. Every other
  format is free functions with inconsistent naming and direction. Python `mrrc.read()`/`write()`
  accept only `marc`. There is no single answer to "how do I convert format X to/from a Record."
- **Fix:** Pick one model and apply it uniformly. Pragmatic choice: a standardized
  `record_to_<fmt>` / `<fmt>_to_record` free-function convention for every format, routed through a
  completed `Format` enum for the generic entry points; delete the unused trait surface if it isn't
  the chosen model. Do not ship 1.0 with both half-built.
- **Backlog:** new. Sequenced-with ARCH-2; PERF-10d (streaming variants) folds into this work.

### ARCH-2 — Format round-trip capability is undeclared; asymmetry is structural
- **Severity:** High · **Effort:** M · **Area:** rust-core, docs
- **What:** JSON/MARCJSON/MARCXML/MODS decode and encode; Dublin Core and CSV are encode-only.
  The docs now declare DC/CSV write-only (fixed in 0.8.2), but there is no single declared
  round-trip contract covering all eight formats, and the absence of a function is still the only
  API-level signal.
- **Fix:** A capability table in `formats/` rustdoc and the format docs as the normative contract;
  implement or explicitly decline each missing direction.
- **Backlog:** extends #92 (DC/CSV read path — reframe as "declare the contract for all eight");
  extends bd-c5sh (MARCJSON str-parity is one instance).

### ARCH-3 — Three copy-pasted ISO 2709 writers with drifted safety posture
- **Severity:** High · **Effort:** M · **Area:** rust-core
- **Where:** `src/writer.rs:137-259`, `src/authority_writer.rs:41-148`,
  `src/holdings_writer.rs:121-133`
- **What:** All three independently reimplement directory/data-area serialization. The
  bibliographic writer uses checked `u32::try_from` with typed `WriterError` and per-record error
  context; authority/holdings use unchecked `as u32` casts under a clippy allow and pass `None`
  for record index. (Security pass verified the casts are currently unreachable-as-truncating
  because `check_iso2709_size` runs first — a quality hazard, not a live bug.) The readers already
  solved this exact problem with the shared `Iso2709Builder` skeleton.
- **Fix:** One generic `write_iso2709_record<R: MarcRecord>` (or writer trait mirroring the reader
  skeleton); all three writers delegate. Deletes ~200 lines and the asymmetry. Include the
  `format!`-per-directory-entry allocation fix (PERF-10a) while there.
- **Backlog:** new. Prerequisite for bd-cdey (MARC-8 write — wire it once, not three times).
  De-risked by TEST-2 (roundtrip properties) — land TEST-2 first or together.

### ARCH-4 — Module layout inconsistency: 44 flat files beside two subdirectories
- **Severity:** Medium · **Effort:** M · **Area:** rust-core
- **What:** `bibframe/` and `formats/` are subdirs; six other format modules, three record types
  with readers/writers, and the query cluster all sit flat, with no rule for the boundary.
- **Fix:** Group into `formats/`, `records/`, `io/`, `query/` — re-export-only churn, one PR per
  group. Natural companion to ARCH-1 and ARCH-5.
- **Backlog:** new.

### ARCH-5 — Query/helper trait sprawl: eight overlapping modules
- **Severity:** Medium · **Effort:** M · **Area:** rust-core
- **Where:** `field_query`, `field_query_helpers`, `format_queries`, `authority_queries`,
  `field_collection`, `record_helpers`, `bibliographic_helpers`, `field_linkage`
- **What:** A user must discover and import up to eight similarly-named traits to query records;
  #225 (query discoverability) is the downstream symptom. `field_collection.rs` is exported,
  referenced nowhere, and untested — delete rather than document.
- **Fix:** Consolidate under `query/`: the `FieldQuery` DSL, one `RecordHelpers`, per-record-type
  query traits. Target: 2-3 imports.
- **Backlog:** extends #225 (this is the structural fix; #225's doc work remains the symptom
  treatment). Dead-module deletion is new.

### ARCH-6 — The Python live-handle protocol papers over a Rust ownership mismatch, expensively
- **Severity:** Medium (architecture) / High (performance) · **Effort:** L · **Area:**
  rust-core, bindings, python
- **Where:** `mrrc/__init__.py` (`_generation`/`_refresh`/`_writeback`/`StaleFieldError`);
  `src-python/src/wrappers.rs:845-851`
- **What:** Rust stores fields by value, so Python field handles resync via a generation counter —
  and `_refresh` deep-clones the field (tag + all subfield Strings) on **every** delegated
  attribute access, `__getitem__`, `get`, `__contains__`, and `value()`. The canonical pymarc loop
  (`for f in record.get_fields(): f['a']`) clones each field at least twice and crosses the PyO3
  boundary 3-4 times per field. `add_ordered_field`/`add_grouped_field` are O(n²) deep clones when
  building records.
- **Fix:** Evaluate solving ownership on the Rust side (live field proxies indexing back into the
  record, or shared interior mutability at the binding boundary) before investing further in the
  Python-side protocol. Cheap interim mitigations: skip `_refresh` when the cached generation
  matches; Rust-side `subfield_first(code)`; Rust-side positional insert.
- **Backlog:** new (the design question). Sequenced-with bd-mcei (`__init__.py` split — isolate
  the handle machinery regardless of outcome). PAR-4 (edge-case test matrix) should land first to
  pin current semantics.

### ARCH-7 — Reader configuration parity: builder options exist only on the bibliographic reader
- **Severity:** Medium · **Effort:** M · **Area:** rust-core, bindings, docs
- **What:** `with_recovery_mode`/`with_validation_level`/`with_source`/`with_max_errors`/
  `from_path` exist on `MarcReader` only; authority/holdings readers expose a narrower surface
  although the shared skeleton already threads the plumbing. The Python layer compounds this:
  `docs/reference/error-handling.md:316-333` documents `MARCReader.from_path()`/`.with_source()`
  in Python, **which do not exist** — both examples raise `AttributeError` (verified).
- **Fix:** Lift the builder options to all three Rust readers; expose source plumbing through the
  bindings (all three Python reader classes); fix the doc section to describe what actually works
  in the interim (that interim doc fix is DOC-3 and should not wait).
- **Backlog:** new. Related to bd-fe3l (ParseContext threading).

### ARCH-8 — `lib.rs` re-exports ~30 types flat with no tiering
- **Severity:** Low · **Effort:** S · **Area:** rust-core
- **Fix:** A `prelude` module for the common path (`Record`, `Field`, `Subfield`, `Leader`,
  `MarcReader`, `MarcWriter`, core helpers); advanced types reachable but not front-and-center.
  Do alongside ARCH-4.
- **Backlog:** new.

### ARCH-9 — Internal parse skeleton is `pub` (`#[doc(hidden)]`) and thus SemVer surface
- **Severity:** Low · **Effort:** S · **Area:** rust-core
- **What:** `iso2709`/`iso2709_skeleton` are hidden from docs but public; external implementors of
  `Iso2709Builder` are not an intended use case.
- **Fix:** `pub(crate)` at the 1.0 boundary.
- **Backlog:** new.

**The 1.0 end state these add up to:** one front door per direction (`Record` as hub; a complete
`Format` story; one generic writer over `MarcRecord`); records, formats, io, and query as visible
module groups; all three record types with equal reader options and test rigor; a focused 2-3-trait
query surface; a tiered crate root with a prelude; the Python wrapper as a thin pymarc-shaped skin
whose field handles are backed by Rust-side ownership rather than a resync protocol. The
architecture is already ~80% of this — the work is finishing started abstractions, not redesign.

---

## Theme 2: Performance

The inner parse loop is genuinely well-tuned (byte-wise scan, `SmallVec`, `OnceLock` validator,
monomorphized skeleton — verified clean). The cost is in the layers around it. Items 1-5 are the
priority order for limited time. **Publishing pymarc-comparison numbers (bd-dra6) should wait until
PERF-1..PERF-3 and PKG-9 land**, since they change the baseline.

### PERF-1 — `RustFile` backend is unbuffered: ≥2 syscalls per record
- **Severity:** High · **Effort:** S · **Area:** bindings
- **Where:** `src-python/src/backend.rs:36`, `:199-241`
- **Fix:** `RustFile(BufReader<File>)`, 64-256 KB capacity. Highest win-to-effort ratio found in
  this review. Check the core Rust `MarcReader::from_path` for the same issue.
- **Backlog:** new (concretizes bd-8zv5).

### PERF-2 — `set_parse_buffer` full-record memcpy on the happy path of every reader
- **Severity:** High · **Effort:** S-M · **Area:** rust-core
- **Where:** `src/iso2709_skeleton.rs:285-286` → `src/iso2709.rs:166-169`
- **What:** Every record is cloned solely so error constructors can produce `bytes_near` hex dumps
  for the <1% that error. Paid by all paths including pure Rust and the published Rust benchmarks.
- **Fix:** Pass the buffer to the `err_*` helpers at raise sites (`&record_data` is in scope at
  every raise) — mechanical, deletes the copy.
- **Backlog:** new (concretizes bd-8zv5).

### PERF-3 — Default Python read path copies each record's bytes ~9 times
- **Severity:** High · **Effort:** M · **Area:** bindings, rust-core
- **Where:** chain through `backend.rs:236-276`, `batched_unified_reader.rs:103/131/173`,
  `readers.rs:330/334/348`, `iso2709.rs:407`, `iso2709_skeleton.rs:286`, `mrrc/__init__.py:1727`
- **What:** Nine copies between `MARCReader(path)` and a parsed record vs a theoretical one.
  `readers.rs:348` `Cursor::new(record_bytes_owned.to_vec())` is pure waste (already owned).
- **Fix:** Move owned `Vec<u8>` through the chain; add a slice-based parse entry point
  (`parse_record_from_slice`) so in-memory bytes skip the Cursor/`read_record_data` copies — this
  also makes the rayon path zero-copy (PERF-7).
- **Backlog:** new (concretizes bd-8zv5). Partially subsumed by PERF-6 if batching is redesigned.

### PERF-4 — pymarc `current_chunk` compatibility costs two extra full-record copies, always
- **Severity:** High · **Effort:** S · **Area:** bindings, python
- **Where:** `readers.rs:334/215`; `mrrc/__init__.py:1727`
- **Fix:** Make `MARCReader.current_chunk` a lazy `@property` fetching `_inner.last_chunk`;
  nobody pays unless they read it.
- **Backlog:** new.

### PERF-5 — Python-file backend: two `file.read()` calls + `getattr("read")` per record, under the GIL
- **Severity:** High · **Effort:** M · **Area:** bindings
- **Where:** `buffered_reader.rs:31,105,152-165`; same shape in `unified_reader.rs`, `backend.rs`
- **Fix:** Chunked reads (256 KB ≈ one Python call per ~150 records) split in Rust (the boundary
  scanner already supports this); bind the `read` method once; `downcast::<PyBytes>` instead of
  `extract::<Vec<u8>>`.
- **Backlog:** new.

### PERF-6 — Batching amortizes nothing; GIL detach is per record; duplicated state machines
- **Severity:** Medium (perf) + High (maintenance) · **Effort:** M-L · **Area:** bindings
- **Where:** `readers.rs:284,288-379`; `batched_reader.rs` and `batched_unified_reader.rs`
  (near-identical, already drifted — only one tracks offsets); queue elements are
  `SmallVec<[u8; 4096]>` (4 KB memcpy per queue op, ~820 KB backing)
- **What:** Comments claim "N to N/100 GIL reduction"; the implementation detaches per record and
  saves zero Python calls. The two batched readers are a line-for-line duplicated state machine.
- **Fix:** One redesign: a generic `BatchedReader<S: RecordByteSource>`, parse the whole batch
  inside one `detach`, queue parsed records (or plain `Vec<u8>`), keep the 200-record/300 KB
  bounds, fix the comments and the matching test docstrings (TEST-7b).
- **Backlog:** new.

### PERF-7 — Parallel-path defects: GIL held during rayon parse; per-record channel sends
- **Severity:** Medium · **Effort:** M · **Area:** bindings, rust-core
- **What:** `rayon_parser_pool_wrapper.rs:53` runs the whole parallel parse **holding the GIL** (no
  `detach`); `producer_consumer_pipeline_wrapper.rs:154-205` blocks on `recv()` holding the GIL;
  `producer_consumer_pipeline.rs:116-120` sends records one at a time (unused `config.batch_size`
  suggests batch sends were intended) and clones a ≥512 KB buffer per chunk (`:98`);
  `rayon_parser_pool.rs:90-94` pays Cursor + copy on already-contiguous bytes;
  `boundary_scanner.rs:109` clones the boundary Vec per scan.
- **Fix:** Detach around parallel work and recv; batch channel sends; reuse the chunk buffer;
  slice-based parse entry (from PERF-3) for the rayon path.
- **Backlog:** new. Sequenced-with bd-ieoe (parallel-throughput measurement — fix before
  measuring).

### PERF-8 — Regexes recompiled on hot paths
- **Severity:** Medium · **Effort:** S · **Area:** rust-core
- **Where:** `src/field_linkage.rs:98` (recompiled per 880-linkage parse, called in loops from
  `record.rs:503,535,587,670`); `src/marcxml.rs:115,120` (two compiles per document, plus the
  `replace_all` preprocessing copies the whole XML document up to twice — peak ~3× document size)
- **Fix:** `static LazyLock<Regex>` (MSRV-safe per PKG-1; pattern already in `marc8_tables.rs`).
  Longer term, replace the marcxml preprocessing with namespace handling in the event loop.
- **Backlog:** new.

### PERF-9 — Three `String` allocations per data field for a 3-byte tag
- **Severity:** Medium · **Effort:** S (short term) / M (Tag newtype) · **Area:** rust-core
- **Where:** `iso2709_skeleton.rs:382-384`, `iso2709.rs:779`, `record.rs:190`
- **Fix:** Short term, thread the already-owned tag String through. Long term: `Tag([u8; 3])` Copy
  newtype — an API-rippling change that only gets cheaper before 1.0; decide during ARCH-4/1.0
  surface work.
- **Backlog:** new.

### PERF-10 — Low-severity grab bag
- **Effort:** S each · (a) `format!` per directory entry in all three writers — fold into ARCH-3;
  (b) whole-`bytes` extract copy at reader construction; (c) `vec![0u8; n]` zero-init memset in
  `read_record_data`; (d) whole-document string-only XML/CSV APIs, no streaming variants — fold
  into ARCH-1; (e) no-arg `get_fields()` makes 9 Rust calls; (f) `Leader::from_bytes` small-String
  noise (listed so it isn't re-investigated).
- **Backlog:** new; (a)→ARCH-3, (d)→ARCH-1.

---

## Theme 3: pymarc parity and wrapper correctness

### PAR-1 — `format_field` silently diverges between Rust and Python
- **Severity:** Medium · **Effort:** S · **Area:** rust-core, python
- **Where:** `src/record.rs:1175-1197` (skips $6, inserts " -- " for 6xx subdivisions) vs
  `mrrc/__init__.py:368-372` (plain space-join reimplementation)
- **What:** The same record formats differently per layer. Existing tests cover only the cases
  where both layers and pymarc agree.
- **Fix:** Check pymarc's actual behavior, make both layers match it; ideally delegate the Python
  method to the Rust implementation via PyO3. Add the parametrized Rust-vs-wrapper parity tests
  (6xx with subdivisions, fields containing $6, control fields) and sweep other reimplemented
  wrapper methods (`value()`, title-ish helpers) the same way.
- **Backlog:** new.

### PAR-2 — pymarc-oracle tests exist but run nowhere
- **Severity:** High · **Effort:** S · **Area:** ci, python-tests
- **What:** `tests/python/test_pymarc_parity.py` and `test_repeated_control_fields.py` are
  skip-guarded on pymarc, which is in no extra and no CI job — both silently skip in every CI and
  local run. The project's north-star compatibility target has zero executing oracle coverage.
- **Fix:** CI step `uv run --with 'pymarc==5.3.1' -m pytest <the two files>`; extend the oracle to
  value-level comparisons (`format_field()`, `value()`, `title()`, `as_marc()` byte-equality) —
  exactly the test shape that would have caught PAR-1.
- **Backlog:** new.

### PAR-3 — Python authority/holdings readers have zero happy-path tests
- **Severity:** Medium · **Effort:** S · **Area:** python-tests
- **What:** `AuthorityMARCReader`/`HoldingsMARCReader` appear only in error-path tests; no Python
  test ever successfully reads either record type or touches their accessors.
- **Fix:** `test_authority_holdings_reading.py` against the existing
  `tests/data/simple_authority.mrc`/`simple_holdings.mrc` fixtures, mirroring the Rust tests.
- **Backlog:** new.

### PAR-4 — Live-handle protocol lacks an edge-case test matrix
- **Severity:** Medium · **Effort:** S-M · **Area:** python-tests
- **What:** `test_field_handle_writeback.py` is good (14 tests) but misses: mutating the record
  while iterating `get_fields()`; handle behavior across `copy`/`deepcopy`; same-tag remove +
  re-add occurrence aliasing (stale vs silently-wrong); reader-produced vs constructed records.
- **Fix:** Extend the file with those four cases. Land before any ARCH-6 redesign to pin current
  semantics.
- **Backlog:** new.

---

## Theme 4: Test-suite integrity and robustness

### TEST-1 — Four vacuous test surfaces; ASAN never runs the integration tests
- **Severity:** High · **Effort:** S · **Area:** rust-tests, ci
- **What:** `tests/concurrent_gil_tests.rs` (10 tests asserting nothing about mrrc — local-variable
  simulations, stdlib semantics, references to Python test files that don't exist);
  `tests/memory_safety_asan.rs` (stdlib-only, **and** `memory-safety.yml` runs without `--tests` so
  the file never executes under ASAN); two `assert!(result.is_ok() || result.is_err())` tautologies
  in `rayon_parser_pool` unit tests; no successful-pipeline-run Rust unit test.
- **Fix:** Delete `concurrent_gil_tests.rs` (the real GIL proof is
  `test_gil_release_verification.py`); delete or rebuild `memory_safety_asan.rs` around actual mrrc
  parsing and add `--tests` to the ASAN workflow; replace tautologies with exact assertions.
- **Backlog:** new.

### TEST-2 — Authority/holdings writers have no real roundtrip verification at any layer
- **Severity:** Medium · **Effort:** M · **Area:** rust-tests
- **What:** Holdings writer output is never parsed back anywhere; authority has one shallow
  integration roundtrip; `tests/properties.rs` generates bibliographic records only. The writers
  flagged in ARCH-3 have no generative verification.
- **Fix:** `arb_authority_record()`/`arb_holdings_record()` strategies + roundtrip properties
  (leader/field strategies are reusable). Requires `PartialEq` on the record types (PKG-10b,
  one-line derives). Land with or before ARCH-3.
- **Backlog:** new.

### TEST-3 — Coverage is measured nowhere that matters
- **Severity:** Medium · **Effort:** S-M · **Area:** ci
- **What:** `coverage.yml` is dispatch-only, Rust-only, `fail_ci_if_error: false`; no pytest-cov.
- **Fix:** Scheduled run + codecov visibility; consider `cargo llvm-cov`; add a Python coverage
  lane. Visibility first, gating later if ever.
- **Backlog:** new.

### TEST-4 — Rust tests run on Linux only
- **Severity:** Medium · **Effort:** S · **Area:** ci
- **What:** No OS matrix on `test.yml`; cross-platform signal comes only from the Python wheel
  suite. Pure-Rust consumers' platform-sensitive paths are never tested on macOS/Windows as Rust.
- **Fix:** OS matrix on the cargo test job (or slim-PR/full-main split like `python-build.yml`).
- **Backlog:** new.

### TEST-5 — Fuzz coverage gaps: authority, holdings, BIBFRAME, lenient-mode salvage
- **Severity:** Medium · **Effort:** M · **Area:** fuzz
- **What:** Nine targets cover the bibliographic reader and the text formats; not fuzzed: authority
  reader, holdings reader (type-specific hooks diverge from bib), the BIBFRAME/RDF read path (the
  largest external-parser dependency surface), and the recovery salvage path gets only incidental
  lenient-mode coverage.
- **Fix:** `parse_authority`, `parse_holdings`, `parse_bibframe` targets + one binary target running
  `with_recovery_mode(Lenient)`. Pairs with OPS-3 (corpus persistence) for compounding value.
- **Backlog:** new.

### TEST-6 — Lenient-mode allocation amplification from claimed record length
- **Severity:** Medium · **Effort:** S-M · **Area:** rust-core, fuzz
- **Where:** `src/iso2709_skeleton.rs:269-279`
- **What:** In non-strict recovery, a truncated record claiming `record_length=99999` yields a
  zero-padded buffer of the claimed length (~4000× amplification per 25-byte stub; bounded per
  record by the ISO ceiling). DoS-class only.
- **Fix:** Size the recovery buffer to `min(claimed, available)`; add a fuzz/property assertion
  bounding allocation by input size on the lenient path.
- **Backlog:** new.

### TEST-7 — Test-suite cleanup strays
- **Severity:** Low · **Effort:** S · (a) `tests/create_sample_data.py` orphaned — delete or move
  to `scripts/`; (b) perf-claim drift in `test_queue_state_machine.py` docstrings — fix with
  PERF-6; (c) `tmp/stubtest-baseline.txt` dead scratch — delete.
- **Backlog:** new.

---

## Theme 5: Documentation accuracy

All verified by executing code against the installed extension. The docs site is otherwise
unusually complete and accurate (see "Verified clean"). DOC-1..DOC-5 are one natural PR.

### DOC-1 — README's primary Python example raises `TypeError`
- **Severity:** High · **Effort:** S · **Where:** `README.md:46` (`record.title()` — `title` is a
  property); same bug in the `mrrc.read()` docstring (`mrrc/__init__.py:1972`, rendered into the
  API reference). The first code a new user copies.
- **Backlog:** new.

### DOC-2 — `MarcError` documented as an importable Python class; it does not exist
- **Severity:** High · **Effort:** S · **Where:** `docs/reference/python-api.md:487-505`,
  `docs/guides/migration-from-pymarc.md:330`. The import raises `ImportError`; python-api's 3-class
  exception sketch contradicts the accurate 20-class hierarchy in `error-handling.md`.
- **Backlog:** new.

### DOC-3 — error-handling.md documents Python `from_path()`/`.with_source()` that don't exist
- **Severity:** High · **Effort:** S (doc fix now) · **Where:**
  `docs/reference/error-handling.md:271,288-289,316-333`. Rust builder methods transplanted into
  Python prose; examples raise `AttributeError`. Rewrite to actual behavior now; ARCH-7 is the
  API-side resolution.
- **Backlog:** new; sequenced-with ARCH-7.

### DOC-4 — python-api.md states the wrong `recovery_mode` default, inverting the safety story
- **Severity:** High · **Effort:** S · **Where:** `docs/reference/python-api.md:226` and kwargs
  table (claims `"strict"`; actual default `"permissive"`); table also omits `validation_level`
  and `max_errors` entirely. A reader of only this page believes errors raise by default when they
  are silently attached to `record.errors`.
- **Backlog:** new.

### DOC-5 — Rust API reference drift
- **Severity:** Medium · **Effort:** S · (a) `rust-api.md:333-338` `parse_batch_parallel` example
  has the wrong arity (tutorial shows it correctly); (b) `rust-api.md` `MarcError` table lists 5
  variants including the removed `InvalidRecord` (enum has 17) — point at `error-codes.md`;
  (c) `parse_batch_parallel(_limited)` are public in `_mrrc.pyi` but absent from the Python API
  reference.
- **Backlog:** new.

### DOC-6 — Documentation low-severity strays
- **Severity:** Low · **Effort:** S · (a) five orphaned doc pages not in nav (link-only — decide
  tier deliberately); (b) `mkdocs.yml:2` comment points at moved `DOCS-REORG.md`; (c)
  self-contradictory KeyError comment in `quickstart-python.md:28-29`; (d) docs.rs front page
  embeds the full README (badges, repo-relative links 404 off GitHub).
- **Backlog:** new.

---

## Theme 6: Packaging and toolchain

### PKG-1 — Declared MSRV 1.71 is false, untested, and steers downstream resolvers wrong
- **Severity:** High · **Effort:** S-M · **Area:** manifests, ci
- **Where:** `Cargo.toml:16`, `fuzz/Cargo.toml:6`, `docs/contributing/release-procedure.md:151`
- **What:** Locked deps require up to Rust 1.87 (oxrdf family; pyo3 needs 1.83, indexmap 1.82).
  The workspace cannot compile on 1.71. No CI job tests any MSRV — only the pinned 1.95 toolchain
  is ever exercised. Cargo's MSRV-aware resolver will use the false claim to select ancient dep
  versions for downstream users that mrrc has never been tested against.
- **Fix:** Set the real floor (1.87 today; or 1.85 if BIBFRAME is feature-gated per PKG-4); add a
  `cargo check --workspace` CI job pinned to the declared MSRV; update the two stale references.
- **Backlog:** new. **Corrects bd-j01q** (its "MSRV bump to 1.79" framing is moot — the de facto
  floor is already past it; the quick-xml 0.40 bump itself remains valid). Confirms the
  `LazyLock` recommendation in PERF-8 is MSRV-safe.

### PKG-2 — `Cargo.lock` is gitignored
- **Severity:** Medium · **Effort:** S · **What:** Non-reproducible CI/wheel builds; upstream
  breakage lands with no bisectable diff; `semver.yml`'s `paths: Cargo.lock` trigger can never
  fire. Cargo guidance since 2023: commit lockfiles even for libraries.
- **Fix:** Remove the ignore line, commit the lockfile(s); Dependabot manages bumps.
- **Backlog:** new.

### PKG-3 — Unused and misplaced dependencies
- **Severity:** Medium · **Effort:** S · **What:** Root `bytes`, `nom`, `encoding_rs`, `csv` have
  zero usage (the `csv::` hits are mrrc's own module); `anyhow`/`flate2` are example-only (belong
  in dev-deps); `src-python` `tempfile` unused; `smallvec` duplicated between deps and dev-deps.
- **Fix:** Delete/demote; consider `cargo machete` in check.sh.
- **Backlog:** new.

### PKG-4 — BIBFRAME/oxrdf stack is unconditional; no feature flags exist
- **Severity:** Medium · **Effort:** M · **What:** The oxrdf family is the heaviest tree cost: sets
  the 1.87 MSRV floor, drags the only duplicate dep (quick-xml 0.37), and is the path to the
  RUSTSEC-flagged `rand` (tracked, not actionable). Most MARC users never touch RDF.
- **Fix:** `bibframe = ["dep:oxrdf", "dep:oxrdfio"]`, default-on for zero behavior change; enable
  from `src-python` so the wheel keeps full functionality. Pre-1.0 is the cheap time.
- **Backlog:** new.

### PKG-5 — Version is triple-maintained across manifests
- **Severity:** Medium · **Effort:** S · **Fix:** `[workspace.package] version` + `version.workspace
  = true`; `dynamic = ["version"]` in pyproject (maturin reads from the bindings manifest).
  Simplifies the release procedure's verify-the-triple-bump step away.
- **Backlog:** new.

### PKG-6 — Distribution surface: placeholder PyPI metadata, no sdist, no Intel-Mac wheels
- **Severity:** High · **Effort:** M · **Area:** packaging, ci
- **What:** `pyproject.toml:12` publishes `email = "contact@example.com"`; authors string differs
  from Cargo.toml; no `[project.urls]` (PyPI page has no homepage/docs/changelog links). No sdist
  is built or published, and macOS wheels are arm64-only — Intel Macs, musl/Alpine, and anything
  outside the matrix gets "no matching distribution" with no source fallback.
- **Fix:** Fix metadata + urls; add `maturin sdist` to the release workflow; add `macos-13` (or
  universal2) to the wheel matrix; consider `musllinux_1_2`. PEP 639 license string and
  `Typing :: Typed` classifier while there.
- **Backlog:** new.

### PKG-7 — Ruff runs the default minimal ruleset
- **Severity:** Medium · **Effort:** S-M · **What:** No `[tool.ruff]` anywhere; bare `ruff check`
  enables only E4/E7/E9+F — a near-vacuous linter for a 3.10+ library.
- **Fix:** `target-version = "py310"`, select at least `E, W, F, I, UP, B, SIM, RUF`; fix the
  one-time fallout.
- **Backlog:** new.

### PKG-8 — `MarcError` (and other grower enums) not `#[non_exhaustive]`
- **Severity:** Medium · **Effort:** S-M · **What:** 17 public-field variants, exhaustively
  matchable downstream; the 0.8.x churn already forced semver-checks allows. Only 3 of ~23 public
  enums carry the attribute.
- **Fix:** `#[non_exhaustive]` on `MarcError` + variants; survey the rest (spec-fixed code lists
  stay exhaustive; growers like `PipelineError`, `RecoveryMode` should not).
- **Backlog:** new; pairs with bd-hgv6 (do both in one error-API pass).

### PKG-9 — Release profile untuned for a performance-marketed extension
- **Severity:** Medium · **Effort:** S · **What:** Only `opt-level = 3` (the default); no
  `lto`/`codegen-units = 1`/`strip`.
- **Fix:** Add them. Sequenced-with bd-dra6: land before publishing comparison numbers.
- **Backlog:** new.

### PKG-10 — Toolchain low-severity items
- **Severity:** Low · **Effort:** S each · (a) bindings crate escapes the whole lint regime
  (`[lints]` is per-crate) — move to `[workspace.lints]`, add `publish = false` to src-python;
  (b) record types lack `PartialEq` (needed by TEST-2) and `Display` (mrk-style rendering pairs
  with bd-g2xo); (c) `lazy_static` single use → `LazyLock`, delete the dep; edition 2021→2024 is
  mechanical once PKG-1 lands; (d) no workspace dependency inheritance for shared deps; (e) the
  per-version-wheels-vs-abi3 decision and free-threaded (3.13t/3.14t) wheel support are both
  unrecorded decisions — document the former, file an evaluation for the latter (PyO3 0.28
  supports free-threading and mrrc's GIL-release architecture is exactly the workload that
  benefits).
- **Backlog:** new.

---

## Theme 7: CI/CD and release operations

### OPS-1 — No required status checks: direct pushes to main bypass every CI gate
- **Severity:** Medium · **Effort:** S · **What:** The main ruleset has only deletion +
  non-fast-forward rules; the entire PR gate is advisory. A direct push lands untested and
  immediately deploys docs.
- **Fix:** Add required checks (Tests, Lint, wheel build) to the ruleset; keep ff-push allowed if
  desired.
- **Backlog:** new.

### OPS-2 — Scheduled-suite failures notify no one
- **Severity:** Medium · **Effort:** S · **What:** fuzz/miri/memory-safety nightlies fail silently
  for a solo dev (currently all green). GitHub also auto-disables cron after 60 days of repo
  inactivity.
- **Fix:** `if: failure()` step filing/updating a GitHub issue on all three.
- **Backlog:** new.

### OPS-3 — Fuzz corpus regrown from 26 seeds nightly and discarded
- **Severity:** Medium · **Effort:** S · **What:** Coverage depth never compounds; each 300s run
  re-explores from the committed seeds.
- **Fix:** `actions/cache` on `fuzz/corpus/<target>` with restore-keys; periodic `cargo fuzz cmin`
  re-commit. Multiplies the value of TEST-5's new targets.
- **Backlog:** new.

### OPS-4 — Dependabot PRs skip the entire Python wheel build/test gate
- **Severity:** Medium · **Effort:** S · **Where:** `python-build.yml:34,73` · **What:** A
  pyo3/maturin bump that breaks the extension merges green and fails first in the 30-job post-merge
  matrix.
- **Fix:** One ungated slim build+test job (1 OS × 1 Python) that runs for dependabot too.
- **Backlog:** new.

### OPS-5 — Release publish is not idempotent on partial failure
- **Severity:** Medium · **Effort:** S · **What:** All 25 wheels upload in one publish invocation
  without `skip-existing`; the GitHub Release step is in the same job; a partial failure makes
  re-runs die on "file already exists" — and the documented remedy (re-tag) collides identically.
- **Fix:** `skip-existing: true`; split GH Release creation into its own job.
- **Backlog:** new.

### OPS-6 — Cross-compiled wheels (10 of 25) are never imported before publish
- **Severity:** Medium · **Effort:** M · **What:** Validation greps the wheel filename only; an
  aarch64 wheel that segfaults on import would publish.
- **Fix:** QEMU import-smoke aarch64; container-based i686 smoke.
- **Backlog:** new.

### OPS-7 — Release-procedure doc has drifted from the actual workflow
- **Severity:** Medium · **Effort:** S · **What:** Says 15 wheels (actual 25); **still commands
  `bd` instead of `br`**; says "Rust 1.71+" (pinned 1.95 — same correction as PKG-1); no TestPyPI
  rehearsal exists anywhere, so first contact with PyPI validation is the production publish.
- **Fix:** Correct the doc; optionally add a `workflow_dispatch` TestPyPI input for rehearsal.
- **Backlog:** new.

### OPS-8 — Workflows lack top-level least-privilege `permissions:` blocks
- **Severity:** Medium · **Effort:** S · **What:** 10 of 15 workflows inherit the default
  `GITHUB_TOKEN` scope; defense-in-depth against a compromised third-party action.
- **Fix:** `permissions: contents: read` top-level everywhere; per-job elevation where needed;
  verify the repo default token setting is read-only.
- **Backlog:** new (bd-e13s covers SECURITY.md but not this).

### OPS-9 — Third-party actions unpinned in the release path
- **Severity:** Low-Medium · **Effort:** S · **What:** `pypa/gh-action-pypi-publish@release/v1` is
  a **moving branch** in the OIDC publish job — the single highest-value pinning target; other
  actions are tag refs.
- **Fix:** Pin release/publish-path actions to commit SHAs (Dependabot keeps them current).
- **Backlog:** new (bd-e13s does not cover this).

### OPS-10 — CI low-severity items
- **Severity:** Low · **Effort:** S each · (a) `PYO3_USE_ABI3_FORWARD_COMPATIBILITY` set in PR CI +
  check.sh but absent from `python-release.yml` — a new CPython fails first at release time;
  (b) three inconsistent Rust cache strategies; raw `actions/cache` in coverage/miri/memory-safety
  keyed only on Cargo.lock (stale); maturin builds compile cold — sccache would shave the 6-min job
  bounding the PR gate; (c) cargo-audit has no schedule trigger; (d) 18-line macOS whole-suite
  retry copy-pasted ×3 — `pytest-rerunfailures` gives per-test signal; (e) mypy/pyright block
  locally in check.sh but never run blocking in CI on `mrrc/` — local gate stricter than CI (gate
  the *current* mode in lint.yml; bd-bn65 stays the `--strict` follow-up); (f) check.sh omits
  cargo-semver-checks (CI-only) — note the gap or add it; the "~30s" claim only holds warm;
  (g) missing `timeout-minutes` on miri/coverage/docs/release jobs (360-min default);
  (h) `unsafe`-block comments cite a nonexistent `GIL_RELEASE_IMPLEMENTATION_PLAN.md`
  (`src-python/src/readers.rs:295`) — replace with the actual invariant.
- **Backlog:** new.

---

## Repository cleanup (one small PR)

- **CLEAN-1** (Low/S): delete empty untracked `examples_tmp/`; delete stray
  `test-output/walkthrough-pr79-*.md`; `site/` is fine (gitignored build output).
- **CLEAN-2** (Low/S): delete placeholder `src/main.rs` (5-line banner binary) until the real CLI
  lands (bd-e24e / #94), or make it that CLI's seed.
- **CLEAN-3** (Low/S): delete dead `src/field_collection.rs` (part of ARCH-5).
- Plus TEST-7 items (orphaned `tests/create_sample_data.py`, `tmp/stubtest-baseline.txt`).

---

## Proposed epics

Suggested groupings for ticketing; each epic lists its findings in rough dependency order.

| Epic | Findings | Notes |
|---|---|---|
| **E1: Docs accuracy sweep** | DOC-1, DOC-2, DOC-3 (doc half), DOC-4, DOC-5, DOC-6, OPS-7 | One or two PRs, all S-effort, highest user-facing value per hour. Do first. |
| **E2: Packaging & toolchain truth** | PKG-1, PKG-2, PKG-3, PKG-5, PKG-9, PKG-10c/d, then PKG-4, PKG-6, PKG-7, PKG-8, PKG-10a/b/e | PKG-1 (MSRV) is the anchor; PKG-4 (feature flag) interacts with the MSRV floor. |
| **E3: CI & release hardening** | OPS-1..OPS-6, OPS-8, OPS-9, OPS-10, TEST-3, TEST-4 | All independent S/M items; can be drained opportunistically. |
| **E4: Test integrity & robustness** | TEST-1, TEST-2 (+PKG-10b), TEST-5, TEST-6, TEST-7, PAR-2, PAR-3, PAR-4 | PAR-2 (oracle in CI) and TEST-1 (delete vacuous tests) first. |
| **E5: Read-path performance** | PERF-1, PERF-2, PERF-3, PERF-4, PERF-5, PERF-6, PERF-8, PERF-7, PERF-9, PERF-10 | Absorbs/concretizes bd-8zv5; gates bd-dra6 (don't publish numbers before PERF-1..3 + PKG-9). |
| **E6: 1.0 API coherence** | ARCH-1..ARCH-9, PAR-1, PERF-9 (Tag decision) | The big one; design-first. ARCH-3 (writers) is independently shippable and unblocks bd-cdey; ARCH-6 needs PAR-4 first. |

A sensible sequence for a solo dev: E1 → quick wins from E2/E3 → E4 → E5 → E6, with E6's ARCH-3
pulled forward whenever bd-cdey gets scheduled.

## Quick wins (first batch, all S-effort)

DOC-1..DOC-4 (one PR) · PERF-1 (BufReader) · PKG-1 (MSRV truth + CI check) · PKG-2 (commit
Cargo.lock) · PKG-3 (unused deps) · OPS-5 (skip-existing) · OPS-7 (release-procedure `bd`→`br` +
wheel count) · TEST-1 (delete vacuous tests, ASAN `--tests`) · CLEAN-1..3.

## Backlog reconciliation

Every existing bead/issue this review touched, and the recommended disposition:

| Item | Disposition |
|---|---|
| bd-8zv5 (continue read-path optimization) | **Replace or re-describe** with the concrete PERF-1..PERF-10 content; consider making it the E5 epic. |
| bd-dra6 (publish pymarc comparison) | **Sequence after** PERF-1..3 + PKG-9 (they change the baseline). Review confirmed no pymarc-comparison code exists (bd-l9zh's finding stands). |
| bd-j01q (quick-xml 0.40, MSRV 1.79) | **Premise corrected by PKG-1**: the de facto MSRV floor is already 1.87. The quick-xml bump itself remains valid; merge into E2 or do alongside PKG-1. |
| bd-cdey (MARC-8 in binary writer) | **Blocked-ish on ARCH-3** — consolidate the writers first so MARC-8 is wired once. |
| bd-hgv6 (MarcError accessor consolidation) | **Pair with PKG-8** (`non_exhaustive`) as one error-API pass. |
| bd-mcei (split `mrrc/__init__.py`) | Keep; **sequenced-with ARCH-6** — isolate the handle machinery as part of the split regardless of the ownership decision. |
| bd-e13s (community infra files) | Keep; note it does **not** cover OPS-8/OPS-9 (those are separate). |
| bd-bn65 (mypy --strict) | Keep as the strict follow-up; OPS-10e (gate current mode in CI) is distinct and should come first. |
| bd-fe3l (ParseContext threading) | Keep; related to ARCH-7. |
| bd-g2xo / #93 (.mrk support) | Keep; PKG-10b's `Display` suggestion pairs with it. |
| bd-e24e / #94 (CLI) | Keep; CLEAN-2 (placeholder main.rs) is its seed or its casualty. |
| bd-1dbu, bd-l9zh (+children), bd-3pd5 (bench infra) | Unchanged; review independently confirmed the dead harnesses, stale results.md, and orphaned perf tests. |
| bd-ieoe (parallel-throughput measurement) | **Sequence after PERF-7** (fix the parallel paths before measuring them). |
| bd-c5sh (MARCJSON str parity) | Subsumable into ARCH-2's contract work, or keep as its concrete first instance. |
| bd-2snh, bd-1pej (query DSL semantics) | Keep; natural inputs to ARCH-5's consolidation. |
| bd-qhll (recovery merge into skeleton) | Keep; TEST-6 touches the same code — coordinate. |
| bd-wuo1 (non-ASCII tag bytes) | Keep; unchanged. |
| bd-tfre (real-corpus test) | Keep; complements PAR-2's oracle. |
| bd-4ap2 (compile-checked doc examples) | Keep; would have caught DOC-5a-style drift. |
| #92 (DC/CSV read path) | **Reframe per ARCH-2**: declare the round-trip contract for all eight formats, not just two functions. |
| #225 (query discoverability) | Keep; ARCH-5 is the structural fix, #225 the doc symptom. |
| #88 (CONTRIBUTING.md) | Keep; fold the OPS-7 release-procedure corrections into the same docs-maintenance pass if convenient. |
| mrrc-mgd (src-python/tests/ not in CI) | **Overtaken by events** — the directory no longer exists on disk; confirm and close. |
| mrrc-3ivc / #48, mrrc-b7l / #4, mrrc-1qku | Unchanged. |

## Verified clean (do not re-investigate)

- **Parsing robustness:** leader invariants validated before use; every directory/data slice
  clamped; fixed-width numeric parsers bound all directory math (no overflow path); array indexing
  on input paths guarded; the two `.expect()`s in the directory walker are proven infallible.
- **XML safety:** both XML readers resolve only built-in entities and process no DTDs — entity
  expansion is structurally impossible. JSON recursion is absorbed by serde_json's 128-level limit
  before mrrc sees it.
- **`unsafe`:** all five blocks are the sound `Python::assume_attached()` idiom inside
  `#[pymethods]`; only owned `Send` data crosses GIL-release sections (comment hygiene: OPS-10h).
- **Publishing:** PyPI uses OIDC Trusted Publishing; no static tokens; crates.io publish is
  deliberately manual.
- **Inner parse loop:** byte-wise, allocation-disciplined, `SmallVec`/`OnceLock`/monomorphization
  all verified appropriate; `Arc<Vec<MarcError>>` shares a static empty instance.
- **Test infrastructure:** the property suite, the three-layer error-coverage harness
  (`error_coverage.toml` → tests → `verify_error_docs.py`), the deterministic GIL-release proof,
  stubtest gating (no allowlist), fuzz regression auto-discovery, and snapshot scoping are all
  exemplary. check.sh is a faithful superset of the PR gate apart from noted items.
- **Docs:** nav integrity (strict build); DC/CSV write-only correctly declared; no MARC-8 write
  claims outside `docs/history/`; the bogus "N/100 GIL" claim appears in code comments only, not
  user-facing docs; version/MSRV strings are mutually consistent (their shared *value* is PKG-1);
  CHANGELOG breaking-change hygiene is good; `missing_docs` enforced at zero warnings;
  `error-codes.md` anchors match `help_url` exactly.
- **PyO3 currency:** 0.28.2, Bound API throughout, no deprecated patterns; single duplicate dep in
  the tree (quick-xml, via oxrdfxml); clippy all+pedantic deny in CI with tuned thresholds;
  semver-checks gate with documented allows; `requires-python` matches the CI matrix; mixed
  maturin layout correct; fuzz workspace correctly isolated.
- **CI topology:** concurrency groups everywhere; Dependabot across all three ecosystems;
  path-filter hygiene with deliberate gap-fillers; slim-PR/full-main matrix split; release wheels
  fully pytest-tested on native platforms before publish; PR gate ≈6.5 min fully parallel.
