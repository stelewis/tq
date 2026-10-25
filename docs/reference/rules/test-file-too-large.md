# Test File Too Large

Rule ID: `test-file-too-large`

## What it does

Flag test files that exceed the configured non-blank, non-comment-only line budget.

## Why this matters

Oversized test modules tend to become monolithic and less actionable when failures occur.

## Default severity

`warning`

## Trigger conditions

- A test file is discovered.
- Non-blank, non-comment line count exceeds configured threshold.
- An accepted file is reported only when it also exceeds its accepted ceiling.

## Examples

- Source module: `n/a`
- Test module: `tests/app/engine/test_runner.py (over configured line limit)`

## How to address

- Split the suite by concern using stable qualifiers.
- Move shared setup into nearby `conftest.py` fixtures.
- Accept a cohesive suite with an exact path, rationale, and finite size ceiling.

## Related configuration and suppression controls

- `--select test-file-too-large`
- `--ignore test-file-too-large`
- `[tool.tq].select / [tool.tq].ignore`
- `[tool.tq].max_test_file_non_blank_lines`
- `[[tool.tq.targets.accepted_findings]]`

## Added in

`pre-1.0`

## Behavior changes

0.13.0: supports bounded per-file acceptances; growth findings include the accepted ceiling.
