# Format Selection Guide

This guide helps you choose the right serialization format for your MARC data based on your use case.

## Quick Decision Tree

```
What's your primary goal?
│
├─ Exchange with other library systems?
│  └─ ISO 2709 (.mrc) - Universal compatibility
│
├─ Build a REST API?
│  ├─ Human-readable responses? → JSON or MARCJSON
│  └─ Smaller payloads? → MessagePack or Protobuf
│
├─ Analytics and data science?
│  └─ Arrow (.arrow) - DuckDB, Polars, Pandas integration
│
├─ Streaming / real-time processing?
│  ├─ Kafka / data lakes? → Avro
│  └─ Memory-constrained? → FlatBuffers
│
├─ Long-term archival?
│  ├─ Standards compliance required? → CBOR (RFC 7049)
│  └─ Wide tool support? → ISO 2709 or JSON
│
└─ Spreadsheet export?
   └─ CSV
```

## Format Comparison

### Core Formats (Always Available)

| Format | Speed | Size | Compatibility | Best For |
|--------|-------|------|---------------|----------|
| **ISO 2709** | Fast | Compact | Universal | Library system interchange |
| **Protobuf** | Very Fast | Very Compact | Good | APIs, microservices |
| **JSON** | Medium | Large | Universal | Web APIs, debugging |
| **MARCJSON** | Medium | Large | LOC Standard | Standards-compliant JSON |
| **XML** | Slow | Very Large | Universal | MARCXML systems |
| **CSV** | Fast | Medium | Universal | Spreadsheets, simple tools |

### Feature-Gated Formats

These formats require feature flags in Rust or optional dependencies in Python.

| Format | Speed | Size | Compatibility | Best For |
|--------|-------|------|---------------|----------|
| **Arrow** | Very Fast | Very Compact | Growing | Analytics, data science |
| **MessagePack** | Very Fast | Compact | 50+ languages | REST APIs, IPC |
| **FlatBuffers** | Fastest | Compact | Good | Mobile, embedded, streaming |

## Detailed Format Profiles

### ISO 2709 (MARC Binary)

**Use when:**
- Exchanging with traditional library systems (ILS, OCLC, etc.)
- Maximum compatibility is required
- Working with existing MARC workflows

**Avoid when:**
- Building modern web APIs
- Need for human-readable output
- Integrating with data science tools

```python
# Reading
with open("records.mrc", "rb") as f:
    for record in mrrc.MARCReader(f):
        print(record.title())

# Writing
with open("output.mrc", "wb") as f:
    with mrrc.MARCWriter(f) as writer:
        writer.write(record)
```

### Protobuf

**Use when:**
- Building microservices or APIs
- Need schema evolution (add fields without breaking old clients)
- Cross-language compatibility (Java, Go, C++, etc.)
- Smaller payload size than JSON

**Avoid when:**
- Human readability is required
- Working with systems that only support MARC

```python
# Writing
writer = mrrc.ProtobufWriter("records.pb")
for record in records:
    writer.write_record(record)
writer.close()

# Reading
for record in mrrc.ProtobufReader("records.pb"):
    print(record.title())
```

### JSON / MARCJSON

**Use when:**
- Building web APIs that need human-readable responses
- Debugging or inspecting records
- JavaScript frontend integration
- MARCJSON when LOC standard compliance is required

**Avoid when:**
- Bandwidth is limited
- Processing millions of records
- Need binary efficiency

```python
# Single record
json_str = record.to_json()
marcjson_str = record.to_marcjson()

# Parse back
restored = mrrc.json_to_record(json_str)
```

### Arrow

**Use when:**
- Analyzing MARC collections with DuckDB or Polars
- Building data science pipelines
- Need SQL queries over MARC data
- Large-scale batch analytics

**Avoid when:**
- Real-time record-by-record processing
- Simple read/write workflows
- Limited memory environments

```python
# Write to Arrow format
writer = mrrc.ArrowWriter("records.arrow")
for record in records:
    writer.write_record(record)
writer.close()

# Query with DuckDB
import duckdb
conn = duckdb.connect()
result = conn.execute("""
    SELECT title, author FROM 'records.arrow'
    WHERE publication_year > 2000
""").fetchall()
```

### MessagePack

**Use when:**
- REST APIs where JSON is too large
- Cross-language systems (50+ language support)
- Need compact binary but simpler than Protobuf
- IPC between processes

**Avoid when:**
- Need human-readable format
- Schema evolution is critical

```python
# Write
writer = mrrc.MessagePackWriter("records.msgpack")
for record in records:
    writer.write_record(record)
writer.close()

# Read
for record in mrrc.MessagePackReader("records.msgpack"):
    print(record.title())
```

### FlatBuffers

**Use when:**
- Mobile applications with memory constraints
- Streaming APIs needing zero-copy access
- Maximum read performance (no deserialization)
- Embedded systems

**Avoid when:**
- Need to modify records after creation
- Simple workflows where complexity isn't justified

```python
# Write
writer = mrrc.FlatbuffersWriter("records.fb")
for record in records:
    writer.write_record(record)
writer.close()

# Read (zero-copy access)
for record in mrrc.FlatbuffersReader("records.fb"):
    print(record.title())
```

### CBOR

**Use when:**
- Government or academic archival requirements
- Need IETF standard compliance (RFC 7049)
- Long-term preservation with standards backing

**Avoid when:**
- Need wide ecosystem tool support
- Real-time processing requirements

### Avro

**Use when:**
- Apache Kafka streaming pipelines
- Data lake ingestion (Hadoop, Spark)
- Schema registry integration
- Event sourcing architectures

**Avoid when:**
- Simple file-based workflows
- Systems outside the Kafka ecosystem

## Use Case Recommendations

### Library System Integration

**Recommended:** ISO 2709

Traditional library systems universally support ISO 2709. Use it for:
- OCLC WorldCat submissions
- ILS data migration
- Z39.50 compatible systems

### Web Application Backend

**Recommended:** Protobuf or MessagePack

For APIs serving web or mobile clients:
- Protobuf if you need schema evolution and multi-language support
- MessagePack for simpler integration with existing JSON tooling

### Data Science / Analytics

**Recommended:** Arrow

Arrow enables direct integration with:
- DuckDB for SQL queries
- Polars for DataFrame operations
- Pandas (via PyArrow)

### Archival / Preservation

**Recommended:** ISO 2709 + CBOR

- ISO 2709 for maximum future compatibility
- CBOR if standards compliance is mandated

### Real-Time Streaming

**Recommended:** FlatBuffers or Avro

- FlatBuffers for lowest latency (zero-copy)
- Avro for Kafka integration

### Spreadsheet Export

**Recommended:** CSV

CSV works with Excel, Google Sheets, and simple data tools.

## Performance Comparison

Based on benchmarks with 10k records:

| Format | Read Speed | Write Speed | File Size |
|--------|-----------|-------------|-----------|
| ISO 2709 | 900k rec/s | 800k rec/s | Baseline |
| Protobuf | 750k rec/s | 700k rec/s | 0.8x |
| Arrow | 865k rec/s | 800k rec/s | 0.04x (96% smaller) |
| MessagePack | 750k rec/s | 700k rec/s | 0.75x |
| FlatBuffers | 1M+ rec/s | 700k rec/s | 0.64x |
| JSON | 200k rec/s | 250k rec/s | 2.5x |

## Migration Path

If you're starting with ISO 2709 and want to adopt modern formats:

1. **Start:** ISO 2709 for all interchange
2. **Add:** JSON/MARCJSON for web APIs
3. **Scale:** Arrow for analytics workloads
4. **Optimize:** Protobuf or MessagePack for high-throughput APIs

## Next Steps

- [Installation Guide](./INSTALLATION_GUIDE.md) - Enable format features
- [Python Tutorial](./PYTHON_TUTORIAL.md) - Format conversion examples
- [Streaming Guide](./STREAMING_GUIDE.md) - Large file handling
