# Formal methods: property tests and fuzzing

This page explains the lightweight formal-methods toolkit mrrc uses to
catch the kinds of bugs unit tests are bad at finding. It is pitched
at a reader who has written assert-style tests before but is new to
property-based testing.

!!! note "Scope: lightweight, not full verification"

    "Formal methods" covers a wide spectrum, from informal
    specifications through property-based testing all the way to
    mechanically-checked proofs in tools like [TLA+], [Coq], or
    [Lean]. **mrrc operates at the lightweight end of that
    spectrum.** We use property-based testing (proptest) and
    coverage-guided fuzzing (cargo-fuzz) to falsify a finite set of
    invariants on a large but bounded sample of inputs. We do **not**
    prove the codec correct, run a model checker, or maintain a
    formal specification in TLA+ / Coq / Lean. The techniques on
    this page are modest by formal-methods standards — they catch
    real bugs cheaply, but they are sampling, not proof. See
    [What this page is *not*](#what-this-page-is-not) below for the
    fuller picture of where this sits in the broader spectrum.

By the end you should know:

1. What property-based testing is and why it complements unit tests.
2. Which invariants mrrc actually asserts and where to find them.
3. How accepted regression seeds become permanent guards.
4. Where coverage-guided fuzzing fits in alongside property tests.

The runnable suite lives in `tests/properties.rs`; the fuzz
infrastructure is documented separately in [Fuzzing](fuzzing.md).

## Property-based testing in five minutes

A unit test asserts a single concrete fact:

```rust
#[test]
fn parses_simple_record() {
    let bytes = include_bytes!("simple.mrc");
    let record = MarcReader::new(Cursor::new(&bytes[..])).read_record().unwrap().unwrap();
    assert_eq!(record.get_control_field("001"), Some("12345"));
}
```

This catches *that one bug*. It says nothing about the next record, or
the same record with a different leader byte, or a record with
subfield values containing tab characters.

A *property* asserts a fact that should hold for **every** input drawn
from some space:

```rust
proptest! {
    #[test]
    fn binary_roundtrip(record in arb_record()) {
        let bytes = serialize(&record);
        let parsed = MarcReader::new(Cursor::new(&bytes[..]))
            .read_record().unwrap().unwrap();
        prop_assert_eq!(record, parsed);
    }
}
```

The framework — [proptest][proptest-book] in mrrc's case — generates
many concrete `record` values from `arb_record()` (a *strategy*) and
runs the assertion against each one. If any input violates the
property, you get a bug.

The interesting parts:

- **Random generation, but structured.** `arb_record()` doesn't
  produce arbitrary bytes; it produces structurally valid `Record`
  values via composable strategies (a leader, then control fields,
  then data fields, with valid tags and indicator characters). The
  test then checks invariants you can't get from raw fuzzing.
- **Shrinking.** When proptest finds a failing input it doesn't just
  hand you the random 200-field record that crashed — it
  *automatically simplifies* it, halving subfield counts, dropping
  fields, replacing strings with shorter ones, until it has the
  smallest input that still triggers the bug. You typically end up
  with a 1-field, 1-subfield, 1-character reproducer.
- **Counter-example seeds.** The shrunk failing input gets saved as a
  hex-encoded seed in `tests/proptest-regressions/properties.txt` and
  re-runs on every test invocation forever after. A regression that
  reintroduces the bug fails immediately on the saved seed, before
  any new random cases run. See [Regression seeds](#regression-seeds)
  below.

The mental model that helps the most: properties are **specifications
written in code**. "Round-tripping a record through the writer and
reader produces an equal record" is a specification of mrrc's binary
codec. Proptest tries to falsify it.

## Properties mrrc enforces

`tests/properties.rs` runs eight properties on every `cargo test`
invocation. The full source is the canonical reference; the table
below is a one-line summary of each.

| Property | What it asserts |
|----------|-----------------|
| `binary_roundtrip` | A record serialized to ISO 2709 and parsed back compares equal to the original — leader fields, control fields, data fields, indicators, subfield codes, and subfield values all preserved. |
| `serialization_never_panics` | `MarcWriter::write_record` returns `Ok` and emits non-empty output for every generated record. |
| `leader_length_matches_emitted_bytes` | The `record_length` field the writer puts in the leader equals the total byte count of the serialized record. |
| `directory_entries_tile_data_area` | Directory entries start immediately after one another with no gaps or overlaps; field lengths sum (plus one for the record terminator) to the data-area size; every entry ends in `FIELD_TERMINATOR`; the record ends in `RECORD_TERMINATOR`. |
| `indicator_bytes_in_valid_set` | Every indicator byte in every emitted data field is either a digit (`0`–`9`) or an ASCII space. |
| `subfield_codes_are_lower_alnum` | Every byte immediately following a `SUBFIELD_DELIMITER` (0x1F) is a lowercase ASCII letter or digit. |
| `marcxml_roundtrip` | A record serialized to MARCXML and parsed back compares equal — including subfield values containing XML metacharacters (`< > & " '`) and arbitrary whitespace. |
| `marcjson_roundtrip` | A record serialized to MARCJSON and parsed back compares equal — including subfield values containing JSON-special characters (`\t \n \r \\ "`). |

The four structural-invariant properties (`leader_length_matches_emitted_bytes`,
`directory_entries_tile_data_area`, `indicator_bytes_in_valid_set`,
`subfield_codes_are_lower_alnum`) inspect the emitted ISO 2709 bytes
directly rather than checking round-trip equality. They guard against
a writer bug that would silently produce malformed-but-self-consistent
output: a writer that drops a field could pass `binary_roundtrip` if
the corresponding parser also dropped it, but the directory-tiling
property would fail because the emitted bytes wouldn't match the
declared structure.

The two non-binary round-trip properties (`marcxml_roundtrip`,
`marcjson_roundtrip`) deliberately exercise format-specific escaping,
which is where text-format codecs typically have bugs — XML entity
references and CDATA sections, JSON backslash escapes, namespace
handling, whitespace treatment.

### Configuration and runtime

`ProptestConfig::cases = 64` keeps the full suite under ten seconds
locally (about one second on an Apple-silicon laptop). Override for a
deeper one-off run:

```bash
PROPTEST_CASES=2000 cargo test --test properties
```

The CI matrix runs the suite at the default 64 cases on every PR.
Saved regression seeds run unconditionally regardless of `cases`.

## Regression seeds

When proptest finds a failing input it writes the shrunk seed to
`tests/proptest-regressions/properties.txt`. The format is
hex-encoded RNG state plus a comment explaining what the seed
shrinks to:

```
# cc <hex-encoded shrunk input>  # <one-line explanation>
cc c55bf86fe34e98890f3851b08dc2838087458f63372964533b0190c7d491c89e # whitespace-only control value — MARCXML reader must preserve whitespace rather than error with "missing field `$value`"
```

The policy is straightforward:

- **Commit accepted seeds.** They become permanent guards. A future
  refactor that reintroduces the bug fails on the seed before any
  random case is generated.
- **Annotate each seed.** A bare hex string ages into mystery; one
  short comment about what shape the input takes saves a future
  reader (or future you) from re-shrinking when triaging.
- **Don't commit `.pending` files.** Proptest writes those during
  shrinking; they are gitignored.

Acceptance happens in the same PR as the fix: the regression test
must fail on the seed before the fix, then pass after.

## How fuzzing complements property tests

Coverage-guided fuzzing — see [Fuzzing](fuzzing.md) for the full
infrastructure — points at the same problem from the opposite end:

| Dimension | Property tests | Coverage-guided fuzzing |
|-----------|----------------|--------------------------|
| Input generation | Structured strategies (`arb_record()` returns valid `Record`) | Mutated bytes guided by code coverage |
| What it asserts | Invariants you write (round-trip, tiling, character sets) | The harness must not panic / OOM / hang |
| Where bugs surface | Logic bugs in the codec layer | Reader-side input-validation bugs in raw byte streams |
| Where it runs | Every PR via `cargo test` | Nightly via `.github/workflows/fuzz.yml` |
| Triage artifact | Seed in `tests/proptest-regressions/properties.txt` | Reproducer in `tests/data/fuzz-regressions/<target>/` |

In other words: property tests prove things about *records you can
build*; fuzzing finds bugs in the *code paths that turn arbitrary
bytes into records*. Both feed regression tests that run on every
PR. Neither replaces the other.

## What this page is *not*

The phrase "formal methods" covers a wide spectrum, from informal
specifications all the way to mechanically-checked proofs in tools
like [TLA+], [Coq], or [Lean]. mrrc operates at the **lightweight
end** of that spectrum — property-based testing and coverage-guided
fuzzing — and does not currently use full formal verification. We do
not prove the codec correct; we falsify a finite set of properties on
a large but bounded sample of inputs.

The companion [mrrc-testbed][testbed] repo's
[formal-methods-verification-strategy.md][testbed-strategy]
positions the techniques mrrc uses as levels 3 and 4 of a 5-level
verification pyramid:

1. Type system (Rust's borrow checker — the always-on baseline)
2. Unit tests (concrete examples, in `src/**/tests` and `tests/`)
3. Property tests (`tests/properties.rs` — this page)
4. Coverage-guided fuzzing ([Fuzzing](fuzzing.md))
5. Bounded model checking ([Kani] over small input spaces — future)

Levels 1 and 2 catch common mistakes; levels 3 and 4 catch the long
tail; level 5 (not yet adopted in mrrc) would prove specific
properties exhaustively for inputs up to a fixed size.

## Further reading

The most approachable starting points if you want to learn more:

- [proptest book][proptest-book] — the framework mrrc uses, with
  a worked introduction to strategies and shrinking.
- [Hypothesis docs][hypothesis] — Python's property-testing library
  is the most thoroughly documented in the wider ecosystem; the
  conceptual material transfers cleanly to proptest.
- [QuickCheck: A Lightweight Tool for Random Testing of Haskell
  Programs (Claessen & Hughes, 2000)][quickcheck-paper] — the
  original paper that introduced property-based testing. Short
  (~12 pages), readable, and explains the core ideas in their
  cleanest form.
- [In Praise of Property-Based Testing (Hughes, 2020)][hughes-praise]
  — a more recent retrospective on what property-based testing
  caught in industrial use, including the famous Volvo ECU bug
  shrunk to a 4-line reproducer.

[proptest-book]: https://proptest-rs.github.io/proptest/
[hypothesis]: https://hypothesis.readthedocs.io/
[quickcheck-paper]: https://www.cs.tufts.edu/~nr/cs257/archive/john-hughes/quick.pdf
[hughes-praise]: https://increment.com/testing/in-praise-of-property-based-testing/
[TLA+]: https://lamport.azurewebsites.net/tla/tla.html
[Coq]: https://coq.inria.fr/
[Lean]: https://leanprover.github.io/
[Kani]: https://model-checking.github.io/kani/
[testbed]: https://github.com/dchud/mrrc-testbed
[testbed-strategy]: https://github.com/dchud/mrrc-testbed/blob/main/formal-methods-verification-strategy.md

## Suite history

The current proptest suite was established in
[#91](https://github.com/dchud/mrrc/issues/91) (bd-ouji, "Enable
proptest with round-trip and structural invariant properties"), which
landed the binary round-trip, the four ISO 2709 structural
invariants, and the MARCXML / MARCJSON round-trips. Before that the
project had only unit tests; the move to properties was driven by
the realization that the codec's correctness is naturally specified
as round-trip equality, which a unit test can only sample.
