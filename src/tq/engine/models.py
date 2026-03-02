"""Engine finding and result models for tq diagnostics."""

from __future__ import annotations

from dataclasses import dataclass
from enum import StrEnum
from pathlib import Path


class Severity(StrEnum):
    """Diagnostic severity levels."""

    ERROR = "error"
    WARNING = "warning"
    INFO = "info"


@dataclass(frozen=True, slots=True)
class Finding:
    """A diagnostic produced by a rule evaluation.

    Attributes:
        rule_id: Stable rule identifier.
        severity: Severity of the diagnostic.
        message: Human-readable diagnostic message.
        path: Path to the relevant file.
        line: Optional 1-based line location.
        suggestion: Optional remediation guidance.
    """

    rule_id: str
    severity: Severity
    message: str
    path: Path
    line: int | None = None
    suggestion: str | None = None


@dataclass(frozen=True, slots=True)
class FindingSummary:
    """Aggregated counts of findings by severity."""

    errors: int
    warnings: int
    infos: int

    @property
    def total(self) -> int:
        """Return total number of findings."""
        return self.errors + self.warnings + self.infos


@dataclass(frozen=True, slots=True)
class EngineResult:
    """Result of a complete engine evaluation run."""

    findings: tuple[Finding, ...]
    summary: FindingSummary

    @property
    def has_errors(self) -> bool:
        """Return whether the result includes any error findings."""
        return self.summary.errors > 0
