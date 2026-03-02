"""Tests for tq config models."""

from __future__ import annotations

from pathlib import Path

from tq.config.models import TqConfig
from tq.engine.rule_id import RuleId
from tq.rules.qualifiers import QualifierStrategy


def test_tq_config_package_paths() -> None:
    """Expose derived package path helpers from canonical package name."""
    config = TqConfig(
        package="pkg.subpkg",
        source_root=Path("src").resolve(),
        test_root=Path("tests").resolve(),
        ignore_init_modules=True,
        max_test_file_non_blank_lines=600,
        qualifier_strategy=QualifierStrategy.ANY_SUFFIX,
        allowed_qualifiers=("regression",),
        select=(RuleId("mapping-missing-test"),),
        ignore=(),
    )

    assert config.package_path == Path("pkg") / "subpkg"
    assert config.source_package_root == Path("src").resolve() / "pkg" / "subpkg"
