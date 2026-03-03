"""Tests for rules docs generation script."""

from __future__ import annotations

from typing import TYPE_CHECKING

import pytest
from scripts.docs import generate_rules_docs

if TYPE_CHECKING:
    from pathlib import Path


def test_generate_rules_docs_writes_index_and_rule_pages(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Generate canonical rules index and per-rule markdown pages."""
    rules_dir = tmp_path / "docs" / "reference" / "rules"
    rules_dir.mkdir(parents=True, exist_ok=True)
    (rules_dir / "manifest.yaml").write_text(
        (
            "severity_vocabulary:\n"
            "  - error\n"
            "  - warning\n"
            "  - info\n"
            "rules:\n"
            "  - id: orphaned-test\n"
            "    default_severity: warning\n"
            "    added_in: 0.4.0\n"
            "    behavior_changes: none\n"
            "    what_it_does: detects tests with no source module\n"
            "    why_this_matters: avoids stale tests\n"
            "    trigger_conditions:\n"
            "      - no corresponding source module exists\n"
            "    examples:\n"
            "      - source: n/a\n"
            "        test: tests/tq/rules/test_obsolete.py\n"
            "    how_to_address:\n"
            "      - delete stale test or restore source module\n"
            "    related_controls:\n"
            "      - --ignore\n"
        ),
        encoding="utf-8",
    )

    monkeypatch.chdir(tmp_path)
    generate_rules_docs.generate_rules_docs()

    index_content = (rules_dir / "index.md").read_text(encoding="utf-8")
    page_content = (rules_dir / "orphaned-test.md").read_text(encoding="utf-8")
    sidebar_content = (
        tmp_path / "docs" / ".vitepress" / "generated" / "rules-sidebar.ts"
    ).read_text(encoding="utf-8")

    assert "# Rules" in index_content
    assert "[`orphaned-test`](./orphaned-test.md)" in index_content
    assert "- `error`" in index_content
    assert "- `warning`" in index_content
    assert "- `info`" in index_content
    assert "[governance policy](../../developer/governance.md)" in index_content
    assert "# orphaned-test" in page_content
    assert "## Trigger conditions" in page_content
    assert "export const rulesSidebarItems = [" in sidebar_content
    assert 'text: "orphaned-test"' in sidebar_content
    assert 'link: "/reference/rules/orphaned-test"' in sidebar_content


def test_generate_rules_docs_fails_for_invalid_manifest(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Fail fast when rules manifest has an invalid schema."""
    rules_dir = tmp_path / "docs" / "reference" / "rules"
    rules_dir.mkdir(parents=True, exist_ok=True)
    (rules_dir / "manifest.yaml").write_text("rules: invalid\n", encoding="utf-8")

    monkeypatch.chdir(tmp_path)
    with pytest.raises(TypeError, match="'rules' list"):
        generate_rules_docs.generate_rules_docs()


def test_generate_rules_docs_fails_when_severity_vocabulary_missing(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Fail fast when severity vocabulary is missing from manifest."""
    rules_dir = tmp_path / "docs" / "reference" / "rules"
    rules_dir.mkdir(parents=True, exist_ok=True)
    (rules_dir / "manifest.yaml").write_text(
        (
            "rules:\n"
            "  - id: orphaned-test\n"
            "    default_severity: warning\n"
            "    added_in: 0.4.0\n"
            "    behavior_changes: none\n"
            "    what_it_does: detects tests with no source module\n"
            "    why_this_matters: avoids stale tests\n"
            "    trigger_conditions:\n"
            "      - no corresponding source module exists\n"
            "    examples:\n"
            "      - source: n/a\n"
            "        test: tests/tq/rules/test_obsolete.py\n"
            "    how_to_address:\n"
            "      - delete stale test or restore source module\n"
            "    related_controls:\n"
            "      - --ignore\n"
        ),
        encoding="utf-8",
    )

    monkeypatch.chdir(tmp_path)
    with pytest.raises(TypeError, match="severity_vocabulary"):
        generate_rules_docs.generate_rules_docs()
