# mapping-missing-test

## What it does

Ensure each discovered source module has at least one matching unit test module in the mirrored tests path.

## Why this matters

Missing unit tests reduce discoverability and leave source modules without direct contract coverage.

## Default severity

`error`

## Trigger conditions

- A source module is discovered.
- No matching unit test filename resolves for that module.
- __init__.py handling follows configured ignore policy.

## Examples

- Source module: `src/app/engine/runner.py`
- Test module: `tests/app/engine/test_runner.py (missing)`

## How to address

- Add a mirrored unit test module under `tests/{package}/...`.
- Use `test_<module>.py` or an allowed qualified form.

## Related configuration and suppression controls

- `--select mapping-missing-test`
- `--ignore mapping-missing-test`
- `[tool.tq].select / [tool.tq].ignore`
- `[tool.tq].init_modules`
- `[tool.tq].qualifier_strategy`
- `[tool.tq].allowed_qualifiers`

## Added in

`pre-1.0`

## Behavior changes

None to date.
