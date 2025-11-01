#!/usr/bin/env python3
"""
Find tests in src/ files that should be moved to tests/ directory.

This script scans Rust source files in src/ and identifies test modules
(#[cfg(test)] or mod tests) that should be moved to the tests/ directory
according to the project's testing guidelines.
"""

import os
import re
from pathlib import Path
from typing import List, Tuple


def find_rust_files(directory: str) -> List[Path]:
    """Find all Rust source files in the given directory."""
    path = Path(directory)
    return list(path.rglob("*.rs"))


def has_test_module(content: str) -> bool:
    """Check if the file contains a test module."""
    # Look for #[cfg(test)] or mod tests
    patterns = [
        r"#\[cfg\(test\)\]",
        r"mod\s+tests\s*\{",
        r"mod\s+\w+_tests\s*\{",
    ]
    return any(re.search(pattern, content) for pattern in patterns)


def count_test_functions(content: str) -> int:
    """Count the number of test functions in the content."""
    # Look for #[test] annotations
    return len(re.findall(r"#\[test\]", content))


def extract_test_info(file_path: Path) -> Tuple[int, List[str]]:
    """Extract test information from a file."""
    content = file_path.read_text()

    if not has_test_module(content):
        return 0, []

    test_count = count_test_functions(content)

    # Extract test function names
    test_names = []
    test_pattern = r"#\[test\]\s*(?:fn\s+(\w+))"
    for match in re.finditer(test_pattern, content):
        test_names.append(match.group(1))

    return test_count, test_names


def main():
    """Main function to find and report misplaced tests."""
    # Get the project root directory (parent of scripts/)
    script_dir = Path(__file__).parent
    project_root = script_dir.parent
    src_dir = project_root / "src"

    if not src_dir.exists():
        print(f"Error: {src_dir} directory not found")
        return

    rust_files = find_rust_files(str(src_dir))

    print("=" * 80)
    print("MISPLACED TESTS REPORT")
    print("=" * 80)
    print()
    print("According to project guidelines, tests should be in tests/ directory,")
    print("not in implementation files.")
    print()

    total_tests = 0
    files_with_tests = []

    for file_path in sorted(rust_files):
        test_count, test_names = extract_test_info(file_path)

        if test_count > 0:
            total_tests += test_count
            relative_path = file_path.relative_to(project_root)
            files_with_tests.append((relative_path, test_count, test_names))

    if files_with_tests:
        print(f"Found {len(files_with_tests)} files with tests ({total_tests} total tests):")
        print()

        for file_path, test_count, test_names in files_with_tests:
            print(f"📄 {file_path}")
            print(f"   Tests: {test_count}")
            if test_names:
                print(f"   Functions:")
                for name in test_names[:5]:  # Show first 5
                    print(f"     - {name}")
                if len(test_names) > 5:
                    print(f"     ... and {len(test_names) - 5} more")
            print()
    else:
        print("✅ No tests found in src/ files - all tests are properly placed!")

    print("=" * 80)
    print(f"Summary: {total_tests} tests in {len(files_with_tests)} files")
    print("=" * 80)


if __name__ == "__main__":
    main()
