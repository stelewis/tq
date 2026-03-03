# `tq` - Test Quality Toolkit

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
uvx tqlint check
```

Install as a persistent global tool:

```sh
uv tool install tqlint
tq check
```

Note: `uvx tq check` is not available because the `tq` package name on PyPI is owned by another project.

## Current scope

`tq` currently analyzes Python source and Python tests (`.py`) only.

## Usage

Run checks:

```sh
uv run tq check
```

Emit machine-readable diagnostics:

```sh
uv run tq check --output-format json
```

## Configuration

Configure `tq` in `pyproject.toml` under `[tool.tq]`:

```toml
[tool.tq]
package = "tq"
source_root = "src"
test_root = "tests"
ignore_init_modules = true
max_test_file_non_blank_lines = 600
qualifier_strategy = "allowlist"
allowed_qualifiers = ["regression"]
```

## Documentation

- Overview: [https://stelewis.github.io/tq/](https://stelewis.github.io/tq/)
- Getting started: [https://stelewis.github.io/tq/guide/getting-started](https://stelewis.github.io/tq/guide/getting-started)
- CLI reference: [https://stelewis.github.io/tq/reference/cli](https://stelewis.github.io/tq/reference/cli)
- Configuration reference: [https://stelewis.github.io/tq/reference/configuration](https://stelewis.github.io/tq/reference/configuration)
- Exit codes: [https://stelewis.github.io/tq/reference/exit-codes](https://stelewis.github.io/tq/reference/exit-codes)
- Rules reference: [https://stelewis.github.io/tq/reference/rules/](https://stelewis.github.io/tq/reference/rules/)

## Development

Contribution guidelines are in [CONTRIBUTING.md](CONTRIBUTING.md). For local development, follow below steps.

Install dependencies:

```sh
uv sync
```

Install pre-commit hooks (including `pre-commit`, `commit-msg`, and `pre-push`):

```sh
uv run prek install
```

Run tests:

```sh
uv run pytest -q
```
