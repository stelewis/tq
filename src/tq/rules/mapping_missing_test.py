"""Mapping rule for missing unit tests.

This rule enforces that every discovered source module has at least one
corresponding unit test module.
"""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING

from tq.engine.models import Finding, Severity
from tq.engine.rule_id import RuleId
from tq.rules.qualifiers import QualifierStrategy, candidate_module_names

if TYPE_CHECKING:
    from tq.engine.context import AnalysisContext


class MappingMissingTestRule:
    """Emit findings for source modules without corresponding tests."""

    def __init__(
        self,
        *,
        ignore_init_modules: bool,
        qualifier_strategy: QualifierStrategy = QualifierStrategy.ANY_SUFFIX,
        allowed_qualifiers: tuple[str, ...] = (),
    ) -> None:
        """Initialize rule with explicit mapping policy.

        Args:
            ignore_init_modules: Skip ``__init__.py`` source modules when
                checking mapping coverage.
            qualifier_strategy: Strategy for qualified test names.
            allowed_qualifiers: Allowed qualifiers when using allowlist strategy.

        Raises:
            ValueError: If allowlist strategy has no allowed qualifiers.
        """
        if qualifier_strategy is QualifierStrategy.ALLOWLIST and not allowed_qualifiers:
            msg = "allowed_qualifiers must be non-empty for allowlist strategy"
            raise ValueError(
                msg,
            )

        self._ignore_init_modules = ignore_init_modules
        self._qualifier_strategy = qualifier_strategy
        self._allowed_qualifiers = tuple(sorted(set(allowed_qualifiers)))

    @property
    def rule_id(self) -> RuleId:
        """Return stable rule identifier."""
        return RuleId("mapping-missing-test")

    def evaluate(self, context: AnalysisContext) -> tuple[Finding, ...]:
        """Evaluate mapping coverage against the immutable analysis index."""
        package_name = context.index.source_root.name
        findings: list[Finding] = []

        for source_file in context.index.source_files:
            if self._ignore_init_modules and source_file.name == "__init__.py":
                continue

            if self._has_matching_test(
                source_file=source_file,
                test_files=context.index.test_files,
                package_name=package_name,
            ):
                continue

            expected_test_path = self._expected_test_path(
                source_file=source_file,
                package_name=package_name,
            )
            findings.append(
                Finding(
                    rule_id=self.rule_id,
                    severity=Severity.ERROR,
                    message=f"No test file found for source module: {source_file}",
                    path=context.index.source_root / source_file,
                    suggestion=f"Create test file at: {expected_test_path.as_posix()}",
                ),
            )

        return tuple(findings)

    def _has_matching_test(
        self,
        *,
        source_file: Path,
        test_files: tuple[Path, ...],
        package_name: str,
    ) -> bool:
        """Check whether at least one matching test path exists."""
        expected_path = self._expected_test_path(
            source_file=source_file,
            package_name=package_name,
        )
        source_stem = source_file.stem

        for test_file in test_files:
            if test_file.parent != expected_path.parent:
                continue

            if not test_file.name.startswith("test_"):
                continue

            module_stem = test_file.stem[5:]
            candidates = candidate_module_names(
                module_stem=module_stem,
                qualifier_strategy=self._qualifier_strategy,
                allowed_qualifiers=self._allowed_qualifiers,
            )
            if source_stem in candidates:
                return True

        return False

    def _expected_test_path(self, *, source_file: Path, package_name: str) -> Path:
        """Build canonical expected unit test path for a source module."""
        if source_file.name == "__init__.py":
            stem = "test___init__"
        else:
            stem = f"test_{source_file.stem}"

        return Path(package_name) / source_file.parent / f"{stem}.py"
