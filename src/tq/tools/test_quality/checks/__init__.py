"""Checks subpackage for test quality validation.

Each module in this package implements a specific check that validates
test suite organization and quality.
"""

from __future__ import annotations

from typing import Protocol

from tq.tools.test_quality.models import Finding


class Check(Protocol):
    """Protocol for test quality checks.

    All checks should implement a run() method that returns a list of findings.
    """

    def run(self) -> list[Finding]:
        """Execute the check and return findings.

        Returns:
            List of findings discovered by this check.
        """
        ...
