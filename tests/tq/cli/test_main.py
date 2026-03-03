"""Tests for tq check CLI behavior and contract semantics."""

from __future__ import annotations

import json
from pathlib import Path

from click.testing import CliRunner

from tq.cli.main import cli


def test_check_returns_zero_when_no_error_findings() -> None:
    """Exit with 0 when no error severity diagnostics are emitted."""
    runner = CliRunner()
    with runner.isolated_filesystem():
        _write_project_config(Path("pyproject.toml"))
        _write(Path("src/tq/engine/runner.py"), "def run() -> None:\n    pass\n")
        _write(
            Path("tests/tq/engine/test_runner.py"),
            "def test_runner() -> None:\n    assert True\n",
        )

        result = runner.invoke(cli, ["check", "--target", "tq"])

    assert result.exit_code == 0
    assert "All checks passed!" in result.output
    assert "Summary:" not in result.output


def test_root_help_supports_short_h_alias() -> None:
    """Expose -h alias at the CLI root."""
    runner = CliRunner()

    result = runner.invoke(cli, ["-h"])

    assert result.exit_code == 0
    assert "Usage:" in result.output


def test_check_help_supports_short_h_alias() -> None:
    """Expose -h alias on the check subcommand."""
    runner = CliRunner()

    result = runner.invoke(cli, ["check", "-h"])

    assert result.exit_code == 0
    assert "Usage:" in result.output


def test_check_returns_one_when_error_findings_exist() -> None:
    """Exit with 1 when at least one error finding exists."""
    runner = CliRunner()
    with runner.isolated_filesystem():
        _write_project_config(Path("pyproject.toml"))
        _write(Path("src/tq/engine/runner.py"), "def run() -> None:\n    pass\n")
        Path("tests").mkdir(parents=True, exist_ok=True)

        result = runner.invoke(cli, ["check", "--target", "tq"])

    assert result.exit_code == 1
    assert "mapping-missing-test" in result.output
    assert "suggestion:" not in result.output


def test_check_shows_suggestions_when_enabled() -> None:
    """Render suggestions only when explicitly requested."""
    runner = CliRunner()
    with runner.isolated_filesystem():
        _write_project_config(Path("pyproject.toml"))
        _write(Path("src/tq/engine/runner.py"), "def run() -> None:\n    pass\n")
        Path("tests").mkdir(parents=True, exist_ok=True)

        result = runner.invoke(cli, ["check", "--target", "tq", "--show-suggestions"])

    assert result.exit_code == 1
    assert "suggestion:" in result.output


def test_check_returns_two_for_invalid_config() -> None:
    """Exit with 2 when configuration is invalid."""
    runner = CliRunner()
    with runner.isolated_filesystem():
        Path("pyproject.toml").write_text(
            "[tool.tq]\nunknown = true",
            encoding="utf-8",
        )

        result = runner.invoke(cli, ["check"])

    assert result.exit_code == 2
    assert "Unknown [tool.tq] key" in result.output


def test_check_supports_json_output_when_clean() -> None:
    """Emit machine-readable JSON payload when output format is json."""
    runner = CliRunner()
    with runner.isolated_filesystem():
        _write_project_config(Path("pyproject.toml"))
        _write(Path("src/tq/engine/runner.py"), "def run() -> None:\n    pass\n")
        _write(
            Path("tests/tq/engine/test_runner.py"),
            "def test_runner() -> None:\n    assert True\n",
        )

        result = runner.invoke(
            cli,
            ["check", "--output-format", "json", "--target", "tq"],
        )

    assert result.exit_code == 0
    payload = json.loads(result.output)
    assert payload["findings"] == []
    assert payload["summary"] == {
        "errors": 0,
        "warnings": 0,
        "infos": 0,
        "total": 0,
    }


def test_check_supports_json_output_with_findings() -> None:
    """Emit findings payload fields using stable machine-readable names."""
    runner = CliRunner()
    with runner.isolated_filesystem():
        _write_project_config(Path("pyproject.toml"))
        _write(Path("src/tq/engine/runner.py"), "def run() -> None:\n    pass\n")
        Path("tests").mkdir(parents=True, exist_ok=True)

        result = runner.invoke(
            cli,
            ["check", "--output-format", "json", "--target", "tq"],
        )

    assert result.exit_code == 1
    payload = json.loads(result.output)
    assert payload["summary"]["errors"] == 1
    assert payload["summary"]["total"] == 1
    assert payload["findings"][0]["rule_id"] == "mapping-missing-test"
    assert payload["findings"][0]["severity"] == "error"
    assert payload["findings"][0]["path"] == "src/tq/engine/runner.py"
    assert payload["findings"][0]["target"] == "tq"


def test_cli_override_takes_precedence_over_config() -> None:
    """Honor CLI value over file-based configuration values."""
    runner = CliRunner()
    with runner.isolated_filesystem():
        Path("pyproject.toml").write_text(
            (
                "[tool.tq]\n"
                "[[tool.tq.targets]]\n"
                'name = "tq"\n'
                'package = "tq"\n'
                'source_root = "src"\n'
                'test_root = "tests"\n'
                "ignore_init_modules = false\n"
            ),
            encoding="utf-8",
        )
        _write(Path("src/tq/__init__.py"), '"""Package."""\n')
        Path("tests").mkdir(parents=True, exist_ok=True)

        default_result = runner.invoke(cli, ["check", "--target", "tq"])
        override_result = runner.invoke(
            cli,
            ["check", "--target", "tq", "--ignore-init-modules"],
        )

    assert default_result.exit_code == 1
    assert override_result.exit_code == 0


def test_unknown_target_fails_fast() -> None:
    """Fail fast when --target includes names not in configured targets."""
    runner = CliRunner()
    with runner.isolated_filesystem():
        _write_project_config(Path("pyproject.toml"))

        result = runner.invoke(cli, ["check", "--target", "does-not-exist"])

    assert result.exit_code == 2
    assert "Unknown target name(s)" in result.output


def test_target_filter_scopes_output() -> None:
    """Scope findings to one configured target using --target."""
    runner = CliRunner()
    with runner.isolated_filesystem():
        _write_project_config(Path("pyproject.toml"))
        _write(Path("scripts/docs/generate.py"), "def gen() -> None:\n    pass\n")
        Path("tests/scripts").mkdir(parents=True, exist_ok=True)

        result = runner.invoke(
            cli,
            ["check", "--output-format", "json", "--target", "scripts"],
        )

    assert result.exit_code == 1
    payload = json.loads(result.output)
    assert payload["summary"] == {
        "errors": 1,
        "warnings": 0,
        "infos": 0,
        "total": 1,
    }
    assert payload["findings"][0]["target"] == "scripts"
    assert payload["findings"][0]["path"] == "scripts/docs/generate.py"


def _write_project_config(path: Path) -> None:
    """Write a minimal valid project tq configuration."""
    path.write_text(
        (
            "[tool.tq]\n"
            "ignore_init_modules = true\n"
            "max_test_file_non_blank_lines = 600\n"
            'qualifier_strategy = "allowlist"\n'
            'allowed_qualifiers = ["regression"]\n\n'
            "[[tool.tq.targets]]\n"
            'name = "tq"\n'
            'package = "tq"\n'
            'source_root = "src"\n'
            'test_root = "tests"\n\n'
            "[[tool.tq.targets]]\n"
            'name = "scripts"\n'
            'package = "scripts"\n'
            'source_root = "."\n'
            'test_root = "tests"\n'
        ),
        encoding="utf-8",
    )


def _write(path: Path, content: str) -> None:
    """Create parent directories and write UTF-8 file content."""
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")
