# BIBFRAME Conversion

Convert MARC records to BIBFRAME 2.0 RDF for linked data applications.

## What is BIBFRAME?

BIBFRAME (Bibliographic Framework) is the Library of Congress's linked data model for bibliographic description. Key differences from MARC:

| MARC | BIBFRAME |
|------|----------|
| Flat records with tagged fields | RDF graph with linked entities |
| Single record per item | Separate Work, Instance, and Item entities |
| Field-based structure (1970s design) | Linked data with URIs (modern web) |

A single MARC record typically becomes multiple BIBFRAME entities:

```
MARC Record → bf:Work (intellectual content)
           → bf:Instance (physical/digital manifestation)
           → bf:Item (specific copy)
           → Related entities (agents, subjects, identifiers)
```

## Requirements

BIBFRAME support is included in MRRC by default for both Rust and Python. No feature flags or extra installation steps are needed.

## Basic Conversion

### MARC to BIBFRAME

=== "Python"

    ```python
    from mrrc import MARCReader, marc_to_bibframe, BibframeConfig

    # Read a MARC record
    for record in MARCReader("records.mrc"):
        # Convert to BIBFRAME
        config = BibframeConfig()
        graph = marc_to_bibframe(record, config)

        # Serialize to Turtle format
        turtle = graph.serialize("turtle")
        print(turtle)
    ```

=== "Rust"

    ```rust
    use mrrc::MarcReader;
    use mrrc::bibframe::{marc_to_bibframe, BibframeConfig, RdfFormat};
    use std::fs::File;

    let mut reader = MarcReader::new(File::open("records.mrc")?);
    while let Some(record) = reader.read_record()? {
        let config = BibframeConfig::default();
        let graph = marc_to_bibframe(&record, &config);

        // Serialize to Turtle format
        let turtle = graph.serialize(RdfFormat::Turtle)?;
        println!("{}", turtle);
    }
    ```

### BIBFRAME to MARC

=== "Python"

    ```python
    from mrrc import bibframe_to_marc, BibframeGraph

    # Parse RDF graph
    graph = BibframeGraph.from_turtle(turtle_string)

    # Convert back to MARC
    record = bibframe_to_marc(graph)

    # Write to file
    with MARCWriter("output.mrc") as writer:
        writer.write(record)
    ```

=== "Rust"

    ```rust
    use mrrc::bibframe::{bibframe_to_marc, BibframeGraph};

    let graph = BibframeGraph::from_turtle(&turtle_string)?;
    let record = bibframe_to_marc(&graph)?;
    ```

## Configuration Options

`BibframeConfig` controls conversion behavior:

### Base URI

Set a base URI for generated entity URIs:

=== "Python"

    ```python
    config = BibframeConfig()
    config.set_base_uri("http://library.example.org/bibframe/")

    graph = marc_to_bibframe(record, config)
    # URIs: http://library.example.org/bibframe/work/123456
    #       http://library.example.org/bibframe/instance/123456
    ```

=== "Rust"

    ```rust
    let config = BibframeConfig {
        base_uri: Some("http://library.example.org/bibframe/".to_string()),
        ..Default::default()
    };
    ```

Without a base URI, MRRC uses blank nodes (`_:work1`, `_:instance1`), which is valid RDF.

### URI Patterns

When `base_uri` is set:

| Entity | Pattern | Example |
|--------|---------|---------|
| Work | `{base}/work/{control-number}` | `http://example.org/work/123456` |
| Instance | `{base}/instance/{control-number}` | `http://example.org/instance/123456` |
| Item | `{base}/item/{control-number}-{seq}` | `http://example.org/item/123456-1` |
| Agent | `{base}/agent/{hash}` | `http://example.org/agent/a1b2c3` |

## RDF Serialization Formats

Serialize the BIBFRAME graph to different RDF formats:

=== "Python"

    ```python
    graph = marc_to_bibframe(record, config)

    # RDF/XML - W3C standard, tool compatibility
    rdf_xml = graph.serialize("rdf-xml")

    # Turtle - Human-readable, prefixed namespaces
    turtle = graph.serialize("turtle")

    # JSON-LD - JSON representation for web APIs
    jsonld = graph.serialize("jsonld")

    # N-Triples - Simple line-based, triple stores
    ntriples = graph.serialize("ntriples")
    ```

=== "Rust"

    ```rust
    use mrrc::bibframe::RdfFormat;

    let rdf_xml = graph.serialize(RdfFormat::RdfXml)?;
    let turtle = graph.serialize(RdfFormat::Turtle)?;
    let jsonld = graph.serialize(RdfFormat::JsonLd)?;
    let ntriples = graph.serialize(RdfFormat::NTriples)?;
    ```

### Format Selection Guide

| Format | Python | Rust | Best For |
|--------|--------|------|----------|
| RDF/XML | `"rdf-xml"` | `RdfFormat::RdfXml` | RDF tool interoperability, SPARQL endpoints |
| Turtle | `"turtle"` | `RdfFormat::Turtle` | Development, debugging, human review |
| JSON-LD | `"jsonld"` | `RdfFormat::JsonLd` | Web applications, JavaScript consumption |
| N-Triples | `"ntriples"` | `RdfFormat::NTriples` | Triple store bulk loading, streaming |

## Batch Conversion

Process multiple records efficiently:

=== "Python"

    ```python
    from mrrc import MARCReader, marc_to_bibframe, BibframeConfig

    config = BibframeConfig()
    config.set_base_uri("http://library.example.org/")

    with open("output.ttl", "w") as out:
        for record in MARCReader("large_file.mrc"):
            graph = marc_to_bibframe(record, config)
            out.write(graph.serialize("turtle"))
            out.write("\n")
    ```

=== "Rust"

    ```rust
    use mrrc::MarcReader;
    use mrrc::bibframe::{marc_to_bibframe, BibframeConfig, RdfFormat};
    use std::fs::File;
    use std::io::Write;

    let config = BibframeConfig {
        base_uri: Some("http://library.example.org/".to_string()),
        ..Default::default()
    };

    let mut reader = MarcReader::new(File::open("large_file.mrc")?);
    let mut out = File::create("output.ttl")?;

    while let Some(record) = reader.read_record()? {
        let graph = marc_to_bibframe(&record, &config);
        writeln!(out, "{}", graph.serialize(RdfFormat::Turtle)?)?;
    }
    ```

## MARC Field Mappings

Common MARC fields map to BIBFRAME as follows:

| MARC Field | BIBFRAME Property |
|------------|-------------------|
| 001 (Control Number) | bf:identifiedBy / bf:Local |
| 020 (ISBN) | bf:identifiedBy / bf:Isbn |
| 100/700 (Personal Name) | bf:contribution / bf:Person |
| 110/710 (Corporate Name) | bf:contribution / bf:Organization |
| 245 (Title) | bf:title / bf:Title |
| 260/264 (Publication) | bf:provisionActivity |
| 300 (Physical Description) | bf:extent |
| 6XX (Subjects) | bf:subject |

## Limitations and Data Loss

BIBFRAME ↔ MARC conversion is not perfectly lossless. Some considerations:

### MARC → BIBFRAME

- **Preserved**: Core bibliographic data, identifiers, relationships
- **Transformed**: Flat structure becomes linked entities
- **Potentially lost**: Some local/proprietary MARC extensions

### BIBFRAME → MARC

- **Preserved**: Standard bibliographic elements
- **Potentially lost**: Rich RDF relationships that have no MARC equivalent
- **Flattened**: Multiple linked entities collapse to single record

### Best Practices for Round-Trip Fidelity

1. **Use BFLC extensions** for Library of Congress compatibility
2. **Preserve control numbers** for record matching
3. **Test with representative samples** before bulk conversion
4. **Keep original MARC** if exact preservation is required

## Troubleshooting

### Empty or Minimal Output

Check that the MARC record has required fields:

- 001 (Control Number) - Used for URI generation
- 245 (Title) - Core bibliographic data
- Leader with valid record type

### Invalid RDF Output

Verify the MARC record is well-formed:

```python
# Check record validity
errors = record.validate()
if errors:
    print(f"Record issues: {errors}")
```

## See Also

- [Format Support Reference](../reference/formats.md) - All supported formats
- [Library of Congress BIBFRAME](https://www.loc.gov/bibframe/) - Official specification
- [BIBFRAME Ontology](https://id.loc.gov/ontologies/bibframe.html) - Class and property definitions
