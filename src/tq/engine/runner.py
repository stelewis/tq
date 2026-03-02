"""Rule orchestration engine with deterministic finding aggregation."""

from __future__ import annotations

from collections.abc import Sequence

from tq.engine.context import AnalysisContext
from tq.engine.models import (
    EngineResult,
    Finding,
    FindingSummary,
    Severity,
    severity_rank,
)
from tq.engine.rule_id import RuleId
from tq.rules.contracts import Rule


class RuleEngine:
    """Execute rules against analysis context and aggregate diagnostics."""

    def __init__(self, *, rules: Sequence[Rule]):
        """Initialize the engine with explicit rule dependencies.

        Args:
            rules: Ordered collection of rule instances to evaluate.
        """
        self._rules = tuple(rules)

        for rule in self._rules:
            if not isinstance(rule.rule_id, RuleId):
                raise TypeError("Rule.rule_id must be a RuleId instance")

    def run(self, *, context: AnalysisContext) -> EngineResult:
        """Evaluate configured rules and aggregate deterministic results.

        Args:
            context: Immutable analysis context for this evaluation.

        Returns:
            EngineResult with sorted findings and severity summary.
        """
        findings: list[Finding] = []
        for rule in self._rules:
            findings.extend(rule.evaluate(context))

        sorted_findings = tuple(sorted(findings, key=_finding_sort_key))
        summary = _build_summary(sorted_findings)
        return EngineResult(findings=sorted_findings, summary=summary)


def _finding_sort_key(finding: Finding) -> tuple[str, int, int, str, int, str]:
    """Build stable sort key for deterministic output ordering."""
    return (
        finding.path.as_posix(),
        finding.line if finding.line is not None else 0,
        severity_rank(finding.severity),
        finding.rule_id.value,
        len(finding.message),
        finding.message,
    )


def _build_summary(findings: tuple[Finding, ...]) -> FindingSummary:
    """Aggregate findings into severity counts."""
    errors = 0
    warnings = 0
    infos = 0
    for finding in findings:
        if finding.severity is Severity.ERROR:
            errors += 1
        elif finding.severity is Severity.WARNING:
            warnings += 1
        else:
            infos += 1

    return FindingSummary(errors=errors, warnings=warnings, infos=infos)
