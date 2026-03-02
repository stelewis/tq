"""Golden-style integration coverage for built-in rules."""

from __future__ import annotations

from pathlib import Path

import pytest

from tq.discovery.index import AnalysisIndex
from tq.engine.context import AnalysisContext
from tq.engine.runner import RuleEngine
from tq.rules.file_too_large import FileTooLargeRule
from tq.rules.mapping_missing_test import MappingMissingTestRule
from tq.rules.orphaned_test import OrphanedTestRule, QualifierStrategy
from tq.rules.structure_mismatch import StructureMismatchRule


@pytest.mark.golden
def test_builtins_emit_expected_findings_for_representative_tree(
    tmp_path: Path,
) -> None:
    """Validate representative tree diagnostics with deterministic output."""
    source_root = tmp_path / "src" / "tq"
    test_root = tmp_path / "tests"

    _write(source_root / "alpha.py", "def run() -> None:\n    pass\n")
    _write(source_root / "beta.py", "def go() -> None:\n    pass\n")
    _write(
        source_root / "pkg" / "gamma.py",
        "def gamma() -> None:\n    pass\n",
    )

    _write(
        test_root / "tq" / "test_alpha.py",
        "def test_alpha() -> None:\n    assert True\n",
    )
    _write(
        test_root / "tq" / "test_gamma.py",
        "def test_gamma_one() -> None:\n    assert True\n"
        "\n"
        "def test_gamma_two() -> None:\n    assert True\n",
    )
    _write(
        test_root / "tq" / "pkg" / "test_orphan.py",
        "def test_orphan() -> None:\n    assert True\n",
    )

    context = AnalysisContext.create(
        index=_build_index(source_root=source_root, test_root=test_root)
    )
    engine = RuleEngine(
        rules=(
            MappingMissingTestRule(ignore_init_modules=True),
            StructureMismatchRule(),
            FileTooLargeRule(max_non_blank_lines=3),
            OrphanedTestRule(
                qualifier_strategy=QualifierStrategy.ALLOWLIST,
                allowed_qualifiers=("regression",),
            ),
        )
    )

    result = engine.run(context=context)

    actual = [
        (finding.rule_id.value, finding.path.relative_to(tmp_path).as_posix())
        for finding in result.findings
    ]

    assert actual == [
        ("mapping-missing-test", "src/tq/beta.py"),
        ("mapping-missing-test", "src/tq/pkg/gamma.py"),
        ("orphaned-test", "tests/tq/pkg/test_orphan.py"),
        ("orphaned-test", "tests/tq/test_gamma.py"),
        ("structure-mismatch", "tests/tq/test_gamma.py"),
        ("test-file-too-large", "tests/tq/test_gamma.py"),
    ]


def _build_index(*, source_root: Path, test_root: Path) -> AnalysisIndex:
    """Create an analysis index from a filesystem tree for integration tests."""
    source_files = [path.relative_to(source_root) for path in source_root.rglob("*.py")]
    test_files = [path.relative_to(test_root) for path in test_root.rglob("test_*.py")]
    return AnalysisIndex.create(
        source_root=source_root,
        test_root=test_root,
        source_files=source_files,
        test_files=test_files,
    )


def _write(path: Path, content: str) -> None:
    """Write fixture file content and create parents."""
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")
