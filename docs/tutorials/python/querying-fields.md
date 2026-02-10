# Querying Fields (Python)

Learn to use MRRC's Query DSL for complex field searching.

## Basic Field Access

```python
import mrrc

for record in mrrc.MARCReader("records.mrc"):
    # Get all fields with a tag
    for field in record.fields_by_tag("650"):
        print(field["a"])
```

## Filter by Indicators

Find fields with specific indicator values:

```python
# Find 650 fields with indicator2='0' (LCSH)
lcsh_subjects = record.fields_by_indicator("650", indicator2="0")

for field in lcsh_subjects:
    print(field["a"])

# Match both indicators
fields = record.fields_by_indicator("245", indicator1="1", indicator2="0")
```

## Filter by Tag Range

Find fields within a tag range:

```python
# Find all subject fields (600-699)
subjects = record.fields_in_range("600", "699")

for field in subjects:
    print(f"{field.tag}: {field['a']}")

# Find all notes (500-599)
notes = record.fields_in_range("500", "599")
```

## FieldQuery (Complex Matching)

Build complex queries with FieldQuery:

```python
# Create a query builder
query = mrrc.FieldQuery()
query.tag("650")
query.indicator2("0")
query.has_subfield("a")

# Execute the query
for field in record.fields_matching(query):
    print(field["a"])
```

### Combining Conditions

```python
query = mrrc.FieldQuery()
query.tag("650")              # Must be 650
query.indicator2("0")         # Indicator2 = 0
query.has_subfield("a")       # Must have $a
query.has_subfield("x")       # Must have $x subdivision

matches = record.fields_matching(query)
```

## Pattern Matching

Match subfield values with regular expressions:

```python
# Find ISBN-13s (start with 978 or 979)
query = mrrc.SubfieldPatternQuery("020", "a", r"^97[89]")
for field in record.fields_matching_pattern(query):
    print(field["a"])

# Find emails in 856 field
query = mrrc.SubfieldPatternQuery("856", "u", r"@.*\.")
for field in record.fields_matching_pattern(query):
    print(field["u"])
```

## Value Matching

Match exact or partial subfield values:

```python
# Exact match
query = mrrc.SubfieldValueQuery("650", "a", "History")
exact_matches = record.fields_matching_value(query)

# Partial match (contains)
query = mrrc.SubfieldValueQuery("650", "a", "History", partial=True)
partial_matches = record.fields_matching_value(query)
```

## Complete Example

```python
#!/usr/bin/env python3
"""Find records with specific subject headings."""

import mrrc

def find_lcsh_subjects(path, search_term):
    """Find records with LCSH subjects containing a term."""

    results = []

    for record in mrrc.MARCReader(path):
        # Query for LCSH subjects (indicator2=0) containing search term
        query = mrrc.SubfieldValueQuery("650", "a", search_term, partial=True)

        # Also check indicator2 = 0 (LCSH)
        for field in record.fields_by_indicator("650", indicator2="0"):
            if search_term.lower() in (field["a"] or "").lower():
                results.append({
                    "title": record.title(),
                    "subject": field["a"]
                })

    return results

# Find records about "computers"
results = find_lcsh_subjects("library.mrc", "computers")
for r in results:
    print(f"{r['title']}: {r['subject']}")
```

## Performance Tips

- Use specific tags when possible (faster than ranges)
- Filter by indicator first, then by subfield content
- For large files, consider using the concurrency features

## Next Steps

- [Reading Records](reading-records.md) - Basic record access
- [Concurrency](concurrency.md) - Parallel processing
- [Query DSL Guide](../../guides/query-dsl.md) - Full Query DSL reference
