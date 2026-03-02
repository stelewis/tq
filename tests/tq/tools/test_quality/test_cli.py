"""Tests for CLI module."""

from __future__ import annotations

from pathlib import Path
from unittest.mock import MagicMock

import pytest

from tq.tools.test_quality.cli import (
    main,
    print_findings_table,
    print_summary,
)
from tq.tools.test_quality.models import Finding, Severity


def test_print_findings_table_empty(capsys):
    """Test printing empty findings table."""
    console = MagicMock()
    findings = []

    print_findings_table(findings, console)

    # Should print success message
    console.print.assert_called_once()
    call_args = console.print.call_args[0][0]
    assert "No issues found" in call_args or "✓" in call_args


def test_print_findings_table_with_findings():
    """Test printing findings table with findings."""
    console = MagicMock()
    findings = [
        Finding(
            category="test_category",
            severity=Severity.ERROR,
            path=Path("test/file.py"),
            message="Test message",
            suggestion="Test suggestion",
        )
    ]

    print_findings_table(findings, console)

    # Should create and print table
    console.print.assert_called()


def test_print_summary_all_passed():
    """Test summary when all checks passed."""
    console = MagicMock()
    summary = {"error": 0, "warning": 0, "info": 0}

    print_summary(summary, console)

    # Should print success message
    console.print.assert_called()


def test_print_summary_with_issues():
    """Test summary with errors and warnings."""
    console = MagicMock()
    summary = {"error": 2, "warning": 3, "info": 1}

    print_summary(summary, console)

    # Should print summary with counts
    console.print.assert_called()


@pytest.fixture
def temp_project(tmp_path: Path, monkeypatch: pytest.MonkeyPatch):
    """Create a temporary project structure for CLI tests."""
    src = tmp_path / "src" / "tq"
    tests = tmp_path / "tests"
    src.mkdir(parents=True)
    tests.mkdir(parents=True)

    # Create a pyproject.toml
    pyproject = tmp_path / "pyproject.toml"
    pyproject.write_text(
        """
[tool.test_quality]
ignore_init = true
"""
    )

    # Change to the temp directory
    monkeypatch.chdir(tmp_path)

    return src, tests


def test_main_success(temp_project: tuple[Path, Path]):
    """Test main function with successful validation."""
    src, tests = temp_project

    # Create source and test
    (src / "foo.py").write_text("# foo")
    (tests / "tq").mkdir()
    (tests / "tq" / "test_foo.py").write_text("# test foo")

    exit_code = main()
    assert exit_code == 0


def test_main_with_errors(temp_project: tuple[Path, Path]):
    """Test main function with validation errors."""
    src, _tests = temp_project

    # Create source without test
    (src / "foo.py").write_text("# foo")

    exit_code = main()
    assert exit_code == 1


def test_main_source_root_not_found(tmp_path: Path, monkeypatch: pytest.MonkeyPatch):
    """Test main function when source root doesn't exist."""
    monkeypatch.chdir(tmp_path)

    exit_code = main()
    assert exit_code == 1


def test_main_test_root_not_found(tmp_path: Path, monkeypatch: pytest.MonkeyPatch):
    """Test main function when test root doesn't exist."""
    src = tmp_path / "src" / "tq"
    src.mkdir(parents=True)

    monkeypatch.chdir(tmp_path)

    exit_code = main()
    assert exit_code == 1
