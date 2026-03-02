"""Generate CLI and configuration option docs from click + manifest."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any

import click
import yaml

from tq.cli.main import cli

_MANIFEST_PATH = Path("docs/reference/cli/options-manifest.yaml")
_CLI_DOC_PATH = Path("docs/reference/cli.md")
_CONFIG_DOC_PATH = Path("docs/reference/configuration.md")

_CLI_MARKER_START = "<!-- BEGIN GENERATED:check-options -->"
_CLI_MARKER_END = "<!-- END GENERATED:check-options -->"
_CONFIG_MARKER_START = "<!-- BEGIN GENERATED:config-cli-mapping -->"
_CONFIG_MARKER_END = "<!-- END GENERATED:config-cli-mapping -->"


@dataclass(frozen=True, slots=True)
class OptionSpec:
    """Manifest metadata for one CLI option."""

    param: str
    config_key: str | None
    notes: str


def _load_manifest() -> tuple[OptionSpec, ...]:
    """Load and validate CLI options manifest."""
    with _MANIFEST_PATH.open(encoding="utf-8") as handle:
        payload = yaml.safe_load(handle)

    if not isinstance(payload, dict):
        msg = "CLI options manifest must be a mapping"
        raise TypeError(msg)

    options = payload.get("cli_options")
    if not isinstance(options, list):
        msg = "CLI options manifest must define a cli_options list"
        raise TypeError(msg)

    specs: list[OptionSpec] = []
    for item in options:
        if not isinstance(item, dict):
            msg = "Each cli_options item must be a mapping"
            raise TypeError(msg)
        param = _require_text(item=item, key="param")
        config_key = item.get("config_key")
        if config_key is not None and (
            not isinstance(config_key, str) or not config_key.strip()
        ):
            msg = "config_key must be null or a non-empty string"
            raise ValueError(msg)
        notes = _require_text(item=item, key="notes")
        specs.append(
            OptionSpec(
                param=param,
                config_key=config_key.strip() if isinstance(config_key, str) else None,
                notes=notes,
            ),
        )

    return tuple(specs)


def _require_text(*, item: dict[str, Any], key: str) -> str:
    """Read a required non-empty string from a mapping."""
    value = item.get(key)
    if not isinstance(value, str) or not value.strip():
        msg = f"Manifest field '{key}' must be a non-empty string"
        raise ValueError(msg)
    return value.strip()


def _load_check_options() -> dict[str, tuple[click.Option, ...]]:
    """Return click options from `tq check` grouped by parameter name."""
    check_command = cli.commands.get("check")
    if not isinstance(check_command, click.Command):
        msg = "Could not resolve `tq check` command"
        raise TypeError(msg)

    grouped: dict[str, list[click.Option]] = {}
    for param in check_command.params:
        if isinstance(param, click.Option) and param.name is not None:
            grouped.setdefault(param.name, []).append(param)

    return {name: tuple(option_list) for name, option_list in grouped.items()}


def _validate_manifest(
    *,
    specs: tuple[OptionSpec, ...],
    options: dict[str, tuple[click.Option, ...]],
) -> None:
    """Ensure manifest and click option sets remain synchronized."""
    manifest_names = {spec.param for spec in specs}
    option_names = set(options)

    missing = option_names - manifest_names
    if missing:
        names = ", ".join(sorted(missing))
        msg = f"Manifest missing click options: {names}"
        raise ValueError(msg)

    unknown = manifest_names - option_names
    if unknown:
        names = ", ".join(sorted(unknown))
        msg = f"Manifest contains unknown click options: {names}"
        raise ValueError(msg)


def _render_cli_table(
    *,
    specs: tuple[OptionSpec, ...],
    options: dict[str, tuple[click.Option, ...]],
) -> str:
    """Render generated `tq check` options table for CLI docs."""
    lines = [
        "## `tq check` options",
        "",
        "The table below documents the command definitions.",
        "",
        "| Flags | Config key | Default | Description |",
        "| --- | --- | --- | --- |",
    ]

    for spec in specs:
        option_group = options[spec.param]
        flags = _join_flags(option_group)
        config_ref = _config_key_ref(spec.config_key)
        default_text = _format_default(option_group)
        description = _render_help(option_group)
        lines.append(
            f"| `{flags}` | {config_ref} | `{default_text}` | {description} |",
        )

    lines.extend(["", "Run `tq check --help` for the runtime source of truth.", ""])
    return "\n".join(lines)


def _join_flags(option_group: tuple[click.Option, ...]) -> str:
    """Render unique flags from grouped click option definitions."""
    seen: set[str] = set()
    flags: list[str] = []
    for option in option_group:
        for flag in [*option.opts, *option.secondary_opts]:
            if flag in seen:
                continue
            seen.add(flag)
            flags.append(flag)
    return ", ".join(flags)


def _render_help(option_group: tuple[click.Option, ...]) -> str:
    """Render merged help text from grouped click option definitions."""
    seen: set[str] = set()
    pieces: list[str] = []
    for option in option_group:
        text = (option.help or "").strip()
        if not text or text in seen:
            continue
        seen.add(text)
        pieces.append(text)

    return " / ".join(pieces).replace("|", "\\|")


def _format_default(option_group: tuple[click.Option, ...]) -> str:
    """Format click option defaults for table display."""
    option = option_group[0]
    if option.default is None:
        return "none"
    if option.multiple:
        return "[]"
    if isinstance(option.default, bool):
        return "true" if option.default else "false"
    return str(option.default)


def _config_key_ref(config_key: str | None) -> str:
    """Return linked config-key reference or em dash for non-config options."""
    if config_key is None:
        return "—"
    suffix = (
        "required"
        if config_key in {"package", "source_root", "test_root"}
        else "optional"
    )
    anchor = f"{config_key}-{suffix}"
    return f"[`{config_key}`](./configuration.md#{anchor})"


def _render_config_mapping(
    *,
    specs: tuple[OptionSpec, ...],
    options: dict[str, tuple[click.Option, ...]],
) -> str:
    """Render generated configuration-to-CLI mapping table."""
    lines = [
        "## CLI mapping",
        "",
        "The table below is generated from the Click command definition and",
        "`docs/reference/cli/options-manifest.yaml`.",
        "",
        "| Config key | CLI flag(s) | Notes |",
        "| --- | --- | --- |",
    ]

    for spec in specs:
        if spec.config_key is None:
            continue
        flags = _join_flags(options[spec.param])
        notes = spec.notes.replace("|", "\\|")
        lines.append(f"| `{spec.config_key}` | `{flags}` | {notes} |")

    lines.append("")
    return "\n".join(lines)


def _replace_between_markers(
    *,
    path: Path,
    start: str,
    end: str,
    replacement: str,
) -> None:
    """Replace content between marker lines in a markdown file."""
    content = path.read_text(encoding="utf-8")
    start_index = content.find(start)
    end_index = content.find(end)
    if start_index == -1 or end_index == -1 or end_index < start_index:
        msg = f"Missing or invalid markers in {path}"
        raise ValueError(msg)

    start_index += len(start)
    new_content = (
        content[:start_index] + "\n\n" + replacement + "\n" + content[end_index:]
    )
    path.write_text(new_content, encoding="utf-8")


def generate_cli_docs() -> None:
    """Generate CLI and config docs sections from click and manifest data."""
    specs = _load_manifest()
    options = _load_check_options()
    _validate_manifest(specs=specs, options=options)

    _replace_between_markers(
        path=_CLI_DOC_PATH,
        start=_CLI_MARKER_START,
        end=_CLI_MARKER_END,
        replacement=_render_cli_table(specs=specs, options=options),
    )
    _replace_between_markers(
        path=_CONFIG_DOC_PATH,
        start=_CONFIG_MARKER_START,
        end=_CONFIG_MARKER_END,
        replacement=_render_config_mapping(specs=specs, options=options),
    )


if __name__ == "__main__":
    generate_cli_docs()
