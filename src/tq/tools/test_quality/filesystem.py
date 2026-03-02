"""Filesystem utilities for scanning source and test trees.

This module provides utilities for discovering and indexing Python source
files and test files, with support for ignore patterns.
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path


@dataclass
class FileIndex:
    """Index of source and test files.

    Attributes:
        source_files: List of source file paths relative to source root.
        test_files: List of test file paths relative to test root.
        source_root: Root directory for source files.
        test_root: Root directory for test files.
    """

    source_files: list[Path]
    test_files: list[Path]
    source_root: Path
    test_root: Path


def count_non_blank_lines(path: Path) -> int:
    """Count non-blank, non-comment-only lines in a Python file.

    Args:
        path: Path to the Python file.

    Returns:
        Number of non-blank lines, excluding comment-only lines.
    """
    try:
        with open(path, encoding="utf-8") as f:
            count = 0
            for line in f:
                stripped = line.strip()
                if stripped and not stripped.startswith("#"):
                    count += 1
            return count
    except OSError, UnicodeDecodeError:
        return 0


def should_ignore(path: Path, patterns: list[str], root: Path) -> bool:
    """Check if a path should be ignored based on patterns.

    Args:
        path: Path to check (relative to root).
        patterns: List of glob patterns or exact paths to ignore.
        root: Root directory for resolving relative patterns.

    Returns:
        True if the path matches any ignore pattern.
    """
    path_str = str(path)
    for pattern in patterns:
        # Try as a glob pattern
        if path.match(pattern):
            return True
        # Try as an exact match
        if path_str == pattern or str(path) == pattern:
            return True
        # Try matching against the full path from root
        full_path = root / path
        if full_path.match(pattern):
            return True
    return False


def scan_files(
    source_root: Path,
    test_root: Path,
    *,
    ignore_patterns: list[str] | None = None,
    ignore_init: bool = False,
) -> FileIndex:
    """Scan and index source and test files.

    Args:
        source_root: Root directory for source files.
        test_root: Root directory for test files.
        ignore_patterns: Optional list of glob patterns to ignore.
        ignore_init: Whether to ignore __init__.py files.

    Returns:
        FileIndex with discovered files.
    """
    ignore_patterns = ignore_patterns or []

    source_files = []
    for path in source_root.rglob("*.py"):
        rel_path = path.relative_to(source_root)

        if ignore_init and path.name == "__init__.py":
            continue

        if should_ignore(rel_path, ignore_patterns, source_root):
            continue

        source_files.append(rel_path)

    test_files = []
    for path in test_root.rglob("test_*.py"):
        rel_path = path.relative_to(test_root)

        if should_ignore(rel_path, ignore_patterns, test_root):
            continue

        test_files.append(rel_path)

    return FileIndex(
        source_files=sorted(source_files),
        test_files=sorted(test_files),
        source_root=source_root,
        test_root=test_root,
    )
