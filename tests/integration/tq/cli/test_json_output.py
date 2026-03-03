"""Integration coverage for JSON output stability."""

from __future__ import annotations

import json
from pathlib import Path

import pytest
from click.testing import CliRunner

from tq.cli.main import cli


@pytest.mark.integration
def test_json_output_is_deterministic_for_representative_project() -> None:
    """Emit stable JSON findings order and payload across repeated runs."""
    runner = CliRunner()
    with runner.isolated_filesystem():
        _write_project_config(Path("pyproject.toml"))
        _write(Path("src/tq/alpha.py"), "def alpha() -> None:\n    pass\n")
        _write(Path("src/tq/beta.py"), "def beta() -> None:\n    pass\n")
        _write(
            Path("tests/tq/test_alpha.py"),
            "def test_alpha() -> None:\n    assert True\n",
        )
        _write(
            Path("tests/tq/test_gamma.py"),
            "def test_gamma() -> None:\n    assert True\n",
        )

        first = runner.invoke(cli, ["check", "--output-format", "json"])
        second = runner.invoke(cli, ["check", "--output-format", "json"])

    assert first.exit_code == 1
    assert second.exit_code == 1
    assert first.output == second.output

    payload = json.loads(first.output)
    assert payload == {
        "findings": [
            {
                "rule_id": "mapping-missing-test",
                "severity": "error",
                "message": "No test file found for source module: beta.py",
                "path": "src/tq/beta.py",
                "line": None,
                "suggestion": "Create test file at: tq/test_beta.py",
                "target": "tq",
            },
            {
                "rule_id": "orphaned-test",
                "severity": "warning",
                "message": (
                    "Test file has no corresponding source module: tq/test_gamma.py"
                ),
                "path": "tests/tq/test_gamma.py",
                "line": None,
                "suggestion": (
                    "Verify this test is still needed or move it "
                    "to integration/e2e scope"
                ),
                "target": "tq",
            },
        ],
        "summary": {
            "errors": 1,
            "warnings": 1,
            "infos": 0,
            "total": 2,
        },
    }


def _write_project_config(path: Path) -> None:
    """Write a minimal valid project tq configuration."""
    path.write_text(
        (
            "[tool.tq]\n"
            "ignore_init_modules = true\n"
            "max_test_file_non_blank_lines = 600\n"
            'qualifier_strategy = "any-suffix"\n\n'
            "[[tool.tq.targets]]\n"
            'name = "tq"\n'
            'package = "tq"\n'
            'source_root = "src"\n'
            'test_root = "tests"\n'
        ),
        encoding="utf-8",
    )


def _write(path: Path, content: str) -> None:
    """Create parent directories and write UTF-8 file content."""
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")
