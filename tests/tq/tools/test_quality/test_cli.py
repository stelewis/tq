"""Tests for legacy compatibility shim CLI module."""

from __future__ import annotations

from typing import cast

import click
import pytest

import tq.tools.test_quality.cli as legacy_cli


def test_main_emits_deprecation_warning(
    capsys: pytest.CaptureFixture[str],
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Print deprecation guidance on every shim invocation."""

    def fake_main(**_: object) -> None:
        raise click.exceptions.Exit(0)

    monkeypatch.setattr(legacy_cli.cli, "main", fake_main)

    exit_code = legacy_cli.main(argv=())
    captured = capsys.readouterr()

    assert exit_code == 0
    assert "[deprecated]" in captured.err
    assert "tq check" in captured.err


def test_main_prefixes_check_for_forwarding(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Forward legacy invocations as tq check subcommand calls."""
    observed_args: list[str] = []

    def fake_main(**kwargs: object) -> None:
        args = kwargs.get("args")
        if isinstance(args, list) and all(isinstance(item, str) for item in args):
            observed_args.extend(cast(list[str], args))
        raise click.exceptions.Exit(0)

    monkeypatch.setattr(legacy_cli.cli, "main", fake_main)

    exit_code = legacy_cli.main(argv=("--isolated", "--package", "tq"))

    assert exit_code == 0
    assert observed_args == ["check", "--isolated", "--package", "tq"]


def test_main_does_not_double_prefix_check(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Preserve explicit check prefix for backward-safe forwarding."""
    observed_args: list[str] = []

    def fake_main(**kwargs: object) -> None:
        args = kwargs.get("args")
        if isinstance(args, list) and all(isinstance(item, str) for item in args):
            observed_args.extend(cast(list[str], args))
        raise click.exceptions.Exit(0)

    monkeypatch.setattr(legacy_cli.cli, "main", fake_main)

    exit_code = legacy_cli.main(argv=("check", "--isolated"))

    assert exit_code == 0
    assert observed_args == ["check", "--isolated"]


def test_main_maps_click_exceptions_to_exit_code(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Return ClickException exit codes from forwarded execution."""

    def fake_main(**_: object) -> None:
        raise click.UsageError("invalid option")

    monkeypatch.setattr(legacy_cli.cli, "main", fake_main)

    assert legacy_cli.main(argv=("--bad-option",)) == 2
