# BIBFRAME Conversion Baselines

This directory contains baseline MARC→BIBFRAME conversions generated using the official
LOC marc2bibframe2 tool (v3.0.0, December 2025). These baselines serve as ground truth
for testing mrrc's BIBFRAME conversion implementation.

## Directory Structure

```
bibframe-baselines/
├── marc-input/          # Source MARCXML files
├── bibframe-output/     # Converted BIBFRAME RDF/XML files
└── README.md            # This file
```

## Test Files

| File | MARC Fields | Purpose |
|------|-------------|---------|
| `simple-record` | Basic bib record | Core Work/Instance structure |
| `collection` | Multiple records | Batch processing |
| `leader-types` | Leader variations | Work/Instance type determination |
| `names-agents` | 1XX, 6XX, 7XX, 8XX | Agent/Contribution mapping |
| `titles` | 245 | Main title handling |
| `uniform-titles` | 240, X30 | Hub creation |
| `publication` | 250-270 | ProvisionActivity mapping |
| `physical-desc` | 3XX | Extent, dimensions, media |
| `notes` | 5XX | Note types |
| `subjects` | 648-662 | Subject entity types |
| `linking-entries` | 760-788 | Related work/instance links |
| `alternate-scripts` | 880 | Non-Latin script handling |

## Generation Command

```bash
xsltproc --stringparam baseuri http://example.org/ \
  tools/marc2bibframe2/xsl/marc2bibframe2.xsl \
  marc-input/FILE.xml > bibframe-output/FILE.rdf.xml
```

## Converter Version

- **Tool**: marc2bibframe2
- **Version**: v3.0.0
- **Date**: December 2025
- **URL**: https://github.com/lcnetdev/marc2bibframe2

## Known Issues Discovered

### 1. Incomplete 008 Fields Cause Failures
The original ConvSpec-250-270/marc.xml has records with minimal 008 fields (e.g.,
`<controlfield tag="008">               xxk</controlfield>`) which cause XPath errors
in the converter. The `publication.xml` baseline uses `marc-process8.xml` instead, which
has complete records.

### 2. Warning: No 'idsource' Parameter
The converter emits warnings when `idsource` parameter is not provided:
```
No 'idsource' runtime value provided. No agent identified as the actor converting the record.
```
This is informational only and doesn't affect conversion output.

### 3. Output Format Limited to RDF/XML
marc2bibframe2 only supports RDF/XML output (`serialization=rdfxml`). For JSON-LD or
Turtle output, post-processing with an RDF library is required.

## Usage in Tests

These baselines should be used to:

1. **Structural comparison**: Verify mrrc creates same entity types (Work, Instance, etc.)
2. **Property coverage**: Verify same MARC fields produce same BIBFRAME properties
3. **Value mapping**: Verify controlled vocabulary URIs match (e.g., relator codes)
4. **Round-trip testing**: MARC → mrrc BIBFRAME → MARC should match MARC → LOC BIBFRAME → MARC

## Regenerating Baselines

If LOC releases a new version of marc2bibframe2:

```bash
# Update the tool
cd tools/marc2bibframe2 && git pull

# Regenerate all baselines
for file in tests/data/bibframe-baselines/marc-input/*.xml; do
    basename=$(basename "$file" .xml)
    xsltproc --stringparam baseuri http://example.org/ \
      tools/marc2bibframe2/xsl/marc2bibframe2.xsl \
      "$file" > "tests/data/bibframe-baselines/bibframe-output/${basename}.rdf.xml"
done
```

---

*Generated: 2026-01-27*
*Task: mrrc-uab.3*
