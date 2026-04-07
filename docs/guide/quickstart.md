# Quickstart

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

## 4. Use as a pre-commit hook

Optional: if you want to use `tq` as a pre-commit hook, add it to `.pre-commit-config.yaml`:

<!-- BEGIN GENERATED:pre-commit-config -->
```yaml
repos:
  - repo: https://github.com/stelewis/tq
    rev: 0.8.1
    hooks:
      - id: tq-check
```
<!-- END GENERATED:pre-commit-config -->

Then run:

```sh
pre-commit run tq-check --all-files
```

Use the latest release tag for `rev`, update with `pre-commit autoupdate`, and use `pre-commit autoupdate --freeze` if your team prefers full commit-SHA pinning.

`pre-commit` installs the hook in an isolated environment, but because `tq` is built from this repository's Rust-backed Python source distribution, the machine building the hook environment must have Python 3.11+ and a working Rust toolchain.

## 5. Tune behavior

Use these common flags:

- `--select` / `--ignore` for rule selection
- `--max-test-file-non-blank-lines` for file size threshold
- `--qualifier-strategy` and `--allowed-qualifier` for qualifier policy

## Next steps

- Read [Configuration](../reference/configuration.md) for keys and precedence.
- Browse the [Rules Index](../reference/rules/index.md) for rule behavior.
- See [CLI Reference](../reference/cli.md) for command options.
