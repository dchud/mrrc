import re

# Read the main __init__.py
with open('mrrc/__init__.py') as f:
    init_code = f.read()

# Read the .pyi stub file
with open('mrrc/_mrrc.pyi') as f:
    pyi_code = f.read()

# Read the wrappers.rs for Rust methods
with open('src-python/src/wrappers.rs') as f:
    rust_code = f.read()

# Define gaps based on bd-wifu issue
gaps = {
    'A': ('Separate ControlField class', 'ControlField class'),
    'B': ('ControlField .value vs .data', '.data attribute'),
    'C': ('Record properties as @property not methods', '@property'),
    'D': ('Record.__getitem__ KeyError not None', 'KeyError'),
    'E': ('Field.__str__/__repr__', '__str__'),
    'F': ('Record.as_marc()/as_marc21()', 'as_marc'),
    'G': ('JSON serialization format mismatch', 'as_json'),
    'H': ('Field.value() method', 'def value'),
    'I': ('Field.format_field()', 'def format_field'),
    'J': ('Record.as_json()/as_dict()', 'as_dict'),
    'K': ('parse_xml_to_array()', 'parse_xml_to_array'),
    'L': ('add_field(*fields) signature', 'add_field.*\\*'),
    'M': ('remove_field(*fields)/remove_fields(*tags)', 'remove_fields'),
    'N': ('add_ordered_field()/add_grouped_field()', 'add_ordered_field'),
    'O': ('Field.add_subfield() pos parameter', 'pos'),
    'P': ('Field.as_marc()/as_marc21()', 'Field.*as_marc'),
    'Q': ('Field.linkage_occurrence_num()', 'linkage_occurrence_num'),
    'R': ('Property names: physicaldescription, uniformtitle, addedentries', 'physicaldescription'),
    'S': ('pubyear return type (int vs str)', 'pubyear'),
    'T': ('Field.convert_legacy_subfields()', 'convert_legacy_subfields'),
    'U': ('Record.as_marc()/as_marc21() (alias for F)', None),
    'V': ('decode_marc()', 'decode_marc'),
    'W': ('Reader/writer variants', 'JSONReader'),
    'X': ('Exception hierarchy', 'class.*Exception'),
    'Y': ('MARC8 Python API', 'MARC8ToUnicode'),
    'Z': ('Convenience functions', 'map_records'),
    'AA': ('Constants (LEADER_LEN, etc)', 'LEADER_LEN'),
    'BB': ('Record instance attributes (force_utf8, pos)', 'force_utf8'),
}

results = {}
for gap, (desc, pattern) in gaps.items():
    if pattern is None:
        status = '---'
        reason = 'Alias for another gap'
    else:
        found_init = bool(re.search(pattern, init_code))
        found_pyi = bool(re.search(pattern, pyi_code))
        found_rust = bool(re.search(pattern, rust_code))
        
        if found_init or found_pyi or found_rust:
            status = 'FIXED'
            reason = []
            if found_rust: reason.append('Rust')
            if found_pyi: reason.append('Stub')
            if found_init: reason.append('Python')
            reason = ' / '.join(reason)
        else:
            status = 'OPEN'
            reason = 'Not found'
    
    results[gap] = (status, reason, desc)

# Print results
for gap in sorted(results.keys()):
    status, reason, desc = results[gap]
    print(f"{gap}. [{status:5}] {desc}")
    if reason and status != '---':
        print(f"     → {reason}")
    print()
