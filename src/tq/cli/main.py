"""Command-line interface for tq."""

from __future__ import annotations

import tomllib
from pathlib import Path
from typing import TYPE_CHECKING

import click
from rich.console import Console

from tq.config.loader import resolve_tq_config
from tq.config.models import CliOverrides, ConfigValidationError, TqTargetConfig
from tq.discovery.filesystem import build_analysis_index
from tq.engine.context import AnalysisContext
from tq.engine.rule_id import RuleId
from tq.engine.runner import RuleEngine, aggregate_results
from tq.reporting.json import print_json_report
from tq.reporting.terminal import print_report
from tq.rules.file_too_large import FileTooLargeRule
from tq.rules.mapping_missing_test import MappingMissingTestRule
from tq.rules.orphaned_test import OrphanedTestRule
from tq.rules.qualifiers import QualifierStrategy
from tq.rules.structure_mismatch import StructureMismatchRule

if TYPE_CHECKING:
    from tq.rules.contracts import Rule

_BUILTIN_RULE_IDS = (
    RuleId("mapping-missing-test"),
    RuleId("structure-mismatch"),
    RuleId("test-file-too-large"),
    RuleId("orphaned-test"),
)


_HELP_OPTION_NAMES = {"help_option_names": ["-h", "--help"]}


@click.group(context_settings=_HELP_OPTION_NAMES)
def cli() -> None:
    """Run tq commands."""


@cli.command("check", context_settings=_HELP_OPTION_NAMES)
@click.option(
    "--config",
    "config_path",
    type=click.Path(path_type=Path, dir_okay=False, exists=True),
    default=None,
    help="Use this pyproject file instead of discovered configuration.",
)
@click.option(
    "--isolated",
    is_flag=True,
    default=False,
    help="Ignore discovered configuration files.",
)
@click.option(
    "--target",
    "target_names",
    multiple=True,
    type=str,
    help="Run only listed target names.",
)
@click.option(
    "--max-test-file-non-blank-lines",
    type=click.IntRange(min=1),
    default=None,
    help="Maximum non-blank, non-comment lines per test file.",
)
@click.option(
    "--qualifier-strategy",
    type=click.Choice([strategy.value for strategy in QualifierStrategy]),
    default=None,
    help="Module-name qualifier policy for qualified test files.",
)
@click.option(
    "--allowed-qualifier",
    "allowed_qualifiers",
    multiple=True,
    type=str,
    help="Allowed qualifier suffix for allowlist strategy.",
)
@click.option(
    "--ignore-init-modules",
    "ignore_init_modules",
    flag_value=True,
    default=None,
    help="Ignore __init__.py modules in mapping checks.",
)
@click.option(
    "--no-ignore-init-modules",
    "ignore_init_modules",
    flag_value=False,
    default=None,
    help="Include __init__.py modules in mapping checks.",
)
@click.option(
    "--select",
    "select_rules",
    multiple=True,
    type=str,
    help="Only run selected rule IDs.",
)
@click.option(
    "--ignore",
    "ignore_rules",
    multiple=True,
    type=str,
    help="Skip listed rule IDs.",
)
@click.option(
    "--exit-zero",
    is_flag=True,
    default=False,
    help="Always exit with code 0 regardless of findings.",
)
@click.option(
    "--show-suggestions",
    is_flag=True,
    default=False,
    help="Render remediation suggestions in diagnostics output.",
)
@click.option(
    "--output-format",
    type=click.Choice(["text", "json"]),
    default="text",
    show_default=True,
    help="Select output format.",
)
def check_command(  # noqa: PLR0913
    *,
    config_path: Path | None,
    isolated: bool,
    target_names: tuple[str, ...],
    max_test_file_non_blank_lines: int | None,
    qualifier_strategy: str | None,
    allowed_qualifiers: tuple[str, ...],
    ignore_init_modules: bool | None,
    select_rules: tuple[str, ...],
    ignore_rules: tuple[str, ...],
    exit_zero: bool,
    show_suggestions: bool,
    output_format: str,
) -> None:
    """Run built-in tq quality rules against configured analysis targets."""
    cwd = Path.cwd()
    console = Console(stderr=False)

    try:
        overrides = CliOverrides(
            ignore_init_modules=ignore_init_modules,
            max_test_file_non_blank_lines=max_test_file_non_blank_lines,
            qualifier_strategy=(
                QualifierStrategy(qualifier_strategy)
                if qualifier_strategy is not None
                else None
            ),
            allowed_qualifiers=allowed_qualifiers or None,
            select=_parse_rule_id_tuple(values=select_rules),
            ignore=_parse_rule_id_tuple(values=ignore_rules),
        )
        config = resolve_tq_config(
            cwd=cwd,
            explicit_config_path=config_path,
            isolated=isolated,
            cli_overrides=overrides,
        )
        active_targets = _select_targets(
            configured_targets=config.targets,
            selected_target_names=target_names,
        )
    except (ConfigValidationError, ValueError, tomllib.TOMLDecodeError) as error:
        raise click.UsageError(str(error)) from error

    for target in active_targets:
        _validate_target_paths(target)

    target_results = []
    for target in active_targets:
        rules = _build_rules(config=target)
        index = build_analysis_index(
            source_root=target.source_package_root,
            test_root=target.test_root,
        )
        context = AnalysisContext.create(
            index=index,
            settings={"target_name": target.name},
        )
        target_results.append(RuleEngine(rules=rules).run(context=context))

    result = aggregate_results(results=tuple(target_results))

    if output_format == "json":
        print_json_report(result=result, console=console, cwd=cwd)
    else:
        print_report(
            result=result,
            console=console,
            cwd=cwd,
            include_suggestions=show_suggestions,
        )

    if exit_zero:
        raise click.exceptions.Exit(0)

    raise click.exceptions.Exit(1 if result.has_errors else 0)


def _build_rules(*, config: TqTargetConfig) -> tuple[Rule, ...]:
    """Build active built-in rule set using select/ignore resolution."""
    selected_rule_ids = _resolve_rule_selection(config=config)
    selected_set = set(selected_rule_ids)

    builtins: dict[RuleId, Rule] = {
        RuleId("mapping-missing-test"): MappingMissingTestRule(
            ignore_init_modules=config.ignore_init_modules,
            qualifier_strategy=config.qualifier_strategy,
            allowed_qualifiers=config.allowed_qualifiers,
        ),
        RuleId("structure-mismatch"): StructureMismatchRule(),
        RuleId("test-file-too-large"): FileTooLargeRule(
            max_non_blank_lines=config.max_test_file_non_blank_lines,
        ),
        RuleId("orphaned-test"): OrphanedTestRule(
            qualifier_strategy=config.qualifier_strategy,
            allowed_qualifiers=config.allowed_qualifiers,
        ),
    }

    return tuple(
        builtins[rule_id] for rule_id in _BUILTIN_RULE_IDS if rule_id in selected_set
    )


def _resolve_rule_selection(*, config: TqTargetConfig) -> tuple[RuleId, ...]:
    """Resolve active rule IDs deterministically from select/ignore."""
    builtin_set = set(_BUILTIN_RULE_IDS)

    requested_select = set(config.select)
    requested_ignore = set(config.ignore)

    unknown = (requested_select | requested_ignore) - builtin_set
    if unknown:
        unknown_ids = ", ".join(sorted(rule_id.value for rule_id in unknown))
        msg = f"Unknown built-in rule ID(s): {unknown_ids}"
        raise ConfigValidationError(msg)

    if config.select:
        selected = tuple(
            rule_id for rule_id in _BUILTIN_RULE_IDS if rule_id in requested_select
        )
    else:
        selected = _BUILTIN_RULE_IDS

    return tuple(rule_id for rule_id in selected if rule_id not in requested_ignore)


def _parse_rule_id_tuple(*, values: tuple[str, ...]) -> tuple[RuleId, ...] | None:
    """Parse optional CLI rule identifier list into RuleId values."""
    if not values:
        return None

    rule_ids: list[RuleId] = []
    for value in values:
        try:
            rule_ids.append(RuleId(value))
        except ValueError as error:
            msg = f"Invalid rule ID: {value}"
            raise ConfigValidationError(msg) from error

    return tuple(rule_ids)


def _select_targets(
    *,
    configured_targets: tuple[TqTargetConfig, ...],
    selected_target_names: tuple[str, ...],
) -> tuple[TqTargetConfig, ...]:
    """Select active targets from configured targets and CLI target names."""
    if not selected_target_names:
        return configured_targets

    by_name = {target.name: target for target in configured_targets}
    unknown_names = sorted(set(selected_target_names) - set(by_name))
    if unknown_names:
        names = ", ".join(unknown_names)
        msg = f"Unknown target name(s): {names}"
        raise ConfigValidationError(msg)

    selected = [by_name[name] for name in selected_target_names]
    return tuple(selected)


def _validate_target_paths(target: TqTargetConfig) -> None:
    """Validate filesystem paths for one configured target."""
    if not target.source_package_root.exists():
        msg = (
            "Configured source package root does not exist "
            f"for target '{target.name}': {target.source_package_root}"
        )
        raise click.UsageError(msg)

    if not target.test_root.exists():
        msg = (
            f"Configured test root does not exist "
            f"for target '{target.name}': {target.test_root}"
        )
        raise click.UsageError(msg)


def main() -> None:
    """Run the tq command group."""
    cli()


if __name__ == "__main__":
    main()
