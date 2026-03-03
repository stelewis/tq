"""Tests for terminal reporting output."""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING

from rich.console import Console

from tq.engine.models import EngineResult, Finding, FindingSummary, Severity
from tq.engine.rule_id import RuleId
from tq.reporting.terminal import print_report

if TYPE_CHECKING:
    import pytest


def test_print_report_all_clear_matches_tooling_style(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """Print only the standard pass line when there are no findings."""
    console = Console(file=None, force_terminal=False)
    result = EngineResult(
        findings=(),
        summary=FindingSummary(errors=0, warnings=0, infos=0),
    )

    print_report(result=result, console=console, cwd=Path.cwd())
    captured = capsys.readouterr()

    assert captured.out.strip() == "All checks passed!"


def test_print_report_hides_suggestions_by_default(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """Do not render suggestion text unless explicitly requested."""
    console = Console(file=None, force_terminal=False)
    result = EngineResult(
        findings=(
            Finding(
                rule_id=RuleId("mapping-missing-test"),
                severity=Severity.ERROR,
                message="missing test",
                path=Path("src/tq/foo.py"),
                suggestion="create test file",
            ),
        ),
        summary=FindingSummary(errors=1, warnings=0, infos=0),
    )

    print_report(result=result, console=console, cwd=Path.cwd())
    captured = capsys.readouterr()

    assert "suggestion:" not in captured.out


def test_print_report_can_show_suggestions(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """Render suggestion text when explicitly enabled."""
    console = Console(file=None, force_terminal=False)
    result = EngineResult(
        findings=(
            Finding(
                rule_id=RuleId("mapping-missing-test"),
                severity=Severity.ERROR,
                message="missing test",
                path=Path("src/tq/foo.py"),
                suggestion="create test file",
            ),
        ),
        summary=FindingSummary(errors=1, warnings=0, infos=0),
    )

    print_report(
        result=result,
        console=console,
        cwd=Path.cwd(),
        include_suggestions=True,
    )
    captured = capsys.readouterr()

    assert "suggestion:" in captured.out


def test_print_report_renders_target_prefix_when_present(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """Render target context prefix in each finding line."""
    console = Console(file=None, force_terminal=False)
    result = EngineResult(
        findings=(
            Finding(
                rule_id=RuleId("mapping-missing-test"),
                severity=Severity.ERROR,
                message="missing test",
                path=Path("src/tq/foo.py"),
                target="core",
            ),
        ),
        summary=FindingSummary(errors=1, warnings=0, infos=0),
    )

    print_report(result=result, console=console, cwd=Path.cwd())
    captured = capsys.readouterr()

    assert "target=core src/tq/foo.py" in captured.out
