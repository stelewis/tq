"""Tests for test_quality.models module."""

from __future__ import annotations

from pathlib import Path

import pytest

from tq.tools.test_quality.models import Finding, Severity


def test_severity_enum():
    """Test Severity enum values."""
    assert Severity.ERROR.value == "error"
    assert Severity.WARNING.value == "warning"
    assert Severity.INFO.value == "info"


def test_finding_creation():
    """Create and validate a Finding instance."""
    finding = Finding(
        category="test_category",
        severity=Severity.ERROR,
        path=Path("test/file.py"),
        message="Test message",
        suggestion="Test suggestion",
    )

    assert finding.category == "test_category"
    assert finding.severity == Severity.ERROR
    assert finding.path == Path("test/file.py")
    assert finding.message == "Test message"
    assert finding.suggestion == "Test suggestion"


def test_finding_without_suggestion():
    """Test Finding without optional suggestion."""
    finding = Finding(
        category="test_category",
        severity=Severity.WARNING,
        path=Path("test/file.py"),
        message="Test message",
    )

    assert finding.suggestion is None


def test_finding_str_with_suggestion():
    """Test Finding string representation with suggestion."""
    finding = Finding(
        category="test_category",
        severity=Severity.ERROR,
        path=Path("test/file.py"),
        message="Test message",
        suggestion="Test suggestion",
    )

    result = str(finding)
    assert "[ERROR] test_category" in result
    assert "File: test/file.py" in result
    assert "Test message" in result
    assert "Suggestion: Test suggestion" in result


def test_finding_str_without_suggestion():
    """Test Finding string representation without suggestion."""
    finding = Finding(
        category="test_category",
        severity=Severity.INFO,
        path=Path("test/file.py"),
        message="Test message",
    )

    result = str(finding)
    assert "[INFO] test_category" in result
    assert "File: test/file.py" in result
    assert "Test message" in result
    assert "Suggestion:" not in result


def test_finding_immutable():
    """Test that Finding instances are frozen."""
    finding = Finding(
        category="test_category",
        severity=Severity.ERROR,
        path=Path("test/file.py"),
        message="Test message",
    )

    with pytest.raises(AttributeError):
        finding.category = "new_category"  # type: ignore[misc]
