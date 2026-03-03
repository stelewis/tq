# orphaned-test

## What it does

Identify unit test files that do not map to an existing source module.

## Why this matters

Orphaned tests often encode stale behavior and add noise in maintenance and review.

## Default severity

`warning`

## Trigger conditions

- A unit test file is discovered.
- No source module resolves for the test's target module name.
- Integration and e2e paths are excluded from this rule.

## Examples

- Source module: `src/tq/engine/old_runner.py (missing)`
- Test module: `tests/tq/engine/test_old_runner.py`

## How to address

- Remove obsolete tests that no longer represent active source modules.
- Restore or create the intended source module when the test is valid.
- Move workflow-level coverage to integration/e2e tests as needed.

## Related configuration and suppression controls

- `--select orphaned-test`
- `--ignore orphaned-test`
- `[tool.tq].select / [tool.tq].ignore`
- `[tool.tq].qualifier_strategy`
- `[tool.tq].allowed_qualifiers`

## Added in

`pre-1.0`

## Behavior changes

None to date.
