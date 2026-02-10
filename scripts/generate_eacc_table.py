#!/usr/bin/env python3
"""
Generate EACC character table for Rust from pymarc's marc8_mapping.py

Requires pymarc source code to be available at /tmp/pymarc
"""

import sys
sys.path.insert(0, '/tmp/pymarc')

from pymarc import marc8_mapping

# Get the EACC table
eacc_map = marc8_mapping.CHARSET_31

print("    /// EACC (East Asian Character Code) - 3-byte characters")
print("    static ref EACC_TABLE: HashMap<u32, CharacterMapping> = {")
print("        let mut m = HashMap::new();")

# Sort by key for readability
sorted_keys = sorted(eacc_map.keys())

for key in sorted_keys:
    unicode_val, combining = eacc_map[key]
    # Format as 3-byte key (concatenated bytes)
    print(f"        m.insert(0x{key:06X}, (0x{unicode_val:X}, {combining}));")

print("        m")
print("    };")

print(f"\n// Total EACC mappings: {len(eacc_map)}", file=sys.stderr)
