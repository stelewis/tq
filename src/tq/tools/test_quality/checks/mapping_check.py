"""Mapping check: Enforce 1:1 test-to-module mapping.

This check ensures that each source module has at least one corresponding
test module following the naming conventions in docs/developer/testing.md.
"""

from __future__ import annotations

from pathlib import Path

from tq.tools.test_quality.config import TestQualityConfig
from tq.tools.test_quality.filesystem import FileIndex
from tq.tools.test_quality.models import Finding, Severity


class MappingCheck:
    """Check that every source module has at least one test module.

    For each src/tq/<path>/<module>.py, require at least one
    matching test in tests/tq/<path>/test_<module>.py or
    tests/tq/<path>/test_<module>_<qualifier>.py.
    """

    def __init__(self, file_index: FileIndex, config: TestQualityConfig):
        """Initialize the mapping check.

        Args:
            file_index: Index of source and test files.
            config: Configuration for the check.
        """
        self.file_index = file_index
        self.config = config
        self.test_package_root = Path(self.file_index.source_root.name)

    def run(self) -> list[Finding]:
        """Execute the mapping check.

        Returns:
            List of findings for source files without matching tests.
        """
        findings = []

        for source_file in self.file_index.source_files:
            if not self._has_matching_test(source_file):
                expected_test = self._get_expected_test_path(source_file)
                findings.append(
                    Finding(
                        category="mapping_missing",
                        severity=Severity.ERROR,
                        path=self.file_index.source_root / source_file,
                        message=f"No test file found for source module: {source_file}",
                        suggestion=f"Create test file at: tests/{expected_test}",
                    )
                )

        return findings

    def _has_matching_test(self, source_file: Path) -> bool:
        """Check if a source file has a matching test file.

        Args:
            source_file: Source file path relative to source root.

        Returns:
            True if at least one matching test exists.
        """
        expected_patterns = self._get_expected_test_patterns(source_file)

        for test_file in self.file_index.test_files:
            # Check for exact match or pattern match with qualifier
            test_path_str = str(test_file)
            for pattern in expected_patterns:
                if test_path_str == pattern or test_path_str.startswith(
                    pattern.replace(".py", "_")
                ):
                    return True

        return False

    def _get_expected_test_patterns(self, source_file: Path) -> list[str]:
        """Get expected test file patterns for a source file.

        Args:
            source_file: Source file path relative to source root
                (src/tq/).

        Returns:
            List of possible test file patterns relative to test root.
        """
        # For src/tq/<path>/<module>.py (stored as <path>/<module>.py)
        # Expect tests/tq/<path>/test_<module>.py
        # or tests/tq/<path>/test_<module>_<qualifier>.py

        # Source files are relative to src/tq/, so we need to add
        # tq/ prefix for the test path

        # Handle __init__.py specially if not ignored
        if source_file.name == "__init__.py":
            # For src/tq/foo/__init__.py (stored as foo/__init__.py)
            # Expect tests/tq/foo/test_foo_init.py
            if source_file.parent != Path("."):
                parent_name = source_file.parent.name
                return [
                    str(
                        self.test_package_root
                        / source_file.parent
                        / f"test_{parent_name}_init.py"
                    )
                ]
            return []

        # Standard module maps to tq/<path>/test_<module>.py
        module_name = source_file.stem
        expected_name = f"test_{module_name}.py"

        if source_file.parent != Path("."):
            expected_path = self.test_package_root / source_file.parent / expected_name
        else:
            expected_path = self.test_package_root / expected_name

        return [str(expected_path)]

    def _get_expected_test_path(self, source_file: Path) -> Path:
        """Get the primary expected test path for a source file.

        Args:
            source_file: Source file path relative to source root.

        Returns:
            Expected test file path relative to test root.
        """
        patterns = self._get_expected_test_patterns(source_file)
        if patterns:
            return Path(patterns[0])
        return self.test_package_root / source_file
