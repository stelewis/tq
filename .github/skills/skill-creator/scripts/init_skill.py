#!/usr/bin/env python3
"""Skill initializer.

Create a new skill scaffold from templates.

Usage:
    init_skill.py <skill-name> --path <path>
"""

from __future__ import annotations

import argparse
import logging
from pathlib import Path

LOGGER = logging.getLogger(__name__)
RESERVED_NAME_TERMS = {"anthropic", "claude"}
MAX_SKILL_NAME_LENGTH = 64


SKILL_TEMPLATE = """---
name: {skill_name}
description: [TODO: Add a clear description of what this skill does and when to use it.]
---

# {skill_title}

## Overview

[TODO: Explain what this skill enables in 1-2 sentences.]

## Structure Guidance

[TODO: Pick the structure that best matches this skill:]

1. Workflow-based: Use for sequential procedures.
2. Task-based: Use for grouped operations or utilities.
3. Reference-based: Use for standards or detailed guidance.
4. Capabilities-based: Use for integrated feature sets.

Skills can combine patterns (e.g., start with task-based, then add workflow
for complex operations).

[TODO: Delete this section after replacing with real content.]

## Resources

The generated scaffold includes optional resource directories:

- scripts/: executable helpers.
- references/: docs loaded as context.
- assets/: files used in generated output.

Delete any unneeded directories.
"""


EXAMPLE_SCRIPT = '''#!/usr/bin/env python3
"""Example helper script for {skill_name}."""


def main() -> None:
    """Run the example script."""
    return None


if __name__ == "__main__":
    main()
'''


EXAMPLE_REFERENCE = """# Reference Documentation for {skill_title}

This file is a placeholder for detailed supporting documentation.

Use it for:
- API details.
- Step-by-step workflows.
- Deep technical references.
"""


EXAMPLE_ASSET = """# Example Asset Placeholder

Replace this file with real assets, or delete the directory.

Common examples:
- templates
- images
- fonts
- data files
"""


def title_case_skill_name(skill_name: str) -> str:
    """Convert a hyphenated skill name into title case."""
    return " ".join(word.capitalize() for word in skill_name.split("-"))


def _get_invalid_skill_char_message(skill_name: str) -> str | None:
    """Return validation message for invalid characters, or None when valid."""
    for char in skill_name:
        if char == "-":
            continue
        if not char.isalnum() or (char.isalpha() and char != char.lower()):
            return (
                "Skill name must use lowercase alphanumeric characters "
                "and hyphens only."
            )
    return None


def validate_skill_name(skill_name: str) -> tuple[bool, str]:
    """Validate a skill name against kebab-case requirements."""
    if not skill_name:
        return False, "Skill name must be a non-empty string."
    if len(skill_name) > MAX_SKILL_NAME_LENGTH:
        return False, f"Skill name must be {MAX_SKILL_NAME_LENGTH} characters or fewer."

    if skill_name.startswith("-") or skill_name.endswith("-") or "--" in skill_name:
        return False, (
            "Skill name cannot start/end with hyphen or contain consecutive hyphens."
        )

    lowered_name = skill_name.lower()
    if any(term in lowered_name for term in RESERVED_NAME_TERMS):
        return False, "Skill name cannot contain reserved words."

    invalid_char_message = _get_invalid_skill_char_message(skill_name)
    if invalid_char_message is not None:
        return False, invalid_char_message

    return True, ""


def _write_file(path: Path, content: str, executable: bool = False) -> None:
    """Write content to a file and optionally mark it executable."""
    path.write_text(content, encoding="utf-8")
    if executable:
        path.chmod(0o755)


def _create_resource_files(skill_dir: Path, skill_name: str, skill_title: str) -> None:
    """Create default scripts, references, and assets directories."""
    scripts_dir = skill_dir / "scripts"
    scripts_dir.mkdir(exist_ok=True)
    _write_file(
        scripts_dir / "example.py",
        EXAMPLE_SCRIPT.format(skill_name=skill_name),
        executable=True,
    )

    references_dir = skill_dir / "references"
    references_dir.mkdir(exist_ok=True)
    _write_file(
        references_dir / "api_reference.md",
        EXAMPLE_REFERENCE.format(skill_title=skill_title),
    )

    assets_dir = skill_dir / "assets"
    assets_dir.mkdir(exist_ok=True)
    _write_file(assets_dir / "example_asset.txt", EXAMPLE_ASSET)


def init_skill(skill_name: str, path: str) -> tuple[Path | None, str]:
    """Initialize a new skill directory with starter files."""
    valid_name, validation_message = validate_skill_name(skill_name)
    if not valid_name:
        return None, validation_message

    skill_dir = Path(path).resolve() / skill_name
    if skill_dir.exists():
        return None, f"Skill directory already exists: {skill_dir}"

    try:
        skill_dir.mkdir(parents=True, exist_ok=False)
        skill_title = title_case_skill_name(skill_name)
        skill_content = SKILL_TEMPLATE.format(
            skill_name=skill_name,
            skill_title=skill_title,
        )
        _write_file(skill_dir / "SKILL.md", skill_content)
        _create_resource_files(skill_dir, skill_name, skill_title)
    except OSError as error:
        return None, f"Failed to initialize skill: {error}"

    return skill_dir, "Skill initialized successfully."


def _build_parser() -> argparse.ArgumentParser:
    """Create and return the CLI parser."""
    parser = argparse.ArgumentParser(description="Initialize a new skill scaffold.")
    parser.add_argument("skill_name", help="Kebab-case skill identifier.")
    parser.add_argument(
        "--path", required=True, help="Directory where skill is created."
    )
    return parser


def main() -> int:
    """Run the CLI entrypoint."""
    logging.basicConfig(level=logging.INFO, format="%(message)s")
    args = _build_parser().parse_args()

    skill_dir, message = init_skill(args.skill_name, args.path)
    if skill_dir is None:
        LOGGER.error("Error: %s", message)
        return 1

    LOGGER.info("Created skill at %s", skill_dir)
    LOGGER.info(message)
    LOGGER.info("Next steps:")
    LOGGER.info("1. Edit SKILL.md and complete TODO sections.")
    LOGGER.info("2. Customize or remove files in scripts/, references/, assets/.")
    LOGGER.info("3. Run quick_validate.py against the skill directory.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
