"""Skill packager.

Create a distributable `.skill` archive from a skill directory.

Usage:
    package_skill.py <path/to/skill-folder> [output-directory]
"""

from __future__ import annotations

import argparse
import logging
import zipfile
from pathlib import Path

from quick_validate import validate_skill

LOGGER = logging.getLogger(__name__)
EXCLUDED_NAMES = {".DS_Store"}
EXCLUDED_DIR_NAMES = {".git", "__pycache__"}
EXCLUDED_SUFFIXES = {".pyc", ".pyo"}


def _is_excluded(file_path: Path) -> bool:
    """Return whether the path should be excluded from the package."""
    if file_path.name in EXCLUDED_NAMES:
        return True
    if file_path.suffix in EXCLUDED_SUFFIXES:
        return True
    return any(part in EXCLUDED_DIR_NAMES for part in file_path.parts)


def _validate_skill_folder(skill_path: Path) -> tuple[bool, str]:
    """Validate the skill folder path and required files."""
    if not skill_path.exists():
        return False, f"Skill folder not found: {skill_path}"
    if not skill_path.is_dir():
        return False, f"Path is not a directory: {skill_path}"
    if not (skill_path / "SKILL.md").exists():
        return False, f"SKILL.md not found in {skill_path}"
    return True, ""


def _resolve_output_file(skill_path: Path, output_dir: str | None) -> Path:
    """Resolve and create the output directory for the .skill archive."""
    output_path = Path(output_dir).resolve() if output_dir else Path.cwd()
    output_path.mkdir(parents=True, exist_ok=True)
    return output_path / f"{skill_path.name}.skill"


def _collect_files(skill_path: Path) -> tuple[list[Path], list[str]]:
    """Collect packable files and informational messages."""
    files: list[Path] = []
    notes: list[str] = []

    for file_path in skill_path.rglob("*"):
        if _is_excluded(file_path):
            continue
        if file_path.is_symlink():
            notes.append(f"Skipped symlink: {file_path.relative_to(skill_path)}")
            continue
        if not file_path.is_file():
            continue

        resolved_file = file_path.resolve()
        if skill_path not in resolved_file.parents:
            notes.append(f"Skipped external file: {file_path.relative_to(skill_path)}")
            continue
        files.append(file_path)

    return files, notes


def package_skill(
    skill_path: str, output_dir: str | None = None
) -> tuple[Path | None, str]:
    """Package a skill folder into a `.skill` archive."""
    resolved_skill_path = Path(skill_path).resolve()

    is_valid, message = _validate_skill_folder(resolved_skill_path)
    if not is_valid:
        return None, message

    valid_skill, validation_message = validate_skill(resolved_skill_path)
    if not valid_skill:
        return None, f"Validation failed: {validation_message}"

    files, notes = _collect_files(resolved_skill_path)
    output_file = _resolve_output_file(resolved_skill_path, output_dir)

    try:
        with zipfile.ZipFile(output_file, "w", zipfile.ZIP_DEFLATED) as archive:
            for file_path in files:
                arcname = file_path.relative_to(resolved_skill_path.parent)
                archive.write(file_path, arcname)
    except OSError as error:
        return None, f"Error creating .skill file: {error}"

    if notes:
        for note in notes:
            LOGGER.info(note)

    return output_file, "Skill packaged successfully."


def _build_parser() -> argparse.ArgumentParser:
    """Build the CLI parser."""
    parser = argparse.ArgumentParser(
        description="Package a skill into a .skill archive."
    )
    parser.add_argument("skill_path", help="Path to the skill folder.")
    parser.add_argument(
        "output_dir",
        nargs="?",
        default=None,
        help="Optional output directory for the generated archive.",
    )
    return parser


def main() -> int:
    """Run the CLI entrypoint."""
    logging.basicConfig(level=logging.INFO, format="%(message)s")
    args = _build_parser().parse_args()

    LOGGER.info("Packaging skill: %s", args.skill_path)
    if args.output_dir:
        LOGGER.info("Output directory: %s", args.output_dir)

    result, message = package_skill(args.skill_path, args.output_dir)
    if result is None:
        LOGGER.error(message)
        return 1

    LOGGER.info(message)
    LOGGER.info("Created archive: %s", result)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
