"""Tests for engine result and finding models."""

from __future__ import annotations

from pathlib import Path

from tq.engine.models import EngineResult, Finding, FindingSummary, Severity


def test_finding_summary_total() -> None:
    """Return total count from severity buckets."""
    summary = FindingSummary(errors=1, warnings=2, infos=3)
    assert summary.total == 6


def test_engine_result_has_errors() -> None:
    """Expose error presence through convenience property."""
    finding = Finding(
        rule_id="mapping-missing-test",
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
