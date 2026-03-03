"""Tests for rule orchestration engine behavior."""

from __future__ import annotations

from pathlib import Path

import pytest

from tq.discovery.index import AnalysisIndex
from tq.engine.context import AnalysisContext
from tq.engine.models import Finding, Severity
from tq.engine.rule_id import RuleId
from tq.engine.runner import RuleEngine, aggregate_results


class _NoFindingRule:
    @property
    def rule_id(self) -> RuleId:
        return RuleId("no-findings")

    def evaluate(self, context: AnalysisContext) -> tuple[Finding, ...]:
        _ = context
        return ()


class _MixedRuleA:
    @property
    def rule_id(self) -> RuleId:
        return RuleId("rule-a")

    def evaluate(self, context: AnalysisContext) -> tuple[Finding, ...]:
        _ = context
        return (
            Finding(
                rule_id=self.rule_id,
                severity=Severity.WARNING,
                message="warn later",
                path=Path("tests/tq/test_beta.py"),
                line=12,
            ),
            Finding(
                rule_id=self.rule_id,
                severity=Severity.ERROR,
                message="error first",
                path=Path("tests/tq/test_alpha.py"),
                line=4,
            ),
        )


class _MixedRuleB:
    @property
    def rule_id(self) -> RuleId:
        return RuleId("rule-b")

    def evaluate(self, context: AnalysisContext) -> tuple[Finding, ...]:
        _ = context
        return (
            Finding(
                rule_id=self.rule_id,
                severity=Severity.INFO,
                message="info same path",
                path=Path("tests/tq/test_alpha.py"),
                line=2,
            ),
        )


def _context() -> AnalysisContext:
    index = AnalysisIndex.create(
        source_root=Path("src/tq"),
        test_root=Path("tests"),
        source_files=[Path("foo.py")],
        test_files=[Path("tq/test_foo.py")],
    )
    return AnalysisContext.create(index=index)


def test_engine_no_rules_returns_empty_result() -> None:
    """Support a no-rules scenario with empty findings and summary."""
    engine = RuleEngine(rules=())

    result = engine.run(context=_context())

    assert result.findings == ()
    assert result.summary.errors == 0
    assert result.summary.warnings == 0
    assert result.summary.infos == 0
    assert result.summary.total == 0
    assert result.has_errors is False


def test_engine_multi_rule_aggregates_and_sorts_deterministically() -> None:
    """Aggregate multiple rules with deterministic finding order."""
    engine = RuleEngine(rules=(_MixedRuleA(), _MixedRuleB()))

    result = engine.run(context=_context())

    assert [finding.path.as_posix() for finding in result.findings] == [
        "tests/tq/test_alpha.py",
        "tests/tq/test_alpha.py",
        "tests/tq/test_beta.py",
    ]
    assert [finding.line for finding in result.findings] == [2, 4, 12]
    assert result.summary.errors == 1
    assert result.summary.warnings == 1
    assert result.summary.infos == 1
    assert result.summary.total == 3
    assert result.has_errors is True


def test_engine_executes_rule_instances() -> None:
    """Call rule evaluate methods and return their findings."""
    engine = RuleEngine(rules=(_NoFindingRule(), _MixedRuleB()))

    result = engine.run(context=_context())

    assert len(result.findings) == 1
    assert result.findings[0].rule_id == RuleId("rule-b")


def test_engine_rejects_blank_rule_id() -> None:
    """Reject configured rules with blank identifiers."""

    class _BlankRule:
        @property
        def rule_id(self) -> str:
            return "blank"

        def evaluate(self, context: AnalysisContext) -> tuple[Finding, ...]:
            _ = context
            return ()

    with pytest.raises(TypeError):
        RuleEngine(rules=(_BlankRule(),))


def test_engine_sorts_severity_for_same_location_and_rule() -> None:
    """Order severities as error, warning, info for equivalent locations."""

    class _SeverityRule:
        @property
        def rule_id(self) -> RuleId:
            return RuleId("same-rule")

        def evaluate(self, context: AnalysisContext) -> tuple[Finding, ...]:
            _ = context
            return (
                Finding(
                    rule_id=self.rule_id,
                    severity=Severity.INFO,
                    message="info",
                    path=Path("tests/tq/test_alpha.py"),
                    line=1,
                ),
                Finding(
                    rule_id=self.rule_id,
                    severity=Severity.ERROR,
                    message="error",
                    path=Path("tests/tq/test_alpha.py"),
                    line=1,
                ),
                Finding(
                    rule_id=self.rule_id,
                    severity=Severity.WARNING,
                    message="warning",
                    path=Path("tests/tq/test_alpha.py"),
                    line=1,
                ),
            )

    result = RuleEngine(rules=(_SeverityRule(),)).run(context=_context())

    assert [finding.severity for finding in result.findings] == [
        Severity.ERROR,
        Severity.WARNING,
        Severity.INFO,
    ]


def test_engine_attaches_target_name_from_context_settings() -> None:
    """Attach context target name to each emitted finding."""
    context = AnalysisContext.create(
        index=_context().index,
        settings={"target_name": "scripts"},
    )

    result = RuleEngine(rules=(_MixedRuleB(),)).run(context=context)

    assert result.findings[0].target == "scripts"


def test_aggregate_results_merges_and_sorts_findings() -> None:
    """Aggregate deterministic findings and summary across target runs."""
    result_a = RuleEngine(rules=(_MixedRuleA(),)).run(context=_context())
    result_b = RuleEngine(rules=(_MixedRuleB(),)).run(context=_context())

    merged = aggregate_results(results=(result_b, result_a))

    assert [finding.path.as_posix() for finding in merged.findings] == [
        "tests/tq/test_alpha.py",
        "tests/tq/test_alpha.py",
        "tests/tq/test_beta.py",
    ]
    assert merged.summary.errors == 1
    assert merged.summary.warnings == 1
    assert merged.summary.infos == 1
