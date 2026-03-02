"""Tests for strict tq configuration loading and precedence."""

from __future__ import annotations

from pathlib import Path

import pytest

from tq.config.loader import resolve_tq_config
from tq.config.models import CliOverrides, ConfigValidationError
from tq.rules.qualifiers import QualifierStrategy


def test_resolve_requires_package_source_and_test_roots(tmp_path: Path) -> None:
    """Reject missing required keys after resolution."""
    with pytest.raises(ConfigValidationError):
        resolve_tq_config(
            cwd=tmp_path,
            explicit_config_path=None,
            isolated=True,
            cli_overrides=CliOverrides(),
        )


def test_resolve_rejects_unknown_tool_tq_keys(tmp_path: Path) -> None:
    """Fail fast when [tool.tq] includes unknown keys."""
    config_path = tmp_path / "pyproject.toml"
    config_path.write_text(
        "\n".join(
            [
                "[tool.tq]",
                'package = "tq"',
                'source_root = "src"',
                'test_root = "tests"',
                "unknown = 1",
            ]
        ),
        encoding="utf-8",
    )

    with pytest.raises(ConfigValidationError):
        resolve_tq_config(
            cwd=tmp_path,
            explicit_config_path=config_path,
            isolated=False,
            cli_overrides=CliOverrides(),
        )


def test_cli_overrides_precede_config_values(tmp_path: Path) -> None:
    """Apply explicit CLI options after file-based configuration."""
    config_path = tmp_path / "pyproject.toml"
    config_path.write_text(
        "\n".join(
            [
                "[tool.tq]",
                'package = "demo"',
                'source_root = "src"',
                'test_root = "tests"',
                'qualifier_strategy = "none"',
                "ignore_init_modules = false",
            ]
        ),
        encoding="utf-8",
    )

    config = resolve_tq_config(
        cwd=tmp_path,
        explicit_config_path=config_path,
        isolated=False,
        cli_overrides=CliOverrides(
            package="tq",
            ignore_init_modules=True,
            qualifier_strategy=QualifierStrategy.ANY_SUFFIX,
        ),
    )

    assert config.package == "tq"
    assert config.ignore_init_modules is True
    assert config.qualifier_strategy is QualifierStrategy.ANY_SUFFIX


def test_explicit_config_overrides_discovered_project_config(tmp_path: Path) -> None:
    """Use explicit --config values instead of discovered pyproject settings."""
    project_config = tmp_path / "pyproject.toml"
    project_config.write_text(
        "\n".join(
            [
                "[tool.tq]",
                'package = "wrong"',
                'source_root = "src"',
                'test_root = "tests"',
            ]
        ),
        encoding="utf-8",
    )

    explicit_config = tmp_path / "alternate.toml"
    explicit_config.write_text(
        "\n".join(
            [
                "[tool.tq]",
                'package = "tq"',
                'source_root = "src"',
                'test_root = "tests"',
            ]
        ),
        encoding="utf-8",
    )

    config = resolve_tq_config(
        cwd=tmp_path,
        explicit_config_path=explicit_config,
        isolated=False,
        cli_overrides=CliOverrides(),
    )

    assert config.package == "tq"
    assert config.source_root == (tmp_path / "src").resolve()
    assert config.test_root == (tmp_path / "tests").resolve()
