<!-- markdownlint-disable MD033 MD041 -->
<p align="center">
  <img src="docs/public/tq-logo-large.svg" alt="tq logo" width="144">
</p>
<!-- markdownlint-enable MD033 MD041 -->

# `tq` - Test Quality Toolkit

[![Documentation](https://img.shields.io/badge/docs-site-blue)](https://stelewis.github.io/tq/)

`tq` inspects a codebase's tests and enforces quality rules so tests remain discoverable, focused, actionable, and maintainable.

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

Note: `uvx tq check` is not currently available because the `tq` package name on PyPI is owned by another project. Use `uvx --from tqlint tq ...` for ephemeral runs.

## Usage

Run checks:

```sh
uv run tq check
```

## Configuration

Configure `tq` in `pyproject.toml` under `[tool.tq]`:

<!-- BEGIN GENERATED:readme-configuration-example -->
```toml
[tool.tq]
init_modules = "ignore"
max_test_file_non_blank_lines = 600
qualifier_strategy = "allowlist"
allowed_qualifiers = ["regression"]

[[tool.tq.targets]]
name = "tq"
package = "tq"
source_root = "src"
test_root = "tests"
```
<!-- END GENERATED:readme-configuration-example -->

## Documentation

[`tq` documentation](https://stelewis.github.io/tq/)

## Language support

`tq` currently only analyzes Python source and Python tests (`.py`).

## Development

Contributions are welcomed!

Contribution guidelines and development setup steps are documented in [CONTRIBUTING.md](CONTRIBUTING.md).
