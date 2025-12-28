"""
MRRC: Fast MARC library written in Rust with Python bindings.

This module provides Python access to the Rust MARC library, enabling fast
reading, writing, and manipulation of MARC bibliographic records.
"""

from mrrc._mrrc import Record

__version__ = "0.1.0"
__all__ = ["Record"]
