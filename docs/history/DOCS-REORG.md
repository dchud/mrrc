# Documentation Reorganization Plan

**Status**: Completed
**Created**: 2026-01-29
**Author**: Documentation planning session

## Executive Summary

This plan proposes reorganizing MRRC's documentation using [Material for MkDocs](https://squidfunk.github.io/mkdocs-material/) to create a navigable documentation site. The current README.md is ~900 lines and serves too many purposes; this reorganization will create focused, discoverable content for different user needs.

## Goals

1. **Clear onboarding** - New users should understand what MRRC does and see working code quickly
2. **Clear navigation** - Users should find what they need without reading everything
3. **Separated concerns** - Quickstart vs. tutorials vs. reference vs. internals
4. **Maintainability** - Easier to update individual sections without cascading changes
5. **Accessible hosting** - Material for MkDocs with light/dark theme toggle (defaults to light, respects system preference), search, and GitHub Pages hosting

## Style and Tone Guidelines

The documentation should be **factual, accurate, and helpful** - not promotional.

### General Principles

1. **Be accurate, not aspirational** - State what the library actually does, not what we wish it did
2. **Avoid marketing language** - No superlatives ("blazing fast", "revolutionary"), no hype
3. **Acknowledge limitations** - If something requires workarounds or has caveats, say so
4. **Let code speak** - Working examples are more convincing than claims

### Specific Guidelines

| Instead of... | Write... |
|---------------|----------|
| "Drop-in replacement for pymarc" | "pymarc-compatible API with minor differences" |
| "Blazing fast performance" | "~4x faster than pymarc in benchmarks (see methodology)" |
| "Seamless migration" | "Migration requires updating imports and minor API adjustments" |
| "Full support for X" | "Supports X" (or "Partial support for X" if applicable) |
| "Simply do X" | "To do X:" (avoid implying things are simple) |

### Accuracy Checklist for Claims

Before making performance or compatibility claims, verify:

- [ ] Benchmark methodology is documented and reproducible
- [ ] Numbers include context (hardware, dataset size, warm-up)
- [ ] Comparisons are fair (same task, same conditions)
- [ ] Limitations and edge cases are noted
- [ ] pymarc compatibility differences are documented in migration guide

### What to Avoid

- Emoji in technical documentation (unless showing output that contains them)
- Exclamation points for emphasis
- Words like "just", "simply", "easily" (they dismiss complexity)
- Vague claims without evidence ("much faster", "highly compatible")
- Comparisons that cherry-pick favorable scenarios

## Current State Analysis

### README.md Problems

| Issue | Lines | Problem |
|-------|-------|---------|
| Too long | 898 | Overwhelming for new users |
| Mixed audiences | - | Rust users, Python users, contributors all reading same doc |
| Duplicated content | ~200 | Same examples appear in README and tutorials |
| Deep technical detail | ~300 | Leader struct definitions, encoding internals in quickstart |
| Buried key info | - | Performance numbers, format matrix hard to find |

### Existing docs/ Structure

**Keep as-is (good organization):**
- `docs/design/` - Internal proposals and design decisions
- `docs/history/` - Historical context and completed work
- `docs/benchmarks/` - Performance data and methodology

**Needs reorganization:**
- Root-level guides are flat and hard to navigate
- No clear progression from beginner to advanced
- Tutorials exist but aren't linked well
- No API reference documentation

## Proposed Structure

```
docs/
├── index.md                    # New landing page (replaces docs/README.md)
├── getting-started/
│   ├── index.md               # Overview of getting started
│   ├── installation.md        # From INSTALLATION_GUIDE.md (condensed)
│   ├── quickstart-python.md   # 5-minute Python intro
│   └── quickstart-rust.md     # 5-minute Rust intro
├── tutorials/
│   ├── index.md               # Tutorial overview
│   ├── python/
│   │   ├── reading-records.md
│   │   ├── writing-records.md
│   │   ├── format-conversion.md
│   │   ├── querying-fields.md
│   │   └── concurrency.md
│   └── rust/
│       ├── reading-records.md
│       ├── writing-records.md
│       ├── format-conversion.md
│       ├── querying-fields.md
│       └── concurrency.md
├── guides/
│   ├── index.md               # Guide overview
│   ├── format-selection.md    # From FORMAT_SELECTION_GUIDE.md
│   ├── migration-from-pymarc.md  # From MIGRATION_GUIDE.md
│   ├── streaming-large-files.md  # From STREAMING_GUIDE.md
│   ├── threading-python.md    # From THREADING.md
│   ├── performance-tuning.md  # From PERFORMANCE.md
│   ├── query-dsl.md           # From QUERY_DSL.md
│   └── bibframe-conversion.md # New, extracted from README
├── reference/
│   ├── index.md               # Reference overview
│   ├── python-api.md          # Generated + hand-written API docs
│   ├── rust-api.md            # Links to docs.rs + key types
│   ├── formats.md             # Format support matrix (detailed)
│   ├── marc-primer.md         # MARC record structure explanation
│   └── encoding.md            # MARC-8 and UTF-8 details
├── examples/
│   ├── index.md               # Examples overview with table
│   ├── python/                # Annotated example walkthroughs
│   │   └── *.md
│   └── rust/
│       └── *.md
├── contributing/
│   ├── index.md               # How to contribute
│   ├── development-setup.md   # Local dev environment
│   ├── testing.md             # Running tests
│   ├── release-procedure.md   # From RELEASE_PROCEDURE.md
│   └── architecture.md        # From ARCHITECTURE.md
├── design/                    # Internal design docs
│   ├── index.md              # NEW: Index of design proposals
│   └── *.md
├── history/                   # Historical archive
│   ├── index.md              # NEW: Index of historical docs
│   └── *.md
└── benchmarks/                # Keep existing
    └── *.md
```

## New Root README.md

The repository README.md should be shortened to ~150-200 lines:

```markdown
# MRRC: MARC Rust Crate

[badges]

A Rust library for reading, writing, and manipulating MARC bibliographic records, with Python bindings.

## Features

- Reads and writes ISO 2709 (MARC21) binary format
- Python bindings with pymarc-compatible API (minor differences documented)
- Multiple serialization formats: JSON, XML, MARCJSON, CSV, Protobuf, Arrow, and others
- MARC-8 and UTF-8 character encoding support
- Benchmarked at ~4x pymarc throughput in Python, ~1M records/sec in Rust

## Installation

**Python** (3.9+):
```bash
pip install mrrc
# or with uv:
uv add mrrc
```

**Rust**:
```bash
cargo add mrrc
```

## Example

**Python:**
```python
from mrrc import MARCReader

with open("records.mrc", "rb") as f:
    for record in MARCReader(f):
        print(record.title())
```

**Rust:**
```rust
use mrrc::MarcReader;
use std::fs::File;

let file = File::open("records.mrc")?;
let mut reader = MarcReader::new(file);
while let Some(record) = reader.read_record()? {
    if let Some(title) = record.title() {
        println!("{}", title);
    }
}
```

## Documentation

- [Getting Started](https://dchud.github.io/mrrc/getting-started/)
- [Python Tutorial](https://dchud.github.io/mrrc/tutorials/python/)
- [Rust Tutorial](https://dchud.github.io/mrrc/tutorials/rust/)
- [API Reference](https://dchud.github.io/mrrc/reference/)
- [Migration from pymarc](https://dchud.github.io/mrrc/guides/migration-from-pymarc/)

## Format Support

| Format | Read | Write | Notes |
|--------|------|-------|-------|
| ISO 2709 | Yes | Yes | Standard MARC binary |
| JSON | Yes | Yes | Generic JSON |
| XML | Yes | Yes | MARCXML |
| Protobuf | Yes | Yes | Feature-gated |
| Arrow | Yes | Yes | Feature-gated |

[Full format matrix](https://dchud.github.io/mrrc/reference/formats/)

## Status

Experimental. The Python API aims for pymarc compatibility but has some differences; see the [migration guide](https://dchud.github.io/mrrc/guides/migration-from-pymarc/). Rust APIs may change between minor versions.

## License

MIT

## Links

- [Documentation](https://dchud.github.io/mrrc/)
- [PyPI](https://pypi.org/project/mrrc/)
- [crates.io](https://crates.io/crates/mrrc/)
- [GitHub](https://github.com/dchud/mrrc)
```

## Content Migration Plan

### Phase 1: Infrastructure Setup

1. **Add MkDocs configuration**
   - Create `mkdocs.yml` with Material theme (light/dark toggle)
   - Configure navigation structure
   - Set up GitHub Actions for deployment
   - Test locally before committing:
     ```bash
     mkdocs serve          # or: uv run mkdocs serve
     ```

2. **Create docs scaffold**
   - Create directory structure
   - Add placeholder index.md files (including design/index.md and history/index.md)
     - **Note**: `docs/history/README.md` already has comprehensive content; adapt it for `history/index.md`
   - Set up navigation
   - Verify build succeeds:
     ```bash
     mkdocs build --strict  # or: uv run mkdocs build --strict
     ```

### Phase 2: Landing Page and Getting Started

1. **Write new `docs/index.md`** (landing page)
   - Brief, factual intro
   - Feature list (not feature "highlights")
   - Language tabs for Python/Rust
   - Links to next steps

2. **Create getting-started section**
   - Condense INSTALLATION_GUIDE.md
   - Write quickstart-python.md
   - Write quickstart-rust.md

### Phase 3: Tutorials

1. **Python tutorials** (from PYTHON_TUTORIAL.md + README sections)
   - Break into focused pages
   - Add progressive complexity
   - Include "next steps" links

2. **Rust tutorials** (from RUST_TUTORIAL.md + README sections)
   - Parallel structure to Python
   - Include builder pattern examples
   - Cover error handling

### Phase 4: Guides and Reference

1. **Migrate existing guides**
   - FORMAT_SELECTION_GUIDE.md → guides/format-selection.md
   - MIGRATION_GUIDE.md → guides/migration-from-pymarc.md
   - STREAMING_GUIDE.md → guides/streaming-large-files.md
   - THREADING.md → guides/threading-python.md
   - PERFORMANCE.md → guides/performance-tuning.md
   - QUERY_DSL.md → guides/query-dsl.md

2. **Create new reference docs**
   - Python API reference (type stubs + descriptions)
   - Rust API reference (links to docs.rs + key examples)
   - Detailed format matrix
   - MARC primer for non-librarians

### Phase 5: Examples Section

1. **Create examples index**
   - Table mapping tasks to examples
   - Difficulty indicators
   - Language indicators

2. **Write example walkthroughs**
   - Annotated versions of key examples
   - Explain the reasoning, not just the code
   - Link to related tutorials

### Phase 6: Contributing Section

1. **Migrate contributor docs**
   - RELEASE_PROCEDURE.md → contributing/release-procedure.md
   - ARCHITECTURE.md → contributing/architecture.md

2. **Create new contributor docs**
   - Development setup guide
   - Testing guide
   - Code style guide

### Phase 7: Final Polish

1. **Update root README.md**
   - Reduce to ~150-200 lines
   - Link to hosted docs for everything else

2. **Cross-linking pass**
   - Ensure all pages link to related content
   - Add "See also" sections
   - Verify no dead links

3. **Accuracy review**
   - Verify all claims against current benchmarks
   - Check pymarc compatibility statements against migration guide
   - Ensure version numbers are current

## MkDocs Configuration

```yaml
# mkdocs.yml
site_name: MRRC Documentation
site_url: https://dchud.github.io/mrrc/
site_description: MARC library for Rust and Python
repo_url: https://github.com/dchud/mrrc
repo_name: dchud/mrrc

theme:
  name: material
  palette:
    # Light mode (default) - listed first so it's the default
    - media: "(prefers-color-scheme: light)"
      scheme: default
      primary: indigo
      accent: indigo
      toggle:
        icon: material/brightness-7
        name: Switch to dark mode
    # Dark mode - auto-activates if user's system prefers dark
    - media: "(prefers-color-scheme: dark)"
      scheme: slate
      primary: indigo
      accent: indigo
      toggle:
        icon: material/brightness-4
        name: Switch to light mode
  features:
    - navigation.tabs
    - navigation.sections
    - navigation.expand
    - navigation.top
    - search.highlight
    - search.share
    - content.code.copy
    - content.tabs.link

plugins:
  - search
  - mkdocstrings:  # For API docs if needed
      handlers:
        python:
          paths: [src-python]

markdown_extensions:
  - pymdownx.highlight:
      anchor_linenums: true
  - pymdownx.superfences
  - pymdownx.tabbed:
      alternate_style: true
  - admonitions
  - tables
  - toc:
      permalink: true

nav:
  - Home: index.md
  - Getting Started:
    - getting-started/index.md
    - Installation: getting-started/installation.md
    - Python Quickstart: getting-started/quickstart-python.md
    - Rust Quickstart: getting-started/quickstart-rust.md
  - Tutorials:
    - tutorials/index.md
    - Python:
      - tutorials/python/reading-records.md
      - tutorials/python/writing-records.md
      - tutorials/python/format-conversion.md
      - tutorials/python/querying-fields.md
      - tutorials/python/concurrency.md
    - Rust:
      - tutorials/rust/reading-records.md
      - tutorials/rust/writing-records.md
      - tutorials/rust/format-conversion.md
      - tutorials/rust/querying-fields.md
      - tutorials/rust/concurrency.md
  - Guides:
    - guides/index.md
    - Format Selection: guides/format-selection.md
    - Migration from pymarc: guides/migration-from-pymarc.md
    - Streaming Large Files: guides/streaming-large-files.md
    - Threading (Python): guides/threading-python.md
    - Performance Tuning: guides/performance-tuning.md
    - Query DSL: guides/query-dsl.md
    - BIBFRAME Conversion: guides/bibframe-conversion.md
  - Reference:
    - reference/index.md
    - Python API: reference/python-api.md
    - Rust API: reference/rust-api.md
    - Format Support: reference/formats.md
    - MARC Primer: reference/marc-primer.md
    - Character Encoding: reference/encoding.md
  - Examples:
    - examples/index.md
  - Contributing:
    - contributing/index.md
    - Development Setup: contributing/development-setup.md
    - Testing: contributing/testing.md
    - Release Procedure: contributing/release-procedure.md
    - Architecture: contributing/architecture.md
  - Benchmarks: benchmarks/RESULTS.md
  - Design: design/index.md
  - History: history/index.md
```

## GitHub Pages Deployment

### GitHub Actions Workflow

Create `.github/workflows/docs.yml`:

```yaml
name: Deploy Documentation

on:
  push:
    branches: [main]
    paths:
      - 'docs/**'
      - 'mkdocs.yml'
      - 'README.md'
  workflow_dispatch:

permissions:
  contents: read
  pages: write
  id-token: write

concurrency:
  group: "pages"
  cancel-in-progress: false

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Set up Python
        uses: actions/setup-python@v5
        with:
          python-version: '3.12'

      - name: Install uv
        uses: astral-sh/setup-uv@v4

      - name: Install dependencies
        run: uv pip install --system mkdocs-material mkdocstrings[python] pymdown-extensions

      - name: Build documentation
        run: mkdocs build --strict

      - name: Upload artifact
        uses: actions/upload-pages-artifact@v3
        with:
          path: site/

  deploy:
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    runs-on: ubuntu-latest
    needs: build
    steps:
      - name: Deploy to GitHub Pages
        id: deployment
        uses: actions/deploy-pages@v4
```

### Repository Settings

1. Go to repository Settings → Pages
2. Set Source to "GitHub Actions"
3. The workflow will handle deployment automatically

## Success Criteria

### For New Users
- [ ] Can understand what MRRC does in 30 seconds from landing page
- [ ] Can install and run first example in under 5 minutes
- [ ] Can find language-specific content (Python vs Rust) easily
- [ ] Search returns relevant results for common queries

### For Existing Users
- [ ] Can find API reference quickly
- [ ] Can find troubleshooting/performance guides
- [ ] Examples are easy to discover and copy
- [ ] Migration guide clearly lists pymarc differences

### For Contributors
- [ ] Development setup is clear
- [ ] Release procedure is findable
- [ ] Architecture docs explain key decisions

### Technical
- [ ] All links work (no 404s)
- [ ] Site builds without warnings
- [ ] Mobile-responsive
- [ ] Search indexes all content
- [ ] Deploys automatically on merge to main
- [ ] Light/dark theme toggle works; respects system preference by default

## Content Guidelines

### Writing Style
- **Factual**: State what things do, not how great they are
- **Concise**: Get to the point quickly
- **Task-oriented**: Focus on what users want to accomplish
- **Code-first**: Show working code, then explain
- **Honest**: Acknowledge limitations and differences

### Code Examples
- **Complete**: Examples should run without modification
- **Annotated**: Explain non-obvious lines
- **Tested**: All examples should be validated
- **Copyable**: Use code copy button feature

### Page Structure
```markdown
# Page Title

Brief intro (1-2 sentences max).

## Prerequisites (if needed)

## Main Content

### Subsection

## Limitations or Caveats (if applicable)

## Next Steps

- Link to related tutorial
- Link to reference docs
- Link to examples
```

## Migration Checklist

### Files to Create
- [ ] `mkdocs.yml`
- [ ] `docs/index.md` (landing page)
- [ ] `docs/getting-started/index.md`
- [ ] `docs/getting-started/installation.md`
- [ ] `docs/getting-started/quickstart-python.md`
- [ ] `docs/getting-started/quickstart-rust.md`
- [ ] `docs/tutorials/index.md`
- [ ] `docs/tutorials/python/*.md` (5 files)
- [ ] `docs/tutorials/rust/*.md` (5 files)
- [ ] `docs/guides/index.md`
- [ ] `docs/guides/bibframe-conversion.md`
- [ ] `docs/reference/index.md`
- [ ] `docs/reference/python-api.md`
- [ ] `docs/reference/rust-api.md`
- [ ] `docs/reference/formats.md`
- [ ] `docs/reference/marc-primer.md`
- [ ] `docs/reference/encoding.md`
- [ ] `docs/examples/index.md`
- [ ] `docs/contributing/index.md`
- [ ] `docs/contributing/development-setup.md`
- [ ] `docs/contributing/testing.md`
- [ ] `docs/design/index.md` (index of design proposals)
- [ ] `docs/history/index.md` (index of historical docs)
- [ ] `.github/workflows/docs.yml`

### Files to Migrate (rename/move)
- [ ] `docs/INSTALLATION_GUIDE.md` → `docs/getting-started/installation.md`
- [ ] `docs/PYTHON_TUTORIAL.md` → split into `docs/tutorials/python/*.md`
- [ ] `docs/RUST_TUTORIAL.md` → split into `docs/tutorials/rust/*.md`
- [ ] `docs/FORMAT_SELECTION_GUIDE.md` → `docs/guides/format-selection.md`
- [ ] `docs/MIGRATION_GUIDE.md` → `docs/guides/migration-from-pymarc.md`
- [ ] `docs/STREAMING_GUIDE.md` → `docs/guides/streaming-large-files.md`
- [ ] `docs/THREADING.md` → `docs/guides/threading-python.md`
- [ ] `docs/PERFORMANCE.md` → `docs/guides/performance-tuning.md`
- [ ] `docs/QUERY_DSL.md` → `docs/guides/query-dsl.md`
- [ ] `docs/RELEASE_PROCEDURE.md` → `docs/contributing/release-procedure.md`
- [ ] `docs/ARCHITECTURE.md` → `docs/contributing/architecture.md`

### Files to Update
- [ ] `README.md` - Reduce to ~150-200 lines
- [ ] `docs/benchmarks/RESULTS.md` - Minor updates for navigation

### Files to Keep As-Is (add index pages for navigation)
- [ ] `docs/design/*.md` - Internal design docs; add `docs/design/index.md` with categorized links
- [ ] `docs/history/*.md` - Historical archive; add `docs/history/index.md` with chronological/topical index
  - **Note**: Existing `docs/history/README.md` is comprehensive and well-organized; adapt it for `index.md` rather than starting from scratch

### Files to Merge or Relocate
- [ ] `docs/MEMORY_SAFETY.md` - Link from contributing/testing.md
- [ ] `docs/CONCURRENCY.md` - Merge relevant content into guides/threading-python.md

### Post-Migration Cleanup
After verifying the new structure works:
- Delete migrated source files (INSTALLATION_GUIDE.md, PYTHON_TUTORIAL.md, etc.)
- Keep design/ and history/ directories as archival reference
- Update any external links that pointed to old file locations

## Estimated Effort

| Phase | New Content | Migration | Review | Total |
|-------|-------------|-----------|--------|-------|
| 1. Infrastructure | 2h | - | 1h | 3h |
| 2. Landing + Getting Started | 4h | 2h | 2h | 8h |
| 3. Tutorials | 8h | 4h | 3h | 15h |
| 4. Guides + Reference | 6h | 4h | 2h | 12h |
| 5. Examples | 4h | 2h | 1h | 7h |
| 6. Contributing | 3h | 2h | 1h | 6h |
| 7. Final Polish | 2h | 2h | 2h | 6h |
| **Total** | **29h** | **16h** | **12h** | **57h** |

## Open Questions

Decisions to make before or during implementation:

1. **API Documentation Generation**: Should we use mkdocstrings to auto-generate Python API docs from docstrings, or maintain hand-written docs?
   - *Recommendation*: Start with hand-written; add mkdocstrings later if maintenance burden is high

2. **Versioned Documentation**: Do we need versioned docs (e.g., v0.5, v0.6) or is latest-only sufficient for now?
   - *Recommendation*: Latest-only until v1.0; versioning adds complexity

3. **Examples Testing**: Should we add CI that validates all code examples in docs compile/run?
   - *Recommendation*: Yes, but as a follow-up task after initial migration

Deferred (not needed for initial migration):

4. **Internationalization**: Not planned; English-only for now

5. **Search Analytics**: Not needed initially; can add later if useful

## Next Steps

1. Review this plan and gather feedback
2. Finalize decisions on open questions
3. Create beads epic with tasks for each phase
4. Begin implementation with Phase 1 (infrastructure)

---

## Appendix: Material for MkDocs Features to Use

### Theme Features
- **Light/dark toggle**: User can switch themes manually
- **System preference detection**: Automatically uses user's OS preference
- **Persistent preference**: Remembers user's choice across sessions

### Content Features
- **Admonitions**: Note, warning, tip, info boxes
- **Code annotations**: Numbered explanations for code
- **Content tabs**: Python/Rust side-by-side
- **Code copy button**: Copying of examples

### Navigation Features
- **Tabs**: Top-level navigation
- **Sections**: Expandable sidebar groups
- **Table of contents**: Per-page navigation
- **Search**: Full-text search with highlighting

### Integrations
- **GitHub**: Edit on GitHub links
- **Analytics**: Optional (if we want usage data)

## Appendix: Landing Page Template

This template shows the recommended structure for `docs/index.md`. It demonstrates the use of Material for MkDocs features (content tabs, admonitions) while following our style guidelines.

```markdown
# MRRC

A Rust library for MARC bibliographic records, with Python bindings.

## What MRRC Does

- Reads and writes ISO 2709 (MARC21) binary format
- Provides Python bindings with a pymarc-compatible API
- Supports multiple serialization formats (JSON, XML, Protobuf, Arrow, etc.)
- Handles MARC-8 and UTF-8 character encodings

## Performance

In benchmarks (see [methodology](benchmarks/RESULTS.md)):

- Python: ~4x throughput compared to pymarc
- Rust: ~1M records/sec

## Quick Example

=== "Python"

    ```python
    from mrrc import MARCReader

    with open("records.mrc", "rb") as f:
        for record in MARCReader(f):
            print(record.title())
    ```

=== "Rust"

    ```rust
    use mrrc::MarcReader;
    use std::fs::File;

    let file = File::open("records.mrc")?;
    let mut reader = MarcReader::new(file);
    while let Some(record) = reader.read_record()? {
        if let Some(title) = record.title() {
            println!("{}", title);
        }
    }
    ```

## Getting Started

- [Installation](getting-started/installation.md)
- [Python Quickstart](getting-started/quickstart-python.md)
- [Rust Quickstart](getting-started/quickstart-rust.md)

## pymarc Users

MRRC's Python API is similar to pymarc but not identical. See the [migration guide](guides/migration-from-pymarc.md) for specific differences.

## Status

This library is experimental. APIs may change between versions.
```
