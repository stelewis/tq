"""Size rule for oversized test modules."""

from __future__ import annotations

from pathlib import Path

from tq.engine.context import AnalysisContext
from tq.engine.models import Finding, Severity
from tq.engine.rule_id import RuleId


class FileTooLargeRule:
    """Emit findings for test files above an explicit line budget."""

    def __init__(self, *, max_non_blank_lines: int) -> None:
        """Initialize rule with explicit non-blank line threshold.

        Args:
            max_non_blank_lines: Maximum allowed count of non-blank,
                non-comment-only lines per test file.

        Raises:
            ValueError: If threshold is less than 1.
        """
        if max_non_blank_lines < 1:
            raise ValueError("max_non_blank_lines must be >= 1")
        self._max_non_blank_lines = max_non_blank_lines

    @property
    def rule_id(self) -> RuleId:
        """Return stable rule identifier."""
        return RuleId("test-file-too-large")

    def evaluate(self, context: AnalysisContext) -> tuple[Finding, ...]:
        """Evaluate test file sizes using explicit counting policy."""
        findings: list[Finding] = []

        for test_file in context.index.test_files:
            full_path = context.index.test_root / test_file
            line_count = _count_non_blank_non_comment_lines(full_path)
            if line_count is None:
                findings.append(
                    Finding(
                        rule_id=self.rule_id,
                        severity=Severity.WARNING,
                        message=(
                            "Could not read test file for size check "
                            f"(path: {test_file.as_posix()})"
                        ),
                        path=full_path,
                        suggestion="Ensure file exists and is UTF-8 decodable",
                    )
                )
                continue

            if line_count <= self._max_non_blank_lines:
                continue

            findings.append(
                Finding(
                    rule_id=self.rule_id,
                    severity=Severity.WARNING,
                    message=(
                        "Test file is too large "
                        f"({line_count} lines, "
                        f"limit: {self._max_non_blank_lines})"
                    ),
                    path=full_path,
                    suggestion="Split this module into smaller focused test files",
                )
            )

        return tuple(findings)


def _count_non_blank_non_comment_lines(path: Path) -> int | None:
    """Count policy: non-blank lines excluding comment-only lines."""
    line_count = 0
    try:
        with path.open(encoding="utf-8") as handle:
            for line in handle:
                stripped = line.strip()
                if not stripped:
                    continue
                if stripped.startswith("#"):
                    continue
                line_count += 1
        return line_count
    except OSError, UnicodeDecodeError:
        return None
