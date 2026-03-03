"""Generate canonical configuration examples in docs from a manifest."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any

import yaml

_MANIFEST_PATH = Path("docs/reference/config/examples-manifest.yaml")
_QUICKSTART_PATH = Path("docs/guide/quickstart.md")
_CONFIGURATION_PATH = Path("docs/reference/configuration.md")

_QUICKSTART_MINIMAL_START = "<!-- BEGIN GENERATED:quickstart-minimal-config -->"
_QUICKSTART_MINIMAL_END = "<!-- END GENERATED:quickstart-minimal-config -->"
_CONFIGURATION_MINIMAL_START = "<!-- BEGIN GENERATED:configuration-minimal-config -->"
_CONFIGURATION_MINIMAL_END = "<!-- END GENERATED:configuration-minimal-config -->"
_CONFIGURATION_TYPICAL_START = "<!-- BEGIN GENERATED:configuration-typical-config -->"
_CONFIGURATION_TYPICAL_END = "<!-- END GENERATED:configuration-typical-config -->"


@dataclass(frozen=True, slots=True)
class ConfigExamples:
    """Canonical configuration examples loaded from manifest."""

    quickstart_minimal: str
    configuration_minimal: str
    configuration_typical: str


def _require_text(*, item: dict[str, Any], key: str) -> str:
    """Return one required non-empty text field from a mapping."""
    value = item.get(key)
    if not isinstance(value, str) or not value.strip():
        msg = f"Config examples manifest field '{key}' must be non-empty text"
        raise ValueError(msg)
    return value.strip()


def _load_manifest() -> ConfigExamples:
    """Load and validate config examples manifest."""
    with _MANIFEST_PATH.open(encoding="utf-8") as handle:
        payload = yaml.safe_load(handle)

    if not isinstance(payload, dict):
        msg = "Config examples manifest must be a mapping"
        raise TypeError(msg)

    examples = payload.get("examples")
    if not isinstance(examples, dict):
        msg = "Config examples manifest must define an examples mapping"
        raise TypeError(msg)

    return ConfigExamples(
        quickstart_minimal=_require_text(item=examples, key="quickstart_minimal"),
        configuration_minimal=_require_text(
            item=examples,
            key="configuration_minimal",
        ),
        configuration_typical=_require_text(item=examples, key="configuration_typical"),
    )


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


def _render_toml_block(*, snippet: str) -> str:
    """Render one TOML fenced code block."""
    return f"```toml\n{snippet}\n```"


def generate_config_examples() -> None:
    """Generate canonical config examples in quickstart and reference docs."""
    examples = _load_manifest()

    _replace_between_markers(
        path=_QUICKSTART_PATH,
        start=_QUICKSTART_MINIMAL_START,
        end=_QUICKSTART_MINIMAL_END,
        replacement=_render_toml_block(snippet=examples.quickstart_minimal),
    )

    _replace_between_markers(
        path=_CONFIGURATION_PATH,
        start=_CONFIGURATION_MINIMAL_START,
        end=_CONFIGURATION_MINIMAL_END,
        replacement=_render_toml_block(snippet=examples.configuration_minimal),
    )

    _replace_between_markers(
        path=_CONFIGURATION_PATH,
        start=_CONFIGURATION_TYPICAL_START,
        end=_CONFIGURATION_TYPICAL_END,
        replacement=_render_toml_block(snippet=examples.configuration_typical),
    )


if __name__ == "__main__":
    generate_config_examples()
