# Installation

Install the published package `tqlint` and run it via the `tq` command.

## Installation modes

- Project dependency: `uv add --dev tqlint`
- Ephemeral run: `uvx tqlint check`
- Global tool: `uv tool install tqlint`
- `pip` install: `python -m pip install tqlint`

## Command naming

- Distribution package: `tqlint`
- CLI command: `tq` or `tqlint` (alias)
- Primary command shape: `tq check`

`uvx tq check` is not currently supported because the `tq` package name on PyPI is owned by another project.
