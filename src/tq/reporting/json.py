"""Machine-readable JSON reporting for tq diagnostics."""

from __future__ import annotations

import json
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path

    from rich.console import Console

    from tq.engine.models import EngineResult, Finding


def print_json_report(*, result: EngineResult, console: Console, cwd: Path) -> None:
    """Print deterministic JSON diagnostics payload.

    Args:
        result: Engine result to serialize.
        console: Rich console to write output to.
        cwd: Path used to relativize diagnostic paths.
    """
    payload = {
        "findings": [
            _finding_payload(finding=finding, cwd=cwd) for finding in result.findings
        ],
        "summary": {
            "errors": result.summary.errors,
            "warnings": result.summary.warnings,
            "infos": result.summary.infos,
            "total": result.summary.total,
        },
    }
    serialized = json.dumps(payload, ensure_ascii=False, separators=(",", ":"))
    console.file.write(f"{serialized}\n")


def _finding_payload(*, finding: Finding, cwd: Path) -> dict[str, str | int | None]:
    """Convert a finding into the stable JSON representation."""
    return {
        "rule_id": finding.rule_id.value,
        "severity": finding.severity.value,
        "message": finding.message,
        "path": _display_path(path=finding.path, cwd=cwd),
        "line": finding.line,
        "suggestion": finding.suggestion,
        "target": finding.target,
    }


def _display_path(*, path: Path, cwd: Path) -> str:
    """Render a path relative to cwd when possible."""
    try:
        return path.relative_to(cwd).as_posix()
    except ValueError:
        return path.as_posix()
