# API Coherence Proposals — 1.0

Companion to [`code-review-findings-v0.8.2.md`](code-review-findings-v0.8.2.md), written 2026-07-01
against `main` @ `4939465`. That document is the review record; this one reconciles its findings
against the bead tracker as it stands after the 2026-06-25 epic restructure (Part A), and proposes
the concrete API shapes for the 1.0 coherence work (Part B). It is input to `bd-d7aq.9`, the gating
design child of the `bd-d7aq` epic, whose deliverable is the spec the implementation children align
to. Every codebase claim below was re-verified against the current tree; where something could not
be verified it is marked as such.

The organizing principle, quoted from `bd-d7aq.9`: **the obvious, pymarc-shaped path must be the
fastest path; performance primitives stay internal.** pymarc compatibility is the primary design
axis for the Python wrapper. Backward compatibility with mrrc's own 0.8.x surface is a non-goal;
breaking the current mrrc shape is acceptable, breaking pymarc-shape parity is not.

---

## Part A: Ticketing reconciliation

The findings doc's remediation section was last updated 2026-06-13; the `bd-d7aq` epic was
restructured 2026-06-25 and the `bd-v9cm` profiling work closed 2026-06-28. This section re-derives
coverage from the current bead state and code rather than trusting that section. Two genuine gaps
were found and filed as `bd-d7aq.10` (GAP-1) and `bd-d7aq.11` (GAP-2), children of the `bd-d7aq`
epic; their descriptions are reproduced at the end of this part.

### Theme 1 and Theme 3: the epic's findings

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

### Theme 2: performance findings

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

### Themes 4-7 and cleanup

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

### Cross-cutting tenets

Each tenet names the mrrc mechanism that enforces it; the precedents are the ones `bd-d7aq.9`
cross-checked (tokenizers, pydantic-core, Polars).

| Tenet | Mechanism in mrrc | Precedent |
|---|---|---|
| **Batch-first**: hot loops cross the boundary per batch, never per record | `BatchedReader` is the only iteration engine; `MARCReader` and `read_all` are two skins over it; a CI benchmark pair (per-record loop vs `read_all`) tracks the ratio so the obvious path regressing against the bulk path is a visible event | tokenizers: `encode_batch` is the recommended, first-class shape for multi-item work |
| **Declarative-into-Rust**: accept the whole request, loop in Rust | `get_fields(*tags)` is one call; the query DSL evaluates in Rust; `extract()` takes a spec and returns columns; the wrapper never loops per field for a built-in operation | pydantic-core: Python declares the model, Rust does all per-element work |
| **Name the slow path**: per-element escape hatches are labeled, not hidden | A fast-path/slow-path table in the performance guide; docstrings on remaining per-element entry points say so and point at the batch equivalent | Polars: the idiomatic expression API runs in Rust; `map_elements` is explicitly documented as the named slow path |
| **Thin wrapper**: Python defines shape, Rust defines behavior | Wrapper methods delegate to one Rust call; the Rust-vs-Python parity tests (`bd-d7aq.2`'s pattern) plus the pymarc oracle in CI enforce that reimplementations cannot silently diverge (PAR-1 is the standing counterexample) | all three: none reimplements logic in the Python layer |

### Build sequence

Ordered by what gates what; aligned with the existing child priorities.

1. **Now, independent of the spec** — `bd-d7aq.1` (writer consolidation; unblocked since
   `bd-pk26.3` closed, and it unblocks `bd-cdey`) and `bd-d7aq.2` (`format_field` parity). Both
   are internal-consistency work whose outcome no Part B decision changes.
2. **The spec** — `bd-d7aq.9`, taking this document as input. Decisions to pin: the bulk entry
   point's name and return shape (Proposal 1), the field-handle model (Proposal 2 — Option A
   recommended, needs a short prototype spike for the `Py<PyRecord>` borrow pattern and the
   double-add rule), the format convention and capability table (Proposal 3), the handle contract
   for query results (Proposal 4), the prelude tier (Proposal 5). No other public-surface change
   should land before this exists.
3. **Read pit-of-success** — `bd-kpl9` (internal batched iteration) plus the GAP-2 consolidation
   bead. Largest single user-visible payoff; tractable medium chunk per `bd-d7aq.9`.
4. **Field ownership** — `bd-mcei` (isolate the handle machinery), then the `bd-d7aq.4` spike and
   implementation. Gates `bd-1pej` and the handle half of `bd-d7aq.7`.
5. **Format front door** — `bd-d7aq.3`, absorbing `bd-c5sh`, GitHub #92, PERF-10d, and GAP-1.
   Independent of 3-4; can run in parallel with them.
6. **Query consolidation** — `bd-d7aq.7` with `bd-2snh` and `bd-1pej` decided inside it; after the
   field-handle decision.
7. **Surface finish** — `bd-d7aq.6` (regroup) then `bd-d7aq.8` (prelude, `pub(crate)` skeleton),
   last, once the surfaces above have settled; `bd-xjws` (inline `Tag`) rides this window because
   it is API-rippling and only gets cheaper before 1.0; `bd-d7aq.5` (reader option parity) is
   independent and can land any time.
