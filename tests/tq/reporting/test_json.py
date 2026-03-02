"""Tests for JSON reporting output."""

from __future__ import annotations

import json
from pathlib import Path
from typing import TYPE_CHECKING

from rich.console import Console

from tq.engine.models import EngineResult, Finding, FindingSummary, Severity
from tq.engine.rule_id import RuleId
from tq.reporting.json import print_json_report

if TYPE_CHECKING:
    import pytest


def test_print_json_report_renders_empty_payload(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """Render summary-only JSON when no findings exist."""
    console = Console(file=None, force_terminal=False)
    result = EngineResult(
        findings=(),
        summary=FindingSummary(errors=0, warnings=0, infos=0),
    )

    print_json_report(result=result, console=console, cwd=Path.cwd())
    payload = json.loads(capsys.readouterr().out)

    assert payload == {
        "findings": [],
        "summary": {
            "errors": 0,
            "warnings": 0,
            "infos": 0,
            "total": 0,
        },
    }


def test_print_json_report_renders_finding_fields(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """Render stable finding keys and values for machine consumers."""
    console = Console(file=None, force_terminal=False)
    result = EngineResult(
        findings=(
            Finding(
                rule_id=RuleId("mapping-missing-test"),
                severity=Severity.ERROR,
                message="missing test",
                path=Path("src/tq/foo.py"),
                line=7,
                suggestion="create tests/tq/test_foo.py",
            ),
        ),
        summary=FindingSummary(errors=1, warnings=0, infos=0),
    )

    print_json_report(result=result, console=console, cwd=Path.cwd())
    payload = json.loads(capsys.readouterr().out)

    assert payload["summary"] == {
        "errors": 1,
        "warnings": 0,
        "infos": 0,
        "total": 1,
    }
    assert payload["findings"] == [
        {
            "rule_id": "mapping-missing-test",
            "severity": "error",
            "message": "missing test",
            "path": "src/tq/foo.py",
            "line": 7,
            "suggestion": "create tests/tq/test_foo.py",
        },
    ]
