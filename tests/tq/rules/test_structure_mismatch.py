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
        ),
    )

    findings = StructureMismatchRule().evaluate(context)

    assert len(findings) == 1
    finding = findings[0]
    assert finding.rule_id.value == "structure-mismatch"
    assert finding.severity is Severity.WARNING
    assert finding.path == Path("tests").resolve() / "tq/test_runner.py"
    assert finding.suggestion == "Move to: tests/tq/engine/test_runner.py"


def test_structure_rule_allows_correctly_placed_test() -> None:
    """Produce no finding when test structure matches source structure."""
    context = AnalysisContext.create(
        index=AnalysisIndex.create(
            source_root=Path("src/tq"),
            test_root=Path("tests"),
            source_files=[Path("engine/runner.py")],
            test_files=[Path("tq/engine/test_runner.py")],
        ),
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
        ),
    )

    findings = StructureMismatchRule().evaluate(context)

    assert findings == ()


def test_structure_rule_ignores_sibling_target_tests() -> None:
    """Ignore test files that belong to another configured target root."""
    context = AnalysisContext.create(
        index=AnalysisIndex.create(
            source_root=Path("scripts"),
            test_root=Path("tests"),
            source_files=[Path("docs/generate.py")],
            test_files=[
                Path("tq/engine/test_runner.py"),
                Path("scripts/docs/test_generate.py"),
            ],
        ),
        settings={
            "package_path": "scripts",
            "known_target_package_paths": ("tq", "scripts"),
        },
    )

    findings = StructureMismatchRule().evaluate(context)

    assert findings == ()


def test_structure_rule_uses_package_path_from_context_settings() -> None:
    """Evaluate package-root placement using explicit package path setting."""
    context = AnalysisContext.create(
        index=AnalysisIndex.create(
            source_root=Path("src/myproj/core"),
            test_root=Path("tests"),
            source_files=[Path("runner.py")],
            test_files=[Path("myproj/test_runner.py")],
        ),
        settings={"package_path": "myproj/core"},
    )

    findings = StructureMismatchRule().evaluate(context)

    assert len(findings) == 1
    assert findings[0].suggestion == "Move test under: tests/myproj/core/test_runner.py"
