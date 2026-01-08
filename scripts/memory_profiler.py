#!/usr/bin/env python3
"""
Memory profiling analysis for MARC reading.

Analyzes heap allocation patterns and memory efficiency.
"""

import subprocess
import json
import re
from pathlib import Path
from dataclasses import dataclass
from typing import List, Dict, Optional


@dataclass
class AllocationSite:
    """Single allocation site info"""
    description: str
    count: int
    total_bytes: int
    avg_bytes: int
    
    @property
    def size_kb(self) -> float:
        return self.total_bytes / 1024


def analyze_record_structure() -> Dict:
    """Analyze MARC record structure to estimate memory usage"""
    print("=== MARC Record Structure Analysis ===\n")
    
    # Load a sample fixture
    fixture_path = Path("tests/data/fixtures/1k_records.mrc")
    if not fixture_path.exists():
        print(f"⚠️  Fixture not found: {fixture_path}")
        return {}
    
    fixture_size = fixture_path.stat().st_size
    record_count = 1000
    
    print(f"Fixture: {fixture_path.name}")
    print(f"File size: {fixture_size:,} bytes ({fixture_size/1024:.1f} KB)")
    print(f"Records: {record_count}")
    print(f"Avg per record: {fixture_size/record_count:.0f} bytes\n")
    
    return {
        "fixture_size": fixture_size,
        "record_count": record_count,
        "avg_per_record": fixture_size / record_count,
    }


def estimate_vec_overhead() -> Dict:
    """Estimate Vec allocation overhead in record parsing"""
    print("=== Vec Allocation Overhead ===\n")
    
    analysis = {
        "record_vec": {
            "items": "Field structs",
            "per_item": 32,  # Rust Field struct size (approximate)
            "capacity_multiplier": 1.5,  # Vec grows by 1.5x
            "items_per_record": 20,  # Average fields per record
        },
        "string_vec": {
            "items": "Subfield chars",
            "per_item": 1,  # UTF-8 bytes
            "capacity_multiplier": 1.5,  # Vec grows by 1.5x
            "items_per_record": 200,  # Average chars per record
        },
    }
    
    print("Vec Allocation Patterns:\n")
    
    total_heap = 0
    for vec_type, config in analysis.items():
        items = config["items_per_record"]
        per_item = config["per_item"]
        capacity = int(items * config["capacity_multiplier"])
        
        overhead = (capacity - items) * per_item
        total = capacity * per_item
        
        print(f"{vec_type}:")
        print(f"  Items: {items}")
        print(f"  Per item: {per_item} bytes")
        print(f"  Capacity: {capacity} (with 1.5x growth factor)")
        print(f"  Heap used: {total} bytes")
        print(f"  Wasted (capacity): {overhead} bytes ({100*overhead/total:.1f}%)\n")
        
        total_heap += total
    
    print(f"Estimated heap per record: ~{total_heap} bytes")
    print(f"For 10k records: ~{total_heap * 10000 / 1024 / 1024:.1f} MB\n")
    
    return analysis


def estimate_string_overhead() -> Dict:
    """Estimate String allocation overhead"""
    print("=== String Allocation Overhead ===\n")
    
    # String has: ptr (8), len (8), capacity (8) = 24 bytes + data
    string_header = 24
    
    analysis = {
        "field_tag": {
            "count_per_record": 20,
            "avg_bytes": 3,
            "overhead": string_header,
        },
        "indicators": {
            "count_per_record": 20,
            "avg_bytes": 2,
            "overhead": string_header,
        },
        "subfield_code": {
            "count_per_record": 50,
            "avg_bytes": 1,
            "overhead": string_header,
        },
        "subfield_data": {
            "count_per_record": 50,
            "avg_bytes": 50,
            "overhead": string_header,
        },
    }
    
    print("String Allocation per Record:\n")
    
    total_heap = 0
    for str_type, config in analysis.items():
        count = config["count_per_record"]
        avg_bytes = config["avg_bytes"]
        overhead = config["overhead"]
        
        # Account for String capacity overhead (usually 25% extra)
        capacity = int(avg_bytes * 1.25)
        per_string = overhead + capacity
        total = per_string * count
        
        print(f"{str_type}:")
        print(f"  Count: {count}/record")
        print(f"  Avg size: {avg_bytes} bytes + {overhead} header = {per_string} total")
        print(f"  Per record: {total} bytes\n")
        
        total_heap += total
    
    print(f"Total string overhead per record: ~{total_heap} bytes\n")
    return analysis


def allocation_hotspots() -> List[AllocationSite]:
    """List likely allocation hotspots"""
    print("=== Likely Allocation Hotspots ===\n")
    
    hotspots = [
        AllocationSite(
            "Field Vec (20 fields/record)",
            count=10000,
            total_bytes=10000 * 32 * 20,  # 20 Field structs per record
            avg_bytes=640,
        ),
        AllocationSite(
            "Subfield data Strings (50/record)",
            count=500000,  # 50 * 10k records
            total_bytes=500000 * 50,  # ~50 bytes avg per subfield
            avg_bytes=50,
        ),
        AllocationSite(
            "Tag Strings (20/record)",
            count=200000,  # 20 * 10k records
            total_bytes=200000 * 3,  # 3 bytes per tag
            avg_bytes=3,
        ),
        AllocationSite(
            "Indicator Strings (20/record)",
            count=200000,  # 20 * 10k records
            total_bytes=200000 * 2,  # 2 bytes per indicator pair
            avg_bytes=2,
        ),
    ]
    
    for hotspot in hotspots:
        print(f"{hotspot.description}:")
        print(f"  Allocations: {hotspot.count:,}")
        print(f"  Total: {hotspot.total_bytes:,} bytes ({hotspot.size_kb:.1f} KB)")
        print(f"  Avg: {hotspot.avg_bytes} bytes/alloc\n")
    
    return hotspots


def optimization_recommendations() -> List[str]:
    """List memory optimization opportunities"""
    print("=== Memory Optimization Opportunities ===\n")
    
    recommendations = [
        "1. Use SmallVec<[Field; 20]> instead of Vec for fields (avoids heap for typical records)",
        "2. Pool String allocations for tags (always 3 bytes) and indicators (always 2 bytes)",
        "3. Consider Cow<str> for immutable subfield data",
        "4. Use compact encoding for field tags (u16) + lookup table",
        "5. Align Field struct to 64 bytes for cache efficiency",
        "6. Consider arena allocation for all subfield strings in a batch",
        "7. Use indices instead of Strings for frequently-accessed fields",
        "8. Measure actual allocation patterns with cargo-valgrind or heaptrack",
    ]
    
    for rec in recommendations:
        print(f"{rec}")
    
    return recommendations


def main():
    print("Memory Profiling Analysis for MARC Reading\n")
    print("=" * 70 + "\n")
    
    # Run analyses
    record_structure = analyze_record_structure()
    print()
    
    vec_overhead = estimate_vec_overhead()
    print()
    
    string_overhead = estimate_string_overhead()
    print()
    
    hotspots = allocation_hotspots()
    print()
    
    recommendations = optimization_recommendations()
    print()
    
    # Summary
    print("=" * 70)
    print("SUMMARY")
    print("=" * 70)
    print("""
Memory usage in MARC record parsing is dominated by:
1. Field Vec allocations (headers + capacity overhead)
2. String allocations (24-byte header per string)
3. Subfield data storage (actual content)

Key observation: Many allocations (tags, indicators) are fixed-size but
treated as variable-length strings, wasting space.

Most impactful optimizations:
- SmallVec for field array (avoids heap for typical records)
- String pooling for tags and indicators (fixed, repeated data)
- Arena allocation for batch subfield processing
""")


if __name__ == "__main__":
    main()
