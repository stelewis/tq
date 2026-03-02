"""Tests for test-file-too-large built-in rule."""

from __future__ import annotations

from pathlib import Path

import pytest

from tq.discovery.index import AnalysisIndex
from tq.engine.context import AnalysisContext
from tq.engine.models import Severity
from tq.rules.file_too_large import FileTooLargeRule


def test_size_rule_counts_non_blank_non_comment_lines(tmp_path: Path) -> None:
    """Warn when non-blank non-comment lines exceed configured limit."""
    test_root = tmp_path / "tests"
    test_root.mkdir()
    test_file = test_root / "tq" / "test_big.py"
    test_file.parent.mkdir(parents=True)
    test_file.write_text(
        "\n# comment\n\n"
        "def test_one():\n"
        "    assert True\n"
        "\n"
        "def test_two():\n"
        "    assert True\n",
        encoding="utf-8",
    )

    context = AnalysisContext.create(
        index=AnalysisIndex.create(
            source_root=tmp_path / "src" / "tq",
            test_root=test_root,
            source_files=[Path("alpha.py")],
            test_files=[Path("tq/test_big.py")],
        )
    )

    findings = FileTooLargeRule(max_non_blank_lines=3).evaluate(context)

    assert len(findings) == 1
    finding = findings[0]
    assert finding.rule_id.value == "test-file-too-large"
    assert finding.severity is Severity.WARNING
    assert finding.path == test_file


def test_size_rule_rejects_non_positive_threshold() -> None:
    """Reject invalid size threshold configuration."""
    with pytest.raises(ValueError):
        FileTooLargeRule(max_non_blank_lines=0)


def test_size_rule_emits_warning_for_unreadable_file(tmp_path: Path) -> None:
    """Emit warning finding instead of raising on unreadable test files."""
    test_root = tmp_path / "tests"
    test_root.mkdir()
    test_file = test_root / "tq" / "test_bad_encoding.py"
    test_file.parent.mkdir(parents=True)
    test_file.write_bytes(b"\xff\xfe\xfa")

    context = AnalysisContext.create(
        index=AnalysisIndex.create(
            source_root=tmp_path / "src" / "tq",
            test_root=test_root,
            source_files=[Path("alpha.py")],
            test_files=[Path("tq/test_bad_encoding.py")],
        )
    )

    findings = FileTooLargeRule(max_non_blank_lines=3).evaluate(context)

    assert len(findings) == 1
    assert findings[0].rule_id.value == "test-file-too-large"
    assert "Could not read test file" in findings[0].message
