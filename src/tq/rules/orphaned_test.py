"""Orphaned test rule for unit tests without matching source modules."""

from __future__ import annotations

from pathlib import Path

from tq.engine.context import AnalysisContext
from tq.engine.models import Finding, Severity
from tq.engine.rule_id import RuleId
from tq.rules.qualifiers import QualifierStrategy, candidate_module_names


class OrphanedTestRule:
    """Emit findings for unit tests that do not map to any source module."""

    def __init__(
        self,
        *,
        qualifier_strategy: QualifierStrategy,
        allowed_qualifiers: tuple[str, ...] = (),
    ) -> None:
        """Initialize orphan rule with explicit qualifier policy.

        Args:
            qualifier_strategy: Strategy for handling test name qualifiers.
            allowed_qualifiers: Allowed qualifier suffixes when using
                ``QualifierStrategy.ALLOWLIST``.

        Raises:
            ValueError: If strategy is allowlist but no qualifiers are provided.
        """
        if qualifier_strategy is QualifierStrategy.ALLOWLIST and not allowed_qualifiers:
            raise ValueError(
                "allowed_qualifiers must be non-empty for allowlist strategy"
            )

        self._qualifier_strategy = qualifier_strategy
        self._allowed_qualifiers = tuple(sorted(set(allowed_qualifiers)))

    @property
    def rule_id(self) -> RuleId:
        """Return stable rule identifier."""
        return RuleId("orphaned-test")

    def evaluate(self, context: AnalysisContext) -> tuple[Finding, ...]:
        """Evaluate orphaned unit tests against the immutable index."""
        package_name = context.index.source_root.name
        source_files = set(context.index.source_files)
        findings: list[Finding] = []

        for test_file in context.index.test_files:
            if _is_non_unit_test_path(test_file):
                continue

            if not _is_unit_test_filename(test_file.name):
                continue

            if test_file.parts[0] != package_name:
                continue

            if self._has_corresponding_source(
                test_file=test_file,
                source_files=source_files,
            ):
                continue

            findings.append(
                Finding(
                    rule_id=self.rule_id,
                    severity=Severity.WARNING,
                    message=(
                        f"Test file has no corresponding source module: {test_file}"
                    ),
                    path=context.index.test_root / test_file,
                    suggestion=(
                        "Verify this test is still needed or move it "
                        "to integration/e2e scope"
                    ),
                )
            )

        return tuple(findings)

    def _has_corresponding_source(
        self,
        *,
        test_file: Path,
        source_files: set[Path],
    ) -> bool:
        """Check whether a unit test resolves to any source module."""
        relative_source_dir = Path(*test_file.parts[1:-1])
        for module_name in candidate_module_names(
            module_stem=test_file.stem[5:],
            qualifier_strategy=self._qualifier_strategy,
            allowed_qualifiers=self._allowed_qualifiers,
        ):
            source_file = relative_source_dir / f"{module_name}.py"
            if source_file in source_files:
                return True

        return False


def _is_non_unit_test_path(test_file: Path) -> bool:
    """Check whether path belongs to integration or e2e test scopes."""
    return "integration" in test_file.parts or "e2e" in test_file.parts


def _is_unit_test_filename(filename: str) -> bool:
    """Check if filename follows unit test naming shape."""
    return filename.startswith("test_") and filename.endswith(".py")
