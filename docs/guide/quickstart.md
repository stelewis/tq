# QuickStart

Get `tq` running in a Python repository in a few steps.

## 1. Install

Choose an install mode:

- project dependency: `uv add --dev tqlint`
- ephemeral run: `uvx --from tqlint tq check`
- global tool: `uv tool install tqlint`
- pip install: `python -m pip install tqlint`

### Package and command naming

- Distribution package: `tqlint`
- CLI command: `tq`
- Primary command shape: `tq check`

Because `tq` is not the published PyPI package name, ephemeral runs must use `uvx --from tqlint tq ...`.

## 2. Add minimal configuration

Create or update `pyproject.toml`:

<!-- BEGIN GENERATED:quickstart-minimal-config -->

```toml
[tool.tq]
ignore_init_modules = true

[[tool.tq.targets]]
name = "app"
package = "your_package"
source_root = "src"
test_root = "tests"
```
<!-- END GENERATED:quickstart-minimal-config -->

## 3. Run a check

```sh
uv run tq check
```

For CI or editor integrations, emit JSON output:

```sh
uv run tq check --output-format json
```

## 4. Tune behavior

Use these common flags:

- `--select` / `--ignore` for rule selection
- `--max-test-file-non-blank-lines` for file size threshold
- `--qualifier-strategy` and `--allowed-qualifier` for qualifier policy

## Next steps

- Read [Configuration](../reference/configuration.md) for keys and precedence.
- Browse the [Rules Index](../reference/rules/index.md) for rule behavior.
- See [CLI Reference](../reference/cli.md) for command options.
