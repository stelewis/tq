"""Command-line interface for test quality checks.

This module provides the CLI entry point for running test quality checks.
"""

from __future__ import annotations

import sys
from pathlib import Path

from rich.console import Console
from rich.table import Table

from tq.tools.test_quality.config import TestQualityConfig
from tq.tools.test_quality.models import Severity
from tq.tools.test_quality.runner import TestQualityRunner


def print_findings_table(findings: list, console: Console) -> None:
    """Print findings in a formatted table.

    Args:
        findings: List of Finding objects to display.
        console: Rich console for output.
    """
    if not findings:
        console.print("[green]✓ No issues found![/green]")
        return

    table = Table(title="Test Quality Findings")
    table.add_column("Severity", style="bold")
    table.add_column("Category", style="cyan")
    table.add_column("File", style="blue")
    table.add_column("Message")

    # Sort by severity (errors first), then category
    severity_order = {Severity.ERROR: 0, Severity.WARNING: 1, Severity.INFO: 2}
    sorted_findings = sorted(
        findings, key=lambda f: (severity_order[f.severity], f.category)
    )

    for finding in sorted_findings:
        severity_color = {
            Severity.ERROR: "red",
            Severity.WARNING: "yellow",
            Severity.INFO: "blue",
        }[finding.severity]

        severity_text = (
            f"[{severity_color}]{finding.severity.value.upper()}[/{severity_color}]"
        )

        # Try to make paths relative for display
        try:
            display_path = finding.path.relative_to(Path.cwd())
        except ValueError:
            display_path = finding.path

        message = finding.message
        if finding.suggestion:
            message = f"{message}\n💡 {finding.suggestion}"

        table.add_row(
            severity_text,
            finding.category,
            str(display_path),
            message,
        )

    console.print(table)


def print_summary(summary: dict[str, int], console: Console) -> None:
    """Print a summary of findings.

    Args:
        summary: Dictionary mapping severity to count.
        console: Rich console for output.
    """
    console.print()
    error_count = summary[Severity.ERROR.value]
    warning_count = summary[Severity.WARNING.value]
    info_count = summary[Severity.INFO.value]

    parts = []
    if error_count > 0:
        parts.append(f"[red]{error_count} error(s)[/red]")
    if warning_count > 0:
        parts.append(f"[yellow]{warning_count} warning(s)[/yellow]")
    if info_count > 0:
        parts.append(f"[blue]{info_count} info[/blue]")

    if parts:
        console.print(f"Found: {', '.join(parts)}")
    else:
        console.print("[green]✓ All checks passed![/green]")


def main() -> int:
    """Run test quality checks from the command line.

    Returns:
        Exit code (0 for success, 1 if errors found).
    """
    console = Console()

    # Default paths
    source_root = Path("src/tq")
    test_root = Path("tests")

    # Validate paths exist
    if not source_root.exists():
        console.print(f"[red]Error: Source root not found: {source_root}[/red]")
        return 1

    if not test_root.exists():
        console.print(f"[red]Error: Test root not found: {test_root}[/red]")
        return 1

    # Load configuration
    config = TestQualityConfig.from_pyproject()

    # Run checks
    console.print("[bold]Running test quality checks...[/bold]\n")

    runner = TestQualityRunner(source_root, test_root, config)
    findings = runner.run()

    # Display results
    print_findings_table(findings, console)
    print_summary(runner.get_summary(), console)

    # Exit with error code if any errors found
    return 1 if runner.has_errors() else 0


if __name__ == "__main__":
    sys.exit(main())
