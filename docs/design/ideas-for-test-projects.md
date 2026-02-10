# Ideas for Test Projects

Testbed design for verifying mrrc functionality. This document is intended for handoff to a project manager to create an implementation plan.

## Overview

A single monorepo (`mrrc-testbed`) containing test suites that exercise mrrc capabilities at scale with real-world data. The testbed supports two modes:

- **CI mode**: Uses small, committed fixture files for fast, reliable automated testing
- **Local mode**: Uses large downloaded datasets for thorough manual validation

### Scope: Real-World Data and Scale Testing

**The testbed focuses exclusively on:**
1. **Real-world data** — Testing against actual MARC records from LOC, Internet Archive, and other sources to discover edge cases that synthetic fixtures miss
2. **Scale testing** — Running against millions of records to surface memory leaks, performance regressions, and concurrency issues invisible at small scale

**The testbed does NOT duplicate:**
- Unit tests for API compatibility (covered by mrrc's `test_pymarc_compatibility.py`)
- Format round-trip correctness (covered by mrrc's `test_format_fidelity.py`)
- Query DSL correctness (covered by mrrc's `test_query_dsl.py`)
- Basic concurrency/GIL tests (covered by mrrc's parallel benchmarks)

The mrrc project already has comprehensive test coverage (~21 test files, 177+ test functions). The testbed extends this by throwing real-world data at mrrc to find bugs that curated fixtures don't expose.

### Testing Layers

The testbed tests mrrc at two levels:

1. **Rust core** (primary focus) — Direct testing of the Rust library using `cargo test` with real-world data, stress tests, and property-based testing
2. **Python bindings** (compatibility focus) — Verifying the Python wrapper works correctly, particularly pymarc API compatibility with latest pymarc release

Rust-level testing is the primary focus because:
- Performance-critical code lives in Rust
- Memory safety and concurrency bugs surface at the Rust level
- Rust tests run faster and can use more aggressive fuzzing

Python testing focuses on wrapper correctness and pymarc compatibility, not re-testing Rust logic through Python.

### Interaction Models

The testbed supports two distinct usage patterns:

**1. Centralized Testbed (mrrc-testbed repository)**

A single canonical repository that accumulates discoveries over time:
- Maintainer runs periodic large-scale tests against LOC, IA, and other public datasets
- Discoveries are committed to the repo (YAML files)
- Fixtures grow as edge cases are discovered and fixed
- Anyone can clone and run verification tests
- Community can submit discovery PRs (single YAML file + record)

**2. Local/Private Testing (fork or standalone)**

Users can run the testbed privately against their own data without sharing:
- Fork the repo or use it standalone
- Configure BYOD paths in `.env`
- Run tests repeatedly over time
- Keep discoveries local (gitignored `results/` directory)
- No obligation to contribute back

Both models use the same tools and workflows — the difference is whether discoveries are committed and shared.

---

## Repository Structure

```
mrrc-testbed/
├── .beads/                     # Beads issue tracking
├── .env.example                # Template for local configuration
├── .gitignore                  # Excludes data/downloads/, .env, state/index.db, etc.
├── Cargo.toml                  # Rust workspace configuration
├── pyproject.toml              # uv-managed Python project
├── uv.lock                     # Locked dependencies
├── README.md                   # Setup and usage instructions
├── mkdocs.yml                  # MkDocs configuration
│
├── data/
│   ├── README.md               # Data sources, licenses, download instructions
│   ├── downloads/              # .gitignored - large datasets go here
│   ├── custom/                 # .gitignored - user's own datasets (BYOD)
│   ├── fixtures/               # Committed - small curated samples (~10MB total)
│   │   ├── bibliographic/      # Sample bibliographic records
│   │   ├── authority/          # Sample authority records
│   │   ├── holdings/           # Sample holdings records
│   │   └── edge_cases/         # Known problematic records
│   └── synthetic/              # Committed - generated test records
│       ├── README.md           # Documents how each was generated
│       ├── malformed/          # Intentionally broken records
│       ├── encoding/           # Encoding test vectors
│       └── generators/         # Scripts that created synthetic data
│
├── state/                      # Cross-run state tracking
│   ├── schema.sql              # SQLite schema (committed)
│   ├── index.db                # SQLite index (.gitignored, rebuilt from YAML)
│   ├── discoveries/            # Discovery YAML files (committed)
│   │   └── *.yaml
│   ├── runs/                   # Run history YAML files (committed)
│   │   └── *.yaml
│   └── records/                # Extracted problematic records (committed)
│       └── *.mrc
│
├── docs/                       # MkDocs documentation source
│   ├── index.md                # Introduction
│   ├── getting-started/        # Installation, first run
│   ├── tutorials/              # Step-by-step guides
│   ├── guides/                 # How-to guides (contributing, etc.)
│   ├── reference/              # Format specs, CLI reference
│   ├── explanation/            # Concepts (scope, state management)
│   └── changelog.md
│
├── crates/
│   └── mrrc_testbed/           # Rust test harness crate
│       ├── Cargo.toml
│       ├── src/
│       │   ├── lib.rs          # Test utilities and dataset loading
│       │   ├── config.rs       # Configuration from .env
│       │   ├── datasets.rs     # Dataset abstraction (CI/local/custom)
│       │   └── discovery.rs    # DiscoveryWriter for recording findings
│       └── tests/
│           ├── stress.rs       # Memory, throughput, scaling tests
│           ├── malformed.rs    # Error recovery with real bad data
│           ├── encoding.rs     # MARC-8/UTF-8 with international records
│           ├── concurrent.rs   # Thread safety under sustained load
│           └── discovery.rs    # Edge case discovery in real datasets
│
├── src/
│   └── mrrc_testbed/           # Python package
│       ├── __init__.py
│       ├── config.py           # Configuration loading (.env, defaults)
│       ├── datasets.py         # Dataset loading with CI/local/custom switching
│       ├── download.py         # On-demand dataset fetching
│       ├── compare.py          # Deep record comparison utilities
│       ├── state.py            # State management (YAML + SQLite)
│       └── report.py           # Unified report generation
│
├── suites/                     # Python test suites (focused on wrapper/compat)
│   ├── conftest.py             # Shared pytest fixtures
│   ├── pymarc_compat/          # pymarc API compatibility at scale
│   ├── encoding/               # Encoding through Python bindings
│   └── discovery/              # Edge case discovery via Python
│
├── scripts/
│   ├── download_datasets.py    # Fetch all/specific datasets
│   ├── generate_report.py      # Generate unified HTML/JSON report
│   ├── validate_fixtures.py    # Verify fixtures valid + manifest in sync
│   ├── curate_fixtures.py      # Initial fixture selection from LOC
│   ├── extract_record.py       # Extract record at byte offset from large file
│   ├── file_issue.py           # File mrrc issue from discovery
│   ├── promote_discovery.py    # Promote discovery to fixture
│   ├── import_run.py           # Import run results, update state
│   ├── rebuild_index.py        # Rebuild SQLite from YAML
│   ├── query.py                # Query discoveries via SQL
│   ├── export_discovery.py     # Export discovery for PR submission
│   └── archive_runs.py         # Archive/prune old run data
│
├── results/                    # .gitignored - local test results
│   └── .gitkeep
│
└── .github/
    └── workflows/
        └── ci.yml              # CI workflow (fixtures only)
```

---

## Data Management Strategy

### Principle: Never commit downloaded public data

Large public datasets (LOC, Internet Archive, etc.) are **never** committed to git. Instead:

1. **Configuration points to local copies** via `.env` file
2. **Download scripts** fetch data on demand to `data/downloads/`
3. **CI uses committed fixtures** only - small, curated samples

### Four categories of test data

| Category | Location | In Git? | Purpose |
|----------|----------|---------|---------|
| **Downloaded** | `data/downloads/` | No | Large public datasets for thorough local testing |
| **Custom (BYOD)** | `data/custom/` | No | User's own MARC files for testing |
| **Fixtures** | `data/fixtures/` | Yes | Small curated samples for CI and quick tests |
| **Synthetic** | `data/synthetic/` | Yes | Generated records for specific test scenarios |

### Bring Your Own Dataset (BYOD)

Users can test mrrc against their own MARC data:

```bash
# Place your MARC files in the custom directory
cp /path/to/my_library.mrc data/custom/

# Or configure paths in .env
echo "MRRC_CUSTOM_DATASET=/path/to/my_library.mrc" >> .env

# Run tests against custom data
MRRC_TEST_MODE=custom uv run pytest suites/
cargo test --features custom-data
```

**Custom dataset configuration:**

```bash
# .env
# Point to individual custom files
MRRC_CUSTOM_DATASET=/path/to/my_records.mrc
MRRC_CUSTOM_AUTHORITY=/path/to/my_authorities.mrc

# Or point to a directory containing multiple .mrc files
MRRC_CUSTOM_DIR=/path/to/my_marc_collection/

# Custom dataset metadata (optional, for reporting)
MRRC_CUSTOM_NAME="My Library Catalog"
MRRC_CUSTOM_RECORD_COUNT=500000
```

The dataset abstraction layer automatically handles custom datasets:

```python
# src/mrrc_testbed/datasets.py

def get_dataset(name: str = "default"):
    """
    Returns path to dataset based on mode and availability.

    Priority order:
    1. Custom dataset (if MRRC_TEST_MODE=custom and configured)
    2. Downloaded dataset (if MRRC_TEST_MODE=local and available)
    3. Fixture dataset (always available, used in CI)
    """
    mode = get_test_mode()

    if mode == "custom":
        custom_path = get_custom_dataset_path(name)
        if custom_path and custom_path.exists():
            return custom_path
        raise DatasetNotFound(f"Custom dataset '{name}' not configured")

    if mode == "local":
        download_path = get_download_path(name)
        if download_path and download_path.exists():
            return download_path

    # Fall back to fixture
    return FIXTURES_DIR / name / "sample.mrc"
```

```rust
// crates/mrrc_testbed/src/datasets.rs

pub fn get_dataset(name: &str) -> Result<PathBuf, DatasetError> {
    let mode = TestMode::from_env();

    match mode {
        TestMode::Custom => {
            get_custom_dataset(name)
                .ok_or_else(|| DatasetError::NotConfigured(name.to_string()))
        }
        TestMode::Local => {
            get_download_path(name)
                .or_else(|| get_fixture_path(name))
                .ok_or_else(|| DatasetError::NotFound(name.to_string()))
        }
        TestMode::Ci => {
            get_fixture_path(name)
                .ok_or_else(|| DatasetError::NotFound(name.to_string()))
        }
    }
}
```

### Configuration via `.env`

```bash
# .env.example (committed)
# Copy to .env and customize (not committed)

# Test mode: "ci" (fixtures), "local" (downloads), "custom" (your data)
MRRC_TEST_MODE=local

# Dataset locations - absolute paths to downloaded data
MRRC_LOC_BOOKS=/path/to/loc_books_all.mrc
MRRC_LOC_NAMES=/path/to/loc_names.mrc
MRRC_LOC_SUBJECTS=/path/to/loc_subjects.mrc
MRRC_IA_LENDABLE=/path/to/ia_lendable.mrc
MRRC_WATSON=/path/to/watson_library.mrc

# Or use the downloads directory
MRRC_DOWNLOADS_DIR=/path/to/mrrc-testbed/data/downloads

# Custom datasets (BYOD)
MRRC_CUSTOM_DATASET=/path/to/my_records.mrc
MRRC_CUSTOM_DIR=/path/to/my_collection/
```

### `.gitignore` essentials

```gitignore
# Local configuration
.env

# Downloaded datasets (never commit)
data/downloads/

# Custom datasets (never commit)
data/custom/

# Local test results
results/

# Rust build artifacts
target/

# Python artifacts
__pycache__/
*.pyc
.pytest_cache/
.venv/

# Editor artifacts
.vscode/
.idea/
```

### Synthetic data policy

Synthetic records in `data/synthetic/` **are committed** because:
- They're small (intentionally minimal for specific test cases)
- They need version control (changes affect test expectations)
- They document edge cases (each has accompanying documentation)
- They're reproducible (generator scripts are included)

Each synthetic dataset includes a README explaining:
- What it tests
- How it was generated
- Expected behavior when processed

### Fixture Curation Strategy

Committed fixtures (~1000 records, ~10MB) are sourced from **Library of Congress** data exports, which are US government works in the public domain.

**Selection approach:**

Two complementary methods:

1. **Random sampling** — Randomly select ~500 records from LOC Books All to get natural distribution of real-world patterns
2. **Targeted selection** — Select ~500 records that exercise specific MARC aspects:
   - Various record types (books, serials, maps, music, etc.)
   - Different encoding levels
   - Complex field structures (many subfields, repeated fields)
   - International content (CJK, Cyrillic, diacritics)
   - Edge cases discovered during testing

**Provenance tracking:**

Every committed fixture record includes provenance metadata. This is critical for:
- Crediting data sources appropriately
- Reproducing issues with original records
- Verifying fixtures against updated source data
- Legal clarity on data licensing

Provenance is tracked via a manifest file:

```
data/fixtures/
├── bibliographic/
│   ├── sample.mrc           # The actual records
│   └── manifest.json        # Provenance for each record
├── authority/
│   ├── sample.mrc
│   └── manifest.json
└── edge_cases/
    ├── sample.mrc
    └── manifest.json
```

**Manifest format:**

```json
{
  "source": "Library of Congress Books All",
  "source_url": "https://www.loc.gov/cds/products/marcDist.php",
  "download_date": "2024-01-15",
  "license": "Public Domain (US Government Work)",
  "records": [
    {
      "index": 0,
      "control_number": "12345678",
      "source_offset": 1048576,
      "selection_reason": "random_sample",
      "notes": null
    },
    {
      "index": 1,
      "control_number": "87654321",
      "source_offset": 2097152,
      "selection_reason": "targeted:cjk_content",
      "notes": "Contains CJK characters in 245$a"
    },
    {
      "index": 42,
      "control_number": "11223344",
      "source_offset": null,
      "source_file": "ia_lendable_books.mrc",
      "selection_reason": "edge_case:discovered",
      "notes": "Truncated directory - discovered in malformed.rs testing",
      "discovered_by": "testbed discovery run 2024-02-01",
      "mrrc_issue": "https://github.com/dchud/mrrc/issues/123"
    }
  ]
}
```

**Selection reasons:**
- `random_sample` — Randomly selected from source
- `targeted:<aspect>` — Selected to test specific aspect (e.g., `targeted:cjk_content`, `targeted:many_subfields`)
- `edge_case:discovered` — Discovered during testbed runs, promoted to fixture
- `edge_case:reported` — Reported by user, added to fixtures

### Initial Fixture Curation

The `curate_fixtures.py` script handles initial fixture population:

```bash
# Random sample from LOC Books All
uv run python scripts/curate_fixtures.py \
    --source /path/to/loc_books_all.mrc \
    --output data/fixtures/bibliographic/ \
    --count 500 \
    --method random \
    --source-name "Library of Congress Books All" \
    --source-url "https://www.loc.gov/cds/products/marcDist.php"

# Targeted selection (interactive or via criteria file)
uv run python scripts/curate_fixtures.py \
    --source /path/to/loc_books_all.mrc \
    --output data/fixtures/bibliographic/ \
    --count 500 \
    --method targeted \
    --criteria criteria/bibliographic_coverage.json
```

**Targeted selection criteria file:**

```json
{
  "criteria": [
    {"name": "cjk_content", "count": 50, "filter": "has_cjk_in_245"},
    {"name": "cyrillic_content", "count": 30, "filter": "has_cyrillic"},
    {"name": "many_subfields", "count": 30, "filter": "max_subfields > 20"},
    {"name": "long_fields", "count": 30, "filter": "max_field_length > 5000"},
    {"name": "serials", "count": 50, "filter": "leader[7] == 's'"},
    {"name": "maps", "count": 30, "filter": "leader[6] == 'e'"},
    {"name": "music", "count": 30, "filter": "leader[6] in ['c', 'd', 'j']"},
    {"name": "pre_1900", "count": 50, "filter": "pub_year < 1900"},
    {"name": "authority_links", "count": 50, "filter": "has_field('100') and subfield_count('100', '0') > 0"},
    {"name": "complex_subjects", "count": 50, "filter": "field_count('650') > 5"}
  ]
}
```

The script generates manifest.json automatically with full provenance.

### Record Extraction Utility

Extracting a single record from a multi-GB file at a known byte offset:

```bash
# Extract record at offset 1234567 from large file
uv run python scripts/extract_record.py \
    /path/to/large_file.mrc \
    --offset 1234567 \
    --output extracted_record.mrc

# Extract by control number (slower - scans file)
uv run python scripts/extract_record.py \
    /path/to/large_file.mrc \
    --control-number "ocm12345678" \
    --output extracted_record.mrc

# Extract and display info without saving
uv run python scripts/extract_record.py \
    /path/to/large_file.mrc \
    --offset 1234567 \
    --info
```

This is essential for reproducing issues found during discovery runs.

### Fixture Validation and Size Monitoring

The `validate_fixtures.py` script enforces fixture integrity:

```bash
# Full validation
uv run python scripts/validate_fixtures.py

# Output:
# Validating data/fixtures/bibliographic/...
#   ✓ sample.mrc: 523 records, 4.2 MB
#   ✓ manifest.json: 523 entries, all records accounted for
#   ✓ No orphaned manifest entries
#   ✓ No untracked records in .mrc file
# Validating data/fixtures/edge_cases/...
#   ✓ sample.mrc: 47 records, 892 KB
#   ✓ manifest.json: 47 entries, all records accounted for
#
# Total fixture size: 8.7 MB (target: <10 MB)
# Status: OK
```

**Validation checks:**

1. **Manifest sync** — Every record in .mrc has a manifest entry, and vice versa
2. **Control number match** — Manifest control_number matches actual record
3. **Size budget** — Total fixtures under 10MB target (warning at 8MB, error at 10MB)
4. **Provenance completeness** — Every record has source, selection_reason
5. **Record validity** — All records parse without error

**CI integration:**

```yaml
# .github/workflows/ci.yml
- name: Validate fixtures
  run: uv run python scripts/validate_fixtures.py --strict
```

Fails CI if fixtures are invalid or over size budget.

---

## CI vs Local Testing

### CI Mode (GitHub Actions)

**Characteristics:**
- Uses only committed fixtures (`data/fixtures/`, `data/synthetic/`)
- Fast execution (target: <10 minutes)
- Runs on every PR and push to main
- No external downloads during CI
- Validates that testbed infrastructure works

**What CI tests:**
- Rust test harness compiles and runs with fixtures
- Python test infrastructure works
- Synthetic malformed record handling
- Basic encoding test vectors

**What CI skips:**
- Large-scale stress tests
- Memory leak detection (requires sustained load)
- Concurrency scaling tests
- Real-world dataset coverage

### Local Mode (Developer workstation)

**Characteristics:**
- Uses full downloaded datasets
- Thorough testing (may take hours for full suite)
- Run manually before releases or when investigating issues
- Catches issues that only appear at scale

**What local mode adds:**
- Memory profiling over millions of records
- Concurrency scaling (1-16+ threads)
- Real-world malformed record discovery
- Full encoding coverage from international data
- Performance benchmarks at scale

### Custom Mode (Bring Your Own Data)

**Characteristics:**
- Uses user-provided datasets
- Validates mrrc against specific institutional data
- Useful for migration validation

### Switching modes

```bash
# CI mode (default if MRRC_TEST_MODE not set)
cargo test
uv run pytest suites/

# Local mode with full datasets
MRRC_TEST_MODE=local cargo test
MRRC_TEST_MODE=local uv run pytest suites/

# Custom mode with your own data
MRRC_TEST_MODE=custom cargo test
MRRC_TEST_MODE=custom uv run pytest suites/

# Or set in .env file
echo "MRRC_TEST_MODE=local" >> .env
```

---

## Reporting

### Approach: Unified local reports + CI green checks

**CI reporting:**
- Standard test output in GitHub Actions
- Green/red checks visible in PR
- Failure details in CI logs
- No persistent report storage (tests should pass)

**Local reporting:**
- Unified HTML report generated after test runs
- JSON export for programmatic analysis
- Benchmark history tracking (local only)
- Discovered edge case catalog

### Running tests and generating reports

```bash
# Run Rust tests
cargo test

# Run Rust tests with local datasets
MRRC_TEST_MODE=local cargo test

# Run Rust stress tests only
MRRC_TEST_MODE=local cargo test stress

# Run Python tests
uv run pytest suites/

# Run with verbose output
cargo test -- --nocapture
uv run pytest suites/ -v

# Generate HTML report
uv run pytest suites/ --html=results/report.html
```

### Report contents

The unified report includes:
- Pass/fail summary by suite
- Execution time per suite and test
- Benchmark results (if run)
- Failure details with record excerpts
- Discovered edge cases catalog
- Dataset statistics (records processed, unique patterns found)

---

## Public MARC Datasets

### Primary sources

| Source | URL | Size | Records | Best For |
|--------|-----|------|---------|----------|
| **LOC Books All** | https://www.loc.gov/cds/products/marcDist.php | ~15GB | ~25M | Stress, scale testing |
| **LOC Name Authority** | https://www.loc.gov/cds/products/marcDist.php | ~5GB | ~10M | Authority testing |
| **LOC Subject Authority** | https://www.loc.gov/cds/products/marcDist.php | ~200MB | ~400K | Authority testing |
| **Internet Archive Lendable** | https://archive.org/details/marc_lendable_books | ~1GB | ~1.4M | Malformed discovery, encoding |
| **Watson Library (Met)** | https://github.com/Thomas-J-Watson-Library/Marc-Record-Sets | ~100MB | ~200K | Quick local testing |

### Supplementary sources for encoding tests

| Source | Content | Notes |
|--------|---------|-------|
| **National Diet Library (Japan)** | CJK records | May require account |
| **Deutsche Nationalbibliothek** | German diacritics | Free access |
| **Russian State Library** | Cyrillic | Check licensing |

### Download script usage

```bash
# List available datasets
uv run python scripts/download_datasets.py --list

# Download specific dataset
uv run python scripts/download_datasets.py watson

# Download all primary datasets (large!)
uv run python scripts/download_datasets.py --all

# Verify downloads
uv run python scripts/download_datasets.py --verify
```

---

## Test Suites

### Rust Test Suites (Primary)

#### `stress.rs` - Scale and Memory Testing

**Purpose:** Validate performance and memory behavior at production scale. This is where bugs invisible at small scale surface.

**Focus:** Issues that only appear with millions of records:
- Cumulative memory leaks (1KB/record = 25GB leak on LOC)
- Unbounded queue/buffer growth
- GC pressure and pause times
- Thread pool exhaustion
- File handle leaks

**Key tests:**
| Test | CI | Local | Description |
|------|-----|-------|-------------|
| `memory_stability` | Skip | Full | No memory growth over 10M+ records |
| `throughput_sustained` | Skip | Full | Stable throughput over extended runs |
| `thread_scaling` | Skip | Full | Near-linear scaling to core count |
| `resource_cleanup` | Basic | Full | No leaked handles/buffers |

**Success criteria:**
- Memory stable (±5%) over extended runs
- No resource leaks after processing completes
- Throughput remains stable (no degradation over time)

---

#### `malformed.rs` - Error Recovery Discovery

**Purpose:** Discover real-world malformed record patterns and verify graceful handling.

**Focus:** Finding unknown malformed patterns in real data, not testing known synthetic cases (mrrc unit tests can cover those).

**Key tests:**
| Test | CI | Local | Description |
|------|-----|-------|-------------|
| `discover_malformed_patterns` | Skip | Full | Catalog malformed records in IA Lendable |
| `no_panics` | Basic | Full | No panics on any input |
| `error_messages_useful` | Basic | Full | Errors identify the problem |

**Discovered malformed patterns are cataloged:**
```rust
// Malformed pattern discovered in IA Lendable
// Record offset: 1234567, Pattern: truncated_directory
// Details: Directory ends mid-entry at byte 45
```

**Success criteria:**
- No crashes or panics on any real-world input
- Catalog of malformed patterns discovered
- Error messages identify specific problems

---

#### `encoding.rs` - International Character Testing

**Purpose:** Verify MARC-8 and UTF-8 handling with real international records.

**Focus:** Real records from international libraries, not synthetic test vectors.

**Key tests:**
| Test | CI | Local | Description |
|------|-----|-------|-------------|
| `cjk_roundtrip` | Skip | Full | CJK records from National Diet Library |
| `cyrillic_roundtrip` | Skip | Full | Cyrillic from Russian State Library |
| `diacritics_roundtrip` | Skip | Full | European diacritics from DNB |
| `mixed_encoding` | Skip | Full | Records mixing MARC-8 and UTF-8 |

**Success criteria:**
- No mojibake in round-trips of real international records
- Encoding detection works on real data
- Combining characters handled properly

---

#### `concurrent.rs` - Thread Safety at Scale

**Purpose:** Verify thread safety under sustained parallel load.

**Focus:** Race conditions and deadlocks that only surface under sustained load, not basic thread safety (covered by mrrc unit tests).

**Key tests:**
| Test | CI | Local | Description |
|------|-----|-------|-------------|
| `sustained_parallel_read` | Skip | Full | 16+ threads for 10M+ records |
| `producer_consumer_stress` | Skip | Full | Pipeline under sustained load |
| `no_data_corruption` | Skip | Full | Verify data integrity under load |

**Success criteria:**
- No race conditions or data corruption
- No deadlocks under sustained load
- Stable performance across thread counts

---

#### `discovery.rs` - Edge Case Discovery

**Purpose:** Systematically discover edge cases in real-world data.

**Focus:** Finding unusual patterns that break assumptions.

**Key tests:**
| Test | CI | Local | Description |
|------|-----|-------|-------------|
| `unusual_field_combinations` | Skip | Full | Rare field patterns in LOC |
| `extreme_values` | Skip | Full | Unusually long fields, many subfields |
| `encoding_edge_cases` | Skip | Full | Unusual encoding patterns |

**Output:** Catalog of discovered edge cases for potential addition to mrrc test fixtures.

#### Rust Discovery Output

Rust tests use a shared discovery library to output findings in the standard JSON format:

```rust
// crates/mrrc_testbed/src/discovery.rs

use crate::discovery::{Discovery, DiscoveryWriter};

#[test]
fn discover_malformed_patterns() {
    let mut writer = DiscoveryWriter::new("malformed.rs", "discover_malformed_patterns");

    let dataset = get_dataset("ia_lendable").unwrap();
    let mut reader = MarcReader::new(File::open(&dataset).unwrap());
    let mut offset = 0u64;

    loop {
        match reader.read_record() {
            Ok(Some(record)) => {
                offset = reader.position();
            }
            Ok(None) => break,  // EOF
            Err(e) => {
                // Record the discovery
                writer.record_error(
                    &dataset,
                    offset,
                    reader.last_raw_bytes(),  // Raw bytes of problematic record
                    &e,
                );
                // Continue to next record
                offset = reader.position();
            }
        }
    }

    // Write discoveries to results/discoveries/
    writer.finalize().unwrap();
}
```

The `DiscoveryWriter` handles:
- Extracting problematic records to individual .mrc files
- Computing sha256 for deduplication
- Writing JSON in the standard format
- Updating the discovery index

---

### Python Test Suites (Compatibility Focus)

#### `pymarc_compat/` - API Compatibility with Real Data

**Purpose:** Verify pymarc API compatibility holds up with real-world data patterns.

**Focus:** Testing against latest pymarc release only. Verifies that real-world usage patterns work through the Python bindings.

**Key tests:**
| Test | CI | Local | Description |
|------|-----|-------|-------------|
| `test_real_scripts.py` | Skip | Full | Port actual pymarc scripts from the wild |
| `test_iteration_scale.py` | Skip | Full | Iterator behavior over large files |

**Success criteria:**
- Real pymarc scripts work unmodified with mrrc
- No behavioral differences at scale

---

#### `encoding/` - Encoding Through Python Bindings

**Purpose:** Verify encoding handling works correctly through Python bindings.

**Key tests:**
| Test | CI | Local | Description |
|------|-----|-------|-------------|
| `test_string_handling.py` | Skip | Full | Unicode strings from real records |

---

#### `discovery/` - Edge Case Discovery via Python

**Purpose:** Python-friendly interface for cataloging discovered edge cases.

---

## Development Workflow

### Initial setup

```bash
# Clone repository
git clone https://github.com/dchud/mrrc-testbed.git
cd mrrc-testbed

# Set up Rust
cargo build

# Set up Python environment with uv
uv sync

# Copy and configure environment
cp .env.example .env
# Edit .env with local paths

# Verify setup
cargo test --no-run
uv run pytest suites/ -v --collect-only
```

### Running tests

```bash
# Run Rust tests (CI mode - fixtures only)
cargo test

# Run Rust tests (local mode - full datasets)
MRRC_TEST_MODE=local cargo test

# Run specific Rust test module
MRRC_TEST_MODE=local cargo test stress

# Run Python tests (CI mode)
uv run pytest suites/

# Run Python tests (local mode)
MRRC_TEST_MODE=local uv run pytest suites/

# Run with custom data
MRRC_TEST_MODE=custom cargo test
MRRC_TEST_MODE=custom uv run pytest suites/
```

### Downloading datasets (local mode)

```bash
# Download Watson Library (smallest, good starting point)
uv run python scripts/download_datasets.py watson

# Download Internet Archive Lendable
uv run python scripts/download_datasets.py ia_lendable

# Download LOC Books All (large! ~15GB)
uv run python scripts/download_datasets.py loc_books

# Verify all downloads
uv run python scripts/download_datasets.py --verify
```

### Adding new tests

1. For Rust tests: Add to appropriate file in `crates/mrrc_testbed/tests/`
2. For Python tests: Add to appropriate directory in `suites/`
3. Use dataset abstraction for data access (handles CI/local/custom)
4. Mark tests requiring local mode with `#[ignore]` (Rust) or `@pytest.mark.local` (Python)
5. Document any discovered edge cases

---

## Edge Case to Issue Workflow

When the testbed discovers a record that breaks mrrc (or exhibits unexpected behavior), the goal is to make it **as easy as possible** to turn that discovery into an actionable mrrc issue. This workflow minimizes friction between "found a problem" and "filed an issue with everything needed to fix it."

### Discovery output format

When tests discover problematic records, they output a structured discovery report:

```
results/discoveries/
├── index.json                    # Index of all discoveries with dedup info
├── 2024-02-01_malformed_discovery.json
├── 2024-02-01_encoding_issues.json
├── latest.json                   # Symlink to most recent
└── records/                      # Extracted problematic records
    ├── disc-2024-02-01-001.mrc
    └── disc-2024-02-01-002.mrc
```

**Discovery record format:**

```json
{
  "discovery_id": "disc-2024-02-01-001",
  "discovered_at": "2024-02-01T14:32:00Z",
  "test_suite": "malformed.rs",
  "test_name": "discover_malformed_patterns",
  "source_dataset": "ia_lendable",
  "source_file": "/path/to/ia_lendable_books.mrc",
  "record": {
    "offset_bytes": 1234567,
    "control_number": "ocm12345678",
    "raw_bytes_base64": "MDEyMzQ1Njc4OTAxMjM0NTY3ODkw...",
    "sha256": "a1b2c3d4...",
    "extracted_to": "results/discoveries/records/disc-2024-02-01-001.mrc"
  },
  "issue": {
    "category": "malformed_record",
    "subcategory": "truncated_directory",
    "severity": "error",
    "message": "Directory ends mid-entry at byte 45",
    "mrrc_error": "ParseError::InvalidDirectory"
  },
  "context": {
    "mrrc_version": "0.6.0",
    "rust_version": "1.75.0",
    "os": "linux-x86_64"
  },
  "status": "new",
  "filed_issue_url": null,
  "duplicate_of": null
}
```

### Discovery deduplication

The same problematic record might be discovered multiple times (across runs, or same pattern in multiple records). The discovery system handles this:

**Deduplication by record content:**

```json
// results/discoveries/index.json
{
  "discoveries": {
    "disc-2024-02-01-001": {
      "record_sha256": "a1b2c3d4...",
      "error_signature": "ParseError::InvalidDirectory:truncated_directory",
      "status": "new"
    },
    "disc-2024-02-01-002": {
      "record_sha256": "a1b2c3d4...",
      "error_signature": "ParseError::InvalidDirectory:truncated_directory",
      "status": "duplicate",
      "duplicate_of": "disc-2024-02-01-001"
    }
  },
  "by_signature": {
    "ParseError::InvalidDirectory:truncated_directory": ["disc-2024-02-01-001", "disc-2024-02-01-002"]
  }
}
```

**Pattern-level discoveries:**

When the same error pattern affects many records, create a single "pattern discovery" with a count:

```json
{
  "discovery_id": "disc-2024-02-01-pattern-001",
  "discovery_type": "pattern",
  "pattern": {
    "error_signature": "ParseError::InvalidDirectory:truncated_directory",
    "affected_count": 47,
    "sample_records": ["disc-2024-02-01-001", "disc-2024-02-01-003"]
  }
}
```

This prevents filing 47 identical issues for the same underlying bug.

### One-command issue filing

The testbed provides a script to file an mrrc issue directly from a discovery:

```bash
# Review recent discoveries (excludes duplicates by default)
uv run python scripts/file_issue.py --list

# Include duplicates and already-filed
uv run python scripts/file_issue.py --list --all

# Preview what the issue would look like
uv run python scripts/file_issue.py disc-2024-02-01-001 --preview

# File the issue (requires GITHUB_TOKEN)
uv run python scripts/file_issue.py disc-2024-02-01-001 --file
```

**How record data is shared:**

GitHub Issues API doesn't support file attachments. The script handles this by:

1. **Creating a GitHub Gist** with the problematic record (`.mrc` file + metadata)
2. **Linking the gist** in the issue body
3. **Including base64-encoded record** in a collapsed details block as backup

```bash
# The script creates:
# 1. Gist: https://gist.github.com/user/abc123 (contains disc-2024-02-01-001.mrc)
# 2. Issue: https://github.com/dchud/mrrc/issues/123 (links to gist)
```

**Generated issue format:**

```markdown
## Summary

Testbed discovered a malformed record that causes `ParseError::InvalidDirectory`.

## Record Details

- **Source**: Internet Archive Lendable Books
- **Control Number**: ocm12345678
- **Discovery**: testbed run 2024-02-01, malformed.rs::discover_malformed_patterns
- **Record**: [disc-2024-02-01-001.mrc](https://gist.github.com/user/abc123) (257 bytes)

## Error

```
ParseError::InvalidDirectory: Directory ends mid-entry at byte 45
```

## Reproduction

```rust
use mrrc::MarcReader;
use std::fs::File;

// Download from gist or use base64 below
let file = File::open("disc-2024-02-01-001.mrc")?;
let mut reader = MarcReader::new(file);
let result = reader.read_record();
// Expected: graceful error handling
// Actual: [describe actual behavior]
```

<details>
<summary>Raw record (base64)</summary>

```
MDEyMzQ1Njc4OTAxMjM0NTY3ODkwMTIzNDU2Nzg5MDEyMzQ1Njc4OTAxMjM0NTY3ODkwMTIz
NDU2Nzg5MDEyMzQ1Njc4OTAxMjM0NTY3ODkwMTIzNDU2Nzg5MDEyMzQ1Njc4OTAxMjM0NTY3
```

Decode with: `base64 -d <<< "..." > record.mrc`

</details>

## Environment

- mrrc version: 0.6.0
- Rust version: 1.75.0
- OS: linux-x86_64

---
*Filed automatically by mrrc-testbed ([discovery](https://github.com/dchud/mrrc-testbed/blob/main/results/discoveries/disc-2024-02-01-001.json))*
```

### Manual workflow (without script)

If the automated script isn't available, the discovery output provides everything needed:

1. **Extract the record**: `results/discoveries/records/disc-xxx.mrc`
2. **Copy the error details**: From the discovery JSON
3. **Include provenance**: Source dataset, offset, control number
4. **File manually**: Create issue at https://github.com/dchud/mrrc/issues

### Linking issues to discoveries

When an issue is filed (automatically or manually), link it back to the discovery:

```bash
# Automatic: file_issue.py updates the discovery JSON after filing
# Manual: use link command
uv run python scripts/file_issue.py disc-2024-02-01-001 --link https://github.com/dchud/mrrc/issues/123
```

This updates the discovery record:

```json
{
  "discovery_id": "disc-2024-02-01-001",
  "status": "filed",
  "filed_issue_url": "https://github.com/dchud/mrrc/issues/123",
  "filed_at": "2024-02-01T15:00:00Z"
}
```

### Promoting discoveries to fixtures

After an issue is filed and fixed, the record can be promoted to the committed fixtures:

```bash
# Add discovered record to edge_cases fixtures with full provenance
uv run python scripts/promote_discovery.py disc-2024-02-01-001 --fixture=edge_cases

# If issue URL not already linked, provide it:
uv run python scripts/promote_discovery.py disc-2024-02-01-001 \
    --fixture=edge_cases \
    --issue https://github.com/dchud/mrrc/issues/123

# This:
# 1. Copies the record to data/fixtures/edge_cases/sample.mrc
# 2. Updates manifest.json with provenance
# 3. Links to the mrrc issue
# 4. Marks discovery as "promoted"
# 5. Runs validate_fixtures.py to ensure consistency
```

The manifest entry automatically includes:
- Original source dataset and offset
- Discovery date and test that found it
- Link to the mrrc issue
- Resolution status

**Promotion guards:**

The script refuses to promote if:
- Discovery has no linked issue (unless `--force`)
- Issue is still open (unless `--force`)
- Record already exists in fixtures (by sha256)
- Promotion would exceed fixture size budget

### Reviewing and triaging discoveries

```bash
# List new discoveries (not filed, not duplicates)
uv run python scripts/file_issue.py --list

# Output:
# ID                     Category              Severity  Records  Status
# disc-2024-02-01-001    truncated_directory   error     1        new
# disc-2024-02-01-003    invalid_encoding      warning   1        new
# disc-2024-02-01-pat-1  truncated_leader      error     47       new (pattern)

# Show details of a discovery
uv run python scripts/file_issue.py disc-2024-02-01-001 --show

# Mark as "won't fix" (not worth filing)
uv run python scripts/file_issue.py disc-2024-02-01-003 --dismiss --reason "Known pymarc limitation, not our bug"

# View dismissed discoveries
uv run python scripts/file_issue.py --list --status=dismissed
```

**Discovery statuses:**

| Status | Meaning |
|--------|---------|
| `new` | Just discovered, needs review |
| `duplicate` | Same as another discovery (by sha256 or pattern) |
| `filed` | Issue created in mrrc |
| `dismissed` | Reviewed and decided not to file |
| `promoted` | Added to fixtures after fix |

### Workflow summary

```
Discovery → Review → File Issue → Fix in mrrc → Promote to Fixture
    ↓          ↓          ↓              ↓               ↓
  Auto      Manual    One cmd      Normal dev      One cmd
 output     review   or manual      workflow       or manual
            or dismiss
```

**Design principle**: Every step after discovery should be optional but easy. An operator can:
- Just review discoveries and ignore them
- Dismiss discoveries that aren't worth filing
- File issues manually with copy-paste from discovery JSON
- Use the one-command filing script
- Promote fixed issues to fixtures for regression testing

No automatic issue creation — humans decide what's worth filing.

---

## State Management

Running the testbed repeatedly over time requires tracking state across runs: which discoveries are new, which are duplicates of known issues, which have been fixed, etc. This section describes how state is managed for both human and automated operators.

### Design: YAML Source of Truth + SQLite Query Layer

**Principle:** Human-readable files are the source of truth; database is a derived index.

```
state/
├── discoveries/           # YAML files (git-tracked)
│   ├── disc-2024-02-01-001.yaml
│   ├── disc-2024-02-01-002.yaml
│   └── ...
├── runs/                  # YAML files (git-tracked)
│   ├── run-2024-02-01-001.yaml
│   └── ...
├── index.db              # SQLite (gitignored, rebuilt from YAML)
└── schema.sql            # Database schema (git-tracked)
```

**Why this hybrid:**

| Concern | YAML | SQLite |
|---------|------|--------|
| Human readability | Excellent | Poor |
| Git diffs/PRs | Clean diffs | Binary conflicts |
| Complex queries | Slow/awkward | Fast/natural |
| Agent automation | Workable | Excellent |
| Rebuild from scratch | N/A (is source) | Yes |

### State Files

**Discovery YAML:**

```yaml
# state/discoveries/disc-2024-02-01-001.yaml
discovery_id: disc-2024-02-01-001
discovered_at: 2024-02-01T14:32:00Z
discovered_in_run: run-2024-02-01-001
mrrc_version: 0.6.0

record:
  sha256: a1b2c3d4e5f6...
  control_number: ocm12345678
  source_dataset: ia_lendable
  source_offset: 1234567
  extracted_file: records/disc-2024-02-01-001.mrc

error:
  category: malformed_record
  signature: "ParseError::InvalidDirectory:truncated_directory"
  message: "Directory ends mid-entry at byte 45"
  severity: error

status: filed
filed_issue_url: https://github.com/dchud/mrrc/issues/123
filed_at: 2024-02-01T15:00:00Z

verification:
  fixed_in_version: 0.7.0
  verified_in_run: run-2024-03-15-001
  verified_at: 2024-03-15T10:00:00Z

promoted_to_fixture: data/fixtures/edge_cases/
promoted_at: 2024-03-16T09:00:00Z
```

**Run YAML:**

```yaml
# state/runs/run-2024-02-01-001.yaml
run_id: run-2024-02-01-001
started_at: 2024-02-01T14:00:00Z
completed_at: 2024-02-01T16:30:00Z

environment:
  mrrc_version: 0.6.0
  rust_version: 1.75.0
  python_version: 3.12.1
  os: linux-x86_64

datasets:
  - name: ia_lendable
    path: /data/ia_lendable_books.mrc
    records_processed: 1423567
  - name: loc_books
    path: /data/loc_books_all.mrc
    records_processed: 25000000

results:
  total_records: 26423567
  errors_found: 47
  new_discoveries: 12
  duplicate_discoveries: 35

discoveries:
  - disc-2024-02-01-001
  - disc-2024-02-01-002
  # ... etc
```

### SQLite Index

The SQLite database is rebuilt from YAML files on demand:

```bash
# Rebuild index from YAML source files
uv run python scripts/rebuild_index.py

# Query discoveries
uv run python scripts/query.py "SELECT * FROM discoveries WHERE status = 'new'"

# Or use the CLI
uv run python scripts/testbed.py discoveries --status=new
```

**Schema (simplified):**

```sql
-- state/schema.sql
CREATE TABLE runs (
    run_id TEXT PRIMARY KEY,
    started_at TEXT,
    completed_at TEXT,
    mrrc_version TEXT,
    total_records INTEGER,
    errors_found INTEGER
);

CREATE TABLE discoveries (
    discovery_id TEXT PRIMARY KEY,
    discovered_in_run TEXT REFERENCES runs(run_id),
    record_sha256 TEXT,
    error_signature TEXT,
    status TEXT,  -- new, duplicate, filed, dismissed, verified, promoted
    filed_issue_url TEXT,
    fixed_in_version TEXT,
    verified_in_run TEXT REFERENCES runs(run_id)
);

CREATE TABLE run_discoveries (
    run_id TEXT REFERENCES runs(run_id),
    discovery_id TEXT REFERENCES discoveries(discovery_id),
    occurrence_type TEXT,  -- new, recurrence, resolved
    PRIMARY KEY (run_id, discovery_id)
);

-- Useful indices
CREATE INDEX idx_discoveries_status ON discoveries(status);
CREATE INDEX idx_discoveries_signature ON discoveries(error_signature);
CREATE INDEX idx_discoveries_sha256 ON discoveries(record_sha256);
```

### Cross-Run Tracking

When a testbed run completes, the system:

1. **Loads existing discoveries** from YAML files
2. **Compares new findings** against known discoveries (by sha256 and error signature)
3. **Categorizes each finding:**
   - `new` — Never seen before
   - `recurrence` — Same as existing unfixed discovery
   - `resolved` — Previously discovered, but no longer errors (fix verified!)
4. **Updates state files** accordingly
5. **Rebuilds SQLite index**

```bash
# After a run, import results and update state
uv run python scripts/import_run.py results/2024-02-01_run/

# Output:
# Importing run results...
#   Total errors found: 47
#   New discoveries: 12
#   Recurrences of known issues: 33
#   Resolved (no longer errors): 2  ← These were fixed!
#
# Updated state/discoveries/ (12 new files)
# Updated state/runs/run-2024-02-01-001.yaml
# Rebuilt state/index.db
```

### Version Tracking and Regression Testing

Every run records the mrrc version (`mrrc.__version__` for Python, `env!("CARGO_PKG_VERSION")` for Rust). This enables:

**1. Tracking when issues were fixed:**

```bash
# Find which version fixed a discovery
uv run python scripts/query.py "
  SELECT discovery_id, error_signature, fixed_in_version
  FROM discoveries
  WHERE status = 'verified'
  ORDER BY fixed_in_version
"
```

**2. Regression testing after mrrc releases:**

```bash
# Run testbed with new mrrc version
MRRC_TEST_MODE=local cargo test

# Import results - system automatically detects resolutions
uv run python scripts/import_run.py results/latest/

# Check what got fixed
uv run python scripts/testbed.py resolved --since-version 0.6.0
```

**3. Detecting regressions:**

If a previously-verified-fixed discovery recurs in a new run:

```bash
# Alert: regression detected!
# disc-2024-02-01-001 was fixed in 0.7.0 but recurred in 0.8.0
```

### Local vs Centralized State

**Centralized (mrrc-testbed repo):**
- `state/discoveries/*.yaml` — Committed, shared
- `state/runs/*.yaml` — Committed (or selected runs)
- `state/index.db` — Gitignored, rebuilt locally

**Local/Private use:**
- Everything in `state/` is gitignored
- User maintains their own local state
- Can optionally export a single discovery for PR submission

```bash
# Export a discovery for PR submission to central repo
uv run python scripts/export_discovery.py disc-2024-02-01-001 --output pr-submission/
# Creates: pr-submission/disc-2024-02-01-001.yaml + pr-submission/records/disc-2024-02-01-001.mrc
```

---

## Repository Growth Over Time

The testbed repository grows as discoveries accumulate. This section describes what changes over time and how to manage growth.

### What Grows

| Content | Location | Growth Pattern |
|---------|----------|----------------|
| Fixtures | `data/fixtures/` | Slow (~10 records/year from promoted discoveries) |
| State files | `state/discoveries/` | Moderate (deduplicated, ~100/year) |
| Run history | `state/runs/` | Configurable (can prune old runs) |
| Documentation | `docs/` | Slow (stable after initial setup) |

### What Doesn't Grow (gitignored)

| Content | Location | Notes |
|---------|----------|-------|
| Downloaded datasets | `data/downloads/` | Re-downloaded as needed |
| Local results | `results/` | Per-run, can be deleted |
| SQLite index | `state/index.db` | Rebuilt from YAML |
| Custom data | `data/custom/` | User's own data |

### Timeline Example

**Month 1 (Initial Setup):**
```
data/fixtures/           ~8 MB (initial curation from LOC)
state/discoveries/       0 files
state/runs/             0 files
```

**Month 6 (Active Testing):**
```
data/fixtures/           ~8.5 MB (+5 promoted edge cases)
state/discoveries/       ~50 files (deduplicated)
state/runs/             ~20 files (weekly runs)
```

**Year 2 (Mature):**
```
data/fixtures/           ~9.5 MB (+15 promoted edge cases)
state/discoveries/       ~150 files
state/runs/             ~50 files (pruned to monthly summaries)
```

### Pruning and Archival

Old run data can be archived or pruned:

```bash
# Archive runs older than 1 year
uv run python scripts/archive_runs.py --older-than 1y --output archive/2023-runs.tar.gz

# Prune archived runs from state/runs/ (keeps discoveries)
uv run python scripts/prune_runs.py --older-than 1y

# Rebuild index after pruning
uv run python scripts/rebuild_index.py
```

Discoveries are never automatically pruned — they're the valuable long-term asset.

---

## Documentation Structure

The testbed uses MkDocs for documentation, hosted alongside the code.

### Directory Structure

```
docs/
├── mkdocs.yml                    # MkDocs configuration
├── index.md                      # Home page / introduction
├── getting-started/
│   ├── index.md                  # Quick start overview
│   ├── installation.md           # Setup instructions
│   └── first-run.md              # Running your first test
├── tutorials/
│   ├── index.md                  # Tutorial overview
│   ├── running-ci-mode.md        # Using fixtures for quick tests
│   ├── running-local-mode.md     # Using downloaded datasets
│   ├── running-custom-mode.md    # Using your own data (BYOD)
│   ├── reviewing-discoveries.md  # Triaging and reviewing findings
│   └── filing-issues.md          # Filing issues to mrrc
├── guides/
│   ├── index.md                  # Guide overview
│   ├── contributing-to-mrrc.md   # How to submit PRs to mrrc
│   ├── contributing-discoveries.md # How to submit discoveries to mrrc-testbed
│   ├── adding-fixtures.md        # How fixtures are curated and added
│   └── regression-testing.md     # Verifying fixes across versions
├── reference/
│   ├── index.md                  # Reference overview
│   ├── discovery-format.md       # Discovery YAML/JSON schema
│   ├── run-format.md             # Run YAML schema
│   ├── manifest-format.md        # Fixture manifest schema
│   ├── cli-reference.md          # Command-line tool reference
│   └── provenance.md             # How provenance is tracked
├── explanation/
│   ├── index.md                  # Explanation overview
│   ├── scope.md                  # What the testbed does and doesn't do
│   ├── state-management.md       # How state is tracked over time
│   └── interaction-models.md     # Centralized vs local usage
└── changelog.md                  # Version history
```

### Key Documentation Pages

**Introduction (`index.md`):**
- What is mrrc-testbed?
- Who is it for?
- Quick example of running tests
- Links to tutorials and guides

**Scope Clarification (`explanation/scope.md`):**
- What the testbed tests (real-world data, scale)
- What it doesn't test (covered by mrrc unit tests)
- Relationship to mrrc proper

**Contributing to mrrc (`guides/contributing-to-mrrc.md`):**
- When to file an issue vs PR
- Issue format for testbed discoveries
- How reproduction files are shared (gists)
- Linking issues back to testbed discoveries

**Contributing Discoveries (`guides/contributing-discoveries.md`):**
- When to submit a discovery
- How to export a discovery for PR
- PR format and review process
- What makes a good discovery submission

**Discovery Format Reference (`reference/discovery-format.md`):**
- Complete YAML schema
- Field descriptions
- Status values and transitions
- Examples for each status

**Provenance Reference (`reference/provenance.md`):**
- Why provenance matters
- Manifest format
- How provenance flows from discovery to fixture
- Citing sources appropriately

---

## Project Management

### Using beads for issue tracking

The testbed uses beads for tracking work:

```bash
# Initialize beads (done once during repo setup)
bd init

# View available work
bd ready

# Create new issue
bd create --title="Implement Rust stress suite" --type=task --priority=2

# Start work
bd update beads-xxx --status=in_progress

# Complete work
bd close beads-xxx

# Sync with git
bd sync
```

### Suggested initial beads issues

**Phase 1: Repository Setup**
- Set up repository structure (Cargo workspace + Python project)
- Implement Rust test harness crate with DiscoveryWriter
- Implement configuration loading (.env) for Rust and Python
- Implement dataset abstraction with CI/local/custom modes
- Create state management system (YAML + SQLite hybrid)
- Create .gitignore and .env.example
- Set up GitHub Actions CI workflow

**Phase 2: Rust Core Suites**
- Implement `stress.rs` - memory and scaling tests
- Implement `malformed.rs` - error recovery discovery
- Implement `discovery.rs` - edge case cataloging

**Phase 3: Encoding and Concurrency**
- Implement `encoding.rs` - international record testing
- Implement `concurrent.rs` - sustained parallel load testing

**Phase 4: Python Compatibility**
- Implement `pymarc_compat/` - real script compatibility
- Implement `encoding/` - encoding through bindings

**Phase 5: Tooling and Scripts**
- Implement `curate_fixtures.py` - initial fixture selection
- Implement `extract_record.py` - record extraction utility
- Implement `validate_fixtures.py` - fixture validation
- Implement `file_issue.py` - issue filing workflow
- Implement `promote_discovery.py` - fixture promotion
- Implement `import_run.py` - run result import
- Implement `rebuild_index.py` - SQLite index rebuild
- Implement `query.py` - discovery querying

**Phase 6: Documentation**
- Set up MkDocs with material theme
- Write introduction and scope documentation
- Write getting started tutorials (CI, local, custom modes)
- Write contribution guides (mrrc PRs, testbed PRs)
- Write reference documentation (discovery format, manifest format)
- Write provenance documentation
- Write state management explanation

**Phase 7: Initial Data**
- Download and verify public datasets
- Run initial fixture curation from LOC
- Validate and commit initial fixtures
- Run first discovery pass against IA Lendable
- Document initial discoveries

No automatic issue creation — humans decide what's worth filing.

---

## Open Questions for Implementation Planning

1. **Holdings data**: Where to source real holdings records? Academic library partnership needed?

2. **International data licensing**: Are national library MARC exports freely usable for testing?

3. **Benchmark baselines**: How to establish and maintain performance baselines?
