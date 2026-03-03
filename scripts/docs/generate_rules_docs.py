"""Generate rules reference docs from the canonical manifest."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any

import yaml


@dataclass(frozen=True, slots=True)
class RuleManifestEntry:
    """Canonical manifest entry for one stable rule."""

    rule_id: str
    default_severity: str
    added_in: str
    behavior_changes: str
    what_it_does: str
    why_this_matters: str
    trigger_conditions: tuple[str, ...]
    examples: tuple[tuple[str, str], ...]
    how_to_address: tuple[str, ...]
    related_controls: tuple[str, ...]


def _load_manifest(*, manifest_path: Path) -> tuple[RuleManifestEntry, ...]:
    """Load and validate rule entries from a manifest YAML file."""
    with manifest_path.open(encoding="utf-8") as handle:
        payload = yaml.safe_load(handle)

    if not isinstance(payload, dict):
        msg = "Rules manifest must be a mapping"
        raise TypeError(msg)

    raw_rules = payload.get("rules")
    if not isinstance(raw_rules, list):
        msg = "Rules manifest must contain a 'rules' list"
        raise TypeError(msg)

    entries: list[RuleManifestEntry] = []
    for item in raw_rules:
        if not isinstance(item, dict):
            msg = "Each rules manifest item must be a mapping"
            raise TypeError(msg)

        entries.append(_parse_entry(item=item))

    return tuple(entries)


def _parse_entry(*, item: dict[str, Any]) -> RuleManifestEntry:
    """Parse one raw manifest mapping into a typed rule entry."""
    examples_value = item.get("examples", [])
    if not isinstance(examples_value, list):
        msg = "Rule manifest 'examples' must be a list"
        raise TypeError(msg)

    examples: list[tuple[str, str]] = []
    for example in examples_value:
        if not isinstance(example, dict):
            msg = "Rule example must be a mapping"
            raise TypeError(msg)
        source = str(example.get("source", "n/a")).strip()
        test = str(example.get("test", "n/a")).strip()
        examples.append((source, test))

    return RuleManifestEntry(
        rule_id=_require_text(item=item, key="id"),
        default_severity=_require_text(item=item, key="default_severity"),
        added_in=_require_text(item=item, key="added_in"),
        behavior_changes=_require_text(item=item, key="behavior_changes"),
        what_it_does=_require_text(item=item, key="what_it_does"),
        why_this_matters=_require_text(item=item, key="why_this_matters"),
        trigger_conditions=tuple(
            _require_text_list(item=item, key="trigger_conditions")
        ),
        examples=tuple(examples),
        how_to_address=tuple(_require_text_list(item=item, key="how_to_address")),
        related_controls=tuple(_require_text_list(item=item, key="related_controls")),
    )


def _require_text(*, item: dict[str, Any], key: str) -> str:
    """Read a required, non-empty text field from a manifest mapping."""
    value = item.get(key)
    if not isinstance(value, str) or not value.strip():
        msg = f"Rule manifest field '{key}' must be a non-empty string"
        raise ValueError(msg)
    return value.strip()


def _require_text_list(*, item: dict[str, Any], key: str) -> list[str]:
    """Read a required list of non-empty text values from a manifest mapping."""
    value = item.get(key)
    if not isinstance(value, list):
        msg = f"Rule manifest field '{key}' must be a list"
        raise TypeError(msg)

    values: list[str] = []
    for raw_value in value:
        if not isinstance(raw_value, str) or not raw_value.strip():
            msg = f"Rule manifest field '{key}' must contain non-empty strings"
            raise ValueError(msg)
        values.append(raw_value.strip())

    return values


def _render_index(*, entries: tuple[RuleManifestEntry, ...]) -> str:
    """Render canonical rules index markdown from manifest entries."""
    lines = [
        "# Rules",
        "",
        "This is the canonical user-facing rules index.",
        "",
        "Rule metadata is sourced from `manifest.yaml` in this directory.",
        "",
        "## Stable rule IDs",
        "",
    ]
    lines.extend(
        [
            (
                f"- [`{entry.rule_id}`](./{entry.rule_id}.md) "
                f"(default severity: `{entry.default_severity}`)"
            )
            for entry in entries
        ],
    )

    lines.extend(
        [
            "",
            "## Severity vocabulary",
            "",
            "- `error`",
            "- `warning`",
            "- `info`",
            "",
            "## Rule policy",
            "",
            "- Rule IDs are stable kebab-case identifiers.",
            "- Severity defaults are part of the external contract.",
            (
                "- Rule selection and suppression use "
                "`--select`/`--ignore` and `[tool.tq]` values."
            ),
            (
                "- Rule additions and severity default changes follow "
                "[governance policy](../governance.md)."
            ),
            "",
            "## Manifest",
            "",
            "Canonical source of truth: [`manifest.yaml`](./manifest.yaml).",
            "",
        ],
    )

    return "\n".join(lines)


def _render_rule_page(*, entry: RuleManifestEntry) -> str:
    """Render one per-rule markdown page from a manifest entry."""
    lines = [
        f"# {entry.rule_id}",
        "",
        "## What it does",
        "",
        entry.what_it_does,
        "",
        "## Why this matters",
        "",
        entry.why_this_matters,
        "",
        "## Default severity",
        "",
        f"`{entry.default_severity}`",
        "",
        "## Trigger conditions",
        "",
    ]

    lines.extend(f"- {condition}" for condition in entry.trigger_conditions)

    lines.extend(
        [
            "",
            "## Examples",
            "",
        ],
    )
    for source, test in entry.examples:
        lines.append(f"- Source module: `{source}`")
        lines.append(f"- Test module: `{test}`")

    lines.extend(
        [
            "",
            "## How to address",
            "",
        ],
    )
    lines.extend(f"- {resolution}" for resolution in entry.how_to_address)

    lines.extend(
        [
            "",
            "## Related configuration and suppression controls",
            "",
        ],
    )
    lines.extend(f"- `{control}`" for control in entry.related_controls)

    lines.extend(
        [
            "",
            "## Added in",
            "",
            f"`{entry.added_in}`",
            "",
            "## Behavior changes",
            "",
            entry.behavior_changes,
            "",
        ],
    )

    return "\n".join(lines)


def generate_rules_docs() -> None:
    """Generate index and per-rule reference pages from canonical manifest."""
    rules_dir = Path("docs/reference/rules")
    manifest_path = rules_dir / "manifest.yaml"
    entries = _load_manifest(manifest_path=manifest_path)

    (rules_dir / "index.md").write_text(
        _render_index(entries=entries),
        encoding="utf-8",
    )

    for entry in entries:
        page_path = rules_dir / f"{entry.rule_id}.md"
        page_path.write_text(_render_rule_page(entry=entry), encoding="utf-8")


if __name__ == "__main__":
    generate_rules_docs()
