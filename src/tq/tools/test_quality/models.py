"""Data models for test quality checks.

This module defines the core data structures used throughout the test quality
checking system.
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import StrEnum
from pathlib import Path


class Severity(StrEnum):
    """Severity levels for findings."""

    ERROR = "error"
    WARNING = "warning"
    INFO = "info"


@dataclass(frozen=True)
class Finding:
    """A test quality issue found during checks.

    Attributes:
        category: Classification of the issue (e.g., 'mapping_missing',
            'structure_mismatch').
        severity: Severity level of the issue.
        path: Path to the file with the issue.
        message: Human-readable description of the issue.
        suggestion: Optional suggestion for how to fix the issue.
    """

    category: str
    severity: Severity
    path: Path
    message: str
    suggestion: str | None = None

    def __str__(self) -> str:
        """Return a formatted string representation."""
        parts = [
            f"[{self.severity.value.upper()}] {self.category}",
            f"  File: {self.path}",
            f"  {self.message}",
        ]
        if self.suggestion:
            parts.append(f"  Suggestion: {self.suggestion}")
        return "\n".join(parts)
