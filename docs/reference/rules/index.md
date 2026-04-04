# Rules

User-facing rules and default severities.

## Stable rule IDs

- [`Mapping Missing Test`](./mapping-missing-test.md) (`mapping-missing-test`; default severity: `error`)
- [`Structure Mismatch`](./structure-mismatch.md) (`structure-mismatch`; default severity: `warning`)
- [`Test File Too Large`](./test-file-too-large.md) (`test-file-too-large`; default severity: `warning`)
- [`Orphaned Test`](./orphaned-test.md) (`orphaned-test`; default severity: `warning`)

## Severity vocabulary

- `error`
- `warning`
- `info`

## Rule policy

- Rule IDs are stable kebab-case identifiers.
- Severity defaults are part of the external contract.
- Rule selection and suppression use `--select`/`--ignore` and `[tool.tq]` values.
- Rule additions and severity default changes follow [governance policy](../../developer/governance.md).
