# CLI Reference

Use `tq` through a subcommand-first CLI with a strict contract surface.

## Distribution and invocation

Published PyPI distribution: `tqlint`.

Installed command: `tq`.

For install and setup, see [QuickStart](../guide/quickstart.md).

## Command model

Primary command:

- `tq check`

This command surface is designed for:

- deterministic diagnostics
- explicit rule-selection controls
- machine-readable reporting for CI tooling

<!-- BEGIN GENERATED:check-options -->

## `tq check` options

The table below documents command options.

| Flags | Config key | Default | Description |
| --- | --- | --- | --- |
| `--config` | — | `none` | Use this pyproject file instead of discovered configuration. |
| `--isolated` | — | `false` | Ignore discovered configuration files. |
| `--target` | — | `[]` | Run only listed target names. |
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

- `text` (default): concise terminal diagnostics
- `json`: machine-readable diagnostics payload

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
- `target`

JSON summary includes:

- `errors`
- `warnings`
- `infos`
- `total`

## Language support

`tq` currently targets Python codebases.
