"""Tests for immutable analysis index model."""

from __future__ import annotations

from pathlib import Path

import pytest

from tq.discovery.index import AnalysisIndex


def test_create_sorts_files_and_freezes_collections() -> None:
    """Create index with deterministic ordering and immutable tuples."""
    index = AnalysisIndex.create(
        source_root=Path("src/tq"),
        test_root=Path("tests"),
        source_files=[Path("z.py"), Path("a.py")],
        test_files=[Path("tq/test_z.py"), Path("tq/test_a.py")],
    )

    assert index.source_files == (Path("a.py"), Path("z.py"))
    assert index.test_files == (Path("tq/test_a.py"), Path("tq/test_z.py"))


def test_create_normalizes_roots_and_deduplicates_paths() -> None:
    """Normalize roots and remove duplicate file paths deterministically."""
    index = AnalysisIndex.create(
        source_root=Path("src") / ".." / "src" / "tq",
        test_root=Path("tests") / ".",
        source_files=[Path("a.py"), Path("a.py")],
        test_files=[Path("tq/test_a.py"), Path("tq/test_a.py")],
    )

    assert index.source_root.is_absolute()
    assert index.test_root.is_absolute()
    assert index.source_files == (Path("a.py"),)
    assert index.test_files == (Path("tq/test_a.py"),)


def test_index_is_frozen() -> None:
    """Reject mutation attempts for immutable index model."""
    index = AnalysisIndex.create(
        source_root=Path("src/tq"),
        test_root=Path("tests"),
        source_files=[Path("a.py")],
        test_files=[Path("tq/test_a.py")],
    )

    with pytest.raises(AttributeError):
        index.source_root = Path("src/other")  # type: ignore[misc]
