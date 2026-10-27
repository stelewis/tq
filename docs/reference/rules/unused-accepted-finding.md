# Unused Accepted Finding

Rule ID: `unused-accepted-finding`

## What it does

Report accepted findings whose underlying mapping or size finding no longer exists.

## Why this matters

Unused exceptions obscure which deliberate departures from the rules still need review.

## Default severity

`warning`

## Trigger conditions

- An acceptance is configured for the active target.
- Both this rule and the acceptance's underlying rule are selected and not ignored.
- The underlying finding no longer exists, including after deletion, a matching test is added, or a threshold changes.
- Growth above an accepted size ceiling is a size finding, not an unused acceptance.

## Examples

- Source module: `src/app/engine/runner.py (mapping acceptance configured)`
- Test module: `tests/app/engine/test_runner.py (now present)`

## How to address

- Remove the unused entry from `accepted_findings`.

## Related configuration and suppression controls

- `--select unused-accepted-finding`
- `--ignore unused-accepted-finding`
- `--severity unused-accepted-finding=info`
- `[tool.tq].select / [tool.tq].ignore`
- `[tool.tq].severity_overrides`
- `[[tool.tq.targets.accepted_findings]]`

## Added in

`0.13.0`

## Behavior changes

None to date.
