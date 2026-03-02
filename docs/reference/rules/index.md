# Rules

This is the canonical user-facing rules index.

## Stable rule IDs

- `mapping-missing-test` (default severity: `error`)
- `structure-mismatch` (default severity: `warning`)
- `test-file-too-large` (default severity: `warning`)
- `orphaned-test` (default severity: `warning`)

## Severity vocabulary

- `error`
- `warning`
- `info`

## Rule behavior

- `mapping-missing-test`: a discovered source module has no matching unit test module.
- `structure-mismatch`: a unit test path does not mirror the corresponding source path.
- `test-file-too-large`: a unit test file exceeds the configured non-blank, non-comment line limit.
- `orphaned-test`: a unit test file does not resolve to an existing source module.

Per-rule pages will be added under this section and kept in sync from a canonical
rules manifest.
