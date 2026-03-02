# Installation

Install the published package `tqlint` and run it via the `tq` command.

## Supported execution modes

- Project dependency: `uv add --dev tqlint`
- Ephemeral run: `uvx tqlint check`
- Global tool: `uv tool install tqlint`

## Command naming

- Distribution package: `tqlint`
- CLI command: `tq`
- Primary command shape: `tq check`

`uvx tq check` is currently not supported because `tq` on PyPI is owned by an
unrelated project.
