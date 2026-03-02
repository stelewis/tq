"""Orphan check: Detect test files with no corresponding source module.

This check identifies unit test files that don't correspond to any
source module according to the naming convention.
"""

from __future__ import annotations

from pathlib import Path

from tq.tools.test_quality.config import TestQualityConfig
from tq.tools.test_quality.filesystem import FileIndex
from tq.tools.test_quality.models import Finding, Severity

NESTED_TEST_PATH_MIN_PARTS = 2


class OrphanCheck:
    """Check for orphaned test files.

    Orphaned tests are unit test files under tests/tq/**/test_*.py
    that do not correspond to any source module.
    """

    def __init__(self, file_index: FileIndex, config: TestQualityConfig):
        """Initialize the orphan check.

        Args:
            file_index: Index of source and test files.
            config: Configuration for the check.
        """
        self.file_index = file_index
        self.config = config

    def run(self) -> list[Finding]:
        """Execute the orphan check.

        Returns:
            List of findings for orphaned test files.
        """
        findings = []

        for test_file in self.file_index.test_files:
            # Skip integration and e2e tests
            test_parts = test_file.parts
            if "integration" in test_parts or "e2e" in test_parts:
                continue

            # Skip __init__ tests if ignore_init is enabled
            if self.config.ignore_init and test_file.name == "test___init__.py":
                continue

            if not self._has_corresponding_source(test_file):
                findings.append(
                    Finding(
                        category="orphaned_test",
                        severity=Severity.WARNING,
                        path=self.file_index.test_root / test_file,
                        message=(
                            f"Test file has no corresponding source module: {test_file}"
                        ),
                        suggestion=(
                            "Verify this test is still needed or move to integration/"
                        ),
                    )
                )

        return findings

    def _build_source_path(self, parts: tuple[str, ...], source_name: str) -> Path:
        """Build source path from test file parts.

        Args:
            parts: Test file path parts (relative to test root).
            source_name: Source file name.

        Returns:
            Source file path relative to source root.
        """
        if len(parts) > NESTED_TEST_PATH_MIN_PARTS:
            # tests/tq/pipeline/utils/test_foo.py
            # -> pipeline/utils/foo.py
            return Path(*parts[1:-1]) / source_name
        else:
            # tests/tq/test_foo.py -> foo.py
            return Path(source_name)

    def _has_corresponding_source(self, test_file: Path) -> bool:
        """Check if a test file has a corresponding source file.

        Args:
            test_file: Test file path relative to test root.

        Returns:
            True if a corresponding source file exists.
        """
        # For tests/tq/<path>/test_<module>.py (or qualified variants),
        # look for src/tq/<path>/<module>.py.

        parts = test_file.parts
        if not parts or parts[0] != "tq":
            # Can't determine source for tests outside tq/
            return True  # Don't flag as orphan if we can't determine

        test_name = test_file.name
        if not test_name.startswith("test_"):
            return True  # Not a standard test file

        # Extract module name from test_<module>.py or test_<module>_<qualifier>.py
        module_part = test_name[5:]  # Remove "test_" prefix
        module_base = module_part.replace(".py", "")

        # Special case: test___init__.py (literal __init__ after test_)
        # This tests the __init__.py in the same directory
        if module_base == "__init__":
            # tests/tq/config/test___init__.py -> config/__init__.py
            if len(parts) > NESTED_TEST_PATH_MIN_PARTS:
                source_path = Path(*parts[1:-1]) / "__init__.py"
                if source_path in self.file_index.source_files:
                    return True
            # tests/tq/test___init__.py -> __init__.py (root package)
            source_path = Path("__init__.py")
            if source_path in self.file_index.source_files:
                return True

        # Build source path by removing 'tq' prefix and 'test_' from
        # filename. tests/tq/pipeline/test_foo.py -> pipeline/foo.py
        source_name = f"{module_base}.py"
        source_path = self._build_source_path(parts, source_name)

        if source_path in self.file_index.source_files:
            return True

        # Try removing qualifiers: test_foo_validation.py -> foo.py We need to
        # be smart about this - try progressively removing words from the end.
        # If allowed_qualifiers is configured, only treat suffixes in that
        # allowlist as legitimate qualifiers; otherwise, allow any suffix.
        if "_" in module_base:
            module_parts = module_base.split("_")
            # Try from longest to shortest (e.g., foo_bar_baz -> foo_bar -> foo)
            for i in range(len(module_parts) - 1, 0, -1):
                potential_module = "_".join(module_parts[:i])
                suffix = "_".join(module_parts[i:])

                if self.config.allowed_qualifiers and suffix not in set(
                    self.config.allowed_qualifiers
                ):
                    continue

                source_name = f"{potential_module}.py"
                source_path = self._build_source_path(parts, source_name)

                if source_path in self.file_index.source_files:
                    return True

        # Check for __init__.py if the test might be for a package init
        if module_base.endswith("_init"):
            # test_pipeline_init.py in tests/tq/pipeline/
            # -> pipeline/__init__.py
            # Extract pkg_name from module_base
            parts_without_init = module_base[:-5]  # Remove "_init"

            # The package name could be in the path or in the test name
            # tests/tq/pipeline/test_pipeline_init.py
            # -> pipeline/__init__.py
            if len(parts) > NESTED_TEST_PATH_MIN_PARTS:
                # Check if the parent directory matches
                parent_dir = parts[-2]  # e.g., "pipeline"
                if parent_dir == parts_without_init:
                    # The test is in the same directory as the package
                    # tests/tq/pipeline/test_pipeline_init.py
                    # -> pipeline/__init__.py
                    source_path = Path(*parts[1:-1]) / "__init__.py"
                    if source_path in self.file_index.source_files:
                        return True

            # Also try treating the pkg name as a subdirectory
            # tests/tq/test_pipeline_init.py -> pipeline/__init__.py
            source_path = Path(parts_without_init) / "__init__.py"
            if source_path in self.file_index.source_files:
                return True

        return False
