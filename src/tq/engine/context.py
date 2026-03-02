"""Immutable analysis context used by the tq rule engine."""

from __future__ import annotations

from dataclasses import dataclass, field
from types import MappingProxyType
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Mapping

    from tq.discovery.index import AnalysisIndex


@dataclass(frozen=True, slots=True)
class AnalysisContext:
    """Rule evaluation context for a single analysis run.

    Attributes:
        index: Immutable source/test index used by rules.
        settings: Immutable key-value settings available to rules.
    """

    index: AnalysisIndex
    settings: MappingProxyType[str, object] = field(
        default_factory=lambda: MappingProxyType({}),
    )

    @classmethod
    def create(
        cls,
        *,
        index: AnalysisIndex,
        settings: Mapping[str, object] | None = None,
    ) -> AnalysisContext:
        """Create immutable analysis context.

        Args:
            index: Immutable source/test index for this run.
            settings: Optional mapping of explicit engine settings.

        Returns:
            AnalysisContext with read-only settings mapping.
        """
        return cls(index=index, settings=MappingProxyType(dict(settings or {})))
