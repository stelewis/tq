"""Rule contract for the tq analysis engine."""

from __future__ import annotations

from typing import Protocol, runtime_checkable

from tq.engine.context import AnalysisContext
from tq.engine.models import Finding


@runtime_checkable
class Rule(Protocol):
    """Contract for analysis rules used by the orchestration engine."""

    @property
    def rule_id(self) -> str:
        """Return stable rule identifier for diagnostics and selection."""

    def evaluate(self, context: AnalysisContext) -> tuple[Finding, ...]:
        """Evaluate a rule against immutable analysis context.

        Args:
            context: Immutable analysis context for this engine run.

        Returns:
            Deterministic collection of findings emitted by this rule.
        """
