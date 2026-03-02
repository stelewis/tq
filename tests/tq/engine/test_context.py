"""Tests for analysis context model."""

from __future__ import annotations

from pathlib import Path

import pytest

from tq.discovery.index import AnalysisIndex
from tq.engine.context import AnalysisContext


def test_context_create_exposes_read_only_settings() -> None:
    """Create context with immutable settings mapping."""
    index = AnalysisIndex.create(
        source_root=Path("src/tq"),
        test_root=Path("tests"),
        source_files=[],
        test_files=[],
    )

    context = AnalysisContext.create(
        index=index,
        settings={"max_test_lines": 300},
    )

    assert context.settings["max_test_lines"] == 300
    with pytest.raises(TypeError):
        context.settings["max_test_lines"] = 200  # type: ignore[index]


def test_context_is_frozen() -> None:
    """Reject direct field mutation for context model."""
    index = AnalysisIndex.create(
        source_root=Path("src/tq"),
        test_root=Path("tests"),
        source_files=[],
        test_files=[],
    )
    context = AnalysisContext.create(index=index)

    with pytest.raises(AttributeError):
        context.index = index  # type: ignore[misc]
