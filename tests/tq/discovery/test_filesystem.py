"""Tests for filesystem-backed discovery helpers."""

from __future__ import annotations

from pathlib import Path

from tq.discovery.filesystem import build_analysis_index


def test_build_analysis_index_discovers_source_and_tests(tmp_path: Path) -> None:
    """Collect source and test files as relative paths in deterministic index."""
    source_root = tmp_path / "src" / "tq"
    test_root = tmp_path / "tests"

    _write(source_root / "engine" / "runner.py")
    _write(test_root / "tq" / "engine" / "test_runner.py")

    index = build_analysis_index(source_root=source_root, test_root=test_root)

    assert index.source_files == (Path("engine/runner.py"),)
    assert index.test_files == (Path("tq/engine/test_runner.py"),)


def test_build_analysis_index_ignores_pycache(tmp_path: Path) -> None:
    """Skip files under __pycache__ while walking discovery roots."""
    source_root = tmp_path / "src" / "tq"
    test_root = tmp_path / "tests"

    _write(source_root / "__pycache__" / "cached.py")
    _write(source_root / "engine" / "runner.py")
    _write(test_root / "__pycache__" / "test_cached.py")
    _write(test_root / "tq" / "engine" / "test_runner.py")

    index = build_analysis_index(source_root=source_root, test_root=test_root)

    assert index.source_files == (Path("engine/runner.py"),)
    assert index.test_files == (Path("tq/engine/test_runner.py"),)


def _write(path: Path) -> None:
    """Create a file and parent directories for discovery fixtures."""
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("pass\n", encoding="utf-8")
