"""Tests for config examples docs generation script."""

from __future__ import annotations

from typing import TYPE_CHECKING

import pytest
from scripts.docs import generate_config_examples

if TYPE_CHECKING:
    from pathlib import Path


def test_generate_config_examples_updates_marked_sections(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Generate canonical TOML examples between markdown markers."""
    docs_dir = tmp_path / "docs"
    quickstart_path = docs_dir / "guide" / "quickstart.md"
    configuration_path = docs_dir / "reference" / "configuration.md"
    manifest_path = docs_dir / "reference" / "config" / "examples-manifest.yaml"

    quickstart_path.parent.mkdir(parents=True, exist_ok=True)
    configuration_path.parent.mkdir(parents=True, exist_ok=True)
    manifest_path.parent.mkdir(parents=True, exist_ok=True)

    quickstart_path.write_text(
        (
            "# QuickStart\n\n"
            "<!-- BEGIN GENERATED:quickstart-minimal-config -->\n"
            "placeholder\n"
            "<!-- END GENERATED:quickstart-minimal-config -->\n"
        ),
        encoding="utf-8",
    )
    configuration_path.write_text(
        (
            "# Configuration\n\n"
            "<!-- BEGIN GENERATED:configuration-minimal-config -->\n"
            "placeholder\n"
            "<!-- END GENERATED:configuration-minimal-config -->\n\n"
            "<!-- BEGIN GENERATED:configuration-typical-config -->\n"
            "placeholder\n"
            "<!-- END GENERATED:configuration-typical-config -->\n"
        ),
        encoding="utf-8",
    )
    manifest_path.write_text(
        (
            "examples:\n"
            "  quickstart_minimal: |\n"
            "    [tool.tq]\n"
            "    [[tool.tq.targets]]\n"
            '    name = "app"\n'
            '    package = "your_package"\n'
            '    source_root = "src"\n'
            '    test_root = "tests"\n'
            "  configuration_minimal: |\n"
            "    [tool.tq]\n"
            "    [[tool.tq.targets]]\n"
            '    name = "app"\n'
            '    package = "your_package"\n'
            '    source_root = "src"\n'
            '    test_root = "tests"\n'
            "  configuration_typical: |\n"
            "    [tool.tq]\n"
            "    ignore_init_modules = true\n"
            "    [[tool.tq.targets]]\n"
            '    name = "app"\n'
            '    package = "your_package"\n'
            '    source_root = "src"\n'
            '    test_root = "tests"\n'
        ),
        encoding="utf-8",
    )

    monkeypatch.chdir(tmp_path)
    generate_config_examples.generate_config_examples()

    quickstart = quickstart_path.read_text(encoding="utf-8")
    configuration = configuration_path.read_text(encoding="utf-8")

    assert "```toml" in quickstart
    assert 'name = "app"' in quickstart
    assert "placeholder" not in quickstart

    assert "```toml" in configuration
    assert "ignore_init_modules = true" in configuration
    assert "placeholder" not in configuration


def test_generate_config_examples_fails_for_invalid_manifest_shape(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Fail fast when config examples manifest is not valid."""
    docs_dir = tmp_path / "docs"
    quickstart_path = docs_dir / "guide" / "quickstart.md"
    configuration_path = docs_dir / "reference" / "configuration.md"
    manifest_path = docs_dir / "reference" / "config" / "examples-manifest.yaml"

    quickstart_path.parent.mkdir(parents=True, exist_ok=True)
    configuration_path.parent.mkdir(parents=True, exist_ok=True)
    manifest_path.parent.mkdir(parents=True, exist_ok=True)

    quickstart_path.write_text(
        (
            "# QuickStart\n\n"
            "<!-- BEGIN GENERATED:quickstart-minimal-config -->\n"
            "placeholder\n"
            "<!-- END GENERATED:quickstart-minimal-config -->\n"
        ),
        encoding="utf-8",
    )
    configuration_path.write_text(
        (
            "# Configuration\n\n"
            "<!-- BEGIN GENERATED:configuration-minimal-config -->\n"
            "placeholder\n"
            "<!-- END GENERATED:configuration-minimal-config -->\n\n"
            "<!-- BEGIN GENERATED:configuration-typical-config -->\n"
            "placeholder\n"
            "<!-- END GENERATED:configuration-typical-config -->\n"
        ),
        encoding="utf-8",
    )
    manifest_path.write_text("examples: invalid\n", encoding="utf-8")

    monkeypatch.chdir(tmp_path)
    with pytest.raises(TypeError, match="examples mapping"):
        generate_config_examples.generate_config_examples()


def test_generate_config_examples_fails_when_markers_are_missing(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Fail fast when target markdown markers are missing."""
    docs_dir = tmp_path / "docs"
    quickstart_path = docs_dir / "guide" / "quickstart.md"
    configuration_path = docs_dir / "reference" / "configuration.md"
    manifest_path = docs_dir / "reference" / "config" / "examples-manifest.yaml"

    quickstart_path.parent.mkdir(parents=True, exist_ok=True)
    configuration_path.parent.mkdir(parents=True, exist_ok=True)
    manifest_path.parent.mkdir(parents=True, exist_ok=True)

    quickstart_path.write_text("# QuickStart\nNo markers\n", encoding="utf-8")
    configuration_path.write_text(
        (
            "# Configuration\n\n"
            "<!-- BEGIN GENERATED:configuration-minimal-config -->\n"
            "placeholder\n"
            "<!-- END GENERATED:configuration-minimal-config -->\n\n"
            "<!-- BEGIN GENERATED:configuration-typical-config -->\n"
            "placeholder\n"
            "<!-- END GENERATED:configuration-typical-config -->\n"
        ),
        encoding="utf-8",
    )
    manifest_path.write_text(
        (
            "examples:\n"
            '  quickstart_minimal: "x"\n'
            '  configuration_minimal: "x"\n'
            '  configuration_typical: "x"\n'
        ),
        encoding="utf-8",
    )

    monkeypatch.chdir(tmp_path)
    with pytest.raises(ValueError, match="Missing or invalid markers"):
        generate_config_examples.generate_config_examples()
