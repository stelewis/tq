"""Human-readable terminal reporting for tq check."""

from __future__ import annotations

from typing import TYPE_CHECKING

from tq.engine.models import EngineResult, Finding, Severity

if TYPE_CHECKING:
    from pathlib import Path

    from rich.console import Console


def print_report(
    *,
    result: EngineResult,
    console: Console,
    cwd: Path,
    include_suggestions: bool = False,
) -> None:
    """Print findings followed by a concise severity summary."""
    if not result.findings:
        console.print("[bold cyan]All checks passed![/bold cyan]")
        return

    for finding in result.findings:
        console.print(
            _render_finding(
                finding=finding,
                cwd=cwd,
                include_suggestions=include_suggestions,
            ),
        )

    summary = result.summary
    console.print(
        "Summary: "
        f"{summary.errors} error(s), "
        f"{summary.warnings} warning(s), "
        f"{summary.infos} info finding(s)",
    )


def _render_finding(
    *,
    finding: Finding,
    cwd: Path,
    include_suggestions: bool,
) -> str:
    """Render a single finding as one terminal line."""
    try:
        display_path = finding.path.relative_to(cwd).as_posix()
    except ValueError:
        display_path = finding.path.as_posix()

    line_part = f":{finding.line}" if finding.line is not None else ""
    severity_style = {
        Severity.ERROR: "[red]error[/red]",
        Severity.WARNING: "[yellow]warning[/yellow]",
        Severity.INFO: "[blue]info[/blue]",
    }[finding.severity]

    rendered = (
        f"{_target_prefix(finding.target)}{display_path}{line_part}: "
        f"{severity_style} "
        f"({finding.rule_id.value}) {finding.message}"
    )

    if include_suggestions and finding.suggestion:
        return f"{rendered} (suggestion: {finding.suggestion})"

    return rendered


def _target_prefix(target_name: str | None) -> str:
    """Render a stable target prefix when a finding has target context."""
    if target_name is None:
        return ""
    return f"target={target_name} "
