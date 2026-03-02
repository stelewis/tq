"""Tests for mapping-missing-test built-in rule."""

from __future__ import annotations

from pathlib import Path

from tq.discovery.index import AnalysisIndex
from tq.engine.context import AnalysisContext
from tq.engine.models import Severity
from tq.rules.mapping_missing_test import MappingMissingTestRule
from tq.rules.qualifiers import QualifierStrategy


def test_mapping_rule_emits_error_for_unmapped_source() -> None:
    """Emit mapping finding when a source module has no matching test."""
    context = AnalysisContext.create(
        index=AnalysisIndex.create(
            source_root=Path("src/tq"),
            test_root=Path("tests"),
            source_files=[Path("alpha.py"), Path("beta.py")],
            test_files=[Path("tq/test_alpha.py")],
        )
    )

    findings = MappingMissingTestRule(ignore_init_modules=True).evaluate(context)

    assert len(findings) == 1
    finding = findings[0]
    assert finding.rule_id.value == "mapping-missing-test"
    assert finding.severity is Severity.ERROR
    assert finding.path == Path("src/tq").resolve() / "beta.py"


def test_mapping_rule_accepts_qualified_test_files() -> None:
    """Treat qualified test names as valid mapping coverage."""
    context = AnalysisContext.create(
        index=AnalysisIndex.create(
            source_root=Path("src/tq"),
            test_root=Path("tests"),
            source_files=[Path("engine/runner.py")],
            test_files=[Path("tq/engine/test_runner_regression.py")],
        )
    )

    findings = MappingMissingTestRule(ignore_init_modules=True).evaluate(context)

    assert findings == ()


def test_mapping_rule_ignores_init_modules_when_configured() -> None:
    """Skip init modules when the rule is configured to ignore them."""
    context = AnalysisContext.create(
        index=AnalysisIndex.create(
            source_root=Path("src/tq"),
            test_root=Path("tests"),
            source_files=[Path("__init__.py")],
            test_files=[],
        )
    )

    findings = MappingMissingTestRule(ignore_init_modules=True).evaluate(context)

    assert findings == ()


def test_mapping_rule_allowlist_blocks_unknown_qualifier() -> None:
    """Do not accept qualified tests with non-allowlisted suffixes."""
    context = AnalysisContext.create(
        index=AnalysisIndex.create(
            source_root=Path("src/tq"),
            test_root=Path("tests"),
            source_files=[Path("engine/runner.py")],
            test_files=[Path("tq/engine/test_runner_smoke.py")],
        )
    )

    findings = MappingMissingTestRule(
        ignore_init_modules=True,
        qualifier_strategy=QualifierStrategy.ALLOWLIST,
        allowed_qualifiers=("regression",),
    ).evaluate(context)

    assert len(findings) == 1
    assert findings[0].rule_id.value == "mapping-missing-test"
