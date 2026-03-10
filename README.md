# `tq` - Test Quality Toolkit

[![Documentation](https://img.shields.io/badge/docs-site-blue)](https://stelewis.github.io/tq/)

`tq` inspects a codebase's tests and enforces quality rules so tests remain discoverable, focused, actionable, and maintainable.

The published `tqlint` package is Rust-backed and installs the `tq`
executable through Python package managers.

## Installation

PyPI distribution name: `tqlint`

Add to a project:

```sh
uv add --dev tqlint
uv run tq check
```

Run without installing (ephemeral):

```sh
uvx --from tqlint tq check
```

Install as a persistent global tool:

```sh
uv tool install tqlint
tq check
```

Install with `pip`:

```sh
python -m pip install tqlint
tq check
```

Note: `uvx tq check` is not available because the `tq` package name on PyPI is owned by another project. Use `uvx --from tqlint tq ...` for ephemeral runs.

## Usage

Run checks:

```sh
uv run tq check
```

## Configuration

Configure `tq` in `pyproject.toml` under `[tool.tq]`:

```toml
[tool.tq]
ignore_init_modules = true
max_test_file_non_blank_lines = 600
qualifier_strategy = "allowlist"
allowed_qualifiers = ["regression"]

[[tool.tq.targets]]
name = "tq"
package = "tq"
source_root = "src"
test_root = "tests"
```

## Documentation

[`tq` documentation](https://stelewis.github.io/tq/)

## Language support

`tq` currently only analyzes Python source and Python tests (`.py`).

## Development

Contributions are welcomed!

Contribution guidelines and development setup steps are documented in [CONTRIBUTING.md](CONTRIBUTING.md).
