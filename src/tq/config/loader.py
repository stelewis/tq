"""Strict configuration loading and precedence resolution for tq."""

from __future__ import annotations

import tomllib
from pathlib import Path
from typing import Any

from tq.config.models import (
    CliOverrides,
    ConfigValidationError,
    PartialTqConfig,
    TqConfig,
)
from tq.engine.rule_id import RuleId
from tq.rules.qualifiers import QualifierStrategy

DEFAULT_IGNORE_INIT_MODULES = False
DEFAULT_MAX_TEST_FILE_NON_BLANK_LINES = 600
DEFAULT_QUALIFIER_STRATEGY = QualifierStrategy.ANY_SUFFIX

_CONFIG_KEYS = {
    "package",
    "source_root",
    "test_root",
    "ignore_init_modules",
    "max_test_file_non_blank_lines",
    "qualifier_strategy",
    "allowed_qualifiers",
    "select",
    "ignore",
}


def resolve_tq_config(
    *,
    cwd: Path,
    explicit_config_path: Path | None,
    isolated: bool,
    cli_overrides: CliOverrides,
) -> TqConfig:
    """Resolve final tq config with strict precedence and validation."""
    discovered = PartialTqConfig()

    if explicit_config_path is not None:
        config_path = explicit_config_path.resolve()
        discovered = _load_partial_from_pyproject(
            path=config_path,
            require_section=True,
        )
    elif not isolated:
        user_config_path = Path.home() / ".config" / "tq" / "pyproject.toml"
        project_config_path = _find_project_pyproject(cwd)

        if user_config_path.exists():
            user_partial = _load_partial_from_pyproject(
                path=user_config_path,
                require_section=False,
            )
            discovered = _merge_partial(discovered, user_partial)

        if project_config_path is not None:
            project_partial = _load_partial_from_pyproject(
                path=project_config_path,
                require_section=False,
            )
            discovered = _merge_partial(discovered, project_partial)

    merged = _merge_partial(discovered, _partial_from_cli(cli_overrides))
    return _materialize_config(cwd=cwd, partial=merged)


def _find_project_pyproject(cwd: Path) -> Path | None:
    """Find nearest project ``pyproject.toml`` starting from cwd."""
    for candidate_dir in (cwd, *cwd.parents):
        candidate = candidate_dir / "pyproject.toml"
        if candidate.exists():
            return candidate
    return None


def _load_partial_from_pyproject(
    *,
    path: Path,
    require_section: bool,
) -> PartialTqConfig:
    """Load strict partial tq config from a pyproject file."""
    if not path.exists():
        msg = f"Config file not found: {path}"
        raise ConfigValidationError(msg)

    with path.open("rb") as handle:
        document = tomllib.load(handle)

    tool_section = document.get("tool", {})
    tq_section = tool_section.get("tq")

    if tq_section is None:
        if require_section:
            msg = f"Missing [tool.tq] section in config file: {path}"
            raise ConfigValidationError(
                msg,
            )
        return PartialTqConfig()

    if not isinstance(tq_section, dict):
        msg = "[tool.tq] must be a table"
        raise ConfigValidationError(msg)

    unknown_keys = set(tq_section) - _CONFIG_KEYS
    if unknown_keys:
        keys = ", ".join(sorted(unknown_keys))
        msg = f"Unknown [tool.tq] key(s): {keys}"
        raise ConfigValidationError(msg)

    return PartialTqConfig(
        package=_expect_optional_str(tq_section, "package"),
        source_root=_expect_optional_str(tq_section, "source_root"),
        test_root=_expect_optional_str(tq_section, "test_root"),
        ignore_init_modules=_expect_optional_bool(tq_section, "ignore_init_modules"),
        max_test_file_non_blank_lines=_expect_optional_positive_int(
            tq_section,
            "max_test_file_non_blank_lines",
        ),
        qualifier_strategy=_expect_optional_qualifier_strategy(
            tq_section,
            "qualifier_strategy",
        ),
        allowed_qualifiers=_expect_optional_string_tuple(
            tq_section,
            "allowed_qualifiers",
        ),
        select=_expect_optional_rule_ids(tq_section, "select"),
        ignore=_expect_optional_rule_ids(tq_section, "ignore"),
    )


def _partial_from_cli(overrides: CliOverrides) -> PartialTqConfig:
    """Convert CLI overrides into a partial config representation."""
    return PartialTqConfig(
        package=overrides.package,
        source_root=overrides.source_root,
        test_root=overrides.test_root,
        ignore_init_modules=overrides.ignore_init_modules,
        max_test_file_non_blank_lines=overrides.max_test_file_non_blank_lines,
        qualifier_strategy=overrides.qualifier_strategy,
        allowed_qualifiers=overrides.allowed_qualifiers,
        select=overrides.select,
        ignore=overrides.ignore,
    )


def _merge_partial(base: PartialTqConfig, override: PartialTqConfig) -> PartialTqConfig:
    """Merge two partial configs where ``override`` takes precedence."""
    return PartialTqConfig(
        package=override.package if override.package is not None else base.package,
        source_root=(
            override.source_root
            if override.source_root is not None
            else base.source_root
        ),
        test_root=(
            override.test_root if override.test_root is not None else base.test_root
        ),
        ignore_init_modules=(
            override.ignore_init_modules
            if override.ignore_init_modules is not None
            else base.ignore_init_modules
        ),
        max_test_file_non_blank_lines=(
            override.max_test_file_non_blank_lines
            if override.max_test_file_non_blank_lines is not None
            else base.max_test_file_non_blank_lines
        ),
        qualifier_strategy=(
            override.qualifier_strategy
            if override.qualifier_strategy is not None
            else base.qualifier_strategy
        ),
        allowed_qualifiers=(
            override.allowed_qualifiers
            if override.allowed_qualifiers is not None
            else base.allowed_qualifiers
        ),
        select=override.select if override.select is not None else base.select,
        ignore=override.ignore if override.ignore is not None else base.ignore,
    )


def _materialize_config(*, cwd: Path, partial: PartialTqConfig) -> TqConfig:
    """Validate and materialize a final runtime config."""
    if not partial.package:
        msg = "Missing required configuration key: tool.tq.package"
        raise ConfigValidationError(
            msg,
        )
    if not partial.source_root:
        msg = "Missing required configuration key: tool.tq.source_root"
        raise ConfigValidationError(
            msg,
        )
    if not partial.test_root:
        msg = "Missing required configuration key: tool.tq.test_root"
        raise ConfigValidationError(
            msg,
        )

    allowed_qualifiers = tuple(sorted(set(partial.allowed_qualifiers or ())))
    qualifier_strategy = partial.qualifier_strategy or DEFAULT_QUALIFIER_STRATEGY

    if qualifier_strategy is QualifierStrategy.ALLOWLIST and not allowed_qualifiers:
        msg = (
            "tool.tq.allowed_qualifiers must be non-empty when "
            "tool.tq.qualifier_strategy is 'allowlist'"
        )
        raise ConfigValidationError(
            msg,
        )

    source_root = _resolve_path(cwd=cwd, value=partial.source_root)
    test_root = _resolve_path(cwd=cwd, value=partial.test_root)

    return TqConfig(
        package=partial.package,
        source_root=source_root,
        test_root=test_root,
        ignore_init_modules=(
            partial.ignore_init_modules
            if partial.ignore_init_modules is not None
            else DEFAULT_IGNORE_INIT_MODULES
        ),
        max_test_file_non_blank_lines=(
            partial.max_test_file_non_blank_lines
            if partial.max_test_file_non_blank_lines is not None
            else DEFAULT_MAX_TEST_FILE_NON_BLANK_LINES
        ),
        qualifier_strategy=qualifier_strategy,
        allowed_qualifiers=allowed_qualifiers,
        select=partial.select or (),
        ignore=partial.ignore or (),
    )


def _resolve_path(*, cwd: Path, value: str) -> Path:
    """Resolve a config path relative to cwd."""
    candidate = Path(value)
    if candidate.is_absolute():
        return candidate
    return (cwd / candidate).resolve()


def _expect_optional_str(document: dict[str, Any], key: str) -> str | None:
    """Read optional non-empty string field from config document."""
    value = document.get(key)
    if value is None:
        return None
    if not isinstance(value, str):
        msg = f"tool.tq.{key} must be a string"
        raise ConfigValidationError(msg)
    if not value.strip():
        msg = f"tool.tq.{key} must be non-empty"
        raise ConfigValidationError(msg)
    return value


def _expect_optional_bool(document: dict[str, Any], key: str) -> bool | None:
    """Read optional boolean field from config document."""
    value = document.get(key)
    if value is None:
        return None
    if not isinstance(value, bool):
        msg = f"tool.tq.{key} must be a boolean"
        raise ConfigValidationError(msg)
    return value


def _expect_optional_positive_int(document: dict[str, Any], key: str) -> int | None:
    """Read optional positive integer field from config document."""
    value = document.get(key)
    if value is None:
        return None
    if not isinstance(value, int):
        msg = f"tool.tq.{key} must be an integer"
        raise ConfigValidationError(msg)
    if value < 1:
        msg = f"tool.tq.{key} must be >= 1"
        raise ConfigValidationError(msg)
    return value


def _expect_optional_qualifier_strategy(
    document: dict[str, Any],
    key: str,
) -> QualifierStrategy | None:
    """Read optional qualifier strategy enum value from config document."""
    value = document.get(key)
    if value is None:
        return None
    if not isinstance(value, str):
        msg = f"tool.tq.{key} must be a string"
        raise ConfigValidationError(msg)

    try:
        return QualifierStrategy(value)
    except ValueError as error:
        choices = ", ".join(strategy.value for strategy in QualifierStrategy)
        msg = f"tool.tq.{key} must be one of: {choices}"
        raise ConfigValidationError(
            msg,
        ) from error


def _expect_optional_string_tuple(
    document: dict[str, Any],
    key: str,
) -> tuple[str, ...] | None:
    """Read optional list of non-empty strings from config document."""
    value = document.get(key)
    if value is None:
        return None
    if not isinstance(value, list):
        msg = f"tool.tq.{key} must be an array of strings"
        raise ConfigValidationError(msg)

    items: list[str] = []
    for item in value:
        if not isinstance(item, str) or not item.strip():
            msg = f"tool.tq.{key} must contain only non-empty strings"
            raise ConfigValidationError(
                msg,
            )
        items.append(item)

    return tuple(items)


def _expect_optional_rule_ids(
    document: dict[str, Any],
    key: str,
) -> tuple[RuleId, ...] | None:
    """Read optional list of rule identifiers from config document."""
    values = _expect_optional_string_tuple(document, key)
    if values is None:
        return None

    rule_ids: list[RuleId] = []
    for value in values:
        try:
            rule_ids.append(RuleId(value))
        except ValueError as error:
            msg = f"tool.tq.{key} contains invalid rule id: {value}"
            raise ConfigValidationError(
                msg,
            ) from error

    return tuple(rule_ids)
