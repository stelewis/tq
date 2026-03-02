"""Runner for orchestrating test quality checks.

This module coordinates running all configured checks and collecting
their findings.
"""

from __future__ import annotations

from pathlib import Path

from tq.tools.test_quality.checks.mapping_check import MappingCheck
from tq.tools.test_quality.checks.orphan_check import OrphanCheck
from tq.tools.test_quality.checks.size_check import SizeCheck
from tq.tools.test_quality.checks.structure_check import StructureCheck
from tq.tools.test_quality.config import TestQualityConfig
from tq.tools.test_quality.filesystem import FileIndex, scan_files
from tq.tools.test_quality.models import Finding, Severity


class TestQualityRunner:
    """Orchestrates test quality checks.

    This class coordinates scanning files, running checks, and collecting
    findings.
    """

    def __init__(
        self,
        source_root: Path,
        test_root: Path,
        config: TestQualityConfig | None = None,
    ):
        """Initialize the runner.

        Args:
            source_root: Root directory for source files.
            test_root: Root directory for test files.
            config: Optional configuration. If None, loads from pyproject.toml.
        """
        self.source_root = source_root
        self.test_root = test_root
        self.config = config or TestQualityConfig.from_pyproject()
        self.file_index: FileIndex | None = None
        self.findings: list[Finding] = []

    def run(self) -> list[Finding]:
        """Run all checks and collect findings.

        Returns:
            List of all findings from all checks.
        """
        # Scan files
        self.file_index = scan_files(
            self.source_root,
            self.test_root,
            ignore_patterns=self.config.ignore,
            ignore_init=self.config.ignore_init,
        )

        self.findings = []

        # Run checks
        self._run_check(MappingCheck(self.file_index, self.config))
        self._run_check(StructureCheck(self.file_index, self.config))
        self._run_check(SizeCheck(self.file_index, self.config))
        self._run_check(OrphanCheck(self.file_index, self.config))

        # Check for redundant ignore patterns
        self._check_redundant_ignores()

        return self.findings

    def _run_check(self, check) -> None:
        """Run a single check and collect its findings.

        Args:
            check: Check instance to run.
        """
        findings = check.run()
        self.findings.extend(findings)

    def _check_redundant_ignores(self) -> None:
        """Check for ignore patterns that don't match any files."""
        if not self.file_index or not self.config.ignore:
            return

        # Track which patterns matched at least one file
        matched_patterns = set()

        all_files = [
            *self.file_index.source_files,
            *self.file_index.test_files,
        ]

        for pattern in self.config.ignore:
            for file_path in all_files:
                if pattern in matched_patterns:
                    break
                if file_path.match(pattern) or str(file_path) == pattern:
                    matched_patterns.add(pattern)
                    break

        # Report unmatched patterns
        for pattern in self.config.ignore:
            if pattern not in matched_patterns:
                self.findings.append(
                    Finding(
                        category="redundant_ignore",
                        severity=Severity.WARNING,
                        path=Path("pyproject.toml"),
                        message=(
                            f"Ignore pattern '{pattern}' does not match any files"
                        ),
                        suggestion=(
                            "Remove this pattern from tool.test_quality.ignore"
                        ),
                    )
                )

    def has_errors(self) -> bool:
        """Check if any findings are errors.

        Returns:
            True if any findings have ERROR severity.
        """
        return any(f.severity == Severity.ERROR for f in self.findings)

    def get_summary(self) -> dict[str, int]:
        """Get a summary count of findings by severity.

        Returns:
            Dictionary mapping severity to count.
        """
        summary = {
            Severity.ERROR.value: 0,
            Severity.WARNING.value: 0,
            Severity.INFO.value: 0,
        }

        for finding in self.findings:
            summary[finding.severity.value] += 1

        return summary
