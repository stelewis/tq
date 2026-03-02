"""Tests for orphaned-test built-in rule."""

from __future__ import annotations

from pathlib import Path

import pytest

from tq.discovery.index import AnalysisIndex
from tq.engine.context import AnalysisContext
from tq.engine.models import Severity
from tq.rules.orphaned_test import (
    OrphanedTestRule,
    QualifierStrategy,
)


def test_orphan_rule_emits_warning_for_missing_source() -> None:
    """Warn when a unit test has no corresponding source module."""
    context = AnalysisContext.create(
        index=AnalysisIndex.create(
            source_root=Path("src/tq"),
            test_root=Path("tests"),
            source_files=[Path("engine/runner.py")],
            test_files=[Path("tq/engine/test_missing.py")],
        )
    )

    findings = OrphanedTestRule(qualifier_strategy=QualifierStrategy.NONE).evaluate(
        context
    )

    assert len(findings) == 1
    finding = findings[0]
    assert finding.rule_id.value == "orphaned-test"
    assert finding.severity is Severity.WARNING
    assert finding.path == Path("tests").resolve() / "tq/engine/test_missing.py"


def test_orphan_rule_allowlist_strategy_accepts_qualified_test() -> None:
    """Allow qualified test names that match explicit allowlist suffixes."""
    context = AnalysisContext.create(
        index=AnalysisIndex.create(
            source_root=Path("src/tq"),
            test_root=Path("tests"),
            source_files=[Path("engine/runner.py")],
            test_files=[Path("tq/engine/test_runner_regression.py")],
        )
    )

    findings = OrphanedTestRule(
        qualifier_strategy=QualifierStrategy.ALLOWLIST,
        allowed_qualifiers=("regression",),
    ).evaluate(context)

    assert findings == ()


def test_orphan_rule_any_suffix_strategy_accepts_qualified_test() -> None:
    """Allow qualified test names when strategy allows any suffix."""
    context = AnalysisContext.create(
        index=AnalysisIndex.create(
            source_root=Path("src/tq"),
            test_root=Path("tests"),
            source_files=[Path("engine/runner.py")],
            test_files=[Path("tq/engine/test_runner_foo.py")],
        )
    )

    findings = OrphanedTestRule(
        qualifier_strategy=QualifierStrategy.ANY_SUFFIX,
    ).evaluate(context)

    assert findings == ()


def test_orphan_rule_rejects_empty_allowlist() -> None:
    """Require explicit qualifiers for allowlist strategy."""
    with pytest.raises(ValueError):
        OrphanedTestRule(qualifier_strategy=QualifierStrategy.ALLOWLIST)
