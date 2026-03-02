# Rules

This is the canonical user-facing rules index.

Rule metadata is sourced from `manifest.yaml` in this directory.

## Stable rule IDs

- [`mapping-missing-test`](./mapping-missing-test.md) (default severity: `error`)
- [`structure-mismatch`](./structure-mismatch.md) (default severity: `warning`)
- [`test-file-too-large`](./test-file-too-large.md) (default severity: `warning`)
- [`orphaned-test`](./orphaned-test.md) (default severity: `warning`)

## Severity vocabulary

- `error`
- `warning`
- `info`

## Rule policy

- Rule IDs are stable kebab-case identifiers.
- Severity defaults are part of the external contract.
- Rule selection and suppression use `--select`/`--ignore` and `[tool.tq]` values.

## Manifest

Canonical source of truth: [`manifest.yaml`](./manifest.yaml).
