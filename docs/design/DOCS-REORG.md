# Documentation Reorganization Plan

**Status**: Draft for review
**Created**: 2026-01-29
**Author**: Documentation planning session

## Executive Summary

This plan proposes reorganizing MRRC's documentation using [Material for MkDocs](https://squidfunk.github.io/mkdocs-material/) to create a professional, navigable documentation site. The current README.md is ~900 lines and serves too many purposes; this reorganization will create focused, discoverable content for different user journeys.

## Goals

1. **Friendly onboarding** - New users should understand what MRRC is and see working code in under 2 minutes
2. **Clear navigation** - Users should find what they need without reading everything
3. **Separated concerns** - Quickstart vs. tutorials vs. reference vs. internals
4. **Maintainability** - Easier to update individual sections without cascading changes
5. **Professional presentation** - Material for MkDocs with light theme, search, and GitHub Pages hosting

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
├── design/                    # Keep existing (internal)
│   └── *.md
├── history/                   # Keep existing (archival)
│   └── *.md
└── benchmarks/                # Keep existing
    └── *.md
```

## New Root README.md

The repository README.md should be dramatically shortened to ~150-200 lines:

```markdown
# MRRC: MARC Rust Crate

[badges]

A high-performance Rust library for MARC bibliographic records with Python bindings.

## Why MRRC?

- **Fast**: ~4x faster than pymarc, ~1M records/sec in pure Rust
- **Compatible**: Drop-in replacement for pymarc API
- **Flexible**: 10+ serialization formats including BIBFRAME

## Quick Install

**Python**: `pip install mrrc`
**Rust**: `cargo add mrrc`

## Quick Example

[Single 10-line Python example]
[Single 10-line Rust example]

## Documentation

📖 **[Full Documentation](https://dchud.github.io/mrrc/)**

- [Getting Started](link) - Installation and first steps
- [Python Tutorial](link) - Complete Python guide
- [Rust Tutorial](link) - Complete Rust guide
- [API Reference](link) - Detailed API documentation
- [Examples](link) - Working code examples

## Format Support

[Condensed 5-row table with links to full matrix]

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
   - Create `mkdocs.yml` with Material theme (light mode)
   - Configure navigation structure
   - Set up GitHub Actions for deployment

2. **Create docs scaffold**
   - Create directory structure
   - Add placeholder index.md files
   - Set up navigation

### Phase 2: Landing Page and Getting Started

1. **Write new `docs/index.md`** (landing page)
   - Brief intro (50 words)
   - Feature highlights with icons
   - Language tabs for Python/Rust
   - Links to next steps

2. **Create getting-started section**
   - Condense INSTALLATION_GUIDE.md
   - Write quickstart-python.md (5-minute guide)
   - Write quickstart-rust.md (5-minute guide)

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
   - Explain why, not just what
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
   - Dramatic reduction to ~150-200 lines
   - Link to hosted docs for everything else

2. **Cross-linking pass**
   - Ensure all pages link to related content
   - Add "See also" sections
   - Verify no dead links

3. **Search optimization**
   - Add appropriate titles and descriptions
   - Ensure code blocks are searchable

## MkDocs Configuration

```yaml
# mkdocs.yml
site_name: MRRC Documentation
site_url: https://dchud.github.io/mrrc/
site_description: High-performance MARC library for Rust and Python
repo_url: https://github.com/dchud/mrrc
repo_name: dchud/mrrc

theme:
  name: material
  palette:
    scheme: default  # Light theme
    primary: indigo
    accent: indigo
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

      - name: Install dependencies
        run: |
          pip install mkdocs-material mkdocstrings[python] pymdown-extensions

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
- [ ] Can understand what MRRC is in 30 seconds from landing page
- [ ] Can install and run first example in under 5 minutes
- [ ] Can find language-specific content (Python vs Rust) easily
- [ ] Search returns relevant results for common queries

### For Existing Users
- [ ] Can find API reference quickly
- [ ] Can find troubleshooting/performance guides
- [ ] Examples are easy to discover and copy
- [ ] Migration guide is prominent for pymarc users

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

## Content Guidelines

### Writing Style
- **Concise**: Get to the point quickly
- **Task-oriented**: Focus on what users want to accomplish
- **Code-first**: Show working code, then explain
- **Progressive**: Simple → intermediate → advanced

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
- [ ] `README.md` - Dramatic reduction to ~150-200 lines
- [ ] `docs/benchmarks/RESULTS.md` - Minor updates for navigation

### Files to Keep As-Is
- [ ] `docs/design/*.md` - Internal design docs
- [ ] `docs/history/*.md` - Historical archive
- [ ] `docs/MEMORY_SAFETY.md` - Link from contributing
- [ ] `docs/CONCURRENCY.md` - Merge into guides or reference

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

1. **API Documentation Generation**: Should we use mkdocstrings to auto-generate Python API docs from docstrings, or maintain hand-written docs?

2. **Versioned Documentation**: Do we need versioned docs (e.g., v0.5, v0.6) or is latest-only sufficient for now?

3. **Internationalization**: Any plans for non-English documentation?

4. **Examples Testing**: Should we add CI that validates all code examples in docs compile/run?

5. **Search Analytics**: Do we want to track what users search for to identify documentation gaps?

## Next Steps

1. Review this plan and gather feedback
2. Finalize decisions on open questions
3. Create beads epic with tasks for each phase
4. Begin implementation with Phase 1 (infrastructure)

---

## Appendix: Material for MkDocs Features to Use

### Content Features
- **Admonitions**: Note, warning, tip, info boxes
- **Code annotations**: Numbered explanations for code
- **Content tabs**: Python/Rust side-by-side
- **Code copy button**: Easy copying of examples

### Navigation Features
- **Tabs**: Top-level navigation
- **Sections**: Expandable sidebar groups
- **Table of contents**: Per-page navigation
- **Search**: Full-text search with highlighting

### Integrations
- **GitHub**: Edit on GitHub links
- **Social cards**: Auto-generated for sharing
- **Analytics**: Optional Google Analytics

## Appendix: Example Landing Page

```markdown
# MRRC

**High-performance MARC library for Rust and Python**

---

<div class="grid cards" markdown>

-   :material-speedometer:{ .lg .middle } **Fast**

    ---

    ~4x faster than pymarc in Python, ~1M records/sec in pure Rust

-   :material-swap-horizontal:{ .lg .middle } **Compatible**

    ---

    Drop-in replacement for pymarc API - migrate with one import change

-   :material-file-multiple:{ .lg .middle } **Flexible**

    ---

    10+ serialization formats including BIBFRAME, Arrow, and MessagePack

-   :material-language-python:{ .lg .middle } **Dual Language**

    ---

    First-class support for both Python and Rust developers

</div>

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
    for record in MarcReader::new(file) {
        println!("{}", record?.title());
    }
    ```

## Get Started

<div class="grid cards" markdown>

-   [:material-download: **Installation**](getting-started/installation.md)

    Install mrrc for Python or Rust

-   [:material-rocket-launch: **Quickstart**](getting-started/quickstart-python.md)

    Your first MARC program in 5 minutes

-   [:material-school: **Tutorials**](tutorials/index.md)

    Learn mrrc step by step

-   [:material-book-open-variant: **API Reference**](reference/index.md)

    Detailed API documentation

</div>
```
