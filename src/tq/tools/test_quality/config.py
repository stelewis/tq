"""Configuration for test quality checks.

This module handles loading and managing configuration from pyproject.toml.
"""

from __future__ import annotations

import tomllib
from dataclasses import dataclass, field
from pathlib import Path


@dataclass
class TestQualityConfig:
    """Configuration for test quality checks.

    Attributes:
        ignore: List of file paths or glob patterns to skip during checks.
        max_test_lines: Maximum allowed non-blank lines in a test file.
        ignore_init: Whether to ignore __init__.py files in mapping checks.
        allowed_qualifiers: Allowed qualifier suffixes for test modules.
            When set, test_foo_<qualifier>.py is only treated as targeting
            foo.py if <qualifier> is in this allowlist.
    """

    ignore: list[str] = field(default_factory=list)
    max_test_lines: int = 600
    ignore_init: bool = False
    allowed_qualifiers: list[str] = field(default_factory=list)

    @classmethod
    def from_pyproject(cls, pyproject_path: Path | None = None) -> TestQualityConfig:
        """Load configuration from pyproject.toml.

        Args:
            pyproject_path: Path to pyproject.toml. If None, searches for it in
                the current directory and parent directories.

        Returns:
            TestQualityConfig instance with loaded settings.
        """
        if pyproject_path is None:
            pyproject_path = cls._find_pyproject()

        if not pyproject_path or not pyproject_path.exists():
            return cls()

        with open(pyproject_path, "rb") as f:
            data = tomllib.load(f)

        config_data = data.get("tool", {}).get("test_quality", {})

        return cls(
            ignore=config_data.get("ignore", []),
            max_test_lines=config_data.get("max_test_lines", 600),
            ignore_init=config_data.get("ignore_init", False),
            allowed_qualifiers=config_data.get("allowed_qualifiers", []),
        )

    @staticmethod
    def _find_pyproject() -> Path | None:
        """Find pyproject.toml in current or parent directories.

        Returns:
            Path to pyproject.toml if found, None otherwise.
        """
        current = Path.cwd()
        for parent in [current, *current.parents]:
            candidate = parent / "pyproject.toml"
            if candidate.exists():
                return candidate
        return None
