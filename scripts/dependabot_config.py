"""Helpers for validating Dependabot configuration coverage.

GitHub Dependabot docs:
<https://docs.github.com/en/code-security/how-tos/secure-your-supply-chain/secure-your-dependencies/configuring-dependabot-version-updates>

Docs explicitly state that workflow files stored in the default location of
`.github/workflows`. You don't need to specify `/.github/workflows` for
`directory`. You can use `directory: "/"`.
"""

from __future__ import annotations

from fnmatch import fnmatch
from typing import TYPE_CHECKING, Any

import yaml

if TYPE_CHECKING:
    from pathlib import Path


def load_config(*, repo_root: Path) -> dict[str, Any]:
    """Load repository Dependabot configuration from disk.

    Args:
        repo_root: Repository root directory.

    Returns:
        Parsed YAML mapping from .github/dependabot.yml.
    """
    dependabot_path = repo_root / ".github" / "dependabot.yml"
    loaded = yaml.safe_load(dependabot_path.read_text(encoding="utf-8"))
    if not isinstance(loaded, dict):
        msg = "Expected mapping in .github/dependabot.yml"
        raise TypeError(msg)
    return loaded


def github_actions_updates(*, config: dict[str, Any]) -> list[dict[str, Any]]:
    """Return github-actions update blocks from Dependabot config.

    Args:
        config: Parsed Dependabot configuration.

    Returns:
        All updates blocks configured for package-ecosystem github-actions.
    """
    updates = config.get("updates")
    if not isinstance(updates, list):
        msg = "Expected 'updates' to be a list in .github/dependabot.yml"
        raise TypeError(msg)

    return [
        update
        for update in updates
        if isinstance(update, dict)
        and update.get("package-ecosystem") == "github-actions"
    ]


def collect_directory_patterns(
    *,
    updates: list[dict[str, Any]],
) -> list[str]:
    """Collect directory and directories patterns for update blocks.

    Args:
        updates: Dependabot update entries.

    Returns:
        Directory patterns from both `directory` and `directories` keys.
    """
    configured_patterns: list[str] = []
    for update in updates:
        directory_value = update.get("directory")
        if isinstance(directory_value, str):
            configured_patterns.append(directory_value)

        directories_value = update.get("directories")
        if isinstance(directories_value, list):
            configured_patterns.extend(
                value for value in directories_value if isinstance(value, str)
            )

    return configured_patterns


def local_action_directories(*, repo_root: Path) -> list[str]:
    """Return local composite action directories in repository-root form.

    Args:
        repo_root: Repository root directory.

    Returns:
        Sorted unique paths that contain action metadata files.
    """
    action_directories = {
        "/" + action_file.parent.relative_to(repo_root).as_posix().rstrip("/")
        for action_file in (repo_root / ".github" / "actions").rglob("action.y*ml")
    }
    return sorted(action_directories)


def local_workflow_files(*, repo_root: Path) -> list[Path]:
    """Return workflow YAML files under .github/workflows.

    Args:
        repo_root: Repository root directory.

    Returns:
        Sorted workflow file paths.
    """
    return sorted((repo_root / ".github" / "workflows").glob("*.y*ml"))


def uncovered_directories(
    *,
    directories: list[str],
    configured_patterns: list[str],
) -> list[str]:
    """Return directories not matched by configured Dependabot patterns.

    Args:
        directories: Directory paths to evaluate.
        configured_patterns: Dependabot directory patterns.

    Returns:
        Sorted unmatched directories.
    """
    return sorted(
        directory
        for directory in directories
        if not any(fnmatch(directory, pattern) for pattern in configured_patterns)
    )
