"""Tests for structure-mismatch built-in rule."""

from __future__ import annotations

from pathlib import Path

from tq.discovery.index import AnalysisIndex
from tq.engine.context import AnalysisContext
from tq.engine.models import Severity
from tq.rules.structure_mismatch import StructureMismatchRule


def test_structure_rule_emits_warning_for_misplaced_test() -> None:
    """Warn when a test file exists but does not mirror source location."""
    context = AnalysisContext.create(
        index=AnalysisIndex.create(
            source_root=Path("src/tq"),
            test_root=Path("tests"),
            source_files=[Path("engine/runner.py")],
            test_files=[Path("tq/test_runner.py")],
        )
    )

    findings = StructureMismatchRule().evaluate(context)

    assert len(findings) == 1
    finding = findings[0]
    assert finding.rule_id.value == "structure-mismatch"
    assert finding.severity is Severity.WARNING
    assert finding.path == Path("tests").resolve() / "tq/test_runner.py"
    assert finding.suggestion == "Move to: tq/engine/test_runner.py"


def test_structure_rule_allows_correctly_placed_test() -> None:
    """Produce no finding when test structure matches source structure."""
    context = AnalysisContext.create(
        index=AnalysisIndex.create(
            source_root=Path("src/tq"),
            test_root=Path("tests"),
            source_files=[Path("engine/runner.py")],
            test_files=[Path("tq/engine/test_runner.py")],
        )
    )

    findings = StructureMismatchRule().evaluate(context)

    assert findings == ()


def test_structure_rule_skips_non_unit_scopes() -> None:
    """Skip integration and e2e paths from structure analysis."""
    context = AnalysisContext.create(
        index=AnalysisIndex.create(
            source_root=Path("src/tq"),
            test_root=Path("tests"),
            source_files=[Path("engine/runner.py")],
            test_files=[Path("integration/test_runner.py"), Path("e2e/test_runner.py")],
        )
    )

    findings = StructureMismatchRule().evaluate(context)

    assert findings == ()
