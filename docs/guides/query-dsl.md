# Query DSL: Advanced Field Searching

The Query DSL is a powerful feature that goes beyond pymarc's simple `get_fields(*tags)` method, enabling complex field filtering based on indicators, tag ranges, subfield presence, and pattern matching.

## Why Query DSL?

In library cataloging, you often need to find fields based on criteria more specific than just the tag. Common scenarios include:

- **Find LCSH subjects only** — 650 fields where indicator2='0' (not MeSH, not local)
- **Find all subject fields** — any 6XX field (600-699)
- **Find ISBN-13s** — 020 fields where subfield $a starts with "978-" or "979-"
- **Find authors with death dates** — 100 fields where subfield $d matches the pattern "YYYY-YYYY"

The Query DSL provides a composable, type-safe way to express these requirements.

## Philosophy

### Why Multiple Query Types?

The Query DSL uses separate query types instead of a single generic type for clarity and performance:

| Query Type | Use Case | Example |
|------------|----------|---------|
| `FieldQuery` | Match by tag, indicators, and required subfields | Find 650 ind2='0' with $a and $x |
| `TagRangeQuery` | Match fields in a tag range with filters | Find all 6XX subjects |
| `SubfieldPatternQuery` | Regex matching on subfield values | Find ISBNs starting with 978 |
| `SubfieldValueQuery` | Exact or substring matching | Find subjects containing "History" |

Each type is optimized for its use case:
- `FieldQuery` and `TagRangeQuery` can short-circuit based on tag before examining indicators
- `SubfieldPatternQuery` compiles the regex once and reuses it
- `SubfieldValueQuery` uses efficient string matching

### Builder Pattern

`FieldQuery` uses the builder pattern for expressive, readable queries:

```python
# Readable chain of constraints
query = (FieldQuery()
    .tag("650")
    .indicator2("0")
    .has_subfield("a")
    .has_subfield("x"))
```

This is clearer than positional arguments and allows adding constraints incrementally.

## Query Types Reference

### FieldQuery

The main query builder for complex field matching.

```python
from mrrc import FieldQuery

# Match all fields (no restrictions)
query = FieldQuery()

# Match by tag
query = FieldQuery().tag("650")

# Match by indicators
query = FieldQuery().tag("650").indicator1(" ").indicator2("0")

# Require specific subfields (AND logic)
query = FieldQuery().tag("650").has_subfield("a").has_subfield("x")

# Shorthand for multiple subfields
query = FieldQuery().tag("650").has_subfields(["a", "x", "v"])
```

**Indicator Wildcards**: Pass `None` (or omit the call) to match any indicator value:

```python
# Match any indicator1, but indicator2 must be '0'
query = FieldQuery().tag("650").indicator1(None).indicator2("0")

# Equivalent (indicator1 not set = wildcard)
query = FieldQuery().tag("650").indicator2("0")
```

### TagRangeQuery

Find fields within a tag range, optionally with indicator and subfield filters.

```python
from mrrc import TagRangeQuery

# All subject fields (600-699)
query = TagRangeQuery("600", "699")

# All 6XX with indicator2='0' (LCSH only)
query = TagRangeQuery("600", "699", indicator2="0")

# With required subfields
query = TagRangeQuery("600", "699", indicator2="0", required_subfields=["a"])
```

You can also create a `TagRangeQuery` from a `FieldQuery`:

```python
# Start with constraints, then specify range
query = FieldQuery().indicator2("0").has_subfield("a").tag_range("600", "699")
```

### SubfieldPatternQuery

Match fields where a subfield value matches a regular expression.

```python
from mrrc import SubfieldPatternQuery

# ISBN-13 (starts with 978 or 979)
query = SubfieldPatternQuery("020", "a", r"^97[89]-")

# Personal names with death dates (YYYY-YYYY pattern)
query = SubfieldPatternQuery("100", "d", r"^\d{4}-\d{4}$")

# Living persons (birth year only, no death)
query = SubfieldPatternQuery("100", "d", r"^\d{4}-$")

# URLs in notes
query = SubfieldPatternQuery("856", "u", r"^https?://")
```

**Note**: The regex uses Rust's regex syntax, which is similar to Python's `re` module but not identical. Most common patterns work the same way.

### SubfieldValueQuery

Match fields where a subfield value matches a string exactly or as a substring.

```python
from mrrc import SubfieldValueQuery

# Exact match (case-sensitive)
query = SubfieldValueQuery("650", "a", "History")

# Partial/substring match
query = SubfieldValueQuery("650", "a", "History", partial=True)

# Will match: "History", "World History", "History of Science"
```

## Record Methods

Once you have a query, use these `Record` methods to find matching fields:

### Convenience Methods

For common patterns, use these simpler methods:

```python
# Find by indicator (no query object needed)
lcsh_subjects = record.fields_by_indicator("650", indicator2="0")

# Find in tag range
all_subjects = record.fields_in_range("600", "699")
```

### Query-Based Methods

For complex queries:

```python
# FieldQuery
results = record.fields_matching(query)

# TagRangeQuery  
results = record.fields_matching_range(range_query)

# SubfieldPatternQuery
results = record.fields_matching_pattern(pattern_query)

# SubfieldValueQuery
results = record.fields_matching_value(value_query)
```

All methods return a list of `Field` objects that you can iterate over.

## Practical Examples

### Example 1: Find LCSH Subject Headings with Subdivisions

A common cataloging analysis task: identify records with rich subject analysis.

```python
from mrrc import FieldQuery, MARCReader

with open("records.mrc", "rb") as f:
    reader = MARCReader(f)
    for record in reader:
        query = (FieldQuery()
            .tag("650")
            .indicator2("0")  # LCSH
            .has_subfield("x"))  # Has topical subdivision
        
        for field in record.fields_matching(query):
            heading = field["a"]
            subdivision = field["x"]
            print(f"  Subject: {heading} -- {subdivision}")
```

### Example 2: Extract All Subject Headings for Export

When converting to Dublin Core or another format, gather all subjects:

```python
from mrrc import TagRangeQuery

# All 6XX fields regardless of thesaurus
query = TagRangeQuery("600", "699")
subjects = []

for field in record.fields_matching_range(query):
    if heading := field["a"]:
        subjects.append(heading)

# subjects now contains all subject headings
```

### Example 3: Find ISBN-13 for Linking

Modern linking systems prefer ISBN-13:

```python
from mrrc import SubfieldPatternQuery

query = SubfieldPatternQuery("020", "a", r"^97[89]-")
for field in record.fields_matching_pattern(query):
    isbn13 = field["a"]
    print(f"ISBN-13: {isbn13}")
```

### Example 4: Identify Records by Subject Content

Find records about a specific topic using partial matching:

```python
from mrrc import SubfieldValueQuery

# Find any subject containing "climate change" (substring)
query = SubfieldValueQuery("650", "a", "climate change", partial=True)

relevant_records = []
for record in reader:
    if record.fields_matching_value(query):
        relevant_records.append(record)
```

### Example 5: Complex Multi-Step Analysis

Chain queries for sophisticated analysis:

```python
# Step 1: Find all subject fields
all_subjects = record.fields_in_range("600", "699")
print(f"Total subjects: {len(all_subjects)}")

# Step 2: Filter to LCSH
lcsh_query = TagRangeQuery("600", "699", indicator2="0")
lcsh = record.fields_matching_range(lcsh_query)
print(f"LCSH subjects: {len(lcsh)}")

# Step 3: Among LCSH, count those with geographic subdivisions
with_geo = [f for f in lcsh if f["z"]]  # $z = geographic
print(f"With geographic: {len(with_geo)}")
```

## Comparison with pymarc

| Task | pymarc | mrrc Query DSL |
|------|--------|----------------|
| Get all 650 fields | `record.get_fields('650')` | `record.get_fields('650')` |
| Get LCSH 650 only | Manual filter loop | `record.fields_by_indicator('650', indicator2='0')` |
| Get all 6XX fields | `record.get_fields('600', '610', '611', ...)` | `record.fields_in_range('600', '699')` |
| Find ISBN-13 | Manual regex loop | `record.fields_matching_pattern(query)` |
| Find by subfield value | Manual loop | `record.fields_matching_value(query)` |

The Query DSL handles filtering at the Rust level, which is faster than Python loops and more expressive than manual filtering.

## Performance Notes

- Queries are evaluated lazily where possible
- Tag-based filtering happens before indicator/subfield checks
- Regex patterns are compiled once when the query is created
- For very large files, the Query DSL is significantly faster than equivalent Python loops

## Error Handling

```python
# Invalid regex raises ValueError
try:
    query = SubfieldPatternQuery("020", "a", r"[invalid")
except ValueError as e:
    print(f"Bad pattern: {e}")

# Empty subfield code raises ValueError
try:
    query = FieldQuery().has_subfield("")
except ValueError as e:
    print(f"Invalid: {e}")
```

## See Also

- [MARC 21 Format for Bibliographic Data](https://www.loc.gov/marc/bibliographic/) - Field and indicator definitions
- [Library of Congress Subject Headings](https://id.loc.gov/authorities/subjects.html) - LCSH (indicator2='0')
- [tests/python/test_query_dsl.py](../../tests/python/test_query_dsl.py) - Comprehensive test examples
