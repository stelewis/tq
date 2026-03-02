"""Tests for rule protocol contract."""

from __future__ import annotations

from pathlib import Path

from tq.discovery.index import AnalysisIndex
from tq.engine.context import AnalysisContext
from tq.engine.models import Finding, Severity
from tq.rules.contracts import Rule


class DemoRule:
    """Simple protocol-conforming rule for tests."""

    @property
    def rule_id(self) -> str:
        """Return stable demo identifier."""
        return "demo-rule"

    def evaluate(self, context: AnalysisContext) -> tuple[Finding, ...]:
        """Emit one static finding."""
        _ = context
        return (
            Finding(
                rule_id=self.rule_id,
                severity=Severity.INFO,
                message="demo finding",
                path=Path("tests/tq/test_demo.py"),
                line=1,
            ),
        )


def test_rule_protocol_shape_runtime() -> None:
    """Validate protocol expectations for rule objects."""
    index = AnalysisIndex.create(
        source_root=Path("src/tq"),
        test_root=Path("tests"),
        source_files=[],
        test_files=[],
    )
    context = AnalysisContext.create(index=index)
    rule = DemoRule()

    assert isinstance(rule, Rule)
    findings = rule.evaluate(context)
    assert len(findings) == 1
    assert findings[0].rule_id == "demo-rule"
