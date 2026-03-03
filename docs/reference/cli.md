# CLI Reference

`tq` uses a subcommand-first CLI and a strict contract surface.

## Distribution and invocation

The published package name is `tqlint` and the installed command is `tq`.

Supported `uv` usage modes:

- project dependency: `uv add --dev tqlint` then `uv run tq check`
- ephemeral run: `uvx tqlint check`
- global tool install: `uv tool install tqlint` then `tq check`

`uvx tq check` is not currently supported because the `tq` package name on PyPI is owned by another project.

## Command model

Primary entrypoint:

- `tq check`

Design goals for this command surface:

- deterministic diagnostics,
- explicit rule selection controls,
- machine-readable reporting for CI tooling.

<!-- BEGIN GENERATED:check-options -->

## `tq check` options

The table below documents the command definitions.

| Flags | Config key | Default | Description |
| --- | --- | --- | --- |
| `--config` | — | `none` | Use this pyproject file instead of discovered configuration. |
| `--isolated` | — | `false` | Ignore discovered configuration files. |
| `--package` | [`package`](./configuration.md#package-required) | `none` | Target package import path. |
| `--source-root` | [`source_root`](./configuration.md#source_root-required) | `none` | Source tree root path. |
| `--test-root` | [`test_root`](./configuration.md#test_root-required) | `none` | Test tree root path. |
| `--max-test-file-non-blank-lines` | [`max_test_file_non_blank_lines`](./configuration.md#max_test_file_non_blank_lines-optional) | `none` | Maximum non-blank, non-comment lines per test file. |
| `--qualifier-strategy` | [`qualifier_strategy`](./configuration.md#qualifier_strategy-optional) | `none` | Module-name qualifier policy for qualified test files. |
| `--allowed-qualifier` | [`allowed_qualifiers`](./configuration.md#allowed_qualifiers-optional) | `[]` | Allowed qualifier suffix for allowlist strategy. |
| `--ignore-init-modules, --no-ignore-init-modules` | [`ignore_init_modules`](./configuration.md#ignore_init_modules-optional) | `none` | Ignore __init__.py modules in mapping checks. / Include __init__.py modules in mapping checks. |
| `--select` | [`select`](./configuration.md#select-optional) | `[]` | Only run selected rule IDs. |
| `--ignore` | [`ignore`](./configuration.md#ignore-optional) | `[]` | Skip listed rule IDs. |
| `--exit-zero` | — | `false` | Always exit with code 0 regardless of findings. |
| `--show-suggestions` | — | `false` | Render remediation suggestions in diagnostics output. |
| `--output-format` | — | `text` | Select output format. |

Run `tq check --help` for the runtime source of truth.

<!-- END GENERATED:check-options -->

## Output formats

`tq check` supports:

- `text` (default): concise terminal diagnostics.
- `json`: machine-readable diagnostics payload.

Use:

```sh
tq check --output-format json
```

JSON findings include stable fields:

- `rule_id`
- `severity`
- `message`
- `path`
- `line`
- `suggestion`

JSON summary includes:

- `errors`
- `warnings`
- `infos`
- `total`

## Language support

`tq` currently targets Python codebases.
