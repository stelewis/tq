"""Filesystem-backed discovery helpers for tq CLI runs."""

from __future__ import annotations

from pathlib import Path

from tq.discovery.index import AnalysisIndex


def build_analysis_index(*, source_root: Path, test_root: Path) -> AnalysisIndex:
    """Build immutable analysis index from source and test trees."""
    source_files = _scan_python_files(source_root)
    test_files = _scan_test_files(test_root)
    return AnalysisIndex.create(
        source_root=source_root,
        test_root=test_root,
        source_files=source_files,
        test_files=test_files,
    )


def _scan_python_files(root: Path) -> list[Path]:
    """Scan relative Python module paths under root."""
    files: list[Path] = []
    for path in root.rglob("*.py"):
        if _is_ignored_path(path):
            continue
        files.append(path.relative_to(root))
    return files


def _scan_test_files(root: Path) -> list[Path]:
    """Scan relative unit/integration test module paths under root."""
    files: list[Path] = []
    for path in root.rglob("test_*.py"):
        if _is_ignored_path(path):
            continue
        files.append(path.relative_to(root))
    return files


def _is_ignored_path(path: Path) -> bool:
    """Check if discovered path should be ignored for index generation."""
    return "__pycache__" in path.parts
