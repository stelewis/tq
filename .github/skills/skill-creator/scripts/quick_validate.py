"""Quick validation script for skills."""

from __future__ import annotations

import argparse
import logging
import re
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Callable

import yaml

LOGGER = logging.getLogger(__name__)
MAX_SKILL_MD_SIZE = 1_000_000
MAX_SKILL_NAME_LENGTH = 64
MAX_DESCRIPTION_LENGTH = 1024
MAX_COMPATIBILITY_LENGTH = 500
ALLOWED_PROPERTIES = {
    "name",
    "description",
    "license",
    "metadata",
    "compatibility",
    "allowed-tools",
    "argument-hint",
    "user-invokable",
    "disable-model-invocation",
}
RESERVED_NAME_TERMS = {"anthropic", "claude"}


def _extract_frontmatter(content: str) -> str | None:
    """Extract YAML frontmatter from SKILL.md content."""
    match = re.match(r"^---\r?\n(.*?)\r?\n---", content, re.DOTALL)
    if not match:
        return None
    return match.group(1)


def _parse_frontmatter(frontmatter_text: str) -> dict[str, object]:
    """Parse frontmatter with PyYAML."""
    parsed = yaml.safe_load(frontmatter_text)
    if parsed is None:
        return {}
    if not isinstance(parsed, dict):
        msg = "Frontmatter must be a YAML dictionary"
        raise TypeError(msg)
    return parsed


def _is_valid_skill_name(name: str) -> bool:
    """Validate skill name against kebab-case constraints."""
    if not name or len(name) > MAX_SKILL_NAME_LENGTH:
        return False
    if name.startswith("-") or name.endswith("-") or "--" in name:
        return False
    for char in name:
        if char == "-":
            continue
        if not char.isalnum():
            return False
        if char.isalpha() and char != char.lower():
            return False
    return True


def _validate_skill_directory(skill_path: Path) -> tuple[bool, str]:
    """Validate skill directory existence and required files."""
    if not skill_path.exists() or not skill_path.is_dir():
        return False, f"Skill directory not found: {skill_path}"
    if not (skill_path / "SKILL.md").exists():
        return False, "SKILL.md not found"
    return True, ""


def _read_skill_markdown(skill_path: Path) -> tuple[str | None, str]:
    """Read and validate SKILL.md size and frontmatter marker."""
    skill_md = skill_path / "SKILL.md"
    file_size = skill_md.stat().st_size
    if file_size > MAX_SKILL_MD_SIZE:
        return (
            None,
            (
                "SKILL.md is too large "
                f"({file_size} bytes). Maximum is {MAX_SKILL_MD_SIZE} bytes."
            ),
        )

    content = skill_md.read_text(encoding="utf-8")
    if not content.startswith("---"):
        return None, "No YAML frontmatter found"
    return content, ""


def _validate_allowed_keys(frontmatter: dict[str, object]) -> tuple[bool, str]:
    """Validate top-level frontmatter keys."""
    unexpected_keys = set(frontmatter.keys()) - ALLOWED_PROPERTIES
    if not unexpected_keys:
        return True, ""

    found_keys = ", ".join(sorted(unexpected_keys))
    allowed_keys = ", ".join(sorted(ALLOWED_PROPERTIES))
    return False, (
        "Unexpected key(s) in SKILL.md frontmatter: "
        f"{found_keys}. Allowed properties are: {allowed_keys}"
    )


def _validate_required_fields(frontmatter: dict[str, object]) -> tuple[bool, str]:
    """Validate required frontmatter fields."""
    if "name" not in frontmatter:
        return False, "Missing 'name' in frontmatter"
    if "description" not in frontmatter:
        return False, "Missing 'description' in frontmatter"
    return True, ""


def _validate_name(name: object, skill_path: Path) -> tuple[bool, str]:
    """Validate the `name` field."""
    if not isinstance(name, str):
        return False, f"Name must be a string, got {type(name).__name__}"

    stripped_name = name.strip()
    if not stripped_name:
        return False, "Name cannot be empty"
    if not _is_valid_skill_name(stripped_name):
        return False, (
            f"Name '{stripped_name}' is invalid. "
            "It must be 1-64 chars, lowercase alphanumeric, and hyphen-safe."
        )

    lowered_name = stripped_name.lower()
    if any(term in lowered_name for term in RESERVED_NAME_TERMS):
        return False, "Name cannot contain reserved words: anthropic, claude"
    if "<" in stripped_name or ">" in stripped_name:
        return False, "Name cannot contain angle brackets (< or >)"
    if skill_path.name != stripped_name:
        return (
            False,
            (
                f"Skill directory name '{skill_path.name}' "
                f"must match frontmatter name '{stripped_name}'"
            ),
        )

    return True, ""


def _validate_description(description: object) -> tuple[bool, str]:
    """Validate the `description` field."""
    if not isinstance(description, str):
        return False, f"Description must be a string, got {type(description).__name__}"

    stripped_description = description.strip()
    if not stripped_description:
        return False, "Description cannot be empty"
    if "<" in stripped_description or ">" in stripped_description:
        return False, "Description cannot contain angle brackets (< or >)"
    if len(stripped_description) > MAX_DESCRIPTION_LENGTH:
        return (
            False,
            (
                "Description is too long "
                f"({len(stripped_description)} characters). "
                f"Maximum is {MAX_DESCRIPTION_LENGTH}."
            ),
        )
    return True, ""


def _validate_license(frontmatter: dict[str, object]) -> tuple[bool, str]:
    """Validate optional `license` field."""
    license_field = frontmatter.get("license")
    if license_field is not None:
        if not isinstance(license_field, str):
            return (
                False,
                f"license must be a string, got {type(license_field).__name__}",
            )
        if not license_field.strip():
            return False, "license cannot be empty when provided"
    return True, ""


def _validate_metadata(frontmatter: dict[str, object]) -> tuple[bool, str]:
    """Validate optional `metadata` field."""
    metadata = frontmatter.get("metadata")
    if metadata is None:
        return True, ""

    if not isinstance(metadata, dict):
        return False, f"metadata must be a mapping, got {type(metadata).__name__}"
    for key, value in metadata.items():
        if not isinstance(key, str):
            return False, "metadata keys must be strings"
        if not isinstance(value, str):
            return False, f"metadata['{key}'] must be a string"
    return True, ""


def _validate_string_option(
    frontmatter: dict[str, object],
    field_name: str,
) -> tuple[bool, str]:
    """Validate optional string frontmatter field."""
    value = frontmatter.get(field_name)
    if value is not None and not isinstance(value, str):
        return False, f"{field_name} must be a string, got {type(value).__name__}"
    return True, ""


def _validate_compatibility(frontmatter: dict[str, object]) -> tuple[bool, str]:
    """Validate optional `compatibility` field."""
    compatibility = frontmatter.get("compatibility")
    if compatibility is None:
        return True, ""

    if not isinstance(compatibility, str):
        return (
            False,
            f"compatibility must be a string, got {type(compatibility).__name__}",
        )
    if len(compatibility) > MAX_COMPATIBILITY_LENGTH:
        return (
            False,
            (
                "Compatibility is too long "
                f"({len(compatibility)} characters). "
                f"Maximum is {MAX_COMPATIBILITY_LENGTH}."
            ),
        )
    return True, ""


def _validate_bool_option(
    frontmatter: dict[str, object],
    field_name: str,
) -> tuple[bool, str]:
    """Validate optional boolean frontmatter field."""
    value = frontmatter.get(field_name)
    if value is not None and not isinstance(value, bool):
        return False, f"{field_name} must be a boolean, got {type(value).__name__}"
    return True, ""


def _validate_optional_fields(frontmatter: dict[str, object]) -> tuple[bool, str]:
    """Validate optional frontmatter fields."""
    validators: tuple[Callable[[dict[str, object]], tuple[bool, str]], ...] = (
        _validate_license,
        _validate_metadata,
        lambda data: _validate_string_option(data, "allowed-tools"),
        _validate_compatibility,
        lambda data: _validate_bool_option(data, "user-invokable"),
        lambda data: _validate_bool_option(data, "disable-model-invocation"),
    )
    for validator in validators:
        ok, message = validator(frontmatter)
        if not ok:
            return False, message
    return True, ""


def validate_skill(skill_path: str | Path) -> tuple[bool, str]:
    """Run basic structural validation for a skill directory."""
    resolved_skill_path = Path(skill_path).resolve()
    ok, message = _validate_skill_directory(resolved_skill_path)
    if not ok:
        return False, message

    content, message = _read_skill_markdown(resolved_skill_path)
    if content is None:
        return False, message

    frontmatter_text = _extract_frontmatter(content)
    if frontmatter_text is None:
        return False, "Invalid frontmatter format"

    try:
        frontmatter = _parse_frontmatter(frontmatter_text)
    except TypeError as error:
        return False, str(error)

    for validator in (_validate_allowed_keys, _validate_required_fields):
        ok, message = validator(frontmatter)
        if not ok:
            return False, message

    ok, message = _validate_name(frontmatter.get("name"), resolved_skill_path)
    if not ok:
        return False, message

    ok, message = _validate_description(frontmatter.get("description"))
    if not ok:
        return False, message

    ok, message = _validate_optional_fields(frontmatter)
    if not ok:
        return False, message

    return True, "Skill is valid!"


def _build_parser() -> argparse.ArgumentParser:
    """Build parser for CLI usage."""
    parser = argparse.ArgumentParser(description="Validate a skill directory quickly.")
    parser.add_argument("skill_directory", help="Path to a skill directory.")
    return parser


def main() -> int:
    """Run the CLI entrypoint."""
    logging.basicConfig(level=logging.INFO, format="%(message)s")
    args = _build_parser().parse_args()

    valid, message = validate_skill(args.skill_directory)
    if valid:
        LOGGER.info(message)
        return 0

    LOGGER.error(message)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
