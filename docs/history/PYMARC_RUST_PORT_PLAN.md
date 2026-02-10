# PyMARC Rust Port: Project Plan

## Overview

This document outlines the comprehensive plan for porting the PyMARC library (a Python library for working with bibliographic data in MARC21 format) to Rust. The goal is to create a comparable API while leveraging Rust's performance, safety, and type system.

**Reference**: https://gitlab.com/pymarc/pymarc

## Project Assessment

### PyMARC Library Structure

The Python library consists of the following core modules:

1. **record.py** - Record class for MARC bibliographic records
2. **field.py** - Field class for MARC fields and indicators
3. **leader.py** - Leader class for record metadata
4. **reader.py** - Binary MARC record reader (ISO 2709 format)
5. **writer.py** - Binary MARC record writer (ISO 2709 format)
6. **marcxml.py** - MARCXML serialization/deserialization
7. **marcjson.py** - MARCJSON serialization/deserialization
8. **marc8.py** - MARC-8 character encoding support
9. **marc8_mapping.py** - MARC-8 character mappings
10. **constants.py** - MARC field constants and definitions
11. **exceptions.py** - Custom exception classes

### Key Features to Port

- ✅ Read and write binary MARC records (ISO 2709 format)
- ✅ Parse and manipulate MARC fields and subfields
- ✅ Handle multiple character encodings (MARC-8, UTF-8)
- ✅ Serialize to/from MARCXML and MARCJSON
- ✅ Helper properties for common fields (title, author, ISBN, etc.)
- ✅ Record leader parsing and manipulation
- ✅ Extensive test suite with real MARC data

### Scope & Constraints

**Data Loss**: None - all MARC data must be preserved with 100% fidelity
**API Changes**: Acceptable to make API more Rust-idiomatic (iterators, builders, error handling)
**Performance**: Rust should provide performance benefits over Python

## Porting Strategy

### Phase 1: Foundation (High Priority)
Establish the core data structures and basic parsing/writing capabilities.

**Epics**:
- **MARC record parsing and data structures** (mrrc-3m0)
  - Implement Record structure and Leader parsing
  - Implement Field structure and Subfield handling
  - Binary MARC record reader (ISO 2709 format)
  - Binary MARC record writer (ISO 2709 format)
  - Record helper properties (title, author, ISBN, etc.)

- **Documentation and API design** (mrrc-734)
  - Set up Cargo project with dependencies (CRITICAL/BLOCKING)
  - Rust-friendly API design (iterators, builder patterns)
  - Write library API documentation and README

### Phase 2: Advanced Features (High Priority)
Implement serialization formats and character encoding.

**Epics**:
- **Serialization formats** (mrrc-8p7)
  - MARCXML reader and writer
  - MARCJSON reader and writer

- **Character encoding support** (mrrc-vx7)
  - MARC-8 character encoding implementation (complex)
  - UTF-8 character encoding handling

### Phase 3: Validation (High Priority)
Port test suite and verify correctness.

**Epics**:
- **Test suite and example data** (mrrc-n03)
  - Port pymarc test data files to Rust project
  - Port pymarc test suite to Rust (unit tests)

## Technical Decisions

### Recommended Dependencies

- **XML**: `quick-xml` or `roxmltree` for fast, zero-copy XML parsing
- **JSON**: `serde_json` for JSON serialization/deserialization
- **Encoding**: `encoding_rs` for character encoding (supports MARC-8 if available, otherwise custom implementation)
- **Error Handling**: Custom error types with `thiserror` or similar
- **Testing**: Rust's built-in test framework with `.binary` files from pymarc test suite

### API Design Principles

1. **Zero-Copy Where Possible**: Leverage Rust's ownership system for efficiency
2. **Iterator-Based**: Use Rust iterators for field/subfield iteration instead of returning vectors
3. **Builder Pattern**: For constructing complex records and fields
4. **Result-Based Errors**: Use `Result<T, Error>` instead of exceptions
5. **Struct-Based**: Use Rust structs and enums instead of Python's dynamic typing
6. **Memory Efficiency**: Rust's move semantics vs Python's reference counting

### Character Encoding Challenge

MARC-8 is a complex encoding with multi-byte character sequences and escape sequences. Options:

1. **Custom Implementation**: Port the marc8.py and marc8_mapping.py logic
2. **External Library**: Check if `encoding_rs` or similar has MARC-8 support
3. **Hybrid Approach**: Use existing library for well-supported encodings, custom for MARC-8

The marc8_mapping.py file contains comprehensive character mappings that will need to be ported.

## Execution Order

### Week 1: Foundational Setup
1. Set up Cargo project with Cargo.toml and dependencies
2. Create module structure (lib.rs with mod declarations)
3. Implement Record, Field, Leader, and Subfield structs
4. Begin binary format reader (ISO 2709 parsing)

### Week 2: Core Reading/Writing
1. Complete binary reader implementation
2. Implement binary writer
3. Add test data from pymarc
4. Begin porting core unit tests

### Week 3: Encoding & Serialization
1. Implement character encoding (UTF-8, MARC-8)
2. Implement MARCXML reader/writer
3. Implement MARCJSON reader/writer
4. Add more comprehensive tests

### Week 4: Polish & Validation
1. Add helper properties (title, author, ISBN, etc.)
2. Complete test porting
3. Documentation and API design
4. Performance benchmarking

## Success Criteria

- ✅ All core MARC reading/writing functionality works
- ✅ Serialization to MARCXML and MARCJSON works
- ✅ Character encoding (MARC-8 and UTF-8) fully implemented
- ✅ Test suite passes all ported tests
- ✅ API is idiomatic Rust
- ✅ No data loss when round-tripping records
- ✅ Performance is comparable to or better than Python implementation

## Known Challenges

1. **MARC-8 Encoding**: Complex multi-byte sequences and escape sequences
2. **API Compatibility**: Python and Rust have different idioms; need sensible translations
3. **XML/JSON Libraries**: Need to evaluate best options for Rust
4. **Test Data**: Binary MARC files need to work across platforms
5. **Large Record Handling**: Ensure streaming readers work for large MARC files

## Repository Structure

```
mrrc/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── record.rs
│   ├── field.rs
│   ├── leader.rs
│   ├── reader.rs
│   ├── writer.rs
│   ├── marcxml.rs
│   ├── marcjson.rs
│   ├── encoding/
│   │   ├── mod.rs
│   │   ├── utf8.rs
│   │   └── marc8.rs
│   ├── constants.rs
│   └── error.rs
├── tests/
│   └── integration_tests.rs
├── data/
│   └── (test data from pymarc)
└── docs/
    └── API.md
```

## Next Steps

1. ✅ Onboard bd issue tracking system
2. ✅ Create epics and tasks in beads
3. Next: Initialize Cargo project and begin implementation with mrrc-734.1 (Set up Cargo project)
