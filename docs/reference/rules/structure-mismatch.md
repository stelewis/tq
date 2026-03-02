# structure-mismatch

## What it does

Detects unit test files that do not mirror the expected source-relative path layout.

## Why this matters

Structure drift makes tests harder to find, weakens navigability, and increases refactor friction.

## Default severity

`warning`

## Trigger conditions

- A unit test file is discovered.
- The file resolves to a source target but lives in a different path.
- Integration and e2e paths are excluded from this rule.

## Examples

- Source module: `src/tq/engine/runner.py`
- Test module: `tests/tq/test_runner.py`

## How to address

- Move the unit test to mirror source structure.
- Keep the filename aligned with the targeted module.

## Related configuration and suppression controls

- `--select structure-mismatch`
- `--ignore structure-mismatch`
- `[tool.tq].select / [tool.tq].ignore`

## Added in

`pre-1.0`

## Behavior changes

None to date.
