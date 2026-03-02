"""Legacy compatibility shim for the ``check_test_quality`` command."""

from __future__ import annotations

import sys
from collections.abc import Sequence

import click

from tq.cli.main import cli

_DEPRECATION_MESSAGE = (
    "[deprecated] `check_test_quality` is a compatibility shim and will be "
    "removed in a future minor release; use `tq check` instead."
)


def main(argv: Sequence[str] | None = None) -> int:
    """Run legacy command as a forwarding shim to ``tq check``."""
    click.echo(_DEPRECATION_MESSAGE, err=True)

    forwarded_args = _build_forwarded_args(argv=argv)
    try:
        cli.main(
            args=forwarded_args,
            prog_name="check_test_quality",
            standalone_mode=False,
        )
    except click.exceptions.Exit as error:
        return int(error.exit_code)
    except click.ClickException as error:
        error.show()
        return int(error.exit_code)

    return 0


def _build_forwarded_args(*, argv: Sequence[str] | None) -> list[str]:
    """Prefix arguments with ``check`` unless already provided."""
    args = list(sys.argv[1:] if argv is None else argv)
    if args and args[0] == "check":
        return args
    return ["check", *args]


if __name__ == "__main__":
    sys.exit(main())
