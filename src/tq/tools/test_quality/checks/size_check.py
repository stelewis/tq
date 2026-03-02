"""Size check: Warn about large test files.

This check identifies test files that exceed the configured maximum
line count, suggesting they should be split.
"""

from __future__ import annotations

from tq.tools.test_quality.config import TestQualityConfig
from tq.tools.test_quality.filesystem import (
    FileIndex,
    count_non_blank_lines,
)
from tq.tools.test_quality.models import Finding, Severity


class SizeCheck:
    """Check for test files that are too large.

    Large test files (exceeding max_test_lines) should be split into
    smaller, more focused test modules.
    """

    def __init__(self, file_index: FileIndex, config: TestQualityConfig):
        """Initialize the size check.

        Args:
            file_index: Index of source and test files.
            config: Configuration for the check.
        """
        self.file_index = file_index
        self.config = config

    def run(self) -> list[Finding]:
        """Execute the size check.

        Returns:
            List of findings for oversized test files.
        """
        findings = []

        for test_file in self.file_index.test_files:
            full_path = self.file_index.test_root / test_file
            line_count = count_non_blank_lines(full_path)

            if line_count > self.config.max_test_lines:
                findings.append(
                    Finding(
                        category="large_test_file",
                        severity=Severity.WARNING,
                        path=full_path,
                        message=(
                            f"Test file is too large ({line_count} lines, "
                            f"limit: {self.config.max_test_lines})"
                        ),
                        suggestion=(
                            "Consider splitting into smaller, more focused test files"
                        ),
                    )
                )

        return findings
