"""Rule orchestration engine with deterministic finding aggregation."""

from __future__ import annotations

from dataclasses import replace
from typing import TYPE_CHECKING

from tq.engine.models import (
    EngineResult,
    Finding,
    FindingSummary,
    Severity,
    severity_rank,
)
from tq.engine.rule_id import RuleId

if TYPE_CHECKING:
    from collections.abc import Sequence

    from tq.engine.context import AnalysisContext
    from tq.rules.contracts import Rule


class RuleEngine:
    """Execute rules against analysis context and aggregate diagnostics."""

    def __init__(self, *, rules: Sequence[Rule]) -> None:
        """Initialize the engine with explicit rule dependencies.

        Args:
            rules: Ordered collection of rule instances to evaluate.
        """
        self._rules = tuple(rules)

        for rule in self._rules:
            if not isinstance(rule.rule_id, RuleId):
                msg = "Rule.rule_id must be a RuleId instance"
                raise TypeError(msg)

    def run(self, *, context: AnalysisContext) -> EngineResult:
        """Evaluate configured rules and aggregate deterministic results.

        Args:
            context: Immutable analysis context for this evaluation.

        Returns:
            EngineResult with sorted findings and severity summary.
        """
        findings: list[Finding] = []
        target_name = _target_name_from_context(context)
        for rule in self._rules:
            findings.extend(
                _attach_target_name(
                    findings=rule.evaluate(context),
                    target_name=target_name,
                ),
            )

        sorted_findings = tuple(sorted(findings, key=_finding_sort_key))
        summary = _build_summary(sorted_findings)
        return EngineResult(findings=sorted_findings, summary=summary)


def aggregate_results(*, results: Sequence[EngineResult]) -> EngineResult:
    """Aggregate many engine results into one deterministic result."""
    findings: list[Finding] = []
    for result in results:
        findings.extend(result.findings)

    sorted_findings = tuple(sorted(findings, key=_finding_sort_key))
    return EngineResult(
        findings=sorted_findings, summary=_build_summary(sorted_findings)
    )


def _finding_sort_key(
    finding: Finding,
) -> tuple[str, str, int, int, str, int, str]:
    """Build stable sort key for deterministic output ordering."""
    return (
        finding.target or "",
        finding.path.as_posix(),
        finding.line if finding.line is not None else 0,
        severity_rank(finding.severity),
        finding.rule_id.value,
        len(finding.message),
        finding.message,
    )


def _target_name_from_context(context: AnalysisContext) -> str | None:
    """Read target name from context settings if configured."""
    value = context.settings.get("target_name")
    if isinstance(value, str) and value.strip():
        return value
    return None


def _attach_target_name(
    *,
    findings: tuple[Finding, ...],
    target_name: str | None,
) -> tuple[Finding, ...]:
    """Attach target name to findings if missing and available."""
    if target_name is None:
        return findings

    return tuple(
        finding if finding.target is not None else replace(finding, target=target_name)
        for finding in findings
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
