"""Structure check: Detect misplaced test files.

This check identifies test files that exist but are not in the correct
location according to the naming conventions.
"""

from __future__ import annotations

from pathlib import Path

from tq.tools.test_quality.config import TestQualityConfig
from tq.tools.test_quality.filesystem import FileIndex
from tq.tools.test_quality.models import Finding, Severity


class StructureCheck:
    """Check that test files are located in the correct directory structure.

    Test files should mirror the source structure:
    src/tq/<path>/<module>.py
    -> tests/tq/<path>/test_<module>.py
    """

    def __init__(self, file_index: FileIndex, config: TestQualityConfig):
        """Initialize the structure check.

        Args:
            file_index: Index of source and test files.
            config: Configuration for the check.
        """
        self.file_index = file_index
        self.config = config

    def run(self) -> list[Finding]:
        """Execute the structure check.

        Returns:
            List of findings for misplaced test files.
        """
        findings = []

        for test_file in self.file_index.test_files:
            # Skip integration and e2e tests
            test_parts = test_file.parts
            if "integration" in test_parts or "e2e" in test_parts:
                continue

            expected_location = self._get_expected_location(test_file)
            if expected_location and expected_location != test_file:
                findings.append(
                    Finding(
                        category="structure_mismatch",
                        severity=Severity.WARNING,
                        path=self.file_index.test_root / test_file,
                        message="Test file is not in the expected location",
                        suggestion=f"Move to: tests/{expected_location}",
                    )
                )

        return findings

    def _get_expected_location(self, test_file: Path) -> Path | None:
        """Determine the expected location for a test file.

        Args:
            test_file: Test file path relative to test root.

        Returns:
            Expected path relative to test root, or None if cannot determine or
            if the file is already in the expected location.
        """
        # Test files should follow: tests/tq/<path>/test_<module>.py
        # They must be under tests/tq/ to match the source structure

        parts = test_file.parts
        if not parts:
            return None

        # Check if it starts with 'tq'
        if parts[0] != "tq":
            # If the test file is not under tq/, try to infer the
            # correct path from the test file name
            test_name = test_file.name
            if test_name.startswith("test_"):
                # Suggest moving to tests/tq/test_<module>.py
                return Path("tq") / test_name

        # Files already under tq/ are considered correctly structured
        # Further validation of exact location would require matching against
        # actual source files, which is done by the orphan check
        return None
