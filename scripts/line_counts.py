#!/usr/bin/env python3
"""
Count lines in Rust source files and output sorted by line count (descending).
Used for identifying files that may need refactoring.
"""

import os
from pathlib import Path


def count_lines(file_path):
    """Count the number of lines in a file."""
    with open(file_path, 'r', encoding='utf-8') as f:
        return len(f.readlines())


def main():
    # Get the src directory relative to the script location
    script_dir = Path(__file__).parent
    src_dir = script_dir.parent / 'src'

    if not src_dir.exists():
        print(f"Error: {src_dir} does not exist")
        return

    # Find all .rs files recursively
    rust_files = list(src_dir.rglob('*.rs'))

    if not rust_files:
        print(f"No Rust files found in {src_dir}")
        return

    # Count lines for each file
    file_line_counts = []
    for file_path in rust_files:
        line_count = count_lines(file_path)
        # Store relative path from src directory
        relative_path = file_path.relative_to(src_dir.parent)
        file_line_counts.append((str(relative_path), line_count))

    # Sort by line count (descending)
    file_line_counts.sort(key=lambda x: x[1], reverse=True)

    # Output results
    print(f"{'File':<60} {'Lines':>10}")
    print("-" * 70)
    for file_path, line_count in file_line_counts:
        print(f"{file_path:<60} {line_count:>10}")

    # Output summary
    total_lines = sum(count for _, count in file_line_counts)
    print("-" * 70)
    print(f"{'Total':<60} {total_lines:>10}")
    print(f"\nTotal files: {len(file_line_counts)}")


if __name__ == '__main__':
    main()
