"""Tests for strict tq configuration loading and precedence."""

from __future__ import annotations

from typing import TYPE_CHECKING

import pytest

from tq.config.loader import resolve_tq_config
from tq.config.models import CliOverrides, ConfigValidationError
from tq.rules.qualifiers import QualifierStrategy

if TYPE_CHECKING:
    from pathlib import Path


def test_resolve_requires_targets(tmp_path: Path) -> None:
    """Reject missing targets after configuration resolution."""
    config_path = tmp_path / "pyproject.toml"
    config_path.write_text("[tool.tq]\n", encoding="utf-8")

    with pytest.raises(ConfigValidationError, match=r"tool\.tq\.targets"):
        resolve_tq_config(
            cwd=tmp_path,
            explicit_config_path=config_path,
            isolated=True,
            cli_overrides=CliOverrides(),
        )


def test_resolve_rejects_unknown_tool_tq_keys(tmp_path: Path) -> None:
    """Fail fast when [tool.tq] includes unknown keys."""
    config_path = tmp_path / "pyproject.toml"
    config_path.write_text(
        "[tool.tq]\nunknown = 1\n",
        encoding="utf-8",
    )

    with pytest.raises(ConfigValidationError, match=r"Unknown \[tool\.tq\] key"):
        resolve_tq_config(
            cwd=tmp_path,
            explicit_config_path=config_path,
            isolated=False,
            cli_overrides=CliOverrides(),
        )


def test_resolve_rejects_duplicate_target_names(tmp_path: Path) -> None:
    """Fail fast when target names are duplicated."""
    config_path = tmp_path / "pyproject.toml"
    config_path.write_text(
        (
            "[tool.tq]\n"
            "[[tool.tq.targets]]\n"
            'name = "core"\n'
            'package = "tq"\n'
            'source_root = "src"\n'
            'test_root = "tests"\n\n'
            "[[tool.tq.targets]]\n"
            'name = "core"\n'
            'package = "scripts"\n'
            'source_root = "."\n'
            'test_root = "tests"\n'
        ),
        encoding="utf-8",
    )

    with pytest.raises(ConfigValidationError, match="Duplicate target name"):
        resolve_tq_config(
            cwd=tmp_path,
            explicit_config_path=config_path,
            isolated=False,
            cli_overrides=CliOverrides(),
        )


def test_resolve_reports_target_key_path_for_invalid_target_field(
    tmp_path: Path,
) -> None:
    """Report precise target key path for invalid target field types."""
    config_path = tmp_path / "pyproject.toml"
    config_path.write_text(
        (
            "[tool.tq]\n"
            "[[tool.tq.targets]]\n"
            "name = 123\n"
            'package = "tq"\n'
            'source_root = "src"\n'
            'test_root = "tests"\n'
        ),
        encoding="utf-8",
    )

    with pytest.raises(
        ConfigValidationError,
        match=r"tool\.tq\.targets\[0\]\.name must be a string",
    ):
        resolve_tq_config(
            cwd=tmp_path,
            explicit_config_path=config_path,
            isolated=False,
            cli_overrides=CliOverrides(),
        )


def test_resolve_reports_indexed_error_for_invalid_targets_entry_type(
    tmp_path: Path,
) -> None:
    """Include array index when target entry is not a TOML table."""
    config_path = tmp_path / "pyproject.toml"
    config_path.write_text(
        "[tool.tq]\ntargets = [123]\n",
        encoding="utf-8",
    )

    with pytest.raises(
        ConfigValidationError,
        match=r"tool\.tq\.targets\[0\] must be a table",
    ):
        resolve_tq_config(
            cwd=tmp_path,
            explicit_config_path=config_path,
            isolated=False,
            cli_overrides=CliOverrides(),
        )


def test_resolve_rejects_invalid_target_package_import_syntax(tmp_path: Path) -> None:
    """Reject package values that are not dotted Python identifiers."""
    config_path = tmp_path / "pyproject.toml"
    config_path.write_text(
        (
            "[tool.tq]\n"
            "[[tool.tq.targets]]\n"
            'name = "core"\n'
            'package = "pkg..core"\n'
            'source_root = "src"\n'
            'test_root = "tests"\n'
        ),
        encoding="utf-8",
    )

    with pytest.raises(
        ConfigValidationError,
        match=r"tool\.tq\.targets\[0\]\.package must be dotted Python identifiers",
    ):
        resolve_tq_config(
            cwd=tmp_path,
            explicit_config_path=config_path,
            isolated=False,
            cli_overrides=CliOverrides(),
        )


def test_resolve_rejects_duplicate_allowed_qualifiers_in_target(tmp_path: Path) -> None:
    """Reject duplicate allowed qualifiers in one target configuration."""
    config_path = tmp_path / "pyproject.toml"
    config_path.write_text(
        (
            "[tool.tq]\n"
            "[[tool.tq.targets]]\n"
            'name = "core"\n'
            'package = "tq"\n'
            'source_root = "src"\n'
            'test_root = "tests"\n'
            'allowed_qualifiers = ["regression", "regression"]\n'
        ),
        encoding="utf-8",
    )

    with pytest.raises(
        ConfigValidationError,
        match=r"tool\.tq\.targets\[0\]\.allowed_qualifiers contains duplicate value",
    ):
        resolve_tq_config(
            cwd=tmp_path,
            explicit_config_path=config_path,
            isolated=False,
            cli_overrides=CliOverrides(),
        )


def test_resolve_rejects_duplicate_rule_ids_in_select(tmp_path: Path) -> None:
    """Reject duplicate rule IDs in one target select list."""
    config_path = tmp_path / "pyproject.toml"
    config_path.write_text(
        (
            "[tool.tq]\n"
            "[[tool.tq.targets]]\n"
            'name = "core"\n'
            'package = "tq"\n'
            'source_root = "src"\n'
            'test_root = "tests"\n'
            'select = ["mapping-missing-test", "mapping-missing-test"]\n'
        ),
        encoding="utf-8",
    )

    with pytest.raises(
        ConfigValidationError,
        match=r"tool\.tq\.targets\[0\]\.select contains duplicate value",
    ):
        resolve_tq_config(
            cwd=tmp_path,
            explicit_config_path=config_path,
            isolated=False,
            cli_overrides=CliOverrides(),
        )


def test_resolve_rejects_duplicate_cli_allowed_qualifiers(tmp_path: Path) -> None:
    """Reject duplicate CLI allowed qualifier overrides."""
    config_path = tmp_path / "pyproject.toml"
    config_path.write_text(
        (
            "[tool.tq]\n"
            "[[tool.tq.targets]]\n"
            'name = "core"\n'
            'package = "tq"\n'
            'source_root = "src"\n'
            'test_root = "tests"\n'
        ),
        encoding="utf-8",
    )

    with pytest.raises(
        ConfigValidationError,
        match=r"cli\.allowed_qualifiers contains duplicate value",
    ):
        resolve_tq_config(
            cwd=tmp_path,
            explicit_config_path=config_path,
            isolated=False,
            cli_overrides=CliOverrides(
                allowed_qualifiers=("regression", "regression"),
            ),
        )


def test_cli_overrides_precede_config_defaults(tmp_path: Path) -> None:
    """Apply explicit CLI options over config defaults for all targets."""
    config_path = tmp_path / "pyproject.toml"
    config_path.write_text(
        (
            "[tool.tq]\n"
            "ignore_init_modules = false\n"
            'qualifier_strategy = "none"\n'
            "[[tool.tq.targets]]\n"
            'name = "core"\n'
            'package = "tq"\n'
            'source_root = "src"\n'
            'test_root = "tests"\n'
        ),
        encoding="utf-8",
    )

    config = resolve_tq_config(
        cwd=tmp_path,
        explicit_config_path=config_path,
        isolated=False,
        cli_overrides=CliOverrides(
            ignore_init_modules=True,
            qualifier_strategy=QualifierStrategy.ANY_SUFFIX,
        ),
    )

    assert len(config.targets) == 1
    assert config.targets[0].ignore_init_modules is True
    assert config.targets[0].qualifier_strategy is QualifierStrategy.ANY_SUFFIX


def test_explicit_config_overrides_discovered_project_config(tmp_path: Path) -> None:
    """Use explicit --config values instead of discovered pyproject settings."""
    project_config = tmp_path / "pyproject.toml"
    project_config.write_text(
        (
            "[tool.tq]\n"
            "[[tool.tq.targets]]\n"
            'name = "wrong"\n'
            'package = "wrong"\n'
            'source_root = "src"\n'
            'test_root = "tests"\n'
        ),
        encoding="utf-8",
    )

    explicit_config = tmp_path / "alternate.toml"
    explicit_config.write_text(
        (
            "[tool.tq]\n"
            "[[tool.tq.targets]]\n"
            'name = "core"\n'
            'package = "tq"\n'
            'source_root = "src"\n'
            'test_root = "tests"\n'
        ),
        encoding="utf-8",
    )

    config = resolve_tq_config(
        cwd=tmp_path,
        explicit_config_path=explicit_config,
        isolated=False,
        cli_overrides=CliOverrides(),
    )

    assert [target.name for target in config.targets] == ["core"]
    assert config.targets[0].source_root == (tmp_path / "src").resolve()
    assert config.targets[0].test_root == (tmp_path / "tests").resolve()
