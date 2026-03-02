"""Tests for shared qualifier policy utilities."""

from __future__ import annotations

from tq.rules.qualifiers import QualifierStrategy, candidate_module_names


def test_candidate_module_names_none_strategy() -> None:
    """Return only full stem when qualifier strategy is none."""
    names = candidate_module_names(
        module_stem="runner_regression",
        qualifier_strategy=QualifierStrategy.NONE,
        allowed_qualifiers=(),
    )

    assert names == ("runner_regression",)


def test_candidate_module_names_any_suffix_strategy() -> None:
    """Return progressive prefixes when any suffixes are allowed."""
    names = candidate_module_names(
        module_stem="income_record_validation_smoke",
        qualifier_strategy=QualifierStrategy.ANY_SUFFIX,
        allowed_qualifiers=(),
    )

    assert names == (
        "income_record_validation_smoke",
        "income_record_validation",
        "income_record",
        "income",
    )


def test_candidate_module_names_allowlist_strategy() -> None:
    """Only allow prefixes with explicitly allowed suffixes."""
    names = candidate_module_names(
        module_stem="runner_regression",
        qualifier_strategy=QualifierStrategy.ALLOWLIST,
        allowed_qualifiers=("regression",),
    )

    assert names == ("runner_regression", "runner")
