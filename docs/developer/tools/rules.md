# tq built-in rules reference

This page is the canonical, stable reference for `tq` built-in rule IDs.

Use these IDs in CLI and configuration selection controls (`--select`, `--ignore`, `[tool.tq].select`, `[tool.tq].ignore`).

## Severity vocabulary

- `error`
- `warning`
- `info`

## Rules

## mapping-missing-test

- Default severity: `error`
- Trigger: a discovered source module has no matching unit test module.
- Example: `src/tq/engine/runner.py` exists but `tests/tq/engine/test_runner.py` does not.
- Typical fix guidance:
  - add at least one matching unit test module under the mirrored `tests/<package>/...` path,
  - ensure the file name follows `test_<module>.py` (or allowed qualifier form).

## structure-mismatch

- Default severity: `warning`
- Trigger: a unit test file does not mirror the expected source-relative path.
- Example: `tests/tq/test_runner.py` exists while source file is `src/tq/engine/runner.py`.
- Typical fix guidance:
  - move the test file to mirror source structure,
  - keep filename aligned with module target (`test_runner.py`).

## test-file-too-large

- Default severity: `warning`
- Trigger: a unit test file exceeds configured max non-blank, non-comment lines.
- Example: `tests/tq/engine/test_runner.py` exceeds `max_test_file_non_blank_lines`.
- Typical fix guidance:
  - split large test modules by concern using qualifiers,
  - extract reusable fixtures into nearby `conftest.py` where appropriate.

## orphaned-test

- Default severity: `warning`
- Trigger: a unit test file does not resolve to an existing source module.
- Example: `tests/tq/engine/test_old_runner.py` exists but `src/tq/engine/old_runner.py` does not.
- Typical fix guidance:
  - remove obsolete tests,
  - or restore/create the intended source module,
  - or move non-unit workflow coverage into integration/e2e scope.
