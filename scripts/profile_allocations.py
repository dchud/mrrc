#!/usr/bin/env python3
"""
Estimate allocation patterns in single-threaded MARC parsing
from the Record structure layout.
"""

def analyze_record_structure():
    """Estimate allocation overhead from Record structure"""
    print("\n" + "="*60)
    print("ALLOCATION ANALYSIS FROM SOURCE CODE")
    print("="*60 + "\n")
    
    # Typical record statistics
    avg_tags_per_record = 35
    avg_subfields_per_field = 6
    avg_string_length_value = 50
    
    # String overhead on 64-bit (with small string optimization)
    # String: 24 bytes (3 x 8byte ptrs for ptr, len, capacity)
    # But may use SSO for small strings <~24 bytes
    string_overhead = 24
    
    # Tag allocations
    tag_allocs = avg_tags_per_record
    tag_memory = tag_allocs * string_overhead  # Each tag is 3 bytes, but String overhead is ~24
    
    # Indicator allocations (if stored as String)
    # Currently stored as char, so no additional allocations
    
    # Subfield value allocations  
    subfield_allocs = avg_tags_per_record * avg_subfields_per_field
    subfield_memory = subfield_allocs * (string_overhead + avg_string_length_value)
    
    # Vec allocations
    field_vec_allocs = avg_tags_per_record
    field_vec_overhead_per_vec = 24  # Vec: 3 x 8byte ptrs
    field_vec_memory = field_vec_allocs * field_vec_overhead_per_vec
    
    # IndexMap allocation (one per record)
    indexmap_allocs = 1
    indexmap_memory = 200  # Approximate for 35 entries
    
    print(f"Per-record allocation analysis:")
    print(f"  Tag allocations:           {tag_allocs:3} (String overhead: {tag_memory:4} bytes)")
    print(f"  Subfield allocations:      {subfield_allocs:3} (overhead + value: {subfield_memory:6} bytes)")
    print(f"  Field Vec allocations:     {field_vec_allocs:3} (overhead: {field_vec_memory:4} bytes)")
    print(f"  IndexMap allocation:       {indexmap_allocs:3} (overhead: {indexmap_memory:4} bytes)")
    print(f"  {'─' * 60}")
    print(f"  Total allocations:         {tag_allocs + subfield_allocs + field_vec_allocs + indexmap_allocs:3}")
    print(f"  Total heap overhead:       {tag_memory + subfield_memory + field_vec_memory + indexmap_memory:6} bytes")
    
    # Optimization potential
    print(f"\nOptimization potential (per-record):")
    
    # SmallVec<[Subfield; 4]> - eliminates 70% of Vec allocations
    smallvec_saving_allocs = int(field_vec_allocs * 0.7)
    smallvec_saving_bytes = smallvec_saving_allocs * field_vec_overhead_per_vec
    print(f"  SmallVec<[Subfield; 4]>:  Save {smallvec_saving_allocs} allocs, {smallvec_saving_bytes} bytes")
    
    # Tag optimization: u16 instead of String
    tag_opt_saving_allocs = tag_allocs
    tag_opt_saving_bytes = tag_memory - (tag_allocs * 2)  # u16 = 2 bytes vs String = 24
    print(f"  Tags as u16:               Save {tag_opt_saving_allocs} allocs, {tag_opt_saving_bytes} bytes")
    
    # Combined savings
    total_savings_allocs = smallvec_saving_allocs + tag_opt_saving_allocs
    total_savings_bytes = smallvec_saving_bytes + tag_opt_saving_bytes
    print(f"  Combined optimization:     Save {total_savings_allocs} allocs, {total_savings_bytes} bytes/record")
    
    # For 10k records
    print(f"\nFor 10,000 records:")
    print(f"  Allocation reduction:      {total_savings_allocs * 10000:,} fewer allocations")
    print(f"  Memory reduction:          {total_savings_bytes * 10000 / 1024 / 1024:.1f} MB")


if __name__ == "__main__":
    print("MARC Record Allocation Profiling")
    print("=" * 60)
    analyze_record_structure()
