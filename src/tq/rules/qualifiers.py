"""Qualifier policy utilities for unit test module name resolution."""

from __future__ import annotations

from enum import StrEnum


class QualifierStrategy(StrEnum):
    """Policy for deriving source module names from qualified tests."""

    NONE = "none"
    ANY_SUFFIX = "any-suffix"
    ALLOWLIST = "allowlist"


def candidate_module_names(
    *,
    module_stem: str,
    qualifier_strategy: QualifierStrategy,
    allowed_qualifiers: tuple[str, ...],
) -> tuple[str, ...]:
    """Build source module-name candidates from a test module stem.

    Example stems:
        ``runner`` from ``test_runner.py``
        ``runner_regression`` from ``test_runner_regression.py``
    """
    names = [module_stem]
    if "_" not in module_stem:
        return tuple(names)

    if qualifier_strategy is QualifierStrategy.NONE:
        return tuple(names)

    stem_parts = module_stem.split("_")
    for index in range(len(stem_parts) - 1, 0, -1):
        candidate = "_".join(stem_parts[:index])
        suffix = "_".join(stem_parts[index:])

        if qualifier_strategy is QualifierStrategy.ANY_SUFFIX:
            names.append(candidate)
            continue

        if suffix in allowed_qualifiers:
            names.append(candidate)

    return tuple(names)
