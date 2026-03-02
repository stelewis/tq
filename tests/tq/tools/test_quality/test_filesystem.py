"""Tests for test_quality.filesystem module."""

from __future__ import annotations

from pathlib import Path

import pytest

from tq.tools.test_quality.filesystem import (
    count_non_blank_lines,
    scan_files,
    should_ignore,
)


def test_count_non_blank_lines(tmp_path: Path):
    """Test counting non-blank lines in a file."""
    test_file = tmp_path / "test.py"
    test_file.write_text(
        """
# Comment
def foo():
    pass

# Another comment

def bar():
    return 42
"""
    )

    count = count_non_blank_lines(test_file)
    # Should count non-blank, non-comment-only lines:
    # "def foo():", "pass", "def bar():", "return 42" = 4 lines
    assert count == 4


def test_count_non_blank_lines_all_blank(tmp_path: Path):
    """Test counting lines in a file with only blank lines."""
    test_file = tmp_path / "test.py"
    test_file.write_text("\n\n\n")

    count = count_non_blank_lines(test_file)
    assert count == 0


def test_count_non_blank_lines_file_not_found():
    """Test counting lines in a non-existent file."""
    count = count_non_blank_lines(Path("/nonexistent/file.py"))
    assert count == 0


def test_should_ignore_exact_match():
    """Test ignore pattern with exact match."""
    path = Path("foo/bar.py")
    patterns = ["foo/bar.py"]
    root = Path(".")

    assert should_ignore(path, patterns, root) is True


def test_should_ignore_glob_pattern():
    """Test ignore pattern with glob."""
    path = Path("tests/integration/test_foo.py")
    patterns = ["**/integration/**"]
    root = Path(".")

    assert should_ignore(path, patterns, root) is True


def test_should_ignore_no_match():
    """Test path that doesn't match any pattern."""
    path = Path("foo/bar.py")
    patterns = ["baz/**", "qux.py"]
    root = Path(".")

    assert should_ignore(path, patterns, root) is False


def test_should_ignore_empty_patterns():
    """Test with empty pattern list."""
    path = Path("foo/bar.py")
    patterns = []
    root = Path(".")

    assert should_ignore(path, patterns, root) is False


@pytest.fixture
def temp_project(tmp_path: Path) -> tuple[Path, Path]:
    """Create a temporary project structure."""
    src = tmp_path / "src" / "tq"
    tests = tmp_path / "tests"

    src.mkdir(parents=True)
    tests.mkdir(parents=True)

    return src, tests


def test_scan_files_basic(temp_project: tuple[Path, Path]):
    """Test basic file scanning."""
    src, tests = temp_project

    # Create some source files
    (src / "foo.py").write_text("# foo")
    (src / "bar").mkdir()
    (src / "bar" / "baz.py").write_text("# baz")

    # Create some test files
    (tests / "test_foo.py").write_text("# test foo")
    (tests / "bar").mkdir()
    (tests / "bar" / "test_baz.py").write_text("# test baz")

    index = scan_files(src, tests)

    assert len(index.source_files) == 2
    assert Path("foo.py") in index.source_files
    assert Path("bar/baz.py") in index.source_files

    assert len(index.test_files) == 2
    assert Path("test_foo.py") in index.test_files
    assert Path("bar/test_baz.py") in index.test_files


def test_scan_files_with_ignore(temp_project: tuple[Path, Path]):
    """Test file scanning with ignore patterns."""
    src, tests = temp_project

    (src / "foo.py").write_text("# foo")
    (src / "bar.py").write_text("# bar")
    (tests / "test_foo.py").write_text("# test foo")
    (tests / "test_bar.py").write_text("# test bar")

    index = scan_files(src, tests, ignore_patterns=["**/bar.py", "test_bar.py"])

    assert Path("foo.py") in index.source_files
    assert Path("bar.py") not in index.source_files

    assert Path("test_foo.py") in index.test_files
    # Note: test files only match test_*.py pattern, and ignore is applied after
    assert Path("test_bar.py") not in index.test_files


def test_scan_files_ignore_init(temp_project: tuple[Path, Path]):
    """Test file scanning with ignore_init flag."""
    src, tests = temp_project

    (src / "__init__.py").write_text("")
    (src / "foo.py").write_text("# foo")

    index = scan_files(src, tests, ignore_init=True)

    assert Path("foo.py") in index.source_files
    assert Path("__init__.py") not in index.source_files


def test_scan_files_include_init(temp_project: tuple[Path, Path]):
    """Test file scanning including __init__.py files."""
    src, tests = temp_project

    (src / "__init__.py").write_text("")
    (src / "foo.py").write_text("# foo")

    index = scan_files(src, tests, ignore_init=False)

    assert Path("foo.py") in index.source_files
    assert Path("__init__.py") in index.source_files


def test_file_index_attributes(temp_project: tuple[Path, Path]):
    """Test FileIndex attributes."""
    src, tests = temp_project

    (src / "foo.py").write_text("# foo")
    (tests / "test_foo.py").write_text("# test foo")

    index = scan_files(src, tests)

    assert index.source_root == src
    assert index.test_root == tests
    assert isinstance(index.source_files, list)
    assert isinstance(index.test_files, list)
