"""Strict configuration loading and precedence resolution for tq."""

from __future__ import annotations

import re
import tomllib
from pathlib import Path
from typing import Any

from tq.config.models import (
    CliOverrides,
    ConfigValidationError,
    PartialRuleConfig,
    PartialTargetConfig,
    PartialTqConfig,
    TqConfig,
    TqTargetConfig,
)
from tq.engine.rule_id import RuleId
from tq.rules.qualifiers import QualifierStrategy

DEFAULT_IGNORE_INIT_MODULES = False
DEFAULT_MAX_TEST_FILE_NON_BLANK_LINES = 600
DEFAULT_QUALIFIER_STRATEGY = QualifierStrategy.ANY_SUFFIX

_TARGET_NAME_PATTERN = re.compile(r"^[a-z0-9]+(?:-[a-z0-9]+)*$")

_TQ_KEYS = {
    "ignore_init_modules",
    "max_test_file_non_blank_lines",
    "qualifier_strategy",
    "allowed_qualifiers",
    "select",
    "ignore",
    "targets",
}

_TARGET_KEYS = {
    "name",
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

    cli_partial = _partial_from_cli(cli_overrides)
    return _materialize_config(
        cwd=cwd,
        partial=discovered,
        cli_defaults=cli_partial.defaults,
    )


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
            raise ConfigValidationError(msg)
        return PartialTqConfig()

    if not isinstance(tq_section, dict):
        msg = "[tool.tq] must be a table"
        raise ConfigValidationError(msg)

    unknown_keys = set(tq_section) - _TQ_KEYS
    if unknown_keys:
        keys = ", ".join(sorted(unknown_keys))
        msg = f"Unknown [tool.tq] key(s): {keys}"
        raise ConfigValidationError(msg)

    defaults = PartialRuleConfig(
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

    targets = _expect_optional_targets(tq_section, "targets")
    return PartialTqConfig(defaults=defaults, targets=targets)


def _partial_from_cli(overrides: CliOverrides) -> PartialTqConfig:
    """Convert CLI overrides into a partial config representation."""
    return PartialTqConfig(
        defaults=PartialRuleConfig(
            ignore_init_modules=overrides.ignore_init_modules,
            max_test_file_non_blank_lines=overrides.max_test_file_non_blank_lines,
            qualifier_strategy=overrides.qualifier_strategy,
            allowed_qualifiers=overrides.allowed_qualifiers,
            select=overrides.select,
            ignore=overrides.ignore,
        ),
    )


def _merge_partial(base: PartialTqConfig, override: PartialTqConfig) -> PartialTqConfig:
    """Merge two partial configs where ``override`` takes precedence."""
    return PartialTqConfig(
        defaults=_merge_rule_partial(base.defaults, override.defaults),
        targets=override.targets if override.targets is not None else base.targets,
    )


def _merge_rule_partial(
    base: PartialRuleConfig,
    override: PartialRuleConfig,
) -> PartialRuleConfig:
    """Merge per-rule partial values with override precedence."""
    return PartialRuleConfig(
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


def _materialize_config(
    *,
    cwd: Path,
    partial: PartialTqConfig,
    cli_defaults: PartialRuleConfig,
) -> TqConfig:
    """Validate and materialize a final runtime config."""
    if not partial.targets:
        msg = "Missing required configuration key: tool.tq.targets"
        raise ConfigValidationError(msg)

    normalized_targets: list[TqTargetConfig] = []
    seen_names: set[str] = set()
    seen_source_package_roots: set[Path] = set()

    for target in partial.targets:
        resolved = _materialize_target(
            cwd=cwd,
            target=target,
            defaults=partial.defaults,
            cli_defaults=cli_defaults,
        )

        if resolved.name in seen_names:
            msg = f"Duplicate target name in tool.tq.targets: {resolved.name}"
            raise ConfigValidationError(msg)
        seen_names.add(resolved.name)

        if resolved.source_package_root in seen_source_package_roots:
            path = resolved.source_package_root.as_posix()
            msg = f"Duplicate source package root across targets: {path}"
            raise ConfigValidationError(msg)
        seen_source_package_roots.add(resolved.source_package_root)

        normalized_targets.append(resolved)

    return TqConfig(
        targets=tuple(sorted(normalized_targets, key=lambda item: item.name)),
    )


def _materialize_target(
    *,
    cwd: Path,
    target: PartialTargetConfig,
    defaults: PartialRuleConfig,
    cli_defaults: PartialRuleConfig,
) -> TqTargetConfig:
    """Resolve one target with strict required and default semantics."""
    name = _require_target_key(target=target, key="name")
    if _TARGET_NAME_PATTERN.fullmatch(name) is None:
        msg = f"tool.tq.targets.name must be kebab-case: {name}"
        raise ConfigValidationError(msg)

    package = _require_target_key(target=target, key="package")
    source_root_value = _require_target_key(target=target, key="source_root")
    test_root_value = _require_target_key(target=target, key="test_root")

    merged_rules = _merge_rule_partial(
        defaults,
        PartialRuleConfig(
            ignore_init_modules=target.ignore_init_modules,
            max_test_file_non_blank_lines=target.max_test_file_non_blank_lines,
            qualifier_strategy=target.qualifier_strategy,
            allowed_qualifiers=target.allowed_qualifiers,
            select=target.select,
            ignore=target.ignore,
        ),
    )

    final_rules = _merge_rule_partial(merged_rules, cli_defaults)

    allowed_qualifiers = tuple(sorted(set(final_rules.allowed_qualifiers or ())))
    qualifier_strategy = final_rules.qualifier_strategy or DEFAULT_QUALIFIER_STRATEGY

    if qualifier_strategy is QualifierStrategy.ALLOWLIST and not allowed_qualifiers:
        msg = (
            "tool.tq.allowed_qualifiers must be non-empty when "
            "tool.tq.qualifier_strategy is 'allowlist'"
        )
        raise ConfigValidationError(msg)

    return TqTargetConfig(
        name=name,
        package=package,
        source_root=_resolve_path(cwd=cwd, value=source_root_value),
        test_root=_resolve_path(cwd=cwd, value=test_root_value),
        ignore_init_modules=(
            final_rules.ignore_init_modules
            if final_rules.ignore_init_modules is not None
            else DEFAULT_IGNORE_INIT_MODULES
        ),
        max_test_file_non_blank_lines=(
            final_rules.max_test_file_non_blank_lines
            if final_rules.max_test_file_non_blank_lines is not None
            else DEFAULT_MAX_TEST_FILE_NON_BLANK_LINES
        ),
        qualifier_strategy=qualifier_strategy,
        allowed_qualifiers=allowed_qualifiers,
        select=final_rules.select or (),
        ignore=final_rules.ignore or (),
    )


def _require_target_key(*, target: PartialTargetConfig, key: str) -> str:
    """Read a required non-empty target key."""
    value = getattr(target, key)
    if value is None:
        msg = f"Missing required target key: tool.tq.targets.{key}"
        raise ConfigValidationError(msg)
    if not value.strip():
        msg = f"tool.tq.targets.{key} must be non-empty"
        raise ConfigValidationError(msg)
    return value


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
        raise ConfigValidationError(msg) from error


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
            raise ConfigValidationError(msg)
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
            raise ConfigValidationError(msg) from error

    return tuple(rule_ids)


def _expect_optional_targets(
    document: dict[str, Any],
    key: str,
) -> tuple[PartialTargetConfig, ...] | None:
    """Read optional list of target mappings from `[tool.tq.targets]`."""
    value = document.get(key)
    if value is None:
        return None
    if not isinstance(value, list):
        msg = "tool.tq.targets must be an array of tables"
        raise ConfigValidationError(msg)

    targets: list[PartialTargetConfig] = []
    for item in value:
        if not isinstance(item, dict):
            msg = "tool.tq.targets entries must be tables"
            raise ConfigValidationError(msg)

        unknown_keys = set(item) - _TARGET_KEYS
        if unknown_keys:
            keys = ", ".join(sorted(unknown_keys))
            msg = f"Unknown tool.tq.targets key(s): {keys}"
            raise ConfigValidationError(msg)

        targets.append(
            PartialTargetConfig(
                name=_expect_optional_str(item, "name"),
                package=_expect_optional_str(item, "package"),
                source_root=_expect_optional_str(item, "source_root"),
                test_root=_expect_optional_str(item, "test_root"),
                ignore_init_modules=_expect_optional_bool(item, "ignore_init_modules"),
                max_test_file_non_blank_lines=_expect_optional_positive_int(
                    item,
                    "max_test_file_non_blank_lines",
                ),
                qualifier_strategy=_expect_optional_qualifier_strategy(
                    item,
                    "qualifier_strategy",
                ),
                allowed_qualifiers=_expect_optional_string_tuple(
                    item,
                    "allowed_qualifiers",
                ),
                select=_expect_optional_rule_ids(item, "select"),
                ignore=_expect_optional_rule_ids(item, "ignore"),
            ),
        )

    return tuple(targets)
