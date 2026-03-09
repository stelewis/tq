# test-file-too-large

## What it does

Flag test files that exceed the configured non-blank, non-comment-only line budget.

## Why this matters

Oversized test modules tend to become monolithic and less actionable when failures occur.

## Default severity

`warning`

## Trigger conditions

- A test file is discovered.
- Non-blank, non-comment line count exceeds configured threshold.

## Examples

- Source module: `n/a`
- Test module: `tests/tq/engine/test_runner.py (over configured line limit)`

## How to address

- Split the suite by concern using stable qualifiers.
- Move shared setup into nearby `conftest.py` fixtures.

## Related configuration and suppression controls

- `--select test-file-too-large`
- `--ignore test-file-too-large`
- `[tool.tq].select / [tool.tq].ignore`
- `[tool.tq].max_test_file_non_blank_lines`

## Added in

`pre-1.0`

## Behavior changes

None to date.
