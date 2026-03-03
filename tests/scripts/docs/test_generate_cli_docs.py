"""Tests for CLI docs generation script."""

from __future__ import annotations

from typing import TYPE_CHECKING

import pytest
from scripts.docs import generate_cli_docs

if TYPE_CHECKING:
    from pathlib import Path


def test_generate_cli_docs_updates_marked_section(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Generate table content between markers from manifest and options."""
    docs_dir = tmp_path / "docs" / "reference"
    cli_doc_path = docs_dir / "cli.md"
    manifest_path = docs_dir / "cli" / "options-manifest.yaml"
    manifest_path.parent.mkdir(parents=True, exist_ok=True)

    _write_valid_manifest(manifest_path)
    cli_doc_path.write_text(
        (
            "# CLI\n\n"
            "<!-- BEGIN GENERATED:check-options -->\n"
            "placeholder\n"
            "<!-- END GENERATED:check-options -->\n"
        ),
        encoding="utf-8",
    )

    monkeypatch.chdir(tmp_path)
    generate_cli_docs.generate_cli_docs()

    generated = cli_doc_path.read_text(encoding="utf-8")
    assert "| `--target` | — | `[]` | Run only listed target names. |" in generated
    assert "Run `tq check --help` for the runtime source of truth." in generated


def test_generate_cli_docs_fails_when_manifest_is_invalid(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Fail fast for invalid manifest payload structure."""
    docs_dir = tmp_path / "docs" / "reference"
    cli_doc_path = docs_dir / "cli.md"
    manifest_path = docs_dir / "cli" / "options-manifest.yaml"
    manifest_path.parent.mkdir(parents=True, exist_ok=True)

    manifest_path.write_text("cli_options: invalid\n", encoding="utf-8")
    cli_doc_path.write_text(
        (
            "# CLI\n\n"
            "<!-- BEGIN GENERATED:check-options -->\n"
            "placeholder\n"
            "<!-- END GENERATED:check-options -->\n"
        ),
        encoding="utf-8",
    )

    monkeypatch.chdir(tmp_path)
    with pytest.raises(TypeError, match="cli_options list"):
        generate_cli_docs.generate_cli_docs()


def test_generate_cli_docs_fails_when_doc_markers_are_missing(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Fail fast when CLI markdown markers are missing."""
    docs_dir = tmp_path / "docs" / "reference"
    cli_doc_path = docs_dir / "cli.md"
    manifest_path = docs_dir / "cli" / "options-manifest.yaml"
    manifest_path.parent.mkdir(parents=True, exist_ok=True)

    _write_valid_manifest(manifest_path)
    cli_doc_path.write_text("# CLI\nNo markers here.\n", encoding="utf-8")

    monkeypatch.chdir(tmp_path)
    with pytest.raises(ValueError, match="Missing or invalid markers"):
        generate_cli_docs.generate_cli_docs()


def _write_valid_manifest(path: Path) -> None:
    """Write a manifest synchronized with the real `tq check` options."""
    path.write_text(
        (
            "cli_options:\n"
            "  - param: config_path\n"
            "    config_key: null\n"
            "  - param: isolated\n"
            "    config_key: null\n"
            "  - param: target_names\n"
            "    config_key: null\n"
            "  - param: max_test_file_non_blank_lines\n"
            "    config_key: max_test_file_non_blank_lines\n"
            "  - param: qualifier_strategy\n"
            "    config_key: qualifier_strategy\n"
            "  - param: allowed_qualifiers\n"
            "    config_key: allowed_qualifiers\n"
            "  - param: ignore_init_modules\n"
            "    config_key: ignore_init_modules\n"
            "  - param: select_rules\n"
            "    config_key: select\n"
            "  - param: ignore_rules\n"
            "    config_key: ignore\n"
            "  - param: exit_zero\n"
            "    config_key: null\n"
            "  - param: show_suggestions\n"
            "    config_key: null\n"
            "  - param: output_format\n"
            "    config_key: null\n"
        ),
        encoding="utf-8",
    )
