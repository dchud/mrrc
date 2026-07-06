# API Coherence Proposals — 1.0

Companion to [`code-review-findings-v0.8.2.md`](code-review-findings-v0.8.2.md), written 2026-07-01
against `main` @ `4939465`. That document is the review record; this one reconciles its findings
against the bead tracker as it stands after the 2026-06-25 epic restructure (Part A), and proposes
the concrete API shapes for the 1.0 coherence work (Part B).

mrrc is a Rust library for MARC bibliographic records (the ISO 2709 binary interchange format plus
several text formats), with a Python API exposed through PyO3 bindings and designed as a faster
drop-in for pymarc, the incumbent pure-Python MARC library. That goal drives everything here.

The organizing principle, quoted from `bd-d7aq.9`: **the obvious, pymarc-shaped path must be the
fastest path; performance primitives stay internal.** pymarc compatibility is the primary design
axis for the Python wrapper. Backward compatibility with mrrc's own 0.8.x surface is a non-goal;
breaking the current mrrc shape is acceptable, breaking pymarc-shape parity is not.

## Who this is for and how to use it

Audience: the implementation team — humans and agents — planning and executing the 1.0 API work
in stages. The pipeline is: this document (design input) → `bd-d7aq.9` (the binding spec, which
pins the remaining decisions) → the implementation beads, which align to that spec. Use it as
follows:

- **Part A** is the reconciliation record. Read it to learn which review findings are already
  resolved, which bead owns each open one, and the two gaps that had fallen through. Nothing in
  Part A needs a new decision; it is the "what is already true" baseline.
- **Part B** is the design content: five proposals, each ending in a definition of done, plus the
  cross-cutting tenets. Its recommendations stand unless the `bd-d7aq.9` spec overturns them with
  recorded rationale.
- **"Decisions the `bd-d7aq.9` spec must pin"** (end of Part B) is the checklist of what the spec
  still has to decide, separated from what this document already settles.
- **"Build sequence"** (last section) is the staging: what can start now, what the spec gates, and
  what runs in parallel. Plan epics directly from its stages.

Verification baseline: every codebase claim and `file:line` anchor below was verified against
`main` @ `4939465`. Anchors may drift as later work merges; the claims were true at that commit.
Claims that could only be sourced from the tracker, not the code, say so inline (e.g. "per
`bd-d7aq.2` comment").

## How to read this document

**Finding codes** (`ARCH-1`, `PERF-10d`, …) are the stable IDs from the companion review document,
where each has full detail. The families: ARCH = architecture and API surface; PERF = performance;
PAR = pymarc parity and wrapper correctness; TEST = test-suite integrity; DOC = documentation
accuracy; PKG = packaging and toolchain; OPS = CI/CD and release operations; CLEAN = repository
cleanup. Codes that matter here are also glossed in one clause where they appear, so the companion
is reference, not required reading.

**Bead IDs** (`bd-xxxx`) are issues in the project's `br` tracker; `bd-d7aq` is the 1.0
API-coherence epic and `bd-d7aq.N` are its children. The bead map below summarizes every bead this
document references, so the document is navigable without tracker access. Statuses are as of
2026-07-01; tracker changes after that date are not reflected here.

**Terms** used throughout:

| Term | Meaning here |
|---|---|
| pymarc | The incumbent pure-Python MARC library. mrrc's wrapper mirrors its API; "pymarc shape" means the names and semantics pymarc users already know. |
| thin wrapper | Design tenet: the Python layer defines API shape only and delegates the work to Rust; it must not reimplement logic. |
| pit of success | Design goal: the obvious way to use the library is also the correct and fast way. The failure mode is a fast path that exists but that only experts find. |
| GIL, detach | Python's Global Interpreter Lock serializes access to Python objects. PyO3's `py.detach` releases it so Rust work can run in parallel with Python threads; "GIL release" and "detach" name the same thing. |
| boundary crossing | One Python→Rust call through PyO3. Each crossing has fixed overhead, so per-record or per-field crossings dominate hot loops. |
| live handle | A Python `Field` object bound to a position inside a record: reads and writes go through to the record. Its opposite, a detached field, owns its own data. |
| generation counter | A per-record version number bumped on field removals; a handle holding an older generation raises `StaleFieldError` instead of touching the wrong field. |
| deep clone | Copying a field's entire contents (tag plus every subfield string) across the boundary. |
| oracle, oracle corpus | Tests that run identical input through mrrc and pymarc and compare the outputs; the fixture set they run over is the oracle corpus. |
| permissive mode | The pymarc-compatible read mode: a record that fails to parse yields `None` (with the exception available on `current_exception`) instead of raising. |
| front door | The one supported public entry point for a capability, as opposed to internal machinery that happens to be reachable. |
| prelude | A Rust module re-exporting the small set of types most programs need, so one glob import covers the common case. |
| tokenizers, pydantic-core, Polars | Widely used Rust-backed Python libraries with mrrc's architecture; `bd-d7aq.9` cross-checked their API conventions as precedent. |

### Bead map

Open beads — the actionable set. "Lands in" points at the Part B proposal and build-sequence stage
that owns the work.

| Bead | Status | One-line description | Lands in |
|---|---|---|---|
| `bd-d7aq` | open | The 1.0 API-coherence epic; every `bd-d7aq.N` below is its child | umbrella |
| `bd-d7aq.1` | open | Consolidate the three copy-pasted ISO 2709 writers (ARCH-3); carries the manual-`PartialEq` residue from TEST-2 | Stage 1 |
| `bd-d7aq.2` | open | Fix the `format_field` Rust/Python divergence and add Rust-vs-wrapper parity tests (PAR-1) | Stage 1 |
| `bd-d7aq.3` | open | Format front door: completed `Format` enum + free-function convention (ARCH-1, ARCH-2); absorbs streaming (PERF-10d), `bd-c5sh`, GitHub #92, and GAP-1 | Proposal 3, Stage 5 |
| `bd-d7aq.4` | open | Field-ownership design spike and implementation (ARCH-6) | Proposal 2, Stage 4 |
| `bd-d7aq.5` | open | Reader builder-option parity across the three record types (ARCH-7) | unstaged, independent |
| `bd-d7aq.6` | open | Regroup the flat Rust module layout (ARCH-4) | Proposal 5, Stage 7 |
| `bd-d7aq.7` | open | Query/helper consolidation under `query/` (ARCH-5, CLEAN-3) | Proposal 4, Stage 6 |
| `bd-d7aq.8` | open | Prelude and re-export tiering; parse skeleton goes `pub(crate)` (ARCH-8, ARCH-9) | Proposal 5, Stage 7 |
| `bd-d7aq.9` | open | The gating design spec this document is input to | Stage 2 |
| `bd-d7aq.10` | open | GAP-1: remove the MARCXML namespace-strip document copies (filed by this reconciliation) | Proposal 3, Stage 5 |
| `bd-d7aq.11` | open | GAP-2: collapse the four public batch/parallel primitives into one bulk entry point (filed by this reconciliation; blocked by `bd-d7aq.9`) | Proposal 1, Stage 3 |
| `bd-kpl9` | open | Batch `MARCReader`'s internal iteration — the measured 1.29x per-record residual of PERF-6 | Proposal 1, Stage 3 |
| `bd-mcei` | open | Split `mrrc/__init__.py` (2200+ lines) into pymarc-shaped modules; isolates the handle machinery | Proposals 2 and 5, Stage 4 |
| `bd-xjws` | open | Inline `Tag` representation (PERF-9 + PERF-10f); API-rippling, so scheduled inside the 1.0 window | Stage 7 |
| `bd-2snh` | open | Query DSL multi-subfield matching semantics; decided inside `bd-d7aq.7` | Proposal 4, Stage 6 |
| `bd-1pej` | open | Handle semantics for query results; gated by the Proposal 2 decision | Proposal 4, Stage 6 |
| `bd-c5sh` | open | MARCJSON string entry-point parity; absorbed into `bd-d7aq.3` | Proposal 3, Stage 5 |
| `bd-cdey` | open | MARC-8 encoding in the binary writer; unblocked once `bd-d7aq.1` consolidates the writers | after Stage 1 |
| `bd-fe3l` | open | ParseContext threading; related input to reader option parity (ARCH-7) | context for `bd-d7aq.5` |
| `bd-g2xo` | open | `Display`/.mrk rendering (the Display half of PKG-10b) | independent |
| `bd-0glt.4`, `bd-0glt.5` | open | Free-threaded wheels and the wheel-strategy doc (PKG-10e), under the `bd-0glt` free-threading epic | independent |

Closed beads referenced for context:

| Bead | Disposition |
|---|---|
| `bd-0pg6` (children .1–.8) | Read-path performance epic; closed 2026-06-21 with all ten children merged |
| `bd-v9cm` | Profiling epic; closed 2026-06-28 — its measurements back several claims below |
| `bd-pk26.3` | Closed (PR #306); was the blocker on `bd-d7aq.1` |
| `bd-pk26.5` | The PAR-4 handle edge-case matrix; closed (PR #301) — pins current handle semantics |
| `bd-pk26.7` | TEST-6 lenient-mode allocation bound; closed |
| `bd-4d8u` | PERF-10c read-into-uninit; closed won't-fix 2026-06-27 (measured as a ~1% regression) |
| `bd-yfe6.9` | Workspace lints + `publish = false` (PKG-10a); closed (PR #340) |
| `bd-mwhj` | Closed, superseded by the `bd-0glt` free-threading epic |
| `bd-9cux`, `bd-g4tn` | OPS follow-ups (the OPS-1 deferral and OPS-10f); both closed |

---

## Part A: Ticketing reconciliation

The findings doc's remediation section was last updated 2026-06-13; the `bd-d7aq` epic was
restructured 2026-06-25 and the `bd-v9cm` profiling work closed 2026-06-28. This section re-derives
coverage from the current bead state and code rather than trusting that section. Two genuine gaps
were found and filed as `bd-d7aq.10` (GAP-1) and `bd-d7aq.11` (GAP-2), children of the `bd-d7aq`
epic; their descriptions are reproduced at the end of this part.

How to read the tables: the first column names the finding with a one-clause gloss; the second
states what was verified in the code at the baseline commit; the last names what covers it —
"closed" means resolved and verified, an open bead means the remaining work is ticketed there.

### Themes 1 and 3 — architecture (ARCH) and pymarc parity (PAR)

Every ARCH and PAR finding maps to an open child of `bd-d7aq` or a recorded resolution. No gap in
this group.

| Finding | Verified current state | Coverage |
|---|---|---|
| ARCH-1 (formats trait covers 1 of 8) | Still true: `Format` enum has one variant (`src/formats/mod.rs:92-95`); Python `read()`/`write()` reject everything but `"marc"` (`mrrc/__init__.py:2252-2256`, `2316-2325`) | `bd-d7aq.3` (open) |
| ARCH-2 (round-trip contract undeclared) | Still true; MODS does read (`src/mods.rs:600`), DC/CSV remain write-only | `bd-d7aq.3` (open); GitHub #92 open, reframed |
| ARCH-3 (three copy-pasted writers) | Still true: authority/holdings use unchecked `as u32` (`src/authority_writer.rs:131-132`, `src/holdings_writer.rs:133-134`) vs checked `u32::try_from` in the bib writer (`src/writer.rs:243-262`). The PERF-10a `format!`-per-directory-entry portion already landed (`push_zero_padded` at `authority_writer.rs:86-87`, `holdings_writer.rs:86-87`, PR #335) | `bd-d7aq.1` (open; now unblocked — its blocker `bd-pk26.3` closed in PR #306) |
| ARCH-4 (flat module layout) | Still true: ~38 flat modules beside `bibframe/` and `formats/` (`src/lib.rs:110-158`) | `bd-d7aq.6` (open) |
| ARCH-5 (query trait sprawl) | Still true: all eight query/helper modules flat; `field_collection` still exported (`src/lib.rs:123`) | `bd-d7aq.7` (open); design inputs `bd-2snh`, `bd-1pej` (open) |
| ARCH-6 (live-handle deep clones) | Still true: `_refresh` fetches a full field clone on every delegated access (`mrrc/__init__.py:275-306` → `src-python/src/wrappers.rs:859-864`); details in Part B | `bd-d7aq.4` (open; now unblocked — its blocker `bd-pk26.5`/PAR-4 closed in PR #301) |
| ARCH-7 (reader builder parity) | Still true: builder options exist on the bibliographic `MarcReader` only | `bd-d7aq.5` (open); `bd-fe3l` related |
| ARCH-8 (flat re-exports, no prelude) | Still true: 25 `pub use` statements re-export ~30 types flat (`src/lib.rs:160-189`) | `bd-d7aq.8` (open) |
| ARCH-9 (parse skeleton is pub) | Still true: `iso2709`/`iso2709_skeleton` public (`src/lib.rs:137,139`) | `bd-d7aq.8` (open) |
| PAR-1 (`format_field` divergence) | Still true: Rust `Field::format_field` (`src/record.rs:1196`) vs the wrapper's plain space-join (`mrrc/__init__.py:442-449`); the oracle corpus lacks the shapes that expose it (per `bd-d7aq.2` comment) | `bd-d7aq.2` (open) |
| PAR-2 (oracle runs nowhere) | Resolved: PR #286; CI installs `[test,oracle]` (`python-build.yml:96,195`) | closed |
| PAR-3 (authority/holdings happy-path tests) | Resolved: PRs #298/#301 | closed |
| PAR-4 (handle edge-case matrix) | Resolved: `bd-pk26.5`, PR #301 — pins current handle semantics ahead of any ARCH-6 redesign | closed |

### Theme 2 — performance (PERF)

The `bd-0pg6` epic closed 2026-06-21 with all ten children merged; the `bd-v9cm` profiling epic
closed 2026-06-28. Residuals were spot-verified:

| Finding | Verified current state | Coverage |
|---|---|---|
| PERF-1..5, PERF-7 | Resolved: `bd-0pg6.1-.5`, `bd-0pg6.8` | closed |
| PERF-6 (batching amortizes nothing) | GIL detach is now per batch — `BatchedReader`, up to 200 records / 300 KB per `py.detach` (`src-python/src/readers.rs:1-6,44-47`). The residual per-record `__next__` crossing + object construction is the measured 1.29x bottleneck | `bd-0pg6.6` closed; residual is `bd-kpl9` (open) |
| PERF-8, regex half | Resolved: `LazyLock` statics (`src/field_linkage.rs:39-40`, `src/marcxml.rs:111-116`; PR #334) | `bd-0pg6.7` closed |
| PERF-8, preprocessing half | Not resolved: `strip_marcxml_ns` still copies the whole document (see GAP-1) | `bd-d7aq.10` (filed) |
| PERF-9 + PERF-10f (tag/leader Strings) | Open by design; profiling confirmed tags are the top parse allocator by count (~37 tiny allocs/record) and tag hashing ~10% of read self-time | `bd-xjws` (open) |
| PERF-10a (writer `format!` per directory entry) | Resolved: PR #335 (`push_zero_padded` in all three writers) | closed; writer consolidation itself stays `bd-d7aq.1` |
| PERF-10b (whole-`bytes` copy at construction) | Resolved: `bytes` input is now a borrowed `PyBytesBuffer` (`src-python/src/backend.rs:79,182-190`; PR #335) | closed |
| PERF-10c (zero-init memset) | Resolved by decision: read-into-uninit measured as a ~1% regression on typical records; `bd-4d8u` closed won't-fix 2026-06-27 | closed |
| PERF-10d (no streaming format APIs) | Open, folds into the format front door per that bead's text | `bd-d7aq.3` (open) |
| PERF-10e (no-arg `get_fields()` = 9 calls) | Resolved: now two Rust calls, `control_fields()` + `fields()` (`mrrc/__init__.py:1118-1131`; PR #335) | closed |

### Themes 4-7 and cleanup — TEST, DOC, PKG, OPS, CLEAN

Verified compactly; no gaps in this group.

| Finding group | Status |
|---|---|
| TEST-1, TEST-3, TEST-4, TEST-5, TEST-7 | Resolved (PRs #303/#305/#310/#315/#319/#320/#321) |
| TEST-2 | Resolved (PR #306); the PartialEq residue (manual impl ignoring `errors: Arc<Vec<MarcError>>`) rides in `bd-d7aq.1` |
| TEST-6 | Resolved: `bd-pk26.7` closed |
| DOC-1..6 | Resolved (PRs #278/#279/#319) |
| PKG-1..9 | Resolved (PRs #280/#287/#289/#291/#296/#299/#304/#307/#313/#318) |
| PKG-10a | Resolved: workspace lints + `publish = false` (PR #340, closed `bd-yfe6.9`) |
| PKG-10b | PartialEq half in `bd-d7aq.1` (open); Display half moved to `bd-g2xo` (open) |
| PKG-10c/d | Resolved (PRs #280/#288/#304) |
| PKG-10e | Carried forward: `bd-mwhj` closed superseded by the `bd-0glt` free-threading epic; wheel-strategy doc is `bd-0glt.5`, free-threaded wheels `bd-0glt.4` (open) |
| OPS-1..9 | Resolved (PRs #279/#295/#300/#302/#308 + `bd-9cux` closed; `main` ruleset now enforces 10 required checks) |
| OPS-10a | Resolved: `PYO3_USE_ABI3_FORWARD_COMPATIBILITY` present in `python-release.yml` |
| OPS-10b | Resolved: unified Rust caching (Swatinem/sccache) across the main workflows; raw `actions/cache` remains only for the fuzz corpus (`fuzz.yml:62,73`), which is OPS-3's persistence fix, not drift |
| OPS-10c | Resolved: `audit.yml` has a daily schedule (`audit.yml:27-28`) with failure-files-an-issue handling |
| OPS-10d | Resolved: `pytest-rerunfailures` per-test reruns (`python-build.yml:208-210`, `python-release.yml:274-276`) |
| OPS-10e | Resolved: mypy + pyright run blocking on `mrrc/` in CI (`lint.yml:134-166`) |
| OPS-10f | Resolved: `bd-g4tn` closed |
| OPS-10g | Resolved: `timeout-minutes` present in all 16 workflows |
| OPS-10h | Resolved: no `GIL_RELEASE_IMPLEMENTATION_PLAN` references remain in `src-python/src/` |
| CLEAN-1, CLEAN-2 | Resolved (PR #319) |
| CLEAN-3 (`field_collection` deletion) | Open, inside `bd-d7aq.7` |

### Genuine gaps

Two items were neither resolved nor covered by an open bead; both are now filed as children of
`bd-d7aq`.

#### GAP-1 — MARCXML namespace-strip preprocessing still copies the whole document

`strip_marcxml_ns` (`src/marcxml.rs:124-127`) runs two `Regex::replace_all` passes plus a
`to_string()` over the full input before any parsing starts, on every call to `marcxml_to_record`
(`src/marcxml.rs:465`) and `marcxml_to_records` (`src/marcxml.rs:509`) — up to two full-document
copies, ~3x peak memory on namespace-prefixed input. This is the second half of PERF-8.
`bd-0pg6.7`'s description named the longer-term fix ("replace the marcxml regex preprocessing with
namespace handling inside the existing event loop"), but the bead closed on PR #334, which
delivered only the `LazyLock` hoist, and no open bead carries the residual. The reader already
walks quick-xml events directly (`read_marcxml_record`), so prefix handling belongs in that loop.
`bd-d7aq.3` folds in PERF-10d (streaming variants), which would likely rework this path, but its
text does not mention the preprocessing copy — the residual is recorded nowhere.

Filed as `bd-d7aq.10`:

```
Title: Remove the MARCXML namespace-strip document copies by handling namespaces in the event loop
Type: task  Priority: p3  Parent: bd-d7aq

strip_marcxml_ns (src/marcxml.rs:124-127) allocates up to two full copies of the input document
(RE_XMLNS_DECL.replace_all, then RE_NS_PREFIX.replace_all plus to_string) before parsing starts, so
marcxml_to_record and marcxml_to_records (src/marcxml.rs:465, 509) peak at roughly 3x document size
on namespace-prefixed input. The reader already walks quick-xml events directly
(read_marcxml_record); resolving namespace prefixes on element names inside that event loop removes
the preprocessing pass entirely. This is the second half of PERF-8 in
docs/design/code-review-findings-v0.8.2.md, named in bd-0pg6.7's description but not delivered by
its fix (PR #334 hoisted the regexes into LazyLock statics only). Sequencing: if bd-d7aq.3's format
front door adds streaming MARCXML read (PERF-10d), fold this into that rework rather than doing it
separately.
```

#### GAP-2 — The public batch/parallel surface consolidation is designed but not ticketed

`bd-d7aq.9`'s scope maps "ONE clean batch/parallel API collapsing
parse_batch_parallel/RecordBoundaryScanner/pipeline/batched-readers" to `bd-d7aq.3` — but
`bd-d7aq.3`'s actual description is the format front door (ARCH-1 + ARCH-2) and never mentions the
parallel primitives, and `bd-kpl9` covers only `MARCReader`'s internal batching. The work of
collapsing the four public bulk-read surfaces — `RecordBoundaryScanner`,
`ProducerConsumerPipeline`, `parse_batch_parallel`, `parse_batch_parallel_limited` (all public in
`mrrc/_mrrc.pyi:974-1005`) — into one supported entry point and demoting the rest to internal has
no implementation bead. It may be deliberate to file implementation beads only after the
`bd-d7aq.9` spec exists; the gap was flagged because the epic's cross-reference implied the
work was covered when it was not. `bd-d7aq.9`'s scope line has since been corrected to point at
`bd-d7aq.11` instead of `bd-d7aq.3`.

Filed as `bd-d7aq.11` (blocked by `bd-d7aq.9`):

```
Title: Collapse the public batch/parallel read primitives into one supported bulk entry point
Type: task  Priority: p2  Parent: bd-d7aq  Blocked by: bd-d7aq.9

The Python surface exports four overlapping ways to bulk-read ISO 2709 besides MARCReader:
RecordBoundaryScanner, ProducerConsumerPipeline, parse_batch_parallel, and
parse_batch_parallel_limited (mrrc/_mrrc.pyi:974-1005). The fast path today requires composing the
scanner and parse_batch_parallel by hand, which is the pit-of-success failure bd-d7aq.9 is built
around. Ship the ONE bulk entry point the bd-d7aq.9 spec pins (see
docs/design/api-coherence-proposals.md, Proposal 1) and demote the scanner, pipeline, and
parse_batch_* functions to internal names or delete them; update mrrc/_mrrc.pyi and the docs
accordingly. Context: bd-d7aq.9's scope line mapped this work to bd-d7aq.3, but bd-d7aq.3 is the
format front door (ARCH-1/ARCH-2) and does not cover the parallel primitives, and bd-kpl9 covers
only MARCReader's internal batching; this bead is that missing implementation ticket.
```

---

## Part B: API coherence proposals

### The pit-of-success failure, measured

The numbers that reframed the epic (measured 2026-06-25, realistic 18-field records, release
build; recorded in `bd-kpl9`):

| Path | Throughput | vs pymarc |
|---|---|---|
| `for record in MARCReader(path)` — the pymarc shape | ~44k rec/s | 1.29x |
| `RecordBoundaryScanner` + `parse_batch_parallel` — hand-composed primitives | ~148k rec/s | 4.4x |

The bulk path proves the Rust parser is fast; the ergonomic API hides it, and a maintainer
reflexively reached for the slow shape. The same failure repeats at field granularity: the
canonical pymarc loop deep-clones every field at least twice (details in Proposal 2). The
proposals below each state how the obvious path becomes the fast path without moving off the
pymarc shape.

### Proposal 1 — Read: batch internally, expose one bulk entry point

**Current state.** The GIL story is already fixed: `PyMARCReader` parses up to 200 records /
300 KB per `py.detach` and serves parsed records from a queue
(`src-python/src/readers.rs:1-6,44-47`). What remains per record is one PyO3 `__next__` crossing
(`readers.rs:213`), one `PyRecord` allocation, and one Python-side `_wrap_record` construction
(`mrrc/__init__.py:1930-1952`) — Python-overhead-bound, per `bd-kpl9`.

**Surface (unchanged) and the one new entry point:**

```python
# The obvious path — identical surface, batched internals (bd-kpl9)
for record in MARCReader("records.mrc"):
    print(record.title)

# The one supported bulk entry point (final name pinned by the bd-d7aq.9 spec)
records = mrrc.read_all("records.mrc")   # list[Record]; boundary scan + parallel parse inside
```

**Boundary shape.** The wrapper's `__next__` pops from a Python-side buffer refilled by a single
Rust call per batch, so the boundary is crossed once per ~200 records instead of once per record:

```rust
/// One crossing per batch. Parse happens inside one GIL release (optionally
/// rayon-parallel for file/bytes backends); the whole batch is materialized
/// as Python objects under a single GIL hold. RecordOutcome (batched_reader.rs)
/// already models per-record success/error ordering, which permissive mode
/// requires: the wrapper must still yield None per failed record and feed
/// current_exception / current_chunk in stream order.
fn next_batch(mut slf: PyRefMut<'_, Self>) -> PyResult<Vec<PyRecordOutcome>>
```

`read_all` is the same machinery ending in one list: scan boundaries, `parse_batch_parallel` the
chunks inside a detach, materialize once. Its name follows the batch-method convention tokenizers
uses (`encode` / `encode_batch`): a first-class, documented batch entry beside the per-item shape,
not a separate expert subsystem. pymarc itself has this shape for XML (`parse_xml_to_array`), so a
list-returning bulk reader is not foreign to pymarc users.

**What gets demoted.** `RecordBoundaryScanner`, `ProducerConsumerPipeline`,
`parse_batch_parallel(_limited)` leave the public Python surface (GAP-2 bead). They are the engine
under `MARCReader` and `read_all`, not user API.

**Parity statement.** The per-record loop keeps its exact pymarc surface and semantics (permissive
`None` yields, `current_exception`, lazy `current_chunk`) and approaches bulk throughput; the only
addition is one new function. Nothing about the fast path requires the user to know it exists.

**Definition of done.**

- Spec decisions consumed (from `bd-d7aq.9`): the bulk entry point's final name and return shape.
  `read_all` returning `list[Record]` is the placeholder used throughout this document.
- Deliverables: (1) the `next_batch` boundary and the Python-side buffer behind
  `MARCReader.__next__` (`bd-kpl9`); (2) the bulk entry point itself; (3) removal of
  `RecordBoundaryScanner`, `ProducerConsumerPipeline`, and `parse_batch_parallel(_limited)` from
  the public surface, with `mrrc/_mrrc.pyi` and the docs updated to match (`bd-d7aq.11`).
- Done when: the per-record loop preserves pymarc semantics exactly — permissive `None` yields,
  `current_exception`, lazy `current_chunk`, all in stream order (the existing reader tests pin
  these); the four primitives are gone from the public stub; and the CI benchmark pair named in
  the batch-first tenet (per-record loop vs `read_all`) exists, making the gap between the obvious
  path and the bulk path a tracked number instead of a rediscovery.
- Dependencies: `bd-d7aq.9` (name and shape). Nothing else blocks this work.

### Proposal 2 — Field access: Rust-backed handles instead of clone-and-resync

This is the hardest and most genuinely open question (`bd-d7aq.4`, design spike). Current
mechanics, verified:

- A handle is `(parent, occurrence, generation)` (`mrrc/__init__.py:129-144`). Every delegated
  attribute read, `__getitem__`, `get`, `__contains__`, and `value()` calls `_refresh`
  (`mrrc/__init__.py:275-306`), which deep-clones the field via `field_at`
  (`src-python/src/wrappers.rs:859-864`). Every write calls `_writeback` → `replace_field_at`,
  which clones the field back in (`wrappers.rs:870-882`).
- `get_fields()` additionally clones every field up front via `fields()` (`wrappers.rs:746-751`).
  The canonical loop `for f in record.get_fields(): f["a"]` therefore costs, per data field, at
  least two deep clones and three boundary crossings (`fields()` amortized + `field_at` +
  `subfields_by_code`, `mrrc/__init__.py:360-376`).
- `record.add_field(f)` clones the field's state in and leaves the Python object detached
  (`mrrc/__init__.py:1146-1152` → `wrappers.rs:713`); `add_ordered_field`/`add_grouped_field`
  reorder via O(n²) clone cycles (`mrrc/__init__.py:1255,1270`).
- Only removals bump `generation`; `replace_field_at` deliberately does not
  (`wrappers.rs:866-869`).

#### Option A — Rust-side live proxy (recommended)

The handle moves into Rust: a `#[pyclass]` holding `{record: Py<PyRecord>, tag, occurrence,
generation}` (a detached field holds an owned `Field` instead). Every operation becomes one
targeted Rust call that borrows the record and returns only the requested scalar — no field is
ever materialized to answer `f["a"]`:

```rust
// Reads borrow the record in place; nothing is cloned but the returned value.
fn subfield_first(&self, py: Python<'_>, code: char) -> PyResult<Option<String>>
fn indicator1(&self, py: Python<'_>) -> PyResult<String>
// Writes mutate in place; no refresh/writeback round-trip.
fn set_subfield_first(&mut self, py: Python<'_>, code: char, value: &str) -> PyResult<()>
fn insert_field_at(...)   // deletes the O(n²) add_ordered_field loops
```

PyO3's pyclass borrow machinery already provides the interior mutability at the binding boundary
(`Py<PyRecord>::borrow`/`borrow_mut`); the Rust core's `Record` stays plain owned data. The
generation counter survives as the staleness contract — `StaleFieldError` semantics are unchanged
and already pinned by the PAR-4 edge-case matrix (PR #301). The canonical loop becomes N+1
crossings and zero deep clones (handles are constructed from `(tag, occurrence)` pairs without
copying subfield data); today it is ~2N+1 crossings and ~2N clones.

**pymarc mutation semantics under Option A:**

- `f["a"] = "x"` on a handle: one Rust call, writes into the record in place. Observable behavior
  identical to today and to pymarc.
- `record.add_field(f)` then mutating `f`: pymarc appends the same object, so later mutation
  affects the record; mrrc today clones in and leaves `f` detached, so later mutation is silently
  lost relative to pymarc. Option A fixes this: `add_field` moves the detached field's data into
  the record and rebinds `f` as a live handle at its new position. The one divergent corner is
  adding the same Field object to two records (pymarc aliases one object into both; a handle binds
  one record) — the spike must pick a rule (rebind-to-last-add, or copy-on-second-add) and pin it
  with tests.
- Iterate-and-mutate: handles read through on every access, and removals bump the generation and
  raise `StaleFieldError` on stale handles — the contract the PAR-4 matrix already asserts.

#### Option B — shared interior mutability in the core (`Arc<RwLock<Field>>` or similar)

Makes handles trivially live, but taxes the core to pay the binding: every pure-Rust caller loses
plain `&Field` access and pays locking; `Send`/`Sync` bounds ripple through the reader/rayon
paths; serde and the pending manual `PartialEq` get awkward; and it adds a per-field allocation to
the exact hot path the 2026-06-26 profiling work just cleaned (`bd-v9cm`). Rejected: it inverts
the thin-wrapper tenet by making the Rust data model serve the Python protocol.

#### Option C — keep the generation protocol, add cheap mitigations

The named mitigations: skip the re-fetch when the cached generation matches; add Rust-side
`subfield_first`; add Rust-side positional insert. The second and third are worth doing regardless
— they are the same Rust methods Option A needs. The first is a trap: `replace_field_at` does not
bump the generation (`wrappers.rs:866-869`), so the unconditional re-fetch is what keeps two
handles to the same field coherent when one of them writes. Skipping it on generation-match makes
sibling handles serve stale cached state; bumping the generation on write instead would invalidate
every other outstanding handle on each write. Option C is acceptable only as an interim measure,
not as the 1.0 answer.

**Recommendation.** Option A, staged: land `bd-mcei` first (isolate the handle machinery in its
own module), add Option A's targeted accessors as wrapper-visible wins immediately (they shrink
`_refresh` traffic even before the switch), then swap the handle internals behind the pinned PAR-4
semantics. Most of `mrrc/__init__.py`'s `_refresh`/`_writeback` machinery deletes at the end.

**Definition of done.**

- Spec decisions consumed: the field-handle model — Option A, ratified after a short prototype
  spike of the `Py<PyRecord>` borrow pattern — and the double-add rule (rebind-to-last-add or
  copy-on-second-add), each pinned with tests.
- Deliverables: (1) `bd-mcei`'s isolation of the handle machinery into its own module; (2) the
  targeted Rust accessors sketched above (`subfield_first`, `indicator1`, `set_subfield_first`,
  `insert_field_at`); (3) the handle-internals swap to the Rust-side proxy; (4) deletion of the
  wrapper's `_refresh`/`_writeback` machinery.
- Done when: the PAR-4 edge-case matrix (PR #301) passes unchanged — `StaleFieldError` semantics
  are preserved; the canonical loop (`for f in record.get_fields(): f["a"]`) performs N+1 boundary
  crossings and zero deep clones; `add_field` leaves the passed field live at its new position,
  with the pinned double-add rule covered by tests; and the O(n²)
  `add_ordered_field`/`add_grouped_field` clone cycles are replaced by the positional insert.
- Dependencies: the PAR-4 matrix is already in place (closed, PR #301); `bd-mcei` lands first.
  This proposal's decision gates `bd-1pej` and the handle half of `bd-d7aq.7` (Proposal 4).

### Proposal 3 — Format front door: free functions routed through a completed `Format` enum

**Pick the free-function model; delete the trait surface.** Six of eight formats already ship
`record_to_<fmt>` / `<fmt>_to_record` free functions (`src/json.rs:52,117`,
`src/marcjson.rs:42,99`, `src/marcxml.rs:380,463`, `src/mods.rs:74,600`,
`src/dublin_core.rs:130,176`, `src/csv.rs:78`, `src/bibframe/mod.rs:99,130`); the
`FormatReader`/`FormatWriter`/`RecordIterator` traits cover one (`src/formats/mod.rs:68,92-95`)
and, per the 2026-06-28 comment on `bd-d7aq.3`, likely descend from discarded mrrc-experiments
exploration. Document-shaped formats (XML collections, CSV with a header row, RDF graphs) do not
fit a record-at-a-time trait without contortion, so completing the trait model means forcing it;
completing the free-function model means normalizing names and filling the enum. Normalize the
stragglers to one convention while at it: string/bytes in, string/bytes out at the front door
(`marcjson_to_record` gains the str entry point `bd-c5sh` asks about; typed intermediates like
`serde_json::Value` and `DublinCoreRecord` stay available in the per-format modules).

```rust
#[non_exhaustive]
pub enum Format { Iso2709, Json, MarcJson, MarcXml, Mods, DublinCore, Csv, Bibframe }

// Generic entry points, routed by the enum; per-format free functions remain
// the precise API underneath.
pub fn read_records(input: impl Read, format: Format) -> Result<Vec<Record>>
pub fn write_records(records: &[Record], sink: impl Write, format: Format) -> Result<()>
```

**The capability table is the normative contract** (in `formats/` rustdoc and the format docs
page), replacing absence-of-a-function as the only signal:

| Format | to `Record` | from `Record` |
|---|---|---|
| ISO 2709 | yes | yes |
| JSON | yes | yes |
| MARCJSON | yes | yes |
| MARCXML | yes | yes |
| MODS | yes | yes |
| Dublin Core | declined — lossy projection, no faithful inverse | yes |
| CSV | declined — same rationale | yes |
| BIBFRAME | yes | yes |

"Declined" is a recommendation, not a decision — GitHub #92 (implement or document) is decided
inside `bd-d7aq.3`; whichever way it goes, the table records it as contract.

**Python:** `mrrc.read()` / `mrrc.write()` accept all eight formats, inferring from the extension
via the completed enum, instead of raising `Unsupported format` for everything but `"marc"`
(`mrrc/__init__.py:2252-2256`, `2316-2325`):

```python
records = list(mrrc.read("data.xml"))     # infers MARCXML
mrrc.write(records, "out.json")           # infers JSON
```

**Parity statement.** pymarc's own format helpers (`parse_xml_to_array`, `record_to_xml`,
`parse_json_to_array`) are free functions of exactly this shape; whether to alias pymarc's names
onto the routed implementations is a one-line decision for the `bd-d7aq.9` spec. Streaming
variants (PERF-10d) and the GAP-1 marcxml preprocessing rework belong to this proposal's
implementation.

**Definition of done.**

- Spec decisions consumed: the front-door convention (string/bytes in, string/bytes out) and the
  normalized function names; the capability table's contents, including implement-or-decline for
  the Dublin Core and CSV read directions (GitHub #92); whether pymarc's helper names are aliased;
  the scope of streaming variants (PERF-10d).
- Deliverables: (1) the `Format` enum completed to all eight variants; (2) the generic
  `read_records`/`write_records` entry points routed by it; (3) deletion of the
  `FormatReader`/`FormatWriter`/`RecordIterator` trait surface; (4) the capability table published
  in `formats/` rustdoc and the format docs page; (5) Python `read()`/`write()` accepting all
  eight formats with extension inference; (6) the `marcjson_to_record` string entry point
  (`bd-c5sh`); (7) the GAP-1 rework — namespace handling inside the MARCXML event loop, removing
  the `strip_marcxml_ns` preprocessing pass (`bd-d7aq.10`).
- Done when: every format is reachable through the enum-routed entry points; the shipped functions
  match the capability table exactly, and the table is documented as the contract; Python
  `read()`/`write()` no longer raise `Unsupported format` for the seven non-ISO 2709 formats; the
  MARCXML whole-document preprocessing copies are gone.
- Dependencies: `bd-d7aq.9` (convention, table, aliasing, streaming scope). Shares no surface with
  Proposals 2 and 4, so it runs in parallel with Stages 3 and 4.

### Proposal 4 — Query surface: three imports, handles out, one declarative extract path

**Rust consolidation** (`bd-d7aq.7`): the eight modules collapse under `query/` into at most three
imports — (1) the `FieldQuery` DSL types; (2) one `RecordQueryExt` merging `FieldQueryHelpers`,
`RecordHelpers`, and the linkage accessors; (3) the per-record-type queries. Where a helper trait
applies to exactly one concrete type (`AuthorityQueries`, `BibliographicQueries`,
`HoldingsSpecificQueries`), prefer inherent methods over traits — inherent methods need no import
at all, which is the strongest form of discoverability (#225's structural fix). `field_collection`
is deleted (CLEAN-3). `bd-2snh` (multi-subfield matching) is decided inside this consolidation.

**Handle semantics for query results** (`bd-1pej`): under Proposal 2's Option A, handles are cheap
`(tag, occurrence)` bindings, so the PyO3 query methods return occurrence info and the wrapper
binds live handles. That yields the one-sentence contract with no carve-outs: *every
field-returning API returns a live handle.* If Option A were rejected, the calculus reverts and
snapshots stay documented — one more reason the field-ownership decision gates this bead.

**Declarative extract** — the pydantic-core/Polars tenet applied: the most common bulk workload
("pull a few subfields from every record in a file") should never construct Python `Record`
objects at all. One crossing per file, loop entirely in Rust, built on Proposal 1's batch engine:

```python
rows = mrrc.extract("records.mrc", ["245$a", "260$b", "001"])
# -> list of tuples, parsed and extracted inside one GIL release per batch
```

This is a candidate shape for the `bd-d7aq.9` spec, not settled API; it is listed because it is
the natural end point of the batch-first + declarative tenets, and because per-record `Record`
construction is the irreducible cost left after Proposal 1.

**Definition of done.**

- Spec decisions consumed: the query-result handle contract (the one-sentence rule above,
  contingent on Proposal 2's Option A); whether `extract()` ships in 1.0 and, if it does, its
  selector syntax.
- Deliverables: (1) the eight query/helper modules consolidated under `query/` into at most three
  imports, with inherent methods replacing single-type traits; (2) `field_collection` deleted
  (CLEAN-3); (3) `bd-2snh` (multi-subfield matching) decided and implemented inside the
  consolidation; (4) `bd-1pej` resolved to the pinned handle contract; (5) `extract()`, if the
  spec accepts it, built on Proposal 1's batch engine.
- Done when: querying a record needs at most three imports (none for inherent methods); every
  field-returning API returns a live handle, documented as the contract with no carve-outs
  (assuming Option A holds); the dead module is gone.
- Dependencies: the field-handle decision (Proposal 2) gates `bd-1pej` and the handle contract;
  `extract()` builds on Proposal 1's engine. Sequenced after Stage 4 in the build sequence.

### Proposal 5 — Crate and wrapper surface: prelude, tiers, thin skin

**Rust** (`bd-d7aq.6` + `bd-d7aq.8`): modules regroup into `formats/`, `records/`, `io/`,
`query/` (re-export-only churn); `lib.rs` stops re-exporting ~30 types flat
(`src/lib.rs:160-189`) and instead re-exports the prelude tier only, everything else reachable
through the module tree:

```rust
pub mod prelude {
    pub use crate::{Record, Field, Subfield, Leader, MarcReader, MarcWriter,
                    MarcError, Result, RecoveryMode, query::RecordQueryExt};
}
```

`iso2709_skeleton` goes `pub(crate)` at the 1.0 boundary (ARCH-9); the slice/shared parse entry
points (`src/lib.rs:182`) stay public for Rust consumers — Rust users are the intended audience
for primitives — but live under `io::`, out of the prelude.

**Python** (`bd-mcei` + the proposals above): `mrrc/__init__.py` (2200+ lines) splits into
pymarc-shaped modules (`record.py`, `field.py`, `leader.py`, `reader.py`, `writer.py`, mirroring
pymarc's layout, per the bead). The wrapper becomes a thin skin: Proposal 2 deletes the
resync/writeback machinery, Proposal 1 removes the per-record plumbing, and the public namespace
exports pymarc-shaped names only — the performance primitives that remain (the batch engine, the
extract internals) are not re-exported (GAP-2 bead).

**Definition of done.**

- Spec decisions consumed: the prelude's membership (the draft above is the starting point); the
  module grouping boundaries; which parse entry points remain public under `io::`.
- Deliverables: (1) the Rust module regroup into `formats/`, `records/`, `io/`, `query/` —
  re-export-only churn (`bd-d7aq.6`); (2) the prelude plus tiered re-exports, with
  `iso2709_skeleton` made `pub(crate)` (`bd-d7aq.8`); (3) the `mrrc/__init__.py` split into
  pymarc-shaped modules (`bd-mcei`); (4) a public Python namespace exporting pymarc-shaped names
  only, with the batch/extract internals unexported.
- Done when: `lib.rs` no longer re-exports ~30 types flat; the parse skeleton is out of the public
  API; the wrapper modules mirror pymarc's layout; and no performance primitive appears in the
  public Python namespace or `mrrc/_mrrc.pyi`.
- Dependencies: last in sequence — the surfaces produced by Proposals 1-4 must settle first.
  `bd-xjws` (inline `Tag`) rides this window because it is API-rippling and only gets cheaper
  before 1.0.

### Cross-cutting tenets

Each tenet names the mrrc mechanism that enforces it; the precedents are the ones `bd-d7aq.9`
cross-checked (tokenizers, pydantic-core, Polars).

| Tenet | Mechanism in mrrc | Precedent |
|---|---|---|
| **Batch-first**: hot loops cross the boundary per batch, never per record | `BatchedReader` is the only iteration engine; `MARCReader` and `read_all` are two skins over it; a CI benchmark pair (per-record loop vs `read_all`) tracks the ratio so the obvious path regressing against the bulk path is a visible event | tokenizers: `encode_batch` is the recommended, first-class shape for multi-item work |
| **Declarative-into-Rust**: accept the whole request, loop in Rust | `get_fields(*tags)` is one call; the query DSL evaluates in Rust; `extract()` takes a spec and returns columns; the wrapper never loops per field for a built-in operation | pydantic-core: Python declares the model, Rust does all per-element work |
| **Name the slow path**: per-element escape hatches are labeled, not hidden | A fast-path/slow-path table in the performance guide; docstrings on remaining per-element entry points say so and point at the batch equivalent | Polars: the idiomatic expression API runs in Rust; `map_elements` is explicitly documented as the named slow path |
| **Thin wrapper**: Python defines shape, Rust defines behavior | Wrapper methods delegate to one Rust call; the Rust-vs-Python parity tests (`bd-d7aq.2`'s pattern) plus the pymarc oracle in CI enforce that reimplementations cannot silently diverge (PAR-1 is the standing counterexample) | all three: none reimplements logic in the Python layer |

### Decisions the `bd-d7aq.9` spec must pin

Already settled by this document's recommendations — the spec ratifies these, or overturns them
with recorded rationale:

- The organizing principle: pymarc shape everywhere; performance primitives stay internal.
- Proposal 1's mechanism: batch internally behind the unchanged per-record surface; one bulk entry
  point; the four public parallel primitives demoted.
- Proposal 2's model: Option A (Rust-side live proxy). Option B is rejected — it inverts the
  thin-wrapper tenet by taxing the Rust core; Option C is acceptable only as an interim measure.
- Proposal 3's model: free functions routed through a completed `Format` enum; the trait surface
  is deleted.
- The build order below.
- Recorded non-decisions that stay closed: PERF-10c (read-into-uninit) is won't-fix per `bd-4d8u`.

Still open — the spec must decide each of these:

| # | Decision | Proposal | Notes |
|---|---|---|---|
| 1 | Bulk entry point: final name and return shape | 1 | `read_all` returning `list[Record]` is the placeholder; precedents are tokenizers' `encode_batch` and pymarc's `parse_xml_to_array` |
| 2 | Ratify Option A after the prototype spike of the `Py<PyRecord>` borrow pattern | 2 | the spike is the spec's one prerequisite experiment |
| 3 | The double-add rule: rebind-to-last-add or copy-on-second-add when one `Field` is added to two records | 2 | pymarc aliases one object into both; a handle can bind only one |
| 4 | Format front-door convention (string/bytes in and out) and the normalized function names | 3 | the `marcjson_to_record` string entry (`bd-c5sh`) is one instance |
| 5 | Capability table contents: implement or decline the Dublin Core and CSV read directions | 3 | GitHub #92; "declined" in Proposal 3's table is a recommendation, not a decision |
| 6 | Whether to alias pymarc's format-helper names (`parse_xml_to_array`, `record_to_xml`, …) | 3 | a one-line decision either way |
| 7 | Scope of streaming variants (PERF-10d) inside the front door | 3 | GAP-1 (`bd-d7aq.10`) folds into whichever path is chosen |
| 8 | Query-result handle contract: "every field-returning API returns a live handle" | 4 | contingent on decision 2; if Option A fell, snapshots stay documented instead |
| 9 | Whether declarative `extract()` ships in 1.0, and its selector syntax | 4 | candidate shape only, not settled API |
| 10 | Prelude membership | 5 | the draft list in Proposal 5 is the starting point |

`bd-2snh` (multi-subfield matching) is deliberately absent from this table: it is decided inside
`bd-d7aq.7`'s implementation, not by the spec.

### Build sequence

Ordered by what gates what; aligned with the existing child priorities. The one hard rule: **no
public-surface change lands before the `bd-d7aq.9` spec exists** (Stage 2). Stage 1 is exempt
because it is internal-consistency work whose outcome no Part B decision changes.

**Stage 1 — internal consistency; start now.**

- Work: `bd-d7aq.1` (writer consolidation; unblocked since `bd-pk26.3` closed) and `bd-d7aq.2`
  (`format_field` parity plus the Rust-vs-wrapper parity tests).
- Depends on: nothing; runs in parallel with Stage 2.
- Unblocks: `bd-cdey` (MARC-8 write — wire it once into the consolidated writer, not three times).
- Done when: one shared writer path serves all three record types with the bibliographic writer's
  checked-cast posture; the record types carry the manual `PartialEq` (ignoring `errors`) that
  rides in `bd-d7aq.1`; and `format_field` agrees between Rust, Python, and pymarc under test.

**Stage 2 — the spec (`bd-d7aq.9`).**

- Work: write the spec, taking this document as input; run the Proposal 2 prototype spike; pin
  every row of the decision table above.
- Depends on: nothing.
- Unblocks: every stage below.
- Done when: each open decision has a pinned answer with rationale, and the implementation beads
  reference the spec.

**Stage 3 — read pit-of-success (Proposal 1).**

- Work: `bd-kpl9` (batched iteration behind `MARCReader`), then `bd-d7aq.11` (the bulk entry
  point; demote the four public primitives).
- Depends on: Stage 2 (decision 1).
- Unblocks: the `extract()` engine (Proposal 4) and Proposal 5's not-re-exported end state.
- Done when: Proposal 1's definition of done holds. Largest single user-visible payoff; a
  tractable medium-sized chunk per `bd-d7aq.9`.

**Stage 4 — field ownership (Proposal 2).**

- Work: `bd-mcei` (isolate the handle machinery), then the `bd-d7aq.4` spike-informed
  implementation.
- Depends on: Stage 2 (decisions 2 and 3). The PAR-4 semantics matrix it must preserve is already
  in place (PR #301).
- Unblocks: `bd-1pej` and the handle half of `bd-d7aq.7` (Stage 6); the wrapper deletions in
  Proposal 5.
- Done when: Proposal 2's definition of done holds.

**Stage 5 — format front door (Proposal 3).**

- Work: `bd-d7aq.3`, absorbing `bd-c5sh`, GitHub #92, PERF-10d, and GAP-1 (`bd-d7aq.10`).
- Depends on: Stage 2 (decisions 4-7). Shares no surface with Stages 3-4, so it runs in parallel
  with them.
- Done when: Proposal 3's definition of done holds.

**Stage 6 — query consolidation (Proposal 4).**

- Work: `bd-d7aq.7`, with `bd-2snh` and `bd-1pej` decided inside it.
- Depends on: the field-handle outcome (Stage 4), because query results return the new handles.
- Done when: Proposal 4's definition of done holds.

**Stage 7 — surface finish (Proposal 5).**

- Work: `bd-d7aq.6` (module regroup), then `bd-d7aq.8` (prelude, `pub(crate)` skeleton);
  `bd-xjws` (inline `Tag`) rides this window.
- Depends on: Stages 3-6 — it freezes the surfaces they produce, so it goes last.
- Done when: Proposal 5's definition of done holds.

**Unstaged:** `bd-d7aq.5` (reader option parity) is independent and can land any time.

> **Open concern (for human review):** `bd-d7aq.5` adds public builder options to the authority
> and holdings readers, so "can land any time" sits uneasily beside the no-public-surface-change
> rule above. Both statements come from the epic's existing sequencing; the `bd-d7aq.9` spec
> should state explicitly whether this parity work is exempt (as additive mirroring of an existing
> surface) or waits for Stage 2 like the rest.

Dependency summary:

| Stage | Blocked by | Can run in parallel with |
|---|---|---|
| 1 — internal consistency | nothing | 2 |
| 2 — spec | nothing | 1 |
| 3 — bulk read | 2 | 4, 5 |
| 4 — field ownership | 2 | 3, 5 |
| 5 — format front door | 2 | 3, 4 |
| 6 — query consolidation | 4 | 5, if 5 is still in flight |
| 7 — surface finish | 3, 4, 5, 6 | nothing — it goes last |
