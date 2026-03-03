# QuickStart

This guide gets `tq` running in a Python repository.

## 1. Install

Choose one installation mode:

- project dependency: `uv add --dev tqlint`
- ephemeral run: `uvx tqlint check`
- global tool: `uv tool install tqlint`
- pip install: `python -m pip install tqlint`

### Command naming

- Distribution package: `tqlint`
- CLI command: `tq` (with `tqlint` alias)
- Primary command shape: `tq check`

`uvx tq check` is not currently supported because the `tq` package name on PyPI is owned by another project.

## 2. Add minimal configuration

Create or update `pyproject.toml`:

```toml
[tool.tq]
[[tool.tq.targets]]
name = "app"
package = "your_package"
source_root = "src"
test_root = "tests"
```

## 3. Run a check

```sh
uv run tq check
```

For CI or editor integrations, use JSON output:

```sh
uv run tq check --output-format json
```

## 4. Tune behavior

Common tuning flags:

- `--select` / `--ignore` for rule selection
- `--max-test-file-non-blank-lines` for file size threshold
- `--qualifier-strategy` and `--allowed-qualifier` for qualifier policy

## Next steps

- Read [Configuration](../reference/configuration.md) for all keys and precedence.
- Browse the [Rules Index](../reference/rules/index.md) for diagnostics behavior.
- See [CLI Reference](../reference/cli.md) for command options.
