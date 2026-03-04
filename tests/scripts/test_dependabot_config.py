"""Tests for Dependabot configuration coverage."""

from __future__ import annotations

from pathlib import Path

from scripts import dependabot_config


def test_dependabot_covers_all_local_github_actions() -> None:
    """Fail when a local composite action is not covered by Dependabot."""
    repo_root = Path(__file__).resolve().parents[2]
    config = dependabot_config.load_config(repo_root=repo_root)
    github_actions_updates = dependabot_config.github_actions_updates(config=config)
    configured_patterns = dependabot_config.collect_directory_patterns(
        updates=github_actions_updates
    )

    local_action_directories = dependabot_config.local_action_directories(
        repo_root=repo_root
    )
    missing_directories = dependabot_config.uncovered_directories(
        directories=local_action_directories,
        configured_patterns=configured_patterns,
    )

    assert not missing_directories, (
        "Dependabot github-actions config does not cover local action directories: "
        + ", ".join(sorted(missing_directories))
    )


def test_dependabot_covers_github_workflow_surface() -> None:
    """Require workflow and local-action directory coverage in one update block."""
    repo_root = Path(__file__).resolve().parents[2]
    config = dependabot_config.load_config(repo_root=repo_root)
    github_actions_updates = dependabot_config.github_actions_updates(config=config)
    assert len(github_actions_updates) == 1

    configured_patterns = dependabot_config.collect_directory_patterns(
        updates=github_actions_updates
    )

    workflow_files = dependabot_config.local_workflow_files(repo_root=repo_root)
    if workflow_files:
        assert "/" in configured_patterns

    assert "/.github/actions/*" in configured_patterns
