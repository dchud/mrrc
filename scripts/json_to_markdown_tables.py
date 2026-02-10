#!/usr/bin/env python3
"""
Convert benchmark comparison JSON to markdown tables.

This utility demonstrates the new JSON structure's capability to generate
markdown tables for automated report generation and visualization.

Usage:
    python3 scripts/json_to_markdown_tables.py [--benchmark NAME]
    
Examples:
    # Generate all tables
    python3 scripts/json_to_markdown_tables.py
    
    # Generate table for specific benchmark
    python3 scripts/json_to_markdown_tables.py --benchmark read_1k
"""

import json
import sys
from pathlib import Path
from typing import Dict, List, Optional


class BenchmarkTableGenerator:
    """Generate markdown tables from benchmark JSON."""
    
    def __init__(self, json_path: Path):
        """Load benchmark data from JSON."""
        with open(json_path) as f:
            self.data = json.load(f)
    
    def generate_benchmark_table(self, benchmark_id: str) -> str:
        """
        Generate markdown table for a single benchmark.
        
        Args:
            benchmark_id: ID of benchmark (e.g., 'read_1k')
        
        Returns:
            Markdown table string
        """
        # Find benchmark
        bench = None
        for b in self.data['benchmarks']:
            if b['id'] == benchmark_id:
                bench = b
                break
        
        if not bench:
            raise ValueError(f"Benchmark {benchmark_id} not found")
        
        lines = []
        lines.append(f"### {bench['title']}\n")
        
        # Main table
        lines.append("| Implementation | Time | Throughput | Relative | Notes |")
        lines.append("|---|---|---|---|---|")
        
        for impl in bench['implementations']:
            time_str = f"{impl['time_ms']:.3f} ms"
            throughput_str = f"{impl['throughput_rec_s']:,} rec/s"
            relative_str = f"{impl['relative']:.2f}x"
            
            lines.append(
                f"| **{impl['name']}** | {time_str} | {throughput_str} | {relative_str} | {impl['notes']} |"
            )
        
        # Analysis section
        if 'analysis' in bench and bench['analysis']:
            analysis = bench['analysis']
            if 'pymrrc_vs_pymarc' in analysis:
                comp = analysis['pymrrc_vs_pymarc']
                speedup = comp.get('speedup', 0)
                improvement = comp.get('improvement_percent', 0)
                lines.append(f"\n**Analysis:**")
                lines.append(f"- **pymrrc is {speedup:.1f}x faster than pymarc** ({improvement:.1f}% improvement)")
            
            if 'rust_vs_pymarc' in analysis:
                comp = analysis['rust_vs_pymarc']
                speedup = comp.get('speedup', 0)
                improvement = comp.get('improvement_percent', 0)
                lines.append(f"- **Rust is {speedup:.1f}x faster than pymarc** ({improvement:.1f}% improvement)")
            
            if 'note' in analysis:
                lines.append(f"\n{analysis['note']}")
        
        return '\n'.join(lines)
    
    def generate_all_tables(self) -> str:
        """Generate markdown tables for all benchmarks."""
        lines = []
        lines.append("# Benchmark Results Tables\n")
        lines.append(f"*Generated from: .benchmarks/comparison.json*\n")
        
        for bench in self.data['benchmarks']:
            lines.append(self.generate_benchmark_table(bench['id']))
            lines.append("")
        
        # Add summary
        lines.append("## Summary\n")
        summary = self.data['summary']
        lines.append(f"- **Average pymrrc speedup vs pymarc**: {summary['average_speedup_pymrrc_vs_pymarc']:.1f}x")
        lines.append(f"- **Best case**: {summary['best_case_pymrrc_vs_pymarc']:.2f}x")
        lines.append(f"- **Worst case**: {summary['worst_case_pymrrc_vs_pymarc']:.2f}x")
        lines.append(f"- **pymrrc is {summary['average_pymrrc_percent_of_rust']:.0%} of Rust performance**\n")
        
        lines.append("### Key Findings\n")
        for finding in summary['key_findings']:
            lines.append(f"- {finding}")
        
        return '\n'.join(lines)


def main():
    """Main entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(
        description="Convert benchmark JSON to markdown tables",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__
    )
    parser.add_argument(
        '--benchmark',
        help='Generate table for specific benchmark (e.g., read_1k)',
        default=None
    )
    parser.add_argument(
        '--json',
        help='Path to comparison.json file',
        default='.benchmarks/comparison.json'
    )
    parser.add_argument(
        '--output',
        help='Output file (default: stdout)',
        default=None
    )
    
    args = parser.parse_args()
    
    json_path = Path(args.json)
    if not json_path.exists():
        print(f"Error: {json_path} not found", file=sys.stderr)
        sys.exit(1)
    
    generator = BenchmarkTableGenerator(json_path)
    
    if args.benchmark:
        output = generator.generate_benchmark_table(args.benchmark)
    else:
        output = generator.generate_all_tables()
    
    if args.output:
        Path(args.output).write_text(output)
        print(f"✓ Written to {args.output}")
    else:
        print(output)


if __name__ == '__main__':
    main()
