"""Format-specific modules for MARC record serialization.

This package provides organized access to supported MARC formats.
Each format module provides consistent read/write functions.

Formats
-------
- **marc**: ISO 2709 binary MARC (standard interchange format)

For BIBFRAME (LOC linked data), see `mrrc.marc_to_bibframe` and
`mrrc.bibframe_to_marc` at the top-level module.

Quick Start
-----------
>>> from mrrc.formats import marc
>>> for record in marc.read("records.mrc"):
...     print(record.title())
>>> count = marc.write(records, "output.mrc")
"""

from . import marc

__all__ = [
    "marc",
]
