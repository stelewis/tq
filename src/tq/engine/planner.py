"""Run planning for target-scoped tq engine execution."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

from tq.discovery.filesystem import build_analysis_index
from tq.engine.context import AnalysisContext

if TYPE_CHECKING:
    from tq.config.models import TqTargetConfig


@dataclass(frozen=True, slots=True)
class PlannedTargetRun:
    """Planned execution unit for a target-scoped analysis run."""

    target: TqTargetConfig
    context: AnalysisContext


def plan_target_runs(
    *,
    configured_targets: tuple[TqTargetConfig, ...],
    active_targets: tuple[TqTargetConfig, ...],
) -> tuple[PlannedTargetRun, ...]:
    """Plan deterministic target runs before rule execution."""
    known_target_package_paths = tuple(
        configured_target.package_path.as_posix()
        for configured_target in configured_targets
    )

    planned_runs: list[PlannedTargetRun] = []
    for target in active_targets:
        index = build_analysis_index(
            source_root=target.source_package_root,
            test_root=target.test_root,
        )
        context = AnalysisContext.create(
            index=index,
            settings={
                "target_name": target.name,
                "package_path": target.package_path.as_posix(),
                "known_target_package_paths": known_target_package_paths,
                "test_root_display": target.test_root.name,
            },
        )
        planned_runs.append(PlannedTargetRun(target=target, context=context))

    return tuple(planned_runs)
