"""Tests for target run planning orchestration."""

from __future__ import annotations

from typing import TYPE_CHECKING

from tq.config.models import TqTargetConfig
from tq.engine.planner import plan_target_runs
from tq.rules.qualifiers import QualifierStrategy

if TYPE_CHECKING:
    from pathlib import Path


def test_plan_target_runs_creates_context_per_active_target(tmp_path: Path) -> None:
    """Create one planned run per active target with expected context fields."""
    _write_file(tmp_path / "src/tq/module.py")
    _write_file(tmp_path / "tests/tq/test_module.py")

    target = _target(
        root=tmp_path,
        name="tq",
        package="tq",
        source_root="src",
        test_root="tests",
    )

    planned_runs = plan_target_runs(
        configured_targets=(target,),
        active_targets=(target,),
    )

    assert len(planned_runs) == 1
    assert planned_runs[0].target.name == "tq"
    assert planned_runs[0].context.settings["target_name"] == "tq"
    assert planned_runs[0].context.settings["package_path"] == "tq"
    assert planned_runs[0].context.settings["test_root_display"] == "tests"
    assert planned_runs[0].context.settings["known_target_package_paths"] == ("tq",)


def test_plan_target_runs_uses_configured_targets_for_known_paths(
    tmp_path: Path,
) -> None:
    """Keep known target package paths scoped to full configured target set."""
    _write_file(tmp_path / "src/tq/module.py")
    _write_file(tmp_path / "tests/tq/test_module.py")
    _write_file(tmp_path / "scripts/docs/generate.py")
    _write_file(tmp_path / "tests/scripts/docs/test_generate.py")

    tq_target = _target(
        root=tmp_path,
        name="tq",
        package="tq",
        source_root="src",
        test_root="tests",
    )
    scripts_target = _target(
        root=tmp_path,
        name="scripts",
        package="scripts",
        source_root=".",
        test_root="tests",
    )

    planned_runs = plan_target_runs(
        configured_targets=(tq_target, scripts_target),
        active_targets=(scripts_target,),
    )

    assert len(planned_runs) == 1
    assert planned_runs[0].target.name == "scripts"
    assert planned_runs[0].context.settings["known_target_package_paths"] == (
        "tq",
        "scripts",
    )


def _target(
    *,
    root: Path,
    name: str,
    package: str,
    source_root: str,
    test_root: str,
) -> TqTargetConfig:
    """Build a resolved target config for planner tests."""
    return TqTargetConfig(
        name=name,
        package=package,
        source_root=(root / source_root).resolve(),
        test_root=(root / test_root).resolve(),
        ignore_init_modules=False,
        max_test_file_non_blank_lines=600,
        qualifier_strategy=QualifierStrategy.ANY_SUFFIX,
        allowed_qualifiers=(),
        select=(),
        ignore=(),
    )


def _write_file(path: Path) -> None:
    """Create parent directories and write a file for discovery scans."""
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("pass\n", encoding="utf-8")
