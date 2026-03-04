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
    targets_base_dir: Path | None = None

    if explicit_config_path is not None:
        config_path = explicit_config_path.resolve()
        discovered = _load_partial_from_pyproject(
            path=config_path,
            require_section=True,
        )
        targets_base_dir = _resolve_targets_base_dir(
            current=targets_base_dir,
            loaded=discovered,
            config_path=config_path,
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
            targets_base_dir = _resolve_targets_base_dir(
                current=targets_base_dir,
                loaded=user_partial,
                config_path=user_config_path,
            )

        if project_config_path is not None:
            project_partial = _load_partial_from_pyproject(
                path=project_config_path,
                require_section=False,
            )
            discovered = _merge_partial(discovered, project_partial)
            targets_base_dir = _resolve_targets_base_dir(
                current=targets_base_dir,
                loaded=project_partial,
                config_path=project_config_path,
            )

    cli_partial = _partial_from_cli(cli_overrides)
    return _materialize_config(
        cwd=cwd,
        partial=discovered,
        cli_defaults=cli_partial.defaults,
        targets_base_dir=targets_base_dir,
    )


def _resolve_targets_base_dir(
    *,
    current: Path | None,
    loaded: PartialTqConfig,
    config_path: Path,
) -> Path | None:
    """Track directory origin for the currently active targets payload."""
    if loaded.targets is None:
        return current
    return config_path.parent.resolve()


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
        ignore_init_modules=_expect_optional_bool(
            tq_section,
            "ignore_init_modules",
            location="tool.tq",
        ),
        max_test_file_non_blank_lines=_expect_optional_positive_int(
            tq_section,
            "max_test_file_non_blank_lines",
            location="tool.tq",
        ),
        qualifier_strategy=_expect_optional_qualifier_strategy(
            tq_section,
            "qualifier_strategy",
            location="tool.tq",
        ),
        allowed_qualifiers=_expect_optional_string_tuple(
            tq_section,
            "allowed_qualifiers",
            location="tool.tq",
            require_unique=True,
        ),
        select=_expect_optional_rule_ids(
            tq_section,
            "select",
            location="tool.tq",
        ),
        ignore=_expect_optional_rule_ids(
            tq_section,
            "ignore",
            location="tool.tq",
        ),
    )

    targets = _expect_optional_targets(tq_section, "targets")
    return PartialTqConfig(defaults=defaults, targets=targets)


def _partial_from_cli(overrides: CliOverrides) -> PartialTqConfig:
    """Convert CLI overrides into a partial config representation."""
    if overrides.allowed_qualifiers is not None:
        _raise_on_duplicate_strings(
            values=overrides.allowed_qualifiers,
            location="cli.allowed_qualifiers",
        )

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
    targets_base_dir: Path | None,
) -> TqConfig:
    """Validate and materialize a final runtime config."""
    if not partial.targets:
        msg = "Missing required configuration key: tool.tq.targets"
        raise ConfigValidationError(msg)

    normalized_targets: list[TqTargetConfig] = []
    seen_names: dict[str, int] = {}
    seen_source_package_roots: dict[Path, int] = {}

    for target_index, target in enumerate(partial.targets):
        resolved = _materialize_target(
            targets_base_dir=targets_base_dir or cwd,
            target=target,
            defaults=partial.defaults,
            cli_defaults=cli_defaults,
            target_index=target_index,
        )

        first_index = seen_names.get(resolved.name)
        if first_index is not None:
            msg = (
                "Duplicate target name in "
                f"tool.tq.targets[{first_index}].name and "
                f"tool.tq.targets[{target_index}].name: {resolved.name}"
            )
            raise ConfigValidationError(msg)
        seen_names[resolved.name] = target_index

        first_source_root_index = seen_source_package_roots.get(
            resolved.source_package_root,
        )
        if first_source_root_index is not None:
            path = resolved.source_package_root.as_posix()
            msg = (
                "Duplicate source package root across "
                f"tool.tq.targets[{first_source_root_index}] and "
                f"tool.tq.targets[{target_index}]: {path}"
            )
            raise ConfigValidationError(msg)
        seen_source_package_roots[resolved.source_package_root] = target_index

        normalized_targets.append(resolved)

    return TqConfig(
        targets=tuple(sorted(normalized_targets, key=lambda item: item.name)),
    )


def _materialize_target(
    *,
    targets_base_dir: Path,
    target: PartialTargetConfig,
    defaults: PartialRuleConfig,
    cli_defaults: PartialRuleConfig,
    target_index: int,
) -> TqTargetConfig:
    """Resolve one target with strict required and default semantics."""
    target_location = f"tool.tq.targets[{target_index}]"

    name = _require_target_key(
        target=target,
        key="name",
        target_location=target_location,
    )
    if _TARGET_NAME_PATTERN.fullmatch(name) is None:
        msg = f"{target_location}.name must be kebab-case: {name}"
        raise ConfigValidationError(msg)

    package = _require_target_key(
        target=target,
        key="package",
        target_location=target_location,
    )
    _validate_python_package_name(
        package,
        location=f"{target_location}.package",
    )
    source_root_value = _require_target_key(
        target=target,
        key="source_root",
        target_location=target_location,
    )
    test_root_value = _require_target_key(
        target=target,
        key="test_root",
        target_location=target_location,
    )

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

    allowed_qualifiers = final_rules.allowed_qualifiers or ()
    qualifier_strategy = final_rules.qualifier_strategy or DEFAULT_QUALIFIER_STRATEGY

    if qualifier_strategy is QualifierStrategy.ALLOWLIST and not allowed_qualifiers:
        msg = (
            f"{target_location}.allowed_qualifiers must be non-empty when "
            "effective qualifier_strategy is 'allowlist'"
        )
        raise ConfigValidationError(msg)

    return TqTargetConfig(
        name=name,
        package=package,
        source_root=_resolve_path(base_dir=targets_base_dir, value=source_root_value),
        test_root=_resolve_path(base_dir=targets_base_dir, value=test_root_value),
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


def _require_target_key(
    *,
    target: PartialTargetConfig,
    key: str,
    target_location: str,
) -> str:
    """Read a required non-empty target key."""
    value = getattr(target, key)
    if value is None:
        msg = f"Missing required target key: {target_location}.{key}"
        raise ConfigValidationError(msg)
    if not value.strip():
        msg = f"{target_location}.{key} must be non-empty"
        raise ConfigValidationError(msg)
    return value


def _validate_python_package_name(value: str, *, location: str) -> None:
    """Validate package string as dotted Python identifier segments."""
    segments = value.split(".")
    if any(not segment or not segment.isidentifier() for segment in segments):
        msg = f"{location} must be dotted Python identifiers"
        raise ConfigValidationError(msg)


def _resolve_path(*, base_dir: Path, value: str) -> Path:
    """Resolve a config path relative to its owning config directory."""
    candidate = Path(value)
    if candidate.is_absolute():
        return candidate
    return (base_dir / candidate).resolve()


def _expect_optional_str(
    document: dict[str, Any],
    key: str,
    *,
    location: str,
) -> str | None:
    """Read optional non-empty string field from config document."""
    value = document.get(key)
    if value is None:
        return None
    if not isinstance(value, str):
        msg = f"{location}.{key} must be a string"
        raise ConfigValidationError(msg)
    if not value.strip():
        msg = f"{location}.{key} must be non-empty"
        raise ConfigValidationError(msg)
    return value


def _expect_optional_bool(
    document: dict[str, Any],
    key: str,
    *,
    location: str,
) -> bool | None:
    """Read optional boolean field from config document."""
    value = document.get(key)
    if value is None:
        return None
    if not isinstance(value, bool):
        msg = f"{location}.{key} must be a boolean"
        raise ConfigValidationError(msg)
    return value


def _expect_optional_positive_int(
    document: dict[str, Any],
    key: str,
    *,
    location: str,
) -> int | None:
    """Read optional positive integer field from config document."""
    value = document.get(key)
    if value is None:
        return None
    if not isinstance(value, int):
        msg = f"{location}.{key} must be an integer"
        raise ConfigValidationError(msg)
    if value < 1:
        msg = f"{location}.{key} must be >= 1"
        raise ConfigValidationError(msg)
    return value


def _expect_optional_qualifier_strategy(
    document: dict[str, Any],
    key: str,
    *,
    location: str,
) -> QualifierStrategy | None:
    """Read optional qualifier strategy enum value from config document."""
    value = document.get(key)
    if value is None:
        return None
    if not isinstance(value, str):
        msg = f"{location}.{key} must be a string"
        raise ConfigValidationError(msg)

    try:
        return QualifierStrategy(value)
    except ValueError as error:
        choices = ", ".join(strategy.value for strategy in QualifierStrategy)
        msg = f"{location}.{key} must be one of: {choices}"
        raise ConfigValidationError(msg) from error


def _expect_optional_string_tuple(
    document: dict[str, Any], key: str, *, location: str, require_unique: bool
) -> tuple[str, ...] | None:
    """Read optional list of non-empty strings from config document."""
    value = document.get(key)
    if value is None:
        return None

    if not isinstance(value, list):
        msg = f"{location}.{key} must be an array of strings"
        raise ConfigValidationError(msg)

    items: list[str] = []
    seen_indices: dict[str, int] = {}
    for index, item in enumerate(value):
        if not isinstance(item, str) or not item.strip():
            msg = f"{location}.{key}[{index}] must be a non-empty string"
            raise ConfigValidationError(msg)
        if require_unique:
            first_index = seen_indices.get(item)
            if first_index is not None:
                msg = (
                    f"{location}.{key} contains duplicate value {item!r} "
                    f"at indices {first_index} and {index}"
                )
                raise ConfigValidationError(msg)
            seen_indices[item] = index
        items.append(item)

    return tuple(items)


def _raise_on_duplicate_strings(*, values: tuple[str, ...], location: str) -> None:
    """Raise if tuple of strings includes duplicate values."""
    seen_indices: dict[str, int] = {}
    for index, value in enumerate(values):
        first_index = seen_indices.get(value)
        if first_index is not None:
            msg = (
                f"{location} contains duplicate value {value!r} "
                f"at indices {first_index} and {index}"
            )
            raise ConfigValidationError(msg)
        seen_indices[value] = index


def _expect_optional_rule_ids(
    document: dict[str, Any],
    key: str,
    *,
    location: str,
) -> tuple[RuleId, ...] | None:
    """Read optional list of rule identifiers from config document."""
    values = _expect_optional_string_tuple(
        document,
        key,
        location=location,
        require_unique=True,
    )
    if values is None:
        return None

    rule_ids: list[RuleId] = []
    for index, value in enumerate(values):
        try:
            rule_ids.append(RuleId(value))
        except ValueError as error:
            msg = f"{location}.{key}[{index}] contains invalid rule id: {value}"
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
    for target_index, item in enumerate(value):
        target_location = f"tool.tq.targets[{target_index}]"
        if not isinstance(item, dict):
            msg = f"{target_location} must be a table"
            raise ConfigValidationError(msg)

        unknown_keys = set(item) - _TARGET_KEYS
        if unknown_keys:
            keys = ", ".join(sorted(unknown_keys))
            msg = f"Unknown key(s) in {target_location}: {keys}"
            raise ConfigValidationError(msg)

        targets.append(
            PartialTargetConfig(
                name=_expect_optional_str(
                    item,
                    "name",
                    location=target_location,
                ),
                package=_expect_optional_str(
                    item,
                    "package",
                    location=target_location,
                ),
                source_root=_expect_optional_str(
                    item,
                    "source_root",
                    location=target_location,
                ),
                test_root=_expect_optional_str(
                    item,
                    "test_root",
                    location=target_location,
                ),
                ignore_init_modules=_expect_optional_bool(
                    item,
                    "ignore_init_modules",
                    location=target_location,
                ),
                max_test_file_non_blank_lines=_expect_optional_positive_int(
                    item,
                    "max_test_file_non_blank_lines",
                    location=target_location,
                ),
                qualifier_strategy=_expect_optional_qualifier_strategy(
                    item,
                    "qualifier_strategy",
                    location=target_location,
                ),
                allowed_qualifiers=_expect_optional_string_tuple(
                    item,
                    "allowed_qualifiers",
                    location=target_location,
                    require_unique=True,
                ),
                select=_expect_optional_rule_ids(
                    item,
                    "select",
                    location=target_location,
                ),
                ignore=_expect_optional_rule_ids(
                    item,
                    "ignore",
                    location=target_location,
                ),
            ),
        )

    return tuple(targets)
