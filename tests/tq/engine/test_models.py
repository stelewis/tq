"""Tests for engine result and finding models."""

from __future__ import annotations

from pathlib import Path

import pytest

from tq.engine.models import (
    EngineResult,
    Finding,
    FindingSummary,
    Severity,
    severity_rank,
)
from tq.engine.rule_id import RuleId


def test_finding_summary_total() -> None:
    """Return total count from severity buckets."""
    summary = FindingSummary(errors=1, warnings=2, infos=3)
    assert summary.total == 6


def test_engine_result_has_errors() -> None:
    """Expose error presence through convenience property."""
    finding = Finding(
        rule_id=RuleId("mapping-missing-test"),
        severity=Severity.ERROR,
        message="missing test",
        path=Path("src/tq/foo.py"),
        line=10,
    )
    result = EngineResult(
        findings=(finding,),
        summary=FindingSummary(errors=1, warnings=0, infos=0),
    )

    assert result.has_errors is True


def test_finding_rejects_invalid_line() -> None:
    """Reject findings with invalid line numbers."""
    with pytest.raises(ValueError):
        Finding(
            rule_id=RuleId("mapping-missing-test"),
            severity=Severity.ERROR,
            message="bad line",
            path=Path("src/tq/foo.py"),
            line=0,
        )


def test_finding_rejects_blank_message_or_rule_id() -> None:
    """Reject findings with blank identifiers or messages."""
    with pytest.raises(ValueError):
        RuleId("")

    with pytest.raises(ValueError):
        Finding(
            rule_id=RuleId("mapping-missing-test"),
            severity=Severity.ERROR,
            message="   ",
            path=Path("src/tq/foo.py"),
        )


def test_summary_rejects_negative_counts() -> None:
    """Reject negative summary counters."""
    with pytest.raises(ValueError):
        FindingSummary(errors=-1, warnings=0, infos=0)


def test_severity_rank_ordering() -> None:
    """Rank severities in canonical error-warning-info order."""
    assert severity_rank(Severity.ERROR) < severity_rank(Severity.WARNING)
    assert severity_rank(Severity.WARNING) < severity_rank(Severity.INFO)
