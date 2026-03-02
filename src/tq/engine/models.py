"""Engine finding and result models for tq diagnostics."""

from __future__ import annotations

from dataclasses import dataclass
from enum import StrEnum
from types import MappingProxyType
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path

    from tq.engine.rule_id import RuleId


class Severity(StrEnum):
    """Diagnostic severity levels."""

    ERROR = "error"
    WARNING = "warning"
    INFO = "info"


SEVERITY_ORDER = MappingProxyType(
    {
        Severity.ERROR: 0,
        Severity.WARNING: 1,
        Severity.INFO: 2,
    },
)


def severity_rank(severity: Severity) -> int:
    """Return stable numeric rank for severity ordering."""
    return SEVERITY_ORDER[severity]


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

    rule_id: RuleId
    severity: Severity
    message: str
    path: Path
    line: int | None = None
    suggestion: str | None = None

    def __post_init__(self) -> None:
        """Validate finding invariants.

        Raises:
            ValueError: If required values are missing or invalid.
        """
        if not self.message.strip():
            msg = "Finding message must be non-empty"
            raise ValueError(msg)

        if self.line is not None and self.line < 1:
            msg = "Finding line must be >= 1 when provided"
            raise ValueError(msg)


@dataclass(frozen=True, slots=True)
class FindingSummary:
    """Aggregated counts of findings by severity."""

    errors: int
    warnings: int
    infos: int

    def __post_init__(self) -> None:
        """Validate summary counters.

        Raises:
            ValueError: If any summary count is negative.
        """
        if self.errors < 0 or self.warnings < 0 or self.infos < 0:
            msg = "FindingSummary counts must be non-negative"
            raise ValueError(msg)

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
