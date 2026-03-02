"""Tests for size_check module."""

from __future__ import annotations

import warnings
from pathlib import Path

import pytest

from tq.tools.test_quality.checks.size_check import SizeCheck
from tq.tools.test_quality.config import TestQualityConfig
from tq.tools.test_quality.filesystem import scan_files
from tq.tools.test_quality.models import Severity

# Suppress expected warnings about test_quality classes that pytest tries to collect
warnings.filterwarnings("ignore", category=pytest.PytestCollectionWarning)


def test_size_check_small_file(tmp_path: Path):
    """Test size check with a small test file."""
    src = tmp_path / "src" / "tq"
    tests = tmp_path / "tests"
    src.mkdir(parents=True)
    tests.mkdir(parents=True)

    # Create small test file
    (tests / "test_foo.py").write_text("def test_foo():\n    pass\n")

    index = scan_files(src, tests)
    config = TestQualityConfig(max_test_lines=600)
    check = SizeCheck(index, config)

    findings = check.run()
    assert len(findings) == 0


def test_size_check_large_file(tmp_path: Path):
    """Test size check with a large test file."""
    src = tmp_path / "src" / "tq"
    tests = tmp_path / "tests"
    src.mkdir(parents=True)
    tests.mkdir(parents=True)

    # Create large test file (> 600 lines)
    large_content = "\n".join([f"def test_{i}():\n    pass" for i in range(400)])
    (tests / "test_large.py").write_text(large_content)

    index = scan_files(src, tests)
    config = TestQualityConfig(max_test_lines=600)
    check = SizeCheck(index, config)

    findings = check.run()
    assert len(findings) == 1
    assert findings[0].category == "large_test_file"
    assert findings[0].severity == Severity.WARNING
    assert "large" in str(findings[0].path)


def test_size_check_custom_limit(tmp_path: Path):
    """Test size check with custom line limit."""
    src = tmp_path / "src" / "tq"
    tests = tmp_path / "tests"
    src.mkdir(parents=True)
    tests.mkdir(parents=True)

    # Create test file with ~50 lines
    content = "\n".join([f"def test_{i}():\n    pass" for i in range(25)])
    (tests / "test_medium.py").write_text(content)

    index = scan_files(src, tests)
    config = TestQualityConfig(max_test_lines=30)
    check = SizeCheck(index, config)

    findings = check.run()
    assert len(findings) == 1
    assert findings[0].category == "large_test_file"


def test_size_check_exactly_at_limit(tmp_path: Path):
    """Test size check with file exactly at the limit."""
    src = tmp_path / "src" / "tq"
    tests = tmp_path / "tests"
    src.mkdir(parents=True)
    tests.mkdir(parents=True)

    # Create test file with exactly max_test_lines non-blank lines
    # Each "def test_X(): pass" is 2 non-blank lines
    # So for 600 lines, we need 300 test functions
    content = "\n".join([f"def test_{i}():\n    pass" for i in range(300)])
    (tests / "test_exact.py").write_text(content)

    index = scan_files(src, tests)
    config = TestQualityConfig(max_test_lines=600)
    check = SizeCheck(index, config)

    findings = check.run()
    # Should not flag (exactly at limit, not over)
    assert len(findings) == 0


def test_size_check_blank_lines_not_counted(tmp_path: Path):
    """Test that blank lines are not counted."""
    src = tmp_path / "src" / "tq"
    tests = tmp_path / "tests"
    src.mkdir(parents=True)
    tests.mkdir(parents=True)

    # Create file with many blank lines but few code lines
    content = "def test_foo():\n    pass\n" + "\n" * 1000
    (tests / "test_blanks.py").write_text(content)

    index = scan_files(src, tests)
    config = TestQualityConfig(max_test_lines=600)
    check = SizeCheck(index, config)

    findings = check.run()
    # Should not flag (only 2 non-blank lines)
    assert len(findings) == 0
