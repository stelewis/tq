"""Tests for rule identifier value object."""

from __future__ import annotations

import pytest

from tq.engine.rule_id import RuleId


def test_rule_id_accepts_kebab_case_values() -> None:
    """Accept valid kebab-case rule identifiers."""
    assert RuleId("mapping-missing-test").value == "mapping-missing-test"


def test_rule_id_rejects_invalid_values() -> None:
    """Reject blank and non-kebab-case identifiers."""
    with pytest.raises(ValueError):
        RuleId("")

    with pytest.raises(ValueError):
        RuleId("Mapping-Missing-Test")

    with pytest.raises(ValueError):
        RuleId("mapping_missing_test")


def test_rule_id_string_conversion_returns_value() -> None:
    """Return raw identifier in string context."""
    assert str(RuleId("orphaned-test")) == "orphaned-test"
