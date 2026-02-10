#!/bin/bash
# Validate the fidelity test set (105 MARC records)

set -e

TEST_FILE="${1:-tests/data/fixtures/fidelity_test_100.mrc}"

if [[ ! -f "$TEST_FILE" ]]; then
    echo "Error: Test file not found: $TEST_FILE"
    exit 1
fi

echo "Validating fidelity test set: $TEST_FILE"
echo "============================================"

# Use Python with pymarc to analyze the test set
python3 - "$TEST_FILE" << 'PYTHON_EOF'
import sys
from pathlib import Path

# Add src-python to path
sys.path.insert(0, str(Path.cwd() / 'src-python'))

from pymarc import MARCReader

test_file = sys.argv[1]

with open(test_file, 'rb') as f:
    reader = MARCReader(f)
    records = list(reader)

print(f"\n✓ Total Records: {len(records)}")

# Check record count (should be 105)
if len(records) != 105:
    print(f"✗ ERROR: Expected 105 records, got {len(records)}")
    sys.exit(1)
else:
    print("✓ Record count: 105 (expected)")

# Analyze record types by leader[6]
leader6_counts = {}
for record in records:
    try:
        leader6 = str(record.leader)[6]
    except (IndexError, TypeError):
        leader6 = '?'
    leader6_counts[leader6] = leader6_counts.get(leader6, 0) + 1

print(f"\n✓ Record Types (leader[6]):")
for lt, count in sorted(leader6_counts.items()):
    type_name = {
        'a': 'Language Material',
        'c': 'Notated Music',
        'd': 'Manuscript Notated Music',
        'e': 'Cartographic Material',
        'f': 'Manuscript Cartographic',
        'g': 'Projected Medium',
        'i': 'Nonmusical Sound',
        'j': 'Musical Sound',
        'k': 'Two-dimensional Nonprojectable',
        'm': 'Computer File',
        'o': 'Kit',
        'p': 'Mixed Materials',
        'r': 'Three-dimensional Object',
        't': 'Manuscript Language Material',
    }.get(lt, 'Unknown')
    print(f"  {lt} ({type_name}): {count}")

# Check for edge cases
print(f"\n✓ Edge Case Detection:")

has_cjk = False
has_rtl = False
has_combining = False
has_large_field = False
has_many_subfields = False
has_empty_subfield = False
has_repeating_subfields = False
has_mixed_script = False

for record in records:
    for field in record.fields:
        if hasattr(field, 'subfields') and field.subfields:
            # Build text content for this field
            field_text = ' '.join(str(sf) for sf in field.subfields if isinstance(sf, str))
            
            # Check for CJK
            if any('\u4e00' <= c <= '\u9fff' for c in field_text):
                has_cjk = True
            # Check for RTL
            if any('\u0600' <= c <= '\u06ff' or '\u0590' <= c <= '\u05ff' for c in field_text):
                has_rtl = True
            # Check for combining marks
            if any('\u0300' <= c <= '\u036f' for c in field_text):
                has_combining = True
            # Check for mixed script
            if (any('\u0600' <= c <= '\u06ff' or '\u0590' <= c <= '\u05ff' for c in field_text) and 
                any(('a' <= c <= 'z') or ('A' <= c <= 'Z') for c in field_text)):
                has_mixed_script = True
            
            # Check for empty subfields (odd indices in subfields array)
            for i in range(1, len(field.subfields), 2):
                val = str(field.subfields[i]) if i < len(field.subfields) else ''
                if val == '':
                    has_empty_subfield = True
                if len(val) > 1000:
                    has_large_field = True
            
            # Check for repeating subfields (same code appears multiple times)
            subfield_codes = [str(field.subfields[i]) for i in range(0, len(field.subfields), 2)]
            if len(subfield_codes) != len(set(subfield_codes)):
                has_repeating_subfields = True
            
            # Check for many subfields
            num_subfields = len(field.subfields) // 2
            if num_subfields > 40:
                has_many_subfields = True

print(f"  CJK characters: {'✓' if has_cjk else '✗'}")
print(f"  RTL scripts (Arabic/Hebrew): {'✓' if has_rtl else '✗'}")
print(f"  Combining diacritics: {'✓' if has_combining else '✗'}")
print(f"  Large fields (>1000 bytes): {'✓' if has_large_field else '✗'}")
print(f"  Many subfields (40+): {'✓' if has_many_subfields else '✗'}")
print(f"  Empty subfield values: {'✓' if has_empty_subfield else '✗'}")
print(f"  Repeating subfields: {'✓' if has_repeating_subfields else '✗'}")
print(f"  Mixed scripts: {'✓' if has_mixed_script else '✗'}")

# Check field type coverage
field_tags = set()
for record in records:
    for field in record.fields:
        field_tags.add(field.tag)

print(f"\n✓ Field Type Coverage: {len(field_tags)} unique tags")
print(f"  001-009 (Control): {len([t for t in field_tags if t.startswith('00')])}")
print(f"  1XX (Main Entry): {len([t for t in field_tags if t.startswith('1')])}")
print(f"  2XX (Title/Edition): {len([t for t in field_tags if t.startswith('2')])}")
print(f"  3XX (Physical): {len([t for t in field_tags if t.startswith('3')])}")
print(f"  4XX (Series): {len([t for t in field_tags if t.startswith('4')])}")
print(f"  5XX (Note): {len([t for t in field_tags if t.startswith('5')])}")
print(f"  6XX (Subject): {len([t for t in field_tags if t.startswith('6')])}")
print(f"  7XX (Added Entry): {len([t for t in field_tags if t.startswith('7')])}")
print(f"  8XX (Series/Other): {len([t for t in field_tags if t.startswith('8')])}")
print(f"  9XX (Local): {len([t for t in field_tags if t.startswith('9')])}")

# Summary
print(f"\n✓ Validation PASSED")
print(f"  Test set is ready for format evaluations")

PYTHON_EOF
