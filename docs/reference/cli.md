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

## Configuration quick reference

| Option                          | Purpose                                           | Reference                                                                                   |
|---------------------------------|---------------------------------------------------|---------------------------------------------------------------------------------------------|
| `package`                       | Target import package path.                       | [package](./configuration.md#package-required)                                              |
| `source_root`                   | Root directory containing source packages.        | [source_root](./configuration.md#source_root-required)                                      |
| `test_root`                     | Root directory containing tests.                  | [test_root](./configuration.md#test_root-required)                                          |
| `ignore_init_modules`           | Include or skip `__init__.py` in mapping checks.  | [ignore_init_modules](./configuration.md#ignore_init_modules-optional)                      |
| `max_test_file_non_blank_lines` | Line budget for `test-file-too-large`.            | [max_test_file_non_blank_lines](./configuration.md#max_test_file_non_blank_lines-optional)  |
| `qualifier_strategy`            | Policy for qualified test name mapping.           | [qualifier_strategy](./configuration.md#qualifier_strategy-optional)                        |
| `allowed_qualifiers`            | Allowed suffixes when using `allowlist` strategy. | [allowed_qualifiers](./configuration.md#allowed_qualifiers-optional)                        |
| `select`                        | Explicit rule allow-list.                         | [select](./configuration.md#select-optional)                                                |
| `ignore`                        | Rule IDs to skip.                                 | [ignore](./configuration.md#ignore-optional)                                                |

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
